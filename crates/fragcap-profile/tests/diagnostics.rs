// SPDX-License-Identifier: Apache-2.0

//! Being wrong well.
//!
//! Specification section 15.4 requires that validation report every problem found
//! rather than stopping at the first, so the tests here are as much about how many
//! diagnostics come back as about which. Every [`DiagnosticCode`] is produced by at
//! least one case in this file; a code that cannot be produced is indistinguishable
//! from one that is wired up wrong.

use std::fs;
use std::path::PathBuf;

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

/// A valid profile with one stage, as a base for one-fault cases.
fn with(body: &str) -> String {
    format!("schema = 1\n[game]\nid   = \"t\"\nname = \"T\"\n{body}")
}

fn one_stage() -> &'static str {
    "[[stage]]\nrole      = \"client\"\nlifecycle = \"session\"\nmatch     = { exe = \"c.exe\" }\n"
}

#[test]
fn a_syntax_fault_is_one_diagnostic_and_suppresses_the_rest() {
    // The document also has no `[game]` table and no stage, so a parser that
    // recovered would report faults against a file the author did not write.
    let d = refused("schema = = 1\n");
    assert_eq!(d.len(), 1);
    assert!(d.has(DiagnosticCode::Syntax));
    let first = d.iter().next().expect("one diagnostic");
    assert!(
        first.position.is_some(),
        "a syntax fault must carry a position; it is the only thing the author can act on"
    );
}

#[test]
fn a_duplicate_key_is_refused_by_the_parser() {
    let d = refused(&with(&format!("{}\n[game]\nid = \"u\"\n", one_stage())));
    assert!(d.has(DiagnosticCode::Syntax));
}

#[test]
fn datetime_is_a_syntax_fault_not_a_type_fault() {
    // Pins the one known divergence from TOML 1.0 in the parser this crate uses,
    // which its own documentation states. No key in schema version 1 has a
    // datetime type, so a datetime can appear only in a profile that is invalid
    // anyway, and the effect is on the message rather than the verdict. See slice
    // S05 research R-1.
    let d = refused(&with(&format!(
        "[capture]\nduration = 1979-05-27T07:32:00Z\n{}",
        one_stage()
    )));
    assert!(
        d.has(DiagnosticCode::Syntax),
        "if this becomes a WrongType diagnostic, the parser gained datetime support \
         and research R-1 should be revisited"
    );
}

#[test]
fn an_unsupported_schema_version_suppresses_the_semantic_set() {
    // The profile below also has an unknown key inside [game] and no stage.
    // Reporting those alongside a version fault would bury the one thing the
    // author needs to know.
    let d = refused("schema = 2\n[game]\nid = \"t\"\nname = \"T\"\nbogus = 1\n");
    assert_eq!(d.len(), 1);
    assert!(d.has(DiagnosticCode::UnsupportedSchema));
    assert!(
        d.iter().next().expect("one").message.contains('1'),
        "the diagnostic must name the version this build supports"
    );
}

#[test]
fn an_unsupported_schema_version_suppresses_a_top_level_unknown_key() {
    // Regression, PR 11 review. The first version of this slice ran the top level
    // key check before the schema gate, so a later-schema profile came back with
    // both an UnsupportedSchema and an UnknownKey diagnostic. A new top level key
    // is the most likely thing a later schema adds, so reporting it is reporting a
    // consequence of the version fault as though it were a second problem.
    //
    // The test above did not catch this because `bogus = 1` after `[game]` is
    // inside that table rather than at the top level, so it passed for the wrong
    // reason. This case places the key where the check actually runs.
    let d = refused("schema = 2\nfuture_key = true\n");
    assert_eq!(
        d.len(),
        1,
        "an unsupported version must be the only diagnostic, got:\n{d}"
    );
    assert!(d.has(DiagnosticCode::UnsupportedSchema));
    assert!(!d.has(DiagnosticCode::UnknownKey));
}

#[test]
fn a_missing_schema_is_reported_and_no_version_is_assumed() {
    let d = refused("[game]\nid = \"t\"\nname = \"T\"\n");
    assert!(d.has(DiagnosticCode::MissingField));
    assert!(
        !d.has(DiagnosticCode::UnsupportedSchema),
        "absent is not the same as unsupported"
    );
}

