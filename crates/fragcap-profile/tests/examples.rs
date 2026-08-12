// SPDX-License-Identifier: Apache-2.0

//! The two worked profiles from specification section 15.2, as JSON.
//!
//! These are the acceptance surface for section 15.1's promise that adding a game
//! means writing a file. If either is refused, the schema disagrees with the
//! architecture of record, and the architecture of record wins.

use std::time::Duration;

use fragcap_profile::{CaptureMode, Lifecycle, Profile};

/// Specification section 15.2, the single-title example, as JSON.
const ESO: &str = r#"{
  "schema": 1,
  "kind": "profile",
  "fidelity": "verified",
  "game": { "id": "eso", "name": "The Elder Scrolls Online", "platform": "steam", "app_id": "306130" },
  "capture": { "mode": "file", "duration": "30m", "roles": ["launcher", "client"], "loopback": true, "payload": true },
  "stage": [
    { "role": "launcher", "lifecycle": "transient", "match": { "exe": "*Launcher.exe", "path_contains": "Elder Scrolls Online" } },
    { "role": "client", "lifecycle": "session", "terminal": true, "match": { "exe": "eso64.exe" } }
  ]
}"#;

/// Specification section 15.2, the three-stage example. The client's recurring
/// image name is pinned by ancestry, which is what keeps this profile out of the
/// ambiguity check (see sections 5.4 and 19).
const DIV2: &str = r#"{
  "schema": 1,
  "kind": "profile",
  "fidelity": "verified",
  "game": { "id": "div2", "name": "Tom Clancy's The Division 2" },
  "stage": [
    { "role": "platform", "lifecycle": "service", "match": { "exe": "upc.exe" } },
    { "role": "client", "lifecycle": "session", "terminal": true, "match": { "exe": "TheDivision2.exe", "descends_from": "anticheat" } },
    { "role": "anticheat", "lifecycle": "transient", "match": { "exe": "EACLaunch.exe" } }
  ]
}"#;

fn eso() -> Profile {
    Profile::parse(ESO).unwrap_or_else(|d| panic!("the section 15.2 example must parse:\n{d}"))
}

fn div2() -> Profile {
    Profile::parse(DIV2).unwrap_or_else(|d| panic!("the section 15.2 example must parse:\n{d}"))
}

#[test]
fn the_single_title_example_parses_field_for_field() {
    let p = eso();

    assert_eq!(p.schema(), 1);
    assert_eq!(p.game().id().as_str(), "eso");
    assert_eq!(p.game().name(), "The Elder Scrolls Online");
    assert_eq!(p.game().platform(), Some("steam"));
    assert_eq!(
        p.game().app_id(),
        Some("306130"),
        "a platform application identifier stays a string even when it looks numeric"
    );

    let c = p.capture();
    assert_eq!(c.mode(), Some(CaptureMode::File));
    assert_eq!(c.duration(), Some(Duration::from_secs(30 * 60)));
    assert_eq!(
        c.roles(),
        Some(&["launcher".to_string(), "client".to_string()][..])
    );
    assert_eq!(c.loopback(), Some(true));
    assert_eq!(c.payload(), Some(true));

    assert_eq!(p.stages().len(), 2);

    let launcher = &p.stages()[0];
    assert_eq!(launcher.role(), "launcher");
    assert_eq!(launcher.lifecycle(), Lifecycle::Transient);
    assert!(!launcher.is_terminal());
    assert_eq!(
        launcher.predicates().exe().map(|e| e.as_str()),
        Some("*Launcher.exe")
    );
    assert_eq!(
        launcher.predicates().path_contains(),
        Some("Elder Scrolls Online")
    );
    assert_eq!(launcher.predicates().path_regex(), None);
    assert_eq!(launcher.predicates().cmdline_contains(), None);
    assert_eq!(launcher.predicates().descends_from(), None);

    let client = &p.stages()[1];
    assert_eq!(client.role(), "client");
    assert_eq!(client.lifecycle(), Lifecycle::Session);
    assert!(client.is_terminal());
    assert_eq!(
        client.predicates().exe().map(|e| e.as_str()),
        Some("eso64.exe")
    );

    assert_eq!(p.terminal_stage().map(|s| s.role()), Some("client"));
    assert_eq!(p.stage("launcher").map(|s| s.role()), Some("launcher"));
    assert_eq!(p.stage("absent"), None);
}

#[test]
fn the_three_stage_example_parses_and_is_accepted() {
    let p = div2();

    assert_eq!(p.stages().len(), 3);
    assert_eq!(
        p.stages().iter().map(|s| s.role()).collect::<Vec<_>>(),
        vec!["platform", "client", "anticheat"],
        "stage order is declaration order, not alphabetical or dependency order"
    );

    let client = p.stage("client").expect("client stage");
    assert_eq!(
        client.predicates().exe().map(|e| e.as_str()),
        Some("TheDivision2.exe")
    );
    assert_eq!(
        client.predicates().descends_from(),
        Some("anticheat"),
        "the recurring image name is pinned by ancestry, which is what keeps this \
         profile out of the ambiguity check"
    );
    assert!(client.predicates().is_pinned());

    assert_eq!(p.stages()[0].lifecycle(), Lifecycle::Service);
    assert_eq!(p.stages()[2].lifecycle(), Lifecycle::Transient);
}

