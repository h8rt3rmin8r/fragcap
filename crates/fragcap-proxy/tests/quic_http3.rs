// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytes::{Buf, Bytes};
use fragcap_proxy::{
    build_quic_client_config, build_quic_server_config, tls_client_config_with_roots,
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, DestinationAuthority,
    DestinationPolicy, EventDisposition, LeafCache, NativeProxyBackend, NativeProxyConfig,
    ProtocolLimits, QuicInspectionPlan, QuicRefusalCode, SessionCertificateAuthority,
};
use h3_quinn::Connection;
use hyper::header::HeaderValue;
use hyper::{HeaderMap, Request, Response};
use quinn::Endpoint;
use rustls::RootCertStore;

#[derive(Default)]
struct Collector(std::sync::Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

fn client_tls(roots: RootCertStore) -> Arc<rustls::ClientConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    Arc::new(
        rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    )
}

async fn h3_round_trip(lineage: u64) {
    let authority = DestinationAuthority::parse("localhost:443").unwrap();
    let ca = SessionCertificateAuthority::generate(
        lineage,
        SystemTime::now(),
        Duration::from_secs(3600),
    )
    .unwrap();
    let mut cache = LeafCache::new(
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(128 * 1024).unwrap(),
        Duration::from_secs(1800),
        lineage,
    )
    .unwrap();
    let limits = ProtocolLimits {
        max_concurrent_streams: 4,
        max_socks_udp_datagram_bytes: 4096,
        idle_timeout: Duration::from_secs(5),
        ..ProtocolLimits::default()
    };
    let server_config =
        build_quic_server_config(&authority, &ca, &mut cache, None, &limits).unwrap();
    let server = Endpoint::server(server_config, "127.0.0.1:0".parse().unwrap()).unwrap();
    let server_address = server.local_addr().unwrap();
    let server_task = tokio::spawn(async move {
        let connection = server.accept().await.unwrap().await.unwrap();
        let negotiated_h3 = {
            let handshake = connection.handshake_data().unwrap();
            let handshake = handshake
                .downcast_ref::<quinn::crypto::rustls::HandshakeData>()
                .unwrap();
            handshake.protocol.as_deref() == Some(b"h3".as_slice())
        };
        assert!(negotiated_h3);
        let mut h3 = h3::server::Connection::new(Connection::new(connection))
            .await
            .unwrap();
        let resolver = h3.accept().await.unwrap().unwrap();
        let (request, mut stream) = resolver.resolve_request().await.unwrap();
        assert_eq!(request.method(), "POST");
        assert_eq!(request.uri().path(), "/observe");
        let mut body = Vec::new();
        while let Some(mut chunk) = stream.recv_data().await.unwrap() {
            while chunk.has_remaining() {
                body.extend_from_slice(chunk.chunk());
                let length = chunk.chunk().len();
                chunk.advance(length);
            }
        }
        stream
            .send_response(Response::builder().status(201).body(()).unwrap())
            .await
            .unwrap();
        stream.send_data(Bytes::from(body.clone())).await.unwrap();
        stream.finish().await.unwrap();
        let _ = h3.accept().await;
        server.wait_idle().await;
        body
    });

    let mut roots = RootCertStore::empty();
    roots.add(ca.der().clone()).unwrap();
    let client_config = build_quic_client_config(&client_tls(roots), &limits).unwrap();
    let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    client.set_default_client_config(client_config);
    let connection = client
        .connect(server_address, "localhost")
        .unwrap()
        .await
        .unwrap();
    let (mut driver, mut sender) = h3::client::new(Connection::new(connection.clone()))
        .await
        .unwrap();
    let driver_task =
        tokio::spawn(async move { std::future::poll_fn(|cx| driver.poll_close(cx)).await });
    let payload = format!("lineage-{lineage}");
    let mut stream = sender
        .send_request(Request::post("https://localhost/observe").body(()).unwrap())
        .await
        .unwrap();
    stream
        .send_data(Bytes::copy_from_slice(payload.as_bytes()))
        .await
        .unwrap();
    stream.finish().await.unwrap();
    let response = stream.recv_response().await.unwrap();
    assert_eq!(response.status(), 201);
    let mut echoed = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await.unwrap() {
        while chunk.has_remaining() {
            echoed.extend_from_slice(chunk.chunk());
            let length = chunk.chunk().len();
            chunk.advance(length);
        }
    }
    connection.close(0_u32.into(), b"complete");
    client.wait_idle().await;
    driver_task.abort();
    assert_eq!(server_task.await.unwrap(), payload.as_bytes());
    assert_eq!(echoed, payload.as_bytes());
}