#[test]
fn a_non_integer_schema_is_a_type_fault() {
    let d = refused("schema = \"1\"\n[game]\nid = \"t\"\nname = \"T\"\n");
    assert!(d.has(DiagnosticCode::WrongType));
}

#[test]
fn every_missing_required_field_is_reported() {
    // Four required fields absent: game.id, game.name, stage role, stage
    // lifecycle. A validator that stopped at the first would report one.
    let d = refused("schema = 1\n[game]\n[[stage]]\nmatch = { exe = \"c.exe\" }\n");
    let missing = d
        .iter()
        .filter(|x| x.code == DiagnosticCode::MissingField)
        .count();
    assert_eq!(
        missing, 4,
        "expected game.id, game.name, stage[0].role, stage[0].lifecycle; got:\n{d}"
    );
    let locations: Vec<&str> = d.iter().map(|x| x.location.as_str()).collect();
    assert!(locations.contains(&"game.id"));
    assert!(locations.contains(&"game.name"));
    assert!(locations.contains(&"stage[0].role"));
    assert!(locations.contains(&"stage[0].lifecycle"));
}

#[test]
fn every_type_fault_is_reported_with_expected_and_found() {
    let d = refused(
        "schema = 1\n[game]\nid = 1\nname = true\n[capture]\nloopback = \"yes\"\n\
         [[stage]]\nrole = 1\nlifecycle = \"session\"\nmatch = { exe = \"c.exe\" }\n",
    );
    let types: Vec<&str> = d
        .iter()
        .filter(|x| x.code == DiagnosticCode::WrongType)
        .map(|x| x.message.as_str())
        .collect();
    assert_eq!(types.len(), 4, "got:\n{d}");
    assert!(types.iter().any(|m| m.contains("expected string")));
    assert!(types.iter().any(|m| m.contains("expected boolean")));
    assert!(types.iter().any(|m| m.contains("found integer")));
    assert!(types.iter().any(|m| m.contains("found boolean")));
}

#[test]
fn an_unknown_key_is_reported_per_table() {
    let cases: &[(&str, &str)] = &[
        (
            "top level",
            "schema = 1\nbogus = 1\n[game]\nid=\"t\"\nname=\"T\"\n",
        ),
        (
            "game",
            "schema = 1\n[game]\nid=\"t\"\nname=\"T\"\nbogus = 1\n",
        ),
        (
            "capture",
            "schema = 1\n[game]\nid=\"t\"\nname=\"T\"\n[capture]\npayloads = false\n",
        ),
        (
            "stage",
            "schema = 1\n[game]\nid=\"t\"\nname=\"T\"\n[[stage]]\nrole=\"c\"\n\
             lifecycle=\"session\"\nmatch={ exe=\"c.exe\" }\nbogus = 1\n",
        ),
        (
            "match",
            "schema = 1\n[game]\nid=\"t\"\nname=\"T\"\n[[stage]]\nrole=\"c\"\n\
             lifecycle=\"session\"\nmatch={ exe=\"c.exe\", bogus = 1 }\n",
        ),
    ];
    for (table, text) in cases {
        let d = refused(text);
        assert!(
            d.has(DiagnosticCode::UnknownKey),
            "unknown key in the {table} table must be reported, not ignored:\n{d}"
        );
        assert!(
            d.iter()
                .any(|x| x.code == DiagnosticCode::UnknownKey && x.message.contains("accepted")),
            "the diagnostic must list the accepted set for that table"
        );
    }
}

#[test]
fn the_payloads_typo_is_refused_rather_than_ignored() {
    // The case the strict-key decision exists for. An author intending
    // `payload = false` who writes `payloads = false` and is not told gets a
    // capture containing contents they meant to exclude.
    let d = refused(&with(&format!(
        "[capture]\npayloads = false\n{}",
        one_stage()
    )));
    assert!(d.has(DiagnosticCode::UnknownKey));
}

#[test]
fn an_invalid_slug_is_refused() {
    for id in ["\"ESO\"", "\"../etc\"", "\"a/b\"", "\"\"", "\"eso.toml\""] {
        let text = format!(
            "schema = 1\n[game]\nid = {id}\nname = \"T\"\n{}",
            one_stage()
        );
        let d = refused(&text);
        assert!(
            d.has(DiagnosticCode::InvalidSlug),
            "id {id} must be refused:\n{d}"
        );
    }
}

