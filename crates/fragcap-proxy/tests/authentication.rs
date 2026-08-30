// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use fragcap_proxy::{NativeProxyBackend, NativeProxyConfig, SessionCapability};

#[test]
fn capability_is_random_redacted_and_exact() {
    let first = SessionCapability::generate().unwrap();
    let second = SessionCapability::generate().unwrap();
    assert_ne!(first.proof().as_bytes(), second.proof().as_bytes());
    assert!(first.authenticates(first.proof().as_bytes()));
    assert!(!first.authenticates(second.proof().as_bytes()));
    assert!(!format!("{first:?}").contains(&format!("{:?}", first.proof().as_bytes())));
}

#[test]
fn listener_refuses_wrong_proof_before_payload_and_counts_it() {
    let config = NativeProxyConfig::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        4,
        64,
        Duration::from_secs(1),
    )
    .unwrap();
    let mut backend = NativeProxyBackend::new(config);
    let mut lease = backend.start(Duration::from_secs(1)).unwrap();
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let mut wrong = TcpStream::connect(endpoint).unwrap();
    wrong.write_all(&[0_u8; 32]).unwrap();
    wrong.write_all(b"SECRET THAT MUST NOT BE READ").unwrap();
    drop(wrong);
    let mut right = TcpStream::connect(endpoint).unwrap();
    right
        .write_all(lease.capability_proof().as_bytes())
        .unwrap();
    right.write_all(b"synthetic").unwrap();
    drop(right);
    std::thread::sleep(Duration::from_millis(50));
    let report = lease.cleanup(Duration::from_secs(1));
    assert_eq!(report.observation.authentication_refused, 1);
    assert_eq!(report.observation.authenticated_connections, 1);
    assert_eq!(report.observation.accepted_connections, 2);
}

#[test]
fn port_reuse_gets_a_new_capability() {
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = reserved.local_addr().unwrap();
    drop(reserved);
    let config = || NativeProxyConfig::new(endpoint, 1, 64, Duration::from_secs(1)).unwrap();
    let mut first_backend = NativeProxyBackend::new(config());
    let mut first = first_backend.start(Duration::from_secs(1)).unwrap();
    let stale = first.capability_proof();
    assert!(first.cleanup(Duration::from_secs(1)).is_clean());
    let mut second_backend = NativeProxyBackend::new(config());
    let mut second = second_backend.start(Duration::from_secs(1)).unwrap();
    assert_ne!(stale.as_bytes(), second.capability_proof().as_bytes());
    assert!(second.cleanup(Duration::from_secs(1)).is_clean());
}
