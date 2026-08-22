// SPDX-License-Identifier: Apache-2.0

//! Presentational derivations for the hero listing (slice S055, split in S065).
//!
//! Every value here is derived from a [`TargetEntry`] at listing time and stored
//! nowhere. The CAPTURE column reports how close a row is to a capture, never
//! whether it is valid: every registered row is capturable in principle
//! (specification section 3.6). The ENGINE and SENSITIVITIES columns are neutral
//! evidence, never a blocker or an endorsement (P-9).
//!
//! # One column, one kind of fact
//!
//! S055 rendered a single KNOWN column that comma-joined every detected product
//! regardless of category, so an engine and a protection product read as one
//! quality, and substituted a sentence about capture readiness when it had neither.
//! S065 splits it on the category the findings already carry: engines in one column,
//! anti-cheat and DRM in the other. The two fallback sentences are gone rather than
//! relocated, because each was a relabeling of a readiness state the CAPTURE column
//! already prints.
//!
//! # A blank is three different facts
//!
//! A row scanned clean, a row never scanned, and a row whose scan could not complete
//! rendered identically before S065. They are different claims and one of them is
//! not a claim at all, so each gets its own marker. Collapsing them is the silent
//! loss P-4 forbids: an operator reading a blank engine column cannot otherwise tell
//! "this title has no detectable engine" from "nobody looked".

use fragcap_profile::SignatureCategory;

use crate::entry::{DetectionScan, TargetEntry};
use crate::hint_provider::entry_windows_clients;

/// Whether a target is capturable now, or still needs launch information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureReadiness {
    /// A capture can resolve a client for this row: it names at least one Windows
    /// client executable, or it carries an anchor a `capture --target` resolves.
    Ready,
    /// The launch chain is unresolved and no anchor gives a client; a capture must
    /// observe and promote the row first.
    NeedsTarget,
}

impl CaptureReadiness {
    /// The CAPTURE column text.
    pub fn label(self) -> &'static str {
        match self {
            CaptureReadiness::Ready => "ready",
            CaptureReadiness::NeedsTarget => "needs a target",
        }
    }
}

/// Derive the CAPTURE readiness of a target. `Ready` when the entry names at least
/// one Windows client (the S054 reduction) or carries an anchor `capture --target`
/// can actually resolve; `NeedsTarget` otherwise. Derived, never stored.
///
/// Only a `steam:` anchor is treated as resolvable, matching what `capture --target`
/// resolves through the install-layout cascade: an anchor of any other form (say
/// `epic:foo`) that names no client would otherwise be listed `ready` and offered as
/// the next command, then fail at capture because it resolves no Windows client.
pub fn capture_readiness(entry: &TargetEntry) -> CaptureReadiness {
    let has_client = !entry_windows_clients(entry).is_empty();
    let has_resolvable_anchor = entry
        .anchor
        .as_deref()
        .is_some_and(|a| a.starts_with("steam:"));
    if has_client || has_resolvable_anchor {
        CaptureReadiness::Ready
    } else {
        CaptureReadiness::NeedsTarget
    }
}

/// Whether a target's recorded install root still exists on disk (slice S066, issue
/// #167). Derived fresh at listing time, never stored: a reconnected drive or a
/// restored folder must read as `Present` again on the very next listing, with no
/// stale verdict surviving from an earlier one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPresence {
    /// `install_root` is recorded and exists.
    Present,
    /// `install_root` is recorded and does not exist.
    Missing,
    /// No `install_root` is recorded at all; distinct from `Missing` (absence and
    /// "never recorded" are different facts, FR-008).
    NotRecorded,
}

/// Derive a target's [`InstallPresence`]. Reads the filesystem once, at call time;
/// never mutates the entry (FR-010, SC-005).
pub fn install_presence(entry: &TargetEntry) -> InstallPresence {
    match &entry.install_root {
        None => InstallPresence::NotRecorded,
        Some(root) => {
            if std::path::Path::new(root).exists() {
                InstallPresence::Present
            } else {
                InstallPresence::Missing
            }
        }
    }
}

