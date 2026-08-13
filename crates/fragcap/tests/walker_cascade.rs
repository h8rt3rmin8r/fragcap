// SPDX-License-Identifier: Apache-2.0

//! End-to-end tests for the Steam platform-walker (S030) composing with the
//! engine rule (S029) and degrading to runtime observation (S027/S028), through
//! the target-resolution cascade.
//!
//! This test lives in the facade because it is the only crate that legitimately
//! depends on the walker (`fragcap-steam`), the engine rule and resolver
//! (`fragcap-profile`), and the process tree observation reads (`fragcap-core`).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap::core::{ProcessEvent, ProcessTree, Timestamp};
use fragcap::profile::{
    BundledSet, EngineRuleProvider, MatchPredicates, ObservationProvider, Profile, ProfileProvider,
    ResolutionError, ResolutionRequest, SearchPath, TargetOrigin, TargetProvider, TargetResolver,
};
use fragcap::steam::{install_root_in, SteamWalkerProvider};

/// A throwaway directory tree under the system temp dir, removed on drop.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(tag: &str) -> TempTree {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "fragcap-walker-cascade-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        std::fs::create_dir_all(&root).expect("create temp root");
        TempTree { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, rel: &str, contents: &str) {
        let full = self.root.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("create parents");
        }
        std::fs::write(&full, contents).expect("write file");
    }

    fn touch_exe(&self, rel: &str) {
        let full = self.root.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("create parents");
        }
        std::fs::write(&full, b"MZ").expect("write exe");
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Build a fake Steam library at `tree` root: one library (the root), one title
/// with the given app id, name, and install directory under
/// `steamapps/common/<install_dir>`.
fn fake_steam_library(tree: &TempTree, app_id: &str, name: &str, install_dir: &str) {
    let escaped = tree.path().to_string_lossy().replace('\\', "\\\\");
    tree.write(
        "steamapps/libraryfolders.vdf",
        &format!("\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\"{escaped}\"\n\t}}\n}}\n"),
    );
    tree.write(
        &format!("steamapps/appmanifest_{app_id}.acf"),
        &format!(
            "\"AppState\"\n{{\n\t\"appid\"\t\"{app_id}\"\n\t\"name\"\t\"{name}\"\n\t\"installdir\"\t\"{install_dir}\"\n}}\n"
        ),
    );
}

fn common_dir(install_dir: &str) -> String {
    format!("steamapps/common/{install_dir}")
}

fn identity(exe: &str) -> MatchPredicates {
    MatchPredicates::with_exe(exe).expect("valid exe pattern")
}

fn tree_with(process: &str, image_path: &str) -> ProcessTree {
    let mut tree = ProcessTree::new();
    tree.apply(ProcessEvent::started(
        42,
        0,
        image_path,
        process,
        Timestamp::from_nanos(1),
    ));
    tree
}

fn profile_for(app_id: &str, exe: &str) -> Profile {
    let text = format!(
        r#"{{"schema":1,"kind":"profile","fidelity":"verified","game":{{"id":"steam-{app_id}","name":"T","platform":"steam","app_id":"{app_id}"}},"stage":[{{"role":"client","lifecycle":"session","terminal":true,"match":{{"exe":"{exe}"}}}}]}}"#
    );
    Profile::parse(&text).expect("valid profile")
}

#[test]
fn a_steam_installed_unreal_title_resolves_via_the_engine_rule() {
    // The walker supplies the install dir; the higher-precedence engine rule hops
    // to the shipping executable (SC-001).
    let tree = TempTree::new("unreal");
    fake_steam_library(&tree, "220", "Half-Life 3", "HL3");
    let common = common_dir("HL3");
    tree.touch_exe(&format!("{common}/HL3.exe")); // stub in the install root
    tree.touch_exe(&format!(
        "{common}/HL3/Binaries/Win64/HL3-Win64-Shipping.exe"
    ));

    let install_root = install_root_in(tree.path(), "220")
        .expect("no error")
        .expect("installed");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let req = ResolutionRequest::for_install(&install_root, &search, &bundled);

    let resolver = TargetResolver::new(vec![
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
    ])
    .expect("distinct precedences");
    let target = resolver.resolve(&req).expect("resolves");
    match target.origin() {
        TargetOrigin::EngineRule(t) => assert_eq!(t.image_name(), "HL3-Win64-Shipping.exe"),
        _ => panic!("expected the engine rule to win over the walker"),
    }
}

