// SPDX-License-Identifier: Apache-2.0

//! The data-driven detection signature matcher (slice S053).
//!
//! Detection recognizes the engine, anti-cheat, and DRM technologies present in a
//! game's install directory. Slice S053 moves the signature list out of code (the
//! vendored SteamDB ruleset that this module replaces) and into a table in the
//! shipped catalog database, matched by one generic matcher. This module is that
//! matcher: it takes a caller-provided set of [`Signature`]s and a filesystem
//! location, and reports the [`DetectionFinding`]s. It reads no database (the store
//! and the seed live in `fragcap-targets`) and calls no platform API (the PE parse
//! reads the file's own bytes), so both the `technologies` command and the discovery
//! classifier reach it through the one crate they already depend on.
//!
//! # Passive and honest
//!
//! Matching reads directory entries and, for a `pe-version-string` signature, the
//! version resource in a binary's on-disk PE header. It opens no process handle,
//! reads no process memory, launches nothing, and makes no network call
//! (constitution P-1). A finding's fidelity is derived from its signature's
//! confidence (a definitive on-disk marker is [`FidelityTier::Verified`]); local
//! evidence outranks remote catalog claims (P-9).
//!
//! # Neutral evidence
//!
//! A detected anti-cheat or DRM product is recorded as a neutral fact. A
//! [`DetectionFinding`] carries no status, risk, or gating value; nothing about it
//! characterizes a title as off limits (specification section 3.6).
//!
//! # No silent loss
//!
//! Loading a signature set partitions rows into applied, inert (a match kind not
//! implemented this slice), and skipped (a malformed pattern); the three counts sum
//! to the rows loaded, so reduced coverage is visible rather than silent (P-4). An
//! unreadable subtree is surfaced distinctly from a clean empty scan.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::pe;
use crate::schema::FidelityTier;

/// The greatest directory depth a scan descends below its root. Bounds the walk so
/// detection stays affordable on a large install, mirroring the prior detector's
/// bound.
pub const MAX_SCAN_DEPTH: usize = 8;

/// A detection signature's category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignatureCategory {
    /// A game or software engine.
    Engine,
    /// An anti-cheat system.
    AntiCheat,
    /// A digital rights management or protection product.
    Drm,
}

impl SignatureCategory {
    /// The fixed display and grouping order.
    pub const ORDER: [SignatureCategory; 3] = [
        SignatureCategory::Engine,
        SignatureCategory::AntiCheat,
        SignatureCategory::Drm,
    ];

    /// The serialized form used in the seed document and the table.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureCategory::Engine => "engine",
            SignatureCategory::AntiCheat => "anti-cheat",
            SignatureCategory::Drm => "drm",
        }
    }

    /// Parse the serialized form, or `None` if it is not a category.
    pub fn parse(text: &str) -> Option<SignatureCategory> {
        match text {
            "engine" => Some(SignatureCategory::Engine),
            "anti-cheat" => Some(SignatureCategory::AntiCheat),
            "drm" => Some(SignatureCategory::Drm),
            _ => None,
        }
    }

    fn order_index(&self) -> usize {
        SignatureCategory::ORDER
            .iter()
            .position(|c| c == self)
            .expect("every category is in ORDER")
    }
}

/// How a signature's pattern is matched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignatureKind {
    /// The pattern is a glob over a file's name (basename). Example:
    /// `EasyAntiCheat*.dll`, `vgk.sys`.
    Filename,
    /// The pattern is a glob over a relative directory or file path with `/`
    /// separators significant. Example: a directory ending in `_Data`, an
    /// `Engine/Binaries/` tree.
    DirectoryShape,
    /// The pattern is matched against a string field of a candidate binary's PE
    /// version resource (`CompanyName`, `ProductName`, and similar). Reads the
    /// binary's on-disk bytes only.
    PeVersionString,
    /// The pattern is a byte or section marker inside an executable. Carried in the
    /// vocabulary but not implemented this slice: a signature of this kind is inert
    /// and never matches, surfaced as not-yet-matchable (P-4).
    BinaryMarker,
}

