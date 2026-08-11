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
    // `run` requires --profile.
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
        // An empty role in the list.
        &["run", "-p", "x", "--roles", "client,"],
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
fn each_stub_reports_not_yet_implemented_names_its_slice_and_exits_two() {
    // `steam` was a stub until slice S17 delivered it; it now has its own wiring
    // tests in cli_steam.rs. `replay` and `extcap` remain stubs.
    for (command, slice) in [("replay", "S15"), ("extcap", "S18")] {
        let (code, _out, err) = run(&[command]);
        assert_eq!(code, 2, "{command} exits 2");
        assert!(err.contains("not yet implemented"), "{command}: {err}");
        assert!(err.contains(slice), "{command} must name {slice}: {err}");
    }
}
