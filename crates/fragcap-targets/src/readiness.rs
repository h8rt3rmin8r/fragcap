// SPDX-License-Identifier: Apache-2.0

//! Presentational derivations for the hero listing (slice S055).
//!
//! Both values are derived from a [`TargetEntry`] at listing time and stored
//! nowhere. The CAPTURE column reports how close a row is to a capture, never
//! whether it is valid: every registered row is capturable in principle
//! (specification section 3.6). The KNOWN column is neutral evidence, never a
//! blocker or an endorsement (P-9).

use crate::entry::TargetEntry;
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
/// one Windows client (the S054 reduction) or carries a non-empty anchor a
/// `capture --target` can resolve; `NeedsTarget` otherwise. Derived, never stored.
pub fn capture_readiness(entry: &TargetEntry) -> CaptureReadiness {
    let has_client = !entry_windows_clients(entry).is_empty();
    let has_anchor = entry.anchor.as_deref().is_some_and(|a| !a.is_empty());
    if has_client || has_anchor {
        CaptureReadiness::Ready
    } else {
        CaptureReadiness::NeedsTarget
    }
}

/// Derive the neutral KNOWN evidence summary. In order: the distinct evidence
/// products (engine / anti-cheat / DRM, e.g. "Denuvo, EasyAntiCheat"); else, for a
/// row that is ready but carries no such evidence, "no online mode recorded"; else
/// "no launch data known". Phrasing is neutral by construction (P-9, FR-021).
pub fn known_summary(entry: &TargetEntry) -> String {
    let products = evidence_products(entry);
    if !products.is_empty() {
        return products.join(", ");
    }
    match capture_readiness(entry) {
        CaptureReadiness::Ready => "no online mode recorded".to_string(),
        CaptureReadiness::NeedsTarget => "no launch data known".to_string(),
    }
}

/// The distinct product names an entry's `evidence` JSON carries. Evidence is an
/// array of finding objects each with a `product` string (the serialized detection
/// findings, slice S053). A malformed or absent value yields none. Order is
/// first-seen; duplicates fold out.
fn evidence_products(entry: &TargetEntry) -> Vec<String> {
    let Some(evidence) = &entry.evidence else {
        return Vec::new();
    };
    let Some(items) = evidence.as_array() else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for item in items {
        if let Some(product) = item.get("product").and_then(|v| v.as_str()) {
            if !product.is_empty() && !out.iter().any(|p| p == product) {
                out.push(product.to_string());
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
    fn an_unresolved_entry_with_no_anchor_needs_a_target() {
        let e = entry(None, Some(json!({ "socket_holder": "unresolved" })), None);
        assert_eq!(capture_readiness(&e), CaptureReadiness::NeedsTarget);
        let bare = entry(None, None, None);
        assert_eq!(capture_readiness(&bare), CaptureReadiness::NeedsTarget);
    }

    #[test]
    fn known_lists_evidence_products() {
        let e = entry(
            Some("steam:1"),
            None,
            Some(json!([
                { "category": "drm", "product": "Denuvo" },
                { "category": "anti-cheat", "product": "EasyAntiCheat" },
                { "category": "drm", "product": "Denuvo" }
            ])),
        );
        assert_eq!(known_summary(&e), "Denuvo, EasyAntiCheat");
    }

    #[test]
    fn known_falls_back_by_readiness() {
        let ready = entry(None, Some(json!([{ "executable": "game.exe" }])), None);
        assert_eq!(known_summary(&ready), "no online mode recorded");
        let needs = entry(None, None, None);
        assert_eq!(known_summary(&needs), "no launch data known");
    }
}
