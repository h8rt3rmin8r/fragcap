// SPDX-License-Identifier: Apache-2.0

mod common;

use common::run;
use tempfile::TempDir;

#[test]
fn missing_or_invalid_seed_source_does_not_create_a_store() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("catalog.db");
    let missing = dir.path().join("missing.json");
    let (code, _, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "catalog",
        "--from",
        &missing.to_string_lossy(),
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_ne!(code, 0, "missing source must fail: {err}");
    assert!(!db.exists(), "bad input must precede store creation");

    let invalid = dir.path().join("invalid.json");
    std::fs::write(&invalid, r#"{"kind":"export","targets":[]}"#).expect("write input");
    let (code, _, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "engine",
        "--from",
        &invalid.to_string_lossy(),
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_ne!(code, 0, "export envelope is not an engine array: {err}");
    assert!(!db.exists(), "invalid input must precede store creation");
}

#[test]
fn explicit_unsupported_launch_tier_has_no_store_effect() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("catalog.db");
    let (code, out, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "launch",
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_eq!(
        code, 2,
        "unsupported action must be a usage error: {out} {err}"
    );
    assert!(!db.exists());
    let (code, out, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "signature",
        "--tier",
        "launch",
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_eq!(
        code, 2,
        "unsupported tier must not cause partial seeding: {out} {err}"
    );
    assert!(!db.exists());
}

#[test]
fn repeated_tier_counts_only_one_attempt() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("catalog.db");
    let (code, out, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "signature",
        "--tier",
        "signature",
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.matches("signature tier:").count(), 1, "{out}");
    assert!(out.contains("(1 tier(s) filled)"), "{out}");
}

#[test]
fn default_seed_and_offline_array_report_actual_work() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("catalog.db");
    let (code, out, err) = run(&["catalog", "seed", "--db", &db.to_string_lossy()]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("(1 tier(s) filled)"), "{out}");
    assert!(out.contains("title tier: skipped"), "{out}");
    #[cfg(not(feature = "net"))]
    {
        assert!(
            !out.contains("--steam"),
            "default build must not advise unavailable flags: {out}"
        );
        assert!(!out.contains("--pcgamingwiki"), "{out}");
    }

    let document = dir.path().join("titles.json");
    std::fs::write(&document, "[]").expect("write bare array");
    let (code, out, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "catalog",
        "--from",
        &document.to_string_lossy(),
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("title tier: fetched 0 written 0"), "{out}");
    assert!(out.contains("(1 tier(s) filled)"), "{out}");
}

#[test]
fn bad_source_preserves_an_existing_store() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("catalog.db");
    let (code, _, err) = run(&[
        "catalog",
        "seed",
        "--tier",
        "signature",
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "{err}");
    let before = std::fs::read(&db).expect("store bytes");
    let missing = dir.path().join("missing.json");
    let (code, _, _) = run(&[
        "catalog",
        "seed",
        "--tier",
        "catalog",
        "--from",
        &missing.to_string_lossy(),
        "--db",
        &db.to_string_lossy(),
    ]);
    assert_ne!(code, 0);
    assert_eq!(std::fs::read(&db).expect("store bytes"), before);
}

#[test]
fn seed_help_names_supported_offline_document_shapes() {
    let (code, out, _) = run(&["catalog", "seed", "--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("bare JSON array"), "{out}");
    assert!(out.contains("catalog export"), "{out}");
    assert!(out.contains("no seeder"), "{out}");
}
