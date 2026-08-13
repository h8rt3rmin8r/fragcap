// SPDX-License-Identifier: Apache-2.0

//! Technology detection: recognizing the engine, anti-cheat, SDK, emulator,
//! container, and launcher technologies present in a game's install directory,
//! from file paths alone.
//!
//! Where [`crate::engine_rule`] reads an install layout only to name the
//! socket-holding client, this module reports the wider set of technologies the
//! same directory reveals. It is built on the open SteamDB
//! `SteamDatabase/FileDetectionRuleSets` ruleset (MIT), the maintained source
//! behind SteamDB's technology attribution, which detects technologies from
//! depot file paths alone, never from file contents. That is exactly fragcap's
//! constraint, so the whole ruleset is vendored (`assets/steamdb/rules.ini`,
//! pinned and hash-locked in `rules.lock.json`) and applied to a real install
//! the operator already has on disk.
//!
//! # Passive and honest
//!
//! Detection reads directory entries and matches the ruleset's path regexes
//! against the relative paths it finds. It opens no process handle, reads no
//! process memory, reads no file content, launches nothing, and makes no network
//! call (constitution P-1). A path regex is a heuristic, so every finding is
//! stamped [`FidelityTier::HeuristicUnverified`], never higher (P-9). Surfacing a
//! detected anti-cheat is a user-safety and consent signal, not an evasion aid:
//! fragcap detects it and never interacts with it.
//!
//! # No silent loss
//!
//! The ruleset is authored for a PCRE-style engine and contains constructs the
//! RE2-family `regex` crate cannot compile (possessive quantifiers, atomic
//! groups). Each pattern is compiled independently; one that fails is skipped,
//! counted, and recorded with the technology it belonged to
//! ([`CompiledRuleset::skipped`]), never silently dropped, and the vendored bytes
//! are never edited to force compilation (P-4). An unreadable install subtree is
//! surfaced in [`ScanOutcome::unreadable`], distinct from a clean empty scan; an
//! unreadable install root is a [`DetectError`].
//!
//! # What this does not do
//!
//! This module labels technologies. It does not decide which executable is the
//! socket-holding client; that stays with the resolver and the engine rule. The
//! ruleset's second-pass `Evidence` deduction (secondary hints plus engine
//! inference) is not applied here; only the direct category sections are.

use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use regex::RegexBuilder;

use crate::schema::FidelityTier;

/// The vendored SteamDB ruleset, embedded at build time. The bytes here are the
/// bytes the lock in [`RULES_LOCK`] hashes.
pub const RULES_INI: &str = include_str!("../assets/steamdb/rules.ini");

/// The integrity and provenance lock for [`RULES_INI`].
pub const RULES_LOCK: &str = include_str!("../assets/steamdb/rules.lock.json");

/// The maximum directory depth the scan descends below the install root. Bounds
/// the walk so detection stays affordable on a large install and does not follow
/// a pathological tree without limit, mirroring the bounded engine-rule scan.
pub const MAX_SCAN_DEPTH: usize = 8;

/// A technology category. The vocabulary is the master schema's `technologies`
/// category enum; the vendored ruleset populates six of the eight, and
/// `Framework`/`Runtime` are defined for future sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    /// Game or software engine.
    Engine,
    /// Anti-cheat system.
    AntiCheat,
    /// Software development kit or middleware library.
    Sdk,
    /// Application framework. Defined for future sources; unpopulated by the
    /// vendored ruleset.
    Framework,
    /// Bundled emulator.
    Emulator,
    /// A wrapper that hosts other technologies (for example Electron).
    Container,
    /// Language or platform runtime. Defined for future sources; unpopulated by
    /// the vendored ruleset.
    Runtime,
    /// A storefront or publisher launcher.
    Launcher,
}

impl Category {
    /// The fixed display and grouping order.
    pub const ORDER: [Category; 8] = [
        Category::Engine,
        Category::AntiCheat,
        Category::Sdk,
        Category::Framework,
        Category::Emulator,
        Category::Container,
        Category::Runtime,
        Category::Launcher,
    ];

