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

    fn write_bytes(&self, path: &Path, contents: &[u8]) {
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
fn every_candidate_carries_its_resolved_install_root() {
    // Review of PR #193: without this, a Steam-sourced registration stored no
    // install_root at all, so the missing-install-root detection (issue #167)
    // never fired for the dominant real-world case (a Steam-discovered target).
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
        cs2.install_root.as_deref(),
        Some(
            tree.path()
                .join("steamapps")
                .join("common")
                .join("cs2")
                .to_str()
                .unwrap()
        )
    );
}

#[test]
fn a_title_with_a_local_engine_marker_is_verified_with_evidence() {
    // Detection runs in the Steam scan phase (FR-006): a title whose install
    // directory carries a definitive engine marker is stamped verified, outranking
    // the remote catalog attribution (P-9), and carries the engine plus any
    // anti-cheat as neutral evidence.
    let tree = TempTree::new();
    fixture_steam_root(&tree);
    // CS2's install directory (steamapps/common/cs2) carries a Unity marker and an
    // anti-cheat marker.
    let install = tree.path().join("steamapps").join("common").join("cs2");
    tree.write(&install.join("UnityPlayer.dll"), "");
    tree.write(
        &install.join("EasyAntiCheat").join("EasyAntiCheat_x64.dll"),
        "",
    );

    let mut catalog = catalog_with_cs2();
    fragcap::targets::seed_bundled(&mut catalog).expect("seed signatures");

    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();

    let cs2 = d
        .candidates
        .iter()
        .find(|c| c.identity == CandidateIdentity::SteamAppId(730))
        .expect("CS2 candidate present");
    assert_eq!(
        cs2.fidelity,
        FidelityTier::Verified,
        "a local definitive engine marker is verified (P-9)"
    );
    assert!(cs2.evidence.iter().any(|f| f.product == "Unity"));
    assert!(cs2.evidence.iter().any(|f| f.product == "Easy Anti-Cheat"));

    // A title with no local marker stays heuristic-unverified.
    let portal = d
        .candidates
        .iter()
        .find(|c| c.identity == CandidateIdentity::SteamAppId(620))
        .expect("Portal 2 candidate present");
    assert_eq!(portal.fidelity, FidelityTier::HeuristicUnverified);
}

