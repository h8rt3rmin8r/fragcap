// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "deep-capture")]

use std::io::Write;

use fragcap::deep_capture::{
    read_lifecycle_prefix, LifecycleStreamStatus, LifecycleWriter, ProxyLifecycleLease,
};
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

#[test]
fn a_torn_final_record_preserves_prior_lifecycle_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("cleanup.jsonl");
    let mut writer = LifecycleWriter::create(&path, "cleanup", "session").unwrap();
    writer
        .append("cleanup.obligation", json!({"resource_id": "proxy"}))
        .unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"{\"type\":\"cleanup.result\"")
        .unwrap();
    let prefix = read_lifecycle_prefix(&path).unwrap();
    assert_eq!(prefix.status, LifecycleStreamStatus::CrashPrefix);
    assert_eq!(prefix.records.len(), 1);
}

#[test]
fn listener_start_is_recorded_only_after_confirmation() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("proxy.jsonl");
    let mut lease =
        ProxyLifecycleLease::open_with_listener(&path, "session", 8, "127.0.0.1:41000").unwrap();
    lease.listener_failed("bind refused for test").unwrap();
    lease.finish().unwrap();
    let prefix = read_lifecycle_prefix(&path).unwrap();
    assert!(prefix
        .records
        .iter()
        .any(|record| record["type"] == "proxy.listener-attempt"));
    assert!(prefix
        .records
        .iter()
        .any(|record| record["type"] == "proxy.listener-failed"));
    assert!(!prefix
        .records
        .iter()
        .any(|record| record["type"] == "proxy.listener-started"));
}
