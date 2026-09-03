// SPDX-License-Identifier: Apache-2.0

//! Deep Capture CLI contract tests.
//!
//! These tests use only placeholder targets and scratch stores. They do not start
//! real games, read local install paths, or require an installed proxy backend.

mod common;

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use common::run;
use fragcap::profile::FidelityTier;
use fragcap::targets::{
    resolved_client_launch, ClassificationSource, CompatibilityEvidenceSource, CompatibilityFact,
    CompatibilityFactKey, CompatibilityLaunchCase, Selection, Store, TargetClassification,
    TargetEntry,
};

fn controlled_environment() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn seed_target(local: &Path, with_compatibility: bool) -> i64 {
    let mut store = Store::open(local).expect("scratch local store");
    let entry = TargetEntry {
        id: None,
        stable_id: 75_000,
        handle: "sample-target".to_string(),
        name: "Sample Target".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: None,
        launch_entries: Some(resolved_client_launch("client.exe")),
        install_root: None,
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: Some("client.exe".to_string()),
    };
    let id = store.insert_target(&entry).expect("insert target");
    if with_compatibility {
        for (key, value) in [
            (CompatibilityFactKey::ProxyRouting, "reached-client"),
            (CompatibilityFactKey::ProxyPropagation, "confirmed"),
        ] {
            let fact =
                CompatibilityFact::new(id, key, value, CompatibilityEvidenceSource::UserConfirmed)
                    .expect("compatibility fact");
            let mut fact = fact;
            fact.launch_case = Some(CompatibilityLaunchCase::DirectExeWarm);
            store
                .insert_compatibility_fact(&fact)
                .expect("insert compatibility fact");
        }
    }
    id
}

#[test]
fn deep_capture_is_listed_on_the_root_help() {
    let (code, out, _err) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("deep-capture"), "root help:\n{out}");
}

#[test]
fn deep_capture_help_exposes_the_operator_contract() {
    let (code, out, err) = run(&["deep-capture", "--help"]);
    assert_eq!(code, 0, "stderr:\n{err}");
    for required in [
        "--launch",
        "--bundle",
        "--duration",
        "--wait",
        "--max-packets",
        "--max-bytes",
        "--interface",
        "--no-payload",
        "--trust-ca",
        "--restart-warm",
        "--calibrate",
        "--launch-case",
        "--har",
        "--key-log",
        "--client-certificate",
        "--client-private-key",
    ] {
        assert!(
            out.contains(required),
            "help must contain {required}:\n{out}"
        );
    }
    assert!(!out.contains("--controlled-target"));
    assert!(!out.contains("--proxy-backend"));
}

#[test]
fn warm_restart_cannot_be_combined_with_calibration() {
    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--restart-warm",
        "--calibrate",
        "reachability",
        "--launch-case",
        "direct-exe-warm",
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("cannot be used with"),
        "conflict refusal: {err}"
    );
}

#[test]
fn calibration_arguments_are_paired() {
    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--calibrate",
        "reachability",
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("--launch-case"), "pairing refusal: {err}");

    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--launch-case",
        "direct-exe-warm",
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("--calibrate"), "pairing refusal: {err}");
}

#[test]
fn client_identity_arguments_are_paired() {
    for (provided, required) in [
        ("--client-certificate", "--client-private-key"),
        ("--client-private-key", "--client-certificate"),
    ] {
        let (code, _out, err) = run(&[
            "deep-capture",
            "sample-target",
            "--launch",
            provided,
            "identity.pem",
        ]);
        assert_eq!(code, 2);
        assert!(err.contains(required), "pairing refusal: {err}");
    }
}

