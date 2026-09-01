// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::collections::BTreeSet;
use std::fs;

use fragcap::deep_capture::{
    project_application_har, read_application_prefix, read_lifecycle_prefix, read_resource_journal,
    validate_v2, ApplicationArtifactLease, ApplicationStreamStatus, JournalStatus,
    LifecycleStreamStatus, LifecycleWriter, ResourceJournal, ResourceKind, ResourceState,
    ResourceTransition,
};
use serde_json::{json, Value};

#[test]
fn complete_bundle_authorities_reconcile_from_one_synthetic_session() {
    const SESSION_ID: &str = "s110-conformance";
    let directory = tempfile::tempdir().unwrap();
    let application_path = directory.path().join("application.jsonl");
    let mut application = ApplicationArtifactLease::open(&application_path, SESSION_ID, 8).unwrap();
    application.finish().unwrap();
    let application_prefix = read_application_prefix(&application_path).unwrap();
    assert_eq!(application_prefix.status, ApplicationStreamStatus::Complete);
    assert!(application_prefix
        .records
        .iter()
        .all(|record| record["session_id"] == SESSION_ID));

    let har_path = directory.path().join("http.har");
    let har = project_application_har(&application_path)
        .unwrap()
        .publish(&har_path)
        .unwrap();
    assert_eq!(har.standard_entries, 0);
    assert_eq!(har.partial_entries, 0);

    for stream in ["proxy", "cleanup"] {
        let path = directory.path().join(format!("{stream}.jsonl"));
        let mut writer = LifecycleWriter::create(&path, stream, SESSION_ID).unwrap();
        writer
            .append("conformance.observation", json!({"outcome":"pass"}))
            .unwrap();
        writer.finish().unwrap();
        let prefix = read_lifecycle_prefix(&path).unwrap();
        assert_eq!(prefix.status, LifecycleStreamStatus::Complete);
        assert_eq!(prefix.session_id, SESSION_ID);
    }

    let mut journal = ResourceJournal::create(directory.path(), SESSION_ID, "plan-s110").unwrap();
    for state in [
        ResourceState::Pending,
        ResourceState::Applied,
        ResourceState::CleanupPending,
        ResourceState::Released,
    ] {
        journal
            .append(ResourceTransition::new(
                "synthetic-listener",
                ResourceKind::Proxy,
                "127.0.0.1:0",
                "session:s110-conformance",
                "close-owned-listener",
                state,
                "synthetic conformance transition",
            ))
            .unwrap();
    }
    journal.finish().unwrap();
    let journal_prefix = read_resource_journal(journal.path()).unwrap();
    assert_eq!(journal_prefix.status, JournalStatus::Complete);
    assert_eq!(journal_prefix.session_id, SESSION_ID);
    assert_eq!(journal_prefix.transitions.len(), 4);

    fs::write(
        directory.path().join("cleanup.json"),
        serde_json::to_vec_pretty(&json!({
            "session_id": SESSION_ID,
            "status": "succeeded",
            "released_resources": 1,
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.path().join("correlation.json"),
        serde_json::to_vec_pretty(&json!({
            "session_id": SESSION_ID,
            "application_records": application_prefix.records.len(),
            "connections": 0,
            "state": "complete",
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.path().join("capture.fcapng"),
        include_bytes!("../../../conformance/native-http-tls/analyzer.pcapng"),
    )
    .unwrap();
    fs::write(
        directory.path().join("tls-keylog.log"),
        include_bytes!("../../../conformance/native-http-tls/tls-keylog.log"),
    )
    .unwrap();

    let manifest = json!({
        "$schema": "https://fragcap.dev/schema/deep-capture-manifest.v2.json",
        "manifest_version": 2,
        "product": {"name":"fragcap","version":env!("CARGO_PKG_VERSION")},
        "session_id": SESSION_ID,
        "state": "complete",
        "artifacts": [
            artifact("application-jsonl", "application.jsonl", "primary-evidence", "application-events", None, "sensitive", "application/x-ndjson"),
            artifact("har", "http.har", "derived-projection", "http-projection", Some("application-jsonl"), "sensitive", "application/json"),
            artifact("tls-key-log", "tls-keylog.log", "analyzer-aid", "analyzer-aid", None, "secret-adjacent", "text/plain"),
            artifact("pcapng", "capture.fcapng", "primary-evidence", "packet-truth", None, "ordinary", "application/x-pcapng"),
            artifact("correlation", "correlation.json", "derived-projection", "correlation-summary", Some("application-jsonl"), "ordinary", "application/json"),
            artifact("proxy-lifecycle", "proxy.jsonl", "primary-evidence", "proxy-lifecycle-events", None, "sensitive", "application/x-ndjson"),
            artifact("cleanup-lifecycle", "cleanup.jsonl", "operational-record", "cleanup-lifecycle-events", None, "ordinary", "application/x-ndjson"),
            artifact("cleanup-summary", "cleanup.json", "derived-projection", "cleanup-projection", Some("cleanup-lifecycle"), "ordinary", "application/json"),
            artifact("resource-journal", "resource-journal.jsonl", "operational-record", "resource-ownership-journal", None, "secret-adjacent", "application/x-ndjson"),
            artifact("manifest-v2", "manifest.json", "bundle-index", "bundle-index", None, "ordinary", "application/json")
        ],
        "omissions": [],
    });
    let manifest_path = directory.path().join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
    validate_v2(&manifest).unwrap();

    let roles = manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["role"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(roles.len(), 10);
    for artifact in manifest["artifacts"].as_array().unwrap() {
        let path = directory.path().join(artifact["path"].as_str().unwrap());
        assert!(path.is_file(), "missing {}", path.display());
        assert!(
            path.metadata().unwrap().len() > 0,
            "empty {}",
            path.display()
        );
    }
    let cleanup: Value =
        serde_json::from_slice(&fs::read(directory.path().join("cleanup.json")).unwrap()).unwrap();
    let correlation: Value =
        serde_json::from_slice(&fs::read(directory.path().join("correlation.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["session_id"], cleanup["session_id"]);
    assert_eq!(manifest["session_id"], correlation["session_id"]);
    assert_eq!(
        correlation["application_records"],
        application_prefix.records.len()
    );
}

fn artifact(
    role: &str,
    path: &str,
    kind: &str,
    owner: &str,
    source_role: Option<&str>,
    sensitivity: &str,
    content_type: &str,
) -> Value {
    json!({
        "role": role,
        "path": path,
        "authority": {"kind":kind,"owner":owner,"source_role":source_role},
        "sensitivity": sensitivity,
        "content_type": content_type,
        "required": true,
        "finalization": "complete",
        "completeness": "complete",
        "loss": {"state":"none"},
        "correlation": if matches!(role, "application-jsonl" | "har" | "pcapng" | "correlation") {
            json!({"state":"complete","records":0})
        } else {
            json!({"state":"not-applicable"})
        },
    })
}

#[test]
fn committed_conformance_report_has_no_required_skip_state() {
    let report: Value = serde_json::from_str(include_str!(
        "../../../conformance/native-http-tls/report-v1.json"
    ))
    .unwrap();
    assert_eq!(report["summary"]["required"], report["summary"]["passed"]);
    for field in ["failed", "skipped", "not_run", "missing", "duplicate"] {
        assert_eq!(report["summary"][field], 0, "{field}");
    }
    assert!(report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row["status"] == "pass"));
}
