// SPDX-License-Identifier: Apache-2.0

//! Schema conformance: a well-formed seed imports and re-exports valid, and a
//! malformed one is rejected, both by the schema validator directly and by the
//! importer, with no partial store written.

use fragcap_profile::jsonschema::{validate_json, SchemaCode, Validation};
use fragcap_targets::{export, import, Store, TargetsError};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn diagnostics_of(text: &str) -> fragcap_profile::jsonschema::SchemaDiagnostics {
    match validate_json(text) {
        Validation::Checked(d) => d,
        Validation::Malformed(m) => panic!("fixture is not valid JSON: {m}"),
    }
}

#[test]
fn valid_seed_conforms_to_the_schema() {
    assert!(validate_json(&fixture("seed.json")).is_valid());
}

#[test]
fn valid_seed_imports_and_reexports_valid() {
    let mut store = Store::open_in_memory().unwrap();
    let summary = import(&mut store, &fixture("seed.json")).unwrap();
    assert_eq!(summary.imported, 3);
    let text = export(&store).unwrap();
    assert!(
        validate_json(&text).is_valid(),
        "re-export of an imported seed must validate; got: {text}"
    );
}

#[test]
fn launch_entry_missing_executable_is_rejected() {
    let text = fixture("malformed-launch.json");
    assert!(
        diagnostics_of(&text).has(SchemaCode::MissingField),
        "a launch entry with no executable must raise missing-field"
    );
    let mut store = Store::open_in_memory().unwrap();
    let err = import(&mut store, &text).unwrap_err();
    assert!(matches!(err, TargetsError::Seed(_)), "got {err:?}");
    // No partial store: nothing was written.
    assert!(store.games().unwrap().is_empty());
}

#[test]
fn out_of_set_engine_source_is_rejected() {
    let text = fixture("malformed-engine.json");
    assert!(
        diagnostics_of(&text).has(SchemaCode::InvalidEngineSource),
        "an out-of-set engine source must raise invalid-engine-source"
    );
    let mut store = Store::open_in_memory().unwrap();
    let err = import(&mut store, &text).unwrap_err();
    assert!(matches!(err, TargetsError::Seed(_)), "got {err:?}");
    assert!(store.games().unwrap().is_empty());
}