#[test]
fn controlled_calibration_runs_reachability_then_tls() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let target_id = seed_target(&local, false);
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let reachability_bundle = dir.path().join("reachability");
    let (code, out, reachability_events) = run(&[
        "--json",
        "deep-capture",
        "sample-target",
        "--launch",
        "--calibrate",
        "reachability",
        "--launch-case",
        "direct-exe-warm",
        "--duration",
        "5s",
        "--wait",
        "7s",
        "--yes",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        reachability_bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "reachability events:\n{reachability_events}");
    assert!(out.is_empty());
    assert!(reachability_events.contains("deep_capture.calibration_plan"));
    assert!(reachability_events.contains("\"status\":\"reached-client\""));
    assert!(!reachability_events.contains("deep_capture.trust"));
    let plan: serde_json::Value = reachability_events
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .find(|event: &serde_json::Value| event["event"] == "deep_capture.calibration_plan")
        .unwrap();
    assert_eq!(plan["launch_timeout_secs"], 7);
    assert_eq!(plan["observation_timeout_secs"], 5);
    let reachability_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(reachability_bundle.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(reachability_manifest["trust"]["state"], "not-requested");
    let compatibility: serde_json::Value = serde_json::from_slice(
        &std::fs::read(reachability_bundle.join("compatibility.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        compatibility["calibration"]["deadlines_seconds"]["launch"],
        7
    );
    assert_eq!(
        compatibility["calibration"]["deadlines_seconds"]["observation"],
        5
    );

    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(target_id).unwrap();
    assert!(facts.iter().any(|fact| {
        fact.key == CompatibilityFactKey::ProxyRouting && fact.value == "reached-client"
    }));
    assert!(facts.iter().any(|fact| {
        fact.key == CompatibilityFactKey::ProxyPropagation && fact.value == "confirmed"
    }));
    assert!(!facts
        .iter()
        .any(|fact| fact.key == CompatibilityFactKey::TlsTrustBehavior));
    drop(store);

    let tls_bundle = dir.path().join("tls");
    let (code, _out, tls_events) = run(&[
        "--json",
        "deep-capture",
        "sample-target",
        "--launch",
        "--calibrate",
        "tls",
        "--launch-case",
        "direct-exe-warm",
        "--yes",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        tls_bundle.to_str().unwrap(),
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");
    assert_eq!(code, 0, "TLS events:\n{tls_events}");
    assert!(tls_events.contains("deep_capture.trust"));
    assert!(tls_events.contains("\"status\":\"local-ca-accepted\""));

    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(target_id).unwrap();
    assert!(facts.iter().any(|fact| {
        fact.key == CompatibilityFactKey::TlsTrustBehavior && fact.value == "accepts-local-ca"
    }));
}

#[test]
fn reachability_calibration_rejects_trust_and_tls_outputs_before_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("bundle");
    seed_target(&local, false);
    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--calibrate",
        "reachability",
        "--launch-case",
        "direct-exe-warm",
        "--trust-ca",
        "--yes",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("does not change trust"), "refusal: {err}");
    assert!(!bundle.exists());
}

#[test]
fn tls_calibration_requires_current_same_case_routing_before_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, false);
    let bundle = dir.path().join("bundle");
    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--calibrate",
        "tls",
        "--launch-case",
        "direct-exe-warm",
        "--yes",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("run reachability calibration first"),
        "refusal: {err}"
    );
    assert!(!bundle.exists());
}

#[test]
fn deep_capture_requires_managed_launch_before_side_effects() {
    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--trust-ca",
        "--controlled-target",
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("requires --launch"),
        "the refusal names managed launch: {err}"
    );
}

#[test]
fn deep_capture_requires_explicit_trust_confirmation() {
    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--controlled-target",
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("explicit CA trust confirmation"),
        "the refusal names trust confirmation: {err}"
    );
}

#[test]
#[cfg(windows)]
fn deep_capture_refuses_unknown_real_target_compatibility_before_backend_lookup() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let bundle = dir.path().join("bundle");
    seed_target(&local, false);

    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--trust-ca",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(
        err.contains("requires current compatibility facts"),
        "the refusal names missing facts rather than backend state: {err}"
    );
}