#[test]
fn absent_optional_fields_report_absence_rather_than_a_default() {
    let p = div2();

    assert_eq!(p.game().platform(), None);
    assert_eq!(p.game().app_id(), None);

    let c = p.capture();
    assert_eq!(c.mode(), None);
    assert_eq!(c.duration(), None);
    assert_eq!(c.roles(), None);
    assert_eq!(
        c.payload(),
        None,
        "a profile that said nothing about payload must be distinguishable from one \
         that chose a value, because the command line overrides one and not the other"
    );
    assert_eq!(c.loopback(), None);
}

#[test]
fn a_minimal_profile_is_accepted() {
    let text = r#"{
  "schema": 1, "kind": "profile", "fidelity": "verified",
  "game": { "id": "min", "name": "Minimal" },
  "stage": [ { "role": "client", "lifecycle": "session", "match": { "exe": "min.exe" } } ]
}"#;
    let p = Profile::parse(text).expect("a profile with only required fields is valid");
    assert_eq!(p.stages().len(), 1);
    assert!(!p.stages()[0].is_terminal(), "terminal defaults to false");
    assert_eq!(p.terminal_stage(), None);
}

#[test]
fn no_declared_value_is_normalized() {
    // Constitution P-9. Case folding an `exe` pattern or trimming a path is the
    // natural convenience, and it would mean the profile fragcap acts on is not
    // the profile the author wrote.
    let text = r#"{
  "schema": 1, "kind": "profile", "fidelity": "verified",
  "game": { "id": "case", "name": "  Spaced   Name  " },
  "stage": [ { "role": "Mixed_Case-Role", "lifecycle": "session", "match": { "exe": "*Launcher.EXE", "path_contains": "C:\\Program Files\\Zenimax  Online" } } ]
}"#;
    let p = Profile::parse(text).expect("valid");
    assert_eq!(p.game().name(), "  Spaced   Name  ");
    assert_eq!(p.stages()[0].role(), "Mixed_Case-Role");
    assert_eq!(
        p.stages()[0].predicates().exe().map(|e| e.as_str()),
        Some("*Launcher.EXE")
    );
    assert_eq!(
        p.stages()[0].predicates().path_contains(),
        Some(r"C:\Program Files\Zenimax  Online"),
        "a string keeps its backslashes and its doubled space"
    );
}

#[test]
fn every_value_form_the_schema_can_contain_is_accepted() {
    let text = r#"{
  "schema": 1, "kind": "profile", "fidelity": "verified",
  "game": { "id": "forms", "name": "The\nMulti Line", "app_id": "literal-string" },
  "capture": { "mode": "file", "duration": "500ms", "roles": ["client"], "loopback": false, "payload": true },
  "stage": [ { "role": "client", "lifecycle": "session", "terminal": true, "match": { "exe": "a?c*.exe", "path_contains": "C:\\Games\\x", "path_regex": "(?i)games", "cmdline_contains": "-launch" } } ]
}"#;
    let p = Profile::parse(text).unwrap_or_else(|d| panic!("must parse:\n{d}"));
    assert_eq!(p.game().name(), "The\nMulti Line");
    assert_eq!(p.game().app_id(), Some("literal-string"));
    assert_eq!(p.capture().duration(), Some(Duration::from_millis(500)));
    assert_eq!(p.capture().loopback(), Some(false));
    assert_eq!(p.capture().roles(), Some(&["client".to_string()][..]));
    let m = p.stages()[0].predicates();
    assert_eq!(m.path_contains(), Some(r"C:\Games\x"));
    assert_eq!(m.path_regex().map(|r| r.as_str()), Some("(?i)games"));
    assert_eq!(m.cmdline_contains(), Some("-launch"));
}

#[test]
fn a_windows_path_keeps_its_backslashes() {
    // The form a profile author will actually use. A regression here would be
    // silently wrong rather than a refusal.
    let text = r#"{
  "schema": 1, "kind": "profile", "fidelity": "verified",
  "game": { "id": "paths", "name": "Paths" },
  "stage": [ { "role": "client", "lifecycle": "session", "match": { "path_contains": "C:\\Program Files (x86)\\Steam\\steamapps\\common" } } ]
}"#;
    let p = Profile::parse(text).unwrap_or_else(|d| panic!("must parse:\n{d}"));
    assert_eq!(
        p.stages()[0].predicates().path_contains(),
        Some(r"C:\Program Files (x86)\Steam\steamapps\common")
    );
}

#[test]
fn parsing_is_deterministic() {
    let a = Profile::parse(ESO).expect("valid");
    let b = Profile::parse(ESO).expect("valid");
    assert_eq!(a, b);
}
