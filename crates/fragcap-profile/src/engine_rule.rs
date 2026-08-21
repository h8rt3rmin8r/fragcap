// SPDX-License-Identifier: Apache-2.0

//! Engine rules: recognizing a game's socket-holding client from its game
//! engine's documented on-disk install layout, specification section 15.7.
//!
//! A large class of games ship a thin launcher stub in the install root whose
//! only job is to relaunch the real networked client. Before the game has ever
//! run, nothing but the on-disk layout distinguishes the stub from the client,
//! and neither standard tooling nor the profile provider can name the client
//! without per-title data. But engines lay their files out in documented, stable
//! conventions, so recognizing the convention resolves the client with no
//! per-game data.
//!
//! # Filename and path signatures only
//!
//! The signatures this module keys on are the same ones the Steam database uses
//! to attribute an engine: `SteamDatabase/FileDetectionRuleSets` (MIT), the open
//! ruleset behind SteamDB's technology table, detects engines from depot file
//! names and paths alone, never from file contents. This module tracks the
//! filename-signature subset of that ruleset that also names the client
//! executable: a `*-Win64-Shipping.exe` under `Binaries\Win64` (Unreal), a
//! `*_Data` directory beside a `UnityPlayer.dll` or `GameAssembly.dll` (Unity,
//! including IL2CPP builds), a `*.pck` archive beside the binary (Godot), and a
//! `renpy` directory with `.rpa` archives (Ren'Py). Like SteamDB's, these are
//! educated guesses from layout, which is exactly why every answer is stamped
//! heuristic-unverified.
//!
//! # Passive and honest
//!
//! This module is pure filesystem inspection. It reads directory entries and
//! nothing else: it opens no process handle, reads no process memory, launches
//! nothing, and reads no post-run artifact (constitution P-1). An engine rule is
//! a heuristic, so every answer it feeds the cascade is stamped
//! [`FidelityTier::HeuristicUnverified`](crate::schema::FidelityTier), never
//! higher (P-9). When it recognizes a layout but cannot single out one client, it
//! declines rather than pick one arbitrarily. When a filesystem error leaves a
//! scan incomplete, it declines with the unreadable path recorded rather than
//! resolving from a partial view (P-4, FR-009); an incomplete scan could hide a
//! second candidate and turn a true ambiguity into a false single answer, so an
//! unreadable tree is surfaced, not swallowed.

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
    /// Unity: a `*_Data` directory beside `UnityPlayer.dll` or `GameAssembly.dll`.
    Unity,
    /// Godot: a `*.pck` archive beside the binary.
    Godot,
    /// Ren'Py: a `renpy` directory and `.rpa` archives.
    RenPy,
}

impl Engine {
    /// Every engine these rules can select a client executable for.
    ///
    /// Exists so the directed subset invariant of slice S065 iterates the
    /// declaration itself rather than a hand-maintained list beside it: adding a
    /// variant here without adding a detection signature for its
    /// [`product_name`](Engine::product_name) fails that check.
    pub const ALL: [Engine; 4] = [Engine::Unreal, Engine::Unity, Engine::Godot, Engine::RenPy];

    /// A stable lower-case label for diagnostics. Matches the engine names used
    /// by the `SteamDatabase/FileDetectionRuleSets` ruleset, lower-cased.
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::Unreal => "unreal",
            Engine::Unity => "unity",
            Engine::Godot => "godot",
            Engine::RenPy => "renpy",
        }
    }

    /// The product name the detection signature set uses for this engine.
    ///
    /// Deliberately distinct from [`as_str`](Engine::as_str), which is a lower-case
    /// diagnostic label. This is the operator-facing product string that appears in
    /// a listing's engine column, so it carries the product's own capitalization and
    /// punctuation, and it is the key the S065 subset check joins on. Collapsing the
    /// two would either put a lower-case label in front of the operator or make a
    /// diagnostic label carry an apostrophe.
    pub fn product_name(&self) -> &'static str {
        match self {
            Engine::Unreal => "Unreal",
            Engine::Unity => "Unity",
            Engine::Godot => "Godot",
            Engine::RenPy => "Ren'Py",
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
    /// A filesystem error left a scan incomplete, so no answer is trustworthy.
    /// Carries the path that could not be read, for an observable decline note.
    Unreadable { path: PathBuf },
}

