// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::path::PathBuf;
use std::time::Duration;

use fragcap::deep_capture::{
    ArtifactRequests, BackendDescriptor, Budget, Deadlines, LaunchCase, LoopbackEndpoint,
    NativeProxyAdapter, PlanId, PreparedTarget, ProxyBackend, SessionMode, SessionPlan,
};

fn plan(port: u16) -> SessionPlan {
    SessionPlan {
        id: PlanId::new("plan-native-foundation"),
        session_id: "session-native-foundation".to_string(),
        target: PreparedTarget {
            id: 1,
            handle: "controlled".to_string(),
            launch_case: LaunchCase::Controlled,
        },
        mode: SessionMode::Capture,
        controlled: true,
        proxy_backend: BackendDescriptor {
            name: "fragcap-native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        endpoint: LoopbackEndpoint { port },
        bundle: PathBuf::from("controlled-native-bundle"),
        trust_ca: false,
        artifacts: ArtifactRequests {
            har: false,
            key_log: false,
        },
        deadlines: Deadlines::default(),
    }
}

#[test]
fn rust_consumer_starts_and_cleans_native_backend_without_cli() {
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reserved.local_addr().unwrap().port();
    drop(reserved);

    let mut backend = NativeProxyAdapter::default();
    assert_eq!(backend.descriptor().name, "fragcap-native");
    let mut lease = backend
        .start(&plan(port), Budget::new(Duration::from_secs(1)))
        .expect("native backend starts");
    assert!(lease
        .observations(Budget::new(Duration::from_secs(1)))
        .expect("native observation succeeds")
        .is_empty());
    let stopped = lease.stop(Budget::new(Duration::from_secs(1)));
    let cleanup = lease.cleanup(Budget::new(Duration::from_secs(1)));
    assert!(matches!(
        stopped.status,
        fragcap::deep_capture::CleanupStatus::Released
    ));
    assert!(cleanup.iter().all(|result| matches!(
        result.status,
        fragcap::deep_capture::CleanupStatus::Released
    )));
}
