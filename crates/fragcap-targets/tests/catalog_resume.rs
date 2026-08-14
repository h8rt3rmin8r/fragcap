// SPDX-License-Identifier: Apache-2.0

//! Resumability: a run continues from the recorded cursor rather than restarting,
//! and the final corpus equals a single uninterrupted seed with no duplicates.

use fragcap_targets::{seed_catalog, CorpusGate, FixtureCatalog, SeedState, SeedTier, Store};

const CATALOG: &str = r#"[
  { "appid": 1, "name": "One",   "classification": "game", "review_count": 1000 },
  { "appid": 2, "name": "Two",   "classification": "game", "review_count": 1000 },
  { "appid": 3, "name": "Three", "classification": "game", "review_count": 1000 },
  { "appid": 4, "name": "Four",  "classification": "game", "review_count": 1000 },
  { "appid": 5, "name": "Five",  "classification": "game", "review_count": 1000 }
]"#;

fn source() -> FixtureCatalog {
    // A small page size so the seed spans several pages and the cursor matters.
    FixtureCatalog::from_json_with_batch(CATALOG, 2).unwrap()
}

fn appids(store: &Store) -> Vec<u32> {
    store.games().unwrap().iter().map(|g| g.appid).collect()
}

#[test]
fn a_resumed_run_processes_only_the_tail_and_matches_a_full_seed() {
    let gate = CorpusGate::new(100);

    // A single uninterrupted seed is the reference.
    let mut full = Store::open_in_memory().unwrap();
    seed_catalog(&mut full, &source(), &gate, None).unwrap();
    assert_eq!(appids(&full), vec![1, 2, 3, 4, 5]);

    // Simulate an interrupted run that got through appid 3 and recorded the cursor,
    // by writing that cursor into the store's catalog seed state.
    let mut resumed = Store::open_in_memory().unwrap();
    seed_catalog(&mut resumed, &source(), &gate, None).unwrap(); // populate 1..=5
    resumed
        .set_seed_state(&SeedState {
            tier: SeedTier::Catalog,
            last_run_at: None,
            resume_cursor: Some("3".to_string()),
        })
        .unwrap();

    // A run from that cursor touches only appids 4 and 5.
    let summary = seed_catalog(&mut resumed, &source(), &gate, None).unwrap();
    assert_eq!(
        summary.written, 2,
        "only the tail (4, 5) is processed on resume"
    );

    // The final corpus matches the full seed, with no duplicate rows.
    assert_eq!(appids(&resumed), appids(&full));
    assert_eq!(resumed.games().unwrap().len(), 5, "no duplicate rows");
}

#[test]
fn a_completed_seed_records_a_none_cursor_and_the_last_run() {
    let mut store = Store::open_in_memory().unwrap();
    seed_catalog(
        &mut store,
        &source(),
        &CorpusGate::new(100),
        Some("2026-08-13".into()),
    )
    .unwrap();
    let state = store.seed_state(SeedTier::Catalog).unwrap().unwrap();
    assert_eq!(
        state.resume_cursor, None,
        "a completed seed leaves no resume point"
    );
    assert_eq!(state.last_run_at.as_deref(), Some("2026-08-13"));
}