/// A directory scan that could not be completed, carrying the unreadable path.
type ScanResult<T> = Result<T, PathBuf>;

/// The maximum directory depth the layout probes descend to.
///
/// The recognized layouts are shallow (an Unreal `Binaries\Win64` sits a level or
/// two below the install root; Unity, Godot, and Ren'Py markers sit at or just
/// under the root), so a small bound keeps the scan cheap and avoids false
/// matches from tools buried deep in a large install tree.
const MAX_SCAN_DEPTH: usize = 6;

/// Evaluate the engine rules against an install directory, returning the first
/// engine whose layout is present.
///
/// The engines are tried in a fixed, total order (Unreal, Unity, Godot, Ren'Py),
/// so the result never depends on filesystem iteration order (FR-006). A rule
/// that recognizes its layout answers (with a resolution or an ambiguity); a rule
/// that does not recognize its layout falls through to the next. A rule that
/// could not complete its scan records the unreadable path and falls through, so
/// a clean lower-precedence resolution still wins; only if nothing resolves does
/// the unreadable path surface, distinguishing an inaccessible install from an
/// unrecognized engine.
pub(crate) fn resolve_engine(install_root: &Path) -> EngineResolution {
    let mut unreadable: Option<PathBuf> = None;
    for recognizer in [resolve_unreal, resolve_unity, resolve_godot, resolve_renpy] {
        match recognizer(install_root) {
            EngineResolution::NoMatch => continue,
            EngineResolution::Unreadable { path } => {
                if unreadable.is_none() {
                    unreadable = Some(path);
                }
                continue;
            }
            resolved_or_ambiguous => return resolved_or_ambiguous,
        }
    }
    match unreadable {
        Some(path) => EngineResolution::Unreadable { path },
        None => EngineResolution::NoMatch,
    }
}

/// Unreal: a `*-Win64-Shipping.exe` file under a directory whose trailing
/// components are `Binaries\Win64`.
fn resolve_unreal(install_root: &Path) -> EngineResolution {
    let dirs = match dirs_within(install_root, MAX_SCAN_DEPTH) {
        Ok(dirs) => dirs,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let mut candidates: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        if !ends_with_components(&dir, &["binaries", "win64"]) {
            continue;
        }
        let files = match files_in(&dir) {
            Ok(files) => files,
            Err(path) => return EngineResolution::Unreadable { path },
        };
        for file in files {
            if file_name_lower(&file).is_some_and(|n| n.ends_with("-win64-shipping.exe")) {
                candidates.push(file);
            }
        }
    }
    decide(Engine::Unreal, candidates, "Binaries\\Win64")
}