impl SignatureKind {
    /// The serialized form used in the seed document and the table.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureKind::Filename => "filename",
            SignatureKind::DirectoryShape => "directory-shape",
            SignatureKind::PeVersionString => "pe-version-string",
            SignatureKind::BinaryMarker => "binary-marker",
        }
    }

    /// Parse the serialized form, or `None` if it is not a kind.
    pub fn parse(text: &str) -> Option<SignatureKind> {
        match text {
            "filename" => Some(SignatureKind::Filename),
            "directory-shape" => Some(SignatureKind::DirectoryShape),
            "pe-version-string" => Some(SignatureKind::PeVersionString),
            "binary-marker" => Some(SignatureKind::BinaryMarker),
            _ => None,
        }
    }

    /// Whether the matcher applies this kind this slice. `BinaryMarker` is inert.
    pub fn is_implemented(&self) -> bool {
        !matches!(self, SignatureKind::BinaryMarker)
    }
}

/// How strong a signature's evidence is, which fidelity a match stamps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignatureConfidence {
    /// A named, product-specific marker (for example `UnityPlayer.dll`). A match
    /// stamps [`FidelityTier::Verified`].
    Definitive,
    /// A generic shape a non-target could also present (for example a lone `*.pck`).
    /// A match stamps [`FidelityTier::HeuristicUnverified`], so a weak local signal
    /// does not over-claim.
    Heuristic,
}

impl SignatureConfidence {
    /// The serialized form used in the seed document and the table.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureConfidence::Definitive => "definitive",
            SignatureConfidence::Heuristic => "heuristic",
        }
    }

    /// Parse the serialized form, or `None` if it is not a confidence value.
    pub fn parse(text: &str) -> Option<SignatureConfidence> {
        match text {
            "definitive" => Some(SignatureConfidence::Definitive),
            "heuristic" => Some(SignatureConfidence::Heuristic),
            _ => None,
        }
    }

    /// The fidelity a match of this confidence stamps.
    pub fn fidelity(&self) -> FidelityTier {
        match self {
            SignatureConfidence::Definitive => FidelityTier::Verified,
            SignatureConfidence::Heuristic => FidelityTier::HeuristicUnverified,
        }
    }
}

/// One detection signature: a row in the catalog `signature` table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    /// What the signature identifies (engine, anti-cheat, or DRM).
    pub category: SignatureCategory,
    /// How the pattern is matched.
    pub kind: SignatureKind,
    /// The pattern, interpreted per `kind`.
    pub pattern: String,
    /// The product named, for example `Unity`, `Easy Anti-Cheat`, `Denuvo`.
    pub product: String,
    /// How strong the evidence is, driving the fidelity a match stamps.
    pub confidence: SignatureConfidence,
}

/// One detected technology in one install directory. Neutral by construction: it
/// carries no status, risk, or gating value (specification section 3.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionFinding {
    /// The matched signature's category.
    pub category: SignatureCategory,
    /// The product named.
    pub product: String,
    /// The relative path or version-string field that matched: the auditable
    /// evidence for the finding.
    pub evidence: String,
    /// The fidelity this match stamps, derived from the signature's confidence.
    pub fidelity: FidelityTier,
}

/// A signature whose pattern could not be compiled, retained so reduced coverage is
/// visible rather than silent (P-4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedSignature {
    /// The product the signature named.
    pub product: String,
    /// The raw pattern.
    pub pattern: String,
    /// Why it was rejected.
    pub error: String,
}

/// One compiled, applicable signature.
struct Applied {
    category: SignatureCategory,
    kind: SignatureKind,
    product: String,
    fidelity: FidelityTier,
    regex: Regex,
}

/// The load-time product of a signature set: the compiled applicable signatures and
/// the accounting of what was not applied.
///
/// Invariant: `applied_count() + inert_count() + skipped_count() == total_count()`.
pub struct SignatureSet {
    applied: Vec<Applied>,
    inert: Vec<Signature>,
    skipped: Vec<SkippedSignature>,
    total: usize,
}

impl SignatureSet {
    /// Compile a set of signatures. An implemented-kind signature with a valid
    /// pattern is applied; a signature of an unimplemented kind (binary-marker) is
    /// inert; a malformed pattern is skipped and counted. One bad row never disables
    /// the rest (P-4).
    pub fn compile(signatures: &[Signature]) -> SignatureSet {
        let mut applied = Vec::new();
        let mut inert = Vec::new();
        let mut skipped = Vec::new();
        let total = signatures.len();

        for sig in signatures {
            if !sig.kind.is_implemented() {
                inert.push(sig.clone());
                continue;
            }
            match compile_pattern(sig.kind, &sig.pattern) {
                Ok(regex) => applied.push(Applied {
                    category: sig.category,
                    kind: sig.kind,
                    product: sig.product.clone(),
                    fidelity: sig.confidence.fidelity(),
                    regex,
                }),
                Err(error) => skipped.push(SkippedSignature {
                    product: sig.product.clone(),
                    pattern: sig.pattern.clone(),
                    error,
                }),
            }
        }

        SignatureSet {
            applied,
            inert,
            skipped,
            total,
        }
    }