#[test]
fn an_invalid_lifecycle_or_mode_names_the_accepted_set() {
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"persistent\"\nmatch = { exe = \"c.exe\" }\n",
    ));
    assert!(d.has(DiagnosticCode::InvalidLifecycle));
    assert!(d
        .iter()
        .any(|x| x.code == DiagnosticCode::InvalidLifecycle && x.message.contains("transient")));

    let d = refused(&with(&format!(
        "[capture]\nmode = \"tape\"\n{}",
        one_stage()
    )));
    assert!(d.has(DiagnosticCode::InvalidMode));
    assert!(d
        .iter()
        .any(|x| x.code == DiagnosticCode::InvalidMode && x.message.contains("stream")));
}

#[test]
fn an_invalid_duration_is_refused() {
    for literal in ["\"30\"", "\"0s\"", "\"1h30m\"", "\"30d\"", "\"1.5h\""] {
        let text = with(&format!("[capture]\nduration = {literal}\n{}", one_stage()));
        let d = refused(&text);
        assert!(
            d.has(DiagnosticCode::InvalidDuration),
            "duration {literal} must be refused:\n{d}"
        );
    }
}

#[test]
fn an_empty_exe_pattern_is_refused() {
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\nmatch = { exe = \"\" }\n",
    ));
    assert!(d.has(DiagnosticCode::InvalidGlob));
}

#[test]
fn a_regex_that_does_not_compile_is_refused_with_the_engines_message() {
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\n\
         match = { path_regex = \"(unclosed\" }\n",
    ));
    assert!(d.has(DiagnosticCode::InvalidRegex));
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::InvalidRegex)
        .expect("regex diagnostic")
        .message;
    assert!(
        msg.contains("regex parse error"),
        "the engine's own message must be carried through, got: {msg}"
    );
}

#[test]
fn a_pathological_regex_is_refused_through_the_engines_own_limit() {
    // fragcap forms no second opinion about which patterns are too large. The
    // engine already refuses this one, and the diagnostic is an ordinary
    // compilation failure rather than a special case.
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\n\
         match = { path_regex = \"(a{100}){100}{100}\" }\n",
    ));
    assert!(d.has(DiagnosticCode::InvalidRegex));
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::InvalidRegex)
        .expect("regex diagnostic")
        .message;
    assert!(
        msg.contains("size limit"),
        "expected the engine's compiled size limit message, got: {msg}"
    );
}

#[test]
fn an_empty_match_table_is_refused() {
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\nmatch = {}\n",
    ));
    assert!(
        d.has(DiagnosticCode::EmptyMatch),
        "an empty predicate set matches every process on the system:\n{d}"
    );
}

#[test]
fn an_empty_roles_list_is_refused() {
    let d = refused(&with(&format!("[capture]\nroles = []\n{}", one_stage())));
    assert!(d.has(DiagnosticCode::EmptyRoles));
}

#[test]
fn a_profile_with_no_stage_is_refused() {
    let d = refused("schema = 1\n[game]\nid = \"t\"\nname = \"T\"\n");
    assert!(d.has(DiagnosticCode::NoStages));

    // Declared but empty is the same refusal.
    let d = refused("schema = 1\n[game]\nid = \"t\"\nname = \"T\"\nstage = []\n");
    assert!(d.has(DiagnosticCode::NoStages));
}

#[test]
fn a_duplicate_role_names_both_stages() {
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\nmatch = { exe = \"a.exe\" }\n\
         [[stage]]\nrole = \"c\"\nlifecycle = \"transient\"\nmatch = { exe = \"b.exe\" }\n",
    ));
    assert!(d.has(DiagnosticCode::DuplicateRole));
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::DuplicateRole)
        .expect("duplicate role diagnostic")
        .message;
    assert!(
        msg.contains("stage[0]"),
        "must name the earlier stage: {msg}"
    );
}

#[test]
fn two_terminal_stages_are_refused() {
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"session\"\nterminal = true\n\
         match = { exe = \"a.exe\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"session\"\nterminal = true\n\
         match = { exe = \"b.exe\" }\n",
    ));
    assert!(d.has(DiagnosticCode::MultipleTerminal));
}

