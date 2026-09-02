// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, DestinationPolicy,
    EventDisposition, NativeProxyBackend, NativeProxyConfig, ProtocolLimits,
};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

fn start_proxy(
    grants: &[SocketAddr],
    limits: ProtocolLimits,
    collector: Arc<Collector>,
) -> fragcap_proxy::NativeProxyLease {
    let config = NativeProxyConfig::new(
        "127.0.0.1:0".parse().unwrap(),
        4,
        1024,
        Duration::from_secs(2),
    )
    .unwrap()
    .with_session_id("s115-socks-udp-test")
    .unwrap()
    .with_protocol_limits(limits);
    let mut policy = DestinationPolicy::new(config.listen());
    for grant in grants {
        policy.grant_for_test(*grant);
    }
    NativeProxyBackend::new(config)
        .with_destination_policy(policy)
        .with_application_event_sink(collector)
        .start(Duration::from_secs(2))
        .unwrap()
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

fn associate(client: &mut TcpStream, source: Option<SocketAddr>) -> SocketAddr {
    let source = source.unwrap_or_else(|| "0.0.0.0:0".parse().unwrap());
    let SocketAddr::V4(source) = source else {
        panic!("test client uses IPv4 loopback");
    };
    let mut request = vec![5, 3, 0, 1];
    request.extend_from_slice(&source.ip().octets());
    request.extend_from_slice(&source.port().to_be_bytes());
    client.write_all(&request).unwrap();
    read_reply(client)
}

fn read_reply(client: &mut TcpStream) -> SocketAddr {
    let mut head = [0_u8; 4];
    client.read_exact(&mut head).unwrap();
    assert_eq!(&head[..3], &[5, 0, 0]);
    match head[3] {
        1 => {
            let mut tail = [0_u8; 6];
            client.read_exact(&mut tail).unwrap();
            SocketAddr::from((
                [tail[0], tail[1], tail[2], tail[3]],
                u16::from_be_bytes([tail[4], tail[5]]),
            ))
        }
        kind => panic!("unexpected relay address type {kind}"),
    }
}

fn ipv4_frame(destination: SocketAddr, payload: &[u8]) -> Vec<u8> {
    let SocketAddr::V4(destination) = destination else {
        panic!("test requires IPv4 destination");
    };
    let mut frame = vec![0, 0, 0, 1];
    frame.extend_from_slice(&destination.ip().octets());
    frame.extend_from_slice(&destination.port().to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn domain_frame(domain: &str, port: u16, payload: &[u8]) -> Vec<u8> {
    let mut frame = vec![0, 0, 0, 3, domain.len().try_into().unwrap()];
    frame.extend_from_slice(domain.as_bytes());
    frame.extend_from_slice(&port.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn ipv6_frame(destination: SocketAddr, payload: &[u8]) -> Vec<u8> {
    let SocketAddr::V6(destination) = destination else {
        panic!("test requires IPv6 destination");
    };
    let mut frame = vec![0, 0, 0, 4];
    frame.extend_from_slice(&destination.ip().octets());
    frame.extend_from_slice(&destination.port().to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn response_payload(frame: &[u8]) -> &[u8] {
    assert_eq!(&frame[..4], &[0, 0, 0, 1]);
    &frame[10..]
}

#[test]
fn authenticated_association_relays_ipv4_and_proxy_resolved_domain() {
    let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
    origin
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for expected in [b"literal".as_slice(), b"domain".as_slice()] {
            let mut buffer = [0_u8; 64];
            let (read, source) = origin.recv_from(&mut buffer).unwrap();
            assert_eq!(&buffer[..read], expected);
            origin.send_to(expected, source).unwrap();
        }
    });
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(
        &[origin_address],
        ProtocolLimits::default(),
        Arc::clone(&collector),
    );
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let relay = associate(&mut control, Some(client.local_addr().unwrap()));

    client
        .send_to(&ipv4_frame(origin_address, b"literal"), relay)
        .unwrap();
    let mut response = [0_u8; 128];
    let (read, _) = client.recv_from(&mut response).unwrap();
    assert_eq!(response_payload(&response[..read]), b"literal");

    client
        .send_to(
            &domain_frame("localhost", origin_address.port(), b"domain"),
            relay,
        )
        .unwrap();
    let (read, _) = client.recv_from(&mut response).unwrap();
    assert_eq!(response_payload(&response[..read]), b"domain");
    drop(control);
    server.join().unwrap();

    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean());
    assert_eq!(report.observation.protocol.socks_udp_associate_succeeded, 1);
    assert_eq!(report.observation.protocol.socks_udp_client_forwarded, 2);
    assert_eq!(report.observation.protocol.socks_udp_upstream_forwarded, 2);
    assert_eq!(report.observation.protocol.socks_udp_peak_peers, 1);
    assert!(collector.0.lock().unwrap().iter().any(|event| matches!(
        &event.kind,
        ApplicationEventKind::SocksUdp(value)
            if value.action == "terminal" && value.active_peers == 0
    )));
}

#[test]
fn association_relays_ipv6_when_loopback_is_available() {
    let Ok(origin) = UdpSocket::bind("[::1]:0") else {
        return;
    };
    origin
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let origin_address = origin.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let mut buffer = [0_u8; 64];
        let (read, source) = origin.recv_from(&mut buffer).unwrap();
        assert_eq!(&buffer[..read], b"ipv6");
        origin.send_to(b"ipv6-reply", source).unwrap();
    });
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(&[origin_address], ProtocolLimits::default(), collector);
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let relay = associate(&mut control, Some(client.local_addr().unwrap()));
    client
        .send_to(&ipv6_frame(origin_address, b"ipv6"), relay)
        .unwrap();
    let mut response = [0_u8; 128];
    let (read, _) = client.recv_from(&mut response).unwrap();
    assert_eq!(&response[..4], &[0, 0, 0, 4]);
    assert_eq!(&response[22..read], b"ipv6-reply");
    drop(control);
    server.join().unwrap();
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_udp_client_forwarded, 1);
    assert_eq!(report.observation.protocol.socks_udp_upstream_forwarded, 1);
}

#[test]
fn peer_limit_refuses_new_mapping_without_evicting_existing_peer() {
    let first = UdpSocket::bind("127.0.0.1:0").unwrap();
    let second = UdpSocket::bind("127.0.0.1:0").unwrap();
    first
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    second
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let first_address = first.local_addr().unwrap();
    let second_address = second.local_addr().unwrap();
    let limits = ProtocolLimits {
        max_socks_udp_peers: 1,
        ..ProtocolLimits::default()
    };
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(&[first_address, second_address], limits, collector);
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let relay = associate(&mut control, Some(client.local_addr().unwrap()));
    client
        .send_to(&ipv4_frame(first_address, b"first"), relay)
        .unwrap();
    let mut buffer = [0_u8; 32];
    let (read, _) = first.recv_from(&mut buffer).unwrap();
    assert_eq!(&buffer[..read], b"first");
    client
        .send_to(&ipv4_frame(second_address, b"second"), relay)
        .unwrap();
    assert!(second.recv_from(&mut buffer).is_err());
    drop(control);
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_udp_peak_peers, 1);
    assert_eq!(report.observation.protocol.socks_udp_peer_limit_dropped, 1);
}

