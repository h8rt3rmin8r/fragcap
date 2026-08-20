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
    // A scratch local store, so these scan calls register into their own db rather
    // than the shared default store other tests observe.
    let local = db(&dir);
    // Seed the signature table, then point scan at a Unity game directory.
    let (code, _out, _err) = run(&["catalog", "seed-signatures", "--db", &catalog]);
    assert_eq!(code, 0);

    let game = dir.path().join("MyGame");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::write(game.join("UnityPlayer.dll"), b"").unwrap();
    let game_path = game.to_string_lossy().into_owned();

    let (code, out, _err) = run(&[
        "targets",
        "scan",
        &game_path,
        "--catalog-db",
        &catalog,
        "--db",
        &local,
    ]);
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
    let (code, out, _err) = run(&["targets", "scan", &game_path, "--db", &local]);
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
    // Show by handle to learn the id (the listing is now a formatted table).
    let (_c, shown, _e) = run(&["targets", "show", "portal_2", "--db", &store]);
    let id = shown
        .lines()
        .find_map(|l| l.strip_prefix("id:"))
        .expect("id line")
        .trim();
    let (code, out, _err) = run(&["targets", "show", "--id", id, "--db", &store]);
    assert_eq!(code, 0, "resolves by --id");
    assert!(out.contains("portal_2"));

    // A bare row index resolves too, against the snapshot the listing writes.
    run(&["targets", "list", "--db", &store]);
    let (code, _out, _err) = run(&["targets", "show", "1", "--db", &store]);
    assert_eq!(code, 0, "bare integer is a row index");
}

#[test]
fn hero_listing_shows_columns_ordered_by_handle_and_names_the_next_command() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    // Register two targets out of handle order; a steam anchor makes them ready.
    run(&[
        "targets", "add", "Zeta", "--db", &store, "--anchor", "steam:2",
    ]);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);

    let (code, out, _err) = run(&["targets", "list", "--db", &store]);
    assert_eq!(code, 0, "{out}");
    // The header and both readiness/evidence columns are present.
    assert!(
        out.contains("TARGET") && out.contains("CAPTURE") && out.contains("KNOWN"),
        "{out}"
    );
    // Ordered by handle: alpha before zeta.
    let alpha = out.find("alpha").expect("alpha row");
    let zeta = out.find("zeta").expect("zeta row");
    assert!(alpha < zeta, "rows are handle-ordered:\n{out}");
    // A steam-anchored target is ready and the listing names the next command.
    assert!(out.contains("ready"), "{out}");
    assert!(
        out.contains("fragcap capture 1"),
        "ends by naming the next command:\n{out}"
    );
}

#[test]
fn a_row_index_resolves_against_the_snapshot_after_a_mutation() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);
    run(&[
        "targets", "add", "Zeta", "--db", &store, "--anchor", "steam:2",
    ]);

    // List pins the snapshot: row 1 = alpha, row 2 = zeta.
    run(&["targets", "list", "--db", &store]);

    // Register a target that sorts first in the live order.
    run(&[
        "targets", "add", "Beta", "--db", &store, "--anchor", "steam:3",
    ]);

    // Row 2 still resolves to zeta (the row the user saw), not the shifted live
    // order where beta would now occupy row 2.
    let (code, out, _err) = run(&["targets", "show", "2", "--db", &store]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("zeta"),
        "row 2 is the snapshot's row, not the live order:\n{out}"
    );

    // A row index past the snapshot is an out-of-range usage error.
    let (code, _out, _err) = run(&["targets", "show", "9", "--db", &store]);
    assert_eq!(code, 2, "an out-of-range row index is a usage error");
}

