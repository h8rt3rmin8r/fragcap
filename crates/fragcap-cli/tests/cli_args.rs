// SPDX-License-Identifier: Apache-2.0

//! The value grammars, the exit-code table, the help surface, and the stubs.
//!
//! Drives the library `run_with` entry directly. No process is spawned and no
//! capture driver, elevation, or game is involved.

mod common;

use common::run;

#[test]
fn help_lists_the_grouped_command_surface() {
    let (code, out, _err) = run(&["--help"]);
    assert_eq!(code, 0, "help is a success");
    // The four presentational headings (section 17.3), nothing hidden.
    for heading in ["Capture:", "Targets:", "Environment:", "Data:"] {
        assert!(
            out.contains(heading),
            "`--help` is missing the `{heading}` group:\n{out}"
        );
    }
    // Every command appears exactly once under a heading.
    for command in [
        "capture",
        "replay",
        "targets",
        "technologies",
        "steam",
        "doctor",
        "extcap",
        "catalog",
        "schema",
    ] {
        assert!(
            out.contains(command),
            "`--help` does not list `{command}`:\n{out}"
        );
    }
    // The retired verbs are gone.
    for gone in ["run ", "tap ", "watch ", "profile "] {
        assert!(
            !out.contains(gone),
            "`--help` still lists retired command `{gone}`:\n{out}"
        );
    }
}

const FOOTER: &str = "Run `fragcap --help` to see all commands.";

#[test]
fn bare_invocation_lists_targets_with_a_footer() {
    // A bare `fragcap` runs the targets listing and appends the `--help` footer
    // (section 17.4). Both calls read the same default store, so they differ only by
    // the footer, whatever targets happen to be registered.
    let (bare_code, bare_out, _err) = run(&[]);
    assert_eq!(bare_code, 0, "bare invocation is a success");
    assert!(
        bare_out.contains(FOOTER),
        "bare `fragcap` appends the footer:\n{bare_out}"
    );

    let (targets_code, targets_out, _err) = run(&["targets"]);
    assert_eq!(targets_code, 0, "explicit targets is a success");
    assert!(
        !targets_out.contains(FOOTER),
        "explicit `targets` omits the footer:\n{targets_out}"
    );

    // The two listings are identical except for the footer line.
    assert_eq!(
        bare_out.replace(FOOTER, "").trim_end(),
        targets_out.trim_end(),
        "bare and explicit listings differ only by the footer"
    );
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
    // `capture` requires exactly one target input (a positional selector, --target,
    // --id, or --process); supplying none is a usage error.
    let (code, _out, _err) = run(&["capture"]);
    assert_eq!(code, 2);
}

#[test]
fn a_positional_selector_is_accepted_as_a_target() {
    // `fragcap capture <n>` (the form the `targets` listing hints, and the README
    // and site docs show) is a valid target input: the positional selector is
    // equivalent to `--target`. It must reach target resolution rather than be
    // rejected by the parser as an unexpected argument. Explicit throwaway stores
    // keep this off the shared default store the other tests use (so it never races
    // another test's connection to it); resolving against its own empty local store
    // yields no match, a usage error (exit 2) with a resolution message, not a
    // parse error.
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.db");
    let catalog = dir.path().join("catalog.db");
    let (code, _out, err) = run(&[
        "capture",
        "1",
        "--local-db",
        local.to_str().unwrap(),
        "--catalog-db",
        catalog.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "an unresolvable positional selector is a usage error"
    );
    assert!(
        !err.contains("unexpected argument"),
        "the positional selector must be accepted by the parser: {err}"
    );
}

#[test]
fn a_positional_selector_and_target_flag_conflict() {
    // The positional selector and --target are two members of the same required
    // group; giving both is a usage error before any resolution.
    let (code, _out, err) = run(&["capture", "1", "--target", "2"]);
    assert_eq!(code, 2, "two target inputs conflict");
    assert!(
        err.contains("cannot be used with") || err.contains("unexpected"),
        "the conflict is reported by the parser: {err}"
    );
}

#[test]
fn the_value_grammars_reject_bad_values_with_exit_two() {
    let cases: &[&[&str]] = &[
        // A bare integer duration has no unit.
        &["capture", "--process", "x", "--duration", "30"],
        // A bare integer size has no unit.
        &["capture", "--process", "x", "--max-bytes", "4"],
        // A sink with no scheme.
        &["capture", "--process", "x", "--sink", "out.fcapng"],
        // A sink with an unknown scheme.
        &["capture", "--process", "x", "--sink", "bogus:x"],
        // A bad direction.
        &["capture", "--process", "x", "--direction", "sideways"],
        // A ring window that is neither a duration nor a size.
        &["capture", "--process", "x", "--ring", "nonsense"],
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
