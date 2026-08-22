// SPDX-License-Identifier: Apache-2.0

//! End-to-end resolution-cascade tests for the hint-database provider (S037).
//!
//! These exercise the real `HintDatabaseProvider` alongside the real profile and
//! engine-rule providers through a `TargetResolver`, proving the precedence
//! ordering (a profile outranks a hint, a hint outranks the engine rule) and that
//! an ambiguous decline surfaces through the not-resolved outcome. Everything runs
//! offline over an in-memory store and a scratch install tree.

use std::path::{Path, PathBuf};

use fragcap_profile::{
    BundledSet, EngineRuleProvider, FidelityTier, Profile, ProfileProvider, ResolutionError,
    ResolutionRequest, SearchPath, TargetOrigin, TargetProvider, TargetResolver,
};
use fragcap_targets::entry::{ClassificationSource, TargetClassification, TargetEntry};
use fragcap_targets::identifier::{anchored_id, steam_anchor};
use fragcap_targets::{Game, HintDatabaseProvider, LaunchEntry, Store};

/// Insert a local target entry for a Steam anchor at a given fidelity, with an
/// optional single Windows launch executable (slice S051).
fn insert_target(store: &mut Store, app_id: u32, exe: Option<&str>, fidelity: FidelityTier) {
    let anchor = steam_anchor(app_id);
    let launch_entries = exe.map(|e| serde_json::json!([{ "executable": e, "os": "windows" }]));
    let entry = TargetEntry {
        id: None,
        stable_id: anchored_id(&anchor),
        handle: format!("target_a{app_id}"),
        name: format!("Target {app_id}"),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity,
        provenance: None,
        anchor: Some(anchor),
        launch_entries,
        install_root: None,
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: None,
    };
    store.insert_target(&entry).expect("insert target");
}

/// Resolve a Steam app id through a lone hint provider over the given store.
fn resolve_hint(store: Store, app_id: u32) -> Result<fragcap_profile::Target, ResolutionError> {
    let resolver = TargetResolver::new(vec![Box::new(HintDatabaseProvider::new(store))])
        .expect("one provider");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let request =
        ResolutionRequest::for_reference("unused", &search, &bundled).with_steam_app_id(app_id);
    resolver.resolve(&request)
}

#[test]
fn an_authored_local_entry_beats_the_catalog_heuristic() {
    // The same app id is present as a catalog games row (heuristic-unverified) and
    // as a locally authored target entry. The entry's higher fidelity wins, and its
    // executable is the one resolved (slice S051, fidelity-ordered store read).
    let mut store = store_with_launch(620, "catalog.exe");
    insert_target(
        &mut store,
        620,
        Some("authored.exe"),
        FidelityTier::Authored,
    );

    let target = resolve_hint(store, 620).expect("resolves");
    assert_eq!(target.fidelity(), FidelityTier::Authored);
    match target.origin() {
        TargetOrigin::HintDatabase(t) => assert_eq!(t.image_name(), "authored.exe"),
        other => panic!("expected a hint origin, got {other:?}"),
    }
}

#[test]
fn a_local_entry_with_no_executable_declines_to_the_catalog() {
    // A sparse local entry (registered by name, no launch executable) cannot name a
    // client, so it declines and the catalog games row answers at
    // heuristic-unverified. The decline is preserved as a fidelity-aware condition.
    let mut store = store_with_launch(730, "catalog.exe");
    insert_target(&mut store, 730, None, FidelityTier::Authored);

    let target = resolve_hint(store, 730).expect("resolves");
    assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
    match target.origin() {
        TargetOrigin::HintDatabase(t) => assert_eq!(t.image_name(), "catalog.exe"),
        other => panic!("expected the catalog answer, got {other:?}"),
    }
}

