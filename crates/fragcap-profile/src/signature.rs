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

/// The greatest walk depth at which an executable is a candidate for a
/// section-marker read (slice S065).
///
/// This is definitional rather than a truncation: a plausible launch target sits
/// near the install root, and an executable below this is a redistributable
/// installer, a crash handler, or a tool, not the binary a DRM wrapper is applied
/// to. Four is the depth of the deepest launch executable in the measured sample
/// (`Pioneer/Binaries/Win64/PioneerGame.exe`), which is also Unreal's convention for
/// every shipping binary. Because it defines the candidate set rather than
/// truncating it, an executable excluded by depth is not counted as loss.
pub const MARKER_SCAN_MAX_DEPTH: usize = 4;

/// The greatest number of candidate executables read for section markers in one
/// install directory.
///
/// Unlike [`MARKER_SCAN_MAX_DEPTH`] this truncates a set that was already defined,
/// so every candidate it drops is counted in
/// [`ScanOutcome::marker_candidates_skipped`] and makes the scan incomplete (P-4).
pub const MARKER_SCAN_MAX_CANDIDATES: usize = 64;

/// The number of leading bytes of a candidate executable read for its section table.
///
/// The DOS stub, the COFF file header, and the section table sit within the first
/// few kilobytes of any real image, so this is enormous margin. Reading a prefix
/// rather than the file keeps a section-marker signature affordable against a
/// launch executable of any size.
pub const MARKER_SCAN_PREFIX_BYTES: usize = 64 * 1024;

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
    /// The pattern is a marker inside an executable. Matchable in one form,
    /// `section:<glob>`, which matches a name in the binary's PE section table
    /// (slice S065). Every other pattern of this kind names a byte sequence that
    /// this build does not implement; such a row is inert and never matches,
    /// surfaced as not-yet-matchable rather than dropped (P-4).
    BinaryMarker,
}

/// The prefix that marks a `binary-marker` pattern as a PE section name. A pattern
/// without it names a byte sequence, which this build does not match.
pub const SECTION_MARKER_PREFIX: &str = "section:";

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

impl Signature {
    /// Whether the matcher can apply this row.
    ///
    /// Matchability is a property of the row rather than of the kind, because a
    /// `binary-marker` is matchable in its `section:` form and inert in every other
    /// form. A row this returns `false` for is carried and counted inert, never
    /// dropped and never reported as malformed (P-4).
    pub fn is_matchable(&self) -> bool {
        match self.kind {
            SignatureKind::BinaryMarker => self.pattern.starts_with(SECTION_MARKER_PREFIX),
            _ => true,
        }
    }
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

/// How a compiled signature is evaluated against the collected entries. Derived
/// from the kind and, for a `binary-marker`, from the pattern form, so the match
/// loop never re-inspects a pattern it already compiled.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MatchMode {
    /// Match the regex against a file's basename.
    Filename,
    /// Match the regex within a relative path at a component boundary.
    DirectoryShape,
    /// Match the regex against a string field of a candidate binary's PE version
    /// resource.
    PeVersionString,
    /// Match the regex against a name in a candidate executable's PE section table.
    PeSectionName,
}