/// The short note the missing-install-root state prints (issue #167).
pub const INSTALL_MISSING_NOTE: &str = "install folder not found";

/// Whether a target's display name and its recorded folder name name the same
/// title, differ only cosmetically, or genuinely diverge (slice S066, issue #173).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameDivergence {
    /// No `folder_name` is recorded, so there is nothing to compare (FR-017).
    None,
    /// The two names are identical after normalization, or one is a substring of
    /// the other (a casing/whitespace difference, or a subtitle-stripped or
    /// prefix-truncated form).
    Cosmetic,
    /// The two names are neither equal after normalization nor a substring of one
    /// another: a genuinely different title, worth surfacing (FR-014).
    Semantic,
}

/// Derive a target's [`NameDivergence`] between `name` and `folder_name`, reusing
/// `crate::handle::normalize` (the same fold a handle is already derived through)
/// rather than a second string-comparison rule set.
///
/// A name `normalize` declines (purely numeric, or nothing left after stripping
/// decorative symbols) folds back to its own lowercased, trimmed form rather than
/// to an empty string. Collapsing every unnormalizable name to `""` would make two
/// distinct such names (`"123"` and `"456"`, or two different symbol-only titles)
/// compare equal and read as merely cosmetic, hiding a real divergence the
/// "unnormalizable" case has no more business asserting away than any other name
/// does (review of PR #193).
pub fn name_divergence(entry: &TargetEntry) -> NameDivergence {
    let Some(folder_name) = &entry.folder_name else {
        return NameDivergence::None;
    };
    let fold = |s: &str| crate::handle::normalize(s).unwrap_or_else(|| s.trim().to_lowercase());
    let a = fold(&entry.name);
    let b = fold(folder_name);
    // `str::contains("")` is vacuously true, so an empty fold (a whitespace-only
    // name normalize declined) must not let a real name "contain" it by accident;
    // only a genuine, non-empty substring relationship counts as cosmetic.
    let substring_match = !a.is_empty() && !b.is_empty() && (a.contains(&b) || b.contains(&a));
    if a == b || substring_match {
        NameDivergence::Cosmetic
    } else {
        NameDivergence::Semantic
    }
}

/// The marker a technology column carries when it has no products: a complete scan
/// that matched nothing.
pub const SCANNED_CLEAN_MARKER: &str = "-";

/// The marker a technology column carries when a scan ran but could not cover
/// everything: something was unreadable, or a scan bound truncated the candidate
/// set. Distinct from [`SCANNED_CLEAN_MARKER`] because a partial answer must not
/// read as a clean one (P-4). The specific path or bound is named in the discovery
/// warnings, which is where a recoverable detail belongs.
pub const SCAN_INCOMPLETE_MARKER: &str = "incomplete";

/// The marker a technology column carries when no scan is recorded for the row at
/// all: a source that ran no detection produced it, or it predates the coverage
/// record. Distinct from [`SCANNED_CLEAN_MARKER`] because "nobody looked" is not
/// "nothing is there" (P-9).
pub const NOT_SCANNED_MARKER: &str = "not scanned";

/// Derive the ENGINE column: the distinct detected engine products, else the
/// coverage marker for the row.
///
/// Neutral by construction: an engine is a fact about the title, never a judgment
/// about it (P-9, specification section 3.6).
pub fn engine_summary(entry: &TargetEntry) -> String {
    summarize(entry, &[SignatureCategory::Engine])
}

/// Derive the SENSITIVITIES column: the distinct detected anti-cheat and DRM
/// products, else the coverage marker for the row.
///
/// Anti-cheat precedes DRM, following the declared category display order, so the
/// column reads the same way for every row. A detected product is neutral evidence:
/// nothing here characterizes a title as off limits (specification section 3.6).
pub fn sensitivities_summary(entry: &TargetEntry) -> String {
    summarize(
        entry,
        &[SignatureCategory::AntiCheat, SignatureCategory::Drm],
    )
}

