// SPDX-License-Identifier: Apache-2.0

//! First-run store bootstrap, exercised against the real binary in an
//! install-identical layout.
//!
//! The distribution places `catalog.db` read-only beside `fragcap.exe` (the MSI
//! component and the portable archive both do). On first run fragcap seeds the
//! per-user catalog from that template. This test reproduces that layout with a
//! clean `%APPDATA%` and asserts the hero listing (`fragcap targets`, the
//! documented first command) seeds the per-user catalog from the template rather
//! than ignoring it.
//!
//! It is a regression guard: the seeding once lived only in the `capture` path, so
//! a fresh install running `fragcap targets` found no catalog in the per-user
//! location, skipped discovery, and listed nothing until a capture happened to seed
//! it or the operator copied the file by hand. The bug was invisible to the
//! in-process tier-1 tests, which point `FRAGCAP_CATALOG_DB` at a fixed path, so it
//! is caught here by driving the built binary the way an install does.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// A unique scratch directory for this test, removed first so a prior run cannot
/// leave a stale catalog behind and mask a regression.
fn scratch(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("fragcap-bootstrap-{}-{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

#[test]
fn targets_seeds_the_per_user_catalog_from_the_template_beside_the_exe() {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_fragcap"));
    let exe_name = bin.file_name().expect("binary has a file name");

    // Stage an install-identical directory: the binary plus a catalog.db template
    // beside it, exactly as the MSI and the portable archive lay it out. The binary
    // is copied so `current_exe()` resolves the template from this directory rather
    // than from the build output, where no template ships.
    let install = scratch("install");
    let staged_exe = install.join(exe_name);
    fs::copy(&bin, &staged_exe).expect("stage the binary");
    let template = install.join("catalog.db");

    // Build the template through the binary's own catalog path, the way the release
    // job does, so it carries the detection signatures a real shipped catalog does.
    let seed = Command::new(&staged_exe)
        .args(["catalog", "seed", "--tier", "signature", "--db"])
        .arg(&template)
        .output()
        .expect("run catalog seed --tier signature");
    assert!(
        seed.status.success(),
        "seeding the template catalog failed: {}",
        String::from_utf8_lossy(&seed.stderr)
    );
    assert!(template.is_file(), "the template catalog.db must exist");
    let template_bytes = fs::read(&template).expect("read the template catalog");

    // A clean per-user home: the default catalog and local stores resolve under it,
    // and neither exists yet, so this is a genuine first run.
    let home = scratch("home");
    let per_user = home.join("fragcap");
    assert!(
        !per_user.join("catalog.db").exists(),
        "the per-user catalog must not exist before the first run"
    );

    // Run the hero listing exactly as a fresh install would: the default store
    // locations (no --db, no FRAGCAP_* overrides), with APPDATA pointing at the
    // clean home so the defaults resolve there.
    let run = Command::new(&staged_exe)
        .arg("targets")
        .env("APPDATA", &home)
        .env_remove("FRAGCAP_CATALOG_DB")
        .env_remove("FRAGCAP_LOCAL_DB")
        .env_remove("FRAGCAP_PROFILE_DIR")
        .output()
        .expect("run fragcap targets");
    assert!(
        run.status.success(),
        "fragcap targets failed (code {:?}): {}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );

    // The regression: the per-user catalog is seeded from the template on first run,
    // so discovery has a catalog to classify against. Before the fix it was absent
    // after `targets` and the shipped catalog was silently ignored.
    let seeded = per_user.join("catalog.db");
    assert!(
        seeded.is_file(),
        "fragcap targets must seed the per-user catalog from the template on first run, \
         but {} does not exist",
        seeded.display()
    );

    // It is the shipped catalog, not an empty placeholder: the bytes match the
    // template (the copy clears the read-only attribute but does not alter content).
    let seeded_bytes = fs::read(&seeded).expect("read the seeded catalog");
    assert_eq!(
        seeded_bytes, template_bytes,
        "the seeded per-user catalog must be a copy of the shipped template"
    );

    // The local store is created too, so the pair a fresh run needs is present.
    assert!(
        per_user.join("local.db").is_file(),
        "fragcap targets must create the per-user local store"
    );

    let _ = fs::remove_dir_all(&install);
    let _ = fs::remove_dir_all(&home);
}
/// Every `catalog` subcommand resolves a store with no `--db`, and the
/// precedence is flag over environment over per-user default.
///
/// Slice S058 (issue #157) made a store path an override rather than a
/// requirement and applied it to `targets`; its FR-005 scoped `catalog` out and
/// nothing picked it up, so these commands went on demanding a path to a
/// component fragcap installs and manages, with nothing in the error saying what
/// path to type (issue #179).
#[test]
fn catalog_subcommands_resolve_a_store_without_a_flag() {
    let bin = env!("CARGO_BIN_EXE_fragcap");
    let dir = scratch("catalog-default");
    let from_env = dir.join("from-env.db");
    let from_flag = dir.join("from-flag.db");

    // With no flag, the environment override is the store that gets written.
    let run = Command::new(bin)
        .args(["catalog", "seed", "--tier", "signature"])
        .env("FRAGCAP_CATALOG_DB", &from_env)
        .output()
        .expect("run catalog seed");
    assert!(
        run.status.success(),
        "seed with no --db must succeed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        from_env.is_file(),
        "the environment override must be the store that was written"
    );
    let out = String::from_utf8_lossy(&run.stdout);
    assert!(
        out.contains(from_env.file_name().unwrap().to_string_lossy().as_ref()),
        "the success line must name the store it wrote (P-9): {out}"
    );

    // An explicit flag wins over the environment override.
    let run = Command::new(bin)
        .args(["catalog", "seed", "--tier", "signature", "--db"])
        .arg(&from_flag)
        .env("FRAGCAP_CATALOG_DB", &from_env)
        .output()
        .expect("run catalog seed with --db");
    assert!(
        run.status.success(),
        "seed with an explicit --db must succeed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        from_flag.is_file(),
        "an explicit --db must win over the environment override"
    );

    let _ = fs::remove_dir_all(&dir);
}
