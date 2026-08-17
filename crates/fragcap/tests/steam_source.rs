// SPDX-License-Identifier: Apache-2.0
#![cfg(feature = "targets")]

//! Tier 1, `SteamSource` through the shared discovery seam (S052 spec US1,
//! FR-004/005/006). Driven against a fixture Steam tree in a temporary directory,
//! no Steam installation and no live catalog. Asserts candidate parity with the
//! underlying `fragcap-steam` walk (no regression) and the catalog join.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap::profile::FidelityTier;
use fragcap::steam::discover_in;
use fragcap::targets::{CandidateIdentity, Store, TargetClassification, TargetSource};
use fragcap::SteamSource;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A throwaway directory tree, removed on drop (mirrors the fragcap-steam pattern).
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new() -> TempTree {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("fragcap-s052-steam-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&root).unwrap();
        TempTree { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn manifest(app_id: &str, name: &str, installdir: &str) -> String {
    format!(
        "\"AppState\"\n{{\n  \"appid\" \"{app_id}\"\n  \"name\" \"{name}\"\n  \
         \"installdir\" \"{installdir}\"\n}}\n"
    )
}

/// Lay out a fixture Steam root: two libraries, three numeric titles, and one
/// manifest whose appid is not a number (to exercise `parse_failed`).
fn fixture_steam_root(tree: &TempTree) {
    let root = tree.path();
    let lib_b = root.join("SteamLibrary");
    let esc = |p: &Path| p.display().to_string().replace('\\', "\\\\");
    tree.write(
        &root.join("steamapps").join("libraryfolders.vdf"),
        &format!(
            "\"libraryfolders\"\n{{\n  \"0\" {{ \"path\" \"{}\" }}\n  \"1\" {{ \"path\" \"{}\" }}\n}}\n",
            esc(root),
            esc(&lib_b),
        ),
    );
    // Root library: CS2 (in catalog) and Portal 2 (not in catalog).
    tree.write(
        &root.join("steamapps").join("appmanifest_730.acf"),
        &manifest("730", "Counter-Strike 2", "cs2"),
    );
    tree.write(
        &root.join("steamapps").join("appmanifest_620.acf"),
        &manifest("620", "Portal 2", "Portal 2"),
    );
    // Second library: one numeric title and one non-numeric appid (a parse fault).
    tree.write(
        &lib_b.join("steamapps").join("appmanifest_400500.acf"),
        &manifest("400500", "Some Indie", "indie"),
    );
    tree.write(
        &lib_b.join("steamapps").join("appmanifest_bogus.acf"),
        &manifest("not-a-number", "Corrupt Appid", "corrupt"),
    );
}

/// A catalog store holding only appid 730, so the join hits CS2 and misses the rest.
fn catalog_with_cs2() -> Store {
    let mut store = Store::open_in_memory().unwrap();
    store
        .merge_catalog(730, Some("Counter-Strike 2"), Some(1_000_000), None, None)
        .unwrap();
    store
}

#[test]
fn steam_source_produces_one_candidate_per_numeric_title() {
    let tree = TempTree::new();
    fixture_steam_root(&tree);
    let catalog = catalog_with_cs2();

    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();

    // Four manifests considered: three numeric titles produced, one non-numeric
    // appid counted parse_failed, none lost.
    assert_eq!(d.account.considered, 4);
    assert_eq!(d.account.produced, 3);
    assert_eq!(d.account.parse_failed, 1);
    assert!(d.account.is_conserved());

    // Every produced candidate is stamped heuristic-unverified and attributed.
    for c in &d.candidates {
        assert_eq!(c.fidelity, FidelityTier::HeuristicUnverified);
        assert_eq!(c.source_name, "steam");
    }
}

#[test]
fn the_catalog_join_classifies_a_hit_and_leaves_a_miss_unknown() {
    let tree = TempTree::new();
    fixture_steam_root(&tree);
    let catalog = catalog_with_cs2();

    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();

    let cs2 = d
        .candidates
        .iter()
        .find(|c| c.identity == CandidateIdentity::SteamAppId(730))
        .expect("CS2 candidate present");
    assert_eq!(
        cs2.classification,
        TargetClassification::Game,
        "a catalog hit is a game"
    );

    let portal = d
        .candidates
        .iter()
        .find(|c| c.identity == CandidateIdentity::SteamAppId(620))
        .expect("Portal 2 candidate present");
    assert_eq!(
        portal.classification,
        TargetClassification::Unknown,
        "an appid absent from the catalog is Unknown, never dropped (P-9)"
    );
}

#[test]
fn candidate_set_matches_the_underlying_steam_walk_no_regression() {
    let tree = TempTree::new();
    fixture_steam_root(&tree);
    let catalog = catalog_with_cs2();

    // The pre-refactor behavior: the fragcap-steam library walk's numeric titles.
    let installation = discover_in(tree.path()).unwrap();
    let walk_numeric_appids: BTreeSet<u32> = installation
        .titles
        .iter()
        .filter_map(|t| t.app_id.parse::<u32>().ok())
        .collect();

    // SteamSource's produced candidates.
    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();
    let source_appids: BTreeSet<u32> = d
        .candidates
        .iter()
        .map(|c| match c.identity {
            CandidateIdentity::SteamAppId(a) => a,
            ref other => panic!("expected a Steam appid identity, got {other:?}"),
        })
        .collect();

    assert_eq!(
        source_appids, walk_numeric_appids,
        "SteamSource must not change the candidate set the library walk produced (FR-006)"
    );
    assert_eq!(source_appids, BTreeSet::from([620, 730, 400500]));
}
