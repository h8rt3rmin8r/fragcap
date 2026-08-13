// SPDX-License-Identifier: Apache-2.0

//! Engine rules: recognizing a game's socket-holding client from its engine's
//! documented on-disk install layout, specification section 15.7.
//!
//! A large class of games ship a thin launcher stub in the install root whose
//! only job is to relaunch the real networked client. Before the game has ever
//! run, nothing but the on-disk layout distinguishes the stub from the client,
//! and neither standard tooling nor the profile provider can name the client
//! without per-title data. But engines lay their files out in documented, stable
//! conventions, so recognizing the convention resolves the client with no
//! per-game data.
//!
//! This module is pure filesystem inspection. It reads directory entries and
//! nothing else: it opens no process handle, reads no process memory, launches
//! nothing, and reads no post-run artifact (constitution P-1). An engine rule is
//! a heuristic, so every answer it feeds the cascade is stamped
//! [`FidelityTier::HeuristicUnverified`](crate::schema::FidelityTier), never
//! higher (P-9). When it recognizes a layout but cannot single out one client, it
//! declines rather than pick one arbitrarily, and the cascade falls through to
//! runtime observation.

use std::fs;
use std::path::{Path, PathBuf};

use crate::glob::ImagePattern;
use crate::schema::MatchPredicates;
use crate::target::EngineRuleTarget;

/// The game engines this module recognizes by install layout.
///
/// All engines share one provider and one provenance source (`engine-rule`);
/// this label is for diagnostics and the ambiguity note, not the provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// Unreal Engine: a `*-Win64-Shipping.exe` under `Binaries\Win64`.
    Unreal,
    /// Unity: a `*_Data` directory and `UnityPlayer.dll` beside the player.
    Unity,
    /// Ren'Py: a `renpy` directory and `.rpa` archives.
    RenPy,
}

impl Engine {
    /// A stable lower-case label for diagnostics.
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::Unreal => "unreal",
            Engine::Unity => "unity",
            Engine::RenPy => "renpy",
        }
    }
}

/// The outcome of evaluating the engine rules against an install directory.
///
/// Internal to this module; the [`EngineRuleProvider`](crate::providers) maps it
/// onto the cascade's `Result<Option<Target>, _>` contract.
#[derive(Debug)]
pub(crate) enum EngineResolution {
    /// Exactly one rule matched with exactly one candidate client. Boxed because
    /// it is much larger than the other variants and this enum is returned by
    /// value.
    Resolved(Box<EngineRuleTarget>),
    /// No rule matched, or a matched rule's client file was absent.
    NoMatch,
    /// A rule recognized its layout but matched more than one candidate client.
    Ambiguous { engine: Engine, candidates: usize },
}

/// The maximum directory depth the layout probes descend to.
///
/// The recognized layouts are shallow (an Unreal `Binaries\Win64` sits a level or
/// two below the install root; Unity and Ren'Py markers sit at or just under the
/// root), so a small bound keeps the scan cheap and avoids false matches from
/// tools buried deep in a large install tree.
const MAX_SCAN_DEPTH: usize = 6;

/// Evaluate the engine rules against an install directory, returning the first
/// engine whose layout is present.
///
/// The engines are tried in a fixed, total order (Unreal, Unity, Ren'Py), so the
/// result never depends on filesystem iteration order (FR-006). A rule that
/// recognizes its layout answers (with a resolution or an ambiguity); only a rule
/// that does not recognize its layout falls through to the next.
pub(crate) fn resolve_engine(install_root: &Path) -> EngineResolution {
    for recognizer in [resolve_unreal, resolve_unity, resolve_renpy] {
        match recognizer(install_root) {
            EngineResolution::NoMatch => continue,
            resolved_or_ambiguous => return resolved_or_ambiguous,
        }
    }
    EngineResolution::NoMatch
}

/// Unreal: a `*-Win64-Shipping.exe` file under a directory whose trailing
/// components are `Binaries\Win64`.
fn resolve_unreal(install_root: &Path) -> EngineResolution {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for dir in dirs_within(install_root, MAX_SCAN_DEPTH) {
        if !ends_with_components(&dir, &["binaries", "win64"]) {
            continue;
        }
        for file in files_in(&dir) {
            if file_name_lower(&file).is_some_and(|n| n.ends_with("-win64-shipping.exe")) {
                candidates.push(file);
            }
        }
    }
    decide(Engine::Unreal, candidates, "Binaries\\Win64")
}

