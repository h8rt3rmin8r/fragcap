// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fragcap_proxy::{
    ApplicationEvent, ApplicationEventKind, ApplicationEventSink, EventDisposition,
    NativeProxyBackend, NativeProxyConfig, ProxyAuthorizationError, SessionCapability,
};

#[derive(Default)]
struct Collector(Mutex<Vec<ApplicationEvent>>);

impl ApplicationEventSink for Collector {
    fn try_emit(&self, event: ApplicationEvent) -> EventDisposition {
        self.0.lock().unwrap().push(event);
        EventDisposition::Accepted
    }
}

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
fn capability_uses_strict_standard_proxy_authorization() {
    let capability = SessionCapability::generate().unwrap();
    let proof = capability.proof();
    let authorization = proof.proxy_authorization();
    assert!(authorization.starts_with("Basic "));
    assert_eq!(
        capability.authenticates_proxy_authorization(Some(authorization.as_bytes())),
        Ok(())
    );
    assert_eq!(
        capability.authenticates_proxy_authorization(None),
        Err(ProxyAuthorizationError::Missing)
    );
    assert_eq!(
        capability.authenticates_proxy_authorization(Some(b"Bearer nope")),
        Err(ProxyAuthorizationError::Malformed)
    );
    assert_eq!(
        capability.authenticates_proxy_authorization(Some(b"Basic bm9wZQ==")),
        Err(ProxyAuthorizationError::Malformed)
    );
    let other = SessionCapability::generate()
        .unwrap()
        .proof()
        .proxy_authorization();
    assert_eq!(
        capability.authenticates_proxy_authorization(Some(other.as_bytes())),
        Err(ProxyAuthorizationError::Refused)
    );
    assert!(!format!("{proof:?}").contains(proof.proxy_password().as_str()));
}

#[test]
fn capability_uses_the_same_secret_for_a_redacted_socks5h_route() {
    let capability = SessionCapability::generate().unwrap();
    let proof = capability.proof();
    let password = proof.proxy_password();
    let other_password = SessionCapability::generate()
        .unwrap()
        .proof()
        .proxy_password();
    assert!(capability.authenticates_socks_credentials(b"fragcap", password.as_bytes()));
    assert!(!capability.authenticates_socks_credentials(b"other", password.as_bytes()));
    assert!(!capability.authenticates_socks_credentials(b"fragcap", other_password.as_bytes()));
    let url = proof.socks5h_url("127.0.0.1:3210".parse().unwrap());
    assert!(url.starts_with("socks5h://fragcap:"));
    assert!(url.ends_with("@127.0.0.1:3210"));
    assert!(!format!("{proof:?}").contains(password.as_str()));
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
    let collector = Arc::new(Collector::default());
    let mut backend =
        NativeProxyBackend::new(config).with_application_event_sink(collector.clone());
    let mut lease = backend.start(Duration::from_secs(1)).unwrap();
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let mut wrong = TcpStream::connect(endpoint).unwrap();
    wrong
        .write_all(
            b"POST http://127.0.0.1/ HTTP/1.1\r\nHost: 127.0.0.1\r\nProxy-Authorization: Basic ZnJhZ2NhcDp3cm9uZw==\r\nContent-Length: 28\r\n\r\nSECRET THAT MUST NOT BE READ",
        )
        .unwrap();
    drop(wrong);
    let mut right = TcpStream::connect(endpoint).unwrap();
    let authorization = lease.capability_proof().proxy_authorization();
    write!(
        right,
        "GET http://127.0.0.1/ HTTP/1.1\r\nHost: 127.0.0.1\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n",
        authorization.as_str()
    )
    .unwrap();
    drop(right);
    std::thread::sleep(Duration::from_millis(50));
    let report = lease.cleanup(Duration::from_secs(1));
    assert_eq!(report.observation.authentication_refused, 1);
    assert_eq!(report.observation.authenticated_connections, 1);
    assert_eq!(report.observation.accepted_connections, 2);
    let events = collector.0.lock().unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event.kind, ApplicationEventKind::ConnectionOpen(_)))
            .count(),
        2,
        "every accepted connection needs a correlation descriptor"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event.kind, ApplicationEventKind::ConnectionTerminal(_)))
            .count(),
        2
    );
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
