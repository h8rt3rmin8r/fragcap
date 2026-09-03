// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use hyper::body::Bytes;
use hyper::{Request, Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use super::ProtocolFamily;

const IPV4_BIND: &str = "127.0.0.1:0";
const IPV6_BIND: &str = "[::1]:0";

pub async fn protocol_round_trip(protocol: ProtocolFamily, payload: &[u8]) -> Vec<u8> {
    match protocol {
        ProtocolFamily::Http1 => http1_round_trip(payload).await,
        ProtocolFamily::Https => tls_round_trip(payload, TlsProtocol::Https, IPV4_BIND).await,
        ProtocolFamily::Http2 => h2_round_trip(payload, false).await,
        ProtocolFamily::StreamingHttp => streaming_http_round_trip(payload).await,
        ProtocolFamily::WebSocket => websocket_round_trip(payload).await,
        ProtocolFamily::Grpc => h2_round_trip(payload, true).await,
        ProtocolFamily::RawTcp => raw_tcp_round_trip(payload).await,
        ProtocolFamily::NonHttpTls => tls_round_trip(payload, TlsProtocol::Binary, IPV4_BIND).await,
        ProtocolFamily::Socks => socks_round_trip(payload).await,
        ProtocolFamily::Udp => udp_round_trip(payload).await,
        ProtocolFamily::Quic => quic_round_trip(payload).await,
    }
}

pub async fn ipv6_required_round_trip(protocol: ProtocolFamily, payload: &[u8]) -> Vec<u8> {
    match protocol {
        ProtocolFamily::Http1 => http1_round_trip_on(payload, IPV6_BIND).await,
        ProtocolFamily::Https => tls_round_trip(payload, TlsProtocol::Https, IPV6_BIND).await,
        ProtocolFamily::Socks => socks_round_trip_on(payload, IPV6_BIND).await,
        ProtocolFamily::RawTcp => raw_tcp_round_trip_on(payload, IPV6_BIND).await,
        ProtocolFamily::Udp => udp_round_trip_on(payload, IPV6_BIND).await,
        ProtocolFamily::Quic => quic_round_trip_on(payload, IPV6_BIND).await,
        _ => panic!("protocol is not an S119 required IPv6 row"),
    }
}

async fn tcp_exchange(
    bind: &'static str,
    request: Vec<u8>,
    handler: impl FnOnce(Vec<u8>) -> Vec<u8> + Send + 'static,
) -> Vec<u8> {
    let listener = TcpListener::bind(bind).await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut received = Vec::new();
        stream.read_to_end(&mut received).await.unwrap();
        let response = handler(received);
        stream.write_all(&response).await.unwrap();
        response
    });
    let mut client = TcpStream::connect(address).await.unwrap();
    client.write_all(&request).await.unwrap();
    client.shutdown().await.unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).await.unwrap();
    assert_eq!(server.await.unwrap(), response);
    response
}

async fn http1_round_trip(payload: &[u8]) -> Vec<u8> {
    http1_round_trip_on(payload, IPV4_BIND).await
}

async fn http1_round_trip_on(payload: &[u8], bind: &'static str) -> Vec<u8> {
    let mut request = format!(
        "POST /capture HTTP/1.1\r\nHost: fragcap.test\r\nContent-Length: {}\r\n\r\n",
        payload.len()
    )
    .into_bytes();
    request.extend_from_slice(payload);
    let expected = payload.to_vec();
    let response = tcp_exchange(bind, request, move |received| {
        let (head, body) = split_http(&received);
        assert!(head.starts_with(b"POST /capture HTTP/1.1\r\n"));
        assert_eq!(body, expected);
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body);
        response
    })
    .await;
    let (head, body) = split_http(&response);
    assert!(head.starts_with(b"HTTP/1.1 200 OK\r\n"));
    body.to_vec()
}

async fn streaming_http_round_trip(payload: &[u8]) -> Vec<u8> {
    let split = payload.len() / 2;
    let request = chunked_request(&payload[..split], &payload[split..]);
    let expected = payload.to_vec();
    let response = tcp_exchange(IPV4_BIND, request, move |received| {
        let (head, body) = split_http(&received);
        assert!(head.starts_with(b"POST /stream HTTP/1.1\r\n"));
        assert_eq!(decode_chunks(body), expected);
        let mut response =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
        append_chunk(&mut response, &expected);
        response.extend_from_slice(b"0\r\n\r\n");
        response
    })
    .await;
    let (_, body) = split_http(&response);
    decode_chunks(body)
}