fn authenticate_and_associate(
    proxy: SocketAddr,
    password: &[u8],
    udp_client: SocketAddr,
) -> (TcpStream, SocketAddr) {
    let mut control = TcpStream::connect(proxy).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    control.write_all(&[5, 1, 2]).unwrap();
    let mut selection = [0_u8; 2];
    control.read_exact(&mut selection).unwrap();
    assert_eq!(selection, [5, 2]);
    let mut auth = vec![1, 7];
    auth.extend_from_slice(b"fragcap");
    auth.push(password.len().try_into().unwrap());
    auth.extend_from_slice(password);
    control.write_all(&auth).unwrap();
    let mut auth_response = [0_u8; 2];
    control.read_exact(&mut auth_response).unwrap();
    assert_eq!(auth_response, [1, 0]);
    let SocketAddr::V4(udp_client) = udp_client else {
        panic!("IPv4 test client")
    };
    let mut associate = vec![5, 3, 0, 1];
    associate.extend_from_slice(&udp_client.ip().octets());
    associate.extend_from_slice(&udp_client.port().to_be_bytes());
    control.write_all(&associate).unwrap();
    let mut head = [0_u8; 4];
    control.read_exact(&mut head).unwrap();
    assert_eq!(head, [5, 0, 0, 1]);
    let mut tail = [0_u8; 6];
    control.read_exact(&mut tail).unwrap();
    let relay = SocketAddr::from((
        [tail[0], tail[1], tail[2], tail[3]],
        u16::from_be_bytes([tail[4], tail[5]]),
    ));
    (control, relay)
}

