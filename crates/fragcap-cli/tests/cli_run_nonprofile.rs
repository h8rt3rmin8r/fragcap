// SPDX-License-Identifier: Apache-2.0

//! `run` over the non-profile capture path: resolve a target from an install
//! location with no authored profile, synthesize a heuristic-unverified identity,
//! and capture it over the offline substrate. No capture driver, no game, no
//! Steam install.

mod common;

use std::fs;

use common::{data, fixture, run};

/// A `run` invocation over the standard offline substrate, appending `extra`.
fn run_offline(extra: &[String]) -> (u8, String, String) {
    let mut args: Vec<String> = vec![
        "run".into(),
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
    run(&refs)
}

#[test]
fn run_captures_a_nonprofile_target_from_an_install_dir() {
    // US1/SC-001: an install directory with one clear client and no profile. The
    // platform walker resolves game.exe; run synthesizes a one-stage
    // heuristic-unverified identity for it and captures it exactly as a profile
    // run would, launch-agnostic.
    let install = tempfile::tempdir().unwrap();
    fs::write(install.path().join("game.exe"), b"MZ placeholder client").unwrap();
    let out = tempfile::tempdir().unwrap();
    let jsonl = out.path().join("run.jsonl");

    let (code, _out, err) = run_offline(&[
        "--install-dir".into(),
        install.path().to_string_lossy().into_owned(),
        "--sink".into(),
        format!("jsonl:{}", jsonl.to_string_lossy()),
    ]);

    assert_eq!(
        code, 0,
        "non-profile capture succeeds like a profile run: {err}"
    );
    let text = fs::read_to_string(&jsonl).unwrap();
    assert!(
        text.contains("\"role\":\"target\""),
        "the synthesized single stage is the target role: {text}"
    );
}

#[test]
fn an_install_dir_with_no_client_is_a_surfaced_failure() {
    // US3/SC-004: an empty install directory resolves to nothing. The command
    // fails (exit 1) naming the reason and captures nothing (P-4).
    let empty = tempfile::tempdir().unwrap();
    let (code, _out, err) = run(&["run", "--install-dir", &empty.path().to_string_lossy()]);
    assert_eq!(
        code, 1,
        "an unresolved install dir is a surfaced failure: {err}"
    );
    assert!(
        err.contains("could not resolve a capture target"),
        "the failure names the reason: {err}"
    );
}

#[test]
fn an_unreadable_install_dir_is_a_surfaced_failure() {
    // US3/SC-004: a nonexistent directory is unreadable, surfaced distinctly from
    // a directory that scanned and matched nothing.
    let missing = std::env::temp_dir().join(format!(
        "fragcap-run-nonprofile-absent-{}",
        std::process::id()
    ));
    let (code, _out, err) = run(&["run", "--install-dir", &missing.to_string_lossy()]);
    assert_eq!(
        code, 1,
        "an unreadable install dir is a surfaced failure: {err}"
    );
    assert!(
        err.contains("could not resolve a capture target"),
        "the failure names the reason: {err}"
    );
}

#[test]
fn a_not_installed_steam_app_is_a_surfaced_failure() {
    // US2/SC-002: app id 0 is never installed. The command fails (exit 1) and
    // captures nothing, whether Steam is present (not-installed) or absent (lookup
    // error).
    let (code, _out, err) = run(&["run", "--steam", "0"]);
    assert_eq!(
        code, 1,
        "a not-installed steam app is a surfaced failure: {err}"
    );
}

#[test]
fn the_three_target_inputs_are_mutually_exclusive_and_one_is_required() {
    // FR-005/SC-005: exactly one of --profile / --install-dir / --steam.
    // None: a usage error.
    let (code, _out, _err) = run(&["run"]);
    assert_eq!(code, 2, "no target input is a usage error");
    // Two: a usage error (a conflict), reported before any resolution.
    let (code, _out, _err) = run(&["run", "--profile", "x", "--install-dir", "y"]);
    assert_eq!(code, 2, "profile + install-dir is a usage error");
    let (code, _out, _err) = run(&["run", "--install-dir", "a", "--steam", "b"]);
    assert_eq!(code, 2, "install-dir + steam is a usage error");
}
