// SPDX-License-Identifier: Apache-2.0

//! `run` and `tap` end to end over the offline substrate: a replay source, a
//! scripted attributor, and a scripted process timeline. No capture driver, no
//! elevation, no game.

mod common;

use std::fs;

use common::{data, fixture};

/// Run with the standard offline substrate, appending `extra`.
fn run_offline(extra: &[String]) -> (u8, String, String) {
    let mut args: Vec<String> = vec![
        "run".into(),
        "--profile".into(),
        data("game.toml"),
        "--replay-source".into(),
        fixture("udp-gameplay.pcap"),
        "--attr-script".into(),
        fixture("udp-gameplay.script"),
        "--process-script".into(),
        data("game.procscript"),
        "--local-addr".into(),
        "192.0.2.10".into(),
    ];
    args.extend(extra.iter().cloned());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    common::run(&refs)
}

#[test]
fn a_run_produces_the_capture_goldens_with_stamped_role_and_stage() {
    let dir = tempfile::tempdir().unwrap();
    let fcapng = dir.path().join("out.fcapng");
    let jsonl = dir.path().join("out.jsonl");

    let (code, _out, err) = run_offline(&[
        "--out".into(),
        fcapng.to_string_lossy().into_owned(),
        "--sink".into(),
        format!("jsonl:{}", jsonl.to_string_lossy()),
    ]);
    assert_eq!(code, 0, "the run succeeds: {err}");

    let fcapng_bytes = fs::read(&fcapng).expect("the pcapng file was written");
    let jsonl_bytes = fs::read(&jsonl).expect("the jsonl file was written");
    common::assert_golden("run.fcapng", &fcapng_bytes);
    common::assert_golden("run.jsonl", &jsonl_bytes);

    let jsonl_text = String::from_utf8(jsonl_bytes).unwrap();
    assert!(
        jsonl_text.contains("\"role\":\"client\""),
        "every attributed packet carries the role"
    );
    assert!(
        jsonl_text.contains("\"stage\":\"client\""),
        "every attributed packet carries the stage"
    );
}

#[test]
fn the_completion_summary_counts_satisfy_the_conservation_identity() {
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("out.jsonl");
    let (code, _out, err) = run_offline(&[
        "--sink".into(),
        format!("jsonl:{}", jsonl.to_string_lossy()),
    ]);
    assert_eq!(code, 0);
    assert!(
        err.contains("packets captured"),
        "the summary is printed: {err}"
    );

    // The jsonl trailer carries the run's own counters. Conservation: everything
    // captured is attributed or unattributed, and fragcap dropped nothing.
    let text = fs::read_to_string(&jsonl).unwrap();
    let trailer: serde_json::Value =
        serde_json::from_str(text.lines().last().unwrap()).expect("the trailer parses");
    let packets = trailer["packets"].as_u64().unwrap();
    let attributed = trailer["attributed"].as_u64().unwrap();
    let unattributed = trailer["unattributed"].as_u64().unwrap();
    assert_eq!(packets, 24);
    assert_eq!(
        attributed + unattributed,
        packets,
        "every captured packet is in exactly one attribution state"
    );
    assert_eq!(trailer["buffer_dropped"].as_u64().unwrap(), 0);
    assert_eq!(trailer["sink_dropped"].as_u64().unwrap(), 0);
}

#[test]
fn the_json_event_sequence_matches_the_golden() {
    let (code, _out, err) = run_offline(&["--json".into()]);
    assert_eq!(code, 0);
    let normalized = common::normalize_timestamps(&err);
    common::assert_golden("run-events.ndjson", normalized.as_bytes());
}

#[test]
fn a_fired_interrupt_stops_cleanly_and_exits_zero() {
    // A process script whose target never exits, so the interrupt is the stop.
    let args: Vec<String> = vec![
        "run".into(),
        "--profile".into(),
        data("game.toml"),
        "--replay-source".into(),
        fixture("udp-gameplay.pcap"),
        "--attr-script".into(),
        fixture("udp-gameplay.script"),
        "--process-script".into(),
        data("game-running.procscript"),
        "--local-addr".into(),
        "192.0.2.10".into(),
        "--fire-interrupt".into(),
    ];
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _out, err) = common::run(&refs);
    assert_eq!(code, 0, "an interrupt during capture is a success");
    assert!(
        err.contains("interrupt"),
        "the stop reason is the interrupt: {err}"
    );
}

#[test]
fn each_bound_stops_for_its_named_reason() {
    let (code, _out, err) = run_offline(&["--duration".into(), "10s".into()]);
    assert_eq!(code, 0);
    assert!(err.contains("duration-reached"), "{err}");

    let (code, _out, err) = run_offline(&["--max-packets".into(), "5".into()]);
    assert_eq!(code, 0);
    assert!(err.contains("volume-reached"), "{err}");

    let (code, _out, err) = run_offline(&["--max-bytes".into(), "100b".into()]);
    assert_eq!(code, 0);
    assert!(err.contains("volume-reached"), "{err}");
}

#[test]
fn an_acquisition_timeout_with_no_target_captures_nothing_and_exits_one() {
    // No process script, so no target is ever acquired.
    let args = [
        "run",
        "--profile",
        &data("game.toml"),
        "--replay-source",
        &fixture("udp-gameplay.pcap"),
        "--attr-script",
        &fixture("udp-gameplay.script"),
        "--local-addr",
        "192.0.2.10",
        "--wait",
        "30s",
    ];
    let (code, _out, err) = common::run(&args);
    assert_eq!(code, 1, "target never acquired is an expected failure");
    assert!(err.contains("never acquired"), "{err}");
    assert!(err.contains("acquisition-timeout"), "{err}");
}

#[test]
fn tap_captures_a_named_process_through_the_same_engine() {
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("tap.jsonl");
    let args = [
        "tap",
        "--process",
        "game.exe",
        "--sink",
        &format!("jsonl:{}", jsonl.to_string_lossy()),
        "--replay-source",
        &fixture("udp-gameplay.pcap"),
        "--attr-script",
        &fixture("udp-gameplay.script"),
        "--process-script",
        &data("game.procscript"),
        "--local-addr",
        "192.0.2.10",
    ];
    let (code, _out, err) = common::run(&args);
    assert_eq!(code, 0, "tap captures like run: {err}");
    assert!(err.contains("capture complete"), "{err}");

    let text = fs::read_to_string(&jsonl).unwrap();
    assert!(
        text.contains("\"role\":\"target\""),
        "the synthesized single stage is the target role: {text}"
    );
}

#[test]
fn tap_without_a_process_is_a_usage_error() {
    let (code, _out, _err) = common::run(&["tap"]);
    assert_eq!(code, 2);
}
