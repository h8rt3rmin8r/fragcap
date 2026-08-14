// SPDX-License-Identifier: Apache-2.0

//! Resumability: an engine run continues from the recorded engine-tier cursor
//! rather than restarting, and the final result equals a single uninterrupted seed
//! with no duplicates.

use fragcap_targets::{seed_engine, FixtureEngineFeed, SeedState, SeedTier, Store};

const ENGINES: &str = r#"[
  { "appid": 1, "engine": "Unity",  "confidence": "high" },
  { "appid": 2, "engine": "Unreal", "confidence": "high" },
  { "appid": 3, "engine": "Godot",  "confidence": "high" },
  { "appid": 4, "engine": "Source", "confidence": "high" },
  { "appid": 5, "engine": "Ren'Py", "confidence": "high" }
]"#;

fn source() -> FixtureEngineFeed {
    // A small page size so the seed spans several pages and the cursor matters.
    FixtureEngineFeed::from_json_with_batch(ENGINES, 2).unwrap()
}

fn engine_appids(store: &Store) -> Vec<u32> {
    store
        .games()
        .unwrap()
        .iter()
        .filter(|g| g.engine.is_some())
        .map(|g| g.appid)
        .collect()
}

#[test]
fn a_resumed_run_processes_only_the_tail_and_matches_a_full_seed() {
    // A single uninterrupted seed is the reference.
    let mut full = Store::open_in_memory().unwrap();
    seed_engine(&mut full, &source(), None).unwrap();
    assert_eq!(engine_appids(&full), vec![1, 2, 3, 4, 5]);

    // Simulate an interrupted run that got through appid 3 and recorded the cursor,
    // by writing that cursor into the store's engine seed state.
    let mut resumed = Store::open_in_memory().unwrap();
    seed_engine(&mut resumed, &source(), None).unwrap(); // populate 1..=5
    resumed
        .set_seed_state(&SeedState {
            tier: SeedTier::Engine,
            last_run_at: None,
            resume_cursor: Some("3".to_string()),
        })
        .unwrap();

    // A run from that cursor touches only appids 4 and 5.
    let summary = seed_engine(&mut resumed, &source(), None).unwrap();
    assert_eq!(
        summary.written, 2,
        "only the tail (4, 5) is processed on resume"
    );

    // The final result matches the full seed, with no duplicate rows.
    assert_eq!(engine_appids(&resumed), engine_appids(&full));
    assert_eq!(resumed.games().unwrap().len(), 5, "no duplicate rows");
}

#[test]
fn a_completed_seed_records_a_none_cursor_under_the_engine_tier() {
    let mut store = Store::open_in_memory().unwrap();
    seed_engine(&mut store, &source(), Some("2026-08-13".into())).unwrap();
    let state = store.seed_state(SeedTier::Engine).unwrap().unwrap();
    assert_eq!(
        state.resume_cursor, None,
        "a completed seed leaves no resume point"
    );
    assert_eq!(state.last_run_at.as_deref(), Some("2026-08-13"));
    // The catalog tier is a separate row and was not written by an engine seed.
    assert!(
        store.seed_state(SeedTier::Catalog).unwrap().is_none(),
        "an engine seed writes only the engine tier's state"
    );
}
