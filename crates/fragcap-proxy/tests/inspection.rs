// SPDX-License-Identifier: Apache-2.0

mod support;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    ApplicationEvent, ApplicationEventSink, EventDisposition, InspectionAction, InspectionDecider,
    InspectionDecision, InspectionReason, InspectionRequest, NativeProxyBackend, NativeProxyConfig,
};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

struct RetiredSink;

impl ApplicationEventSink for RetiredSink {
    fn try_emit(&self, _event: ApplicationEvent) -> EventDisposition {
        EventDisposition::Retired
    }
}

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

#[derive(Default)]
struct RefuseInspection {
    seen: Mutex<Vec<InspectionRequest>>,
}

impl InspectionDecider for RefuseInspection {
    fn decide(&self, request: InspectionRequest, _budget: Duration) -> InspectionDecision {
        self.seen.lock().unwrap().push(request);
        InspectionDecision::uninspected(InspectionReason::KnownMiddleware, 1)
    }
}

#[derive(Default)]
struct SequencedDecider(AtomicUsize);

struct GameDecider;

impl InspectionDecider for GameDecider {
    fn decide(&self, _request: InspectionRequest, _budget: Duration) -> InspectionDecision {
        InspectionDecision::verified_game(1)
    }
}

impl InspectionDecider for SequencedDecider {
    fn decide(&self, _request: InspectionRequest, _budget: Duration) -> InspectionDecision {
        if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
            InspectionDecision::uninspected(InspectionReason::KnownMiddleware, 1)
        } else {
            InspectionDecision::verified_game(1)
        }
    }
}

#[test]
fn pinned_middleware_and_inspected_game_share_one_origin_hostname() {
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], private.into())
        .unwrap();
    let server_config = Arc::new(server_config);
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..2 {
            let (tcp, _) = origin.accept().unwrap();
            let connection = rustls::ServerConnection::new(Arc::clone(&server_config)).unwrap();
            let mut tls = rustls::StreamOwned::new(connection, tcp);
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                tls.read_exact(&mut byte).unwrap();
                head.push(byte[0]);
            }
            assert!(head.starts_with(b"GET /same-host HTTP/1.1"));
            tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .unwrap();
            tls.flush().unwrap();
        }
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        4,
        16 * 1024,
        Duration::from_secs(3),
    )
    .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let mut upstream_roots = rustls::RootCertStore::empty();
    upstream_roots.add(certificate.clone()).unwrap();
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(SequencedDecider::default()))
        .with_tls_client_config(
            fragcap_proxy::tls_client_config_with_roots(upstream_roots).unwrap(),
        )
        .start(Duration::from_secs(3))
        .unwrap();
    for root in [certificate, CertificateDer::from(lease.ca_der().to_vec())] {
        let mut tcp = TcpStream::connect(lease.endpoint()).unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let auth = lease.capability_proof().proxy_authorization();
        write!(tcp, "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {}\r\n\r\n", origin_address.port(), origin_address.port(), auth.as_str()).unwrap();
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            tcp.read_exact(&mut byte).unwrap();
            head.push(byte[0]);
        }
        assert_eq!(head, b"HTTP/1.1 200 Connection Established\r\n\r\n");
        let mut roots = rustls::RootCertStore::empty();
        roots.add(root).unwrap();
        let config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
        let mut connection = rustls::ClientConnection::new(
            Arc::new(config),
            ServerName::try_from("localhost").unwrap().to_owned(),
        )
        .unwrap();
        write!(
            connection.writer(),
            "GET /same-host HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
            origin_address.port()
        )
        .unwrap();
        let mut tls = rustls::StreamOwned::new(connection, tcp);
        tls.flush().unwrap();
        let mut response = String::new();
        if let Err(error) = tls.read_to_string(&mut response) {
            assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof, "{error}");
        }
        assert!(response.ends_with("OK"), "{response}");
    }
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.client_tls_completed, 1);
    assert_eq!(report.observation.protocol.upstream_tls_completed, 1);
    assert_eq!(report.observation.connection_diagnostics.len(), 2);
    assert_eq!(
        report.observation.connection_diagnostics[0]
            .inspection
            .unwrap()
            .action,
        InspectionAction::Uninspected
    );
    assert_eq!(
        report.observation.connection_diagnostics[1]
            .inspection
            .unwrap()
            .action,
        InspectionAction::Inspect
    );
}

