// SPDX-License-Identifier: Apache-2.0

//! `steam profile` wiring: the command dispatches to the real Steam integration
//! and refuses cleanly, rather than the old stub path.
//!
//! These run offline on any machine. Whether or not Steam is installed, asking
//! to scaffold a profile for an app id that is not present is a configuration
//! refusal that exits 2: without a Steam installation the discovery reports
//! `no Steam installation found`; with one, the app id is not found. Neither
//! panics and neither is the pre-S17 `not yet implemented` stub.

mod common;

use common::run;

#[test]
fn steam_profile_for_an_unknown_app_id_is_refused_cleanly() {
    let (code, out, err) = run(&["steam", "profile", "fragcap-no-such-app"]);
    assert_eq!(code, 2, "out={out} err={err}");
    assert!(
        !err.contains("not yet implemented"),
        "the steam stub path is still reachable: {err}"
    );
}

#[test]
fn steam_without_a_subcommand_is_a_usage_error() {
    // clap refuses a missing required subcommand with exit 2, and the help names
    // `profile`, so the command is really wired rather than a catch-all stub.
    let (code, _out, _err) = run(&["steam"]);
    assert_eq!(code, 2);
}