#[test]
fn a_terminal_stage_must_be_a_session_stage() {
    // Section 10.4 defines a transient exit as normal and expected, so a terminal
    // transient ends the capture the moment the launcher hands off, which is the
    // point the whole launcher chain exists to survive.
    for lifecycle in ["transient", "service"] {
        let d = refused(&with(&format!(
            "[[stage]]\nrole = \"a\"\nlifecycle = \"{lifecycle}\"\nterminal = true\n\
             match = {{ exe = \"a.exe\" }}\n\
             [[stage]]\nrole = \"b\"\nlifecycle = \"session\"\nmatch = {{ exe = \"b.exe\" }}\n"
        )));
        assert!(
            d.has(DiagnosticCode::TerminalLifecycle),
            "terminal on a {lifecycle} stage must be refused:\n{d}"
        );
    }
}

#[test]
fn descends_from_must_name_a_declared_role() {
    let d = refused(&with(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\n\
         match = { exe = \"c.exe\", descends_from = \"ghost\" }\n",
    ));
    assert!(d.has(DiagnosticCode::UnknownDescendsFrom));
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::UnknownDescendsFrom)
        .expect("diagnostic")
        .message;
    assert!(msg.contains("ghost"));
    assert!(
        msg.contains('c'),
        "the declared roles must be listed: {msg}"
    );
}

#[test]
fn a_descends_from_cycle_is_refused_and_names_the_cycle() {
    // Two stages pointing at each other. No process assignment satisfies it.
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"session\"\n\
         match = { exe = \"a.exe\", descends_from = \"b\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"transient\"\n\
         match = { exe = \"b.exe\", descends_from = \"a\" }\n",
    ));
    assert!(d.has(DiagnosticCode::DescendsFromCycle));
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::DescendsFromCycle)
        .expect("diagnostic")
        .message;
    assert!(msg.contains('a') && msg.contains('b'), "names both: {msg}");
}

#[test]
fn a_self_referencing_descends_from_is_a_cycle() {
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"session\"\n\
         match = { exe = \"a.exe\", descends_from = \"a\" }\n",
    ));
    assert!(
        d.has(DiagnosticCode::DescendsFromCycle),
        "a stage cannot be its own ancestor:\n{d}"
    );
}

#[test]
fn a_longer_cycle_is_found_too() {
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"session\"\n\
         match = { exe = \"a.exe\", descends_from = \"b\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"transient\"\n\
         match = { exe = \"b.exe\", descends_from = \"c\" }\n\
         [[stage]]\nrole = \"c\"\nlifecycle = \"transient\"\n\
         match = { exe = \"c.exe\", descends_from = \"a\" }\n",
    ));
    assert!(d.has(DiagnosticCode::DescendsFromCycle));
}

#[test]
fn an_acyclic_chain_is_accepted() {
    // The shape a real launcher chain has. Guards against a cycle check that
    // fires on any path of length two.
    let text = with(
        "[[stage]]\nrole = \"platform\"\nlifecycle = \"service\"\nmatch = { exe = \"p.exe\" }\n\
         [[stage]]\nrole = \"launcher\"\nlifecycle = \"transient\"\n\
         match = { exe = \"l.exe\", descends_from = \"platform\" }\n\
         [[stage]]\nrole = \"client\"\nlifecycle = \"session\"\nterminal = true\n\
         match = { exe = \"c.exe\", descends_from = \"launcher\" }\n",
    );
    Profile::parse(&text).unwrap_or_else(|d| panic!("an acyclic chain is valid:\n{d}"));
}

#[test]
fn capture_roles_must_name_declared_roles() {
    let d = refused(&with(&format!(
        "[capture]\nroles = [\"client\", \"ghost\"]\n{}",
        one_stage()
    )));
    assert!(
        d.has(DiagnosticCode::UndeclaredCaptureRole),
        "a role nothing declares captures nothing under it:\n{d}"
    );
}

#[test]
fn a_profile_of_only_services_is_refused() {
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"service\"\nmatch = { exe = \"a.exe\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"service\"\nmatch = { exe = \"b.exe\" }\n",
    ));
    assert!(
        d.has(DiagnosticCode::AllServices),
        "section 10.4: a service is never awaited, so nothing here can trigger \
         acquisition:\n{d}"
    );
}

