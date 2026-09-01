// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use flate2::{Compress, Compression, FlushCompress};
use fragcap_proxy::{
    tls_client_config_with_roots, ApplicationEvent, ApplicationEventKind, ApplicationEventSink,
    DestinationPolicy, EventDisposition, NativeProxyBackend, NativeProxyConfig, ProtocolLimits,
};
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

async fn read_head(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).await.unwrap();
        head.push(byte[0]);
    }
    head
}

fn compressed_websocket_frame(payload: &[u8], masking_key: Option<[u8; 4]>) -> Bytes {
    let mut compressor = Compress::new(Compression::fast(), false);
    let mut compressed = Vec::with_capacity(128);
    compressor
        .compress_vec(payload, &mut compressed, FlushCompress::Sync)
        .unwrap();
    assert!(compressed.ends_with(&[0, 0, 0xff, 0xff]));
    compressed.truncate(compressed.len() - 4);
    let mut frame = vec![
        0xc1,
        compressed.len() as u8 | if masking_key.is_some() { 0x80 } else { 0 },
    ];
    if let Some(key) = masking_key {
        frame.extend_from_slice(&key);
        frame.extend(
            compressed
                .iter()
                .enumerate()
                .map(|(index, byte)| byte ^ key[index % 4]),
        );
    } else {
        frame.extend_from_slice(&compressed);
    }
    Bytes::from(frame)
}