#[test]
fn a_steam_installed_non_engine_title_resolves_via_the_walker() {
    // The engine rule does not recognize the layout, so the walker answers from
    // its install-directory classification (SC-002).
    let tree = TempTree::new("nonengine");
    fake_steam_library(&tree, "400", "Portal", "Portal");
    let common = common_dir("Portal");
    tree.touch_exe(&format!("{common}/Portal.exe"));
    tree.touch_exe(&format!("{common}/vc_redist.x64.exe")); // dropped as non-game

    let install_root = install_root_in(tree.path(), "400")
        .expect("no error")
        .expect("installed");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let req = ResolutionRequest::for_install(&install_root, &search, &bundled);

    let resolver = TargetResolver::new(vec![
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
    ])
    .expect("distinct precedences");
    let target = resolver.resolve(&req).expect("resolves");
    assert_eq!(target.provenance().source(), "steam-library");
    match target.origin() {
        TargetOrigin::PlatformWalker(t) => {
            assert_eq!(t.image_name(), "Portal.exe");
            assert_eq!(t.platform(), "steam");
        }
        _ => panic!("expected the walker to answer"),
    }
}

#[test]
fn an_authored_profile_outranks_the_engine_rule_and_the_walker() {
    // A request carries a profile reference and the install dir. The profile wins,
    // regardless of provider registration order (SC-004).
    let tree = TempTree::new("profile");
    fake_steam_library(&tree, "220", "Half-Life 3", "HL3");
    let common = common_dir("HL3");
    tree.touch_exe(&format!("{common}/HL3.exe"));
    tree.touch_exe(&format!(
        "{common}/HL3/Binaries/Win64/HL3-Win64-Shipping.exe"
    ));

    let install_root = install_root_in(tree.path(), "220")
        .expect("no error")
        .expect("installed");
    let search = SearchPath::new();
    let bundled =
        BundledSet::new(vec![profile_for("220", "HL3-Win64-Shipping.exe")]).expect("one profile");
    let req = ResolutionRequest::for_reference("steam-220", &search, &bundled)
        .with_install_root(&install_root);

    for order in 0..2 {
        let providers: Vec<Box<dyn TargetProvider>> = if order == 0 {
            vec![
                Box::new(ProfileProvider::new()),
                Box::new(EngineRuleProvider::new()),
                Box::new(SteamWalkerProvider::new()),
            ]
        } else {
            vec![
                Box::new(SteamWalkerProvider::new()),
                Box::new(EngineRuleProvider::new()),
                Box::new(ProfileProvider::new()),
            ]
        };
        let resolver = TargetResolver::new(providers).expect("distinct precedences");
        let target = resolver.resolve(&req).expect("resolves");
        assert!(
            target.profile().is_some(),
            "the profile outranks the engine rule and the walker"
        );
    }
}

#[test]
fn a_not_installed_title_degrades_to_runtime_observation() {
    // The title is absent from the library, so no install_root is offered; the
    // walker and engine rule decline and observation resolves (SC-003).
    let tree = TempTree::new("notinstalled");
    fake_steam_library(&tree, "220", "Half-Life 3", "HL3");
    assert!(
        install_root_in(tree.path(), "999")
            .expect("no error")
            .is_none(),
        "the requested title is not installed"
    );

    let id = identity("game.exe");
    let ptree = tree_with("game.exe", "C:\\Games\\game.exe");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    // Not installed: the request carries the observation inputs and no install_root.
    let req = ResolutionRequest::for_observation(&id, &ptree, &search, &bundled);

    let resolver = TargetResolver::new(vec![
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
        Box::new(ObservationProvider::new()),
    ])
    .expect("distinct precedences");
    let target = resolver.resolve(&req).expect("resolves via observation");
    assert!(matches!(target.origin(), TargetOrigin::Observed(_)));
}

#[test]
fn an_ambiguous_install_degrades_to_runtime_observation() {
    // The install dir has several plausible clients; the walker declines and
    // observation resolves (SC-003). Without observation, the ambiguity surfaces.
    let tree = TempTree::new("ambiguous");
    fake_steam_library(&tree, "500", "Division", "Division");
    let common = common_dir("Division");
    tree.touch_exe(&format!("{common}/ClientA.exe"));
    tree.touch_exe(&format!("{common}/ClientB.exe"));

    let install_root = install_root_in(tree.path(), "500")
        .expect("no error")
        .expect("installed");
    let id = identity("ClientA.exe");
    let ptree = tree_with("ClientA.exe", "C:\\Games\\Division\\ClientA.exe");
    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let req = ResolutionRequest::for_observation(&id, &ptree, &search, &bundled)
        .with_install_root(&install_root);

    let resolver = TargetResolver::new(vec![
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
        Box::new(ObservationProvider::new()),
    ])
    .expect("distinct precedences");
    let target = resolver.resolve(&req).expect("resolves via observation");
    assert!(matches!(target.origin(), TargetOrigin::Observed(_)));

    // With no observation provider, the walker's ambiguity is the surfaced reason.
    let no_obs = ResolutionRequest::for_install(&install_root, &search, &bundled);
    let resolver = TargetResolver::new(vec![
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
    ])
    .expect("distinct precedences");
    match resolver.resolve(&no_obs) {
        Err(ResolutionError::Unresolved(u)) => {
            assert_eq!(u.walker_ambiguous().expect("recorded").candidates(), 2);
        }
        _ => panic!("expected Unresolved with the walker ambiguity surfaced"),
    }
}
