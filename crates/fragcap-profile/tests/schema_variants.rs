// SPDX-License-Identifier: Apache-2.0

//! US2: one vocabulary across all four artifact forms.

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

fn check_str(text: &str) -> SchemaDiagnostics {
    match validate_json(text) {
        Validation::Checked(d) => d,
        Validation::Malformed(m) => panic!("should parse but was malformed: {m}"),
    }
}

#[test]
fn a_package_validates_as_the_strict_shape() {
    assert!(checked("package-valid.json").is_empty());
}

#[test]
fn a_hint_validates_while_omitting_profile_required_fields() {
    // hint-valid has no game.id and no stage, which a profile would require.
    assert!(checked("hint-valid.json").is_empty());
}

#[test]
fn a_hint_without_fidelity_is_refused() {
    let d = checked("hint-no-fidelity.json");
    assert!(d.has(SchemaCode::MissingFidelity), "diagnostics: {d}");
}

#[test]
fn a_hint_without_provenance_is_refused() {
    let d = checked("hint-no-provenance.json");
    assert!(d.has(SchemaCode::MissingProvenance), "diagnostics: {d}");
}

#[test]
fn an_export_single_and_envelope_both_validate() {
    assert!(checked("export-valid.json").is_empty(), "single export");
    assert!(
        checked("export-envelope.json").is_empty(),
        "export envelope"
    );
}

#[test]
fn a_fidelity_outside_the_enum_is_refused_in_every_variant() {
    for kind in ["profile", "package", "hint", "export"] {
        let body = match kind {
            "profile" | "package" => format!(
                r#"{{"schema":1,"kind":"{kind}","fidelity":"platinum","game":{{"id":"x","name":"X"}},"stage":[{{"role":"c","lifecycle":"session","match":{{"exe":"x.exe"}}}}]}}"#
            ),
            _ => format!(
                r#"{{"schema":1,"kind":"{kind}","fidelity":"platinum","provenance":{{"source":"user"}}}}"#
            ),
        };
        let d = check_str(&body);
        assert!(
            d.has(SchemaCode::InvalidFidelity),
            "{kind}: expected invalid-fidelity, got: {d}"
        );
    }
}

#[test]
fn a_notes_string_is_accepted() {
    // profile-valid carries a notes string; it must not be flagged.
    let d = checked("profile-valid.json");
    assert!(!d.iter().any(|x| x.pointer == "/notes"), "notes: {d}");
}
