// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use fragcap_proxy::{
    tls_client_config_with_roots, ApplicationEvent, ApplicationEventKind, ApplicationEventSink,
    DestinationPolicy, EventDisposition, NativeProxyBackend, NativeProxyConfig, StreamingEvent,
};
use http_body_util::{BodyExt, Empty, Full};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

fn config(session: &str) -> NativeProxyConfig {
    NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        8,
        32 * 1024,
        Duration::from_secs(3),
    )
    .unwrap()
    .with_session_id(session)
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hyper_http1_client_and_origin_interoperate_through_native_proxy() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let _ = hyper::server::conn::http1::Builder::new()
            .serve_connection(
                TokioIo::new(tcp),
                service_fn(|request: Request<hyper::body::Incoming>| async move {
                    assert_eq!(request.uri().path(), "/hyper");
                    assert!(request.headers().get("proxy-authorization").is_none());
                    Ok::<_, std::convert::Infallible>(Response::new(Full::new(Bytes::from_static(
                        b"hyper-origin",
                    ))))
                }),
            )
            .await;
    });
    let mut policy = DestinationPolicy::new("127.0.0.1:0".parse().unwrap());
    policy.grant_for_test(origin);
    let mut lease = NativeProxyBackend::new(config("s110-hyper-http1"))
        .with_destination_policy(policy)
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(tcp))
        .await
        .unwrap();
    let driver = tokio::spawn(connection);
    let response = sender
        .send_request(
            Request::builder()
                .uri(format!("http://{origin}/hyper"))
                .header(
                    "proxy-authorization",
                    lease.capability_proof().proxy_authorization().as_str(),
                )
                .body(Empty::<Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "hyper-origin"
    );
    drop(sender);
    driver.await.unwrap().unwrap();
    origin_task.await.unwrap();
    assert!(lease.cleanup(Duration::from_secs(2)).is_clean());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wire_http1_sse_client_and_origin_interoperate_through_native_proxy() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let (mut tcp, _) = listener.accept().await.unwrap();
        let head = read_http_head(&mut tcp).await;
        assert!(head.starts_with(b"GET /events HTTP/1.1\r\n"));
        tcp.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: 18\r\nConnection: close\r\n\r\nid: 1\ndata: tick\n\n")
            .await
            .unwrap();
    });
    let collector = Arc::new(Collector::default());
    let mut policy = DestinationPolicy::new("127.0.0.1:0".parse().unwrap());
    policy.grant_for_test(origin);
    let mut lease = NativeProxyBackend::new(config("s110-wire-sse"))
        .with_destination_policy(policy)
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    tcp.write_all(
        format!(
            "GET http://{origin}/events HTTP/1.1\r\nHost: {origin}\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n",
            lease.capability_proof().proxy_authorization().as_str()
        )
        .as_bytes(),
    )
    .await
    .unwrap();
    let mut response = Vec::new();
    tcp.read_to_end(&mut response).await.unwrap();
    assert!(response.ends_with(b"id: 1\ndata: tick\n\n"));
    origin_task.await.unwrap();
    assert!(lease.cleanup(Duration::from_secs(2)).is_clean());
    assert!(collector.0.lock().unwrap().iter().any(|event| matches!(
        event.kind,
        ApplicationEventKind::Streaming(StreamingEvent::SseEvent(_))
    )));
}

fn tls_origin_config() -> (Arc<rustls::ServerConfig>, CertificateDer<'static>) {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let key = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], key.into())
        .unwrap();
    (Arc::new(config), certificate)
}