    /// The schema serialization for this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Engine => "engine",
            Category::AntiCheat => "anti_cheat",
            Category::Sdk => "sdk",
            Category::Framework => "framework",
            Category::Emulator => "emulator",
            Category::Container => "container",
            Category::Runtime => "runtime",
            Category::Launcher => "launcher",
        }
    }

    /// The applied category for a ruleset section name, or `None` for a section
    /// this slice does not apply (`Evidence`) or does not recognize. Returning
    /// `None` means the section's rule lines are skipped entirely rather than
    /// miscounted.
    pub fn from_section(section: &str) -> Option<Category> {
        match section {
            "Engine" => Some(Category::Engine),
            "AntiCheat" => Some(Category::AntiCheat),
            "SDK" => Some(Category::Sdk),
            "Emulator" => Some(Category::Emulator),
            "Container" => Some(Category::Container),
            "Launcher" => Some(Category::Launcher),
            _ => None,
        }
    }

    fn order_index(&self) -> usize {
        Category::ORDER
            .iter()
            .position(|c| c == self)
            .expect("every category is in ORDER")
    }
}

/// One detected technology in one install directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TechnologyFinding {
    /// The technology category.
    pub category: Category,
    /// The technology name (the ruleset key, as written).
    pub name: String,
    /// The relative install-directory path of a representative file or directory
    /// that matched: the auditable evidence for the finding.
    pub marker_path: String,
    /// Always [`FidelityTier::HeuristicUnverified`]: a path match is a guess.
    pub fidelity: FidelityTier,
}

/// A ruleset pattern that could not be compiled, retained so reduced coverage is
/// visible rather than silent (P-4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedPattern {
    /// The category the pattern belonged to.
    pub category: Category,
    /// The technology the pattern belonged to, so the affected technology is
    /// identifiable.
    pub technology: String,
    /// The raw pattern text, as vendored.
    pub pattern: String,
    /// Why the engine rejected it.
    pub error: String,
}

/// One successfully compiled rule.
struct CompiledRule {
    category: Category,
    technology: String,
    regex: regex::Regex,
}

/// The load-time product of the vendored ruleset: the compiled applied rules plus
/// the accounting of patterns skipped as incompatible.
pub struct CompiledRuleset {
    rules: Vec<CompiledRule>,
    skipped: Vec<SkippedPattern>,
    total: usize,
}

impl CompiledRuleset {
    /// Parse and compile the embedded vendored ruleset.
    pub fn embedded() -> CompiledRuleset {
        CompiledRuleset::parse(RULES_INI)
    }

