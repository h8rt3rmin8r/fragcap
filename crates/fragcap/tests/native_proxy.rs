// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::path::PathBuf;
use std::time::Duration;

use fragcap::deep_capture::{
    run_controlled_native_requests, ArtifactRequests, BackendDescriptor, Budget, CleanupStatus,
    Deadlines, LaunchCase, LoopbackEndpoint, NativeProxyAdapter, PlanId, PreparedTarget,
    ProxyBackend, SessionMode, SessionPlan,
};

fn plan(session: &str) -> SessionPlan {
    SessionPlan {
        id: PlanId::new(format!("plan-{session}")),
        session_id: session.to_string(),
        target: PreparedTarget {
            id: 1,
            handle: "controlled".to_string(),
            launch_case: LaunchCase::Controlled,
        },
        mode: SessionMode::TlsCalibration,
        controlled: true,
        proxy_backend: BackendDescriptor {
            name: "fragcap-native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        endpoint: LoopbackEndpoint { port: 0 },
        bundle: PathBuf::from("unused-controlled-bundle"),
        trust_ca: true,
        artifacts: ArtifactRequests {
            har: false,
            key_log: false,
        },
        deadlines: Deadlines::default(),
    }
}

#[test]
fn public_native_adapter_runs_real_controlled_http_and_tls_without_leaking_route_secrets() {
    let mut adapter = NativeProxyAdapter::default();
    let mut lease = adapter
        .start(&plan("s104-facade"), Budget::new(Duration::from_secs(2)))
        .unwrap();
    let route = lease.route().unwrap();
    let debug = format!("{route:?}");
    assert!(!debug.contains(route.proxy_authorization()));
    assert!(!debug.contains(route.proxy_url()));
    let (http, https) = route.controlled_origins().unwrap();
    run_controlled_native_requests(
        format!("127.0.0.1:{}", route.endpoint().port)
            .parse()
            .unwrap(),
        route.proxy_authorization(),
        http,
        https,
        route.ca_der().to_vec(),
        true,
    )
    .unwrap();

    let observations = lease
        .observations(Budget::new(Duration::from_secs(1)))
        .unwrap();
    assert!(observations.iter().any(|item| item.protocol == "http"));
    assert!(observations.iter().any(|item| item.protocol == "https"));
    assert_eq!(
        lease.stop(Budget::new(Duration::from_secs(2))).status,
        CleanupStatus::Released
    );
    assert!(lease
        .cleanup(Budget::new(Duration::from_secs(2)))
        .iter()
        .all(|result| result.status == CleanupStatus::Released));
}