async fn read_http_head(stream: &mut (impl AsyncReadExt + Unpin)) -> Vec<u8> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).await.unwrap();
        head.push(byte[0]);
    }
    head
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hyper_https_client_and_origin_interoperate_through_native_proxy() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let (origin_tls, origin_certificate) = tls_origin_config();
    let origin_task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let tls = tokio_rustls::TlsAcceptor::from(origin_tls)
            .accept(tcp)
            .await
            .unwrap();
        let mut builder = hyper::server::conn::http1::Builder::new();
        builder.keep_alive(false);
        let _ = builder
            .serve_connection(
                TokioIo::new(tls),
                service_fn(|request: Request<hyper::body::Incoming>| async move {
                    assert_eq!(request.uri().path(), "/secure-hyper");
                    Ok::<_, std::convert::Infallible>(
                        Response::builder()
                            .header("content-length", "21")
                            .header("connection", "close")
                            .body(Full::new(Bytes::from_static(b"verified-hyper-origin")))
                            .unwrap(),
                    )
                }),
            )
            .await;
    });
    let mut roots = rustls::RootCertStore::empty();
    roots.add(origin_certificate).unwrap();
    let collector = Arc::new(Collector::default());
    let mut policy = DestinationPolicy::new("127.0.0.1:0".parse().unwrap());
    policy.grant_for_test(origin);
    let mut lease = NativeProxyBackend::new(config("s110-hyper-https"))
        .with_destination_policy(policy)
        .with_tls_client_config(tls_client_config_with_roots(roots).unwrap())
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    tcp.write_all(
        format!(
            "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {}\r\n\r\n",
            origin.port(),
            origin.port(),
            lease.capability_proof().proxy_authorization().as_str()
        )
        .as_bytes(),
    )
    .await
    .unwrap();
    assert_eq!(
        read_http_head(&mut tcp).await,
        b"HTTP/1.1 200 Connection Established\r\n\r\n"
    );
    let mut client_roots = rustls::RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(lease.ca_der().to_vec()))
        .unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut client_tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(client_roots)
        .with_no_client_auth();
    client_tls.alpn_protocols = vec![b"http/1.1".to_vec()];
    let tls = tokio_rustls::TlsConnector::from(Arc::new(client_tls))
        .connect(ServerName::try_from("localhost").unwrap().to_owned(), tcp)
        .await
        .unwrap();
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(tls))
        .await
        .unwrap();
    let driver = tokio::spawn(connection);
    let response = sender
        .send_request(
            Request::builder()
                .uri("/secure-hyper")
                .header("host", format!("localhost:{}", origin.port()))
                .header("connection", "close")
                .body(Empty::<Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap_or_else(|error| {
            let events = collector.0.lock().unwrap();
            panic!("Hyper HTTPS response failed: {error:?}; events={events:?}")
        });
    assert_eq!(response.status(), 200);
    drop(sender);
    let _ = driver.await;
    origin_task.await.unwrap();
    assert!(lease.cleanup(Duration::from_secs(2)).is_clean());
}

async fn read_h2_frame(stream: &mut tokio::net::TcpStream, stage: &str) -> (u8, u8, u32, Vec<u8>) {
    let mut head = [0_u8; 9];
    tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut head))
        .await
        .unwrap_or_else(|_| panic!("HTTP/2 frame header timed out at {stage}"))
        .unwrap();
    let length = ((head[0] as usize) << 16) | ((head[1] as usize) << 8) | head[2] as usize;
    let mut payload = vec![0_u8; length];
    tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut payload))
        .await
        .unwrap_or_else(|_| panic!("HTTP/2 frame payload timed out at {stage}"))
        .unwrap();
    (
        head[3],
        head[4],
        u32::from_be_bytes([head[5] & 0x7f, head[6], head[7], head[8]]),
        payload,
    )
}

async fn write_h2_frame(
    stream: &mut tokio::net::TcpStream,
    kind: u8,
    flags: u8,
    stream_id: u32,
    payload: &[u8],
) {
    let length = payload.len() as u32;
    let mut head = [0_u8; 9];
    head[0] = (length >> 16) as u8;
    head[1] = (length >> 8) as u8;
    head[2] = length as u8;
    head[3] = kind;
    head[4] = flags;
    head[5..].copy_from_slice(&stream_id.to_be_bytes());
    stream.write_all(&head).await.unwrap();
    stream.write_all(payload).await.unwrap();
}