    /// Parse and compile a ruleset from its text. Only the applied category
    /// sections are compiled; a pattern the engine rejects is skipped and
    /// counted.
    pub fn parse(text: &str) -> CompiledRuleset {
        let mut rules = Vec::new();
        let mut skipped = Vec::new();
        let mut total = 0usize;
        let mut section: Option<Category> = None;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }
            if let Some(name) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = Category::from_section(name);
                continue;
            }
            // Only lines inside an applied section are rules; anything under
            // Evidence or an unrecognized section is left uncounted.
            let Some(category) = section else {
                continue;
            };
            let Some((key_raw, value_raw)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key_raw.trim();
            let technology = key.strip_suffix("[]").unwrap_or(key).to_string();
            let pattern = strip_inline_comment(value_raw.trim()).trim().to_string();
            if pattern.is_empty() {
                continue;
            }

            total += 1;
            match RegexBuilder::new(&pattern).case_insensitive(true).build() {
                Ok(regex) => rules.push(CompiledRule {
                    category,
                    technology,
                    regex,
                }),
                Err(error) => skipped.push(SkippedPattern {
                    category,
                    technology,
                    pattern,
                    error: error.to_string(),
                }),
            }
        }

        CompiledRuleset {
            rules,
            skipped,
            total,
        }
    }

    /// The number of patterns that compiled.
    pub fn compiled_count(&self) -> usize {
        self.rules.len()
    }

    /// The patterns skipped as incompatible, each naming its category and
    /// technology.
    pub fn skipped(&self) -> &[SkippedPattern] {
        &self.skipped
    }

    /// The number of patterns skipped as incompatible.
    pub fn skipped_count(&self) -> usize {
        self.skipped.len()
    }

    /// The total number of patterns in the applied sections. Invariant:
    /// `compiled_count() + skipped_count() == total_count()`.
    pub fn total_count(&self) -> usize {
        self.total
    }

    /// Scan an install directory and report the technologies detected in it.
    ///
    /// Returns an error only when the install root itself cannot be read; an
    /// unreadable subtree is surfaced in [`ScanOutcome::unreadable`] and the scan
    /// continues over the readable remainder.
    pub fn detect(&self, root: &Path) -> Result<ScanOutcome, DetectError> {
        // Establish that the root is a readable directory, so an unreadable root
        // is distinguishable from a readable-but-empty one.
        if let Err(source) = fs::read_dir(root) {
            return Err(DetectError {
                path: root.to_path_buf(),
                source,
            });
        }

        let mut entries: Vec<MatchEntry> = Vec::new();
        let mut unreadable: Vec<PathBuf> = Vec::new();
        walk(root, root, 1, &mut entries, &mut unreadable);

        let mut findings: Vec<TechnologyFinding> = Vec::new();
        let mut seen: HashSet<(Category, String)> = HashSet::new();
        for entry in &entries {
            for rule in &self.rules {
                if rule.regex.is_match(&entry.match_str) {
                    let key = (rule.category, rule.technology.clone());
                    if seen.insert(key) {
                        findings.push(TechnologyFinding {
                            category: rule.category,
                            name: rule.technology.clone(),
                            marker_path: entry.display.clone(),
                            fidelity: FidelityTier::HeuristicUnverified,
                        });
                    }
                }
            }
        }

        findings.sort_by(|a, b| {
            a.category
                .order_index()
                .cmp(&b.category.order_index())
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(ScanOutcome {
            findings,
            unreadable,
        })
    }
}

/// The result of scanning one install directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanOutcome {
    /// The detected technologies, deduplicated per (category, technology) and
    /// grouped by category then technology name.
    pub findings: Vec<TechnologyFinding>,
    /// Paths under the root that could not be read. Non-empty means coverage was
    /// reduced; an empty vector with empty findings means a complete scan found
    /// nothing.
    pub unreadable: Vec<PathBuf>,
}

/// The install root could not be read, so no scan was possible.
#[derive(Debug)]
pub struct DetectError {
    /// The install root that could not be read.
    pub path: PathBuf,
    /// The underlying filesystem error.
    pub source: io::Error,
}

impl fmt::Display for DetectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot read install directory {}", self.path.display())
    }
}

impl std::error::Error for DetectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// A path collected during the walk, in both its match form and its display
/// form. A directory's match form carries a trailing `/` so a directory-marker
/// rule such as `(?:^|/)EasyAntiCheat/` matches even when the directory is empty.
struct MatchEntry {
    match_str: String,
    display: String,
}

/// Strip a trailing ` ; comment` from a rule's value. A `;` is treated as a
/// comment delimiter only when preceded by whitespace, so a `;` inside a regex
/// is left intact.
fn strip_inline_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] == b';' && (bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
            return &value[..i];
        }
    }
    value
}

