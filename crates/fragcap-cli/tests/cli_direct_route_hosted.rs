// SPDX-License-Identifier: Apache-2.0

//! Public staged-binary direct-route journey on a disposable Windows runner.

#![cfg(windows)]

use fragcap::profile::FidelityTier;
use fragcap::targets::{
    resolved_client_launch, ClassificationSource, CompatibilityFactKey, Store,
    TargetClassification, TargetEntry,
};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[test]
#[ignore = "requires the signed staged component and disposable elevated Windows runner"]
fn staged_public_direct_route_authorizes_one_composite_plan_and_cleans_up() {
    use std::os::windows::process::CommandExt;

    let binary = PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_BINARY").unwrap());
    let scratch = PathBuf::from(std::env::var_os("FRAGCAP_WINDOWS_SCRATCH").unwrap());
    std::fs::create_dir_all(&scratch).unwrap();
    let local = scratch.join("direct-cli.db");
    let bundle = scratch.join("direct-cli-bundle");
    let mut store = Store::open(&local).unwrap();
    let row_id = store
        .insert_target(&TargetEntry {
            id: None,
            stable_id: 75_000,
            handle: "sample-target".into(),
            name: "Sample Target".into(),
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
            executable_hint: Some("client.exe".into()),
        })
        .unwrap();
    drop(store);

    let mut child = Command::new(&binary)
        .args([
            "--json",
            "deep-capture",
            "--id",
            "75000",
            "--launch",
            "--direct-route",
            "--authorize-stdin",
            "--controlled-target",
            "--calibrate",
            "reachability",
            "--calibration-protocol",
            "routing",
            "--launch-case",
            "direct-exe-cold",
            "--local-db",
            local.to_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
        ])
        .env("FRAGCAP_SESSION_DIR", &scratch)
        .env("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE", &binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut events = Vec::new();
    for line in BufReader::new(child.stderr.take().unwrap()).lines() {
        let record: Value = serde_json::from_str(&line.unwrap()).unwrap();
        if record["event"] == "deep_capture.authorization_plan" {
            assert_eq!(
                record["plan"]["proxy"]["route_identity"],
                "child-environment+direct-socket-v1"
            );
            assert!(record["plan"]["proxy"]["direct_socket"].is_object());
            writeln!(stdin, "{}", record["plan_id"].as_str().unwrap()).unwrap();
            stdin.flush().unwrap();
        }
        events.push(record);
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "events: {events:#?}");
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "deep_capture.authorization_plan")
            .count(),
        1
    );
    assert!(events
        .iter()
        .any(|event| event["event"] == "deep_capture.complete"));
    let store = Store::open(&local).unwrap();
    let facts = store.compatibility_facts_for_target(row_id).unwrap();
    assert!(
        facts.iter().all(|fact| {
            fact.key != CompatibilityFactKey::ProxyRouting || fact.value != "reached-client"
        }),
        "environment-only controlled traffic must not qualify a direct receipt"
    );
}
