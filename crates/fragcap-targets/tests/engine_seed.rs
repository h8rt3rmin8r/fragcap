// SPDX-License-Identifier: Apache-2.0

//! The MVP: an offline engine seed fills the Tier 3 columns for exactly the
//! resolved titles, accounts for every title truthfully, and exports schema-valid,
//! with every record still heuristic-unverified (P-9).

use fragcap_profile::jsonschema::validate_json;
use fragcap_targets::{
    export, seed_engine, EngineConfidence, EngineSource, FixtureEngineFeed, Store,
};

fn feed() -> FixtureEngineFeed {
    let text = include_str!("fixtures/engine.json");
    FixtureEngineFeed::from_json(text).unwrap()
}

#[test]
fn a_seed_writes_engine_for_exactly_the_resolved_titles() {
    let mut store = Store::open_in_memory().unwrap();
    let summary = seed_engine(&mut store, &feed(), Some("2026-08-13".into())).unwrap();

    // 7 resolve a single engine (single string or one-element array):
    // 570, 620, 730, 400, 220, 1091500, 292030.
    assert_eq!(summary.written, 7, "{summary:?}");
    // 3 have no or an ambiguous engine: 8930 (""), 4000 (absent), 105600 (two engines).
    assert_eq!(summary.excluded, 3, "{summary:?}");
    // 2 are malformed: 12345 (engine is a number), 67890 (confidence out of set).
    assert_eq!(summary.failed, 2, "{summary:?}");
    assert_eq!(summary.duplicates, 0, "{summary:?}");
    assert_eq!(summary.fetched, 12, "{summary:?}");

    // Only the resolved titles carry an engine, and it is stamped pcgamingwiki.
    let games = store.games().unwrap();
    let with_engine: Vec<u32> = games
        .iter()
        .filter(|g| g.engine.is_some())
        .map(|g| g.appid)
        .collect();
    assert_eq!(with_engine, vec![220, 400, 570, 620, 730, 292030, 1091500]);
    for game in &games {
        if let Some(engine) = &game.engine {
            assert_eq!(engine.source, EngineSource::Pcgamingwiki);
        }
    }
}

#[test]
fn confidence_tokens_round_trip_and_the_default_applies() {
    let mut store = Store::open_in_memory().unwrap();
    seed_engine(&mut store, &feed(), None).unwrap();
    let games = store.games().unwrap();
    let conf = |appid: u32| {
        games
            .iter()
            .find(|g| g.appid == appid)
            .and_then(|g| g.engine.as_ref())
            .map(|e| e.confidence)
    };
    assert_eq!(conf(570), Some(EngineConfidence::Confirmed));
    assert_eq!(conf(620), Some(EngineConfidence::High));
    assert_eq!(conf(730), Some(EngineConfidence::Medium));
    assert_eq!(conf(400), Some(EngineConfidence::Low));
    assert_eq!(conf(1091500), Some(EngineConfidence::Unknown));
    // 220 omits confidence: the documented default (high) applies.
    assert_eq!(conf(220), Some(EngineConfidence::High));
}

#[test]
fn every_title_is_accounted_for_no_silent_loss() {
    let mut store = Store::open_in_memory().unwrap();
    let summary = seed_engine(&mut store, &feed(), None).unwrap();
    assert!(
        summary.is_conserved(),
        "fetched must equal written + excluded + duplicates + failed: {summary:?}"
    );
}

#[test]
fn a_malformed_entry_is_counted_not_fatal() {
    // The fixture carries two malformed entries (a numeric engine and an out-of-set
    // confidence); the run completes and writes the good titles regardless.
    let mut store = Store::open_in_memory().unwrap();
    let summary = seed_engine(&mut store, &feed(), None).unwrap();
    assert_eq!(summary.failed, 2);
    assert!(summary.written > 0, "good titles were still written");
}

#[test]
fn an_engine_only_row_exports_valid() {
    // Every appid in the fixture is new to the store, so each written row is an
    // engine-only row (appid + engine, no name). The export must still validate.
    let mut store = Store::open_in_memory().unwrap();
    seed_engine(&mut store, &feed(), None).unwrap();

    let text = export(&store).unwrap();
    assert!(
        validate_json(&text).is_valid(),
        "seeded store must export valid JSON: {text}"
    );
    // The resolved engine is present with its source and stays heuristic-unverified.
    assert!(text.contains("\"pcgamingwiki\""));
    assert!(text.contains("Source 2"));
    assert!(text.contains("\"heuristic-unverified\""));
    // An engine-only row carries no name for that title.
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let rec = doc["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["game"]["app_id"] == "570")
        .unwrap();
    assert!(rec["engine"].is_object(), "570 carries an engine");
    assert!(rec["game"].get("name").is_none(), "570 has no name");
}
