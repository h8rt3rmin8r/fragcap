// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::path::PathBuf;
use std::time::Duration;

use fragcap::deep_capture::{
    run_controlled_native_requests, ArtifactRequests, BackendDescriptor, Budget, CleanupStatus,
    Deadlines, LaunchCase, LoopbackEndpoint, NativeProxyAdapter, PlanId, PreparedTarget,
    ProxyBackend, RoutingPlan, SessionMode, SessionPlan,
};
use fragcap::targets::CompatibilityProtocol;

fn plan(session: &str) -> SessionPlan {
    let endpoint = LoopbackEndpoint::new("127.0.0.1:0".parse().unwrap()).unwrap();
    SessionPlan {
        id: PlanId::new(format!("plan-{session}")),
        session_id: session.to_string(),
        target: PreparedTarget {
            id: 1,
            handle: "controlled".to_string(),
            launch_case: LaunchCase::Controlled,
        },
        mode: SessionMode::TlsCalibration,
        calibration_protocol: Some(CompatibilityProtocol::Https),
        controlled: true,
        proxy_backend: BackendDescriptor {
            name: "fragcap-native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        endpoint,
        bundle: PathBuf::from("unused-controlled-bundle"),
        routing: RoutingPlan::child_environment(endpoint, &[]).unwrap(),
        trust_ca: true,
        client_identity: false,
        artifacts: ArtifactRequests {
            har: false,
            key_log: false,
            sensitive_retention: fragcap::deep_capture::SensitiveRetention::Retain,
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
        route.endpoint().address().to_string().parse().unwrap(),
        route.proxy_authorization(),
        http,
        https,
        route.ca_der().to_vec(),
        true,
    )
    .unwrap();

    // Stopping joins every connection worker and caches its terminal observation.
    // Sampling before that join races the worker's final HTTPS publication.
    assert_eq!(
        lease.stop(Budget::new(Duration::from_secs(2))).status,
        CleanupStatus::Released
    );
    let observations = lease
        .observations(Budget::new(Duration::from_secs(1)))
        .unwrap();
    let http = observations
        .iter()
        .find(|item| item.protocol == "http")
        .unwrap();
    assert_eq!(http.classification.family().as_str(), "http1");
    assert_eq!(http.classification.detection().as_str(), "identified");
    let https = observations
        .iter()
        .find(|item| item.protocol == "https" && item.inspectability.as_str() == "full")
        .unwrap();
    assert_eq!(https.classification.family().as_str(), "https");
    assert_eq!(https.classification.inspectability().as_str(), "full");
    assert!(lease
        .cleanup(Budget::new(Duration::from_secs(2)))
        .iter()
        .all(|result| result.status == CleanupStatus::Released));
}