#[test]
fn failed_game_upstream_tls_never_switches_to_blind_relay() {
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let certificate = CertificateDer::from(generated.cert);
    let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![certificate], private.into())
    .unwrap();
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (tcp, _) = origin.accept().unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, tcp);
        let mut byte = [0_u8; 1];
        let _ = tls.read(&mut byte);
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(2),
    )
    .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(address);
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(GameDecider))
        .with_tls_client_config(support::isolated_tls_client_config())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut tcp = TcpStream::connect(lease.endpoint()).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    write!(
        tcp,
        "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {}\r\n\r\n",
        address.port(),
        address.port(),
        lease.capability_proof().proxy_authorization().as_str()
    )
    .unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        tcp.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(lease.ca_der().to_vec()))
        .unwrap();
    let client = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();
    let mut connection = rustls::ClientConnection::new(
        Arc::new(client),
        ServerName::try_from("localhost").unwrap().to_owned(),
    )
    .unwrap();
    write!(
        connection.writer(),
        "GET /same-host HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        address.port()
    )
    .unwrap();
    let mut tls = rustls::StreamOwned::new(connection, tcp);
    tls.flush().unwrap();
    let mut response = Vec::new();
    let _ = tls.read_to_end(&mut response);
    assert!(!response.ends_with(b"OK"));
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.client_tls_completed, 1);
    assert_eq!(report.observation.protocol.upstream_tls_completed, 0);
    assert_eq!(report.observation.protocol.relay_client_bytes, 0);
    assert_eq!(report.observation.protocol.relay_upstream_bytes, 0);
    let diagnostic = &report.observation.connection_diagnostics[0];
    assert_eq!(
        diagnostic.inspection.unwrap().action,
        InspectionAction::Inspect
    );
    assert_eq!(diagnostic.terminal, "failed");
}

#[test]
fn native_owner_decision_precedes_all_protocol_and_tls_work() {
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(1),
    )
    .unwrap();
    let decider = Arc::new(RefuseInspection::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_inspection_decider(decider.clone())
        .start(Duration::from_secs(1))
        .unwrap();
    let endpoint = lease.endpoint();
    let client = TcpStream::connect(endpoint).unwrap();
    drop(client);
    std::thread::sleep(Duration::from_millis(50));
    let report = lease.cleanup(Duration::from_secs(1));
    assert!(report.is_clean(), "{report:?}");
    let seen = decider.seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].proxy_local, endpoint);
    let diagnostic = &report.observation.connection_diagnostics[0];
    let decision = diagnostic.inspection.unwrap();
    assert_eq!(decision.action, InspectionAction::Uninspected);
    assert_eq!(decision.reason, InspectionReason::KnownMiddleware);
    assert_eq!(decision.requested_scope, "exact-game");
    assert_eq!(decision.policy_id, "exact-game-v1");
    assert_eq!(decision.owner_proof, "unproven");
    assert!(diagnostic.stages.iter().all(|stage| {
        !matches!(
            stage.stage,
            fragcap_proxy::ConnectionStage::ClientTls
                | fragcap_proxy::ConnectionStage::UpstreamTls
                | fragcap_proxy::ConnectionStage::FirstRequest
        )
    }));
}

#[test]
fn middleware_connect_relays_original_bytes_without_tls_or_application_evidence() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut input = Vec::new();
        stream.read_to_end(&mut input).unwrap();
        assert_eq!(input, b"original-client-tls-bytes");
        stream.write_all(b"original-origin-tls-bytes").unwrap();
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(2),
    )
    .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let decider = Arc::new(RefuseInspection::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(decider)
        .with_application_event_sink(Arc::new(RetiredSink))
        .start(Duration::from_secs(2))
        .unwrap();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let auth = lease.capability_proof().proxy_authorization();
    write!(client, "CONNECT {origin_address} HTTP/1.1\r\nHost: {origin_address}\r\nProxy-Authorization: {}\r\n\r\n", auth.as_str()).unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        client.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    assert_eq!(head, b"HTTP/1.1 200 Connection Established\r\n\r\n");
    client.write_all(b"original-client-tls-bytes").unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert_eq!(response, b"original-origin-tls-bytes");
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    let observation = &report.observation;
    assert_eq!(observation.protocol.connect_requests, 1);
    assert_eq!(
        observation.protocol.relay_client_bytes,
        b"original-client-tls-bytes".len() as u64
    );
    assert_eq!(
        observation.protocol.relay_upstream_bytes,
        b"original-origin-tls-bytes".len() as u64
    );
    assert_eq!(observation.protocol.client_tls_completed, 0);
    assert_eq!(observation.protocol.upstream_tls_completed, 0);
    assert!(observation
        .application
        .iter()
        .all(|item| item.protocol != "tls"));
    let diagnostic = &observation.connection_diagnostics[0];
    assert_eq!(
        diagnostic.inspection.unwrap().action,
        InspectionAction::Uninspected
    );
    assert!(diagnostic
        .stages
        .iter()
        .all(|stage| stage.stage != fragcap_proxy::ConnectionStage::ClientTls));
}

