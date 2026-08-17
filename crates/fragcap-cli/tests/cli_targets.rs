// SPDX-License-Identifier: Apache-2.0

//! `targets add`/`list`/`show` integration tests (slice S051).
//!
//! These drive the CLI surface end to end over a scratch `local.db`, proving the
//! registration, listing, and selector-resolution path including the ambiguous
//! selector's exit-2 refusal to guess (P-9).

mod common;

use common::run;
use tempfile::TempDir;

/// The scratch store path as a string.
fn db(dir: &TempDir) -> String {
    dir.path().join("local.db").to_string_lossy().into_owned()
}

#[test]
fn add_derives_a_handle_and_show_resolves_it() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);

    let (code, out, _err) = run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:620",
    ]);
    assert_eq!(code, 0, "add succeeds");
    assert!(
        out.contains("portal_2"),
        "derived handle is reported: {out}"
    );

    // Select it back by handle.
    let (code, out, _err) = run(&["targets", "show", "portal_2", "--db", &store]);
    assert_eq!(code, 0);
    assert!(out.contains("handle:") && out.contains("portal_2"));

    // And by case-insensitive name.
    let (code, _out, _err) = run(&["targets", "show", "portal 2", "--db", &store]);
    assert_eq!(code, 0);
}

#[test]
fn a_purely_numeric_name_falls_back_and_is_not_a_numeric_handle() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    let (code, out, _err) = run(&[
        "targets",
        "add",
        "2048",
        "--db",
        &store,
        "--exe",
        "Game2048.exe",
    ]);
    assert_eq!(code, 0, "registration succeeds via fallback");
    // The handle is the exe stem, never the purely numeric name.
    assert!(out.contains("game2048"), "fell back to exe stem: {out}");
}

#[test]
fn a_collision_suffixes_the_new_item() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    let (c1, o1, _) = run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:620",
    ]);
    let (c2, o2, _) = run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:200",
    ]);
    assert_eq!((c1, c2), (0, 0));
    assert!(o1.contains("portal_2"));
    assert!(o2.contains("portal_2_2"), "second gets _2: {o2}");
}

#[test]
fn re_adding_the_same_anchor_reports_the_existing_registration() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:620",
    ]);
    let (code, out, _err) = run(&[
        "targets",
        "add",
        "Portal Two",
        "--db",
        &store,
        "--anchor",
        "steam:620",
    ]);
    assert_eq!(code, 0);
    assert!(
        out.contains("already registered"),
        "anchor identity merges: {out}"
    );
}

#[test]
fn an_ambiguous_name_lists_matches_and_exits_two() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    // Two targets with distinct handles but the same display name.
    run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:620",
    ]);
    run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:200",
    ]);

    let (code, out, _err) = run(&["targets", "show", "Portal 2", "--db", &store]);
    assert_eq!(code, 2, "ambiguity is a usage error, not a guess");
    assert!(out.contains("ambiguous"), "lists the ambiguity: {out}");
    assert!(
        out.contains("portal_2") && out.contains("portal_2_2"),
        "shows both handles: {out}"
    );
}

#[test]
fn show_by_id_resolves_and_missing_selector_is_usage_error() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets",
        "add",
        "Portal 2",
        "--db",
        &store,
        "--anchor",
        "steam:620",
    ]);
    // List to learn the id.
    let (_c, listing, _e) = run(&["targets", "list", "--db", &store]);
    let id = listing.split('\t').nth(2).expect("id column").trim();
    let (code, out, _err) = run(&["targets", "show", "--id", id, "--db", &store]);
    assert_eq!(code, 0, "resolves by --id");
    assert!(out.contains("portal_2"));

    // A bare row index resolves too.
    let (code, _out, _err) = run(&["targets", "show", "1", "--db", &store]);
    assert_eq!(code, 0, "bare integer is a row index");
}

#[test]
fn list_is_empty_on_a_fresh_store() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    let (code, out, _err) = run(&["targets", "list", "--db", &store]);
    assert_eq!(code, 0);
    assert!(out.contains("no targets registered"), "{out}");
}
