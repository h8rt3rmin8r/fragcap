// SPDX-License-Identifier: Apache-2.0

//! `profile validate`, `list`, and `show` over fixture profiles and directories
//! a test constructs.

mod common;

use std::fs;

use common::run;

const VALID: &str = r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"game","name":"Test Game"},"stage":[{"role":"client","lifecycle":"session","terminal":true,"match":{"exe":"game.exe"}}]}"#;

/// A profile with several independent mistakes: no game.id, and a stage missing
/// its role and match and carrying an invalid lifecycle.
const INVALID: &str = r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"name":"X"},"stage":[{"lifecycle":"bogus"}]}"#;

#[test]
fn a_valid_profile_by_path_validates_without_repeating_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.json");
    fs::write(&path, VALID).unwrap();
    let path_str = path.to_string_lossy().to_string();

    let (code, out, _err) = run(&["profile", "validate", &path_str]);
    assert_eq!(code, 0);
    assert!(out.contains("is valid"), "{out}");
    // The explicit-path reference is not echoed a second time as "(path ...)".
    assert!(!out.contains("(path"), "the path is not repeated: {out}");
    assert_eq!(
        out.matches(&path_str).count(),
        1,
        "the profile path appears exactly once: {out}"
    );
}

#[test]
fn an_invalid_profile_reports_every_diagnostic_in_one_pass_and_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    fs::write(&path, INVALID).unwrap();

    let (code, _out, err) = run(&["profile", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 2);
    // More than one problem in one invocation: the schema, the id, and the
    // stage faults are all present.
    let diagnostics = err.matches("missing").count();
    assert!(
        diagnostics >= 2,
        "every diagnostic must be reported in one pass, saw: {err}"
    );
}

#[test]
fn list_reports_the_bundled_and_per_directory_counts() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("one.json"), VALID).unwrap();
    fs::write(dir.path().join("two.json"), VALID).unwrap();

    let (code, out, _err) = run(&[
        "profile",
        "--profile-dir",
        &dir.path().to_string_lossy(),
        "list",
    ]);
    assert_eq!(code, 0);
    assert!(out.contains("bundled: 0"), "{out}");
    assert!(
        out.contains(&format!("{}: 2", dir.path().display())),
        "the constructed directory's two profiles are counted: {out}"
    );
}

#[test]
fn show_reports_the_resolved_profile_and_its_source() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.json");
    fs::write(&path, VALID).unwrap();

    let (code, out, _err) = run(&["profile", "show", &path.to_string_lossy()]);
    assert_eq!(code, 0);
    assert!(out.contains("resolved from"), "{out}");
    assert!(out.contains("client"), "the stage is shown: {out}");
}

#[test]
fn a_well_formed_reference_that_resolves_to_nothing_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let (code, _out, _err) = run(&[
        "profile",
        "--profile-dir",
        &dir.path().to_string_lossy(),
        "show",
        "nonexistent",
    ]);
    assert_eq!(
        code, 1,
        "a valid but unresolvable reference is an expected failure"
    );
}

#[test]
fn show_and_validate_agree_on_the_exit_for_a_reference_that_resolves_to_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_string_lossy().to_string();

    // An absent id-slug: both exit 1.
    let (show_slug, _, _) = run(&["profile", "--profile-dir", &d, "show", "nonexistent"]);
    let (val_slug, _, _) = run(&["profile", "--profile-dir", &d, "validate", "nonexistent"]);
    assert_eq!(show_slug, 1, "show on an absent slug exits 1");
    assert_eq!(val_slug, 1, "validate on an absent slug exits 1");

    // An unresolvable path-shaped reference: both exit 1 as well (the reclassified
    // InvalidReference), never a split between 1 and 2.
    let missing = dir.path().join("missing.json");
    let missing = missing.to_string_lossy().to_string();
    let (show_path, _, _) = run(&["profile", "show", &missing]);
    let (val_path, _, _) = run(&["profile", "validate", &missing]);
    assert_eq!(show_path, 1, "show on a missing path exits 1");
    assert_eq!(val_path, 1, "validate on a missing path exits 1");
}

#[test]
fn a_profile_file_that_exists_but_is_invalid_still_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    fs::write(&path, INVALID).unwrap();
    let (code, _out, _err) = run(&["profile", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 2, "an invalid profile file is a configuration error");
}

#[test]
fn validate_json_emits_one_event_per_diagnostic_and_a_summary() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    fs::write(&path, INVALID).unwrap();

    let (code, out, _err) = run(&["--json", "profile", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 2);

    let lines: Vec<serde_json::Value> = out
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).expect("each line is JSON"))
        .collect();
    let diagnostics: Vec<&serde_json::Value> = lines
        .iter()
        .filter(|v| v["event"] == "diagnostic")
        .collect();
    assert!(
        diagnostics.len() >= 2,
        "one structured record per diagnostic, saw: {out}"
    );
    for d in &diagnostics {
        assert!(d["code"].is_string(), "each diagnostic carries a code: {d}");
        assert!(d["path"].is_string(), "each diagnostic carries a path: {d}");
        assert!(
            d["message"].is_string(),
            "each diagnostic carries a message: {d}"
        );
    }
    let summary = lines
        .iter()
        .find(|v| v["event"] == "summary")
        .expect("a terminal summary record");
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["diagnostics"], diagnostics.len());
}

#[test]
fn validate_json_on_a_valid_profile_is_a_clean_summary() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.json");
    fs::write(&path, VALID).unwrap();

    let (code, out, _err) = run(&["--json", "profile", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 0);
    let lines: Vec<serde_json::Value> = out
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).expect("each line is JSON"))
        .collect();
    assert!(
        lines.iter().all(|v| v["event"] != "diagnostic"),
        "a valid profile emits no diagnostic records: {out}"
    );
    let summary = lines
        .iter()
        .find(|v| v["event"] == "summary")
        .expect("a terminal summary record");
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["diagnostics"], 0);
}

#[test]
fn list_json_emits_structured_counts() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("one.json"), VALID).unwrap();
    fs::write(dir.path().join("two.json"), VALID).unwrap();

    let (code, out, _err) = run(&[
        "--json",
        "profile",
        "--profile-dir",
        &dir.path().to_string_lossy(),
        "list",
    ]);
    assert_eq!(code, 0);
    let record: serde_json::Value =
        serde_json::from_str(out.lines().next().expect("a record")).expect("JSON");
    assert_eq!(record["event"], "profiles");
    assert_eq!(record["bundled"], 0);
    assert_eq!(record["user_total"], 2);
    assert!(
        record["directories"]
            .as_array()
            .expect("directories array")
            .iter()
            .any(|d| d["count"] == 2),
        "the constructed directory's two profiles are counted: {out}"
    );
}
