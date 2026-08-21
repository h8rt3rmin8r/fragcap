// SPDX-License-Identifier: Apache-2.0

//! `capture` end to end over the offline substrate: a replay source, a scripted
//! attributor, and a scripted process timeline. No capture driver, no elevation, no
//! game. The identity is a `--process` image name synthesizing a single
//! `target`-role stage, the path the retired `run`, `tap`, and `watch` all fed.

mod common;

use std::fs;

use common::{data, fixture};
use fragcap::profile::FidelityTier;
use fragcap::targets::{
    capture_readiness, launch_is_unresolved, resolve_positional, CaptureReadiness, Selection,
    Store, TargetEntry,
};

/// Run with the standard offline substrate, appending `extra`.
fn run_offline(extra: &[String]) -> (u8, String, String) {
    let mut args: Vec<String> = vec![
        "capture".into(),
        "--process".into(),
        "game.exe".into(),
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
    common::assert_golden("capture.fcapng", &fcapng_bytes);
    common::assert_golden("capture.jsonl", &jsonl_bytes);

    let jsonl_text = String::from_utf8(jsonl_bytes).unwrap();
    assert!(
        jsonl_text.contains("\"role\":\"target\""),
        "every attributed packet carries the role"
    );
    assert!(
        jsonl_text.contains("\"stage\":\"target\""),
        "every attributed packet carries the stage"
    );
}

// Regression: `--roles target` once panicked at access time because the roles flag
// used a `value_parser` returning `Vec<String>` against a `Vec<String>` field (clap
// could not downcast the element). The flag now uses `value_delimiter`, so the
// capture parses, succeeds, and scopes to the named role.
#[test]
fn run_with_roles_succeeds_and_scopes_to_the_named_role() {
    let (code, _out, err) = run_offline(&["--roles".into(), "target".into()]);
    assert_eq!(code, 0, "capture --roles target parses and captures: {err}");
    assert!(
        err.contains("roles target"),
        "the capture is scoped to the named role: {err}"
    );
    // Since slice S064 the roles claim is true of what the file contains, not
    // only of which stages trigger acquisition: the write gate consults the same
    // set under the default target scope. The fixture's traffic is all
    // `role=target`, so scoping to that role retains all of it and discards
    // nothing, which is what distinguishes an enforced scope from a claimed one.
    assert!(
        err.contains("scope: writing target traffic"),
        "the run states what it writes: {err}"
    );
    let out_of_scope = err
        .lines()
        .find_map(|l| l.trim().strip_prefix("out of scope"))
        .map(|v| v.trim().to_string());
    assert_eq!(
        out_of_scope.as_deref(),
        Some("0"),
        "nothing was out of scope, because every packet is the named role: {err}"
    );
}

// The comma-separated form splits into the role list: the synthesized `target`
// stage is still in the allowed set, so the capture succeeds and stays scoped. This
// documents the accepted behavior of dropping the empty-role rejection, matching the
// `extcap` surface.
#[test]
fn run_with_a_comma_separated_role_list_splits_and_scopes() {
    let (code, _out, err) = run_offline(&["--roles".into(), "target,launcher".into()]);
    assert_eq!(code, 0, "a comma-separated role list parses: {err}");
    assert!(
        err.contains("roles target,launcher"),
        "both roles are carried into the scope: {err}"
    );
    assert!(
        err.contains("scope: writing target traffic"),
        "the run states what it writes: {err}"
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
    common::assert_golden("capture-events.ndjson", normalized.as_bytes());
}

#[test]
fn a_fired_interrupt_stops_cleanly_and_exits_zero() {
    // A process script whose target never exits, so the interrupt is the stop.
    let args: Vec<String> = vec![
        "capture".into(),
        "--process".into(),
        "game.exe".into(),
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
        "capture",
        "--process",
        "game.exe",
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
        "capture".into(),
        "--process".into(),
        "game.exe".into(),
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
    // The evictions the small window forced are surfaced in the summary, not
    // silently dropped (Codex review of PR #30, P1; constitution P-4).
    assert!(
        err.contains("evicted from the rolling window"),
        "the ring eviction count is surfaced: {err}"
    );
    let evicted_line = err
        .lines()
        .find(|l| l.contains("evicted from the rolling window"))
        .expect("the eviction line is present");
    assert!(
        !evicted_line.contains("0 packet(s)"),
        "a window smaller than the capture evicts at least one packet: {evicted_line}"
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
fn capture_by_process_name_produces_a_target_role_stage() {
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("tap.jsonl");
    let args = [
        "capture",
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
    assert_eq!(code, 0, "capture --process succeeds: {err}");
    assert!(err.contains("capture complete"), "{err}");

    let text = fs::read_to_string(&jsonl).unwrap();
    assert!(
        text.contains("\"role\":\"target\""),
        "the synthesized single stage is the target role: {text}"
    );
}

#[test]
fn capture_without_a_target_input_is_a_usage_error() {
    // Neither --target nor --process: the clap group requires exactly one.
    let (code, _out, _err) = common::run(&["capture"]);
    assert_eq!(code, 2);
}

#[test]
fn the_three_target_inputs_are_mutually_exclusive() {
    // --target, --id, and --process form one required, exactly-one group.
    for argv in [
        vec!["capture", "--target", "eso", "--process", "eso64.exe"],
        vec!["capture", "--target", "eso", "--id", "1"],
        vec!["capture", "--id", "1", "--process", "eso64.exe"],
    ] {
        let (code, _out, _err) = common::run(&argv);
        assert_eq!(code, 2, "mutually exclusive target inputs: {argv:?}");
    }
}

#[test]
fn capture_process_with_launch_is_a_usage_error() {
    // A raw process image carries no launchable platform anchor, so --launch is a
    // usage error rather than a silent no-op (FR-005).
    let (code, _out, err) = common::run(&["capture", "--process", "game.exe", "--launch"]);
    assert_eq!(code, 2, "a process capture cannot launch: {err}");
}

/// Register a target with a stored exe as the resolved client and return
/// `(db_path, handle, stable_id)`. `--socket-holder yes` records the executable as
/// the client the capture will address; without an answer the chain would be left
/// unresolved (the tool never assumes the holder, P-9).
fn register_exe_target(dir: &tempfile::TempDir, name: &str, exe: &str) -> (String, String, String) {
    let db = dir.path().join("local.db").to_string_lossy().into_owned();
    let (code, out, err) = common::run(&[
        "targets",
        "add",
        name,
        "--db",
        &db,
        "--exe",
        exe,
        "--socket-holder",
        "yes",
    ]);
    assert_eq!(code, 0, "targets add: {err}");
    // "registered <handle> (id <id>)"
    let rest = out
        .lines()
        .find_map(|l| l.strip_prefix("registered "))
        .expect("a registered line");
    let handle = rest.split_whitespace().next().unwrap().to_string();
    let id = rest
        .rsplit_once("(id ")
        .unwrap()
        .1
        .trim_end_matches(')')
        .to_string();
    (db, handle, id)
}

/// Capture a stored target through the offline substrate. The catalog store points
/// at a fresh nonexistent path so no per-user store is bootstrapped as a side effect.
fn capture_target_offline(db: &str, selector: &[&str]) -> (u8, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("t.jsonl");
    let catalog = dir.path().join("catalog.db");
    let mut args: Vec<String> = vec!["capture".into()];
    args.extend(selector.iter().map(|s| s.to_string()));
    args.extend([
        "--local-db".into(),
        db.into(),
        "--catalog-db".into(),
        catalog.to_string_lossy().into_owned(),
        "--sink".into(),
        format!("jsonl:{}", jsonl.to_string_lossy()),
        "--replay-source".into(),
        fixture("udp-gameplay.pcap"),
        "--attr-script".into(),
        fixture("udp-gameplay.script"),
        "--process-script".into(),
        data("game.procscript"),
        "--local-addr".into(),
        "192.0.2.10".into(),
    ]);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _o, err) = common::run(&refs);
    let text = fs::read_to_string(&jsonl).unwrap_or_default();
    (code, err, text)
}

#[test]
fn capture_by_stored_target_selector_and_id() {
    // A target registered with a launch executable captures that client, addressed
    // by its handle selector and by its durable id.
    let dir = tempfile::tempdir().unwrap();
    let (db, handle, id) = register_exe_target(&dir, "Test Game", "game.exe");

    let (code, err, text) = capture_target_offline(&db, &["--target", &handle]);
    assert_eq!(code, 0, "capture --target <handle>: {err}");
    assert!(
        text.contains("\"role\":\"target\""),
        "captured the stored client: {text}"
    );

    let (code, err, text) = capture_target_offline(&db, &["--id", &id]);
    assert_eq!(code, 0, "capture --id <stable_id>: {err}");
    assert!(
        text.contains("\"role\":\"target\""),
        "the durable id selects the same client: {text}"
    );
}

#[test]
fn capture_target_with_no_client_executable_is_refused() {
    // A name-only target names no launch executable, so it is refused rather than
    // captured empty (P-4).
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local.db").to_string_lossy().into_owned();
    let (code, _o, err) = common::run(&["targets", "add", "Nameless", "--db", &db]);
    assert_eq!(code, 0, "add: {err}");
    let (code, err, _t) = capture_target_offline(&db, &["--target", "nameless"]);
    assert_eq!(code, 2, "a target with no client is refused: {err}");
    assert!(err.contains("no Windows client"), "{err}");
}

#[test]
fn capture_steam_anchored_target_rejects_a_path_anchor() {
    // A path anchor cannot attach to a Steam-anchored target (the cascade determines
    // the client), so it is refused rather than silently discarded (P-9). No Steam
    // install is needed: the anchor check precedes the install lookup.
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local.db").to_string_lossy().into_owned();
    let (code, _o, err) = common::run(&[
        "targets",
        "add",
        "Anchored",
        "--db",
        &db,
        "--anchor",
        "steam:620",
    ]);
    assert_eq!(code, 0, "add: {err}");
    let (code, err, _t) = capture_target_offline(&db, &["--target", "anchored", "--path", "Games"]);
    assert_eq!(code, 2, "a path anchor on a steam target is refused: {err}");
    assert!(
        err.contains("do not apply to a Steam-anchored target"),
        "{err}"
    );
}

// Slice S059: launch-and-observe capture and capture-time promotion.

/// Register an unresolved target: `--socket-holder no` records the executable as a
/// launcher stage with the holder unresolved, so the stored launch chain names no
/// client and CAPTURE reads "needs a target" until a run observes one. Returns
/// `(db_path, handle)`.
fn register_unresolved_target(dir: &tempfile::TempDir, name: &str, exe: &str) -> (String, String) {
    let db = dir.path().join("local.db").to_string_lossy().into_owned();
    let (code, out, err) = common::run(&[
        "targets",
        "add",
        name,
        "--db",
        &db,
        "--exe",
        exe,
        "--socket-holder",
        "no",
    ]);
    assert_eq!(code, 0, "targets add --socket-holder no: {err}");
    let rest = out
        .lines()
        .find_map(|l| l.strip_prefix("registered "))
        .expect("a registered line");
    let handle = rest.split_whitespace().next().unwrap().to_string();
    (db, handle)
}

/// Read a stored target back by its handle, so a test can assert what a capture
/// wrote to the store.
fn read_target(db: &str, handle: &str) -> TargetEntry {
    let store = Store::open(db).expect("open the local store");
    match resolve_positional(&store, handle).expect("resolve the handle") {
        Selection::Resolved(entry) => *entry,
        other => panic!("expected exactly one target for {handle}, got {other:?}"),
    }
}

/// Capture an unresolved target through the observe-mode substrate: a launcher
/// spawns the child that holds the sockets. `attributed` chooses whether the attr
/// script attributes the child's flow (a dominant holder is observed) or not
/// (nothing is observed). The catalog store points at a fresh path so no per-user
/// store is bootstrapped as a side effect.
fn capture_observe_offline(db: &str, selector: &str, attributed: bool) -> (u8, String) {
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("o.jsonl");
    let catalog = dir.path().join("catalog.db");
    let mut args: Vec<String> = vec![
        "capture".into(),
        selector.into(),
        "--local-db".into(),
        db.into(),
        "--catalog-db".into(),
        catalog.to_string_lossy().into_owned(),
        "--sink".into(),
        format!("jsonl:{}", jsonl.to_string_lossy()),
        "--replay-source".into(),
        fixture("udp-gameplay.pcap"),
        "--process-script".into(),
        data("observe.procscript"),
        "--local-addr".into(),
        "192.0.2.10".into(),
    ];
    // With the attr script the child (game.exe) holds the flow and is attributed;
    // without it the empty scripted attributor attributes nothing, so the target is
    // acquired (the launcher binds) but no socket holder is observed.
    if attributed {
        args.push("--attr-script".into());
        args.push(fixture("udp-gameplay.script"));
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, _o, err) = common::run(&refs);
    (code, err)
}

#[test]
fn an_unresolved_target_is_captured_and_promoted_to_its_observed_holder() {
    // US1 scenario 1: a `no`-authored target names launcher.exe as the observed
    // executable but no client. A capture observes the child game.exe holding the
    // sockets and promotes the target to that resolved client at verified fidelity.
    let dir = tempfile::tempdir().unwrap();
    let (db, handle) = register_unresolved_target(&dir, "Observe Game", "launcher.exe");

    // Before capture: unresolved, needs a target.
    let before = read_target(&db, &handle);
    assert!(launch_is_unresolved(&before), "registered unresolved");
    assert_eq!(capture_readiness(&before), CaptureReadiness::NeedsTarget);

    let (code, err) = capture_observe_offline(&db, &handle, true);
    assert_eq!(code, 0, "the observe-mode capture succeeds: {err}");
    assert!(
        err.contains("promoted the target to its observed socket holder game.exe"),
        "the promotion is surfaced: {err}"
    );

    // After capture: resolved to the observed child, verified, and now ready.
    let after = read_target(&db, &handle);
    assert!(
        !launch_is_unresolved(&after),
        "the promoted target is no longer unresolved"
    );
    assert_eq!(after.fidelity, FidelityTier::Verified, "raised to verified");
    assert_eq!(capture_readiness(&after), CaptureReadiness::Ready);
    let launch = after.launch_entries.expect("a resolved launch chain");
    let clients = launch.as_array().expect("a resolved client array");
    assert_eq!(clients.len(), 1, "one resolved client");
    assert_eq!(
        clients[0].get("executable").and_then(|v| v.as_str()),
        Some("game.exe"),
        "the observed child is the resolved client"
    );
    assert_eq!(
        clients[0].get("role").and_then(|v| v.as_str()),
        Some("client")
    );
}

#[test]
fn an_unresolved_target_that_observes_nothing_is_left_unchanged() {
    // US1 scenario 2 (the P-9 branch): the target is acquired (the launcher binds)
    // but nothing is attributed, so no socket holder is observed and the stored
    // target is left exactly as registered. Promoting on no observation would
    // fabricate a holder the run never saw.
    let dir = tempfile::tempdir().unwrap();
    let (db, handle) = register_unresolved_target(&dir, "Observe Game", "launcher.exe");
    let before = read_target(&db, &handle);

    let (code, err) = capture_observe_offline(&db, &handle, false);
    assert_eq!(code, 0, "the capture completes without error: {err}");
    assert!(
        err.contains("observed no socket-holding process"),
        "the no-observation outcome is surfaced: {err}"
    );

    let after = read_target(&db, &handle);
    assert!(
        launch_is_unresolved(&after),
        "the target is still unresolved"
    );
    assert_eq!(
        after.launch_entries, before.launch_entries,
        "the launch chain is byte-identical to what was registered"
    );
    assert_eq!(after.fidelity, before.fidelity, "the fidelity is unchanged");
}

#[test]
fn the_retired_verbs_and_profile_command_do_not_parse() {
    // No aliases, no shims: run/tap/watch and the profile command are gone.
    for argv in [
        vec!["run", "--profile", "game"],
        vec!["tap", "--process", "game.exe"],
        vec!["watch", "--exe", "game.exe"],
        vec!["profile", "validate", "game"],
    ] {
        let (code, _out, _err) = common::run(&argv);
        assert_eq!(code, 2, "retired command still parses: {argv:?}");
    }
    // schema validate remains the general artifact validator.
    let (code, _out, _err) = common::run(&["schema", "--help"]);
    assert_eq!(code, 0, "schema stays available");
}
/// The machine-readable stream accounts for scope exclusions too.
///
/// Raised in review of PR #191. `--json` suppresses the human completion
/// summary entirely and emits `session.complete` in its place, so carrying the
/// two scope counters only in the human renderer meant a machine consumer saw a
/// capture that observed packets, wrote none of them, and accounted for none of
/// the difference. Every discard path has a named counter on every surface, not
/// only the one a person reads (P-4).
#[test]
fn the_json_completion_event_accounts_for_scope_exclusions() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("noise.script");
    // The same flow, owned by a process no profile stage binds: the issue #184
    // case, background traffic while the target has opened nothing.
    std::fs::write(
        &script,
        "flow udp 192.0.2.10:30000 * always owner 9999 docker.exe\nendpoint udp 192.0.2.10:30000\n",
    )
    .unwrap();
    let out_file = dir.path().join("out.pcapng");

    let (code, _out, err) = common::run(&[
        "capture",
        "--process",
        "game.exe",
        "--json",
        "--replay-source",
        &fixture("udp-gameplay.pcap"),
        "--attr-script",
        &script.to_string_lossy(),
        "--process-script",
        &data("game.procscript"),
        "--local-addr",
        "192.0.2.10",
        "--out",
        &out_file.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "the run succeeds: {err}");

    let complete = err
        .lines()
        .find(|l| l.contains("\"event\":\"session.complete\""))
        .expect("a session.complete event is emitted");

    assert!(
        complete.contains("\"scope_discarded\":24"),
        "every excluded packet is accounted for in the machine stream: {complete}"
    );
    assert!(
        complete.contains("\"scope_unresolved_discarded\":0"),
        "the two exclusion reasons are reported apart: {complete}"
    );
}