fn hpack_literal_name(target: &mut Vec<u8>, name: &[u8], value: &[u8]) {
    assert!(name.len() < 127 && value.len() < 127);
    target.push(0);
    target.push(name.len() as u8);
    target.extend_from_slice(name);
    target.push(value.len() as u8);
    target.extend_from_slice(value);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_http2_client_multiplexes_through_native_proxy() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut connection = h2::server::handshake(tcp).await.unwrap();
        for _ in 0..2 {
            let (request, mut respond) = connection.accept().await.unwrap().unwrap();
            let body = match request.uri().path() {
                "/raw-grpc" => b"\0\0\0\0\x02ok".as_slice(),
                "/raw-grpc-second" => b"\0\0\0\0\x02go".as_slice(),
                value => panic!("unexpected HTTP/2 origin path {value}"),
            };
            let response = Response::builder()
                .status(200)
                .header("content-type", "application/grpc")
                .body(())
                .unwrap();
            let mut send = respond.send_response(response, false).unwrap();
            send.send_data(Bytes::copy_from_slice(body), false).unwrap();
            let mut trailers = hyper::HeaderMap::new();
            trailers.insert("grpc-status", "0".parse().unwrap());
            send.send_trailers(trailers).unwrap();
        }
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while connection.accept().await.is_some() {}
        })
        .await;
        connection.graceful_shutdown();
    });
    let mut policy = DestinationPolicy::new("127.0.0.1:0".parse().unwrap());
    policy.grant_for_test(origin);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config("s110-raw-h2"))
        .with_destination_policy(policy)
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    tcp.write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")
        .await
        .unwrap();
    write_h2_frame(&mut tcp, 4, 0, 0, &[]).await;
    let mut saw_settings = false;
    while !saw_settings {
        let (kind, flags, _, _) = read_h2_frame(&mut tcp, "raw client settings").await;
        if kind == 4 && flags == 0 {
            saw_settings = true;
            write_h2_frame(&mut tcp, 4, 1, 0, &[]).await;
        }
    }
    let mut headers = vec![0x83, 0x86];
    hpack_literal_name(&mut headers, b":path", b"/raw-grpc");
    hpack_literal_name(&mut headers, b":authority", origin.to_string().as_bytes());
    hpack_literal_name(&mut headers, b"content-type", b"application/grpc");
    hpack_literal_name(
        &mut headers,
        b"proxy-authorization",
        lease
            .capability_proof()
            .proxy_authorization()
            .as_str()
            .as_bytes(),
    );
    write_h2_frame(&mut tcp, 1, 5, 1, &headers).await;
    let mut second_headers = vec![0x83, 0x86];
    hpack_literal_name(&mut second_headers, b":path", b"/raw-grpc-second");
    hpack_literal_name(
        &mut second_headers,
        b":authority",
        origin.to_string().as_bytes(),
    );
    hpack_literal_name(&mut second_headers, b"content-type", b"application/grpc");
    hpack_literal_name(
        &mut second_headers,
        b"proxy-authorization",
        lease
            .capability_proof()
            .proxy_authorization()
            .as_str()
            .as_bytes(),
    );
    write_h2_frame(&mut tcp, 1, 5, 3, &second_headers).await;
    let mut ended_streams = BTreeSet::new();
    while ended_streams.len() != 2 {
        let (_, flags, stream, _) = read_h2_frame(&mut tcp, "raw multiplexed response").await;
        if matches!(stream, 1 | 3) && flags & 1 == 1 {
            ended_streams.insert(stream);
        }
    }
    let terminal_streams: BTreeSet<_> = collector
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                ApplicationEventKind::Streaming(StreamingEvent::GrpcTerminal { .. })
            )
        })
        .filter_map(|event| event.stream_id)
        .collect();
    assert_eq!(terminal_streams, BTreeSet::from([1, 3]));
    drop(tcp);
    origin_task.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.http2_streams_completed, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h2_grpc_client_interoperates_with_raw_http2_origin_through_native_proxy() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let (mut tcp, _) = listener.accept().await.unwrap();
        let mut preface = [0_u8; 24];
        tcp.read_exact(&mut preface).await.unwrap();
        assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
        let (kind, flags, _, _) = read_h2_frame(&mut tcp, "raw origin settings").await;
        assert_eq!((kind, flags & 1), (4, 0));
        write_h2_frame(&mut tcp, 4, 0, 0, &[]).await;
        write_h2_frame(&mut tcp, 4, 1, 0, &[]).await;
        loop {
            let (kind, flags, stream, _) = read_h2_frame(&mut tcp, "raw origin request").await;
            if kind == 4 && flags & 1 == 0 {
                write_h2_frame(&mut tcp, 4, 1, 0, &[]).await;
            }
            if kind == 1 && stream == 1 && flags & 1 == 1 {
                break;
            }
        }
        let mut response_headers = vec![0x88];
        hpack_literal_name(&mut response_headers, b"content-type", b"application/grpc");
        write_h2_frame(&mut tcp, 1, 4, 1, &response_headers).await;
        write_h2_frame(&mut tcp, 0, 0, 1, b"\0\0\0\0\x02ok").await;
        let mut trailers = Vec::new();
        hpack_literal_name(&mut trailers, b"grpc-status", b"0");
        write_h2_frame(&mut tcp, 1, 5, 1, &trailers).await;
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let mut head = [0_u8; 9];
                if tcp.read_exact(&mut head).await.is_err() {
                    break;
                }
                let length =
                    ((head[0] as usize) << 16) | ((head[1] as usize) << 8) | head[2] as usize;
                let mut payload = vec![0_u8; length];
                if tcp.read_exact(&mut payload).await.is_err() {
                    break;
                }
            }
        })
        .await;
    });

    let mut policy = DestinationPolicy::new("127.0.0.1:0".parse().unwrap());
    policy.grant_for_test(origin);
    let mut lease = NativeProxyBackend::new(config("s110-raw-h2-origin"))
        .with_destination_policy(policy)
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (mut sender, connection) = h2::client::handshake(tcp).await.unwrap();
    let driver = tokio::spawn(connection);
    let request = Request::builder()
        .method("POST")
        .uri(format!("http://{origin}/raw-origin"))
        .header("content-type", "application/grpc")
        .header(
            "proxy-authorization",
            lease.capability_proof().proxy_authorization().as_str(),
        )
        .body(())
        .unwrap();
    let (response, _) = sender.send_request(request, true).unwrap();
    let mut body = response.await.unwrap().into_body();
    let mut observed = Vec::new();
    while let Some(chunk) = body.data().await {
        let chunk = chunk.unwrap();
        body.flow_control().release_capacity(chunk.len()).unwrap();
        observed.extend_from_slice(&chunk);
    }
    assert_eq!(observed, b"\0\0\0\0\x02ok");
    assert_eq!(body.trailers().await.unwrap().unwrap()["grpc-status"], "0");
    drop(sender);
    driver.abort();
    let _ = driver.await;
    origin_task.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.http2_streams_completed, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h2_sse_and_grpc_peers_interoperate_through_native_proxy() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        for _ in 0..2 {
            let (tcp, _) = listener.accept().await.unwrap();
            let mut connection = h2::server::handshake(tcp).await.unwrap();
            let (request, mut respond) = connection.accept().await.unwrap().unwrap();
            let grpc = request.uri().path() == "/grpc";
            let response = Response::builder()
                .status(200)
                .header(
                    "content-type",
                    if grpc {
                        "application/grpc"
                    } else {
                        "text/event-stream"
                    },
                )
                .body(())
                .unwrap();
            let mut body = respond.send_response(response, false).unwrap();
            if grpc {
                body.send_data(Bytes::from_static(b"\0\0\0\0\x02ok"), false)
                    .unwrap();
                let mut trailers = hyper::HeaderMap::new();
                trailers.insert("grpc-status", "0".parse().unwrap());
                body.send_trailers(trailers).unwrap();
            } else {
                body.send_data(Bytes::from_static(b"id: 1\ndata: tick\n\n"), true)
                    .unwrap();
            }
            let _ = tokio::time::timeout(Duration::from_millis(100), connection.accept()).await;
            connection.graceful_shutdown();
        }
    });
    let collector = Arc::new(Collector::default());
    let mut policy = DestinationPolicy::new("127.0.0.1:0".parse().unwrap());
    policy.grant_for_test(origin);
    let mut lease = NativeProxyBackend::new(config("s110-h2-streaming"))
        .with_destination_policy(policy)
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    for (path, content_type) in [("/sse", None), ("/grpc", Some("application/grpc"))] {
        let tcp = tokio::net::TcpStream::connect(lease.endpoint())
            .await
            .unwrap();
        let (mut sender, connection) = h2::client::handshake(tcp).await.unwrap();
        let driver = tokio::spawn(connection);
        let mut request = Request::builder()
            .method(if content_type.is_some() {
                "POST"
            } else {
                "GET"
            })
            .uri(format!("http://{origin}{path}"))
            .header(
                "proxy-authorization",
                lease.capability_proof().proxy_authorization().as_str(),
            );
        if let Some(value) = content_type {
            request = request.header("content-type", value);
        }
        let (response, _) = sender
            .send_request(request.body(()).unwrap(), true)
            .unwrap();
        let mut body = response
            .await
            .unwrap_or_else(|error| {
                let events = collector.0.lock().unwrap();
                panic!("{path} response failed: {error:?}; events={events:?}")
            })
            .into_body();
        tokio::time::timeout(Duration::from_secs(2), async {
            while body.data().await.is_some() {}
            let _ = body.trailers().await;
        })
        .await
        .unwrap();
        drop(sender);
        driver.abort();
        let _ = driver.await;
    }
    origin_task.await.unwrap();
    assert!(lease.cleanup(Duration::from_secs(2)).is_clean());
    let events = collector.0.lock().unwrap();
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ApplicationEventKind::Streaming(StreamingEvent::SseEvent(_))
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ApplicationEventKind::Streaming(StreamingEvent::GrpcMessage(_))
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ApplicationEventKind::Streaming(StreamingEvent::GrpcTerminal { .. })
    )));
}
