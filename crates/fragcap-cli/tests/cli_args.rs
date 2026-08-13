// SPDX-License-Identifier: Apache-2.0

//! The value grammars, the exit-code table, the help surface, and the stubs.
//!
//! Drives the library `run_with` entry directly. No process is spawned and no
//! capture driver, elevation, or game is involved.

mod common;

use common::run;

#[test]
fn help_lists_all_seven_commands() {
    let (code, out, _err) = run(&["--help"]);
    assert_eq!(code, 0, "help is a success");
    for command in [
        "run", "tap", "replay", "profile", "steam", "doctor", "extcap",
    ] {
        assert!(
            out.contains(command),
            "`--help` does not list `{command}`:\n{out}"
        );
    }
}

#[test]
fn version_prints_and_succeeds() {
    let (code, out, _err) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.contains("fragcap"), "version output: {out}");
}

#[test]
fn a_bad_subcommand_is_a_usage_error() {
    let (code, _out, _err) = run(&["nonesuch"]);
    assert_eq!(code, 2);
}

#[test]
fn a_missing_required_flag_is_a_usage_error() {
    // `run` requires exactly one target input (--profile / --install-dir /
    // --steam); supplying none is a usage error.
    let (code, _out, _err) = run(&["run"]);
    assert_eq!(code, 2);
    // `tap` requires --process.
    let (code, _out, _err) = run(&["tap"]);
    assert_eq!(code, 2);
}

#[test]
fn the_value_grammars_reject_bad_values_with_exit_two() {
    let cases: &[&[&str]] = &[
        // A bare integer duration has no unit.
        &["run", "-p", "x", "--duration", "30"],
        // A bare integer size has no unit.
        &["run", "-p", "x", "--max-bytes", "4"],
        // A sink with no scheme.
        &["run", "-p", "x", "--sink", "out.fcapng"],
        // A sink with an unknown scheme.
        &["run", "-p", "x", "--sink", "bogus:x"],
        // A bad direction.
        &["run", "-p", "x", "--direction", "sideways"],
        // A ring window that is neither a duration nor a size.
        &["run", "-p", "x", "--ring", "nonsense"],
    ];
    for case in cases {
        let (code, _out, _err) = run(case);
        assert_eq!(code, 2, "expected a usage error for {case:?}");
    }
}

#[test]
fn the_replay_stub_reports_not_yet_implemented_without_an_internal_slice_id() {
    // `steam` and `extcap` were stubs until they were delivered; both now have
    // their own wiring tests (cli_steam.rs, cli_extcap.rs). `replay` remains the
    // one stub, and its message carries no internal roadmap identifier (#67).
    let (code, _out, err) = run(&["replay"]);
    assert_eq!(code, 2, "replay exits 2");
    assert!(err.contains("not yet implemented"), "replay: {err}");
    assert!(
        !err.contains("S15") && !err.contains("slice"),
        "replay must not leak an internal slice id: {err}"
    );
}
