// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use fragcap_proxy::{BackendKind, LifecycleState, NativeProxyBackend, NativeProxyConfig};

fn config(port: u16, max_connections: usize) -> NativeProxyConfig {
    NativeProxyConfig::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        max_connections,
        1024,
        Duration::from_millis(500),
    )
    .expect("valid loopback config")
}

#[test]
fn configuration_refuses_non_loopback_and_zero_bounds() {
    let non_loopback = NativeProxyConfig::new(
        "192.0.2.1:8080".parse().unwrap(),
        1,
        1,
        Duration::from_secs(1),
    );
    assert!(non_loopback.is_err());
    assert!(
        NativeProxyConfig::new("127.0.0.1:0".parse().unwrap(), 0, 1, Duration::from_secs(1))
            .is_err()
    );
    assert!(
        NativeProxyConfig::new("127.0.0.1:0".parse().unwrap(), 1, 0, Duration::from_secs(1))
            .is_err()
    );
    assert!(NativeProxyConfig::new("127.0.0.1:0".parse().unwrap(), 1, 1, Duration::ZERO).is_err());
}

#[test]
fn start_refuses_an_exhausted_budget_and_an_occupied_endpoint() {
    let mut zero_budget_backend = NativeProxyBackend::new(config(0, 1));
    let zero_budget = match zero_budget_backend.start(Duration::ZERO) {
        Ok(_) => panic!("zero start budget must fail"),
        Err(error) => error,
    };
    assert_eq!(zero_budget.code, "start-budget-exhausted");

    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = reserved.local_addr().unwrap();
    let mut occupied_backend = NativeProxyBackend::new(config(endpoint.port(), 1));
    let occupied = match occupied_backend.start(Duration::from_secs(1)) {
        Ok(_) => panic!("occupied endpoint must fail"),
        Err(error) => error,
    };
    assert_eq!(occupied.code, "listener-bind-failed");
}

#[test]
fn startup_consumes_the_exact_listener_reserved_before_authorization() {
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = reserved.local_addr().unwrap();
    let mut backend =
        NativeProxyBackend::new(config(endpoint.port(), 1)).with_reserved_listener(reserved);
    let mut lease = backend.start(Duration::from_secs(1)).unwrap();

    assert_eq!(lease.endpoint(), endpoint);
    assert!(lease.cleanup(Duration::from_secs(1)).is_clean());
}

#[test]
fn identity_is_stable_and_claims_native_http_tls() {
    let backend = NativeProxyBackend::new(config(0, 1));
    let identity = backend.identity();
    assert_eq!(identity.kind, BackendKind::NativeRust);
    assert_eq!(identity.name, "fragcap-native");
    assert_eq!(identity.version, env!("CARGO_PKG_VERSION"));
    assert!(identity.capabilities.foundation_listener);
    assert!(identity.capabilities.forwards_upstream);
    assert!(identity.capabilities.observes_http);
    assert!(identity.capabilities.inspects_tls);
}

#[test]
fn start_stop_and_cleanup_are_owned_and_idempotent() {
    let mut backend = NativeProxyBackend::new(config(0, 2));
    let mut lease = backend.start(Duration::from_secs(1)).expect("start");
    let running = lease.observation(Duration::from_secs(1)).expect("observe");
    assert_eq!(running.state, LifecycleState::Running);
    assert!(running.endpoint.ip().is_loopback());

    let first = lease.stop(Duration::from_secs(1));
    let second = lease.stop(Duration::from_secs(1));
    let cleanup = lease.cleanup(Duration::from_secs(1));
    assert_eq!(first, second);
    assert_eq!(second, cleanup);
    assert!(cleanup.is_clean());
    assert_eq!(cleanup.observation.state, LifecycleState::Stopped);
}

#[test]
fn saturation_is_bounded_and_conserved() {
    let mut backend = NativeProxyBackend::new(config(0, 1));
    let mut lease = backend.start(Duration::from_secs(1)).expect("start");
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;
    let first = TcpStream::connect(endpoint).expect("first connection");
    let second = TcpStream::connect(endpoint).expect("second connection");
    std::thread::sleep(Duration::from_millis(100));

    let observation = lease.observation(Duration::from_secs(1)).unwrap();
    assert_eq!(observation.peak_live_connections, 1);
    assert!(observation.saturated_connections >= 1);
    assert!(observation.live_connections <= 1);
    drop(first);
    drop(second);

    let report = lease.cleanup(Duration::from_secs(1));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        report.observation.accepted_connections,
        report.observation.completed_connections
            + report.observation.failed_connections
            + report.observation.forced_connections
    );
}

#[test]
fn completed_tasks_are_reaped_before_more_connections_are_admitted() {
    let mut backend = NativeProxyBackend::new(config(0, 2));
    let mut lease = backend.start(Duration::from_secs(1)).expect("start");
    let endpoint = lease.observation(Duration::from_secs(1)).unwrap().endpoint;

    for _ in 0..256 {
        let mut client = TcpStream::connect(endpoint).expect("connect queued client");
        let _ = client.write_all(&[1]);
    }
    std::thread::sleep(Duration::from_millis(100));

    let report = lease.cleanup(Duration::from_secs(1));
    assert!(report.is_clean(), "{report:?}");
    assert!(report.observation.accepted_connections > 2);
    assert!(report.observation.peak_live_connections <= 2);
    assert_eq!(
        report.observation.accepted_connections,
        report.observation.completed_connections
            + report.observation.failed_connections
            + report.observation.forced_connections
    );
}

#[test]
fn ten_cycles_leave_no_listener_or_task_residue() {
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = reserved.local_addr().unwrap();
    drop(reserved);

    for _ in 0..10 {
        let mut backend = NativeProxyBackend::new(config(endpoint.port(), 2));
        let mut lease = backend.start(Duration::from_secs(1)).expect("start cycle");
        let client = TcpStream::connect(endpoint).expect("connect cycle");
        drop(client);
        let report = lease.cleanup(Duration::from_secs(1));
        assert!(report.is_clean(), "{report:?}");
    }
}