fn h2_origin_config() -> (Arc<rustls::ServerConfig>, CertificateDer<'static>) {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], private.into())
        .unwrap();
    config.alpn_protocols = vec![b"h2".to_vec()];
    (Arc::new(config), certificate)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multiplexes_thirty_two_streams_with_distinct_terminal_evidence() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let (origin_config, origin_certificate) = h2_origin_config();
    let origin_task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let tls = tokio_rustls::TlsAcceptor::from(origin_config)
            .accept(tcp)
            .await
            .unwrap();
        let mut connection = h2::server::handshake(tls).await.unwrap();
        let mut served = 0;
        let mut responses = tokio::task::JoinSet::new();
        while let Some(stream) = connection.accept().await {
            let (request, mut response) = stream.unwrap();
            let index: u64 = request
                .uri()
                .path()
                .trim_start_matches('/')
                .parse()
                .unwrap();
            responses.spawn(async move {
                tokio::time::sleep(Duration::from_millis(32 - index)).await;
                if index == 7 {
                    response.send_reset(h2::Reason::CANCEL);
                    return;
                }
                let mut body = response
                    .send_response(
                        hyper::Response::builder()
                            .status(200)
                            .header("x-fragcap-stream", index.to_string())
                            .body(())
                            .unwrap(),
                        false,
                    )
                    .unwrap();
                if index == 10 {
                    tokio::time::sleep(Duration::from_millis(120)).await;
                    body.send_data(Bytes::from_static(b"1"), false).unwrap();
                    tokio::time::sleep(Duration::from_millis(120)).await;
                    body.send_data(Bytes::from_static(b"0"), false).unwrap();
                } else {
                    body.send_data(Bytes::from(index.to_string()), false)
                        .unwrap();
                }
                let mut trailers = hyper::HeaderMap::new();
                trailers.insert("x-finished", index.to_string().parse().unwrap());
                body.send_trailers(trailers).unwrap();
            });
            served += 1;
            if served == 32 {
                break;
            }
        }
        while !responses.is_empty() {
            tokio::select! {
                result = responses.join_next() => result.unwrap().unwrap(),
                extra = connection.accept() => {
                    assert!(extra.is_none(), "origin received an unexpected stream");
                }
            }
        }
        let _ = tokio::time::timeout(Duration::from_millis(100), connection.accept()).await;
        connection.graceful_shutdown();
        served
    });

    let limits = ProtocolLimits {
        idle_timeout: Duration::from_millis(200),
        ..ProtocolLimits::default()
    };
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        4,
        16 * 1024,
        Duration::from_secs(3),
    )
    .unwrap()
    .with_session_id("s105-http2")
    .unwrap()
    .with_protocol_limits(limits);
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let mut origin_roots = rustls::RootCertStore::empty();
    origin_roots.add(origin_certificate).unwrap();
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_tls_client_config(tls_client_config_with_roots(origin_roots).unwrap())
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();

    let mut tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let authorization = lease.capability_proof().proxy_authorization();
    tcp.write_all(
        format!(
            "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {}\r\n\r\n",
            origin.port(),
            origin.port(),
            authorization.as_str()
        )
        .as_bytes(),
    )
    .await
    .unwrap();
    assert_eq!(
        read_head(&mut tcp).await,
        b"HTTP/1.1 200 Connection Established\r\n\r\n"
    );

    let mut client_roots = rustls::RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(lease.ca_der().to_vec()))
        .unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut client_config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(client_roots)
        .with_no_client_auth();
    client_config.alpn_protocols = vec![b"h2".to_vec()];
    let tls = tokio_rustls::TlsConnector::from(Arc::new(client_config))
        .connect(ServerName::try_from("localhost").unwrap().to_owned(), tcp)
        .await
        .unwrap();
    let (sender, connection) = h2::client::handshake(tls).await.unwrap();
    let driver = tokio::spawn(connection);
    let mut responses = Vec::new();
    let mut ready = sender.clone().ready().await.unwrap();
    let mismatched = hyper::Request::builder()
        .method("GET")
        .uri(format!("https://localhost:{}/outside", origin.port() + 1))
        .body(())
        .unwrap();
    let (mismatched, _) = ready.send_request(mismatched, true).unwrap();
    let mismatch = tokio::time::timeout(Duration::from_secs(3), mismatched)
        .await
        .unwrap()
        .unwrap_err();
    assert!(mismatch.is_reset());
    assert_eq!(mismatch.reason(), Some(h2::Reason::REFUSED_STREAM));
    for index in 0..32 {
        let mut ready = sender.clone().ready().await.unwrap();
        let request = hyper::Request::builder()
            .method("GET")
            .uri(format!("https://localhost:{}/{index}", origin.port()))
            .body(())
            .unwrap();
        let (response, _) = ready.send_request(request, true).unwrap();
        responses.push((index, response));
    }
    for (index, response) in responses {
        let response = tokio::time::timeout(Duration::from_secs(5), response)
            .await
            .expect("response deadline");
        if index == 7 {
            assert!(response.unwrap_err().is_reset());
            continue;
        }
        let response = response.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["x-fragcap-stream"], index.to_string());
        let mut body = response.into_body();
        let mut data = Vec::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            body.flow_control().release_capacity(chunk.len()).unwrap();
            data.extend_from_slice(&chunk);
        }
        assert_eq!(data, index.to_string().as_bytes());
        let trailers = body.trailers().await.unwrap().unwrap();
        assert_eq!(trailers["x-finished"], index.to_string());
    }
    drop(sender);
    driver.abort();
    let _ = driver.await;
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(origin_task.await.unwrap(), 32);
    assert_eq!(report.observation.protocol.requests, 32);
    assert_eq!(report.observation.protocol.responses, 31);
    assert_eq!(report.observation.protocol.http2_streams_reset, 1);
    assert_eq!(report.observation.protocol.parse_refused, 1);
    let events = collector.0.lock().unwrap();
    let opened = events
        .iter()
        .filter(|event| matches!(event.kind, ApplicationEventKind::HttpStreamOpen))
        .count();
    let completed = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                ApplicationEventKind::HttpStreamTerminal(fragcap_proxy::StreamTerminal::Complete)
            )
        })
        .count();
    let reset = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                ApplicationEventKind::HttpStreamTerminal(fragcap_proxy::StreamTerminal::Reset)
            )
        })
        .count();
    let trailers = events
        .iter()
        .filter(|event| {
            matches!(
                &event.kind,
                ApplicationEventKind::Metadata(block)
                    if block.kind == fragcap_proxy::MetadataKind::Trailers
            )
        })
        .count();
    let refused = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                ApplicationEventKind::HttpStreamTerminal(fragcap_proxy::StreamTerminal::Refused)
            )
        })
        .count();
    assert_eq!((opened, completed, reset, trailers), (32, 31, 1, 31));
    assert_eq!(refused, 1);
    let stream_ids: std::collections::BTreeSet<_> =
        events.iter().filter_map(|event| event.stream_id).collect();
    assert_eq!(stream_ids.len(), 33);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn explicitly_routed_cleartext_http2_is_authenticated_and_authority_bound() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut connection = h2::server::handshake(tcp).await.unwrap();
        let (request, mut response) = connection.accept().await.unwrap().unwrap();
        assert_eq!(request.uri().path(), "/h2c");
        assert!(request
            .headers()
            .get(hyper::header::PROXY_AUTHORIZATION)
            .is_none());
        response
            .send_response(
                hyper::Response::builder().status(204).body(()).unwrap(),
                true,
            )
            .unwrap();
        let second = tokio::time::timeout(Duration::from_millis(100), connection.accept()).await;
        assert!(!matches!(second, Ok(Some(Ok(_)))));
        connection.graceful_shutdown();
    });

    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        16 * 1024,
        Duration::from_secs(3),
    )
    .unwrap()
    .with_session_id("s105-h2c")
    .unwrap();
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (sender, connection) = h2::client::handshake(tcp).await.unwrap();
    let driver = tokio::spawn(connection);
    let mut sender = sender.ready().await.unwrap();
    let request = hyper::Request::builder()
        .method("GET")
        .uri(format!("http://{origin}/h2c"))
        .header(
            hyper::header::PROXY_AUTHORIZATION,
            lease.capability_proof().proxy_authorization().as_str(),
        )
        .body(())
        .unwrap();
    let (response, _) = sender.send_request(request, true).unwrap();
    let mut sender = sender.ready().await.unwrap();
    let mismatched = hyper::Request::builder()
        .method("GET")
        .uri(format!("http://127.0.0.1:{}/outside", origin.port() + 1))
        .body(())
        .unwrap();
    let (mismatched, _) = sender.send_request(mismatched, true).unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(3), response)
            .await
            .unwrap()
            .unwrap()
            .status(),
        204
    );
    let mismatch = tokio::time::timeout(Duration::from_secs(3), mismatched)
        .await
        .unwrap()
        .unwrap_err();
    assert!(mismatch.is_reset());
    assert_eq!(mismatch.reason(), Some(h2::Reason::REFUSED_STREAM));
    drop(sender);
    driver.abort();
    let _ = driver.await;
    origin_task.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.http2_streams, 1);
    assert_eq!(report.observation.protocol.http2_streams_completed, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn extended_connect_websocket_is_forwarded_and_observed_as_frames() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = listener.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut builder = h2::server::Builder::new();
        builder.enable_connect_protocol();
        let mut connection = builder.handshake(tcp).await.unwrap();
        let (request, mut respond) = connection.accept().await.unwrap().unwrap();
        assert_eq!(request.method(), hyper::Method::CONNECT);
        assert_eq!(
            request
                .extensions()
                .get::<h2::ext::Protocol>()
                .unwrap()
                .as_str(),
            "websocket"
        );
        let handler = tokio::spawn(async move {
            let mut body = respond
                .send_response(
                    hyper::Response::builder()
                        .status(200)
                        .header(
                            "sec-websocket-extensions",
                            "permessage-deflate; client_no_context_takeover; server_no_context_takeover",
                        )
                        .body(())
                        .unwrap(),
                    false,
                )
                .unwrap();
            let mut request_body = request.into_body();
            let frame = request_body.data().await.unwrap().unwrap();
            request_body
                .flow_control()
                .release_capacity(frame.len())
                .unwrap();
            assert_eq!(frame, compressed_websocket_frame(b"hi", Some([1, 2, 3, 4])));
            body.send_data(compressed_websocket_frame(b"ok", None), true)
                .unwrap();
        });
        if let Some(stream) = connection.accept().await {
            assert!(stream.is_err(), "a second stream was not expected");
        }
        handler.await.unwrap();
        connection.graceful_shutdown();
    });

    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        16 * 1024,
        Duration::from_secs(3),
    )
    .unwrap()
    .with_session_id("s106-rfc8441")
    .unwrap();
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (sender, connection) = h2::client::handshake(tcp).await.unwrap();
    let driver = tokio::spawn(connection);
    tokio::time::timeout(Duration::from_secs(2), async {
        while !sender.is_extended_connect_protocol_enabled() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let mut request = hyper::Request::builder()
        .method("CONNECT")
        .uri(format!("http://{origin}/socket"))
        .header(
            hyper::header::PROXY_AUTHORIZATION,
            lease.capability_proof().proxy_authorization().as_str(),
        )
        .header(
            "sec-websocket-extensions",
            "permessage-deflate; client_no_context_takeover; server_no_context_takeover",
        )
        .body(())
        .unwrap();
    request
        .extensions_mut()
        .insert(h2::ext::Protocol::from_static("websocket"));
    let mut ready = sender.ready().await.unwrap();
    let (response, mut body) = ready.send_request(request, false).unwrap();
    let response = response.await.unwrap_or_else(|error| {
        let events = collector.0.lock().unwrap();
        panic!("extended CONNECT reset: {error:?}; events={events:?}");
    });
    assert_eq!(response.status(), 200);
    body.send_data(compressed_websocket_frame(b"hi", Some([1, 2, 3, 4])), true)
        .unwrap();
    let mut response_body = response.into_body();
    let response_frame = match response_body.data().await {
        Some(Ok(bytes)) => bytes,
        value => {
            let events = collector.0.lock().unwrap();
            panic!("extended CONNECT body failed: {value:?}; events={events:?}");
        }
    };
    assert_eq!(response_frame, compressed_websocket_frame(b"ok", None));
    drop(response_body);
    drop(body);
    drop(ready);
    driver.abort();
    let _ = driver.await;
    origin_task.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    let events = collector.0.lock().unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.kind,
                ApplicationEventKind::Streaming(fragcap_proxy::StreamingEvent::WebSocketFrame(_))
            ))
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                &event.kind,
                ApplicationEventKind::Streaming(
                    fragcap_proxy::StreamingEvent::WebSocketMessage(message)
                ) if message.compressed && matches!(message.payload.as_slice(), b"hi" | b"ok")
            ))
            .count(),
        2
    );
}