#[test]
fn middleware_connect_uses_same_relay_on_ipv6_when_available() {
    let Ok(origin) = TcpListener::bind("[::1]:0") else {
        return;
    };
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut bytes = [0_u8; 4];
        stream.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"ping");
        stream.write_all(b"pong").unwrap();
    });
    let config =
        NativeProxyConfig::new("[::1]:0".parse().unwrap(), 2, 4096, Duration::from_secs(2))
            .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(address);
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .start(Duration::from_secs(2))
        .unwrap();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        client,
        "CONNECT {address} HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\n\r\n",
        lease.capability_proof().proxy_authorization().as_str()
    )
    .unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        client.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    assert_eq!(head, b"HTTP/1.1 200 Connection Established\r\n\r\n");
    client.write_all(b"ping").unwrap();
    let mut reply = [0_u8; 4];
    client.read_exact(&mut reply).unwrap();
    assert_eq!(&reply, b"pong");
    drop(client);
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.relay_client_bytes, 4);
    assert_eq!(report.observation.protocol.relay_upstream_bytes, 4);
}

#[test]
fn established_relay_uses_activity_idle_instead_of_http_header_deadline() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        for byte in [b'a', b'b', b'c'] {
            stream.write_all(&[byte]).unwrap();
            std::thread::sleep(Duration::from_millis(45));
        }
    });
    let limits = fragcap_proxy::ProtocolLimits {
        header_timeout: Duration::from_millis(20),
        idle_timeout: Duration::from_millis(110),
        ..Default::default()
    };
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_protocol_limits(limits);
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(address);
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .start(Duration::from_secs(2))
        .unwrap();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        client,
        "CONNECT {address} HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\n\r\n",
        lease.capability_proof().proxy_authorization().as_str()
    )
    .unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        client.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    let mut bytes = [0_u8; 3];
    client.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"abc");
    client.shutdown(Shutdown::Write).unwrap();
    drop(client);
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.relay_upstream_bytes, 3);
    assert_eq!(report.observation.protocol.timed_out, 0);
}

#[test]
fn genuinely_idle_relay_ends_with_a_named_timeout() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (_stream, _) = origin.accept().unwrap();
        std::thread::sleep(Duration::from_millis(130));
    });
    let limits = fragcap_proxy::ProtocolLimits {
        idle_timeout: Duration::from_millis(45),
        ..Default::default()
    };
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_protocol_limits(limits);
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(address);
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .start(Duration::from_secs(2))
        .unwrap();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        client,
        "CONNECT {address} HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\n\r\n",
        lease.capability_proof().proxy_authorization().as_str()
    )
    .unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        client.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    let mut remainder = Vec::new();
    client.read_to_end(&mut remainder).unwrap();
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        report.observation.connection_diagnostics[0].code,
        Some("relay-idle-timeout")
    );
    assert_eq!(report.observation.protocol.timed_out, 1);
}

