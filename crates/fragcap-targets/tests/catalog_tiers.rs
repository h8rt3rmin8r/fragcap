// SPDX-License-Identifier: Apache-2.0

//! The Tier 1 merge updates only the catalog columns: an engine (Tier 3) and
//! launch entries (Tier 2) survive a catalog refresh, and the seeder never prunes.

use fragcap_targets::{
    seed_catalog, CorpusGate, Engine, EngineConfidence, EngineSource, FixtureCatalog, Game,
    LaunchEntry, Store,
};

fn enriched_game(appid: u32, name: &str) -> Game {
    let mut g = Game::new(appid);
    g.name = Some(name.to_string());
    g.launcher_mediated = Some(true);
    g.launch = vec![LaunchEntry::new("game.exe").unwrap()];
    g.engine = Some(Engine {
        name: Some("Source 2".to_string()),
        source: EngineSource::ExeHeuristic,
        confidence: EngineConfidence::Medium,
    });
    g
}

#[test]
fn a_tier1_seed_updates_the_name_but_preserves_engine_and_launch() {
    let mut store = Store::open_in_memory().unwrap();
    // A fully enriched game (Tiers 2 and 3 present).
    store.upsert_game(&enriched_game(570, "Old Name")).unwrap();

    // A catalog carrying the same appid with a new name.
    let catalog = FixtureCatalog::from_json(
        r#"[ { "appid": 570, "name": "Dota 2", "classification": "game", "review_count": 2000000 } ]"#,
    )
    .unwrap();
    seed_catalog(&mut store, &catalog, &CorpusGate::new(100), None).unwrap();

    let game = store
        .games()
        .unwrap()
        .into_iter()
        .find(|g| g.appid == 570)
        .unwrap();
    assert_eq!(
        game.name.as_deref(),
        Some("Dota 2"),
        "the name was refreshed"
    );
    assert_eq!(
        game.engine.as_ref().map(|e| e.source),
        Some(EngineSource::ExeHeuristic),
        "the engine (Tier 3) survives a Tier 1 refresh"
    );
    assert_eq!(game.launch.len(), 1, "the launch entries (Tier 2) survive");
    assert_eq!(game.launcher_mediated, Some(true));
}

#[test]
fn the_seeder_never_prunes_titles_absent_from_a_run() {
    let mut store = Store::open_in_memory().unwrap();
    // A stored game the catalog run will not mention.
    store.upsert_game(&enriched_game(999, "Kept Game")).unwrap();

    let catalog = FixtureCatalog::from_json(
        r#"[ { "appid": 570, "name": "Dota 2", "classification": "game", "review_count": 2000000 } ]"#,
    )
    .unwrap();
    seed_catalog(&mut store, &catalog, &CorpusGate::new(100), None).unwrap();

    let kept = store.games().unwrap().into_iter().find(|g| g.appid == 999);
    assert!(
        kept.is_some(),
        "a title absent from the run must not be pruned"
    );
    assert!(kept.unwrap().engine.is_some(), "its data is untouched");
}