#[test]
fn the_section_5_4_ambiguity_is_refused() {
    // Two stages matching on `exe` alone that can bind the same process. This is
    // the check that prevents a complete, well formed, empty capture.
    let d = refused(&with(
        "[[stage]]\nrole = \"first\"\nlifecycle = \"transient\"\n\
         match = { exe = \"TheDivision2.exe\" }\n\
         [[stage]]\nrole = \"client\"\nlifecycle = \"session\"\n\
         match = { exe = \"TheDivision2.exe\" }\n",
    ));
    assert!(d.has(DiagnosticCode::AmbiguousImageMatch));
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::AmbiguousImageMatch)
        .expect("diagnostic")
        .message;
    assert!(
        msg.contains("stage[0]") && msg.contains("stage[1]"),
        "{msg}"
    );
    assert!(
        msg.contains("descends_from"),
        "the diagnostic must state the remedy: {msg}"
    );
}

#[test]
fn a_glob_that_can_match_a_literal_is_ambiguous_with_it() {
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"transient\"\nmatch = { exe = \"*Launcher.exe\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"session\"\nmatch = { exe = \"ESOLauncher.exe\" }\n",
    ));
    assert!(d.has(DiagnosticCode::AmbiguousImageMatch));
}

#[test]
fn one_pinned_stage_is_not_enough() {
    // The rule is that a pair is refused unless BOTH stages are pinned. One
    // pinned stage still leaves the other able to bind the wrong process.
    let d = refused(&with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"transient\"\n\
         match = { exe = \"x.exe\", descends_from = \"b\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"session\"\nmatch = { exe = \"x.exe\" }\n",
    ));
    assert!(d.has(DiagnosticCode::AmbiguousImageMatch));
}

#[test]
fn two_pinned_stages_sharing_an_image_name_are_accepted() {
    let text = with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"transient\"\n\
         match = { exe = \"x.exe\", path_contains = \"first\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"session\"\n\
         match = { exe = \"x.exe\", path_contains = \"second\" }\n",
    );
    Profile::parse(&text)
        .unwrap_or_else(|d| panic!("both stages are pinned, so this is legal:\n{d}"));
}

#[test]
fn disjoint_patterns_are_not_reported_as_ambiguous() {
    let text = with(
        "[[stage]]\nrole = \"a\"\nlifecycle = \"transient\"\nmatch = { exe = \"a*.exe\" }\n\
         [[stage]]\nrole = \"b\"\nlifecycle = \"session\"\nmatch = { exe = \"b*.exe\" }\n",
    );
    Profile::parse(&text).unwrap_or_else(|d| panic!("these cannot match one name:\n{d}"));
}

#[test]
fn several_faults_are_reported_in_one_call() {
    // The slice's stated purpose. Six distinct faults, one call.
    let text = "schema = 1\n\
                [game]\n\
                id = \"BAD-ID\"\n\
                name = \"T\"\n\
                [capture]\n\
                mode = \"tape\"\n\
                duration = \"30\"\n\
                payloads = false\n\
                [[stage]]\n\
                role = \"c\"\n\
                lifecycle = \"persistent\"\n\
                match = { exe = \"c.exe\", descends_from = \"ghost\" }\n";
    let d = refused(text);
    for code in [
        DiagnosticCode::InvalidSlug,
        DiagnosticCode::InvalidMode,
        DiagnosticCode::InvalidDuration,
        DiagnosticCode::UnknownKey,
        DiagnosticCode::InvalidLifecycle,
        DiagnosticCode::UnknownDescendsFrom,
    ] {
        assert!(d.has(code), "expected {code} in one report, got:\n{d}");
    }
    assert!(
        d.len() >= 6,
        "expected at least six diagnostics, got {}:\n{d}",
        d.len()
    );
}

#[test]
fn the_report_is_ordered_by_position_in_the_file() {
    let text = "schema = 1\n\
                [game]\n\
                id = \"BAD-ID\"\n\
                name = 1\n\
                [[stage]]\n\
                role = \"c\"\n\
                lifecycle = \"persistent\"\n\
                match = { exe = \"c.exe\" }\n";
    let d = refused(text);
    let lines: Vec<usize> = d
        .iter()
        .filter_map(|x| x.position.map(|p| p.line))
        .collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(
        lines, sorted,
        "diagnostics must come back in the order the author reads their file"
    );
}