#[test]
fn middleware_plain_http_forwards_without_exporting_request_or_response_content() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            head.push(byte[0]);
        }
        assert!(String::from_utf8_lossy(&head).starts_with("POST /private HTTP/1.1"));
        assert!(!String::from_utf8_lossy(&head).contains("Proxy-Authorization"));
        let mut body = [0_u8; 6];
        stream.read_exact(&mut body).unwrap();
        assert_eq!(&body, b"secret");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nreply")
            .unwrap();
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_protocol_limits(fragcap_proxy::ProtocolLimits {
        max_body_bytes: 1,
        max_session_body_bytes: 1,
        ..Default::default()
    });
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(client, "POST http://{origin_address}/private HTTP/1.1\r\nHost: {origin_address}\r\nProxy-Authorization: {}\r\nContent-Length: 6\r\nConnection: close\r\n\r\nsecret", lease.capability_proof().proxy_authorization().as_str()).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert!(response.ends_with(b"reply"));
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert!(collector.0.lock().unwrap().iter().all(|event| matches!(
        event.kind,
        fragcap_proxy::ApplicationEventKind::ConnectionOpen(_)
            | fragcap_proxy::ApplicationEventKind::ConnectionTerminal(_)
    )));
    assert_eq!(report.observation.application.len(), 1);
    let summary = &report.observation.application[0];
    assert_eq!(summary.protocol, "http-relay");
    assert_eq!(
        summary.url.as_deref(),
        Some(origin_address.to_string().as_str())
    );
    assert_eq!(summary.method, None);
    assert!(report.observation.protocol.relay_client_bytes > 6);
    assert!(report.observation.protocol.relay_upstream_bytes > 5);
    assert_eq!(report.observation.protocol.body_bytes_retained, 0);
    assert_eq!(report.observation.protocol.body_bytes_omitted, 0);
}