#[test]
fn socket_holder_answer_decides_the_launch_chain() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);

    // `yes`: the exe is the client, so the row is ready.
    run(&[
        "targets",
        "add",
        "Yes Game",
        "--db",
        &store,
        "--exe",
        "yes.exe",
        "--socket-holder",
        "yes",
    ]);
    // `unsure`: unresolved chain, so the row needs a target and nothing is fabricated.
    run(&[
        "targets",
        "add",
        "Unsure Game",
        "--db",
        &store,
        "--exe",
        "maybe.exe",
        "--socket-holder",
        "unsure",
    ]);
    // `no`: a different process holds the sockets, so the row needs a target too.
    run(&[
        "targets",
        "add",
        "No Game",
        "--db",
        &store,
        "--exe",
        "launcher.exe",
        "--socket-holder",
        "no",
    ]);

    let (code, out, _err) = run(&["targets", "list", "--db", &store]);
    assert_eq!(code, 0, "{out}");
    let ready_line = out
        .lines()
        .find(|l| l.contains("yes_game"))
        .expect("yes row");
    assert!(ready_line.contains("ready"), "yes is ready: {ready_line}");
    let unsure_line = out
        .lines()
        .find(|l| l.contains("unsure_game"))
        .expect("unsure row");
    assert!(
        unsure_line.contains("needs a target"),
        "unsure needs a target: {unsure_line}"
    );
    let no_line = out.lines().find(|l| l.contains("no_game")).expect("no row");
    assert!(
        no_line.contains("needs a target"),
        "no needs a target: {no_line}"
    );
}

#[test]
fn exe_without_a_socket_holder_answer_is_unresolved_not_a_fabricated_client() {
    // Non-interactive `add --exe X` with no `--socket-holder` must not assume the
    // executable is the client (P-9): the row is unresolved, needs a target.
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets",
        "add",
        "Launcher Game",
        "--db",
        &store,
        "--exe",
        "launcher.exe",
    ]);
    let (code, out, _err) = run(&["targets", "list", "--db", &store]);
    assert_eq!(code, 0, "{out}");
    let line = out
        .lines()
        .find(|l| l.contains("launcher_game"))
        .expect("row");
    assert!(
        line.contains("needs a target"),
        "unresolved, not fabricated: {line}"
    );
}

#[test]
fn export_preserves_the_selector_kind_on_a_miss() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);
    run(&["targets", "list", "--db", &store]);

    // An unmatched handle/name is a clean miss: empty array, exit 0.
    let (code, out, _err) = run(&["targets", "export", "nonesuch", "--db", &store]);
    assert_eq!(code, 0, "unmatched name is a clean miss: {out}");
    assert_eq!(out.trim(), "[]", "emits an empty array: {out}");

    // An out-of-range row index and an unknown --id are invalid machine references.
    let (code, _o, _e) = run(&["targets", "export", "9", "--db", &store]);
    assert_eq!(code, 2, "out-of-range row index is a usage error");
    let (code, _o, _e) = run(&["targets", "export", "--id", "999999", "--db", &store]);
    assert_eq!(code, 2, "unknown --id is a usage error");
}

#[test]
fn zero_and_out_of_range_row_indexes_are_usage_errors() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);
    run(&["targets", "list", "--db", &store]);
    let (code, _o, _e) = run(&["targets", "show", "0", "--db", &store]);
    assert_eq!(
        code, 2,
        "row index 0 is a usage error, not a clean name miss"
    );
    let (code, _o, _e) = run(&["targets", "remove", "0", "--db", &store]);
    assert_eq!(code, 2, "remove 0 is a usage error too");
}

