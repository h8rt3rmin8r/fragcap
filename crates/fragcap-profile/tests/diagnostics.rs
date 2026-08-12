// SPDX-License-Identifier: Apache-2.0

//! Being wrong well, over JSON profiles.
//!
//! Specification section 15.4 requires that validation report every problem found
//! rather than stopping at the first, so the tests here are as much about how many
//! diagnostics come back as about which. Every [`DiagnosticCode`] that a profile
//! can produce is exercised by at least one case; a code that cannot be produced
//! is indistinguishable from one that is wired up wrong.
//!
//! Structural faults (types, required keys, enum ranges, unknown keys, an empty
//! match, missing or empty stages) are owned by the master-schema validator and
//! mapped into these codes; the fragcap layer owns glob, regex, and duration
//! compilation, the stage-count limit, and the semantic graph checks. This file
//! covers both, and the all-errors-at-once property across them.

use std::fs;

use fragcap_profile::{
    load, DiagnosticCode, Diagnostics, LoadError, Profile, MAX_PATTERN_CHARS, MAX_PROFILE_BYTES,
    MAX_STAGES,
};

/// Parse text that must be refused, and return everything found.
fn refused(text: &str) -> Diagnostics {
    match Profile::parse(text) {
        Ok(_) => panic!("expected refusal, profile was accepted:\n{text}"),
        Err(d) => {
            assert!(
                !d.is_empty(),
                "a refusal must carry at least one diagnostic, or an author has \
                 nothing to act on"
            );
            d
        }
    }
}

// --- suppression behaviors -------------------------------------------------

#[test]
fn a_json_syntax_fault_is_one_diagnostic_and_suppresses_the_rest() {
    // The document is also missing every required key, so a parser that recovered
    // would report faults against a file the author did not write.
    let d = refused("{ \"schema\": = 1 ");
    assert_eq!(d.len(), 1, "a syntax fault stops accumulation");
    assert!(d.has(DiagnosticCode::Syntax));
}

#[test]
fn the_former_toml_format_is_refused_as_invalid_json() {
    // A leftover TOML profile is not silently accepted; it is not valid JSON.
    let d = refused("schema = 1\n[game]\nid = \"t\"\nname = \"T\"\n");
    assert!(d.has(DiagnosticCode::Syntax));
}

#[test]
fn an_unsupported_schema_version_suppresses_the_semantic_set() {
    // Version 2 with an otherwise-broken body: only the version fault comes back,
    // because the rest is likely a consequence of a newer schema.
    let d = refused(
        r#"{"schema":2,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[]}"#,
    );
    assert!(d.has(DiagnosticCode::UnsupportedSchema));
    assert_eq!(
        d.len(),
        1,
        "an unsupported version suppresses everything else"
    );
}

// --- structural codes (owned by the schema validator) ----------------------

#[test]
fn a_missing_required_field_is_reported() {
    // game.name absent.
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::MissingField));
}

#[test]
fn a_missing_kind_or_fidelity_is_a_missing_field() {
    let d = refused(
        r#"{"schema":1,"game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(
        d.has(DiagnosticCode::MissingField),
        "missing kind and fidelity: {d}"
    );
}

#[test]
fn a_wrong_type_is_reported() {
    // game.name is a number.
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":123},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::WrongType));
}

#[test]
fn an_unknown_key_is_refused_not_ignored() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","payloads":false,"game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::UnknownKey));
}

#[test]
fn a_non_profile_kind_cannot_be_loaded_as_a_profile() {
    // A hint is a valid artifact but not a capture profile.
    let d = refused(
        r#"{"schema":1,"kind":"hint","fidelity":"heuristic-unverified","provenance":{"source":"user"},"game":{"name":"T"}}"#,
    );
    assert!(
        d.has(DiagnosticCode::WrongType),
        "kind hint refused for load: {d}"
    );
}

#[test]
fn an_unrecognized_kind_is_reported_exactly_once() {
    // The structural layer already reports an unknown kind; the load-path check
    // must not add a second diagnostic for the same fault.
    let d = refused(
        r#"{"schema":1,"kind":"bogus","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    let at_kind = d.iter().filter(|x| x.location == "/kind").count();
    assert_eq!(at_kind, 1, "one mistake, one diagnostic at /kind: {d}");
}

#[test]
fn a_root_level_fault_keeps_the_empty_json_pointer() {
    // A non-object document is a root fault; its location is the empty JSON
    // pointer, which a consumer can apply, not a synthetic placeholder.
    let d = refused("[]");
    assert!(
        d.iter().any(|x| x.location.is_empty()),
        "a root fault uses the empty pointer: {d}"
    );
}

#[test]
fn an_invalid_slug_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"Not A Slug","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::InvalidSlug));
}

#[test]
fn an_invalid_lifecycle_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"bogus","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::InvalidLifecycle));
}

#[test]
fn an_invalid_mode_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"capture":{"mode":"bogus"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::InvalidMode));
}

#[test]
fn an_empty_match_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{}}]}"#,
    );
    assert!(d.has(DiagnosticCode::EmptyMatch));
}

#[test]
fn an_empty_or_missing_stage_array_is_reported() {
    let empty = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[]}"#,
    );
    assert!(empty.has(DiagnosticCode::NoStages), "empty stage: {empty}");

    let missing = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"}}"#,
    );
    assert!(
        missing.has(DiagnosticCode::MissingField),
        "missing stage: {missing}"
    );
}

// --- fragcap-specific compilation codes ------------------------------------