    /// The number of applied (compiled, applicable) signatures.
    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }

    /// Signatures of an unimplemented kind, carried but not applied.
    pub fn inert(&self) -> &[Signature] {
        &self.inert
    }

    /// The number of inert (not-yet-matchable) signatures.
    pub fn inert_count(&self) -> usize {
        self.inert.len()
    }

    /// Signatures rejected at load, each naming its product.
    pub fn skipped(&self) -> &[SkippedSignature] {
        &self.skipped
    }

    /// The number of signatures skipped as malformed.
    pub fn skipped_count(&self) -> usize {
        self.skipped.len()
    }

    /// The total number of signatures loaded. Invariant:
    /// `applied_count() + inert_count() + skipped_count() == total_count()`.
    pub fn total_count(&self) -> usize {
        self.total
    }

    /// Scan a directory's bounded subtree and report every technology detected in
    /// it, deduplicated per (category, product) and grouped by category.
    ///
    /// Returns an error only when `root` itself cannot be read; an unreadable subtree
    /// is surfaced in [`ScanOutcome::unreadable`] and the scan continues.
    pub fn detect(&self, root: &Path) -> Result<ScanOutcome, DetectError> {
        if let Err(source) = fs::read_dir(root) {
            return Err(DetectError {
                path: root.to_path_buf(),
                source,
            });
        }

        let mut entries: Vec<Entry> = Vec::new();
        let mut unreadable: Vec<PathBuf> = Vec::new();
        walk(root, root, 1, &mut entries, &mut unreadable);

        let mut findings: Vec<DetectionFinding> = Vec::new();
        for rule in &self.applied {
            // A binary-marker never reaches here (inert). PE-version-string reads the
            // candidate file; the path kinds match against the collected entries.
            let hit = match rule.kind {
                SignatureKind::Filename => entries
                    .iter()
                    .find(|e| !e.is_dir && rule.regex.is_match(&e.base)),
                SignatureKind::DirectoryShape => {
                    entries.iter().find(|e| rule.regex.is_match(&e.match_path))
                }
                SignatureKind::PeVersionString => entries
                    .iter()
                    .find(|e| !e.is_dir && pe_version_matches(&e.full, &rule.regex)),
                SignatureKind::BinaryMarker => None,
            };
            if let Some(entry) = hit {
                let key = (rule.category, rule.product.clone());
                if !findings
                    .iter()
                    .any(|f| (f.category, f.product.clone()) == key)
                {
                    findings.push(DetectionFinding {
                        category: rule.category,
                        product: rule.product.clone(),
                        evidence: entry.display.clone(),
                        fidelity: rule.fidelity,
                    });
                }
            }
        }

        findings.sort_by(|a, b| {
            a.category
                .order_index()
                .cmp(&b.category.order_index())
                .then_with(|| a.product.cmp(&b.product))
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
    /// The detected technologies, deduplicated per (category, product), grouped by
    /// category then product.
    pub findings: Vec<DetectionFinding>,
    /// Paths under the root that could not be read. A non-empty list means coverage
    /// was reduced; an empty list with empty findings means a complete scan found
    /// nothing.
    pub unreadable: Vec<PathBuf>,
}

impl ScanOutcome {
    /// The highest-fidelity engine finding, if any: the detected engine for a
    /// candidate. The scan dedupes per product, so at most one finding per engine
    /// product; this returns the strongest by fidelity.
    pub fn detected_engine(&self) -> Option<&DetectionFinding> {
        self.findings
            .iter()
            .filter(|f| f.category == SignatureCategory::Engine)
            .max_by_key(|f| f.fidelity)
    }
}

/// The install root could not be read, so no scan was possible.
#[derive(Debug)]
pub struct DetectError {
    /// The root that could not be read.
    pub path: PathBuf,
    /// The underlying filesystem error.
    pub source: std::io::Error,
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

/// A path collected during the walk: the base name, the relative match path (a
/// directory carries a trailing `/` so a directory-shape pattern such as
/// `(?:^|/)EasyAntiCheat/` matches even when the directory is empty), a display
/// form, and the absolute path (for a PE read).
struct Entry {
    base: String,
    match_path: String,
    display: String,
    full: PathBuf,
    is_dir: bool,
}

/// Compile one signature pattern to a regex, per kind.
///
/// A filename glob matches a basename: `*` becomes `.*`, `?` becomes `.`, anchored
/// full-match, case-insensitive. A directory-shape glob matches within a relative
/// path: `*` becomes `[^/]*`, `?` becomes `[^/]`, searched (unanchored) so a nested
/// tree such as `Engine/Binaries/` matches anywhere in the path. A
/// pe-version-string pattern is a literal-insensitive substring, compiled as a
/// case-insensitive regex-escaped needle.
fn compile_pattern(kind: SignatureKind, pattern: &str) -> Result<Regex, String> {
    if pattern.is_empty() {
        return Err("empty pattern".to_string());
    }
    let body = match kind {
        SignatureKind::Filename => format!("^{}$", glob_to_regex(pattern, false)),
        SignatureKind::DirectoryShape => glob_to_regex(pattern, true),
        SignatureKind::PeVersionString => regex::escape(pattern),
        SignatureKind::BinaryMarker => unreachable!("binary-marker is inert"),
    };
    Regex::new(&format!("(?i){body}")).map_err(|e| e.to_string())
}

/// Convert a glob to a regex body. When `path_aware`, `*` matches within a path
/// segment (`[^/]*`); otherwise it matches any run (`.*`). Every other character is
/// regex-escaped.
fn glob_to_regex(glob: &str, path_aware: bool) -> String {
    let star = if path_aware { "[^/]*" } else { ".*" };
    let question = if path_aware { "[^/]" } else { "." };
    let mut out = String::new();
    for ch in glob.chars() {
        match ch {
            '*' => out.push_str(star),
            '?' => out.push_str(question),
            other => out.push_str(&regex::escape(&other.to_string())),
        }
    }
    out
}

/// Whether any string field of the file's PE version resource matches `needle`.
fn pe_version_matches(path: &Path, needle: &Regex) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    pe::version_strings(&bytes)
        .iter()
        .any(|s| needle.is_match(s))
}

/// The relative path of `path` under `base`, with `/` separators.
fn relative_path(base: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Collect matchable entries under `dir`, descending up to [`MAX_SCAN_DEPTH`].
/// Entries are visited in sorted order so a representative marker path is stable.
/// An unreadable directory is recorded and the walk continues.
fn walk(
    dir: &Path,
    base: &Path,
    depth: usize,
    out: &mut Vec<Entry>,
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
    let mut entry_error = false;
    for entry in read {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                entry_error = true;
                continue;
            }
        };
        match entry.file_type() {
            Ok(file_type) => children.push((entry.path(), file_type.is_dir())),
            Err(_) => unreadable.push(entry.path()),
        }
    }
    if entry_error {
        unreadable.push(dir.to_path_buf());
    }
    children.sort_by(|a, b| a.0.cmp(&b.0));

    for (path, is_dir) in children {
        let rel = relative_path(base, &path);
        let base_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if is_dir {
            out.push(Entry {
                base: base_name,
                match_path: format!("{rel}/"),
                display: rel,
                full: path.clone(),
                is_dir: true,
            });
            if depth < MAX_SCAN_DEPTH {
                walk(&path, base, depth + 1, out, unreadable);
            }
        } else {
            out.push(Entry {
                base: base_name,
                match_path: rel.clone(),
                display: rel,
                full: path,
                is_dir: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new(tag: &str) -> TempTree {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "fragcap-signature-{}-{}-{}",
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

        fn write(&self, rel: &str, bytes: &[u8]) {
            let full = self.root.join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("create parents");
            }
            fs::write(&full, bytes).expect("write file");
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

    fn sig(
        category: SignatureCategory,
        kind: SignatureKind,
        pattern: &str,
        product: &str,
        confidence: SignatureConfidence,
    ) -> Signature {
        Signature {
            category,
            kind,
            pattern: pattern.to_string(),
            product: product.to_string(),
            confidence,
        }
    }

    #[test]
    fn a_filename_signature_matches_and_stamps_verified() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::Filename,
            "UnityPlayer.dll",
            "Unity",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("unity");
        tree.touch("UnityPlayer.dll");
        let outcome = set.detect(tree.path()).expect("readable");
        let engine = outcome.detected_engine().expect("unity detected");
        assert_eq!(engine.product, "Unity");
        assert_eq!(engine.fidelity, FidelityTier::Verified);
    }

    #[test]
    fn a_filename_glob_matches() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::AntiCheat,
            SignatureKind::Filename,
            "EasyAntiCheat*.dll",
            "Easy Anti-Cheat",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("eac");
        tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll");
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(outcome
            .findings
            .iter()
            .any(|f| f.product == "Easy Anti-Cheat"));
    }

    #[test]
    fn a_directory_shape_matches_a_nested_tree() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::DirectoryShape,
            "Engine/Binaries/",
            "Unreal",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("unreal");
        tree.mkdir("Engine/Binaries/Win64");
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(outcome.findings.iter().any(|f| f.product == "Unreal"));
    }

    #[test]
    fn a_directory_shape_wildcard_segment_matches() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::DirectoryShape,
            "*_Data/",
            "Unity",
            SignatureConfidence::Heuristic,
        )]);
        let tree = TempTree::new("unity-data");
        tree.mkdir("MyGame_Data");
        let outcome = set.detect(tree.path()).expect("readable");
        let unity = outcome
            .findings
            .iter()
            .find(|f| f.product == "Unity")
            .expect("unity by data shape");
        assert_eq!(unity.fidelity, FidelityTier::HeuristicUnverified);
    }

    #[test]
    fn an_inert_kind_is_counted_not_applied() {
        let set = SignatureSet::compile(&[
            sig(
                SignatureCategory::Drm,
                SignatureKind::BinaryMarker,
                "denuvo-marker",
                "Denuvo",
                SignatureConfidence::Definitive,
            ),
            sig(
                SignatureCategory::Drm,
                SignatureKind::Filename,
                "steam_api64.dll",
                "Steam DRM",
                SignatureConfidence::Definitive,
            ),
        ]);
        assert_eq!(set.total_count(), 2);
        assert_eq!(set.applied_count(), 1);
        assert_eq!(set.inert_count(), 1);
        assert_eq!(set.skipped_count(), 0);
        assert_eq!(
            set.applied_count() + set.inert_count() + set.skipped_count(),
            set.total_count()
        );
    }

    #[test]
    fn a_malformed_pattern_is_skipped_not_dropped() {
        // A directory-shape pattern that yields an invalid regex fragment. `[` is
        // regex-escaped by the glob compiler, so force an error via a raw regex
        // metacharacter path that escaping cannot rescue is hard; instead use a
        // filename pattern with an unmatched group is also escaped. Use a pattern
        // that compiles to an invalid regex by embedding a lone `\` at end after
        // escaping is not possible. Simplest: an empty pattern is rejected.
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::Filename,
            "",
            "Bad",
            SignatureConfidence::Heuristic,
        )]);
        assert_eq!(set.total_count(), 1);
        assert_eq!(set.applied_count(), 0);
        assert_eq!(set.skipped_count(), 1);
        assert_eq!(set.skipped()[0].product, "Bad");
    }

    #[test]
    fn an_empty_install_reports_nothing_and_no_error() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::Filename,
            "UnityPlayer.dll",
            "Unity",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("empty");
        tree.touch("readme.txt");
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(outcome.findings.is_empty());
        assert!(outcome.unreadable.is_empty());
    }

    #[test]
    fn an_unreadable_root_is_an_error() {
        let set = SignatureSet::compile(&[]);
        let missing =
            std::env::temp_dir().join(format!("fragcap-signature-absent-{}", std::process::id()));
        assert!(set.detect(&missing).is_err());
    }

    #[test]
    fn a_finding_carries_no_gate_only_neutral_fields() {
        // Structural guard for the neutral-evidence rule: a DetectionFinding exposes
        // category, product, evidence, and fidelity, and nothing that characterizes a
        // title as off limits. A future status/gate field would break this
        // construction (the test is edited with the field, forcing the reviewer to
        // see it).
        let f = DetectionFinding {
            category: SignatureCategory::AntiCheat,
            product: "Vanguard".to_string(),
            evidence: "vgk.sys".to_string(),
            fidelity: FidelityTier::Verified,
        };
        assert_eq!(f.product, "Vanguard");
    }

    #[test]
    fn a_pe_version_string_matches_a_crafted_binary() {
        // A minimal PE image carrying a VS_VERSIONINFO with a ProductName is built by
        // the pe test helper; the signature matches that product name.
        let bytes =
            crate::pe::tests_support::minimal_pe_with_version_string("ProductName", "Frostbite");
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::PeVersionString,
            "Frostbite",
            "Frostbite",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("pe");
        tree.write("game.exe", &bytes);
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(
            outcome.findings.iter().any(|f| f.product == "Frostbite"),
            "pe version string matched: {:?}",
            outcome.findings
        );
    }
}