/// Unity: a `*_Data` directory and a `UnityPlayer.dll` in the install root, with
/// the player executable named after the `*_Data` stem.
fn resolve_unity(install_root: &Path) -> EngineResolution {
    let root_files = files_in(install_root);
    let has_unity_player = root_files
        .iter()
        .any(|f| file_name_lower(f).is_some_and(|n| n == "unityplayer.dll"));
    if !has_unity_player {
        return EngineResolution::NoMatch;
    }
    let mut candidates: Vec<PathBuf> = Vec::new();
    for dir in dirs_in(install_root) {
        let Some(name) = dir_name_lower(&dir) else {
            continue;
        };
        let Some(stem) = name.strip_suffix("_data") else {
            continue;
        };
        // The player executable matching the data-directory stem, in the root.
        let player = format!("{stem}.exe");
        if let Some(exe) = root_files
            .iter()
            .find(|f| file_name_lower(f).is_some_and(|n| n == player))
        {
            candidates.push(exe.clone());
        }
    }
    decide(Engine::Unity, candidates, "")
}

/// Ren'Py: a `renpy` directory in the install root and at least one `.rpa`
/// archive, with a launcher executable in the root.
fn resolve_renpy(install_root: &Path) -> EngineResolution {
    let has_renpy_dir = dirs_in(install_root)
        .iter()
        .any(|d| dir_name_lower(d).is_some_and(|n| n == "renpy"));
    if !has_renpy_dir {
        return EngineResolution::NoMatch;
    }
    let has_rpa = files_within(install_root, MAX_SCAN_DEPTH)
        .iter()
        .any(|f| file_name_lower(f).is_some_and(|n| n.ends_with(".rpa")));
    if !has_rpa {
        return EngineResolution::NoMatch;
    }
    // The launcher executables in the root. Ren'Py commonly ships more than one
    // (for example a 32-bit sibling); the rule cannot tell pre-launch which holds
    // sockets, so more than one is an honest ambiguity the cascade resolves at
    // runtime rather than a tie the rule breaks arbitrarily.
    let candidates: Vec<PathBuf> = files_in(install_root)
        .into_iter()
        .filter(|f| file_name_lower(f).is_some_and(|n| n.ends_with(".exe")))
        .collect();
    decide(Engine::RenPy, candidates, "")
}

/// Turn a candidate set into an [`EngineResolution`]: zero is no match, one
/// resolves, more than one is an honest ambiguity.
fn decide(engine: Engine, mut candidates: Vec<PathBuf>, path_contains: &str) -> EngineResolution {
    match candidates.len() {
        0 => EngineResolution::NoMatch,
        1 => {
            let path = candidates.remove(0);
            match build_target(engine, &path, path_contains) {
                Some(target) => EngineResolution::Resolved(Box::new(target)),
                // A candidate whose name is not a usable match pattern is treated
                // as no match rather than a fabricated target.
                None => EngineResolution::NoMatch,
            }
        }
        candidates_len => EngineResolution::Ambiguous {
            engine,
            candidates: candidates_len,
        },
    }
}

/// Build the engine-rule target for a resolved client path, or `None` if the file
/// name is not a usable match pattern.
fn build_target(engine: Engine, path: &Path, path_contains: &str) -> Option<EngineRuleTarget> {
    let image_name = path.file_name()?.to_string_lossy().into_owned();
    let image_path = path.to_string_lossy().into_owned();
    let mut identity = MatchPredicates::default();
    identity.set_exe(ImagePattern::new(&image_name).ok()?);
    if !path_contains.is_empty() {
        identity.set_path_contains(path_contains.to_string());
    }
    Some(EngineRuleTarget::new(
        engine, image_name, image_path, identity,
    ))
}

// Filesystem helpers. All read directory entries only; none opens a process or
// reads process memory.

/// The immediate file entries of a directory (non-recursive), empty on error.
fn files_in(dir: &Path) -> Vec<PathBuf> {
    read_entries(dir, |ft| ft.is_file())
}

/// The immediate subdirectories of a directory (non-recursive), empty on error.
fn dirs_in(dir: &Path) -> Vec<PathBuf> {
    read_entries(dir, |ft| ft.is_dir())
}

/// Immediate entries of a directory whose file type satisfies `keep`.
fn read_entries(dir: &Path, keep: impl Fn(&fs::FileType) -> bool) -> Vec<PathBuf> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in read.flatten() {
        if let Ok(ft) = entry.file_type() {
            if keep(&ft) {
                out.push(entry.path());
            }
        }
    }
    out
}

/// All directories under `root` within `max_depth` levels, including `root`.
fn dirs_within(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    collect_dirs(root, max_depth, &mut out);
    out
}

fn collect_dirs(dir: &Path, remaining_depth: usize, out: &mut Vec<PathBuf>) {
    if remaining_depth == 0 {
        return;
    }
    for sub in dirs_in(dir) {
        collect_dirs(&sub, remaining_depth - 1, out);
        out.push(sub);
    }
}

