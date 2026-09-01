// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, DestinationPolicy,
    EventDisposition, NativeProxyBackend, NativeProxyConfig, ProtocolLimits, StreamTerminal,
};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

fn start_proxy(origin: SocketAddr) -> fragcap_proxy::NativeProxyLease {
    start_proxy_with(origin, ProtocolLimits::default(), None)
}

fn start_proxy_with(
    origin: SocketAddr,
    limits: ProtocolLimits,
    collector: Option<Arc<Collector>>,
) -> fragcap_proxy::NativeProxyLease {
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        4,
        1024,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_session_id("s114-socks-test")
    .unwrap()
    .with_protocol_limits(limits);
    let mut policy = DestinationPolicy::new(config.listen());
    policy.grant_for_test(origin);
    let backend = NativeProxyBackend::new(config).with_destination_policy(policy);
    let mut backend = match collector {
        Some(collector) => backend.with_application_event_sink(collector),
        None => backend,
    };
    backend.start(Duration::from_secs(2)).unwrap()
}

fn authenticate(client: &mut TcpStream, password: &[u8]) {
    client.write_all(&[5, 1, 2]).unwrap();
    let mut selection = [0_u8; 2];
    client.read_exact(&mut selection).unwrap();
    assert_eq!(selection, [5, 2]);
    let mut request = vec![1, 7];
    request.extend_from_slice(b"fragcap");
    request.push(password.len().try_into().unwrap());
    request.extend_from_slice(password);
    client.write_all(&request).unwrap();
    let mut response = [0_u8; 2];
    client.read_exact(&mut response).unwrap();
    assert_eq!(response, [1, 0]);
}

fn connect_ipv4(client: &mut TcpStream, destination: SocketAddr) {
    let SocketAddr::V4(destination) = destination else {
        panic!("test requires IPv4");
    };
    let mut request = vec![5, 1, 0, 1];
    request.extend_from_slice(&destination.ip().octets());
    request.extend_from_slice(&destination.port().to_be_bytes());
    client.write_all(&request).unwrap();
    let mut reply = [0_u8; 10];
    client.read_exact(&mut reply).unwrap();
    assert_eq!(&reply[..4], &[5, 0, 0, 1]);
}

fn read_reply(client: &mut TcpStream) -> (u8, SocketAddr) {
    let mut head = [0_u8; 4];
    client.read_exact(&mut head).unwrap();
    assert_eq!(head[0], 5);
    let address = match head[3] {
        1 => {
            let mut tail = [0_u8; 6];
            client.read_exact(&mut tail).unwrap();
            SocketAddr::from((
                std::net::Ipv4Addr::new(tail[0], tail[1], tail[2], tail[3]),
                u16::from_be_bytes([tail[4], tail[5]]),
            ))
        }
        4 => {
            let mut tail = [0_u8; 18];
            client.read_exact(&mut tail).unwrap();
            let mut octets = [0_u8; 16];
            octets.copy_from_slice(&tail[..16]);
            SocketAddr::from((
                std::net::Ipv6Addr::from(octets),
                u16::from_be_bytes([tail[16], tail[17]]),
            ))
        }
        value => panic!("unexpected reply address type {value}"),
    };
    (head[1], address)
}

#[test]
fn authenticated_ipv4_connect_preserves_bytes_and_half_close() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut request = Vec::new();
        stream.read_to_end(&mut request).unwrap();
        assert_eq!(request, b"opaque request");
        stream.write_all(b"opaque response").unwrap();
    });
    let mut lease = start_proxy(address);
    let endpoint = lease.endpoint();
    let password = lease.capability_proof().proxy_password();
    let mut client = TcpStream::connect(endpoint).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut client, password.as_bytes());
    connect_ipv4(&mut client, address);
    client.write_all(b"opaque request").unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut response = Vec::new();
    client.read_to_end(&mut response).unwrap();
    assert_eq!(response, b"opaque response");
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_connect_succeeded, 1);
    assert_eq!(report.observation.protocol.socks_tcp_opaque, 1);
    assert_eq!(report.observation.protocol.socks_client_bytes, 14);
    assert_eq!(report.observation.protocol.socks_upstream_bytes, 15);
}