#[test]
fn declared_client_port_blocks_local_hijack() {
    let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
    origin
        .set_read_timeout(Some(Duration::from_millis(250)))
        .unwrap();
    let origin_address = origin.local_addr().unwrap();
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(&[origin_address], ProtocolLimits::default(), collector);
    let legitimate = UdpSocket::bind("127.0.0.1:0").unwrap();
    let attacker = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let relay = associate(&mut control, Some(legitimate.local_addr().unwrap()));
    attacker
        .send_to(&ipv4_frame(origin_address, b"hijack"), relay)
        .unwrap();
    let mut buffer = [0_u8; 64];
    assert!(origin.recv_from(&mut buffer).is_err());
    drop(control);
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_udp_source_dropped, 1);
    assert_eq!(report.observation.protocol.socks_udp_client_forwarded, 0);
}

#[test]
fn fragments_malformed_frames_and_refused_destinations_are_counted() {
    let allowed = UdpSocket::bind("127.0.0.1:0").unwrap();
    let collector = Arc::new(Collector::default());
    let limits = ProtocolLimits {
        max_socks_udp_datagram_bytes: 32,
        ..ProtocolLimits::default()
    };
    let mut lease = start_proxy(&[allowed.local_addr().unwrap()], limits, collector);
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let relay = associate(&mut control, Some(client.local_addr().unwrap()));
    client.send_to(&[0, 0, 0, 1], relay).unwrap();
    let mut fragment = ipv4_frame(allowed.local_addr().unwrap(), b"fragment");
    fragment[2] = 1;
    client.send_to(&fragment, relay).unwrap();
    client
        .send_to(
            &ipv4_frame("127.0.0.1:9".parse().unwrap(), b"refused"),
            relay,
        )
        .unwrap();
    client.send_to(&[0_u8; 33], relay).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    drop(control);
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_udp_malformed_dropped, 1);
    assert_eq!(report.observation.protocol.socks_udp_fragment_dropped, 1);
    assert_eq!(report.observation.protocol.socks_udp_policy_dropped, 1);
    assert_eq!(report.observation.protocol.socks_udp_oversized_dropped, 1);
}