#[test]
#[cfg(windows)]
fn deep_capture_refuses_an_unlaunchable_direct_target_before_session_resources() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let target_id = seed_target(&local, false);
    let mut store = Store::open(&local).unwrap();
    let mut fact = CompatibilityFact::new(
        target_id,
        CompatibilityFactKey::ProxyRouting,
        "reached-client",
        CompatibilityEvidenceSource::UserConfirmed,
    )
    .unwrap();
    fact.launch_case = Some(CompatibilityLaunchCase::DirectExeCold);
    store.insert_compatibility_fact(&fact).unwrap();
    drop(store);
    let bundle = dir.path().join("bundle");

    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--trust-ca",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);

    assert_eq!(code, 2);
    assert!(
        err.contains("stored install root"),
        "the refusal names the missing direct-launch fact: {err}"
    );
    assert!(
        !bundle.exists(),
        "preflight refusal must not create session storage"
    );
}

#[test]
fn deep_capture_refuses_a_nonempty_bundle_before_starting_the_proxy() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, false);
    let bundle = dir.path().join("bundle");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::write(bundle.join("keep.txt"), "operator-owned").unwrap();

    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--trust-ca",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("is not empty"), "refusal: {err}");
    assert_eq!(
        std::fs::read_to_string(bundle.join("keep.txt")).unwrap(),
        "operator-owned"
    );
}