/// The relative path of `path` under `base`, with `/` separators, matching the
/// ruleset's path convention.
fn relative_path(base: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Collect matchable relative paths under `dir`, descending up to
/// [`MAX_SCAN_DEPTH`]. Entries are visited in sorted order so a representative
/// marker path is stable across runs. An unreadable directory is recorded and
/// the walk continues.
fn walk(
    dir: &Path,
    base: &Path,
    depth: usize,
    out: &mut Vec<MatchEntry>,
    unreadable: &mut Vec<PathBuf>,
) {
    let read = match fs::read_dir(dir) {
        Ok(read) => read,
        Err(_) => {
            unreadable.push(dir.to_path_buf());
            return;
        }
    };

    let mut children: Vec<(PathBuf, bool)> = Vec::new();
    for entry in read {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        match entry.file_type() {
            Ok(file_type) => children.push((entry.path(), file_type.is_dir())),
            Err(_) => unreadable.push(entry.path()),
        }
    }
    children.sort_by(|a, b| a.0.cmp(&b.0));

    for (path, is_dir) in children {
        let rel = relative_path(base, &path);
        if is_dir {
            out.push(MatchEntry {
                match_str: format!("{rel}/"),
                display: rel,
            });
            if depth < MAX_SCAN_DEPTH {
                walk(&path, base, depth + 1, out, unreadable);
            }
        } else {
            out.push(MatchEntry {
                match_str: rel.clone(),
                display: rel,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A throwaway directory tree under the system temp dir, removed on drop, in
    /// the spirit of the engine-rule and Steam-crate test helpers.
    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new(tag: &str) -> TempTree {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "fragcap-technologies-{}-{}-{}",
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

        fn touch(&self, rel: &str) {
            let full = self.root.join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("create parents");
            }
            fs::write(&full, b"").expect("write file");
        }

        fn mkdir(&self, rel: &str) {
            fs::create_dir_all(self.root.join(rel)).expect("create dir");
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn embedded() -> CompiledRuleset {
        CompiledRuleset::embedded()
    }

    // --- US3: skip-count conservation over the real vendored ruleset ---

    #[test]
    fn conservation_holds_over_the_vendored_ruleset() {
        let ruleset = embedded();
        assert_eq!(
            ruleset.compiled_count() + ruleset.skipped_count(),
            ruleset.total_count(),
            "every applied pattern is either compiled or a counted skip"
        );
        assert_eq!(
            ruleset.total_count(),
            376,
            "the pinned ruleset has 376 applied-section patterns"
        );
    }

    #[test]
    fn incompatible_patterns_are_skipped_and_surfaced() {
        let ruleset = embedded();
        // The pinned ruleset's applied sections contain RE2-incompatible
        // constructs (possessive quantifiers, atomic groups), so at least one
        // pattern is skipped, and each skip names its affected technology.
        assert!(
            ruleset.skipped_count() >= 1,
            "the vendored ruleset has incompatible patterns to skip"
        );
        for skip in ruleset.skipped() {
            assert!(!skip.technology.is_empty());
            assert!(!skip.pattern.is_empty());
        }
    }

    #[test]
    fn a_synthetic_incompatible_pattern_is_skipped_not_dropped() {
        // A two-rule mini-ruleset: one compiles, one uses an atomic group the
        // RE2-family engine rejects. Both are counted; the good one is usable.
        let text = "[Engine]\nGood = (?:^|/)good\\.exe$\nBad = (?:^|/)ba(?>d)\\.exe$\n";
        let ruleset = CompiledRuleset::parse(text);
        assert_eq!(ruleset.total_count(), 2);
        assert_eq!(ruleset.compiled_count(), 1);
        assert_eq!(ruleset.skipped_count(), 1);
        assert_eq!(ruleset.skipped()[0].technology, "Bad");
        assert_eq!(ruleset.skipped()[0].category, Category::Engine);
    }

    #[test]
    fn evidence_section_lines_are_not_counted() {
        let text = "[Evidence]\nPCK = \\.pck$\n[Engine]\nGodot = (?:^|/)x\\.pck$\n";
        let ruleset = CompiledRuleset::parse(text);
        assert_eq!(ruleset.total_count(), 1, "only the Engine rule is counted");
    }

    // --- US1: detection from an install layout ---

    #[test]
    fn engine_and_anticheat_are_detected_with_marker_paths() {
        let ruleset = embedded();
        let tree = TempTree::new("eac-unreal");
        // An Unreal asset marker (the ruleset keys Unreal on .uasset/.upk, not on
        // the shipping-exe name that fragcap's own engine rule uses) and an
        // EasyAntiCheat dll marker.
        tree.touch("Content/Maps/Level.uasset");
        tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll");
        let outcome = ruleset.detect(tree.path()).expect("root is readable");

        let engine = outcome
            .findings
            .iter()
            .find(|f| f.category == Category::Engine)
            .expect("an engine was detected");
        assert_eq!(engine.name, "Unreal");
        assert!(engine.marker_path.ends_with("Level.uasset"));
        assert_eq!(engine.fidelity, FidelityTier::HeuristicUnverified);

        let ac = outcome
            .findings
            .iter()
            .find(|f| f.category == Category::AntiCheat)
            .expect("an anti-cheat was detected");
        assert_eq!(ac.name, "EasyAntiCheat");
        assert!(ac.marker_path.ends_with("EasyAntiCheat_x64.dll"));
        assert_eq!(ac.fidelity, FidelityTier::HeuristicUnverified);
    }

    #[test]
    fn a_directory_marker_is_detected_even_when_empty() {
        let ruleset = embedded();
        let tree = TempTree::new("dir-marker");
        // FredaikisAntiCheat is a directory-marker rule: (?:^|/)FredaikisAntiCheat/
        tree.mkdir("FredaikisAntiCheat");
        let outcome = ruleset.detect(tree.path()).expect("root is readable");
        assert!(
            outcome
                .findings
                .iter()
                .any(|f| f.name == "FredaikisAntiCheat"),
            "an empty directory marker is still detected"
        );
    }

    #[test]
    fn a_technology_with_several_markers_is_reported_once() {
        let ruleset = embedded();
        let tree = TempTree::new("dedup");
        // Two EasyAntiCheat markers; the technology must appear once.
        tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll");
        tree.touch("EasyAntiCheat/EasyAntiCheat_EOS_Setup.exe");
        let outcome = ruleset.detect(tree.path()).expect("root is readable");
        let count = outcome
            .findings
            .iter()
            .filter(|f| f.name == "EasyAntiCheat")
            .count();
        assert_eq!(count, 1, "one finding per technology, not per marker file");
    }

    #[test]
    fn an_empty_install_reports_nothing_and_no_error() {
        let ruleset = embedded();
        let tree = TempTree::new("empty");
        tree.touch("readme.txt");
        let outcome = ruleset.detect(tree.path()).expect("root is readable");
        assert!(outcome.findings.is_empty());
        assert!(outcome.unreadable.is_empty());
    }

    #[test]
    fn an_unreadable_root_is_an_error_not_an_empty_scan() {
        let ruleset = embedded();
        let missing = std::env::temp_dir().join(format!(
            "fragcap-technologies-absent-{}-{}",
            std::process::id(),
            "x"
        ));
        let result = ruleset.detect(&missing);
        assert!(result.is_err(), "an absent root is a DetectError");
    }

    // --- US4: the vendored asset is faithful and locked ---

    #[test]
    fn the_embedded_ruleset_matches_the_recorded_hash() {
        let lock: serde_json::Value =
            serde_json::from_str(RULES_LOCK).expect("the lock is valid JSON");
        let recorded = lock["sha256"].as_str().expect("the lock records a sha256");
        let actual = crate::sha256::hex_digest(RULES_INI.as_bytes());
        assert_eq!(
            actual, recorded,
            "the embedded rules.ini hashes to the value in rules.lock.json"
        );
    }

    #[test]
    fn the_lock_records_its_provenance() {
        let lock: serde_json::Value =
            serde_json::from_str(RULES_LOCK).expect("the lock is valid JSON");
        assert!(lock["source"]
            .as_str()
            .is_some_and(|s| s.contains("FileDetectionRuleSets")));
        assert_eq!(lock["commit"].as_str().map(|s| s.len()), Some(40));
        assert_eq!(lock["license"].as_str(), Some("MIT"));
        assert!(lock["sha256"].as_str().is_some_and(|s| s.len() == 64));
    }
}