#[test]
fn the_report_is_byte_identical_across_runs() {
    let text = "schema = 1\n[game]\nid = \"BAD\"\nname = 1\nbogus = 2\n";
    let a = refused(text).to_string();
    let b = refused(text).to_string();
    assert_eq!(a, b);
}

#[test]
fn a_file_over_the_size_limit_is_refused_before_its_contents_are_read() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("oversized");
    fs::create_dir_all(&dir).expect("scratch directory");
    let path = dir.join("huge.toml");

    // Deliberately not valid TOML. If the contents were read and parsed before
    // the size was checked, this would come back as a Syntax diagnostic, so the
    // assertion below distinguishes a check from an accident.
    let filler = "= = = not toml = = =\n".repeat(60_000);
    assert!(
        filler.len() as u64 > MAX_PROFILE_BYTES,
        "filler must exceed the limit"
    );
    fs::write(&path, &filler).expect("write");

    match load(&path) {
        Err(LoadError::Invalid(d)) => {
            assert!(d.has(DiagnosticCode::FileTooLarge), "got:\n{d}");
            assert!(
                !d.has(DiagnosticCode::Syntax),
                "a syntax diagnostic here means the contents were read first:\n{d}"
            );
            assert!(
                d.iter()
                    .any(|x| x.message.contains(&MAX_PROFILE_BYTES.to_string())),
                "the diagnostic must name the limit"
            );
        }
        other => panic!("expected an oversized refusal, got {other:?}"),
    }
}

#[test]
fn a_roles_entry_with_the_wrong_type_does_not_hide_the_other_entries() {
    // Regression, PR 11 review. The first version discarded the whole list when
    // any element failed to parse, so `["ghost", 1]` reported the type fault and
    // silently dropped the fact that `ghost` is a role no stage declares. Two
    // independent faults, one reported, which is what FR-013 forbids.
    let d = refused(&with(&format!(
        "[capture]\nroles = [\"ghost\", 1]\n{}",
        one_stage()
    )));
    assert!(
        d.has(DiagnosticCode::WrongType),
        "the second element's type is still a fault:\n{d}"
    );
    assert!(
        d.has(DiagnosticCode::UndeclaredCaptureRole),
        "and the first element still names a role nothing declares:\n{d}"
    );
    assert!(
        !d.has(DiagnosticCode::EmptyRoles),
        "a two element list is not empty, whatever survived parsing:\n{d}"
    );
}

#[test]
fn a_roles_list_of_only_bad_entries_is_not_reported_as_empty() {
    // The other half of the same rule. Emptiness is judged on what the author
    // declared, so a list with one bad element is a type fault and not an empty
    // list.
    let d = refused(&with(&format!("[capture]\nroles = [1]\n{}", one_stage())));
    assert!(d.has(DiagnosticCode::WrongType));
    assert!(
        !d.has(DiagnosticCode::EmptyRoles),
        "the author wrote one entry, so the list is not empty:\n{d}"
    );
}

#[test]
fn an_over_long_exe_pattern_is_refused() {
    // The bound exists because the ambiguity check allocates a table proportional
    // to the product of two pattern lengths. Without it, two patterns inside the
    // one mebibyte file limit ask for about 10^12 cells and abort the process
    // instead of returning a diagnostic. Found in PR 11 review.
    let pattern = "a".repeat(MAX_PATTERN_CHARS + 1);
    let d = refused(&with(&format!(
        "[[stage]]\nrole = \"c\"\nlifecycle = \"session\"\nmatch = {{ exe = \"{pattern}\" }}\n"
    )));
    assert!(d.has(DiagnosticCode::InvalidGlob), "got:\n{d}");
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::InvalidGlob)
        .expect("glob diagnostic")
        .message;
    assert!(
        msg.contains(&MAX_PATTERN_CHARS.to_string()),
        "the diagnostic must name the limit: {msg}"
    );
}