#[test]
fn controlled_deep_capture_writes_a_bundle_and_compatibility_facts() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let target_id = seed_target(&local, false);
    let bundle = dir.path().join("bundle");

    let controlled_executable = std::env::var_os("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, out, err) = run(&[
        "--json",
        "deep-capture",
        "sample-target",
        "--launch",
        "--trust-ca",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
        "--har",
        "--key-log",
    ]);
    match controlled_executable {
        Some(value) => std::env::set_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE", value),
        None => std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE"),
    }
    assert_eq!(code, 0, "stderr:\n{err}");
    assert!(out.is_empty(), "Deep Capture writes status to stderr");
    assert!(
        err.contains("\"event\":\"deep_capture.preflight\"")
            && err.contains("\"event\":\"deep_capture.complete\""),
        "JSON events include Deep Capture lifecycle:\n{err}"
    );
    let completion: serde_json::Value = err
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .find(|line: &serde_json::Value| line["event"] == "deep_capture.complete")
        .unwrap();
    assert!(completion["inspectable"]
        .as_u64()
        .is_some_and(|count| count > 0));
    assert_eq!(completion["unknown"], 0);
    assert_eq!(completion["failed"], 0);
    assert_eq!(completion["unclassified_lost"], 0);

    for artifact in [
        "manifest.json",
        "capture.fcapng",
        "application.jsonl",
        "http.har",
        "proxy.jsonl",
        "process-trace.jsonl",
        "compatibility.json",
        "cleanup.json",
        "tls-keylog.log",
    ] {
        assert!(
            bundle.join(artifact).is_file(),
            "{artifact} must be present in the bundle"
        );
    }

    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["mode"], "deep-capture");
    assert_eq!(manifest["manifest_version"], 2);
    assert_eq!(manifest["product"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["state"], "complete");
    assert_eq!(manifest["target"]["handle"], "sample-target");
    assert_eq!(manifest["cleanup"]["status"], "succeeded");
    assert_eq!(manifest["trust"]["state"], "simulated-current-user");
    assert!(
        manifest["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|artifact| artifact["role"] == "tls-key-log"
                && artifact["sensitivity"] == "secret-adjacent"),
        "the native backend declares the produced proxy-owned key log"
    );
    let key_log = std::fs::read_to_string(bundle.join("tls-keylog.log")).unwrap();
    assert!(
        key_log.lines().all(|line| {
            let fields: Vec<_> = line.split_ascii_whitespace().collect();
            fields.len() == 3
                && fields[1].bytes().all(|byte| byte.is_ascii_hexdigit())
                && fields[2].bytes().all(|byte| byte.is_ascii_hexdigit())
        }) && !key_log.is_empty(),
        "the native backend writes complete NSS key-log records"
    );
    assert!(err.contains("\"event\":\"deep_capture.key_log_ready\""));

    let app = std::fs::read_to_string(bundle.join("application.jsonl")).unwrap();
    let application_records: Vec<serde_json::Value> = app
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        application_records.first().unwrap()["type"],
        "application.header"
    );
    assert_eq!(
        application_records.last().unwrap()["type"],
        "application.trailer"
    );
    assert_eq!(application_records.first().unwrap()["schema_version"], 2);
    assert_eq!(
        application_records.first().unwrap()["classification_schema_version"],
        1
    );
    assert!(application_records[1..application_records.len() - 1]
        .iter()
        .filter(|record| record["type"] != "application.correlation")
        .all(|record| record["classification"]["schema_version"] == 1));
    assert_eq!(
        application_records.last().unwrap()["classified_records"],
        application_records[1..application_records.len() - 1]
            .iter()
            .filter(|record| record.get("classification").is_some())
            .count()
    );
    assert_eq!(
        application_records.last().unwrap()["classification_records_lost"],
        0
    );
    assert!(application_records.last().unwrap()["written_records"]
        .as_u64()
        .is_some_and(|records| records > 0));
    assert!(app.contains("\"protocol\":\"http/1.1\""));
    assert!(app.contains("\"type\":\"tls.negotiation\""));
    assert!(app.contains("\"inspectability\":\"full\""));
    assert!(application_records[1..application_records.len() - 1]
        .iter()
        .any(|record| record["process_id"].as_u64().is_some_and(|pid| pid > 0)));
    assert!(application_records.iter().any(|record| {
        record["type"] == "application.correlation"
            && record["flow_id"].is_null()
            && record["correlation_reason"] == "controlled-harness-has-no-packet-flow"
    }));

    let har: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle.join("http.har")).unwrap()).unwrap();
    assert_eq!(har["log"]["version"], "1.2");
    assert!(har["log"]["entries"]
        .as_array()
        .is_some_and(|entries| !entries.is_empty()));
    assert!(har["log"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| {
            entry["response"]["status"].as_u64().is_some()
                && entry["time"].as_f64().is_some_and(|value| value >= 0.0)
                && entry["timings"]["send"]
                    .as_f64()
                    .is_some_and(|value| value >= 0.0)
                && entry["timings"]["wait"]
                    .as_f64()
                    .is_some_and(|value| value >= 0.0)
                && entry["timings"]["receive"]
                    .as_f64()
                    .is_some_and(|value| value >= 0.0)
        }));

    let process_trace = std::fs::read_to_string(bundle.join("process-trace.jsonl")).unwrap();
    assert!(process_trace.contains("controlled-harness.exited"));
    let process_event: serde_json::Value = serde_json::from_str(process_trace.trim()).unwrap();
    let child_pid = process_event["pid"].as_u64().expect("controlled child PID");
    assert!(child_pid > 0);
    assert_ne!(child_pid, u64::from(std::process::id()));
    assert!(application_records[1..application_records.len() - 1]
        .iter()
        .filter(|record| record["process_id"].as_u64().is_some())
        .all(|record| record["process_id"] == child_pid));

    let cleanup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle.join("cleanup.json")).unwrap()).unwrap();
    assert!(cleanup["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["resource"] == "trust-entry" && resource["status"] == "not-needed"
        }));
    assert!(cleanup["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["resource"] == "native-proxy-listener" && resource["status"] == "succeeded"
        }));
    assert!(cleanup["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|resource| {
            resource["resource"] == "bundle-evidence" && resource["status"] == "retained"
        }));

    let pcap = std::fs::read(bundle.join("capture.fcapng")).unwrap();
    assert_eq!(&pcap[0..4], &0x0A0D_0D0Au32.to_le_bytes());
    let pcap_text = String::from_utf8_lossy(&pcap);
    for ordinal in 1..=2 {
        assert!(
            pcap_text.contains(&format!("flow_id=flow-{ordinal:08}")),
            "controlled packet truth must carry flow-{ordinal:08}"
        );
    }

    let store = Store::open(&local).expect("reopen store");
    let read = match fragcap::targets::resolve_positional(&store, "sample-target").unwrap() {
        Selection::Resolved(t) => t,
        Selection::NoMatch => panic!("target must resolve after run"),
        Selection::Ambiguous(_) => panic!("target must resolve unambiguously after run"),
    };
    assert_eq!(read.id, Some(target_id));
    let facts = store
        .compatibility_facts_for_target(target_id)
        .expect("facts");
    assert!(
        facts.iter().any(|fact| {
            fact.key == CompatibilityFactKey::Inspectability && fact.value == "full"
        }),
        "Deep Capture writes inspectability facts"
    );
    assert!(
        facts.iter().any(|fact| {
            fact.key == CompatibilityFactKey::ProtocolBehavior && fact.value == "https"
        }),
        "Deep Capture writes protocol facts"
    );
    for (key, value) in [
        (CompatibilityFactKey::ProxyRouting, "reached-client"),
        (CompatibilityFactKey::ProxyPropagation, "confirmed"),
        (CompatibilityFactKey::TlsTrustBehavior, "accepts-local-ca"),
        (CompatibilityFactKey::FinalSocketOwnerRole, "client"),
    ] {
        assert!(
            facts
                .iter()
                .any(|fact| fact.key == key && fact.value == value),
            "controlled observation must write {}={value}",
            key.as_str()
        );
    }
}