#[test]
fn a_local_entry_naming_two_executables_declines() {
    // A local entry naming two distinct Windows clients cannot reduce to one, so it
    // declines exactly as an ambiguous catalog row does; the catalog games answer
    // takes over. The multi-executable decline is preserved for entries too.
    let mut store = store_with_launch(570, "catalog.exe");
    let anchor = steam_anchor(570);
    let entry = TargetEntry {
        id: None,
        stable_id: anchored_id(&anchor),
        handle: "target_multi".to_string(),
        name: "Multi".to_string(),
        classification: TargetClassification::Game,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: None,
        anchor: Some(anchor),
        launch_entries: Some(serde_json::json!([
            { "executable": "one.exe", "os": "windows" },
            { "executable": "two.exe", "os": "windows" },
        ])),
        install_root: None,
        evidence: None,
        detection_scan: None,
        folder_name: None,
        executable_hint: None,
    };
    store.insert_target(&entry).expect("insert");

    let target = resolve_hint(store, 570).expect("resolves");
    assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
    match target.origin() {
        TargetOrigin::HintDatabase(t) => assert_eq!(t.image_name(), "catalog.exe"),
        other => panic!("expected the catalog answer, got {other:?}"),
    }
}

/// A store holding one game with one Windows launch executable.
fn store_with_launch(app_id: u32, exe: &str) -> Store {
    let mut store = Store::open_in_memory().expect("in-memory store");
    let mut game = Game::new(app_id);
    let mut entry = LaunchEntry::new(exe).expect("non-empty");
    entry.os = Some("windows".to_string());
    game.launch = vec![entry];
    store.upsert_game(&game).expect("upsert");
    store
}

/// A minimal Unreal install tree the engine rule resolves, removed on drop.
struct UnrealTree {
    root: PathBuf,
}

impl UnrealTree {
    fn new(game: &str, shipping_exe: &str) -> UnrealTree {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "fragcap-hint-cascade-{}-{}-{}",
            std::process::id(),
            game,
            n
        ));
        let win64 = root.join(game).join("Binaries").join("Win64");
        std::fs::create_dir_all(&win64).expect("create win64");
        std::fs::write(root.join(format!("{game}.exe")), b"").expect("stub");
        std::fs::write(win64.join(shipping_exe), b"").expect("shipping");
        UnrealTree { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for UnrealTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn a_hint_outranks_the_engine_rule_for_the_same_request() {
    // The request carries both an app id (the hint provider answers it) and an
    // install root (the engine rule would resolve an Unreal client). The hint,
    // at precedence 2, wins over the engine rule at precedence 3, regardless of
    // registration order.
    let tree = UnrealTree::new("Cascade", "Cascade-Win64-Shipping.exe");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();

    for order in 0..2 {
        let store = store_with_launch(730, "hinted.exe");
        let providers: Vec<Box<dyn TargetProvider>> = if order == 0 {
            vec![
                Box::new(HintDatabaseProvider::new(store)),
                Box::new(EngineRuleProvider::new()),
            ]
        } else {
            vec![
                Box::new(EngineRuleProvider::new()),
                Box::new(HintDatabaseProvider::new(store)),
            ]
        };
        let resolver = TargetResolver::new(providers).expect("distinct precedences");
        let request =
            ResolutionRequest::for_install(tree.path(), &search, &bundled).with_steam_app_id(730);
        let target = resolver.resolve(&request).expect("resolves");
        assert_eq!(
            target.provenance().source(),
            "hint-db",
            "the hint answer wins over the engine rule regardless of order"
        );
        assert_eq!(target.fidelity(), FidelityTier::HeuristicUnverified);
        match target.origin() {
            TargetOrigin::HintDatabase(t) => assert_eq!(t.image_name(), "hinted.exe"),
            other => panic!("expected a hint origin, got {other:?}"),
        }
    }
}

#[test]
fn a_profile_outranks_a_hint_for_the_same_request() {
    // A request carries a profile reference and an app id. The profile at
    // precedence 1 wins over the hint at precedence 2, regardless of order, and
    // the hint answer is never stamped observed or authored.
    let profile_json = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "verified",
        "game": { "id": "eso", "name": "ESO" },
        "stage": [
            { "role": "client", "lifecycle": "session", "terminal": true,
              "match": { "exe": "eso64.exe" } }
        ]
    });
    let profile = Profile::parse(&profile_json.to_string()).expect("valid profile");
    let bundled = BundledSet::new(vec![profile]).expect("one profile");
    let search = SearchPath::new();

    for order in 0..2 {
        let store = store_with_launch(480, "hinted.exe");
        let providers: Vec<Box<dyn TargetProvider>> = if order == 0 {
            vec![
                Box::new(ProfileProvider::new()),
                Box::new(HintDatabaseProvider::new(store)),
            ]
        } else {
            vec![
                Box::new(HintDatabaseProvider::new(store)),
                Box::new(ProfileProvider::new()),
            ]
        };
        let resolver = TargetResolver::new(providers).expect("distinct precedences");
        let request =
            ResolutionRequest::for_reference("eso", &search, &bundled).with_steam_app_id(480);
        let target = resolver.resolve(&request).expect("resolves");
        assert_eq!(
            target.fidelity(),
            FidelityTier::Verified,
            "the profile outranks the hint regardless of registration order"
        );
        assert!(target.profile().is_some());
    }
}

