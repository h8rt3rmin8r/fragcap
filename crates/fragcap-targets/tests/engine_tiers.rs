// SPDX-License-Identifier: Apache-2.0

//! The Tier 3 merge writes only the engine columns: a catalog name (Tier 1) and
//! launch entries (Tier 2) survive an engine seed, and the seeder never prunes.

use fragcap_targets::{
    seed_engine, Engine, EngineConfidence, EngineSource, FixtureEngineFeed, Game, LaunchEntry,
    Store,
};

fn enriched_game(appid: u32, name: &str) -> Game {
    let mut g = Game::new(appid);
    g.name = Some(name.to_string());
    g.launcher_mediated = Some(true);
    g.launch = vec![LaunchEntry::new("game.exe").unwrap()];
    g
}

#[test]
fn an_engine_seed_fills_engine_but_preserves_catalog_and_launch() {
    let mut store = Store::open_in_memory().unwrap();
    // A game already carrying Tier 1 (name, launcher flag) and Tier 2 (launch) data.
    store.upsert_game(&enriched_game(570, "Dota 2")).unwrap();

    // An engine feed carrying the same appid with an engine.
    let feed = FixtureEngineFeed::from_json(
        r#"[ { "appid": 570, "engine": "Source 2", "confidence": "high" } ]"#,
    )
    .unwrap();
    seed_engine(&mut store, &feed, None).unwrap();

    let game = store
        .games()
        .unwrap()
        .into_iter()
        .find(|g| g.appid == 570)
        .unwrap();
    // The engine (Tier 3) is now set, stamped pcgamingwiki.
    assert_eq!(
        game.engine.as_ref().map(|e| e.source),
        Some(EngineSource::Pcgamingwiki),
        "the engine was written"
    );
    assert_eq!(
        game.engine.as_ref().map(|e| e.confidence),
        Some(EngineConfidence::High)
    );
    assert_eq!(
        game.engine.as_ref().and_then(|e| e.name.as_deref()),
        Some("Source 2")
    );
    // The name (Tier 1) survives the Tier 3 seed. This is the SC-003 guarantee.
    assert_eq!(game.name.as_deref(), Some("Dota 2"), "the name survives");
    // The launch entries (Tier 2) and launcher flag survive.
    assert_eq!(game.launch.len(), 1, "the launch entries survive");
    assert_eq!(game.launcher_mediated, Some(true));
}

#[test]
fn an_engine_seed_overwrites_a_prior_engine_from_a_different_source() {
    let mut store = Store::open_in_memory().unwrap();
    let mut g = enriched_game(730, "CS2");
    g.engine = Some(Engine {
        name: Some("Placeholder".to_string()),
        source: EngineSource::ExeHeuristic,
        confidence: EngineConfidence::Low,
    });
    store.upsert_game(&g).unwrap();

    let feed = FixtureEngineFeed::from_json(
        r#"[ { "appid": 730, "engine": "Source 2", "confidence": "confirmed" } ]"#,
    )
    .unwrap();
    seed_engine(&mut store, &feed, None).unwrap();

    let game = store
        .games()
        .unwrap()
        .into_iter()
        .find(|g| g.appid == 730)
        .unwrap();
    assert_eq!(
        game.engine.as_ref().map(|e| e.source),
        Some(EngineSource::Pcgamingwiki)
    );
    assert_eq!(game.name.as_deref(), Some("CS2"), "the name still survives");
    assert_eq!(game.launch.len(), 1);
}

#[test]
fn the_seeder_never_prunes_titles_absent_from_a_run() {
    let mut store = Store::open_in_memory().unwrap();
    // A stored game the engine run will not mention.
    store.upsert_game(&enriched_game(999, "Kept Game")).unwrap();

    let feed = FixtureEngineFeed::from_json(
        r#"[ { "appid": 570, "engine": "Source 2", "confidence": "high" } ]"#,
    )
    .unwrap();
    seed_engine(&mut store, &feed, None).unwrap();

    let kept = store.games().unwrap().into_iter().find(|g| g.appid == 999);
    assert!(
        kept.is_some(),
        "a title absent from the run must not be pruned"
    );
    let kept = kept.unwrap();
    assert_eq!(
        kept.name.as_deref(),
        Some("Kept Game"),
        "its data is untouched"
    );
    assert_eq!(kept.launch.len(), 1);
    assert!(kept.engine.is_none(), "no engine was invented for it");
}