#[test]
fn unsolicited_origin_cannot_reflect_to_client() {
    let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
    origin
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let attacker = UdpSocket::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(&[origin_address], ProtocolLimits::default(), collector);
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client
        .set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let relay = associate(&mut control, Some(client.local_addr().unwrap()));
    client
        .send_to(&ipv4_frame(origin_address, b"authorize"), relay)
        .unwrap();
    let mut buffer = [0_u8; 64];
    let (_, upstream_relay) = origin.recv_from(&mut buffer).unwrap();
    attacker.send_to(b"reflection", upstream_relay).unwrap();
    assert!(client.recv_from(&mut buffer).is_err());
    drop(control);
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_udp_unsolicited_dropped, 1);
}

#[test]
fn idle_timeout_releases_association() {
    let collector = Arc::new(Collector::default());
    let limits = ProtocolLimits {
        idle_timeout: Duration::from_millis(50),
        ..ProtocolLimits::default()
    };
    let mut lease = start_proxy(&[], limits, Arc::clone(&collector));
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let _relay = associate(&mut control, None);
    let mut byte = [0_u8; 1];
    assert_eq!(control.read(&mut byte).unwrap(), 0);
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.timed_out, 1);
    assert_eq!(report.observation.live_connections, 0);
    assert!(collector.0.lock().unwrap().iter().any(|event| matches!(
        &event.kind,
        ApplicationEventKind::SocksUdp(value)
            if value.action == "terminal" && value.outcome == "timed-out"
    )));
}

#[test]
fn rejected_domain_client_claim_is_charged_to_udp_associate() {
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(&[], ProtocolLimits::default(), collector);
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    control
        .write_all(&[
            5, 3, 0, 3, 9, b'l', b'o', b'c', b'a', b'l', b'h', b'o', b's', b't', 0, 0,
        ])
        .unwrap();
    let mut reply = [0_u8; 10];
    control.read_exact(&mut reply).unwrap();
    assert_eq!(&reply[..4], &[5, 8, 0, 1]);
    let report = lease.cleanup(Duration::from_secs(2));
    assert_eq!(report.observation.protocol.socks_udp_associate_requested, 1);
    assert_eq!(report.observation.protocol.socks_udp_associate_refused, 1);
    assert_eq!(report.observation.protocol.socks_connect_refused, 0);
}

#[test]
fn forced_cleanup_revokes_an_active_association_and_releases_its_relay() {
    let collector = Arc::new(Collector::default());
    let mut lease = start_proxy(&[], ProtocolLimits::default(), collector);
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut control = TcpStream::connect(lease.endpoint()).unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    authenticate(
        &mut control,
        lease.capability_proof().proxy_password().as_bytes(),
    );
    let relay = associate(&mut control, Some(client.local_addr().unwrap()));
    let report = lease.cleanup(Duration::from_secs(2));
    assert!(report.is_clean());
    assert_eq!(report.observation.live_connections, 0);
    assert!(UdpSocket::bind(relay).is_ok());
}