#[test]
fn an_invalid_glob_is_reported() {
    let long = "a".repeat(MAX_PATTERN_CHARS + 1);
    let text = format!(
        r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"t","name":"T"}},"stage":[{{"role":"c","lifecycle":"session","match":{{"exe":"{long}"}}}}]}}"#
    );
    let d = refused(&text);
    assert!(d.has(DiagnosticCode::InvalidGlob));
}

#[test]
fn an_invalid_regex_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"path_regex":"("}}]}"#,
    );
    assert!(d.has(DiagnosticCode::InvalidRegex));
}

#[test]
fn an_invalid_duration_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"capture":{"duration":"not-a-duration"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::InvalidDuration));
}

#[test]
fn too_many_stages_is_reported_without_running_the_quadratic_pass() {
    // Every stage shares one exe: if the over-limit array were extracted into the
    // semantic draft, the quadratic ambiguity check would flood the report with
    // AmbiguousImageMatch diagnostics. The limit must short-circuit that.
    let stages: Vec<String> = (0..=MAX_STAGES)
        .map(|i| format!(r#"{{"role":"r{i}","lifecycle":"session","match":{{"exe":"dup.exe"}}}}"#))
        .collect();
    let text = format!(
        r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"t","name":"T"}},"stage":[{}]}}"#,
        stages.join(",")
    );
    let d = refused(&text);
    assert!(d.has(DiagnosticCode::TooManyStages));
    assert!(
        !d.has(DiagnosticCode::AmbiguousImageMatch),
        "the semantic draft must not be populated past the stage limit: {}",
        d.len()
    );
    assert_eq!(
        d.len(),
        1,
        "an over-limit profile reports only the limit: {d}"
    );
}

// --- semantic graph codes --------------------------------------------------

#[test]
fn a_duplicate_role_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"a.exe"}},{"role":"c","lifecycle":"transient","match":{"exe":"b.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::DuplicateRole));
}

#[test]
fn more_than_one_terminal_stage_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"a","lifecycle":"session","terminal":true,"match":{"exe":"a.exe"}},{"role":"b","lifecycle":"session","terminal":true,"match":{"exe":"b.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::MultipleTerminal));
}

#[test]
fn a_terminal_stage_that_is_not_session_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"transient","terminal":true,"match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::TerminalLifecycle));
}

#[test]
fn an_unknown_descends_from_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe","descends_from":"nope"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::UnknownDescendsFrom));
}

#[test]
fn a_descends_from_cycle_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"a","lifecycle":"session","match":{"exe":"a.exe","descends_from":"b"}},{"role":"b","lifecycle":"session","match":{"exe":"b.exe","descends_from":"a"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::DescendsFromCycle));
}

#[test]
fn an_empty_capture_roles_list_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"capture":{"roles":[]},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::EmptyRoles));
}

#[test]
fn an_undeclared_capture_role_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"capture":{"roles":["ghost"]},"stage":[{"role":"c","lifecycle":"session","match":{"exe":"c.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::UndeclaredCaptureRole));
}

#[test]
fn a_profile_of_only_services_is_reported() {
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"a","lifecycle":"service","match":{"exe":"a.exe"}},{"role":"b","lifecycle":"service","match":{"exe":"b.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::AllServices));
}

#[test]
fn an_ambiguous_image_match_is_reported() {
    // Two stages match the same image on exe alone; neither is pinned.
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"a","lifecycle":"session","match":{"exe":"game.exe"}},{"role":"b","lifecycle":"transient","match":{"exe":"game.exe"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::AmbiguousImageMatch));
}

// --- file-level -----------------------------------------------------------

#[test]
fn a_file_over_the_size_limit_is_refused_before_reading() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diagnostics");
    fs::create_dir_all(&dir).expect("create scratch");
    let path = dir.join("huge.json");
    let big = vec![b' '; (MAX_PROFILE_BYTES + 1) as usize];
    fs::write(&path, big).unwrap();
    match load(&path) {
        Err(LoadError::Invalid(d)) => assert!(d.has(DiagnosticCode::FileTooLarge)),
        other => panic!("expected FileTooLarge, got {other:?}"),
    }
}

// --- all errors at once ----------------------------------------------------

#[test]
fn every_problem_is_reported_in_one_pass_across_both_layers() {
    // Four independent faults, mixing structural and fragcap-specific layers:
    //   1. an unknown top-level key (structural)
    //   2. game.name wrong type (structural)
    //   3. an invalid lifecycle on the stage (structural)
    //   4. an invalid regex on the stage (fragcap)
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","bogus":true,"game":{"id":"t","name":5},"stage":[{"role":"c","lifecycle":"nope","match":{"exe":"c.exe","path_regex":"("}}]}"#,
    );
    assert!(d.has(DiagnosticCode::UnknownKey), "diagnostics: {d}");
    assert!(d.has(DiagnosticCode::WrongType), "diagnostics: {d}");
    assert!(d.has(DiagnosticCode::InvalidLifecycle), "diagnostics: {d}");
    assert!(d.has(DiagnosticCode::InvalidRegex), "diagnostics: {d}");
    assert_eq!(
        d.len(),
        4,
        "exactly four independent faults, reported in one pass, not one at a time:\n{d}"
    );
}

#[test]
fn a_semantically_broken_but_structurally_valid_profile_is_still_refused() {
    // Structural validation passes; only the semantic cycle is wrong. Proves the
    // semantic layer runs even when the structural layer is clean.
    let d = refused(
        r#"{"schema":1,"kind":"profile","fidelity":"verified","game":{"id":"t","name":"T"},"stage":[{"role":"a","lifecycle":"session","match":{"exe":"a.exe","descends_from":"a"}}]}"#,
    );
    assert!(d.has(DiagnosticCode::DescendsFromCycle));
}