#[test]
fn wrong_password_is_refused_before_origin_accept() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    origin.set_nonblocking(true).unwrap();
    let address = origin.local_addr().unwrap();
    let mut lease = start_proxy(address);
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client.write_all(&[5, 1, 2]).unwrap();
    let mut selection = [0_u8; 2];
    client.read_exact(&mut selection).unwrap();
    client
        .write_all(&[
            1, 7, b'f', b'r', b'a', b'g', b'c', b'a', b'p', 5, b'w', b'r', b'o', b'n', b'g',
        ])
        .unwrap();
    let mut auth = [0_u8; 2];
    client.read_exact(&mut auth).unwrap();
    assert_eq!(auth, [1, 1]);
    std::thread::sleep(Duration::from_millis(50));
    assert!(origin.accept().is_err());
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.authentication_refused, 1);
    assert_eq!(report.observation.protocol.socks_auth_refused, 1);
}

#[test]
fn domain_connect_is_proxy_resolved_and_http_classified() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut bytes = [0_u8; 18];
        stream.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"GET / HTTP/1.1\r\n\r\n");
    });
    let mut lease = start_proxy(address);
    let password = lease.capability_proof().proxy_password();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut client, password.as_bytes());
    let mut request = vec![5, 1, 0, 3, 9];
    request.extend_from_slice(b"localhost");
    request.extend_from_slice(&address.port().to_be_bytes());
    client.write_all(&request).unwrap();
    assert_eq!(read_reply(&mut client).0, 0);
    client.write_all(b"GET / HTTP/1.1\r\n\r\n").unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_domain, 1);
    assert_eq!(report.observation.protocol.socks_dns_owned, 1);
    assert_eq!(report.observation.protocol.socks_http, 1);
}

#[test]
fn tls_prefix_is_classified_without_consuming_it() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let payload = [0x16, 0x03, 0x03, 0, 2, 1, 0];
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut received = [0_u8; 7];
        stream.read_exact(&mut received).unwrap();
        assert_eq!(received, payload);
    });
    let mut lease = start_proxy(address);
    let password = lease.capability_proof().proxy_password();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut client, password.as_bytes());
    connect_ipv4(&mut client, address);
    client.write_all(&payload).unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_tls, 1);
    assert_eq!(report.observation.protocol.socks_client_bytes, 7);
}

#[test]
fn no_auth_method_and_unsupported_command_are_finite_refusals() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    origin.set_nonblocking(true).unwrap();
    let mut lease = start_proxy(origin.local_addr().unwrap());
    let mut unauthenticated = TcpStream::connect(lease.endpoint()).unwrap();
    unauthenticated
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    unauthenticated.write_all(&[5, 1, 0]).unwrap();
    let mut refused = [0_u8; 2];
    unauthenticated.read_exact(&mut refused).unwrap();
    assert_eq!(refused, [5, 0xff]);

    let password = lease.capability_proof().proxy_password();
    let mut unsupported = TcpStream::connect(lease.endpoint()).unwrap();
    unsupported
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut unsupported, password.as_bytes());
    unsupported
        .write_all(&[5, 3, 0, 1, 127, 0, 0, 1, 0, 53])
        .unwrap();
    assert_eq!(read_reply(&mut unsupported).0, 7);
    assert!(origin.accept().is_err());
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.authentication_refused, 1);
    assert_eq!(report.observation.protocol.socks_connect_refused, 1);
}

#[test]
fn ipv6_connect_uses_ipv6_reply_when_loopback_is_available() {
    let Ok(origin) = TcpListener::bind("[::1]:0") else {
        return;
    };
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut value = [0_u8; 4];
        stream.read_exact(&mut value).unwrap();
        assert_eq!(&value, b"ipv6");
    });
    let mut lease = start_proxy(address);
    let password = lease.capability_proof().proxy_password();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut client, password.as_bytes());
    let SocketAddr::V6(destination) = address else {
        unreachable!();
    };
    let mut request = vec![5, 1, 0, 4];
    request.extend_from_slice(&destination.ip().octets());
    request.extend_from_slice(&destination.port().to_be_bytes());
    client.write_all(&request).unwrap();
    assert_eq!(read_reply(&mut client).0, 0);
    client.write_all(b"ipv6").unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_ipv6, 1);
}