#[test]
fn a_directory_marker_and_an_appinfo_signal_for_the_same_product_merge_to_one_finding() {
    // FR-005 (slice S068, issue #170): a title whose install directory carries an
    // Easy Anti-Cheat marker AND whose appinfo launch entry also signals Easy
    // Anti-Cheat must report the product exactly once, not twice.
    use fragcap_steam::appinfo::fixtures::{appinfo_bytes, FixtureApp, FixtureLaunch, V29};

    let tree = TempTree::new();
    fixture_steam_root(&tree);
    let install = tree.path().join("steamapps").join("common").join("cs2");
    tree.write(
        &install.join("EasyAntiCheat").join("EasyAntiCheat_x64.dll"),
        "",
    );
    let appinfo = appinfo_bytes(
        V29,
        &[FixtureApp {
            appid: 730,
            change_number: 1,
            launch: vec![FixtureLaunch {
                description: Some("eac-release".to_string()),
                ..FixtureLaunch::windows("cs2.exe")
            }],
            common_type: Some("Game".to_string()),
        }],
    );
    tree.write_bytes(&tree.path().join("appcache").join("appinfo.vdf"), &appinfo);

    let mut catalog = catalog_with_cs2();
    fragcap::targets::seed_bundled(&mut catalog).expect("seed signatures");

    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();

    let cs2 = d
        .candidates
        .iter()
        .find(|c| c.identity == CandidateIdentity::SteamAppId(730))
        .expect("CS2 candidate present");
    let eac_findings: Vec<_> = cs2
        .evidence
        .iter()
        .filter(|f| f.product == "Easy Anti-Cheat")
        .collect();
    assert_eq!(
        eac_findings.len(),
        1,
        "the directory marker and the appinfo signal must merge to one finding: {:?}",
        cs2.evidence
    );
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

#[test]
fn non_game_app_types_are_never_candidates_and_are_counted_not_a_game() {
    // Issues #166 and #212: Steam records several installed app classes that are
    // not capture targets. Each is counted through the existing
    // considered_not_a_game outcome (P-4) rather than dropped silently.
    use fragcap_steam::appinfo::fixtures::{appinfo_bytes, FixtureApp, V29};

    let tree = TempTree::new();
    fixture_steam_root(&tree); // CS2 (730), Portal 2 (620), Some Indie (400500)
    let excluded = [
        (
            450070,
            "Oblivion Soundtrack",
            "Oblivion Soundtrack",
            "Music",
        ),
        (
            228980,
            "Steamworks Common Redistributables",
            "Steamworks Shared",
            "Tool",
        ),
        (900010, "Dedicated Server Config", "Server Config", "Config"),
        (900011, "Trailer Collection", "Trailer Collection", "Video"),
        (
            900012,
            "Benchmark Companion",
            "Benchmark Companion",
            "Application",
        ),
    ];
    for (appid, name, installdir, _) in excluded {
        tree.write(
            &tree
                .path()
                .join("steamapps")
                .join(format!("appmanifest_{appid}.acf")),
            &manifest(&appid.to_string(), name, installdir),
        );
    }
    let bytes = appinfo_bytes(
        V29,
        &excluded
            .iter()
            .map(|(appid, _, _, app_type)| FixtureApp {
                appid: *appid,
                change_number: 1,
                launch: vec![],
                common_type: Some((*app_type).to_string()),
            })
            .collect::<Vec<_>>(),
    );
    tree.write_bytes(&tree.path().join("appcache").join("appinfo.vdf"), &bytes);

    let catalog = catalog_with_cs2();
    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();

    for (appid, _, _, app_type) in excluded {
        assert!(
            !d.candidates
                .iter()
                .any(|c| c.identity == CandidateIdentity::SteamAppId(appid)),
            "a {app_type}-typed app must never be produced as a candidate"
        );
    }
    assert_eq!(
        d.account.considered, 9,
        "the four fixture titles plus the five excluded app types"
    );
    assert_eq!(d.account.produced, 3, "unchanged from the non-music case");
    assert_eq!(
        d.account.considered_not_a_game, 5,
        "every excluded app type is counted, not silently dropped"
    );
    assert!(d.account.is_conserved());
}

#[test]
fn demo_game_and_unknown_app_types_remain_candidates() {
    // S086 keeps the filter narrow: a demo can be playable, and an unknown app
    // type is an observation gap rather than evidence that the title is non-game.
    use fragcap_steam::appinfo::fixtures::{appinfo_bytes, FixtureApp, V29};

    let tree = TempTree::new();
    let root = tree.path();
    tree.write(
        &root.join("steamapps").join("libraryfolders.vdf"),
        &format!(
            "\"libraryfolders\"\n{{\n  \"0\" {{ \"path\" \"{}\" }}\n}}\n",
            root.display().to_string().replace('\\', "\\\\"),
        ),
    );
    let preserved = [
        (111, "Playable Demo", "Playable Demo"),
        (222, "Full Game", "Full Game"),
        (333, "Untyped Game", "Untyped Game"),
    ];
    for (appid, name, installdir) in preserved {
        tree.write(
            &root
                .join("steamapps")
                .join(format!("appmanifest_{appid}.acf")),
            &manifest(&appid.to_string(), name, installdir),
        );
    }
    let bytes = appinfo_bytes(
        V29,
        &[
            FixtureApp {
                appid: 111,
                change_number: 1,
                launch: vec![],
                common_type: Some("Demo".to_string()),
            },
            FixtureApp {
                appid: 222,
                change_number: 1,
                launch: vec![],
                common_type: Some("Game".to_string()),
            },
        ],
    );
    tree.write_bytes(&root.join("appcache").join("appinfo.vdf"), &bytes);

    let catalog = Store::open_in_memory().unwrap();
    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();
    let appids: BTreeSet<u32> = d
        .candidates
        .iter()
        .map(|c| match c.identity {
            CandidateIdentity::SteamAppId(appid) => appid,
            ref other => panic!("expected a Steam appid identity, got {other:?}"),
        })
        .collect();

    assert_eq!(appids, BTreeSet::from([111, 222, 333]));
    assert_eq!(d.account.considered, 3);
    assert_eq!(d.account.produced, 3);
    assert_eq!(d.account.considered_not_a_game, 0);
    assert!(d.account.is_conserved());
}

#[test]
fn a_malformed_manifest_is_counted_parse_failed_and_surfaced() {
    // One good title plus one appmanifest that is present but unparseable: the
    // library walk drops it from the title set, and SteamSource must reflect the
    // omission in the account rather than reporting a clean run (P-4).
    let tree = TempTree::new();
    let steamapps = tree.path().join("steamapps");
    std::fs::create_dir_all(&steamapps).unwrap();
    std::fs::write(
        steamapps.join("appmanifest_620.acf"),
        "\"AppState\"\n{\n  \"appid\" \"620\"\n  \"name\" \"Portal 2\"\n  \
         \"installdir\" \"Portal 2\"\n}\n",
    )
    .unwrap();
    std::fs::write(
        steamapps.join("appmanifest_999.acf"),
        "this is not valid vdf {{{ \"unterminated",
    )
    .unwrap();

    let catalog = Store::open_in_memory().unwrap();
    let source = SteamSource::new(tree.path(), &catalog);
    let d = source.discover().unwrap();

    assert_eq!(d.account.produced, 1, "the good title is produced");
    assert_eq!(
        d.account.parse_failed, 1,
        "the malformed manifest is counted, not silently dropped"
    );
    assert_eq!(d.account.considered, 2);
    assert!(d.account.is_conserved());
    assert!(
        d.warnings.iter().any(|w| w.contains("appmanifest_999")),
        "the failing manifest is named in the warnings: {:?}",
        d.warnings
    );
}