/// The coverage marker for a row: what a technology column says when it has no
/// products to name.
fn coverage_marker(entry: &TargetEntry) -> &'static str {
    match entry.detection_scan {
        Some(DetectionScan::Complete) => SCANNED_CLEAN_MARKER,
        Some(DetectionScan::Incomplete) => SCAN_INCOMPLETE_MARKER,
        None => NOT_SCANNED_MARKER,
    }
}

/// The distinct products in `categories`, joined, or the coverage marker when there
/// are none.
fn summarize(entry: &TargetEntry, categories: &[SignatureCategory]) -> String {
    let products = evidence_products(entry, categories);
    if products.is_empty() {
        return coverage_marker(entry).to_string();
    }
    products.join(", ")
}

/// The distinct product names an entry carries in its `evidence` JSON, restricted
/// to the requested categories, in category order then first-seen order.
///
/// Evidence is an array of finding objects each with a `category` and a `product`
/// string (the serialized detection findings, slice S053). A finding whose category
/// is absent or unrecognized belongs to no requested category and is therefore not
/// rendered in either column; that is deliberate, because guessing which column an
/// unknown category belongs in would put a product under a heading it does not
/// answer to (P-9). A malformed or absent evidence value yields none.
fn evidence_products(entry: &TargetEntry, categories: &[SignatureCategory]) -> Vec<String> {
    let Some(evidence) = &entry.evidence else {
        return Vec::new();
    };
    let Some(items) = evidence.as_array() else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for wanted in categories {
        for item in items {
            let category = item
                .get("category")
                .and_then(|v| v.as_str())
                .and_then(SignatureCategory::parse);
            if category != Some(*wanted) {
                continue;
            }
            if let Some(product) = item.get("product").and_then(|v| v.as_str()) {
                if !product.is_empty() && !out.iter().any(|p| p == product) {
                    out.push(product.to_string());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{ClassificationSource, TargetClassification};
    use fragcap_profile::FidelityTier;
    use serde_json::json;

    fn entry(
        anchor: Option<&str>,
        launch_entries: Option<serde_json::Value>,
        evidence: Option<serde_json::Value>,
    ) -> TargetEntry {
        TargetEntry {
            id: None,
            stable_id: 1,
            handle: "some_game".to_string(),
            name: "Some Game".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: anchor.map(str::to_string),
            launch_entries,
            install_root: None,
            evidence,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        }
    }

    #[test]
    fn a_resolved_client_is_ready() {
        let e = entry(None, Some(json!([{ "executable": "game.exe" }])), None);
        assert_eq!(capture_readiness(&e), CaptureReadiness::Ready);
    }

    #[test]
    fn a_steam_anchor_alone_is_ready() {
        let e = entry(Some("steam:620"), None, None);
        assert_eq!(capture_readiness(&e), CaptureReadiness::Ready);
    }

    #[test]
    fn a_non_steam_anchor_with_no_client_needs_a_target() {
        // Only a `steam:` anchor is resolvable by capture; an anchor of another form
        // that names no client must not be listed ready and offered as next command.
        let e = entry(Some("epic:foo"), None, None);
        assert_eq!(capture_readiness(&e), CaptureReadiness::NeedsTarget);
    }

    #[test]
    fn an_unresolved_entry_with_no_anchor_needs_a_target() {
        let e = entry(None, Some(json!({ "socket_holder": "unresolved" })), None);
        assert_eq!(capture_readiness(&e), CaptureReadiness::NeedsTarget);
        let bare = entry(None, None, None);
        assert_eq!(capture_readiness(&bare), CaptureReadiness::NeedsTarget);
    }

    #[test]
    fn an_engine_and_a_protection_product_never_share_a_column() {
        let mut e = entry(
            Some("steam:1"),
            None,
            Some(json!([
                { "category": "engine", "product": "Unreal" },
                { "category": "drm", "product": "Denuvo" },
                { "category": "anti-cheat", "product": "Easy Anti-Cheat" },
                { "category": "drm", "product": "Denuvo" }
            ])),
        );
        e.detection_scan = Some(DetectionScan::Complete);

        assert_eq!(engine_summary(&e), "Unreal");
        // Anti-cheat precedes DRM, per the declared category display order, and the
        // duplicate DRM row folds out.
        assert_eq!(sensitivities_summary(&e), "Easy Anti-Cheat, Denuvo");

        assert!(
            !engine_summary(&e).contains("Denuvo") && !engine_summary(&e).contains("Anti-Cheat"),
            "the engine column carries no protection product"
        );
        assert!(
            !sensitivities_summary(&e).contains("Unreal"),
            "the sensitivities column carries no engine"
        );
    }

    #[test]
    fn a_column_with_no_products_carries_the_rows_coverage_marker() {
        // Scanned clean: a real answer.
        let mut clean = entry(Some("steam:1"), None, Some(json!([])));
        clean.detection_scan = Some(DetectionScan::Complete);
        assert_eq!(engine_summary(&clean), SCANNED_CLEAN_MARKER);
        assert_eq!(sensitivities_summary(&clean), SCANNED_CLEAN_MARKER);

        // Scanned, coverage reduced: not an answer.
        let mut partial = entry(Some("steam:1"), None, Some(json!([])));
        partial.detection_scan = Some(DetectionScan::Incomplete);
        assert_eq!(engine_summary(&partial), SCAN_INCOMPLETE_MARKER);

        // Never scanned: not even an attempt.
        let unscanned = entry(Some("steam:1"), None, None);
        assert_eq!(unscanned.detection_scan, None);
        assert_eq!(engine_summary(&unscanned), NOT_SCANNED_MARKER);

        // The three must be mutually distinguishable, which is the whole point.
        let markers = [
            engine_summary(&clean),
            engine_summary(&partial),
            engine_summary(&unscanned),
        ];
        let mut unique = markers.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 3, "three states, three markers: {markers:?}");
    }

    #[test]
    fn one_populated_column_does_not_suppress_the_other_columns_marker() {
        // A title with an engine and no protection product: the engine column names
        // the engine, and the sensitivities column still reports coverage rather
        // than going blank.
        let mut e = entry(
            Some("steam:1"),
            None,
            Some(json!([{ "category": "engine", "product": "GameMaker" }])),
        );
        e.detection_scan = Some(DetectionScan::Complete);
        assert_eq!(engine_summary(&e), "GameMaker");
        assert_eq!(sensitivities_summary(&e), SCANNED_CLEAN_MARKER);
    }

    #[test]
    fn a_finding_with_an_unknown_category_is_rendered_in_neither_column() {
        // Guessing a column for a category this build does not know would file a
        // product under a heading it does not answer to (P-9).
        let mut e = entry(
            Some("steam:1"),
            None,
            Some(json!([{ "category": "platform-sdk", "product": "Steamworks SDK" }])),
        );
        e.detection_scan = Some(DetectionScan::Complete);
        assert_eq!(engine_summary(&e), SCANNED_CLEAN_MARKER);
        assert_eq!(sensitivities_summary(&e), SCANNED_CLEAN_MARKER);
    }

    #[test]
    fn the_retired_readiness_sentences_appear_in_no_summary() {
        // They were relabelings of the two CAPTURE labels and are gone, not moved.
        // Rendering them beside `ready` or `needs a target` would state one fact
        // twice in one row.
        for e in [
            entry(None, Some(json!([{ "executable": "game.exe" }])), None),
            entry(None, None, None),
        ] {
            for text in [engine_summary(&e), sensitivities_summary(&e)] {
                assert!(!text.contains("no online mode recorded"), "{text}");
                assert!(!text.contains("no launch data known"), "{text}");
            }
        }
    }

    #[test]
    fn the_readiness_labels_are_unchanged_by_the_split() {
        // The split cost the readiness column no width: it still says exactly what
        // it said, in one place, which is what keeps the widened row inside 80
        // columns.
        assert_eq!(CaptureReadiness::Ready.label(), "ready");
        assert_eq!(CaptureReadiness::NeedsTarget.label(), "needs a target");
    }

    #[test]
    fn an_apostrophe_bearing_product_renders_whole() {
        let mut e = entry(
            Some("steam:1"),
            None,
            Some(json!([{ "category": "engine", "product": "Ren'Py" }])),
        );
        e.detection_scan = Some(DetectionScan::Complete);
        assert_eq!(engine_summary(&e), "Ren'Py");
    }

    #[test]
    fn install_presence_distinguishes_present_missing_and_not_recorded() {
        let tmp = std::env::temp_dir();
        let mut e = entry(None, None, None);

        e.install_root = None;
        assert_eq!(install_presence(&e), InstallPresence::NotRecorded);

        e.install_root = Some(
            tmp.join("fragcap-s066-does-not-exist")
                .display()
                .to_string(),
        );
        assert_eq!(install_presence(&e), InstallPresence::Missing);

        e.install_root = Some(tmp.display().to_string());
        assert_eq!(install_presence(&e), InstallPresence::Present);
    }

    #[test]
    fn install_presence_never_mutates_the_entry() {
        // SC-005/FR-010: computing presence is a pure read. Insert, compute, and
        // re-read from the store to prove nothing was written back.
        let mut store = crate::store::Store::open_in_memory().expect("store");
        let mut e = entry(None, None, None);
        e.install_root = Some("C:/does/not/exist".to_string());
        e.handle = "presence_check".to_string();
        store.insert_target(&e).expect("insert");
        let before = store
            .target_by_handle("presence_check")
            .expect("query")
            .expect("present");

        let _ = install_presence(&before);

        let after = store
            .target_by_handle("presence_check")
            .expect("query")
            .expect("present");
        assert_eq!(
            before, after,
            "a presence check never writes back to the store"
        );
    }

    #[test]
    fn name_divergence_distinguishes_none_cosmetic_and_semantic() {
        let mut e = entry(None, None, None);

        // No folder_name recorded: nothing to compare.
        e.folder_name = None;
        assert_eq!(name_divergence(&e), NameDivergence::None);

        // Identical after normalization (casing/whitespace only).
        e.name = "The Division 2".to_string();
        e.folder_name = Some("the division 2".to_string());
        assert_eq!(name_divergence(&e), NameDivergence::Cosmetic);

        // Truncation: one is a substring of the other.
        e.name = "The Elder Scrolls IV: Oblivion Remastered".to_string();
        e.folder_name = Some("Oblivion Remastered".to_string());
        assert_eq!(name_divergence(&e), NameDivergence::Cosmetic);

        // Semantic: neither equal nor a substring of the other.
        e.name = "Trapped with Ivy & Piper".to_string();
        e.folder_name = Some("Escape from Ivy & Piper".to_string());
        assert_eq!(name_divergence(&e), NameDivergence::Semantic);
    }

    #[test]
    fn name_divergence_does_not_collapse_distinct_unnormalizable_names() {
        // Review of PR #193 (Copilot, suppressed comment on readiness.rs:135):
        // both "123" and "456" decline to normalize (purely numeric); folding both
        // to "" via unwrap_or_default would make them compare equal and hide a
        // real, semantic divergence. Two different such names must still read as
        // Semantic.
        let mut e = entry(None, None, None);
        e.name = "123".to_string();
        e.folder_name = Some("456".to_string());
        assert_eq!(name_divergence(&e), NameDivergence::Semantic);

        // The same unnormalizable value on both sides is genuinely identical.
        e.name = "123".to_string();
        e.folder_name = Some("123".to_string());
        assert_eq!(name_divergence(&e), NameDivergence::Cosmetic);

        // A whitespace-only name (normalize declines to None, and trims to "")
        // must not vacuously "contain" a real folder name via `str::contains("")`.
        e.name = "  ".to_string();
        e.folder_name = Some("Real Folder Name".to_string());
        assert_eq!(name_divergence(&e), NameDivergence::Semantic);
    }
}
