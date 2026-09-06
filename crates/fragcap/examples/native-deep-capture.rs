// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::time::Duration;

use fragcap::deep_capture::api::{
    run_controlled_native_requests, ArtifactRequests, BackendDescriptor, Budget,
    CompatibilityProtocol, Deadlines, LaunchCase, LoopbackEndpoint, NativeListenerReservation,
    NativeProxyAdapter, PlanId, PreparedTarget, ProxyBackend, RoutingPlan, SensitiveRetention,
    SessionMode, SessionPlan,
};

fn main() -> Result<(), Box<dyn Error>> {
    let reservation = NativeListenerReservation::default();
    let endpoint = reservation.reserve("127.0.0.1:0".parse()?)?;
    let mut backend = NativeProxyAdapter::default().with_listener_reservation(reservation);
    let plan = controlled_plan(endpoint, backend.descriptor());
    let mut lease = backend.start(&plan, Budget::new(Duration::from_secs(10)))?;
    let route = lease.route()?;
    let (http_origin, https_origin) = route
        .controlled_origins()
        .ok_or("controlled origins were not created")?;
    run_controlled_native_requests(
        endpoint.address(),
        route.proxy_authorization(),
        http_origin,
        https_origin,
        route.ca_der().to_vec(),
        true,
    )?;
    let observations = lease.observations(Budget::new(Duration::from_secs(10)))?;
    if observations.len() < 2 {
        return Err("controlled native run retained fewer than two observations".into());
    }
    let listener = lease.stop(Budget::new(Duration::from_secs(10)));
    let cleanup = lease.cleanup(Budget::new(Duration::from_secs(10)));
    if !matches!(
        listener.status,
        fragcap::deep_capture::api::CleanupStatus::Released
    ) || cleanup.iter().any(|result| {
        matches!(
            result.status,
            fragcap::deep_capture::api::CleanupStatus::Failed
                | fragcap::deep_capture::api::CleanupStatus::TimedOut
        )
    }) {
        return Err(format!("native cleanup was incomplete: {listener:?}, {cleanup:?}").into());
    }
    println!(
        "native Deep Capture API example completed with {} observations and exact cleanup",
        observations.len()
    );
    Ok(())
}

fn controlled_plan(endpoint: LoopbackEndpoint, proxy_backend: BackendDescriptor) -> SessionPlan {
    SessionPlan {
        id: PlanId::new("native-example-plan"),
        session_id: "native-example-session".to_string(),
        target: PreparedTarget {
            id: 0,
            handle: "controlled-target".to_string(),
            launch_case: LaunchCase::Controlled,
        },
        mode: SessionMode::TlsCalibration,
        calibration_protocol: Some(CompatibilityProtocol::Https),
        controlled: true,
        proxy_backend,
        endpoint,
        bundle: std::env::temp_dir().join("fragcap-native-api-example"),
        routing: RoutingPlan::child_environment(endpoint, &[]).expect("controlled route"),
        trust_ca: false,
        client_identity: false,
        artifacts: ArtifactRequests {
            har: false,
            key_log: false,
            sensitive_retention: SensitiveRetention::Retain,
        },
        deadlines: Deadlines::default(),
    }
}