#[test]
fn malformed_truncated_and_policy_refusals_are_finite_and_distinct() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    origin.set_nonblocking(true).unwrap();
    let limits = ProtocolLimits {
        header_timeout: Duration::from_millis(50),
        ..ProtocolLimits::default()
    };
    let mut lease = start_proxy_with(origin.local_addr().unwrap(), limits, None);

    let mut truncated = TcpStream::connect(lease.endpoint()).unwrap();
    truncated
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    truncated.write_all(&[5, 1]).unwrap();
    let mut terminal = [0_u8; 1];
    assert_eq!(truncated.read(&mut terminal).unwrap(), 0);

    let password = lease.capability_proof().proxy_password();
    let mut unsupported_address = TcpStream::connect(lease.endpoint()).unwrap();
    unsupported_address
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    authenticate(&mut unsupported_address, password.as_bytes());
    unsupported_address.write_all(&[5, 1, 0, 9]).unwrap();
    assert_eq!(read_reply(&mut unsupported_address).0, 8);

    let mut refused = TcpStream::connect(lease.endpoint()).unwrap();
    refused
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    authenticate(&mut refused, password.as_bytes());
    refused
        .write_all(&[5, 1, 0, 1, 127, 0, 0, 1, 0, 9])
        .unwrap();
    assert_eq!(read_reply(&mut refused).0, 5);
    assert!(origin.accept().is_err());

    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.authentication_refused, 0);
    assert_eq!(report.observation.protocol.parse_refused, 2);
    assert_eq!(report.observation.protocol.policy_refused, 1);
    assert_eq!(report.observation.protocol.timed_out, 1);
    assert_eq!(report.observation.protocol.socks_connect_refused, 2);
}

#[test]
fn cancellation_is_terminal_and_typed_events_share_connection_identity() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut byte = [0_u8; 1];
        let _ = stream.read(&mut byte);
    });
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy_with(address, ProtocolLimits::default(), Some(collector.clone()));
    let password = lease.capability_proof().proxy_password();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut client, password.as_bytes());
    connect_ipv4(&mut client, address);

    let report = lease.cleanup(Duration::from_secs(2));
    drop(client);
    server.join().unwrap();
    assert_eq!(
        report.observation.accepted_connections,
        report.observation.completed_connections
            + report.observation.failed_connections
            + report.observation.forced_connections
    );
    let events = collector.0.lock().unwrap();
    let connection_id = events
        .iter()
        .find(|event| matches!(event.kind, ApplicationEventKind::SocksNegotiation(_)))
        .unwrap()
        .connection_id;
    assert!(events.iter().any(|event| {
        event.connection_id == connection_id
            && matches!(event.kind, ApplicationEventKind::SocksConnect(_))
    }));
    assert!(events.iter().any(|event| {
        event.connection_id == connection_id
            && matches!(
                event.kind,
                ApplicationEventKind::ConnectionTerminal(StreamTerminal::Shutdown)
            )
    }));
}

#[test]
fn bounded_relay_preserves_a_payload_larger_than_its_buffers() {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = origin.local_addr().unwrap();
    let payload = vec![0x5a_u8; 128 * 1024];
    let expected = payload.clone();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = origin.accept().unwrap();
        let mut received = Vec::new();
        stream.read_to_end(&mut received).unwrap();
        assert_eq!(received, expected);
        stream.write_all(&received).unwrap();
    });
    let mut lease = start_proxy(address);
    let password = lease.capability_proof().proxy_password();
    let mut client = TcpStream::connect(lease.endpoint()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(&mut client, password.as_bytes());
    connect_ipv4(&mut client, address);
    client.write_all(&payload).unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut echoed = Vec::new();
    client.read_to_end(&mut echoed).unwrap();
    assert_eq!(echoed, payload);
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_client_bytes, 128 * 1024);
    assert_eq!(report.observation.protocol.socks_upstream_bytes, 128 * 1024);
}