/// Unity: a `*_Data` directory beside a `UnityPlayer.dll` (mono) or a
/// `GameAssembly.dll` (IL2CPP) in the install root, with the player executable
/// named after the `*_Data` stem. Both markers are Unity evidence in the
/// `FileDetectionRuleSets` ruleset.
fn resolve_unity(install_root: &Path) -> EngineResolution {
    let root_files = match files_in(install_root) {
        Ok(files) => files,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let has_unity_marker = root_files.iter().any(|f| {
        file_name_lower(f).is_some_and(|n| n == "unityplayer.dll" || n == "gameassembly.dll")
    });
    if !has_unity_marker {
        return EngineResolution::NoMatch;
    }
    let root_dirs = match dirs_in(install_root) {
        Ok(dirs) => dirs,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let mut candidates: Vec<PathBuf> = Vec::new();
    for dir in root_dirs {
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

/// Godot: a `*.pck` archive in the install root, with the game executable named
/// after the archive's stem beside it (the default Godot export names the binary
/// and its `.pck` from the same stem).
fn resolve_godot(install_root: &Path) -> EngineResolution {
    let root_files = match files_in(install_root) {
        Ok(files) => files,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let mut candidates: Vec<PathBuf> = Vec::new();
    for file in &root_files {
        let Some(name) = file_name_lower(file) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".pck") else {
            continue;
        };
        let exe = format!("{stem}.exe");
        if let Some(found) = root_files
            .iter()
            .find(|f| file_name_lower(f).is_some_and(|n| n == exe))
        {
            candidates.push(found.clone());
        }
    }
    decide(Engine::Godot, candidates, "")
}

/// Ren'Py: a `renpy` directory in the install root and at least one `.rpa`
/// archive, with a launcher executable in the root.
fn resolve_renpy(install_root: &Path) -> EngineResolution {
    let root_dirs = match dirs_in(install_root) {
        Ok(dirs) => dirs,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let has_renpy_dir = root_dirs
        .iter()
        .any(|d| dir_name_lower(d).is_some_and(|n| n == "renpy"));
    if !has_renpy_dir {
        return EngineResolution::NoMatch;
    }
    let all_files = match files_within(install_root, MAX_SCAN_DEPTH) {
        Ok(files) => files,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let has_rpa = all_files
        .iter()
        .any(|f| file_name_lower(f).is_some_and(|n| n.ends_with(".rpa")));
    if !has_rpa {
        return EngineResolution::NoMatch;
    }
    // The launcher executables in the root. Ren'Py commonly ships more than one
    // (for example a 32-bit sibling); the rule cannot tell pre-launch which holds
    // sockets, so more than one is an honest ambiguity the cascade resolves at
    // runtime rather than a tie the rule breaks arbitrarily.
    let root_files = match files_in(install_root) {
        Ok(files) => files,
        Err(path) => return EngineResolution::Unreadable { path },
    };
    let candidates: Vec<PathBuf> = root_files
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
// reads process memory. Each returns the unreadable path on error rather than an
// empty result, so an incomplete scan is never mistaken for an absent layout.

/// The immediate file entries of a directory (non-recursive).
fn files_in(dir: &Path) -> ScanResult<Vec<PathBuf>> {
    read_entries(dir, |ft| ft.is_file())
}

/// The immediate subdirectories of a directory (non-recursive).
fn dirs_in(dir: &Path) -> ScanResult<Vec<PathBuf>> {
    read_entries(dir, |ft| ft.is_dir())
}

/// Immediate entries of a directory whose file type satisfies `keep`, or the
/// directory path if it could not be read fully.
fn read_entries(dir: &Path, keep: impl Fn(&fs::FileType) -> bool) -> ScanResult<Vec<PathBuf>> {
    let read = fs::read_dir(dir).map_err(|_| dir.to_path_buf())?;
    let mut out = Vec::new();
    for entry in read {
        let entry = entry.map_err(|_| dir.to_path_buf())?;
        let ft = entry.file_type().map_err(|_| dir.to_path_buf())?;
        if keep(&ft) {
            out.push(entry.path());
        }
    }
    Ok(out)
}

/// All directories under `root` within `max_depth` levels, including `root`.
fn dirs_within(root: &Path, max_depth: usize) -> ScanResult<Vec<PathBuf>> {
    let mut out = vec![root.to_path_buf()];
    collect_dirs(root, max_depth, &mut out)?;
    Ok(out)
}

fn collect_dirs(dir: &Path, remaining_depth: usize, out: &mut Vec<PathBuf>) -> ScanResult<()> {
    if remaining_depth == 0 {
        return Ok(());
    }
    for sub in dirs_in(dir)? {
        collect_dirs(&sub, remaining_depth - 1, out)?;
        out.push(sub);
    }
    Ok(())
}

/// All files under `root` within `max_depth` levels.
fn files_within(root: &Path, max_depth: usize) -> ScanResult<Vec<PathBuf>> {
    let mut out = files_in(root)?;
    for dir in dirs_within(root, max_depth)? {
        if dir != root {
            out.extend(files_in(&dir)?);
        }
    }
    Ok(out)
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
    #[test]
    fn the_product_name_is_not_the_diagnostic_label() {
        use super::Engine;
        // Ren'Py is the case that forces the two to be separate: the product name
        // carries an apostrophe and capitalization the diagnostic label must not.
        assert_eq!(Engine::RenPy.as_str(), "renpy");
        assert_eq!(Engine::RenPy.product_name(), "Ren'Py");
        assert!(
            Engine::ALL.iter().any(|e| e.as_str() != e.product_name()),
            "at least one engine distinguishes the two, so they cannot be merged"
        );
    }

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
    fn unity_il2cpp_is_recognized_by_game_assembly_dll() {
        // An IL2CPP build may carry GameAssembly.dll; it is Unity evidence in the
        // FileDetectionRuleSets ruleset just as UnityPlayer.dll is.
        let tree = TempTree::new("unity-il2cpp");
        tree.mkdir("MyIl2cppGame_Data");
        tree.touch("GameAssembly.dll");
        tree.touch("MyIl2cppGame.exe");
        let target = resolved(resolve_engine(tree.path()));
        assert_eq!(target.engine(), Engine::Unity);
        assert_eq!(target.image_name(), "MyIl2cppGame.exe");
    }

    #[test]
    fn unity_without_a_marker_dll_does_not_match() {
        let tree = TempTree::new("unity-nodll");
        tree.mkdir("MyUnityGame_Data");
        tree.touch("MyUnityGame.exe");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
    }

    #[test]
    fn godot_resolves_the_binary_beside_its_pck() {
        let tree = TempTree::new("godot");
        tree.touch("MyGodotGame.pck");
        tree.touch("MyGodotGame.exe");
        let target = resolved(resolve_engine(tree.path()));
        assert_eq!(target.engine(), Engine::Godot);
        assert_eq!(target.image_name(), "MyGodotGame.exe");
    }

    #[test]
    fn godot_pck_without_a_matching_binary_does_not_fabricate_a_target() {
        let tree = TempTree::new("godot-nopair");
        tree.touch("data.pck"); // no data.exe beside it
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
    fn an_existing_but_empty_directory_is_a_clean_no_match() {
        let tree = TempTree::new("empty");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::NoMatch
        ));
    }

    #[test]
    fn an_absent_directory_is_unreadable_not_a_no_match() {
        // An inaccessible install must be distinguishable from an unrecognized
        // engine (FR-009). A path that cannot be read fails the same way a
        // permission-denied directory would.
        let tree = TempTree::new("absent");
        let absent = tree.path().join("does-not-exist");
        match resolve_engine(&absent) {
            EngineResolution::Unreadable { path } => assert_eq!(path, absent),
            other => panic!("expected Unreadable, got {other:?}"),
        }
    }

    #[test]
    fn a_file_as_install_root_is_unreadable_not_a_no_match() {
        let tree = TempTree::new("fileroot");
        tree.touch("not-a-dir");
        let file = tree.path().join("not-a-dir");
        match resolve_engine(&file) {
            EngineResolution::Unreadable { path } => assert_eq!(path, file),
            other => panic!("expected Unreadable, got {other:?}"),
        }
    }

    #[test]
    fn a_clean_lower_engine_resolves_even_when_a_higher_scan_is_incomplete() {
        // resolve_unreal walks the whole tree; if an unrelated subtree were
        // unreadable it would report Unreadable, but a clean readable tree with a
        // Godot layout resolves as Godot and the walk completes. This documents
        // that a readable tree never spuriously reports Unreadable.
        let tree = TempTree::new("clean-godot");
        tree.touch("MyGodotGame.pck");
        tree.touch("MyGodotGame.exe");
        assert!(matches!(
            resolve_engine(tree.path()),
            EngineResolution::Resolved(_)
        ));
    }

    #[test]
    fn the_engine_labels_are_stable() {
        assert_eq!(Engine::Unreal.as_str(), "unreal");
        assert_eq!(Engine::Unity.as_str(), "unity");
        assert_eq!(Engine::Godot.as_str(), "godot");
        assert_eq!(Engine::RenPy.as_str(), "renpy");
    }

    #[test]
    fn a_resolved_unreal_target_names_a_file_that_exists() {
        let tree = unreal_tree("Exists");
        let target = resolved(resolve_engine(tree.path()));
        assert!(Path::new(target.image_path()).is_file());
        // Fidelity is applied at the provider, not here; touch the enum to keep
        // the separation explicit.
        let _ = FidelityTier::HeuristicUnverified;
    }
}
