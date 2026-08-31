// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, BodyDirection, BodyOutcome,
    BodyRepresentation, DestinationPolicy, EventDisposition, NativeProxyBackend, NativeProxyConfig,
    ProtocolLimits, StreamTerminal,
};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

fn read_head(stream: &mut impl Read) -> Vec<u8> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    head
}

fn start_proxy(origin: SocketAddr) -> fragcap_proxy::NativeProxyLease {
    start_proxy_with_sink(origin, None)
}

fn start_proxy_with_sink(
    origin: SocketAddr,
    sink: Option<Arc<Collector>>,
) -> fragcap_proxy::NativeProxyLease {
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        8,
        16 * 1024,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_session_id("s104-http-test")
    .unwrap();
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let backend = NativeProxyBackend::new(config).with_destination_policy(policy);
    let mut backend = match sink {
        Some(sink) => backend.with_application_event_sink(sink),
        None => backend,
    };
    backend.start(Duration::from_secs(2)).unwrap()
}

#[test]
fn forwards_absolute_form_and_preserves_informational_response() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut byte = [0_u8; 1];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        let request = String::from_utf8(request).unwrap();
        assert!(request.starts_with("GET /status HTTP/1.1\r\n"));
        assert!(!request.to_ascii_lowercase().contains("proxy-authorization"));
        stream
            .write_all(
                b"HTTP/1.1 103 Early Hints\r\nLink: </style.css>; rel=preload\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK",
            )
            .unwrap();
    });

    let mut lease = start_proxy(origin_address);
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let authorization = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        client,
        "GET http://{origin_address}/status HTTP/1.1\r\nHost: {origin_address}\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n",
        authorization.as_str()
    )
    .unwrap();
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 103 Early Hints\r\n"));
    assert!(response.contains("HTTP/1.1 200 OK\r\n"));
    assert!(response.ends_with("OK"));
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.requests, 1);
    assert_eq!(report.observation.protocol.responses, 1);
    assert_eq!(report.observation.protocol.informational_responses, 1);
    assert_eq!(report.observation.application.len(), 1);
    assert_eq!(report.observation.application[0].status, Some(200));
    assert!(report.observation.application[0]
        .transformations
        .contains(&"absolute-to-origin-form"));
}

#[test]
fn forwards_early_hints_and_continue_before_waiting_for_the_request_body() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let request = String::from_utf8(read_head(&mut stream)).unwrap();
        assert!(request.contains("Expect: 100-continue\r\n"));
        stream
            .write_all(b"HTTP/1.1 103 Early Hints\r\n\r\nHTTP/1.1 100 Continue\r\n\r\n")
            .unwrap();
        let mut body = [0_u8; 5];
        stream.read_exact(&mut body).unwrap();
        assert_eq!(&body, b"hello");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
            .unwrap();
    });

    let mut lease = start_proxy(address);
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let auth = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(client, "POST http://{address}/upload HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nExpect: 100-continue\r\nContent-Length: 5\r\nConnection: close\r\n\r\n", auth.as_str()).unwrap();
    assert!(read_head(&mut client).starts_with(b"HTTP/1.1 103"));
    assert!(read_head(&mut client).starts_with(b"HTTP/1.1 100"));
    client.write_all(b"hello").unwrap();
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"));
    assert!(response.ends_with("OK"));
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.requests, 1);
    assert_eq!(report.observation.protocol.responses, 1);
    assert_eq!(report.observation.protocol.informational_responses, 2);
    assert_eq!(report.observation.application.len(), 1);
    assert_eq!(report.observation.application[0].status, Some(200));
}

#[test]
fn retains_request_evidence_when_the_origin_closes_before_responding() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let request = String::from_utf8(read_head(&mut stream)).unwrap();
        assert!(request.starts_with("GET /lost HTTP/1.1\r\n"));
    });

    let mut lease = start_proxy(address);
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let auth = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    write!(client, "GET http://{address}/lost HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n", auth.as_str()).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert!(response.is_empty());
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.requests, 1);
    assert_eq!(report.observation.application.len(), 1);
    assert_eq!(
        report.observation.application[0].method.as_deref(),
        Some("GET")
    );
    assert!(report.observation.application[0]
        .url
        .as_deref()
        .is_some_and(|url| url.ends_with("/lost")));
    assert_eq!(report.observation.application[0].status, None);
    assert_eq!(
        report.observation.application[0].inspectability,
        "metadata-only"
    );
    assert_eq!(
        report.observation.application[0].reason.as_deref(),
        Some("upstream-response-eof")
    );
    assert_eq!(report.observation.protocol.observations_dropped_oldest, 0);
}

