// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use fragcap::deep_capture::{read_lifecycle_prefix, LifecycleStreamStatus, LifecycleWriter};
use serde_json::json;

#[test]
fn crash_prefix_and_reconciling_trailer_are_distinct() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("cleanup.jsonl");
    let mut writer = LifecycleWriter::create(&path, "cleanup", "session").unwrap();
    writer
        .append("cleanup.obligation", json!({"resource_id": "proxy"}))
        .unwrap();
    assert_eq!(
        read_lifecycle_prefix(&path).unwrap().status,
        LifecycleStreamStatus::CrashPrefix
    );
    writer.finish().unwrap();
    let prefix = read_lifecycle_prefix(&path).unwrap();
    assert_eq!(prefix.status, LifecycleStreamStatus::Complete);
    assert_eq!(prefix.records.len(), 1);
}

#[test]
fn unavailable_evidence_is_a_typed_gap() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("proxy.jsonl");
    let mut writer = LifecycleWriter::create(&path, "proxy", "session").unwrap();
    writer.gap("dns.success", "not exposed", None).unwrap();
    writer.finish().unwrap();
    let prefix = read_lifecycle_prefix(&path).unwrap();
    assert_eq!(prefix.records[0]["type"], "lifecycle.gap");
}
