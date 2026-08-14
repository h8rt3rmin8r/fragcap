// SPDX-License-Identifier: Apache-2.0

//! The three seeding tiers fill independently: a Tier-1-only game exports valid,
//! is later enriched with a Tier-3 engine without disturbing Tier 1, and per-tier
//! seed state round-trips.

use fragcap_profile::jsonschema::validate_json;
use fragcap_targets::{
    export, Engine, EngineConfidence, EngineSource, Game, SeedState, SeedTier, Store,
};
use serde_json::Value;

#[test]
fn a_game_can_be_enriched_one_tier_at_a_time() {
    let mut store = Store::open_in_memory().unwrap();

    // Tier 1 only: appid + name.
    let mut g = Game::new(730);
    g.name = Some("Counter-Strike 2".to_string());
    store.upsert_game(&g).unwrap();

    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    let record = &doc["records"][0];
    assert_eq!(record["game"]["app_id"], "730");
    assert!(record.get("engine").is_none());
    assert!(record.get("launch").is_none());
    assert!(validate_json(&export(&store).unwrap()).is_valid());

    // Tier 3 enrichment: add an engine to the same appid.
    g.engine = Some(Engine {
        name: Some("Source 2".to_string()),
        source: EngineSource::Pcgamingwiki,
        confidence: EngineConfidence::Confirmed,
    });
    store.upsert_game(&g).unwrap();

    let games = store.games().unwrap();
    assert_eq!(games.len(), 1, "enrichment must not create a second game");
    let stored = &games[0];
    assert_eq!(stored.name.as_deref(), Some("Counter-Strike 2"));
    assert_eq!(
        stored.engine.as_ref().map(|e| e.source),
        Some(EngineSource::Pcgamingwiki)
    );

    let doc: Value = serde_json::from_str(&export(&store).unwrap()).unwrap();
    let record = &doc["records"][0];
    assert_eq!(record["engine"]["source"], "pcgamingwiki");
    assert_eq!(record["engine"]["confidence"], "confirmed");
    assert_eq!(
        record["game"]["name"], "Counter-Strike 2",
        "Tier 1 data must survive Tier 3 enrichment"
    );
    assert!(validate_json(&export(&store).unwrap()).is_valid());
}

#[test]
fn seed_state_round_trips_per_tier() {
    let mut store = Store::open_in_memory().unwrap();
    assert!(store.seed_state(SeedTier::Catalog).unwrap().is_none());

    store
        .set_seed_state(&SeedState {
            tier: SeedTier::Catalog,
            last_run_at: Some("2026-08-13T00:00:00Z".to_string()),
            resume_cursor: Some("cursor-42".to_string()),
        })
        .unwrap();

    let state = store.seed_state(SeedTier::Catalog).unwrap().unwrap();
    assert_eq!(state.tier, SeedTier::Catalog);
    assert_eq!(state.resume_cursor.as_deref(), Some("cursor-42"));
    // A different tier is still unset.
    assert!(store.seed_state(SeedTier::Engine).unwrap().is_none());
}

#[test]
fn reimport_is_idempotent() {
    let mut store = Store::open_in_memory().unwrap();
    let mut g = Game::new(440);
    g.name = Some("Team Fortress 2".to_string());
    store.upsert_game(&g).unwrap();
    store.upsert_game(&g).unwrap();
    assert_eq!(
        store.games().unwrap().len(),
        1,
        "re-upsert must replace, not duplicate"
    );
}
