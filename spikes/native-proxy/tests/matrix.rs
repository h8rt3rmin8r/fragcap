// SPDX-License-Identifier: Apache-2.0

use fragcap_native_proxy_spike::{
    evidence::{BackendRun, Comparison, Observation, Status},
    run_baseline, run_candidate,
};

#[test]
fn negative_states_never_become_parity() {
    let candidate = run_with(Observation::result(
        "https-http2",
        "proxy-request",
        Status::Unsupported,
        "unsupported",
    ));
    let baseline = run_with(Observation::complete(
        "https-http2",
        "proxy-request",
        Some("HTTP/2.0"),
        b"body",
    ));
    let comparison = Comparison::new(candidate, baseline);
    assert_eq!(comparison.rows.len(), 1);
    assert!(!comparison.rows[0].parity);
}

#[test]
fn missing_rows_are_explicitly_not_measured() {
    let candidate = run_with(Observation::complete(
        "http1",
        "proxy-request",
        Some("HTTP/1.1"),
        b"body",
    ));
    let baseline = run_with(Observation::result(
        "matrix",
        "backend-run",
        Status::Failed,
        "missing backend",
    ));
    let comparison = Comparison::new(candidate, baseline);
    let row = comparison
        .rows
        .iter()
        .find(|row| row.scenario == "http1")
        .expect("candidate-only row");
    assert_eq!(row.baseline, Status::NotMeasured);
    assert!(!row.parity);
}

#[tokio::test]
async fn candidate_proves_protocol_fidelity_key_logging_and_cleanup() {
    let run = run_candidate().await;
    assert!(run.loopback_only);
    assert!(!run.trust_store_mutated);
    assert_eq!(run.cache_capacity, Some(32));
    assert!(run.key_log_lines > 0, "client-facing TLS keys are logged");
    assert_eq!(run.shutdown_trials.len(), 10);
    assert!(run
        .shutdown_trials
        .iter()
        .all(|status| *status == Status::Complete));
    assert!(run.observations.iter().any(|row| {
        row.scenario == "lifecycle"
            && row.kind == "active-connection-shutdown"
            && row.status == Status::Complete
    }));
    for scenario in ["http1", "https-http1", "https-http2", "websocket"] {
        assert!(
            run.observations
                .iter()
                .any(|row| row.scenario == scenario && row.status == Status::Complete),
            "missing complete observation for {scenario}"
        );
    }
    assert!(run.observations.iter().any(|row| {
        row.scenario == "https-http2"
            && row.kind == "proxy-request"
            && row.protocol.as_deref() == Some("HTTP/2.0")
    }));
    assert!(run.observations.iter().any(|row| {
        row.scenario == "matrix" && row.kind == "har-source" && row.status == Status::Complete
    }));
    let directions: Vec<_> = run
        .observations
        .iter()
        .filter(|row| row.kind == "proxy-message")
        .filter_map(|row| row.direction.as_deref())
        .collect();
    assert!(directions.contains(&"client-to-server"));
    assert!(directions.contains(&"server-to-client"));
}

#[tokio::test]
async fn baseline_is_bounded_when_installed() {
    let run = run_baseline().await;
    if run.version == "unavailable" {
        return;
    }
    assert!(run.loopback_only);
    assert!(!run.trust_store_mutated);
    assert!(!run.shutdown_trials.is_empty());
    assert!(run.observations.iter().any(|row| {
        row.scenario == "http1" && row.kind == "proxy-request" && row.status == Status::Complete
    }));
}

fn run_with(observation: Observation) -> BackendRun {
    BackendRun {
        backend: "test".to_string(),
        version: "0".to_string(),
        platform: "windows-x86_64".to_string(),
        loopback_only: true,
        trust_store_mutated: false,
        cache_capacity: None,
        key_log_lines: 0,
        shutdown_trials: Vec::new(),
        observations: vec![observation],
        limitations: Vec::new(),
    }
}