#[test]
fn an_ambiguous_hint_declines_and_surfaces_through_unresolved() {
    // A row naming two distinct Windows executables is an ambiguous decline. With
    // nothing lower to answer, the not-resolved outcome carries the hint ambiguity
    // note so it can explain why the database did not answer (P-4).
    let mut store = Store::open_in_memory().expect("in-memory store");
    let mut game = Game::new(999);
    let mut a = LaunchEntry::new("client.exe").expect("non-empty");
    a.os = Some("windows".to_string());
    let mut b = LaunchEntry::new("editor.exe").expect("non-empty");
    b.os = Some("windows".to_string());
    game.launch = vec![a, b];
    store.upsert_game(&game).expect("upsert");

    let resolver = TargetResolver::new(vec![Box::new(HintDatabaseProvider::new(store))])
        .expect("one provider");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let request =
        ResolutionRequest::for_reference("unused", &search, &bundled).with_steam_app_id(999);
    match resolver.resolve(&request) {
        Err(ResolutionError::Unresolved(u)) => {
            let ambiguity = u.hint_ambiguous().expect("the hint ambiguity is recorded");
            assert_eq!(ambiguity.app_id(), 999);
            assert_eq!(ambiguity.candidates(), 2);
        }
        other => panic!("expected Unresolved with a hint ambiguity, got {other:?}"),
    }
}

#[test]
fn a_hint_identity_is_a_usable_capture_identity() {
    // The identity a hint carries is the same shape the non-profile capture path
    // consumes: an exe predicate keyed on the executable file name, which a live
    // process can be re-matched against (the override/refine path). Round-tripping
    // it through Profile::parse (the validating path run/watch take) succeeds.
    let store = store_with_launch(570, "dota2.exe");
    let resolver = TargetResolver::new(vec![Box::new(HintDatabaseProvider::new(store))])
        .expect("one provider");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let request =
        ResolutionRequest::for_reference("unused", &search, &bundled).with_steam_app_id(570);
    let target = resolver.resolve(&request).expect("resolves");

    let identity = target
        .identity()
        .expect("a hint carries a capture identity");
    let exe = identity.exe().expect("keyed on the executable");
    assert!(exe.matches("dota2.exe"), "matches the named executable");

    // The identity serializes into a one-stage profile and validates, exactly as
    // the non-profile `run` path synthesizes it.
    let profile_json = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "heuristic-unverified",
        "game": { "id": "target", "name": "ad hoc target" },
        "stage": [
            { "role": "target", "lifecycle": "session", "terminal": true,
              "match": { "exe": exe.as_str() } }
        ]
    });
    Profile::parse(&profile_json.to_string()).expect("the hint identity yields a valid profile");
}
