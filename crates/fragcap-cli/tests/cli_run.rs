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

/// Run the offline substrate to real output files, returning the exit code, the
/// stderr, the produced pcapng bytes, and the produced JSON Lines text.
fn run_offline_to_files(extra: &[String]) -> (u8, String, Vec<u8>, String) {
    let dir = tempfile::tempdir().unwrap();
    let fcapng = dir.path().join("out.fcapng");
    let jsonl = dir.path().join("out.jsonl");
    let mut args: Vec<String> = vec![
        "--out".into(),
        fcapng.to_string_lossy().into_owned(),
        "--sink".into(),
        format!("jsonl:{}", jsonl.to_string_lossy()),
    ];
    args.extend(extra.iter().cloned());
    let (code, _out, err) = run_offline(&args);
    let fcapng_bytes = fs::read(&fcapng).expect("the pcapng file was written");
    let jsonl_text = fs::read_to_string(&jsonl).expect("the jsonl file was written");
    (code, err, fcapng_bytes, jsonl_text)
}

/// The number of packet records in a JSON Lines capture: every line carries a
/// `"ts":` field except the `header` and `trailer` lines, which do not.
fn jsonl_packet_count(text: &str) -> usize {
    text.lines().filter(|l| l.contains("\"ts\":")).count()
}

/// The number of Enhanced Packet Blocks in a pcapng stream, by walking the
/// length-prefixed block structure and counting the EPB block type (0x00000006).
/// Robust against block contents, so it counts exactly the packet records on disk.
fn pcapng_epb_count(bytes: &[u8]) -> usize {
    let mut i = 0usize;
    let mut epbs = 0usize;
    while i + 8 <= bytes.len() {
        let block_type = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        let total_len = u32::from_le_bytes(bytes[i + 4..i + 8].try_into().unwrap()) as usize;
        if total_len < 12 || i + total_len > bytes.len() {
            break;
        }
        if block_type == 0x0000_0006 {
            epbs += 1;
        }
        i += total_len;
    }
    epbs
}

#[test]
fn each_bound_stops_for_its_named_reason() {
    // Duration is not a packet or byte count, so it keeps the stop-reason
    // assertion; the hard packet and byte bounds are asserted by produced count
    // in the two tests below.
    let (code, _out, err) = run_offline(&["--duration".into(), "10s".into()]);
    assert_eq!(code, 0);
    assert!(err.contains("duration-reached"), "{err}");
}

// SC-001, SC-002, FR-006, FR-007. A packet bound produces a file with exactly N
// packet records in both writers, and the summary's retained count equals what is
// on disk. The fixture has 24 packets; the bound of 5 is well below it, so the
// soft-bound behavior this replaces would have written more than 5.
#[test]
fn a_packet_bound_produces_an_exactly_bounded_file() {
    let (code, err, fcapng, jsonl) = run_offline_to_files(&["--max-packets".into(), "5".into()]);
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains("volume-reached"),
        "the bound stops the run: {err}"
    );

    assert_eq!(
        jsonl_packet_count(&jsonl),
        5,
        "the JSON Lines file holds exactly the bound"
    );
    assert_eq!(
        pcapng_epb_count(&fcapng),
        5,
        "the pcapng file holds exactly the bound"
    );
    // The summary's retained count equals the packets on disk, and no packet is
    // both written and counted as discarded (the retained line reports exactly 5).
    assert!(
        err.contains("retained"),
        "the summary reports retained: {err}"
    );
    assert!(
        err.lines()
            .any(|l| l.contains("retained") && l.contains('5')),
        "the summary reports 5 retained, matching the file: {err}"
    );
}

// Review of PR #26 (Codex P2). A zero packet bound produces an empty but
// well-formed capture and stops for volume-reached, not a later source-exhausted
// reason. The run still acquired a target, so it exits zero.
#[test]
fn a_zero_packet_bound_produces_an_empty_capture_and_stops_for_volume() {
    let (code, err, fcapng, jsonl) = run_offline_to_files(&["--max-packets".into(), "0".into()]);
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains("volume-reached"),
        "a zero bound stops for volume-reached: {err}"
    );
    assert_eq!(jsonl_packet_count(&jsonl), 0, "no packet is written");
    assert_eq!(
        pcapng_epb_count(&fcapng),
        0,
        "the pcapng holds no packet block"
    );
    assert!(
        err.lines()
            .any(|l| l.contains("retained") && l.contains('0')),
        "the summary reports zero retained: {err}"
    );
}