fn chunked_request(first: &[u8], second: &[u8]) -> Vec<u8> {
    let mut request =
        b"POST /stream HTTP/1.1\r\nHost: fragcap.test\r\nTransfer-Encoding: chunked\r\n\r\n"
            .to_vec();
    append_chunk(&mut request, first);
    append_chunk(&mut request, second);
    request.extend_from_slice(b"0\r\n\r\n");
    request
}

fn append_chunk(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(format!("{:X}\r\n", value.len()).as_bytes());
    target.extend_from_slice(value);
    target.extend_from_slice(b"\r\n");
}

fn decode_chunks(mut bytes: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::new();
    loop {
        let end = bytes.windows(2).position(|item| item == b"\r\n").unwrap();
        let length =
            usize::from_str_radix(std::str::from_utf8(&bytes[..end]).unwrap(), 16).unwrap();
        bytes = &bytes[end + 2..];
        if length == 0 {
            break;
        }
        decoded.extend_from_slice(&bytes[..length]);
        bytes = &bytes[length + 2..];
    }
    decoded
}

async fn websocket_round_trip(payload: &[u8]) -> Vec<u8> {
    assert!(payload.len() < 126);
    let mut request = b"GET /socket HTTP/1.1\r\nHost: fragcap.test\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n".to_vec();
    let mask = [0x12, 0x34, 0x56, 0x78];
    request.extend_from_slice(&[0x82, 0x80 | payload.len() as u8]);
    request.extend_from_slice(&mask);
    request.extend(
        payload
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ mask[index % 4]),
    );
    let expected = payload.to_vec();
    let response = tcp_exchange(IPV4_BIND, request, move |received| {
        let boundary = received.windows(4).position(|item| item == b"\r\n\r\n").unwrap() + 4;
        assert!(received.starts_with(b"GET /socket HTTP/1.1\r\n"));
        let frame = &received[boundary..];
        assert_eq!(frame[0], 0x82);
        assert_ne!(frame[1] & 0x80, 0);
        let length = (frame[1] & 0x7f) as usize;
        let decoded: Vec<_> = frame[6..6 + length]
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ frame[2 + index % 4])
            .collect();
        assert_eq!(decoded, expected);
        let mut response = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n".to_vec();
        response.extend_from_slice(&[0x82, decoded.len() as u8]);
        response.extend_from_slice(&decoded);
        response
    })
    .await;
    let boundary = response
        .windows(4)
        .position(|item| item == b"\r\n\r\n")
        .unwrap()
        + 4;
    assert!(response.starts_with(b"HTTP/1.1 101 Switching Protocols\r\n"));
    let frame = &response[boundary..];
    frame[2..2 + (frame[1] & 0x7f) as usize].to_vec()
}

async fn raw_tcp_round_trip(payload: &[u8]) -> Vec<u8> {
    raw_tcp_round_trip_on(payload, IPV4_BIND).await
}

async fn raw_tcp_round_trip_on(payload: &[u8], bind: &'static str) -> Vec<u8> {
    let mut request = (payload.len() as u32).to_be_bytes().to_vec();
    request.extend_from_slice(payload);
    let expected = payload.to_vec();
    let response = tcp_exchange(bind, request, move |received| {
        let length = u32::from_be_bytes(received[..4].try_into().unwrap()) as usize;
        assert_eq!(&received[4..4 + length], expected);
        let mut response = (length as u32).to_be_bytes().to_vec();
        response.extend_from_slice(&expected);
        response
    })
    .await;
    let length = u32::from_be_bytes(response[..4].try_into().unwrap()) as usize;
    response[4..4 + length].to_vec()
}

async fn socks_round_trip(payload: &[u8]) -> Vec<u8> {
    socks_round_trip_on(payload, IPV4_BIND).await
}