#[test]
fn import_rejects_a_constraint_violating_batch_whole() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Existing", "--db", &store, "--anchor", "steam:1",
    ]);
    // A two-element import: a clean new row, then one whose new stable id reuses the
    // existing handle. The whole batch must roll back, leaving only "existing".
    let bad = dir.path().join("bad.json");
    std::fs::write(
        &bad,
        r#"[
          {"stable_id":2,"handle":"fresh","name":"Fresh","classification":"game","classification_source":"user","fidelity":"authored"},
          {"stable_id":3,"handle":"existing","name":"Dup","classification":"game","classification_source":"user","fidelity":"authored"}
        ]"#,
    )
    .expect("write");
    let bad = bad.to_string_lossy().into_owned();
    let (code, _o, _e) = run(&["targets", "import", &bad, "--db", &store]);
    assert_eq!(code, 2, "a constraint-violating batch is rejected");
    // Only the original row survived; "fresh" was rolled back.
    let (_c, shown, _e) = run(&["targets", "show", "fresh", "--db", &store]);
    assert!(
        shown.contains("no target matches"),
        "fresh was rolled back: {shown}"
    );
}

#[test]
fn socket_holder_requires_exe_and_rejects_a_bad_value() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    // Without --exe the flag is a usage error, not a blocking prompt.
    let (code, _out, _err) = run(&[
        "targets",
        "add",
        "Game",
        "--db",
        &store,
        "--socket-holder",
        "yes",
    ]);
    assert_eq!(code, 2, "--socket-holder requires --exe");
    // A bad value is a usage error.
    let (code, _out, _err) = run(&[
        "targets",
        "add",
        "Game",
        "--db",
        &store,
        "--exe",
        "g.exe",
        "--socket-holder",
        "maybe",
    ]);
    assert_eq!(code, 2, "an unknown socket-holder value is a usage error");
}

#[test]
fn remove_deletes_exactly_one_and_ambiguous_refuses() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);
    run(&[
        "targets", "add", "Beta", "--db", &store, "--anchor", "steam:2",
    ]);

    let (code, out, _err) = run(&["targets", "remove", "alpha", "--db", &store]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("removed alpha"), "{out}");
    // Alpha is gone; beta remains.
    let (_c, shown, _e) = run(&["targets", "show", "alpha", "--db", &store]);
    assert!(shown.contains("no target matches"), "{shown}");
    let (_c, beta, _e) = run(&["targets", "show", "beta", "--db", &store]);
    assert!(beta.contains("beta"), "beta remains: {beta}");

    // Two targets sharing a name are ambiguous: remove refuses (exit 2).
    run(&[
        "targets", "add", "Same", "--db", &store, "--anchor", "steam:3",
    ]);
    run(&[
        "targets", "add", "Same", "--db", &store, "--anchor", "steam:4",
    ]);
    let (code, out, _err) = run(&["targets", "remove", "Same", "--db", &store]);
    assert_eq!(code, 2, "ambiguous remove refuses: {out}");
    assert!(out.contains("ambiguous"), "{out}");
}

#[test]
fn export_round_trips_through_import_with_identical_ids() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);
    run(&[
        "targets", "add", "Beta", "--db", &store, "--anchor", "steam:2",
    ]);

    // Export to a file.
    let (code, doc, _err) = run(&["targets", "export", "--db", &store]);
    assert_eq!(code, 0, "{doc}");
    let file = dir.path().join("targets.json");
    std::fs::write(&file, &doc).expect("write export");
    let file = file.to_string_lossy().into_owned();

    // Import into a fresh store.
    let fresh = dir.path().join("fresh.db").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["targets", "import", &file, "--db", &fresh]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("imported 2 new"), "{out}");

    // Re-exporting the fresh store yields the same document (identical ids, no dupes).
    let (code, doc2, _err) = run(&["targets", "export", "--db", &fresh]);
    assert_eq!(code, 0, "{doc2}");
    assert_eq!(doc, doc2, "export round-trips identically");

    // A second import is idempotent on identity (updates in place).
    let (code, out, _err) = run(&["targets", "import", &file, "--db", &fresh]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("imported 0 new, 2 updated"),
        "idempotent: {out}"
    );
}

#[test]
fn import_rejects_a_nonconforming_file_whole() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("local.db").to_string_lossy().into_owned();
    let file = dir.path().join("bad.json");
    std::fs::write(&file, "{ not an array }").expect("write");
    let file = file.to_string_lossy().into_owned();
    let (code, _out, _err) = run(&["targets", "import", &file, "--db", &store]);
    assert_eq!(code, 2, "a nonconforming file is a usage error");
}

