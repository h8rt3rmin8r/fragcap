// SPDX-License-Identifier: Apache-2.0

//! The MVP: an offline catalog seed fills the corpus, accounts for every title
//! truthfully, and exports schema-valid.

use fragcap_profile::jsonschema::validate_json;
use fragcap_targets::{export, seed_catalog, CorpusGate, FixtureCatalog, Store};

fn catalog() -> FixtureCatalog {
    let text = include_str!("fixtures/catalog.json");
    FixtureCatalog::from_json(text).unwrap()
}

#[test]
fn a_seed_fills_exactly_the_in_corpus_titles() {
    let mut store = Store::open_in_memory().unwrap();
    let gate = CorpusGate::new(100);
    let summary = seed_catalog(&mut store, &catalog(), &gate, Some("2026-08-13".into())).unwrap();

    // 3 games clear the game+threshold gate: 306130, 570, 730.
    assert_eq!(summary.written, 3);
    // 440 is below the threshold; 700 is not a game.
    assert_eq!(summary.excluded, 2);
    // The entry with no appid could not be parsed.
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.fetched, 6);

    let appids: Vec<u32> = store.games().unwrap().iter().map(|g| g.appid).collect();
    assert_eq!(appids, vec![570, 730, 306130]);
}

#[test]
fn every_title_is_accounted_for_no_silent_loss() {
    let mut store = Store::open_in_memory().unwrap();
    let summary = seed_catalog(&mut store, &catalog(), &CorpusGate::new(100), None).unwrap();
    assert!(
        summary.is_conserved(),
        "fetched must equal written + excluded + failed: {summary:?}"
    );
}

#[test]
fn a_nameless_in_corpus_title_is_written_without_a_name() {
    let mut store = Store::open_in_memory().unwrap();
    seed_catalog(&mut store, &catalog(), &CorpusGate::new(100), None).unwrap();
    let game = store
        .games()
        .unwrap()
        .into_iter()
        .find(|g| g.appid == 730)
        .expect("730 is in corpus");
    assert_eq!(game.name, None, "an empty/absent name is stored as absent");
}

#[test]
fn the_store_exports_schema_valid_after_seeding() {
    let mut store = Store::open_in_memory().unwrap();
    seed_catalog(&mut store, &catalog(), &CorpusGate::new(100), None).unwrap();
    let text = export(&store).unwrap();
    assert!(
        validate_json(&text).is_valid(),
        "a seeded store must export schema-valid JSON: {text}"
    );
}

#[test]
fn a_wrong_typed_field_is_counted_failed_not_excluded() {
    // A present but wrong-typed review_count must count as failed (malformed), not
    // be coerced to "no reviews" and reported as excluded.
    let catalog = FixtureCatalog::from_json(
        r#"[
          { "appid": 10, "name": "Good", "classification": "game", "review_count": 5000 },
          { "appid": 20, "name": "BadReviews", "classification": "game", "review_count": "lots" }
        ]"#,
    )
    .unwrap();
    let mut store = Store::open_in_memory().unwrap();
    let summary = seed_catalog(&mut store, &catalog, &CorpusGate::new(100), None).unwrap();
    assert_eq!(summary.written, 1);
    assert_eq!(summary.failed, 1, "the wrong-typed entry is failed");
    assert_eq!(summary.excluded, 0, "it is not miscounted as excluded");
    assert!(summary.is_conserved());
}

#[test]
fn a_repeated_appid_is_written_once() {
    let catalog = FixtureCatalog::from_json(
        r#"[
          { "appid": 42, "name": "First",  "classification": "game", "review_count": 5000 },
          { "appid": 42, "name": "Second", "classification": "game", "review_count": 6000 }
        ]"#,
    )
    .unwrap();
    let mut store = Store::open_in_memory().unwrap();
    let summary = seed_catalog(&mut store, &catalog, &CorpusGate::new(100), None).unwrap();
    assert_eq!(summary.written, 1, "a distinct title is written once");
    assert_eq!(
        summary.duplicates, 1,
        "the repeat is a duplicate, not a second write"
    );
    assert_eq!(store.games().unwrap().len(), 1, "the store holds one row");
    assert!(summary.is_conserved());
}

#[test]
fn a_higher_threshold_shrinks_the_corpus() {
    let mut store = Store::open_in_memory().unwrap();
    // Only 570 (2,000,000) and 730 (100,000) clear 1,000,000... only 570.
    let summary = seed_catalog(&mut store, &catalog(), &CorpusGate::new(1_000_000), None).unwrap();
    assert_eq!(summary.written, 1);
    assert_eq!(store.games().unwrap()[0].appid, 570);
    assert!(summary.is_conserved());
}