fn domain_frame(port: u16, payload: &[u8]) -> Vec<u8> {
    let mut frame = vec![0, 0, 0, 3, 9];
    frame.extend_from_slice(b"localhost");
    frame.extend_from_slice(&port.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn response_payload(frame: &[u8]) -> &[u8] {
    match frame[3] {
        1 => &frame[10..],
        4 => &frame[22..],
        3 => &frame[7 + frame[4] as usize..],
        value => panic!("unexpected SOCKS address type {value}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn authenticated_socks_udp_route_proxies_http3_end_to_end() {
    let origin_authority = DestinationAuthority::parse("localhost:443").unwrap();
    let origin_ca =
        SessionCertificateAuthority::generate(91, SystemTime::now(), Duration::from_secs(3600))
            .unwrap();
    let mut origin_cache = LeafCache::new(
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(128 * 1024).unwrap(),
        Duration::from_secs(1800),
        91,
    )
    .unwrap();
    let limits = ProtocolLimits {
        max_concurrent_streams: 4,
        max_socks_udp_datagram_bytes: 4096,
        idle_timeout: Duration::from_secs(5),
        ..ProtocolLimits::default()
    };
    let origin_config = build_quic_server_config(
        &origin_authority,
        &origin_ca,
        &mut origin_cache,
        None,
        &limits,
    )
    .unwrap();
    let origin = Endpoint::server(origin_config, "127.0.0.1:0".parse().unwrap()).unwrap();
    let origin_address = origin.local_addr().unwrap();
    let origin_task = tokio::spawn(async move {
        let connection = origin.accept().await.unwrap().await.unwrap();
        let datagrams = connection.clone();
        let datagram_task = tokio::spawn(async move {
            let value = datagrams.read_datagram().await.unwrap();
            datagrams.send_datagram(value).unwrap();
        });
        let mut h3 = h3::server::Connection::new(Connection::new(connection))
            .await
            .unwrap();
        let mut handlers = tokio::task::JoinSet::new();
        for _ in 0..2 {
            let resolver = h3.accept().await.unwrap().unwrap();
            handlers.spawn(async move {
                let (request, mut stream) = resolver.resolve_request().await.unwrap();
                assert!(request.uri().path().starts_with("/through-proxy/"));
                let early_response = request.uri().path().ends_with("/one");
                let mut response_sent = false;
                let mut body = Vec::new();
                while let Some(mut data) = stream.recv_data().await.unwrap() {
                    while data.has_remaining() {
                        body.extend_from_slice(data.chunk());
                        let length = data.chunk().len();
                        data.advance(length);
                    }
                    if early_response && !response_sent {
                        stream
                            .send_response(Response::builder().status(202).body(()).unwrap())
                            .await
                            .unwrap();
                        response_sent = true;
                    }
                }
                let request_trailers = stream.recv_trailers().await.unwrap().unwrap();
                assert_eq!(request_trailers["x-request-proof"], "observed");
                if !response_sent {
                    stream
                        .send_response(Response::builder().status(202).body(()).unwrap())
                        .await
                        .unwrap();
                }
                stream.send_data(Bytes::from(body)).await.unwrap();
                let mut response_trailers = HeaderMap::new();
                response_trailers.insert("x-response-proof", HeaderValue::from_static("observed"));
                stream.send_trailers(response_trailers).await.unwrap();
                stream.finish().await.unwrap();
            });
        }
        while let Some(result) = handlers.join_next().await {
            result.unwrap();
        }
        datagram_task.await.unwrap();
        let _ = h3.accept().await;
    });

    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        4,
        1024,
        Duration::from_secs(3),
    )
    .unwrap()
    .with_session_id("s118-http3-route")
    .unwrap()
    .with_protocol_limits(limits.clone());
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let mut origin_roots = RootCertStore::empty();
    origin_roots.add(origin_ca.der().clone()).unwrap();
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_tls_client_config(tls_client_config_with_roots(origin_roots).unwrap())
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(3))
        .unwrap();

    let shuttle = UdpSocket::bind("127.0.0.1:0").unwrap();
    shuttle
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let shuttle_address = shuttle.local_addr().unwrap();
    let (mut control, relay) = authenticate_and_associate(
        lease.endpoint(),
        lease.capability_proof().proxy_password().as_bytes(),
        shuttle_address,
    );
    let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    let client_address = client.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let shuttle_stop = Arc::clone(&stop);
    let shuttle_task = std::thread::spawn(move || {
        let mut buffer = vec![0_u8; 8192];
        while !shuttle_stop.load(Ordering::Relaxed) {
            match shuttle.recv_from(&mut buffer) {
                Ok((read, source)) if source == client_address => {
                    shuttle
                        .send_to(&domain_frame(origin_address.port(), &buffer[..read]), relay)
                        .unwrap();
                }
                Ok((read, source)) if source == relay => {
                    shuttle
                        .send_to(response_payload(&buffer[..read]), client_address)
                        .unwrap();
                }
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => panic!("shuttle failed: {error}"),
            }
        }
    });

    let mut client_roots = RootCertStore::empty();
    client_roots.add(lease.ca_der().to_vec().into()).unwrap();
    let client_config = build_quic_client_config(&client_tls(client_roots), &limits).unwrap();
    client.set_default_client_config(client_config);
    let connection = client
        .connect(shuttle_address, "localhost")
        .unwrap()
        .await
        .unwrap();
    let (mut driver, sender) = h3::client::new(Connection::new(connection.clone()))
        .await
        .unwrap();
    let driver_task =
        tokio::spawn(async move { std::future::poll_fn(|cx| driver.poll_close(cx)).await });
    let mut requests = tokio::task::JoinSet::new();
    for (path, payload) in [("one", "scoped-h3-one"), ("two", "scoped-h3-two")] {
        let mut sender = sender.clone();
        requests.spawn(async move {
            let mut stream = sender
                .send_request(
                    Request::post(format!("https://localhost/through-proxy/{path}"))
                        .body(())
                        .unwrap(),
                )
                .await
                .unwrap();
            let split = payload.len() / 2;
            stream
                .send_data(Bytes::copy_from_slice(&payload.as_bytes()[..split]))
                .await
                .unwrap();
            let response = if path == "one" {
                Some(stream.recv_response().await.unwrap())
            } else {
                None
            };
            stream
                .send_data(Bytes::copy_from_slice(&payload.as_bytes()[split..]))
                .await
                .unwrap();
            let mut request_trailers = HeaderMap::new();
            request_trailers.insert("x-request-proof", HeaderValue::from_static("observed"));
            stream.send_trailers(request_trailers).await.unwrap();
            stream.finish().await.unwrap();
            let response = match response {
                Some(response) => response,
                None => stream.recv_response().await.unwrap(),
            };
            assert_eq!(response.status(), 202);
            let mut echoed = Vec::new();
            while let Some(mut data) = stream.recv_data().await.unwrap() {
                while data.has_remaining() {
                    echoed.extend_from_slice(data.chunk());
                    let length = data.chunk().len();
                    data.advance(length);
                }
            }
            let response_trailers = stream.recv_trailers().await.unwrap().unwrap();
            assert_eq!(response_trailers["x-response-proof"], "observed");
            assert_eq!(echoed, payload.as_bytes());
        });
    }
    while let Some(result) = requests.join_next().await {
        result.unwrap();
    }
    connection
        .send_datagram(Bytes::from_static(b"scoped-datagram"))
        .unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), connection.read_datagram())
            .await
            .unwrap()
            .unwrap(),
        Bytes::from_static(b"scoped-datagram")
    );
    drop(sender);
    let close = tokio::time::timeout(Duration::from_secs(2), driver_task)
        .await
        .unwrap()
        .unwrap();
    assert!(close.is_h3_no_error());
    client.wait_idle().await;
    // Keep the UDP shuttle and its owning control connection alive until the
    // proxy closes the control side. That EOF proves the QUIC terminal reached
    // the association before cleanup reads its accounting.
    let mut eof = [0_u8; 1];
    assert_eq!(control.read(&mut eof).unwrap(), 0);
    drop(control);
    origin_task.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(3));
    stop.store(true, Ordering::Relaxed);
    shuttle_task.join().unwrap();
    assert!(report.is_clean());
    assert_eq!(report.observation.protocol.quic_pairs_started, 1);
    assert_eq!(report.observation.protocol.quic_pairs_completed, 1);
    assert_eq!(report.observation.protocol.http3_streams, 2);
    assert_eq!(report.observation.protocol.http3_streams_completed, 2);
    assert_eq!(report.observation.protocol.quic_datagrams, 2);
    let events = collector.0.lock().unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event.kind, ApplicationEventKind::QuicConnection(_))));
    assert!(events.iter().any(|event| matches!(event.kind, ApplicationEventKind::Metadata(ref block) if block.version == fragcap_proxy::ProtocolVersion::Http3)));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event.kind, ApplicationEventKind::Metadata(ref block) if block.kind == fragcap_proxy::MetadataKind::Trailers))
            .count(),
        4
    );
    assert!(events
        .iter()
        .any(|event| matches!(event.kind, ApplicationEventKind::QuicStream(_))));
}

