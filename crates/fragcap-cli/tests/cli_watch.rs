// SPDX-License-Identifier: Apache-2.0

//! `watch` end to end over the offline substrate: launch-agnostic identity
//! capture, attach-to-running, and the loud give-up. No capture driver, no
//! elevation, no game.

mod common;

use std::fs;

use common::{data, fixture};

/// A `watch` invocation over the standard offline substrate, appending `extra`.
/// The identity flags (`--exe`, `--path`, `--wait`) come from `extra`.
fn watch_offline(procscript: &str, extra: &[String]) -> (u8, String, String) {
    let mut args: Vec<String> = vec![
        "watch".into(),
        "--replay-source".into(),
        fixture("udp-gameplay.pcap"),
        "--attr-script".into(),
        fixture("udp-gameplay.script"),
        "--process-script".into(),
        data(procscript),
        "--local-addr".into(),
        "192.0.2.10".into(),
    ];
    args.extend(extra.iter().cloned());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    common::run(&refs)
}

#[test]
fn watch_captures_a_target_by_identity_launch_agnostic() {
    // US1: no authored profile, no steam://; the target starts and watch catches
    // it. The synthesized single stage is the target role.
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("watch.jsonl");
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--sink".into(),
            format!("jsonl:{}", jsonl.to_string_lossy()),
        ],
    );
    assert_eq!(code, 0, "watch captures like run/tap: {err}");
    let text = fs::read_to_string(&jsonl).unwrap();
    assert!(
        text.contains("\"role\":\"target\""),
        "the synthesized single stage is the target role: {text}"
    );
}

#[test]
fn watch_honors_the_path_anchor() {
    // US1/SC-006: the path anchor is part of the identity. A matching anchor
    // acquires; a non-matching one does not (and times out).
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("watch.jsonl");
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--path".into(),
            "Games".into(), // C:\Games\game.exe contains it
            "--sink".into(),
            format!("jsonl:{}", jsonl.to_string_lossy()),
        ],
    );
    assert_eq!(code, 0, "the path anchor matches, so watch captures: {err}");

    // A non-matching anchor: the same process is not the target, so with a wait
    // bound the target is never acquired and the run exits one.
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--path".into(),
            "no-such-directory".into(),
            "--wait".into(),
            "1s".into(),
        ],
    );
    assert_eq!(
        code, 1,
        "a non-matching path anchor acquires nothing: {err}"
    );
    assert!(err.contains("never acquired"), "{err}");
}

#[test]
fn watch_attaches_to_an_already_running_target() {
    // US2: the target is already running at arm (in the startup snapshot) with no
    // start event; attach-to-running acquires it and the cascade reports the
    // observed answer naming it.
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("watch.jsonl");
    let (code, _out, err) = watch_offline(
        "game-snapshot.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--sink".into(),
            format!("jsonl:{}", jsonl.to_string_lossy()),
        ],
    );
    assert_eq!(code, 0, "attach-to-running captures: {err}");
    assert!(
        err.contains("attached to already-running pid 4242"),
        "the cascade reports the observed attach: {err}"
    );
    assert!(
        err.contains("(observed)"),
        "the attach is stamped observed by the runtime-observation provider: {err}"
    );
    let text = fs::read_to_string(&jsonl).unwrap();
    assert!(
        text.contains("\"role\":\"target\""),
        "the already-running process was captured: {text}"
    );
}

#[test]
fn watch_gives_up_loudly_when_the_target_never_appears() {
    // US3/P-4: a never-appearing target with a wait bound ends with the named
    // acquisition timeout, surfaced, exit one.
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--exe".into(),
            "never.exe".into(),
            "--wait".into(),
            "1s".into(),
        ],
    );
    assert_eq!(
        code, 1,
        "target never acquired is an expected failure: {err}"
    );
    assert!(err.contains("never acquired"), "{err}");
    assert!(err.contains("acquisition-timeout"), "{err}");
}

#[test]
fn watch_output_is_byte_identical_to_an_equivalent_profile_capture() {
    // FR-007/SC-004: watch reuses the shared capture engine, so its output equals
    // a `run` with a profile identical to what watch synthesizes.
    let dir = tempfile::tempdir().unwrap();

    let watch_out = dir.path().join("watch.fcapng");
    let (code, _o, err) = watch_offline(
        "game.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--out".into(),
            watch_out.to_string_lossy().into_owned(),
        ],
    );
    assert_eq!(code, 0, "watch capture: {err}");

    let run_out = dir.path().join("run.fcapng");
    let run_args = [
        "run",
        "--profile",
        &data("watch-equiv.json"),
        "--out",
        &run_out.to_string_lossy(),
        "--replay-source",
        &fixture("udp-gameplay.pcap"),
        "--attr-script",
        &fixture("udp-gameplay.script"),
        "--process-script",
        &data("game.procscript"),
        "--local-addr",
        "192.0.2.10",
    ];
    let (code, _o, err) = common::run(&run_args);
    assert_eq!(code, 0, "equivalent profile capture: {err}");

    let watch_bytes = fs::read(&watch_out).unwrap();
    let run_bytes = fs::read(&run_out).unwrap();
    assert_eq!(
        watch_bytes, run_bytes,
        "watch output equals an equivalent single-stage profile capture"
    );
}

#[test]
fn watch_warns_when_a_path_anchor_cannot_check_an_already_running_process() {
    // Honesty (review of PR #84): the startup snapshot carries only the
    // executable name, so a path anchor cannot be checked against an
    // already-running process. When one whose executable matches is running,
    // watch says so rather than let a silent wait-until-timeout look like nothing
    // is running. The already-running game does not restart, so with a wait bound
    // it times out (exit one), but the warning names the reason.
    let (code, _out, err) = watch_offline(
        "game-snapshot.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--path".into(),
            "Mod Organizer 2".into(),
            "--wait".into(),
            "1s".into(),
        ],
    );
    assert_eq!(code, 1, "the already-running target is not attached: {err}");
    assert!(
        err.contains("path anchor cannot be checked against the startup snapshot"),
        "watch names why the path-anchored already-running target was not attached: {err}"
    );
}

#[test]
fn watch_without_an_exe_is_a_usage_error() {
    let (code, _out, _err) = common::run(&["watch"]);
    assert_eq!(code, 2);
}

#[test]
fn watch_with_a_bad_path_regex_is_a_usage_error() {
    // FR-008/SC-005: a non-compiling path regex is refused at construction with
    // the profile's own diagnostic, exit two.
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--exe".into(),
            "game.exe".into(),
            "--path-regex".into(),
            "(unclosed".into(),
        ],
    );
    assert_eq!(code, 2, "a bad path regex is a configuration error: {err}");
}
