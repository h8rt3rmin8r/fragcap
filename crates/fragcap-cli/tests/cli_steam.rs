// SPDX-License-Identifier: Apache-2.0

//! `steam` wiring after the S054 surface rework: `steam list` enumerates installed
//! titles, and registering a title as a target moved to `targets add --steam`
//! (the retired `steam profile` scaffolding is gone).
//!
//! These run offline on any machine. Whether or not Steam is installed, none
//! panics and none is the pre-S17 `not yet implemented` stub.

mod common;

use common::run;

#[test]
fn steam_list_is_wired_and_not_a_stub() {
    // With Steam installed the listing succeeds (exit 0); without it, discovery
    // reports no installation (a usage error, exit 2). Either way it is really
    // wired, not the old stub.
    let (code, _out, err) = run(&["steam", "list"]);
    assert!(code == 0 || code == 2, "unexpected exit {code}: {err}");
    assert!(
        !err.contains("not yet implemented"),
        "the steam stub path is still reachable: {err}"
    );
}

#[test]
fn targets_add_steam_for_an_unknown_app_id_is_refused_cleanly() {
    // Registering an installed Steam title is now `targets add --steam`. An app id
    // that is not installed (or no Steam at all) is a clean configuration refusal
    // (exit 2), not a panic and not a stub.
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local.db");
    let (code, _out, err) = run(&[
        "targets",
        "add",
        "--db",
        &db.to_string_lossy(),
        "--steam",
        "fragcap-no-such-app",
    ]);
    assert_eq!(code, 2, "an unknown steam app is refused: {err}");
    assert!(
        !err.contains("not yet implemented"),
        "the steam stub path is still reachable: {err}"
    );
}

#[test]
fn steam_without_a_subcommand_is_a_usage_error() {
    // clap refuses a missing required subcommand with exit 2.
    let (code, _out, _err) = run(&["steam"]);
    assert_eq!(code, 2);
}

#[test]
fn steam_profile_is_retired() {
    // The retired `steam profile` scaffolding no longer parses.
    let (code, _out, _err) = run(&["steam", "profile", "620"]);
    assert_eq!(code, 2, "steam profile is gone");
}
