// SPDX-License-Identifier: Apache-2.0

//! End-to-end offline tests for local Steam launch-data accumulation (issue #78,
//! slice S038).
//!
//! These drive the facade's `accumulate_launch_data` orchestrator against a
//! fixture Steam-root tree (text app manifests plus a synthetic binary
//! `appinfo.vdf`) and an in-memory store. No Steam, no network, no capture driver.
//! They live in the CLI crate because it carries the `targets` feature
//! unconditionally, so they run under the default `cargo test --workspace` gate.

use std::path::Path;

use fragcap::accumulate_launch_data;
use fragcap::targets::{Game, LaunchEntry, Store};
use fragcap_steam::appinfo::fixtures::{
    appinfo_bytes, appinfo_bytes_with_bad_section, FixtureApp, FixtureLaunch, V29,
};

fn app(appid: u32, change_number: u32, exe: &str) -> FixtureApp {
    FixtureApp {
        appid,
        change_number,
        launch: vec![FixtureLaunch::windows(exe)],
    }
}

fn manifest(appid: u32, name: &str) -> String {
    format!(
        "\"AppState\"\n{{\n  \"appid\" \"{appid}\"\n  \"name\" \"{name}\"\n  \
         \"installdir\" \"{name}\"\n}}\n"
    )
}

/// Write a fixture Steam root: an app manifest per installed app, and the appinfo
/// cache bytes.
fn write_root(root: &Path, installed: &[(u32, &str)], appinfo: &[u8]) {
    let steamapps = root.join("steamapps");
    std::fs::create_dir_all(&steamapps).expect("mkdir steamapps");
    for (appid, name) in installed {
        std::fs::write(
            steamapps.join(format!("appmanifest_{appid}.acf")),
            manifest(*appid, name),
        )
        .expect("write manifest");
    }
    let appcache = root.join("appcache");
    std::fs::create_dir_all(&appcache).expect("mkdir appcache");
    std::fs::write(appcache.join("appinfo.vdf"), appinfo).expect("write appinfo");
}

fn noop_progress(_: fragcap::AccumulationProgress) {}

#[test]
fn first_run_writes_installed_apps() {
    let dir = tempfile::tempdir().expect("tempdir");
    let appinfo = appinfo_bytes(V29, &[app(1, 10, "a.exe"), app(2, 20, "b.exe")]);
    write_root(dir.path(), &[(1, "A"), (2, "B")], &appinfo);

    let mut store = Store::open_in_memory().expect("store");
    let summary =
        accumulate_launch_data(dir.path(), &mut store, &mut noop_progress).expect("accumulate");

    assert_eq!(summary.considered, 2);
    assert_eq!(summary.written, 2);
    assert!(summary.is_conserved());
    assert_eq!(
        store.game(1).unwrap().unwrap().launch[0].executable(),
        "a.exe"
    );
    assert_eq!(store.stored_change_number(2).unwrap(), Some(20));
}

#[test]
fn second_run_skips_and_a_change_number_bump_refreshes_only_that_app() {
    let dir = tempfile::tempdir().expect("tempdir");
    let appinfo = appinfo_bytes(V29, &[app(1, 10, "a.exe"), app(2, 20, "b.exe")]);
    write_root(dir.path(), &[(1, "A"), (2, "B")], &appinfo);

    let mut store = Store::open_in_memory().expect("store");
    accumulate_launch_data(dir.path(), &mut store, &mut noop_progress).expect("first run");

    // Second run over an unchanged cache: everything already current.
    let second =
        accumulate_launch_data(dir.path(), &mut store, &mut noop_progress).expect("second run");
    assert_eq!(second.skipped, 2);
    assert_eq!(second.written, 0);

    // Advance app 2's change-number (and its exe) in the cache; app 1 unchanged.
    let bumped = appinfo_bytes(V29, &[app(1, 10, "a.exe"), app(2, 21, "b2.exe")]);
    std::fs::write(dir.path().join("appcache").join("appinfo.vdf"), &bumped).expect("rewrite");

    let third =
        accumulate_launch_data(dir.path(), &mut store, &mut noop_progress).expect("third run");
    assert_eq!(third.written, 1, "only the bumped app is re-read");
    assert_eq!(third.skipped, 1);
    assert_eq!(
        store.game(2).unwrap().unwrap().launch[0].executable(),
        "b2.exe",
        "the refreshed launch data replaced the old"
    );
}

#[test]
fn outcomes_conserve_across_a_mixed_library() {
    let dir = tempfile::tempdir().expect("tempdir");
    // App 3's section is malformed; app 4 is installed but absent from the cache.
    let appinfo = appinfo_bytes_with_bad_section(
        V29,
        &[
            app(1, 10, "a.exe"),
            app(2, 20, "b.exe"),
            app(3, 30, "c.exe"),
        ],
        2,
    );
    write_root(
        dir.path(),
        &[(1, "A"), (2, "B"), (3, "C"), (4, "D")],
        &appinfo,
    );

    let mut store = Store::open_in_memory().expect("store");
    // Pre-seed app 2 as already current at change 20, so it is skipped.
    store
        .merge_launch(2, 20, &[LaunchEntry::new("b.exe").unwrap()])
        .expect("pre-seed");

    let summary =
        accumulate_launch_data(dir.path(), &mut store, &mut noop_progress).expect("accumulate");

    assert_eq!(summary.considered, 4);
    assert_eq!(summary.written, 1, "app 1");
    assert_eq!(summary.skipped, 1, "app 2 already current");
    assert_eq!(summary.failed, 1, "app 3 malformed section");
    assert_eq!(summary.empty, 1, "app 4 absent from the cache");
    assert!(summary.is_conserved());
    // The malformed app 3 did not block app 1 from being written.
    assert!(store.game(1).unwrap().is_some());
}

#[test]
fn a_row_with_launch_data_but_no_change_number_is_refreshed() {
    // FR-016: a game carrying authored launch data but no learned change-number
    // (for example from a hand-authored import) is refreshed like any other.
    let dir = tempfile::tempdir().expect("tempdir");
    let appinfo = appinfo_bytes(V29, &[app(1, 5, "real.exe")]);
    write_root(dir.path(), &[(1, "A")], &appinfo);

    let mut store = Store::open_in_memory().expect("store");
    let mut g = Game::new(1);
    g.name = Some("A".to_string());
    g.launch = vec![LaunchEntry::new("authored.exe").unwrap()];
    store.upsert_game(&g).expect("seed authored");
    assert_eq!(store.stored_change_number(1).unwrap(), None);

    let summary =
        accumulate_launch_data(dir.path(), &mut store, &mut noop_progress).expect("accumulate");
    assert_eq!(summary.written, 1, "a null change-number means stale");
    let g1 = store.game(1).unwrap().unwrap();
    assert_eq!(g1.launch.len(), 1);
    assert_eq!(g1.launch[0].executable(), "real.exe");
}