/// All files under `root` within `max_depth` levels.
fn files_within(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut out = files_in(root);
    for dir in dirs_within(root, max_depth) {
        if dir != root {
            out.extend(files_in(&dir));
        }
    }
    out
}

/// The lower-cased final file name of a path, if it has one.
fn file_name_lower(path: &Path) -> Option<String> {
    path.file_name().map(|n| n.to_string_lossy().to_lowercase())
}

/// The lower-cased final directory name of a path, if it has one.
fn dir_name_lower(path: &Path) -> Option<String> {
    path.file_name().map(|n| n.to_string_lossy().to_lowercase())
}

/// Whether a path's trailing components equal the given lower-case component
/// names, compared case-insensitively.
///
/// Component-based so a directory literally named `Win64Extra` does not match
/// `win64`, and separator-agnostic because it compares [`Path`] components rather
/// than raw strings.
fn ends_with_components(path: &Path, tail: &[&str]) -> bool {
    let components: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(os) => Some(os.to_string_lossy().to_lowercase()),
            _ => None,
        })
        .collect();
    if components.len() < tail.len() {
        return false;
    }
    let start = components.len() - tail.len();
    components[start..]
        .iter()
        .zip(tail.iter())
        .all(|(have, want)| have == want)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    use crate::schema::FidelityTier;

    /// A throwaway directory tree under the system temp dir, removed on drop. In
    /// the spirit of `fragcap-steam`'s `TempTree`; this crate has no shared one.
    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new(tag: &str) -> TempTree {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "fragcap-engine-rule-{}-{}-{}",
                std::process::id(),
                tag,
                n
            ));
            fs::create_dir_all(&root).expect("create temp root");
            TempTree { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        /// Create an empty placeholder file at a path relative to the root,
        /// creating parent directories.
        fn touch(&self, rel: &str) {
            let full = self.root.join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("create parents");
            }
            fs::write(&full, b"").expect("write file");
        }

        /// Create a directory at a path relative to the root.
        fn mkdir(&self, rel: &str) {
            fs::create_dir_all(self.root.join(rel)).expect("create dir");
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn resolved(res: EngineResolution) -> EngineRuleTarget {
        match res {
            EngineResolution::Resolved(t) => *t,
            other => panic!("expected Resolved, got {other:?}"),
        }
    }

    // A sample of at least three distinct Unreal layouts, differing game names
    // (SC-001). Each ships a root stub and a shipping client under Binaries\Win64.
    fn unreal_tree(game: &str) -> TempTree {
        let tree = TempTree::new(game);
        tree.touch(&format!("{game}.exe")); // the launcher stub in the root
        tree.touch(&format!("{game}/Binaries/Win64/{game}-Win64-Shipping.exe"));
        tree
    }

    #[test]
    fn unreal_resolves_the_shipping_client_for_several_titles() {
        for game in ["Atlas", "Borderlands", "Crysis"] {
            let tree = unreal_tree(game);
            let target = resolved(resolve_engine(tree.path()));
            assert_eq!(target.engine(), Engine::Unreal);
            assert_eq!(target.image_name(), format!("{game}-Win64-Shipping.exe"));
            assert!(target.image_path().to_lowercase().contains("binaries"));
            // The identity carries the exe and the Binaries\Win64 path anchor, so
            // the pipeline can bind the process once it appears.
            assert!(target.identity().exe().is_some());
            assert_eq!(target.identity().path_contains(), Some("Binaries\\Win64"));
        }
    }

    #[test]
    fn unreal_matches_binaries_win64_case_insensitively() {
        let tree = TempTree::new("casey");
        tree.touch("Casey/BINARIES/win64/Casey-Win64-Shipping.exe");
        let target = resolved(resolve_engine(tree.path()));
        assert_eq!(target.image_name(), "Casey-Win64-Shipping.exe");
    }

    #[test]
    fn a_tree_with_no_binaries_win64_does_not_match() {
        let tree = TempTree::new("nomatch");
        tree.touch("Game.exe");
        tree.touch("Game/Content/paks/data.pak");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
    }

    #[test]
    fn a_binaries_win64_with_no_shipping_exe_does_not_fabricate_a_target() {
        let tree = TempTree::new("empty64");
        tree.mkdir("Game/Binaries/Win64");
        tree.touch("Game/Binaries/Win64/Game.exe"); // not a *-Win64-Shipping.exe
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
    }

    #[test]
    fn a_directory_named_like_win64_but_not_a_component_does_not_match() {
        let tree = TempTree::new("win64extra");
        // A single directory named "Win64Extra" must not satisfy the win64
        // component, and "BinariesX/Win64" must not satisfy the binaries one.
        tree.touch("Game/Win64Extra/Game-Win64-Shipping.exe");
        tree.touch("Game/BinariesX/Win64/Game-Win64-Shipping.exe");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
    }

    #[test]
    fn two_shipping_exes_are_an_ambiguity_not_a_pick() {
        let tree = TempTree::new("ambiguous");
        tree.touch("A/Binaries/Win64/A-Win64-Shipping.exe");
        tree.touch("B/Binaries/Win64/B-Win64-Shipping.exe");
        match resolve_engine(tree.path()) {
            EngineResolution::Ambiguous { engine, candidates } => {
                assert_eq!(engine, Engine::Unreal);
                assert_eq!(candidates, 2);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn resolution_is_the_same_across_repeated_runs() {
        let tree = unreal_tree("Deterministic");
        let first = resolved(resolve_engine(tree.path()));
        // Add unrelated sibling files in a different order between runs; the
        // result must not depend on directory iteration order (FR-006, SC-003).
        tree.touch("Deterministic/Binaries/Win64/zzz.txt");
        tree.touch("Deterministic/Binaries/Win64/aaa.txt");
        let second = resolved(resolve_engine(tree.path()));
        assert_eq!(first.image_path(), second.image_path());
    }

    #[test]
    fn unity_resolves_the_player_beside_its_data_directory() {
        let tree = TempTree::new("unity");
        tree.mkdir("MyUnityGame_Data");
        tree.touch("UnityPlayer.dll");
        tree.touch("MyUnityGame.exe");
        let target = resolved(resolve_engine(tree.path()));
        assert_eq!(target.engine(), Engine::Unity);
        assert_eq!(target.image_name(), "MyUnityGame.exe");
        assert!(target.identity().exe().is_some());
    }

    #[test]
    fn unity_without_a_player_dll_does_not_match() {
        let tree = TempTree::new("unity-nodll");
        tree.mkdir("MyUnityGame_Data");
        tree.touch("MyUnityGame.exe");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
    }

    #[test]
    fn renpy_resolves_the_launcher() {
        let tree = TempTree::new("renpy");
        tree.mkdir("renpy");
        tree.touch("game/archive.rpa");
        tree.touch("MyVisualNovel.exe");
        let target = resolved(resolve_engine(tree.path()));
        assert_eq!(target.engine(), Engine::RenPy);
        assert_eq!(target.image_name(), "MyVisualNovel.exe");
    }

    #[test]
    fn renpy_with_two_launchers_declines_as_ambiguous() {
        let tree = TempTree::new("renpy-dual");
        tree.mkdir("renpy");
        tree.touch("game/archive.rpa");
        tree.touch("MyVisualNovel.exe");
        tree.touch("MyVisualNovel-32.exe");
        match resolve_engine(tree.path()) {
            EngineResolution::Ambiguous { engine, candidates } => {
                assert_eq!(engine, Engine::RenPy);
                assert_eq!(candidates, 2);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn engine_order_is_fixed_when_more_than_one_layout_is_present() {
        // A tree carrying both an Unreal and a Unity signature resolves as Unreal
        // every time, because the engine order is declared, not incidental.
        let tree = TempTree::new("multi");
        tree.touch("Game/Binaries/Win64/Game-Win64-Shipping.exe");
        tree.mkdir("Game_Data");
        tree.touch("UnityPlayer.dll");
        tree.touch("Game.exe");
        for _ in 0..3 {
            let target = resolved(resolve_engine(tree.path()));
            assert_eq!(target.engine(), Engine::Unreal);
        }
    }

    #[test]
    fn an_empty_or_absent_directory_does_not_match() {
        let tree = TempTree::new("empty");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
        let absent = tree.path().join("does-not-exist");
        assert!(matches!(resolve_engine(&absent), EngineResolution::NoMatch));
    }

    #[test]
    fn the_engine_labels_are_stable() {
        assert_eq!(Engine::Unreal.as_str(), "unreal");
        assert_eq!(Engine::Unity.as_str(), "unity");
        assert_eq!(Engine::RenPy.as_str(), "renpy");
    }

    #[test]
    fn a_resolved_unreal_target_is_stamped_by_the_provider_layer_only() {
        // The module itself carries no fidelity; the provider stamps it. This test
        // documents that the resolved target names a real file that exists.
        let tree = unreal_tree("Exists");
        let target = resolved(resolve_engine(tree.path()));
        assert!(Path::new(target.image_path()).is_file());
        // Fidelity is applied at the provider; assert the tier exists as expected
        // there via the provider tests. Here we only touch the enum to keep the
        // separation explicit.
        let _ = FidelityTier::HeuristicUnverified;
    }
}