#[test]
fn forwards_a_final_expectation_rejection_without_waiting_for_a_body() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let request = String::from_utf8(read_head(&mut stream)).unwrap();
        assert!(request.contains("Expect: 100-continue\r\n"));
        stream
            .write_all(
                b"HTTP/1.1 417 Expectation Failed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let mut body = Vec::new();
        stream.read_to_end(&mut body).unwrap();
        assert!(body.is_empty());
    });

    let mut lease = start_proxy(address);
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let auth = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(client, "POST http://{address}/upload HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nExpect: 100-continue\r\nContent-Length: 5\r\n\r\n", auth.as_str()).unwrap();
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 417"));
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.responses, 1);
    assert_eq!(report.observation.application[0].status, Some(417));
}

#[test]
fn refuses_ambiguous_framing_before_connecting() {
    let unused = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = unused.local_addr().unwrap();
    let mut lease = start_proxy(address);
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let authorization = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    write!(
        client,
        "POST http://{address}/ HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n",
        authorization.as_str()
    )
    .unwrap();
    drop(client);
    std::thread::sleep(Duration::from_millis(50));
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.failed_connections, 1);
    assert!(report
        .observation
        .failures
        .iter()
        .any(|failure| failure.code == "http-framing-ambiguous"));
    assert!(unused.set_nonblocking(true).is_ok());
    assert!(unused.accept().is_err());
}

#[test]
fn relays_chunked_trailers_and_reuses_a_persistent_client_connection() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut first = read_head(&mut stream);
        let mut byte = [0_u8; 1];
        while !first.ends_with(b"0\r\nX-End: yes\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            first.push(byte[0]);
        }
        let text = String::from_utf8(first).unwrap();
        assert!(text.starts_with("POST /chunked HTTP/1.1\r\n"));
        assert!(text.contains("Transfer-Encoding: chunked\r\n"));
        assert!(text.ends_with("5\r\nhello\r\n0\r\nX-End: yes\r\n\r\n"));
        stream
            .write_all(
                b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK",
            )
            .unwrap();
        drop(stream);
        let (mut stream, _) = origin.accept().unwrap();
        let second = String::from_utf8(read_head(&mut stream)).unwrap();
        assert!(second.starts_with("OPTIONS * HTTP/1.1\r\n"));
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
            .unwrap();
    });

    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy_with_sink(address, Some(Arc::clone(&collector)));
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let auth = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(client, "POST http://{address}/chunked HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nTransfer-Encoding: chunked\r\nTrailer: X-End\r\n\r\n5\r\nhello\r\n0\r\nX-End: yes\r\n\r\n", auth.as_str()).unwrap();
    let first_response = read_head(&mut client);
    assert!(first_response.starts_with(b"HTTP/1.1 100"));
    let final_response = read_head(&mut client);
    assert!(final_response.starts_with(b"HTTP/1.1 200"));
    let mut body = [0_u8; 2];
    client.read_exact(&mut body).unwrap();
    assert_eq!(&body, b"OK");
    write!(
        client,
        "OPTIONS * HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\n\r\n",
        auth.as_str()
    )
    .unwrap();
    assert!(read_head(&mut client).starts_with(b"HTTP/1.1 204"));
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.requests, 2);
    assert_eq!(report.observation.protocol.responses, 2);
    assert_eq!(report.observation.protocol.informational_responses, 1);
    let events = collector.0.lock().unwrap();
    assert!(events.iter().any(|event| {
        matches!(
            &event.kind,
            ApplicationEventKind::Body(segment)
                if segment.representation == BodyRepresentation::Raw
                    && segment.bytes.as_ref() == b"5\r\n"
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            &event.kind,
            ApplicationEventKind::Body(segment)
                if segment.representation == BodyRepresentation::TransferDecoded
                    && segment.bytes.as_ref() == b"hello"
        )
    }));
}