#[test]
fn middleware_socks_tcp_preserves_half_close_without_generic_payload_export() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut input = Vec::new();
        stream.read_to_end(&mut input).unwrap();
        assert_eq!(input, b"opaque-socks");
        stream.write_all(b"opaque-reply").unwrap();
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(2),
    )
    .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client.write_all(&[5, 1, 2]).unwrap();
    let mut selection = [0_u8; 2];
    client.read_exact(&mut selection).unwrap();
    assert_eq!(selection, [5, 2]);
    let password = lease.capability_proof().proxy_password();
    let mut auth = vec![1, 7];
    auth.extend_from_slice(b"fragcap");
    auth.push(password.len() as u8);
    auth.extend_from_slice(password.as_bytes());
    client.write_all(&auth).unwrap();
    let mut auth_reply = [0_u8; 2];
    client.read_exact(&mut auth_reply).unwrap();
    assert_eq!(auth_reply, [1, 0]);
    let std::net::SocketAddr::V4(address) = origin_address else {
        unreachable!()
    };
    let mut request = vec![5, 1, 0, 1];
    request.extend_from_slice(&address.ip().octets());
    request.extend_from_slice(&address.port().to_be_bytes());
    client.write_all(&request).unwrap();
    let mut reply = [0_u8; 10];
    client.read_exact(&mut reply).unwrap();
    assert_eq!(&reply[..2], &[5, 0]);
    client.write_all(b"opaque-socks").unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert_eq!(response, b"opaque-reply");
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.relay_client_bytes, 12);
    assert_eq!(report.observation.protocol.relay_upstream_bytes, 12);
    assert!(!collector.0.lock().unwrap().iter().any(|event| matches!(
        event.kind,
        fragcap_proxy::ApplicationEventKind::GenericStreamChunk(_)
    )));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn middleware_http2_connect_preserves_original_stream() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = origin.accept().await.unwrap();
        let mut input = Vec::new();
        stream.read_to_end(&mut input).await.unwrap();
        assert_eq!(input, b"opaque-h2-connect");
        stream.write_all(b"opaque-origin-reply").await.unwrap();
        drop(stream);
        let (stream, _) = origin.accept().await.unwrap();
        let mut h2 = h2::server::handshake(stream).await.unwrap();
        let (request, mut respond) = h2.accept().await.unwrap().unwrap();
        assert_eq!(request.uri().path(), "/later");
        let response = hyper::Response::builder().status(204).body(()).unwrap();
        respond.send_response(response, true).unwrap();
        while h2.accept().await.is_some() {}
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(3),
    )
    .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (sender, connection) = h2::client::handshake(tcp).await.unwrap();
    let driver = tokio::spawn(connection);
    let mut sender = sender.ready().await.unwrap();
    let request = hyper::Request::builder()
        .method("CONNECT")
        .uri(origin_address.to_string())
        .header(
            hyper::header::PROXY_AUTHORIZATION,
            lease.capability_proof().proxy_authorization().as_str(),
        )
        .body(())
        .unwrap();
    let (response, mut send) = sender.send_request(request, false).unwrap();
    let response = tokio::time::timeout(Duration::from_secs(3), response)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.status(), 200);
    send.send_data(bytes::Bytes::from_static(b"opaque-h2-connect"), true)
        .unwrap();
    let mut body = response.into_body();
    let mut output = Vec::new();
    while let Some(chunk) = tokio::time::timeout(Duration::from_secs(3), body.data())
        .await
        .unwrap()
    {
        output.extend_from_slice(&chunk.unwrap());
    }
    assert_eq!(output, b"opaque-origin-reply");
    let mut sender = sender.ready().await.unwrap();
    let later = hyper::Request::builder()
        .method("GET")
        .uri(format!("http://{origin_address}/later"))
        .body(())
        .unwrap();
    let (later, _) = sender.send_request(later, true).unwrap();
    assert_eq!(later.await.unwrap().status(), 204);
    drop(sender);
    driver.abort();
    let _ = driver.await;
    server.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.http2_streams_completed, 2);
    assert!(report.observation.application.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn middleware_http2_connect_keeps_destination_policy() {
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_address = origin.local_addr().unwrap();
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(3),
    )
    .unwrap();
    let mut lease = NativeProxyBackend::new(config)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (sender, connection) = h2::client::handshake(tcp).await.unwrap();
    let driver = tokio::spawn(connection);
    let mut sender = sender.ready().await.unwrap();
    let request = hyper::Request::builder()
        .method("CONNECT")
        .uri(origin_address.to_string())
        .header(
            hyper::header::PROXY_AUTHORIZATION,
            lease.capability_proof().proxy_authorization().as_str(),
        )
        .body(())
        .unwrap();
    let (response, _) = sender.send_request(request, true).unwrap();
    assert!(response.await.is_err());
    assert!(
        tokio::time::timeout(Duration::from_millis(100), origin.accept())
            .await
            .is_err()
    );
    drop(sender);
    driver.abort();
    let _ = driver.await;
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.policy_refused, 1);
    assert_eq!(report.observation.protocol.relay_client_bytes, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn middleware_http2_concurrent_requests_forward_without_application_exports() {
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (tcp, _) = origin.accept().await.unwrap();
        let mut connection = h2::server::handshake(tcp).await.unwrap();
        let mut replies = tokio::task::JoinSet::new();
        for _ in 0..2 {
            let (request, mut respond) = connection.accept().await.unwrap().unwrap();
            replies.spawn(async move {
                assert!(request.uri().path().starts_with("/private"));
                let response = hyper::Response::builder().status(204).body(()).unwrap();
                respond.send_response(response, true).unwrap();
            });
        }
        while replies.join_next().await.is_some() {}
        while connection.accept().await.is_some() {}
    });
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        2,
        4096,
        Duration::from_secs(3),
    )
    .unwrap();
    let mut policy = fragcap_proxy::DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin_address);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_inspection_decider(Arc::new(RefuseInspection::default()))
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    let tcp = tokio::net::TcpStream::connect(lease.endpoint())
        .await
        .unwrap();
    let (sender, connection) = h2::client::handshake(tcp).await.unwrap();
    let driver = tokio::spawn(connection);
    let mut sender = sender.ready().await.unwrap();
    let first = hyper::Request::builder()
        .method("GET")
        .uri(format!("http://{origin_address}/private-one"))
        .header(
            hyper::header::PROXY_AUTHORIZATION,
            lease.capability_proof().proxy_authorization().as_str(),
        )
        .body(())
        .unwrap();
    let (first, _) = sender.send_request(first, true).unwrap();
    let mut sender = sender.ready().await.unwrap();
    let second = hyper::Request::builder()
        .method("GET")
        .uri(format!("http://{origin_address}/private-two"))
        .body(())
        .unwrap();
    let (second, _) = sender.send_request(second, true).unwrap();
    assert_eq!(first.await.unwrap().status(), 204);
    assert_eq!(second.await.unwrap().status(), 204);
    drop(sender);
    driver.abort();
    let _ = driver.await;
    server.await.unwrap();
    let report = lease.cleanup(Duration::from_secs(3));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.observation.protocol.http2_streams_completed, 2);
    assert!(collector.0.lock().unwrap().iter().all(|event| matches!(
        event.kind,
        fragcap_proxy::ApplicationEventKind::ConnectionOpen(_)
            | fragcap_proxy::ApplicationEventKind::ConnectionTerminal(_)
    )));
}
