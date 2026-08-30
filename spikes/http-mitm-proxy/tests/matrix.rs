// SPDX-License-Identifier: Apache-2.0

use fragcap_http_mitm_proxy_spike::{
    evidence::{BackendRun, Observation, Status, compare},
    run_candidate,
};

#[test]
fn missing_and_negative_rows_never_become_parity() {
    let left = run_with(vec![Observation::result(
        "https-http2",
        "proxy-request",
        Status::Unsupported,
        "unsupported",
    )]);
    let right = run_with(vec![Observation::complete(
        "https-http2",
        "proxy-request",
        Some("HTTP/2.0"),
        b"body",
    )]);
    let rows = compare(&left, &right);
    let row = rows
        .iter()
        .find(|row| row.scenario == "https-http2" && row.kind == "proxy-request")
        .expect("required row");
    assert_eq!(row.left, Status::Unsupported);
    assert_eq!(row.right, Status::Complete);
    assert!(!row.parity);
    let shared_missing = rows
        .iter()
        .find(|row| row.scenario == "https-http1" && row.kind == "proxy-response")
        .expect("seeded row");
    assert_eq!(shared_missing.left, Status::NotMeasured);
    assert_eq!(shared_missing.right, Status::NotMeasured);
    assert!(!shared_missing.parity);
}

#[test]
fn parity_requires_complete_protocol_length_and_digest() {
    let base = Observation::complete("http1", "proxy-response", Some("HTTP/1.1"), b"body");
    let mut wrong_protocol = base.clone();
    wrong_protocol.protocol = Some("HTTP/2.0".into());
    let mut wrong_length = base.clone();
    wrong_length.byte_length += 1;
    let wrong_digest = Observation::complete("http1", "proxy-response", Some("HTTP/1.1"), b"copy");
    for candidate in [wrong_protocol, wrong_length, wrong_digest] {
        let rows = compare(&run_with(vec![candidate]), &run_with(vec![base.clone()]));
        assert!(
            !rows
                .iter()
                .find(|row| row.scenario == "http1" && row.kind == "proxy-response")
                .expect("row")
                .parity
        );
    }
    let rows = compare(&run_with(vec![base.clone()]), &run_with(vec![base]));
    assert!(
        rows.iter()
            .find(|row| row.scenario == "http1" && row.kind == "proxy-response")
            .expect("row")
            .parity
    );
}

#[tokio::test]
async fn candidate_records_fidelity_and_hard_limitations() {
    let run = run_candidate().await;
    assert!(run.loopback_only);
    assert!(!run.trust_store_mutated);
    assert_eq!(run.cache_capacity, Some(32));
    assert_eq!(run.key_log_lines, 0);
    assert_eq!(run.shutdown_trials.len(), 10);
    assert!(
        run.shutdown_trials
            .iter()
            .all(|status| *status == Status::Complete)
    );
    for scenario in ["http1", "https-http1", "https-http2", "websocket"] {
        assert!(
            run.observations
                .iter()
                .any(|row| row.scenario == scenario && row.status == Status::Complete),
            "missing complete result for {scenario}"
        );
    }
    assert!(
        run.observations
            .iter()
            .any(|row| row.scenario == "https-http2"
                && row.kind == "proxy-request"
                && row.protocol.as_deref() == Some("HTTP/2.0"))
    );
    assert!(run.observations.iter().any(|row| row.scenario == "matrix"
        && row.kind == "har-source"
        && row.status == Status::Complete));
    assert!(run.observations.iter().any(|row| row.scenario == "matrix"
        && row.kind == "client-facing-key-log"
        && row.status == Status::Unsupported));
    assert!(
        run.observations
            .iter()
            .any(|row| row.scenario == "lifecycle"
                && row.kind == "active-connection-shutdown"
                && row.status == Status::Unsupported)
    );
}

fn run_with(observations: Vec<Observation>) -> BackendRun {
    BackendRun {
        backend: "test".into(),
        version: "0".into(),
        platform: "windows-x86_64".into(),
        loopback_only: true,
        trust_store_mutated: false,
        cache_capacity: None,
        key_log_lines: 0,
        shutdown_trials: Vec::new(),
        observations,
        limitations: Vec::new(),
    }
}