// FR-006, D-4. A byte bound produces the first prefix of packets whose cumulative
// captured length reaches or crosses the bound, identically in both writers.
#[test]
fn a_byte_bound_produces_an_exactly_bounded_file() {
    let (code, err, fcapng, jsonl) = run_offline_to_files(&["--max-bytes".into(), "100b".into()]);
    assert_eq!(code, 0, "{err}");
    assert!(err.contains("volume-reached"), "{err}");

    let jsonl_packets = jsonl_packet_count(&jsonl);
    assert!(jsonl_packets >= 1, "at least one packet is retained");
    assert_eq!(
        jsonl_packets,
        pcapng_epb_count(&fcapng),
        "both writers hold the same packet count under a byte bound"
    );
    // The bound is small (100 bytes), so the file is a small prefix of the 24
    // fixture packets rather than all of them.
    assert!(
        jsonl_packets < 24,
        "a byte bound below the fixture size caps the file"
    );
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

// Slice S16, US1. The first four bytes of a pcapng stream are the Section Header
// Block magic in little-endian, so a dump that starts with them is a real file.
fn is_valid_pcapng(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0..4] == 0x0A0D_0D0Au32.to_le_bytes()
}

// Slice S16, US1, SC-001. Ring mode dumped on interrupt: a rolling window smaller
// than the fixture, the target never exits, and the operator interrupt is the
// trigger. The dump is a valid pcapng holding only the recent tail.
#[test]
fn a_ring_capture_dumps_the_recent_tail_on_interrupt() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ring.fcapng");
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
        "--mode".into(),
        "ring".into(),
        "--ring".into(),
        "200b".into(),
        "--out".into(),
        out.to_string_lossy().into_owned(),
    ];
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _out, err) = common::run(&refs);
    assert_eq!(code, 0, "an interrupt is a clean stop: {err}");
    assert!(
        err.contains("interrupt"),
        "the stop reason is the interrupt: {err}"
    );

    let bytes = fs::read(&out).expect("the ring window was dumped");
    assert!(is_valid_pcapng(&bytes), "the dump is a valid pcapng");
    let epbs = pcapng_epb_count(&bytes);
    assert!(epbs >= 1, "a seen capture dumps at least one packet");
    assert!(
        epbs < 24,
        "a small window holds only the tail, not all 24 packets"
    );
}

// Slice S16, US1, SC-002, SC-003, FR-012. A non-interrupt trigger (the terminal
// stage exits) also dumps the window, and a window larger than the whole capture
// dumps every retained packet, equal in count to a plain file capture of the same
// input.
#[test]
fn a_large_ring_window_dumps_the_whole_capture_on_a_non_interrupt_stop() {
    // Reference: the same run as a plain file capture.
    let (code, _err, file_bytes, _jsonl) = run_offline_to_files(&[]);
    assert_eq!(code, 0);
    let reference = pcapng_epb_count(&file_bytes);
    assert!(reference > 0, "the reference capture has packets");

    // The same run in ring mode with a window larger than the whole capture.
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ring.fcapng");
    let (code, _o, err) = run_offline(&[
        "--mode".into(),
        "ring".into(),
        "--ring".into(),
        "100mb".into(),
        "--out".into(),
        out.to_string_lossy().into_owned(),
    ]);
    assert_eq!(
        code, 0,
        "a terminal-stage-exit stop dumps the window: {err}"
    );
    assert!(
        !err.contains("interrupt"),
        "this run stops on a non-interrupt condition: {err}"
    );

    let bytes = fs::read(&out).expect("the ring window was dumped");
    assert!(is_valid_pcapng(&bytes), "the dump is a valid pcapng");
    assert_eq!(
        pcapng_epb_count(&bytes),
        reference,
        "a whole-input ring dump holds the same packets as a plain file capture"
    );
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