#[test]
fn controlled_human_summary_reports_shared_classification_counts() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, false);
    let bundle = dir.path().join("bundle");
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );

    let (code, _out, err) = run(&[
        "deep-capture",
        "sample-target",
        "--launch",
        "--trust-ca",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 0, "stderr:\n{err}");
    assert!(
        err.contains("Protocol classification: observations=")
            && err.contains("unknown=0")
            && err.contains("failed=0")
            && err.contains("unclassified_lost=0"),
        "human summary reconciles classification counts:\n{err}"
    );
}

#[test]
fn partial_controlled_session_writes_observed_facts_and_manifest() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let target_id = seed_target(&local, false);
    let bundle = dir.path().join("bundle");
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER", "2");

    let (code, _out, err) = run(&[
        "--json",
        "deep-capture",
        "sample-target",
        "--launch",
        "--trust-ca",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER");
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 1, "partial session must preserve the target failure");
    assert!(err.contains("\"status\":\"partial\""), "events:\n{err}");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["state"], "partial");
    assert_eq!(
        manifest["sensitive_artifacts"]["tls_key_log"]["state"],
        "not-requested"
    );
    assert!(!bundle.join("tls-keylog.log").exists());
    assert!(bundle.join("capture.fcapng").is_file());
    let app = std::fs::read_to_string(bundle.join("application.jsonl")).unwrap();
    let trailer: serde_json::Value = serde_json::from_str(app.lines().last().unwrap()).unwrap();
    assert!(trailer["written_records"]
        .as_u64()
        .is_some_and(|records| records > 0));
    assert_eq!(trailer["writer_status"], "complete");

    let store = Store::open(&local).unwrap();
    let facts = store
        .compatibility_facts_for_target(target_id)
        .expect("partial facts");
    assert!(facts.iter().any(|fact| {
        fact.key == CompatibilityFactKey::ProtocolBehavior && fact.value == "https"
    }));
}

#[test]
fn partial_calibration_persists_the_same_failed_terminal_outcome_it_emits() {
    let _environment = controlled_environment().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    seed_target(&local, true);
    let bundle = dir.path().join("bundle");
    std::env::set_var(
        "FRAGCAP_CONTROLLED_TARGET_EXECUTABLE",
        env!("CARGO_BIN_EXE_fragcap"),
    );
    std::env::set_var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER", "2");

    let (code, _out, events) = run(&[
        "--json",
        "deep-capture",
        "sample-target",
        "--launch",
        "--calibrate",
        "tls",
        "--launch-case",
        "direct-exe-warm",
        "--yes",
        "--controlled-target",
        "--local-db",
        local.to_str().unwrap(),
        "--bundle",
        bundle.to_str().unwrap(),
    ]);
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER");
    std::env::remove_var("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE");

    assert_eq!(code, 1, "events:\n{events}");
    assert!(events.contains("\"status\":\"failed\""));
    let compatibility: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle.join("compatibility.json")).unwrap()).unwrap();
    assert_eq!(compatibility["calibration"]["outcome"], "failed");
}
