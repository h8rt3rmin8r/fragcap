// SPDX-License-Identifier: Apache-2.0

//! `profile validate`, `list`, and `show` over fixture profiles and directories
//! a test constructs.

mod common;

use std::fs;

use common::run;

const VALID: &str = "schema = 1\n\
[game]\nid = \"game\"\nname = \"Test Game\"\n\
[[stage]]\nrole = \"client\"\nlifecycle = \"session\"\nterminal = true\n\
match = { exe = \"game.exe\" }\n";

/// A profile with several independent mistakes: no schema, no game.id, and a
/// stage missing its role and carrying an invalid lifecycle.
const INVALID: &str = "[game]\nname = \"X\"\n\
[[stage]]\nlifecycle = \"bogus\"\n";

#[test]
fn a_valid_profile_validates_with_its_source_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.toml");
    fs::write(&path, VALID).unwrap();

    let (code, out, _err) = run(&["profile", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 0);
    assert!(out.contains("is valid"), "{out}");
    assert!(out.contains("path"), "the source is reported: {out}");
}

#[test]
fn an_invalid_profile_reports_every_diagnostic_in_one_pass_and_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.toml");
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
    fs::write(dir.path().join("one.toml"), VALID).unwrap();
    fs::write(dir.path().join("two.toml"), VALID).unwrap();

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
    let path = dir.path().join("game.toml");
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
