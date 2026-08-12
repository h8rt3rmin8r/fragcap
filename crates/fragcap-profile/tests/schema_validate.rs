// SPDX-License-Identifier: Apache-2.0

//! US1: validate any file, every mistake at once.

use std::path::PathBuf;

use fragcap_profile::jsonschema::{validate_json, SchemaCode, SchemaDiagnostics, Validation};

fn fixture(name: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/schema");
    p.push(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn checked(name: &str) -> SchemaDiagnostics {
    match validate_json(&fixture(name)) {
        Validation::Checked(d) => d,
        Validation::Malformed(m) => panic!("{name} should parse but was malformed: {m}"),
    }
}

#[test]
fn four_faults_reports_exactly_four_distinct_locations() {
    let d = checked("profile-four-faults.json");
    assert_eq!(d.len(), 4, "expected four diagnostics, got: {d}");
    let mut pointers: Vec<String> = d.iter().map(|x| x.pointer.clone()).collect();
    pointers.sort();
    pointers.dedup();
    assert_eq!(
        pointers.len(),
        4,
        "the four faults are at distinct locations"
    );
}

#[test]
fn diagnostic_output_is_stable_across_runs() {
    assert_eq!(
        checked("profile-four-faults.json").to_string(),
        checked("profile-four-faults.json").to_string(),
        "reporting order must be deterministic"
    );
}

#[test]
fn a_valid_profile_has_no_diagnostics() {
    let d = checked("profile-valid.json");
    assert!(d.is_empty(), "unexpected diagnostics: {d}");
}

#[test]
fn an_unknown_key_is_reported_not_ignored() {
    let d = checked("profile-four-faults.json");
    assert!(d.has(SchemaCode::UnknownKey));
    assert!(
        d.iter().any(|x| x.pointer == "/extra"),
        "the unknown key is located by pointer"
    );
}

#[test]
fn an_unsupported_version_names_the_supported_one() {
    let d = checked("unsupported-version.json");
    assert!(d.has(SchemaCode::UnsupportedSchema));
    assert!(
        d.iter().any(|x| x.message.contains("version 1")),
        "the message names the supported version"
    );
}

#[test]
fn an_absent_or_unknown_kind_is_an_undetermined_variant() {
    let d = checked("unknown-kind.json");
    assert!(d.has(SchemaCode::UnknownKind));
}

#[test]
fn broken_json_is_malformed_and_distinct_from_a_schema_violation() {
    match validate_json(&fixture("not-json.json")) {
        Validation::Malformed(_) => {}
        Validation::Checked(_) => panic!("not-json.json must be reported as malformed JSON"),
    }
}