#[test]
fn accepted_upgrade_relays_bidirectionally_and_observes_half_close() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let request = String::from_utf8(read_head(&mut stream)).unwrap();
        assert!(request.contains("Connection: Upgrade\r\n"));
        stream
            .write_all(b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: fragcap-test\r\n\r\n")
            .unwrap();
        let mut payload = [0_u8; 4];
        stream.read_exact(&mut payload).unwrap();
        assert_eq!(&payload, b"PING");
        stream.write_all(b"PONG").unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
    });

    let mut lease = start_proxy(address);
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let auth = lease.capability_proof().proxy_authorization();
    let mut client = TcpStream::connect(endpoint).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(client, "GET http://{address}/upgrade HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nConnection: Upgrade\r\nUpgrade: fragcap-test\r\n\r\n", auth.as_str()).unwrap();
    assert!(read_head(&mut client).starts_with(b"HTTP/1.1 101"));
    client.write_all(b"PING").unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert_eq!(response, b"PONG");
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.failed_connections, 0);
    assert_eq!(report.observation.protocol.responses, 1);
}

#[test]
fn verified_websocket_upgrade_preserves_frames_and_messages() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let request = String::from_utf8(read_head(&mut stream)).unwrap();
        assert!(request.contains("Upgrade: websocket\r\n"));
        stream.write_all(b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n").unwrap();
        let mut frame = [0_u8; 8];
        stream.read_exact(&mut frame).unwrap();
        assert_eq!(frame, [0x81, 0x82, 1, 2, 3, 4, b'h' ^ 1, b'i' ^ 2]);
        stream.write_all(b"\x81\x02ok").unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
    });

    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy_with_sink(address, Some(collector.clone()));
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let auth = lease.capability_proof().proxy_authorization();
    write!(client, "GET http://{address}/socket HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n", auth.as_str()).unwrap();
    assert!(read_head(&mut client).starts_with(b"HTTP/1.1 101"));
    client
        .write_all(&[0x81, 0x82, 1, 2, 3, 4, b'h' ^ 1, b'i' ^ 2])
        .unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert_eq!(response, b"\x81\x02ok");
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
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
                event.kind,
                ApplicationEventKind::Streaming(fragcap_proxy::StreamingEvent::WebSocketMessage(_))
            ))
            .count(),
        2
    );
}

#[test]
fn session_body_retention_is_shared_across_http1_connections() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = origin.accept().unwrap();
            let _ = read_head(&mut stream);
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n12345678",
                )
                .unwrap();
        }
    });
    let limits = ProtocolLimits {
        max_body_bytes: 8,
        max_session_body_bytes: 10,
        ..ProtocolLimits::default()
    };
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        8,
        16 * 1024,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_session_id("s105-session-retention")
    .unwrap()
    .with_protocol_limits(limits);
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(address);
    let collector = Arc::new(Collector::default());
    let mut lease = NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_application_event_sink(collector.clone())
        .start(Duration::from_secs(2))
        .unwrap();
    for path in ["one", "two"] {
        let mut client = TcpStream::connect(lease.endpoint()).unwrap();
        let auth = lease.capability_proof().proxy_authorization();
        write!(client, "GET http://{address}/{path} HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n", auth.as_str()).unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();
        assert!(response.ends_with(b"12345678"));
    }
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean(), "{report:?}");
    let events = collector.0.lock().unwrap();
    let response_bodies: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.kind {
            ApplicationEventKind::Body(segment)
                if segment.direction == BodyDirection::Response
                    && segment.representation == BodyRepresentation::Raw
                    && segment.observed_len > 0 =>
            {
                Some(segment)
            }
            _ => None,
        })
        .collect();
    assert_eq!(response_bodies.len(), 2);
    assert_eq!(
        response_bodies
            .iter()
            .map(|body| body.observed_len)
            .sum::<u64>(),
        16
    );
    assert_eq!(
        response_bodies
            .iter()
            .map(|body| body.bytes.len())
            .sum::<usize>(),
        10
    );
    assert_eq!(response_bodies[1].outcome, BodyOutcome::RetentionLimit);
}

#[test]
fn failed_http1_response_body_emits_body_and_stream_terminals() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let _ = read_head(&mut stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nabc")
            .unwrap();
    });
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy_with_sink(address, Some(collector.clone()));
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    let auth = lease.capability_proof().proxy_authorization();
    write!(client, "GET http://{address}/partial HTTP/1.1\r\nHost: {address}\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n", auth.as_str()).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert!(response.ends_with(b"abc"));
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.failed_connections, 1);
    let events = collector.0.lock().unwrap();
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        ApplicationEventKind::Body(segment)
            if segment.direction == BodyDirection::Response
                && segment.outcome == BodyOutcome::Partial
                && segment.observed_len == 0
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ApplicationEventKind::HttpStreamTerminal(StreamTerminal::TransportError)
    )));
}
