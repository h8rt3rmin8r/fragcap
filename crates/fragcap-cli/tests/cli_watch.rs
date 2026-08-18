// SPDX-License-Identifier: Apache-2.0

//! `capture --process` identity behaviors over the offline substrate: the path
//! anchor, attach-to-running, and the loud give-up, all the launch-agnostic
//! capture the retired `watch` carried. No capture driver, no elevation, no game.

mod common;

use std::fs;

use common::{data, fixture};

/// A `capture --process` invocation over the standard offline substrate, appending
/// `extra`. The identity flags (`--process`, `--path`, `--wait`) come from `extra`.
fn watch_offline(procscript: &str, extra: &[String]) -> (u8, String, String) {
    let mut args: Vec<String> = vec![
        "capture".into(),
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
fn capture_by_identity_is_launch_agnostic() {
    // No authored profile, no steam://; the target starts and capture catches it.
    // The synthesized single stage is the target role.
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("watch.jsonl");
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--process".into(),
            "game.exe".into(),
            "--sink".into(),
            format!("jsonl:{}", jsonl.to_string_lossy()),
        ],
    );
    assert_eq!(code, 0, "capture by process identity: {err}");
    let text = fs::read_to_string(&jsonl).unwrap();
    assert!(
        text.contains("\"role\":\"target\""),
        "the synthesized single stage is the target role: {text}"
    );
}

#[test]
fn capture_honors_the_path_anchor() {
    // The path anchor is part of the identity. A matching anchor acquires; a
    // non-matching one does not (and times out).
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("watch.jsonl");
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--process".into(),
            "game.exe".into(),
            "--path".into(),
            "Games".into(), // C:\Games\game.exe contains it
            "--sink".into(),
            format!("jsonl:{}", jsonl.to_string_lossy()),
        ],
    );
    assert_eq!(
        code, 0,
        "the path anchor matches, so capture succeeds: {err}"
    );

    // A non-matching anchor: the same process is not the target, so with a wait
    // bound the target is never acquired and the run exits one.
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--process".into(),
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
fn capture_attaches_to_an_already_running_target() {
    // The target is already running at arm (in the startup snapshot) with no start
    // event; attach-to-running acquires it and the cascade reports the observed
    // answer naming it.
    let dir = tempfile::tempdir().unwrap();
    let jsonl = dir.path().join("watch.jsonl");
    let (code, _out, err) = watch_offline(
        "game-snapshot.procscript",
        &[
            "--process".into(),
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
fn capture_gives_up_loudly_when_the_target_never_appears() {
    // A never-appearing target with a wait bound ends with the named acquisition
    // timeout, surfaced, exit one (P-4).
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--process".into(),
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
fn capture_warns_when_a_path_anchor_cannot_check_an_already_running_process() {
    // The startup snapshot carries only the executable name, so a path anchor
    // cannot be checked against an already-running process. When one whose
    // executable matches is running, capture says so rather than let a silent
    // wait-until-timeout look like nothing is running (review of PR #84).
    let (code, _out, err) = watch_offline(
        "game-snapshot.procscript",
        &[
            "--process".into(),
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
        "capture names why the path-anchored already-running target was not attached: {err}"
    );
}

#[test]
fn capture_with_a_bad_path_regex_is_a_usage_error() {
    // A non-compiling path regex is refused at construction with the profile's own
    // diagnostic, exit two.
    let (code, _out, err) = watch_offline(
        "game.procscript",
        &[
            "--process".into(),
            "game.exe".into(),
            "--path-regex".into(),
            "(unclosed".into(),
        ],
    );
    assert_eq!(code, 2, "a bad path regex is a configuration error: {err}");
}
