// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{
    project_application_har, read_application_prefix, read_lifecycle_prefix, read_resource_journal,
    validate_v2, ApplicationArtifactLease, ApplicationStreamStatus, JournalStatus,
    LifecycleStreamStatus, LifecycleWriter, ResourceJournal, ResourceKind, ResourceState,
    ResourceTransition,
};
use serde_json::{json, Value};

#[test]
fn complete_bundle_authorities_reconcile_from_one_synthetic_session() {
    let directory = tempfile::tempdir().unwrap();
    let application_path = directory.path().join("application.jsonl");
    let mut application =
        ApplicationArtifactLease::open(&application_path, "s110-conformance", 8).unwrap();
    application.finish().unwrap();
    assert_eq!(
        read_application_prefix(&application_path).unwrap().status,
        ApplicationStreamStatus::Complete
    );
    let har = project_application_har(&application_path).unwrap();
    assert_eq!(har.standard_entries, 0);
    assert_eq!(har.partial_entries, 0);

    for stream in ["proxy", "cleanup"] {
        let path = directory.path().join(format!("{stream}.jsonl"));
        let mut writer = LifecycleWriter::create(&path, stream, "s110-conformance").unwrap();
        writer
            .append("conformance.observation", json!({"outcome":"pass"}))
            .unwrap();
        writer.finish().unwrap();
        assert_eq!(
            read_lifecycle_prefix(&path).unwrap().status,
            LifecycleStreamStatus::Complete
        );
    }

    let mut journal =
        ResourceJournal::create(directory.path(), "s110-conformance", "plan-s110").unwrap();
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
    assert_eq!(
        read_resource_journal(journal.path()).unwrap().status,
        JournalStatus::Complete
    );

    let manifest: Value = serde_json::from_str(include_str!(
        "../../../docs/schema/examples/deep-capture-manifest-v2-complete.json"
    ))
    .unwrap();
    validate_v2(&manifest).unwrap();
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