#[test]
fn a_profile_at_the_stage_limit_is_accepted_and_one_above_it_is_not() {
    // The pairwise ambiguity pass is quadratic in stage count, and the file size
    // limit does not bound it: a one mebibyte profile can declare thousands of
    // stages. Found in PR 11 review, which also falsified this slice's original
    // claim that no stage limit was needed.
    let stage = |i: usize| {
        format!(
            "[[stage]]\nrole = \"r{i}\"\nlifecycle = \"transient\"\n\
             match = {{ exe = \"e{i}.exe\" }}\n"
        )
    };

    let at_limit: String = (0..MAX_STAGES).map(stage).collect();
    let p = Profile::parse(&with(&at_limit))
        .unwrap_or_else(|d| panic!("the limit itself must be accepted:\n{d}"));
    assert_eq!(p.stages().len(), MAX_STAGES);

    let over: String = (0..MAX_STAGES + 1).map(stage).collect();
    let d = refused(&with(&over));
    assert!(d.has(DiagnosticCode::TooManyStages), "got:\n{d}");
    let msg = &d
        .iter()
        .find(|x| x.code == DiagnosticCode::TooManyStages)
        .expect("stage limit diagnostic")
        .message;
    assert!(
        msg.contains(&MAX_STAGES.to_string()),
        "the diagnostic must name the limit: {msg}"
    );
}

#[test]
fn every_diagnostic_code_is_produced_by_a_test_in_this_file() {
    // SC-003. Listed explicitly rather than derived, because the enumeration is
    // non-exhaustive to callers and a new variant should force a decision about
    // which case produces it rather than silently pass.
    //
    // Each entry names the test above that produces it.
    let expected: &[(DiagnosticCode, &str)] = &[
        (
            DiagnosticCode::Syntax,
            "a_syntax_fault_is_one_diagnostic...",
        ),
        (
            DiagnosticCode::UnsupportedSchema,
            "an_unsupported_schema_version...",
        ),
        (
            DiagnosticCode::MissingField,
            "every_missing_required_field...",
        ),
        (DiagnosticCode::WrongType, "every_type_fault..."),
        (DiagnosticCode::UnknownKey, "an_unknown_key_is_reported..."),
        (
            DiagnosticCode::FileTooLarge,
            "a_file_over_the_size_limit...",
        ),
        (DiagnosticCode::InvalidSlug, "an_invalid_slug_is_refused"),
        (
            DiagnosticCode::InvalidLifecycle,
            "an_invalid_lifecycle_or_mode...",
        ),
        (
            DiagnosticCode::InvalidMode,
            "an_invalid_lifecycle_or_mode...",
        ),
        (DiagnosticCode::InvalidDuration, "an_invalid_duration..."),
        (DiagnosticCode::InvalidGlob, "an_empty_exe_pattern..."),
        (
            DiagnosticCode::InvalidRegex,
            "a_regex_that_does_not_compile...",
        ),
        (DiagnosticCode::EmptyMatch, "an_empty_match_table..."),
        (DiagnosticCode::EmptyRoles, "an_empty_roles_list..."),
        (DiagnosticCode::NoStages, "a_profile_with_no_stage..."),
        (
            DiagnosticCode::TooManyStages,
            "a_profile_at_the_stage_limit...",
        ),
        (DiagnosticCode::DuplicateRole, "a_duplicate_role..."),
        (DiagnosticCode::MultipleTerminal, "two_terminal_stages..."),
        (
            DiagnosticCode::TerminalLifecycle,
            "a_terminal_stage_must_be_a_session_stage",
        ),
        (
            DiagnosticCode::UnknownDescendsFrom,
            "descends_from_must_name_a_declared_role",
        ),
        (
            DiagnosticCode::DescendsFromCycle,
            "a_descends_from_cycle_is_refused...",
        ),
        (
            DiagnosticCode::UndeclaredCaptureRole,
            "capture_roles_must_name_declared_roles",
        ),
        (DiagnosticCode::AllServices, "a_profile_of_only_services..."),
        (
            DiagnosticCode::AmbiguousImageMatch,
            "the_section_5_4_ambiguity_is_refused",
        ),
    ];
    assert_eq!(
        expected.len(),
        24,
        "if a code was added to the enumeration, add the case that produces it"
    );
    // The pairing is documentation; this asserts the codes are distinct so a
    // copy-paste cannot make the count look right while covering less.
    let mut codes: Vec<DiagnosticCode> = expected.iter().map(|(c, _)| *c).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), 24);
}
