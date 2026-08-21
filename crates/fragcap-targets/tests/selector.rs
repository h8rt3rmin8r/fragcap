// SPDX-License-Identifier: Apache-2.0

//! Selector resolution tests (slice S051, specification section 15.8).

use fragcap_profile::FidelityTier;
use fragcap_targets::entry::{ClassificationSource, TargetClassification, TargetEntry};
use fragcap_targets::identifier::{anchored_id, unanchored_id};
use fragcap_targets::selector::{resolve_id, resolve_positional, Selection};
use fragcap_targets::Store;

fn target(handle: &str, name: &str, anchor: Option<&str>) -> TargetEntry {
    TargetEntry {
        id: None,
        stable_id: anchor.map(anchored_id).unwrap_or_else(unanchored_id),
        handle: handle.to_string(),
        name: name.to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: anchor.map(|a| a.to_string()),
        launch_entries: None,
        install_root: None,
        evidence: None,
        detection_scan: None,
    }
}

fn store_with(entries: &[TargetEntry]) -> Store {
    let mut store = Store::open_in_memory().expect("store");
    for e in entries {
        store.insert_target(e).expect("insert");
    }
    store
}

#[test]
fn exact_handle_resolves_one() {
    let store = store_with(&[target("portal_2", "Portal 2", Some("steam:620"))]);
    match resolve_positional(&store, "portal_2").expect("resolve") {
        Selection::Resolved(t) => assert_eq!(t.handle, "portal_2"),
        other => panic!("expected Resolved, got {other:?}"),
    }
}

#[test]
fn case_insensitive_exact_name_resolves_one() {
    let store = store_with(&[target("portal_2", "Portal 2", Some("steam:620"))]);
    match resolve_positional(&store, "portal 2").expect("resolve") {
        Selection::Resolved(t) => assert_eq!(t.handle, "portal_2"),
        other => panic!("expected Resolved, got {other:?}"),
    }
}

#[test]
fn id_resolves_by_stable_id() {
    let store = store_with(&[target("portal_2", "Portal 2", Some("steam:620"))]);
    let id = anchored_id("steam:620");
    match resolve_id(&store, id).expect("resolve") {
        Selection::Resolved(t) => assert_eq!(t.handle, "portal_2"),
        other => panic!("expected Resolved, got {other:?}"),
    }
}

#[test]
fn bare_integer_indexes_the_listing_snapshot_one_based() {
    let mut store = store_with(&[
        target("first", "First", Some("steam:1")),
        target("second", "Second", Some("steam:2")),
    ]);
    // A row index resolves against the snapshot the listing wrote (slice S055), so
    // it names the row the user saw. Pin a snapshot: row 1 = first, row 2 = second.
    store
        .write_listing_snapshot(&[
            (anchored_id("steam:1"), "first"),
            (anchored_id("steam:2"), "second"),
        ])
        .expect("snapshot");
    match resolve_positional(&store, "2").expect("resolve") {
        Selection::Resolved(t) => assert_eq!(t.handle, "second"),
        other => panic!("expected Resolved, got {other:?}"),
    }
    // Out-of-range and zero are clean no-matches, not panics.
    assert!(matches!(
        resolve_positional(&store, "9").expect("resolve"),
        Selection::NoMatch
    ));
    assert!(matches!(
        resolve_positional(&store, "0").expect("resolve"),
        Selection::NoMatch
    ));
}

#[test]
fn a_bare_integer_with_no_snapshot_is_a_no_match() {
    // Before any listing writes a snapshot, a row index resolves nothing (rather
    // than silently indexing the live store order).
    let store = store_with(&[target("only", "Only", Some("steam:1"))]);
    assert!(matches!(
        resolve_positional(&store, "1").expect("resolve"),
        Selection::NoMatch
    ));
}

#[test]
fn zero_is_an_invalid_row_index_not_a_name() {
    use fragcap_targets::is_row_index;
    // `0` is a numeric token, so it is the row-index path (an invalid position), not
    // a handle/name lookup. Callers map a row-index no-match to a usage error.
    assert!(is_row_index("0"), "0 is classified as a row index");
    let store = store_with(&[target("only", "Only", Some("steam:1"))]);
    assert!(matches!(
        resolve_positional(&store, "0").expect("resolve"),
        Selection::NoMatch
    ));
}

#[test]
fn a_name_matching_more_than_one_is_ambiguous() {
    let store = store_with(&[
        target("portal_2", "Portal 2", Some("steam:620")),
        target("portal_2_2", "Portal 2", Some("steam:200")),
    ]);
    match resolve_positional(&store, "Portal 2").expect("resolve") {
        Selection::Ambiguous(matches) => {
            assert_eq!(matches.len(), 2);
            // The caller has the handles and ids it needs to print a disambiguation.
            let handles: Vec<_> = matches.iter().map(|t| t.handle.as_str()).collect();
            assert!(handles.contains(&"portal_2") && handles.contains(&"portal_2_2"));
        }
        other => panic!("expected Ambiguous, got {other:?}"),
    }
}

#[test]
fn zero_matches_is_a_distinct_no_match() {
    let store = store_with(&[target("portal_2", "Portal 2", Some("steam:620"))]);
    assert!(matches!(
        resolve_positional(&store, "nonexistent").expect("resolve"),
        Selection::NoMatch
    ));
}

#[test]
fn id_resolves_a_superseded_alias() {
    let mut store = Store::open_in_memory().expect("store");
    let t = target("my_game", "My Game", None);
    let old_id = t.stable_id;
    let row = store.insert_target(&t).expect("insert");
    store
        .supersede_with_anchor(row, "steam:2221490")
        .expect("supersede");
    // The old (now superseded) id still resolves via the alias.
    match resolve_id(&store, old_id).expect("resolve") {
        Selection::Resolved(t) => assert_eq!(t.handle, "my_game"),
        other => panic!("expected Resolved via alias, got {other:?}"),
    }
}
