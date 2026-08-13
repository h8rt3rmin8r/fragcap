// SPDX-License-Identifier: Apache-2.0

//! US3: the schema is authoritative and cannot drift.
//!
//! The conformance corpus binds the published schema document to the
//! hand-rolled validator: every fixture has a declared expected outcome, and the
//! validator must produce it. The drift tests bind the embedded schema to the
//! repository-published copy.

use std::path::PathBuf;

use fragcap_profile::jsonschema::{schema_document, validate_json, SchemaCode, Validation};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(name: &str) -> String {
    let mut p = manifest();
    p.push("tests/fixtures/schema");
    p.push(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Expected outcome for one fixture.
enum Expect {
    Valid,
    Invalid(SchemaCode),
    Malformed,
}

#[test]
fn the_conformance_corpus_matches_the_validator() {
    use Expect::*;
    let corpus = [
        ("profile-valid.json", Valid),
        ("package-valid.json", Valid),
        ("hint-valid.json", Valid),
        ("export-valid.json", Valid),
        ("export-envelope.json", Valid),
        ("profile-four-faults.json", Invalid(SchemaCode::UnknownKey)),
        ("profile-with-records.json", Invalid(SchemaCode::UnknownKey)),
        (
            "hint-no-fidelity.json",
            Invalid(SchemaCode::MissingFidelity),
        ),
        (
            "hint-no-provenance.json",
            Invalid(SchemaCode::MissingProvenance),
        ),
        ("unknown-kind.json", Invalid(SchemaCode::UnknownKind)),
        (
            "unsupported-version.json",
            Invalid(SchemaCode::UnsupportedSchema),
        ),
        ("profile-technologies-valid.json", Valid),
        ("profile-technologies-empty.json", Valid),
        (
            "technology-missing-category.json",
            Invalid(SchemaCode::MissingField),
        ),
        (
            "technology-bad-category.json",
            Invalid(SchemaCode::InvalidCategory),
        ),
        ("hint-loose-valid.json", Valid),
        ("export-loose-record.json", Valid),
        (
            "engine-bad-source.json",
            Invalid(SchemaCode::InvalidEngineSource),
        ),
        (
            "engine-bad-confidence.json",
            Invalid(SchemaCode::InvalidEngineConfidence),
        ),
        (
            "launch-no-executable.json",
            Invalid(SchemaCode::MissingField),
        ),
        ("profile-with-launch.json", Invalid(SchemaCode::UnknownKey)),
        ("not-json.json", Malformed),
    ];

    for (name, expect) in corpus {
        let outcome = validate_json(&fixture(name));
        match (expect, &outcome) {
            (Valid, Validation::Checked(d)) => {
                assert!(d.is_empty(), "{name} should be valid, got: {d}");
            }
            (Invalid(code), Validation::Checked(d)) => {
                assert!(!d.is_empty(), "{name} should be invalid");
                assert!(d.has(code), "{name} should carry {code}, got: {d}");
            }
            (Malformed, Validation::Malformed(_)) => {}
            (_, other) => panic!("{name}: outcome did not match expectation: {other:?}"),
        }
    }
}

#[test]
fn print_output_equals_the_embedded_asset() {
    // `schema print` writes exactly schema_document(); assert that equals the
    // asset file the binary embeds.
    let mut asset = manifest();
    asset.push("assets/target-schema.v1.json");
    let asset = std::fs::read_to_string(&asset).expect("read embedded asset");
    assert_eq!(
        schema_document(),
        asset,
        "the emitted schema must equal the embedded asset"
    );
}

#[test]
fn the_embedded_schema_matches_the_published_repository_copy() {
    // repo root is two levels up from crates/fragcap-profile.
    let mut published = manifest();
    published.pop();
    published.pop();
    published.push("docs/schema/target-schema.v1.json");
    let published = std::fs::read_to_string(&published)
        .unwrap_or_else(|e| panic!("read published copy {}: {e}", published.display()));
    assert_eq!(
        schema_document(),
        published,
        "the published docs/schema copy must match the embedded schema; regenerate it on change"
    );
}
