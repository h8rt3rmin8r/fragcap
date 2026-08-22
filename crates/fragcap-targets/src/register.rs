// SPDX-License-Identifier: Apache-2.0

//! Registering discovery candidates as target entries (slice S055).
//!
//! Every source (Steam walk, known-roots walk, a directory scan, the hero
//! listing's discovery pass) becomes a registered [`TargetEntry`] through this one
//! operation, so there is a single creation path and a single stored form (P-10).
//! Registration is idempotent: a candidate whose identity is already registered is
//! skipped, not duplicated. A Steam candidate dedups on its anchor; a path
//! candidate dedups on its install root.

use serde_json::{json, Value};

use crate::entry::{ClassificationSource, TargetEntry};
use crate::source::{CandidateIdentity, CandidateTarget};
use crate::store::Store;
use crate::{handle, identifier, TargetsError};

/// The result of registering a batch of candidates.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegistrationOutcome {
    /// Candidates newly inserted as target entries.
    pub registered: usize,
    /// Candidates already registered (skipped, not duplicated).
    pub already_present: usize,
}

/// Register every candidate, idempotently. Returns the conserved account of how
/// many were newly registered versus already present, so a caller can surface both
/// (P-4). Existing entries are never modified or removed.
pub fn register_candidates(
    store: &mut Store,
    candidates: &[CandidateTarget],
) -> Result<RegistrationOutcome, TargetsError> {
    let mut outcome = RegistrationOutcome::default();
    for candidate in candidates {
        if register_candidate(store, candidate)? {
            outcome.registered += 1;
        } else {
            outcome.already_present += 1;
        }
    }
    Ok(outcome)
}

/// Register one candidate. Returns whether it was newly registered (`false` means
/// it was already present). The stored entry carries the candidate's classification,
/// fidelity, and detected evidence; its launch chain is left unresolved (a
/// `capture --target` resolves it from the anchor or install root, or a capture
/// observes and promotes it). Nothing here fabricates a socket holder (P-9).
pub fn register_candidate(
    store: &mut Store,
    candidate: &CandidateTarget,
) -> Result<bool, TargetsError> {
    let anchor = match &candidate.identity {
        CandidateIdentity::SteamAppId(appid) => {
            Some(identifier::canonicalize_anchor(&format!("steam:{appid}")))
        }
        CandidateIdentity::Path(_) => None,
    };
    // Carried explicitly by the candidate rather than derived from `identity`: a
    // Steam candidate's identity is its app id, not a path, so deriving from
    // identity alone left every Steam-sourced registration with no install_root at
    // all (review of PR #193, issue #167).
    let install_root = candidate.install_root.clone();

    // Read the table once and reuse it for both the install-root dedup and the
    // handle-derivation index, so registering a discovery set is one table read per
    // candidate rather than two.
    let existing = store.targets()?;

    // Idempotency. An anchored identity is deterministic, so an existing anchor is a
    // duplicate. A path candidate has no durable id, so dedup on its install root.
    if let Some(anchor) = &anchor {
        if store.target_by_anchor(anchor)?.is_some() {
            return Ok(false);
        }
    } else if let Some(root) = &install_root {
        if existing
            .iter()
            .any(|t| t.install_root.as_deref() == Some(root.as_str()))
        {
            return Ok(false);
        }
    }

    let exe_stem = match &candidate.identity {
        CandidateIdentity::Path(path) => path_stem(path),
        CandidateIdentity::SteamAppId(_) => None,
    };
    let index = existing.len() as u64 + 1;
    let base = handle::derive_handle(&candidate.display_name, exe_stem.as_deref(), index);
    let handle_value = handle::disambiguate(&base, |h| store.handle_exists(h))?;

    let stable_id = match &anchor {
        Some(a) => identifier::anchored_id(a),
        None => identifier::unanchored_id(),
    };

    let entry = TargetEntry {
        id: None,
        stable_id,
        handle: handle_value,
        name: candidate.display_name.clone(),
        classification: candidate.classification,
        classification_source: ClassificationSource::Platform,
        fidelity: candidate.fidelity,
        provenance: Some(json!({ "source": candidate.source_name })),
        anchor,
        launch_entries: None,
        install_root,
        evidence: evidence_value(candidate),
        detection_scan: candidate.detection_scan,
        folder_name: candidate.folder_name.clone(),
        executable_hint: candidate.executable_hint.clone(),
    };
    store.insert_target(&entry)?;
    Ok(true)
}

