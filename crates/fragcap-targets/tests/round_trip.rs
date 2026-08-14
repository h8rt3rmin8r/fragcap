// SPDX-License-Identifier: Apache-2.0

//! The MVP: a store populated across the three tiers exports to schema-valid
//! JSON, and the record shapes honor the omission and honesty rules.

use fragcap_profile::jsonschema::validate_json;
use fragcap_targets::{export, Engine, EngineConfidence, EngineSource, Game, LaunchEntry, Store};
use serde_json::Value;

fn eso() -> Game {
    let mut g = Game::new(306130);
    g.name = Some("The Elder Scrolls Online".to_string());
    g.launcher_mediated = Some(true);
    let mut first = LaunchEntry::new("The Elder Scrolls Online.exe").unwrap();
    first.os = Some("windows".to_string());
    first.osarch = Some("64".to_string());
    let mut second = LaunchEntry::new("eso64.exe").unwrap();
    second.beta_branch = Some("pts".to_string());
    g.launch = vec![first, second];
    g
}

fn dota() -> Game {
    let mut g = Game::new(570);
    g.name = Some("Dota 2".to_string());
    g.launcher_mediated = Some(false);
    g.launch = vec![LaunchEntry::new("dota2.exe").unwrap()];
    g.engine = Some(Engine {
        name: Some("Source 2".to_string()),
        source: EngineSource::ExeHeuristic,
        confidence: EngineConfidence::Medium,
    });
    g
}

fn tf2() -> Game {
    let mut g = Game::new(440);
    g.name = Some("Team Fortress 2".to_string());
    g
}

fn populated_store() -> Store {
    let mut store = Store::open_in_memory().unwrap();
    for g in [eso(), dota(), tf2()] {
        store.upsert_game(&g).unwrap();
    }
    store
}

#[test]
fn export_of_a_populated_store_validates_against_the_schema() {
    let store = populated_store();
    let text = export(&store).unwrap();
    assert!(
        validate_json(&text).is_valid(),
        "export must validate against the master schema; got: {text}"
    );
}

#[test]
fn envelope_is_a_heuristic_unverified_export() {
    let store = populated_store();
    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    assert_eq!(doc["schema"], 1);
    assert_eq!(doc["kind"], "export");
    assert_eq!(doc["fidelity"], "heuristic-unverified");
    assert_eq!(doc["provenance"]["source"], "hint-db");
    // Top-level hint fields are forbidden on an export envelope.
    assert!(doc.get("launch").is_none());
    assert!(doc.get("engine").is_none());
    assert!(doc.get("launcher_mediated").is_none());
}

#[test]
fn every_record_is_heuristic_unverified_regardless_of_engine_confidence() {
    let store = populated_store();
    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    for record in doc["records"].as_array().unwrap() {
        assert_eq!(
            record["fidelity"], "heuristic-unverified",
            "engine confidence must never become a fidelity tier (P-9)"
        );
    }
}

fn record_for<'a>(doc: &'a Value, app_id: &str) -> &'a Value {
    doc["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["game"]["app_id"] == app_id)
        .unwrap_or_else(|| panic!("no record for app_id {app_id}"))
}

#[test]
fn unknown_engine_is_omitted_not_nulled() {
    let store = populated_store();
    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    let tf2 = record_for(&doc, "440");
    assert!(
        tf2.get("engine").is_none(),
        "unknown engine must be omitted"
    );
    assert!(tf2.get("launch").is_none(), "empty launch must be omitted");
    assert!(tf2.get("launcher_mediated").is_none());
}

#[test]
fn launch_array_is_carried_whole_and_ordered() {
    let store = populated_store();
    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    let eso = record_for(&doc, "306130");
    assert_eq!(eso["launcher_mediated"], true);
    let launch = eso["launch"].as_array().unwrap();
    assert_eq!(launch.len(), 2, "both launch entries must be kept");
    assert_eq!(launch[0]["executable"], "The Elder Scrolls Online.exe");
    assert_eq!(launch[1]["executable"], "eso64.exe");
    assert_eq!(launch[1]["beta_branch"], "pts");
}

#[test]
fn engine_maps_source_and_confidence() {
    let store = populated_store();
    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    let dota = record_for(&doc, "570");
    assert_eq!(dota["engine"]["source"], "exe_heuristic");
    assert_eq!(dota["engine"]["confidence"], "medium");
    assert_eq!(dota["engine"]["name"], "Source 2");
}

#[test]
fn empty_store_exports_a_valid_empty_envelope() {
    let store = Store::open_in_memory().unwrap();
    let text = export(&store).unwrap();
    assert!(validate_json(&text).is_valid());
    let doc: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(doc["records"].as_array().unwrap().len(), 0);
}