async fn socks_round_trip_on(payload: &[u8], bind: &'static str) -> Vec<u8> {
    let destination = if bind == IPV6_BIND {
        let mut value = vec![5, 1, 0, 4];
        value.extend_from_slice(&std::net::Ipv6Addr::LOCALHOST.octets());
        value.extend_from_slice(&80_u16.to_be_bytes());
        value
    } else {
        vec![5, 1, 0, 1, 127, 0, 0, 1, 0, 80]
    };
    let mut request = vec![5, 1, 0];
    request.extend_from_slice(&destination);
    request.extend_from_slice(payload);
    let expected = payload.to_vec();
    let expected_destination = destination.clone();
    let response = tcp_exchange(bind, request, move |received| {
        assert_eq!(&received[..3], &[5, 1, 0]);
        assert_eq!(
            &received[3..3 + expected_destination.len()],
            expected_destination
        );
        assert_eq!(&received[3 + expected_destination.len()..], expected);
        let mut response = vec![5, 0];
        response.extend_from_slice(&expected_destination);
        response.extend_from_slice(&expected);
        response
    })
    .await;
    assert_eq!(&response[..2], &[5, 0]);
    assert_eq!(&response[2..2 + destination.len()], destination);
    response[2 + destination.len()..].to_vec()
}

#[derive(Clone, Copy)]
enum TlsProtocol {
    Https,
    Binary,
}

async fn tls_round_trip(payload: &[u8], protocol: TlsProtocol, bind: &'static str) -> Vec<u8> {
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};

    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private_key = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], private_key.into())
        .unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let client_config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let listener = TcpListener::bind(bind).await.unwrap();
    let address = listener.local_addr().unwrap();
    let expected = payload.to_vec();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = tokio_rustls::TlsAcceptor::from(Arc::new(server_config))
            .accept(stream)
            .await
            .unwrap();
        let mut received = Vec::new();
        stream.read_to_end(&mut received).await.unwrap();
        let response = match protocol {
            TlsProtocol::Https => {
                let (head, body) = split_http(&received);
                assert!(head.starts_with(b"POST /secure HTTP/1.1\r\n"));
                assert_eq!(body, expected);
                let mut response =
                    format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len())
                        .into_bytes();
                response.extend_from_slice(body);
                response
            }
            TlsProtocol::Binary => {
                assert_eq!(received, expected);
                expected
            }
        };
        stream.write_all(&response).await.unwrap();
        stream.shutdown().await.unwrap();
    });
    let stream = TcpStream::connect(address).await.unwrap();
    let mut stream = tokio_rustls::TlsConnector::from(Arc::new(client_config))
        .connect(ServerName::try_from("localhost").unwrap(), stream)
        .await
        .unwrap();
    if matches!(protocol, TlsProtocol::Https) {
        stream
            .write_all(
                format!(
                    "POST /secure HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n",
                    payload.len()
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    }
    stream.write_all(payload).await.unwrap();
    stream.shutdown().await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    server.await.unwrap();
    match protocol {
        TlsProtocol::Https => split_http(&response).1.to_vec(),
        TlsProtocol::Binary => response,
    }
}

async fn h2_round_trip(payload: &[u8], grpc: bool) -> Vec<u8> {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let expected = if grpc {
        grpc_frame(payload)
    } else {
        payload.to_vec()
    };
    let server_expected = expected.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut connection = h2::server::handshake(stream).await.unwrap();
        let (request, mut respond) = connection.accept().await.unwrap().unwrap();
        assert_eq!(
            request.uri().path(),
            if grpc { "/fragcap.Lab/Echo" } else { "/h2" }
        );
        if grpc {
            assert_eq!(request.headers()["content-type"], "application/grpc");
        }
        let mut body = request.into_body();
        let mut received = Vec::new();
        while let Some(chunk) = body.data().await {
            received.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(received, server_expected);
        let response = Response::builder().status(200).body(()).unwrap();
        let mut send = respond.send_response(response, false).unwrap();
        send.send_data(Bytes::from(received), true).unwrap();
        drop(send);
        connection.graceful_shutdown();
        while connection.accept().await.is_some() {}
    });
    let stream = TcpStream::connect(address).await.unwrap();
    let (mut send_request, connection) = h2::client::handshake(stream).await.unwrap();
    let client_connection = tokio::spawn(connection);
    let uri = if grpc { "/fragcap.Lab/Echo" } else { "/h2" };
    let mut builder = Request::builder().method("POST").uri(uri);
    if grpc {
        builder = builder.header("content-type", "application/grpc");
    }
    let (response, mut send) = send_request
        .send_request(builder.body(()).unwrap(), false)
        .unwrap();
    send.send_data(Bytes::from(expected), true).unwrap();
    let mut body = response.await.unwrap().into_body();
    let mut received = Vec::new();
    while let Some(chunk) = body.data().await {
        received.extend_from_slice(&chunk.unwrap());
    }
    server.await.unwrap();
    client_connection.await.unwrap().unwrap();
    if grpc {
        decode_grpc_frame(&received)
    } else {
        received
    }
}