/// Serialize a candidate's detection findings to the `evidence` JSON the KNOWN
/// column and export read: an array of `{category, product, evidence, fidelity}`
/// objects, or `None` when the source ran no detection.
fn evidence_value(candidate: &CandidateTarget) -> Option<Value> {
    if candidate.evidence.is_empty() {
        return None;
    }
    let findings: Vec<Value> = candidate
        .evidence
        .iter()
        .map(|f| {
            json!({
                "category": f.category.as_str(),
                "product": f.product,
                "evidence": f.evidence,
                "fidelity": f.fidelity.as_str(),
            })
        })
        .collect();
    Some(Value::Array(findings))
}

/// The file stem of a path, used as a handle-derivation hint. A directory path
/// yields its last component; an executable yields its name without extension.
fn path_stem(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::TargetClassification;
    use crate::source::CandidateTarget;
    use fragcap_profile::FidelityTier;

    fn steam_candidate(appid: u32, name: &str) -> CandidateTarget {
        CandidateTarget {
            identity: CandidateIdentity::SteamAppId(appid),
            display_name: name.to_string(),
            fidelity: FidelityTier::Observed,
            classification: TargetClassification::Game,
            evidence: Vec::new(),
            detection_scan: None,
            source_name: "steam".to_string(),
            install_root: Some(format!("C:/Games/Steam/steamapps/common/{name}")),
            folder_name: None,
            executable_hint: None,
        }
    }

    fn path_candidate(path: &str, name: &str) -> CandidateTarget {
        CandidateTarget {
            identity: CandidateIdentity::Path(path.to_string()),
            display_name: name.to_string(),
            fidelity: FidelityTier::HeuristicUnverified,
            classification: TargetClassification::Game,
            evidence: Vec::new(),
            detection_scan: None,
            source_name: "known-roots".to_string(),
            install_root: Some(path.to_string()),
            folder_name: None,
            executable_hint: None,
        }
    }

    #[test]
    fn registering_a_candidate_is_idempotent_by_anchor() {
        let mut store = Store::open_in_memory().expect("store");
        assert!(register_candidate(&mut store, &steam_candidate(620, "Portal 2")).expect("first"));
        assert!(
            !register_candidate(&mut store, &steam_candidate(620, "Portal 2")).expect("second"),
            "the same app id does not register twice"
        );
        assert_eq!(store.targets().expect("targets").len(), 1);
    }

    #[test]
    fn a_steam_candidates_install_root_is_stored_not_dropped() {
        // Review of PR #193: a Steam candidate's identity is its app id, not a
        // path, so install_root must be carried explicitly by the candidate
        // rather than derived from identity, or the missing-install-root
        // detection (issue #167) could never fire for a Steam-sourced target.
        let mut store = Store::open_in_memory().expect("store");
        register_candidate(&mut store, &steam_candidate(620, "Portal 2")).expect("register");
        let entry = &store.targets().expect("targets")[0];
        assert_eq!(
            entry.install_root.as_deref(),
            Some("C:/Games/Steam/steamapps/common/Portal 2")
        );
    }

    #[test]
    fn registering_a_path_candidate_dedups_on_install_root() {
        let mut store = Store::open_in_memory().expect("store");
        let outcome =
            register_candidates(&mut store, &[path_candidate("C:/Games/Foo", "Foo")]).expect("reg");
        assert_eq!(outcome.registered, 1);
        let again =
            register_candidates(&mut store, &[path_candidate("C:/Games/Foo", "Foo")]).expect("reg");
        assert_eq!(again.registered, 0);
        assert_eq!(again.already_present, 1);
        assert_eq!(store.targets().expect("targets").len(), 1);
    }

    #[test]
    fn folder_name_and_executable_hint_are_stored_verbatim_and_never_fabricated() {
        let mut store = Store::open_in_memory().expect("store");
        let mut candidate = steam_candidate(2413210, "Trapped with Ivy & Piper");
        candidate.folder_name = Some("Escape from Ivy & Piper".to_string());
        candidate.executable_hint = Some("TrappedWithIvyAndPiper-EA.exe".to_string());
        register_candidate(&mut store, &candidate).expect("register");

        let entry = &store.targets().expect("targets")[0];
        assert_eq!(
            entry.folder_name.as_deref(),
            Some("Escape from Ivy & Piper")
        );
        assert_eq!(
            entry.executable_hint.as_deref(),
            Some("TrappedWithIvyAndPiper-EA.exe")
        );

        // A candidate with neither observed leaves both None, never invented.
        let mut store2 = Store::open_in_memory().expect("store");
        register_candidate(&mut store2, &steam_candidate(730, "Counter-Strike 2"))
            .expect("register");
        let entry2 = &store2.targets().expect("targets")[0];
        assert_eq!(entry2.folder_name, None);
        assert_eq!(entry2.executable_hint, None);
    }
}
