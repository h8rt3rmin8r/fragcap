// SPDX-License-Identifier: Apache-2.0

//! `targets add`/`list`/`show` integration tests (slice S051).
//!
//! These drive the CLI surface end to end over a scratch `local.db`, proving the
//! registration, listing, and selector-resolution path including the ambiguous
//! selector's exit-2 refusal to guess (P-9).

mod common;

use std::path::Path;

use common::run;
use tempfile::TempDir;

/// The scratch store path as a string.
fn db(dir: &TempDir) -> String {
    dir.path().join("local.db").to_string_lossy().into_owned()
}

#[test]
fn seed_signatures_populates_the_catalog_and_is_idempotent() {
    let dir = TempDir::new().expect("tempdir");
    let catalog = dir.path().join("catalog.db").to_string_lossy().into_owned();

    let (code, out, _err) = run(&["catalog", "seed-signatures", "--db", &catalog]);
    assert_eq!(code, 0, "seed-signatures succeeds: {out}");
    assert!(
        out.contains("detection signatures"),
        "reports the seeded count: {out}"
    );

    // Idempotent: re-running succeeds and reports the same count.
    let (code2, out2, _err) = run(&["catalog", "seed-signatures", "--db", &catalog]);
    assert_eq!(code2, 0);
    assert_eq!(out, out2, "re-seeding is idempotent");
}

#[test]
fn scan_with_a_catalog_detects_and_reports_evidence() {
    let dir = TempDir::new().expect("tempdir");
    let catalog = dir.path().join("catalog.db").to_string_lossy().into_owned();
    // Seed the signature table, then point scan at a Unity game directory.
    let (code, _out, _err) = run(&["catalog", "seed-signatures", "--db", &catalog]);
    assert_eq!(code, 0);

    let game = dir.path().join("MyGame");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::write(game.join("UnityPlayer.dll"), b"").unwrap();
    let game_path = game.to_string_lossy().into_owned();

    let (code, out, _err) = run(&["targets", "scan", &game_path, "--catalog-db", &catalog]);
    assert_eq!(code, 0, "scan succeeds: {out}");
    assert!(
        out.contains("Unity"),
        "detected engine reported as evidence: {out}"
    );
    assert!(
        out.contains("verified"),
        "a definitive engine is verified: {out}"
    );

    // Without a catalog, scan still works but carries no evidence.
    let (code, out, _err) = run(&["targets", "scan", &game_path]);
    assert_eq!(code, 0);
    assert!(
        !out.contains("Unity"),
        "no detection without a catalog: {out}"
    );
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

#[test]
fn no_match_exit_codes_distinguish_the_selector_kind() {
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
    // A handle/name miss is a clean miss: exit 0.
    let (code, out, _err) = run(&["targets", "show", "nonexistent", "--db", &store]);
    assert_eq!(code, 0, "text-selector miss is a clean 0: {out}");
    // An unknown --id is a bad machine reference: exit 2.
    let (code, _o, _e) = run(&["targets", "show", "--id", "123456789", "--db", &store]);
    assert_eq!(code, 2, "unknown --id is a usage error");
    // An out-of-range row index is a bad reference: exit 2.
    let (code, _o, _e) = run(&["targets", "show", "99", "--db", &store]);
    assert_eq!(code, 2, "out-of-range row index is a usage error");
}

#[test]
fn a_non_canonical_anchor_prefix_merges_with_the_canonical_one() {
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
    // STEAM:620 canonicalizes to steam:620, so it is the same identity.
    let (code, out, _err) = run(&[
        "targets",
        "add",
        "Portal 2 again",
        "--db",
        &store,
        "--anchor",
        "STEAM:620",
    ]);
    assert_eq!(code, 0);
    assert!(
        out.contains("already registered"),
        "a non-canonical prefix resolves to the same identity: {out}"
    );
}

#[test]
fn a_unicode_name_resolves_case_insensitively() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    // A stored name with non-ASCII letters must match a differently-cased selector,
    // which SQLite's ASCII-only NOCASE would miss.
    run(&[
        "targets",
        "add",
        "Pokémon Élan",
        "--db",
        &store,
        "--anchor",
        "steam:1",
    ]);
    let (code, out, _err) = run(&["targets", "show", "pokémon élan", "--db", &store]);
    assert_eq!(code, 0, "Unicode-cased name resolves: {out}");
    assert!(out.contains("handle:"));
}

// --- Discovery subcommands (slice S052) ------------------------------------

/// Write a minimal fixture Steam root: one installed title under the root
/// library's `steamapps`. `discover_in` reads the implicit root library, so no
/// `libraryfolders.vdf` is needed.
fn write_steam_fixture(root: &Path) {
    let steamapps = root.join("steamapps");
    std::fs::create_dir_all(&steamapps).expect("steamapps dir");
    std::fs::write(
        steamapps.join("appmanifest_620.acf"),
        "\"AppState\"\n{\n  \"appid\" \"620\"\n  \"name\" \"Portal 2\"\n  \
         \"installdir\" \"Portal 2\"\n}\n",
    )
    .expect("write manifest");
}

#[test]
fn scan_lists_a_directory_as_one_candidate() {
    let (code, out, _err) = run(&["targets", "scan", "D:/Games/Celeste"]);
    assert_eq!(code, 0, "scan succeeds: {out}");
    assert!(
        out.contains("Celeste"),
        "the pointed-at directory is listed: {out}"
    );
    assert!(
        out.contains("directory"),
        "attributed to the directory source: {out}"
    );
    assert!(
        out.contains("account:"),
        "the conserved account is surfaced: {out}"
    );
}

#[test]
fn discover_lists_steam_titles_through_the_cli() {
    let dir = TempDir::new().expect("tempdir");
    let steam_root = dir.path().join("steam");
    write_steam_fixture(&steam_root);

    let catalog = dir.path().join("catalog.db").to_string_lossy().into_owned();
    let local = dir.path().join("local.db").to_string_lossy().into_owned();
    let steam_root_s = steam_root.to_string_lossy().into_owned();

    let (code, out, _err) = run(&[
        "targets",
        "discover",
        "--catalog-db",
        &catalog,
        "--local-db",
        &local,
        "--steam-root",
        &steam_root_s,
    ]);
    assert_eq!(code, 0, "discover succeeds: {out}");
    assert!(out.contains("Portal 2"), "a Steam title is listed: {out}");
    assert!(
        out.contains("steam:620"),
        "the appid identity is shown: {out}"
    );
    assert!(out.contains("account:"), "the account is surfaced: {out}");
}