#[test]
fn list_is_empty_on_a_fresh_store() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    let (code, out, _err) = run(&["targets", "list", "--db", &store]);
    assert_eq!(code, 0);
    // The empty case names the commands that populate the store (FR-006, SC-006).
    assert!(out.contains("No targets yet."), "{out}");
    assert!(out.contains("fragcap targets add"), "{out}");
    assert!(out.contains("fragcap targets scan"), "{out}");
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
    // A scratch local store, so this registers into its own db rather than the shared
    // default store other tests observe.
    let dir = TempDir::new().expect("tempdir");
    let local = db(&dir);
    let (code, out, _err) = run(&["targets", "scan", "D:/Games/Celeste", "--db", &local]);
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

#[test]
fn a_subcommand_defaults_to_the_local_store_when_db_is_omitted() {
    // Slice S058 (#157): a targets subcommand run with no `--db` resolves the same
    // default local store the bare `fragcap targets` command uses. The test harness
    // points `FRAGCAP_LOCAL_DB` at a per-process scratch store, so an add with no
    // `--db` writes there and a show with no `--db` reads the same store back. This is
    // the only test in this binary that operates on that default store without `--db`
    // (the scan tests pass an explicit `--db` for exactly this reason), so it does not
    // race another test's connection to the shared store.
    let name = "S058 Default Store Game";
    let (add_code, _out, add_err) = run(&[
        "targets",
        "add",
        name,
        "--exe",
        "game.exe",
        "--socket-holder",
        "unsure",
    ]);
    assert_eq!(add_code, 0, "add against the default store: {add_err}");

    let (show_code, show_out, show_err) = run(&["targets", "show", name]);
    assert_eq!(
        show_code, 0,
        "show resolves the same default store with no --db: {show_err}"
    );
    assert!(
        show_out.contains("game.exe") || show_out.to_lowercase().contains("s058"),
        "the target added with no --db is found by show with no --db: {show_out}"
    );
}
/// A numeric selector that matches no row says what it did with the number.
///
/// Issue #181. An operator read a table of Steam app ids from `fragcap steam
/// list`, passed one to `--target`, and was told only "no target matches". The
/// number was resolved as a listing row index, because `is_row_index` gates
/// first and never falls through to a handle or name lookup, so the operator's
/// number was interpreted in a namespace they did not know existed. The
/// resolver knew all of that and reported none of it, which is the P-9 half of
/// the defect: an observation was made and withheld.
#[test]
fn a_numeric_selector_miss_names_the_interpretation_and_the_listing_size() {
    let dir = TempDir::new().expect("tempdir");
    let store = db(&dir);
    run(&[
        "targets", "add", "Alpha", "--db", &store, "--anchor", "steam:1",
    ]);
    // Write a listing snapshot so the reported size is a real one.
    run(&["targets", "list", "--db", &store]);

    let (_code, out, _err) = run(&["targets", "show", "999999", "--db", &store]);
    assert!(
        out.contains("was read as a listing row number"),
        "the message names the interpretation it used: {out}"
    );
    assert!(
        out.contains("the listing has "),
        "the message names the size of the space it searched: {out}"
    );
    assert!(
        out.contains("targets add --steam 999999"),
        "the message names the route to the app-id namespace: {out}"
    );

    // A non-numeric miss keeps the short message: there is no second namespace
    // to disambiguate, so the long form would be noise.
    let (_c, plain, _e) = run(&["targets", "show", "no-such-handle", "--db", &store]);
    assert!(
        plain.contains("no target matches"),
        "a name miss still reports cleanly: {plain}"
    );
    assert!(
        !plain.contains("was read as a listing row number"),
        "a name miss does not claim a row-index interpretation: {plain}"
    );
}