fn grpc_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = vec![0];
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn decode_grpc_frame(frame: &[u8]) -> Vec<u8> {
    assert_eq!(frame[0], 0);
    let length = u32::from_be_bytes(frame[1..5].try_into().unwrap()) as usize;
    frame[5..5 + length].to_vec()
}

fn split_http(bytes: &[u8]) -> (&[u8], &[u8]) {
    let boundary = bytes
        .windows(4)
        .position(|item| item == b"\r\n\r\n")
        .unwrap()
        + 4;
    (&bytes[..boundary], &bytes[boundary..])
}

pub async fn udp_round_trip(payload: &[u8]) -> Vec<u8> {
    udp_round_trip_on(payload, IPV4_BIND).await
}

async fn udp_round_trip_on(payload: &[u8], bind: &'static str) -> Vec<u8> {
    let server = UdpSocket::bind(bind).await.unwrap();
    let client = UdpSocket::bind(bind).await.unwrap();
    client.connect(server.local_addr().unwrap()).await.unwrap();
    server.connect(client.local_addr().unwrap()).await.unwrap();
    client.send(payload).await.unwrap();
    let mut bytes = [0_u8; 256];
    let length = server.recv(&mut bytes).await.unwrap();
    server.send(&bytes[..length]).await.unwrap();
    let length = client.recv(&mut bytes).await.unwrap();
    bytes[..length].to_vec()
}

pub async fn quic_round_trip(payload: &[u8]) -> Vec<u8> {
    quic_round_trip_on(payload, IPV4_BIND).await
}

async fn quic_round_trip_on(payload: &[u8], bind: &'static str) -> Vec<u8> {
    use quinn::{ClientConfig, Endpoint, ServerConfig};
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private_key = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let server_config =
        ServerConfig::with_single_cert(vec![certificate.clone()], private_key.into()).unwrap();
    let server = Endpoint::server(server_config, bind.parse().unwrap()).unwrap();
    let server_address = server.local_addr().unwrap();
    let server_task = tokio::spawn(async move {
        let incoming = server.accept().await.unwrap();
        let connection = incoming.await.unwrap();
        let (mut send, mut receive) = connection.accept_bi().await.unwrap();
        let bytes = receive.read_to_end(256).await.unwrap();
        send.write_all(&bytes).await.unwrap();
        send.finish().unwrap();
        connection.closed().await;
        server.wait_idle().await;
        bytes
    });

    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let mut client = if bind == IPV6_BIND {
        let socket = std::net::UdpSocket::bind(bind).unwrap();
        Endpoint::new(
            quinn::EndpointConfig::default(),
            None,
            socket,
            quinn::default_runtime().unwrap(),
        )
        .unwrap()
    } else {
        Endpoint::client(bind.parse().unwrap()).unwrap()
    };
    client
        .set_default_client_config(ClientConfig::with_root_certificates(Arc::new(roots)).unwrap());
    let connection = client
        .connect(server_address, "localhost")
        .unwrap()
        .await
        .unwrap();
    let (mut send, mut receive) = connection.open_bi().await.unwrap();
    send.write_all(payload).await.unwrap();
    send.finish().unwrap();
    let echoed = receive.read_to_end(256).await.unwrap();
    connection.close(0_u32.into(), b"complete");
    client.wait_idle().await;
    assert_eq!(server_task.await.unwrap(), payload);
    echoed
}
