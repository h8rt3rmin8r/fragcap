// SPDX-License-Identifier: Apache-2.0

//! `schema validate` and `schema print` over files a test constructs.

mod common;

use std::fs;

use common::run;

const VALID: &str = r#"{
  "schema": 1,
  "kind": "profile",
  "fidelity": "verified",
  "game": { "id": "game", "name": "Test Game" },
  "stage": [ { "role": "client", "lifecycle": "session", "terminal": true, "match": { "exe": "game.exe" } } ]
}
"#;

// Three independent faults: an unknown key, a missing game.name, an empty match.
const INVALID: &str = r#"{
  "schema": 1,
  "kind": "profile",
  "fidelity": "verified",
  "extra": true,
  "game": { "id": "game" },
  "stage": [ { "role": "client", "lifecycle": "session", "match": {} } ]
}
"#;

const NOT_JSON: &str = "{ this is not valid json,\n";

#[test]
fn a_valid_file_validates_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("target.json");
    fs::write(&path, VALID).unwrap();

    let (code, out, _err) = run(&["schema", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 0, "out={out}");
    assert!(out.contains("is valid"), "{out}");
}

#[test]
fn an_invalid_file_reports_every_violation_and_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    fs::write(&path, INVALID).unwrap();

    let (code, _out, err) = run(&["schema", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown-key"), "{err}");
    assert!(err.contains("missing-field"), "{err}");
    assert!(err.contains("empty-match"), "{err}");
}

#[test]
fn broken_json_reports_a_syntax_error_and_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.json");
    fs::write(&path, NOT_JSON).unwrap();

    let (code, _out, err) = run(&["schema", "validate", &path.to_string_lossy()]);
    assert_eq!(code, 2);
    assert!(err.contains("not valid JSON"), "{err}");
}

#[test]
fn a_missing_file_is_an_expected_failure_exit_one() {
    let (code, _out, err) = run(&["schema", "validate", "does-not-exist.json"]);
    assert_eq!(code, 1, "a file that cannot be read is exit 1: {err}");
}

#[test]
fn print_emits_the_schema_document() {
    let (code, out, _err) = run(&["schema", "print"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("\"$id\": \"https://fragcap.com/schema/target/v1.json\""),
        "print emits the master schema"
    );
    assert!(out.contains("json-schema.org/draft/2020-12"), "{out}");
}