/// One compiled, applicable signature.
struct Applied {
    category: SignatureCategory,
    mode: MatchMode,
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
    /// Compile a set of signatures. A matchable signature with a valid pattern is
    /// applied; a signature whose kind and pattern form this build cannot match (a
    /// byte-sequence binary-marker) is inert; a malformed pattern is skipped and
    /// counted. One bad row never disables the rest (P-4).
    pub fn compile(signatures: &[Signature]) -> SignatureSet {
        let mut applied = Vec::new();
        let mut inert = Vec::new();
        let mut skipped = Vec::new();
        let total = signatures.len();

        for sig in signatures {
            if !sig.is_matchable() {
                inert.push(sig.clone());
                continue;
            }
            match compile_pattern(sig.kind, &sig.pattern) {
                Ok((mode, regex)) => applied.push(Applied {
                    category: sig.category,
                    mode,
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

        // The section-marker candidate set, read once and shared by every
        // section-name rule so a tree is not re-read per rule. Only executables near
        // the root are candidates (see `MARKER_SCAN_MAX_DEPTH`), and the set is
        // truncated by `MARKER_SCAN_MAX_CANDIDATES` with the drop counted.
        let mut marker_candidates_skipped = 0usize;
        let sections = if self
            .applied
            .iter()
            .any(|r| r.mode == MatchMode::PeSectionName)
        {
            self.section_candidates(&entries, &mut unreadable, &mut marker_candidates_skipped)
        } else {
            // No section rule is loaded, so no executable is opened at all.
            Vec::new()
        };

        let mut findings: Vec<DetectionFinding> = Vec::new();
        for rule in &self.applied {
            // The path modes match against the collected entries; the two binary
            // modes read a candidate file's own bytes.
            let hit = match rule.mode {
                MatchMode::Filename => entries
                    .iter()
                    .find(|e| !e.is_dir && rule.regex.is_match(&e.base)),
                MatchMode::DirectoryShape => {
                    entries.iter().find(|e| rule.regex.is_match(&e.match_path))
                }
                // Only a file that could be a PE image is opened, so a large tree of
                // data files is not read byte-for-byte on the chance one matches.
                MatchMode::PeVersionString => entries.iter().find(|e| {
                    !e.is_dir
                        && is_pe_image_name(&e.base)
                        && pe_version_matches(&e.full, &rule.regex)
                }),
                MatchMode::PeSectionName => sections
                    .iter()
                    .find(|(_, names)| names.iter().any(|n| rule.regex.is_match(n)))
                    .map(|(entry, _)| *entry),
            };
            if let Some(entry) = hit {
                let candidate = DetectionFinding {
                    category: rule.category,
                    product: rule.product.clone(),
                    evidence: entry.display.clone(),
                    fidelity: rule.fidelity,
                };
                // Deduplicate per (category, product), but keep the strongest
                // fidelity: a definitive marker must not be shadowed by a weaker
                // shape whose row happened to come first.
                match findings
                    .iter_mut()
                    .find(|f| f.category == candidate.category && f.product == candidate.product)
                {
                    Some(existing) if candidate.fidelity > existing.fidelity => {
                        *existing = candidate
                    }
                    Some(_) => {}
                    None => findings.push(candidate),
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
            marker_candidates_skipped,
        })
    }

    /// The section names of every candidate executable, paired with the entry that
    /// produced them.
    ///
    /// A candidate is a file whose name ends in `.exe` at walk depth
    /// [`MARKER_SCAN_MAX_DEPTH`] or less, in the walk's sorted order, so the result
    /// does not depend on filesystem iteration order. The set is truncated at
    /// [`MARKER_SCAN_MAX_CANDIDATES`] and every drop is counted. Each candidate is
    /// read as a bounded [`MARKER_SCAN_PREFIX_BYTES`] prefix, never whole, and a
    /// candidate that cannot be opened is recorded unreadable rather than treated as
    /// carrying no marker: absent is not the same as unread (P-9).
    fn section_candidates<'e>(
        &self,
        entries: &'e [Entry],
        unreadable: &mut Vec<PathBuf>,
        skipped: &mut usize,
    ) -> Vec<(&'e Entry, Vec<String>)> {
        let mut out = Vec::new();
        for entry in entries.iter().filter(|e| {
            !e.is_dir && e.depth <= MARKER_SCAN_MAX_DEPTH && is_executable_name(&e.base)
        }) {
            if out.len() >= MARKER_SCAN_MAX_CANDIDATES {
                *skipped += 1;
                continue;
            }
            match read_prefix(&entry.full, MARKER_SCAN_PREFIX_BYTES) {
                Some(bytes) => out.push((entry, pe::section_names(&bytes))),
                None => unreadable.push(entry.full.clone()),
            }
        }
        out
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
    /// Candidate executables not read for section markers because
    /// [`MARKER_SCAN_MAX_CANDIDATES`] truncated the candidate set. A non-zero count
    /// means coverage was reduced, so it is reported rather than only tallied (P-4).
    pub marker_candidates_skipped: usize,
}

impl ScanOutcome {
    /// The named diagnostics for everything this scan did not cover: one line per
    /// unreadable path, and one naming the candidates a scan bound truncated.
    ///
    /// P-4 asks for a loss to be counted and surfaced. [`is_complete`] answers the
    /// counted half and this answers the named half, so an operator seeing a row
    /// marked incomplete can recover which subtree or which bound caused it. It
    /// lives here rather than at each caller because every caller needs the same
    /// lines, and two hand-written copies would be free to drift apart.
    ///
    /// [`is_complete`]: ScanOutcome::is_complete
    pub fn coverage_warnings(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .unreadable
            .iter()
            .map(|p| format!("could not read subtree during detection: {}", p.display()))
            .collect();
        if self.marker_candidates_skipped > 0 {
            out.push(format!(
                "detection read only the first {} executable(s) for binary markers; \
                 {} more were not examined",
                MARKER_SCAN_MAX_CANDIDATES, self.marker_candidates_skipped
            ));
        }
        out
    }

    /// Whether the scan covered everything it set out to cover: nothing was
    /// unreadable and no candidate was dropped by a cap.
    ///
    /// A complete scan with no findings is a real answer ("nothing is here"); an
    /// incomplete scan with no findings is not, and the two must not render alike
    /// (P-4, P-9).
    pub fn is_complete(&self) -> bool {
        self.unreadable.is_empty() && self.marker_candidates_skipped == 0
    }

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
    /// The walk depth, 1 for a direct child of the root. Bounds the section-marker
    /// candidate set (see [`MARKER_SCAN_MAX_DEPTH`]).
    depth: usize,
}

/// Compile one signature pattern to a regex, per kind.
///
/// A filename glob matches a basename: `*` becomes `.*`, `?` becomes `.`, anchored
/// full-match, case-insensitive. A directory-shape glob matches within a relative
/// path but only at a path-component boundary: the pattern is prefixed with
/// `(?:^|/)` so `GameGuard/` matches `GameGuard/` or `sub/GameGuard/` but not
/// `NotGameGuard/`, and `Engine/Binaries/` does not match `MyEngine/Binaries/`.
/// Inside it `*` becomes `[^/]*` and `?` becomes `[^/]`. A pe-version-string pattern
/// is a literal-insensitive substring, compiled as a case-insensitive regex-escaped
/// needle. A binary-marker pattern of the form `section:<glob>` compiles the glob
/// after the prefix the same way a filename glob compiles, anchored and
/// case-insensitive, and is matched against a candidate executable's PE section
/// names; `section:` with nothing after it is a malformed row, not an inert one.
fn compile_pattern(kind: SignatureKind, pattern: &str) -> Result<(MatchMode, Regex), String> {
    if pattern.is_empty() {
        return Err("empty pattern".to_string());
    }
    let (mode, body) = match kind {
        SignatureKind::Filename => (
            MatchMode::Filename,
            format!("^{}$", glob_to_regex(pattern, false)),
        ),
        SignatureKind::DirectoryShape => (
            MatchMode::DirectoryShape,
            format!("(?:^|/){}", glob_to_regex(pattern, true)),
        ),
        SignatureKind::PeVersionString => (MatchMode::PeVersionString, regex::escape(pattern)),
        SignatureKind::BinaryMarker => {
            // Only the section form reaches here: `Signature::is_matchable` routes
            // every other binary-marker row to inert before compilation.
            let name = pattern
                .strip_prefix(SECTION_MARKER_PREFIX)
                .ok_or_else(|| "binary-marker pattern is not a section marker".to_string())?;
            if name.is_empty() {
                return Err("empty section name".to_string());
            }
            (
                MatchMode::PeSectionName,
                format!("^{}$", glob_to_regex(name, false)),
            )
        }
    };
    let regex = Regex::new(&format!("(?i){body}")).map_err(|e| e.to_string())?;
    Ok((mode, regex))
}

/// Whether a file name looks like a Windows PE image (`.exe`, `.dll`, or `.sys`),
/// so a `pe-version-string` signature reads only files that could carry a version
/// resource rather than every file in the tree.
fn is_pe_image_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".exe") || lower.ends_with(".dll") || lower.ends_with(".sys")
}

/// Whether a file name is a Windows executable image (`.exe`), the candidate class
/// for a section-marker read. Narrower than [`is_pe_image_name`] on purpose: a DRM
/// wrapper is applied to the launch target, not to every library beside it.
fn is_executable_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".exe")
}

/// Read at most `limit` leading bytes of a file, or `None` if it cannot be opened or
/// read. A short file yields what it has; a large one yields exactly `limit`.
fn read_prefix(path: &Path, limit: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    let file = fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    file.take(limit as u64).read_to_end(&mut buf).ok()?;
    Some(buf)
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
                depth,
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
                depth,
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
    fn a_directory_shape_matches_only_on_a_component_boundary() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::AntiCheat,
            SignatureKind::DirectoryShape,
            "GameGuard/",
            "nProtect GameGuard",
            SignatureConfidence::Definitive,
        )]);
        // A directory whose name only ends with the pattern must not match.
        let tree = TempTree::new("notgameguard");
        tree.mkdir("NotGameGuard");
        assert!(
            set.detect(tree.path())
                .expect("readable")
                .findings
                .is_empty(),
            "NotGameGuard/ must not match GameGuard/"
        );
        // The real component does match, at the root or nested.
        let tree2 = TempTree::new("gameguard");
        tree2.mkdir("bin/GameGuard");
        assert!(set
            .detect(tree2.path())
            .expect("readable")
            .findings
            .iter()
            .any(|f| f.product == "nProtect GameGuard"));
    }

    #[test]
    fn a_nested_engine_tree_does_not_match_a_prefixed_sibling() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::DirectoryShape,
            "Engine/Binaries/",
            "Unreal",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("myengine");
        tree.mkdir("MyEngine/Binaries/Win64");
        assert!(
            set.detect(tree.path())
                .expect("readable")
                .findings
                .is_empty(),
            "MyEngine/Binaries/ must not match Engine/Binaries/"
        );
    }

    #[test]
    fn the_strongest_fidelity_row_wins_regardless_of_row_order() {
        // A heuristic shape row precedes a definitive filename row for the same
        // product; a directory carrying both must be reported verified.
        let set = SignatureSet::compile(&[
            sig(
                SignatureCategory::Engine,
                SignatureKind::DirectoryShape,
                "*_Data/",
                "Unity",
                SignatureConfidence::Heuristic,
            ),
            sig(
                SignatureCategory::Engine,
                SignatureKind::Filename,
                "UnityPlayer.dll",
                "Unity",
                SignatureConfidence::Definitive,
            ),
        ]);
        let tree = TempTree::new("unity-both");
        tree.mkdir("Game_Data");
        tree.touch("UnityPlayer.dll");
        let engine = set
            .detect(tree.path())
            .expect("readable")
            .detected_engine()
            .cloned()
            .expect("unity detected");
        assert_eq!(
            engine.fidelity,
            FidelityTier::Verified,
            "definitive wins over heuristic"
        );
        // And exactly one Unity finding, not two.
        let count = set
            .detect(tree.path())
            .expect("readable")
            .findings
            .iter()
            .filter(|f| f.product == "Unity")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn an_inert_marker_form_is_counted_not_applied() {
        let set = SignatureSet::compile(&[
            sig(
                SignatureCategory::Drm,
                SignatureKind::BinaryMarker,
                "denuvo-marker",
                "Denuvo",
                SignatureConfidence::Definitive,
            ),
            sig(
                SignatureCategory::AntiCheat,
                SignatureKind::Filename,
                "vgk.sys",
                "Vanguard",
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
    fn matchability_is_a_property_of_the_row_not_the_kind() {
        // The same kind is applied in its section form and inert in a byte form, so
        // matchability cannot be decided from the kind alone.
        let section = sig(
            SignatureCategory::Drm,
            SignatureKind::BinaryMarker,
            "section:.bind",
            "Steam DRM",
            SignatureConfidence::Definitive,
        );
        let bytes = sig(
            SignatureCategory::Drm,
            SignatureKind::BinaryMarker,
            "vmprotect-section-marker",
            "VMProtect",
            SignatureConfidence::Definitive,
        );
        assert!(section.is_matchable());
        assert!(!bytes.is_matchable());

        let set = SignatureSet::compile(&[section, bytes]);
        assert_eq!(set.applied_count(), 1);
        assert_eq!(set.inert_count(), 1);
        assert_eq!(set.skipped_count(), 0);
        assert_eq!(set.inert()[0].product, "VMProtect");
        assert_eq!(
            set.applied_count() + set.inert_count() + set.skipped_count(),
            set.total_count(),
            "the accounting sums whatever the mix of forms"
        );
    }

    #[test]
    fn a_section_marker_with_no_name_is_skipped_not_inert() {
        // `section:` is the recognized form, so the row is not inert; it is simply
        // malformed, and a malformed row is skipped and counted (P-4).
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Drm,
            SignatureKind::BinaryMarker,
            "section:",
            "Nameless",
            SignatureConfidence::Definitive,
        )]);
        assert_eq!(set.applied_count(), 0);
        assert_eq!(set.inert_count(), 0);
        assert_eq!(set.skipped_count(), 1);
        assert_eq!(set.skipped()[0].product, "Nameless");
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

    /// A `section:.bind` DRM signature, the one this project ships (slice S065).
    fn bind_signature() -> Signature {
        sig(
            SignatureCategory::Drm,
            SignatureKind::BinaryMarker,
            "section:.bind",
            "Steam DRM",
            SignatureConfidence::Definitive,
        )
    }

    #[test]
    fn a_wrapped_launch_executable_reports_the_drm_and_an_unwrapped_one_does_not() {
        let set = SignatureSet::compile(&[bind_signature()]);

        // Both trees ship the Steamworks SDK library, which is exactly the false
        // signal this slice removed: only the section table distinguishes them.
        let wrapped = TempTree::new("wrapped");
        wrapped.touch("steam_api64.dll");
        wrapped.write(
            "Game.exe",
            &pe::fixtures::minimal_pe_with_sections(&[".text", ".rdata", ".bind"]),
        );
        let outcome = set.detect(wrapped.path()).expect("readable");
        let finding = outcome
            .findings
            .iter()
            .find(|f| f.product == "Steam DRM")
            .expect("a wrapped binary reports Steam DRM");
        assert_eq!(finding.category, SignatureCategory::Drm);
        assert_eq!(finding.fidelity, FidelityTier::Verified);
        assert_eq!(
            finding.evidence, "Game.exe",
            "the evidence names the binary"
        );
        assert!(outcome.is_complete());

        let unwrapped = TempTree::new("unwrapped");
        unwrapped.touch("steam_api64.dll");
        unwrapped.write(
            "Game.exe",
            &pe::fixtures::minimal_pe_with_sections(&[".text", ".rdata", ".reloc"]),
        );
        let outcome = set.detect(unwrapped.path()).expect("readable");
        assert!(
            outcome.findings.is_empty(),
            "shipping the SDK is not DRM: {:?}",
            outcome.findings
        );
        assert!(outcome.is_complete());
    }

    #[test]
    fn a_launch_executable_at_the_unreal_depth_is_read_and_a_deeper_one_is_not() {
        let set = SignatureSet::compile(&[bind_signature()]);

        // The measured ARC Raiders layout: the shipping binary four levels down.
        let tree = TempTree::new("deep-ok");
        tree.write(
            "Pioneer/Binaries/Win64/PioneerGame.exe",
            &pe::fixtures::minimal_pe_with_sections(&[".text", ".bind"]),
        );
        assert!(
            set.detect(tree.path())
                .expect("readable")
                .findings
                .iter()
                .any(|f| f.product == "Steam DRM"),
            "an executable at depth 4 is a candidate"
        );

        // One level deeper is a redistributable installer or a tool, not a launch
        // target, and is deliberately outside the candidate set.
        let tree = TempTree::new("deep-too-far");
        tree.write(
            "a/b/c/d/Prereq.exe",
            &pe::fixtures::minimal_pe_with_sections(&[".text", ".bind"]),
        );
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(
            outcome.findings.is_empty(),
            "an executable below the depth bound is not read"
        );
        assert_eq!(
            outcome.marker_candidates_skipped, 0,
            "the depth bound defines the candidate set, so it counts no loss"
        );
        assert!(
            outcome.is_complete(),
            "a scan that excluded nothing it set out to read is complete"
        );
    }

    #[test]
    fn a_file_named_exe_that_is_not_a_pe_image_produces_no_finding_and_no_error() {
        let set = SignatureSet::compile(&[bind_signature()]);
        let tree = TempTree::new("not-a-pe");
        tree.write("Game.exe", b"this is not a PE image, it is a text file");
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(outcome.findings.is_empty());
        assert!(outcome.unreadable.is_empty(), "readable, just not a PE");
        assert!(outcome.is_complete());
    }

    #[test]
    fn candidates_beyond_the_cap_are_counted_and_make_the_scan_incomplete() {
        let set = SignatureSet::compile(&[bind_signature()]);
        let tree = TempTree::new("many-exes");
        let extra = 5;
        let total = MARKER_SCAN_MAX_CANDIDATES + extra;
        for i in 0..total {
            tree.write(
                &format!("aa-{i:04}.exe"),
                &pe::fixtures::minimal_pe_with_sections(&[".text"]),
            );
        }
        let outcome = set.detect(tree.path()).expect("readable");
        assert_eq!(
            outcome.marker_candidates_skipped, extra,
            "every candidate the cap dropped is counted (P-4)"
        );
        assert!(
            !outcome.is_complete(),
            "a truncated scan is not reported as a clean one"
        );
    }

    #[test]
    fn a_directory_named_like_an_executable_is_not_a_candidate() {
        let set = SignatureSet::compile(&[bind_signature()]);
        let tree = TempTree::new("dir-named-exe");
        tree.mkdir("Game.exe");
        tree.touch("Game.exe/inner.txt");
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(outcome.findings.is_empty());
        assert!(
            outcome.is_complete(),
            "a directory is not a file candidate at all"
        );
    }

    #[test]
    fn a_candidate_that_cannot_be_read_is_recorded_unreadable_not_scanned_clean() {
        // The edge case the spec names: a launch executable that is present but
        // cannot be opened. It must not be reported as carrying a marker, and it
        // must not be reported as having been scanned clean either. A path that
        // vanished between the walk and the read is the portable way to produce an
        // open failure on a candidate, and is also a real race on a live install.
        let set = SignatureSet::compile(&[bind_signature()]);
        let tree = TempTree::new("vanished-exe");
        let entries = vec![Entry {
            base: "Game.exe".to_string(),
            match_path: "Game.exe".to_string(),
            display: "Game.exe".to_string(),
            full: tree.path().join("Game.exe"),
            is_dir: false,
            depth: 1,
        }];
        let mut unreadable = Vec::new();
        let mut skipped = 0;
        let candidates = set.section_candidates(&entries, &mut unreadable, &mut skipped);
        assert!(
            candidates.is_empty(),
            "an unopenable candidate yields nothing"
        );
        assert_eq!(unreadable.len(), 1, "it is recorded unreadable (P-4)");
        assert_eq!(skipped, 0, "it was not dropped by the cap");

        let outcome = ScanOutcome {
            findings: Vec::new(),
            unreadable,
            marker_candidates_skipped: skipped,
        };
        assert!(
            !outcome.is_complete(),
            "unread is not the same as absent, so the scan is not clean"
        );
    }

    #[test]
    fn a_bounded_prefix_read_stops_at_the_limit() {
        let tree = TempTree::new("prefix");
        tree.write("big.bin", &vec![7u8; 4096]);
        let bytes = read_prefix(&tree.path().join("big.bin"), 100).expect("readable");
        assert_eq!(bytes.len(), 100, "the read stops at the limit");
        let all = read_prefix(&tree.path().join("big.bin"), 1_000_000).expect("readable");
        assert_eq!(all.len(), 4096, "a short file yields what it has");
        assert!(
            read_prefix(&tree.path().join("absent.bin"), 100).is_none(),
            "a missing file is None, never an empty read"
        );
    }

    #[test]
    fn no_executable_is_opened_when_no_section_rule_is_loaded() {
        // The candidate read is skipped entirely when nothing would consume it, so a
        // set with no section rule costs no file opens.
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::Filename,
            "UnityPlayer.dll",
            "Unity",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("no-section-rule");
        tree.write("Game.exe", b"not a pe");
        let outcome = set.detect(tree.path()).expect("readable");
        assert!(outcome.unreadable.is_empty());
        assert_eq!(outcome.marker_candidates_skipped, 0);
        assert!(outcome.is_complete());
    }

    #[test]
    fn the_renpy_layout_is_detected_and_deduplicates_to_one_engine() {
        // The measured Trapped with Ivy and Piper layout.
        let set = SignatureSet::compile(&[
            sig(
                SignatureCategory::Engine,
                SignatureKind::DirectoryShape,
                "renpy/",
                "Ren'Py",
                SignatureConfidence::Definitive,
            ),
            sig(
                SignatureCategory::Engine,
                SignatureKind::Filename,
                "librenpython.dll",
                "Ren'Py",
                SignatureConfidence::Definitive,
            ),
            sig(
                SignatureCategory::Engine,
                SignatureKind::Filename,
                "*.rpa",
                "Ren'Py",
                SignatureConfidence::Heuristic,
            ),
        ]);
        let tree = TempTree::new("renpy");
        tree.touch("TrappedWithIvyAndPiper-EA.exe");
        tree.touch("renpy/bootstrap.py");
        tree.touch("game/archive.rpa");
        tree.touch("lib/py3-windows-x86_64/librenpython.dll");
        let findings = set.detect(tree.path()).expect("readable").findings;
        assert_eq!(findings.len(), 1, "one engine after dedup: {findings:?}");
        assert_eq!(findings[0].product, "Ren'Py");
        assert_eq!(
            findings[0].fidelity,
            FidelityTier::Verified,
            "a definitive marker outranks the heuristic archive row"
        );
    }

    #[test]
    fn a_renpy_archive_alone_is_only_heuristic() {
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::Filename,
            "*.rpa",
            "Ren'Py",
            SignatureConfidence::Heuristic,
        )]);
        let tree = TempTree::new("rpa-only");
        tree.touch("game/archive.rpa");
        let findings = set.detect(tree.path()).expect("readable").findings;
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].fidelity, FidelityTier::HeuristicUnverified);
    }

    #[test]
    fn the_gamemaker_layout_is_detected() {
        // The measured Shale Hill Secrets layout.
        let set = SignatureSet::compile(&[
            sig(
                SignatureCategory::Engine,
                SignatureKind::Filename,
                "data.win",
                "GameMaker",
                SignatureConfidence::Definitive,
            ),
            sig(
                SignatureCategory::Engine,
                SignatureKind::Filename,
                "Steamworks_x64.dll",
                "GameMaker",
                SignatureConfidence::Heuristic,
            ),
        ]);
        let tree = TempTree::new("gamemaker");
        tree.touch("Shale Hill Secrets.exe");
        tree.touch("data.win");
        tree.touch("Steamworks_x64.dll");
        tree.touch("steam_api64.dll");
        let findings = set.detect(tree.path()).expect("readable").findings;
        assert_eq!(findings.len(), 1, "one engine after dedup: {findings:?}");
        assert_eq!(findings[0].product, "GameMaker");
        assert_eq!(findings[0].fidelity, FidelityTier::Verified);
    }

    #[test]
    fn a_filename_glob_does_not_treat_a_dot_as_a_wildcard() {
        // `data.win` must not match `dataXwin`: the glob compiler escapes the dot.
        let set = SignatureSet::compile(&[sig(
            SignatureCategory::Engine,
            SignatureKind::Filename,
            "data.win",
            "GameMaker",
            SignatureConfidence::Definitive,
        )]);
        let tree = TempTree::new("dotglob");
        tree.touch("dataXwin");
        assert!(set
            .detect(tree.path())
            .expect("readable")
            .findings
            .is_empty());
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
        let bytes = crate::pe::fixtures::minimal_pe_with_version_string("ProductName", "Frostbite");
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