#[tokio::test]
async fn two_independent_http3_lineages_use_session_tls_and_h3() {
    h3_round_trip(41).await;
    h3_round_trip(42).await;
}

#[test]
fn scoped_plan_refuses_endpoint_migration_and_origin_changes() {
    let authority = DestinationAuthority::parse("localhost:443").unwrap();
    let client = "127.0.0.1:50000".parse().unwrap();
    let origin = "127.0.0.1:4433".parse().unwrap();
    let plan = QuicInspectionPlan::new("session", 7, client, origin, &authority, true).unwrap();
    assert_eq!(
        plan.admits("127.0.0.1:50001".parse().unwrap(), origin, &authority),
        Err(QuicRefusalCode::MigrationRefused)
    );
    assert_eq!(
        plan.admits(client, "127.0.0.1:4434".parse().unwrap(), &authority),
        Err(QuicRefusalCode::OriginChanged)
    );
    let alias = DestinationAuthority::parse("127.0.0.1:443").unwrap();
    assert_eq!(
        plan.admits(client, origin, &alias),
        Err(QuicRefusalCode::OriginChanged)
    );
}

#[test]
fn transport_limits_are_finite_and_zero_rtt_is_disabled() {
    let mut roots = RootCertStore::empty();
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    roots.add(generated.cert.der().clone()).unwrap();
    let tls = client_tls(roots);
    assert!(!tls.enable_early_data);
    let _config = build_quic_client_config(&tls, &ProtocolLimits::default()).unwrap();
}

#[tokio::test]
async fn upstream_quic_rejects_an_untrusted_session_authority() {
    let authority = DestinationAuthority::parse("localhost:443").unwrap();
    let ca =
        SessionCertificateAuthority::generate(73, SystemTime::now(), Duration::from_secs(3600))
            .unwrap();
    let mut cache = LeafCache::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(64 * 1024).unwrap(),
        Duration::from_secs(1800),
        73,
    )
    .unwrap();
    let limits = ProtocolLimits::default();
    let server_config =
        build_quic_server_config(&authority, &ca, &mut cache, None, &limits).unwrap();
    let server = Endpoint::server(server_config, "127.0.0.1:0".parse().unwrap()).unwrap();
    let server_address = server.local_addr().unwrap();
    let accept = tokio::spawn(async move {
        let incoming = server.accept().await.unwrap();
        let _ = incoming.await;
    });
    let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    client.set_default_client_config(
        build_quic_client_config(&client_tls(RootCertStore::empty()), &limits).unwrap(),
    );
    assert!(tokio::time::timeout(
        Duration::from_secs(2),
        client.connect(server_address, "localhost").unwrap()
    )
    .await
    .unwrap()
    .is_err());
    accept.await.unwrap();
}
