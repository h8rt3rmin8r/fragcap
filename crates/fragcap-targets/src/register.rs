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
use fragcap_profile::{FidelityTier, SignatureCategory};

/// Why a discovery candidate may be persisted without an explicit user choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomaticRegistrationBasis {
    /// A platform source supplied its durable application identity.
    AuthoritativePlatformIdentity,
    /// A local path carries positive, verified engine evidence.
    VerifiedEngineEvidence,
}

impl AutomaticRegistrationBasis {
    /// Stable CLI spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthoritativePlatformIdentity => "authoritative-platform-identity",
            Self::VerifiedEngineEvidence => "verified-engine-evidence",
        }
    }
}

/// Why a discovered candidate is visible but withheld from automatic persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomaticRegistrationRefusal {
    /// The candidate has neither a platform identity nor verified engine evidence.
    InsufficientTitleEvidence,
}

impl AutomaticRegistrationRefusal {
    /// Stable CLI spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InsufficientTitleEvidence => "insufficient-title-evidence",
        }
    }
}

/// The automatic-registration decision for one discovery candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomaticRegistrationDecision {
    /// Safe for automatic persistence for the named reason.
    Eligible(AutomaticRegistrationBasis),
    /// Available to explicit discovery but withheld from automatic persistence.
    Refused(AutomaticRegistrationRefusal),
}

/// One withheld candidate and the exact refusal reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusedAutomaticCandidate {
    /// The original candidate, unchanged.
    pub candidate: CandidateTarget,
    /// The policy reason that withheld it.
    pub reason: AutomaticRegistrationRefusal,
}

/// Conserved partition of one produced discovery set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AutomaticRegistrationPlan {
    /// Candidates supplied by discovery.
    pub produced: usize,
    /// Candidates admitted to the single registration operation.
    pub accepted: Vec<CandidateTarget>,
    /// Candidates retained only for explicit discovery.
    pub refused: Vec<RefusedAutomaticCandidate>,
}

/// Conserved result of policy admission followed by idempotent registration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutomaticRegistrationOutcome {
    /// Candidates evaluated.
    pub produced: usize,
    /// Candidates admitted by the precision policy.
    pub eligible: usize,
    /// Candidates withheld from automatic persistence.
    pub refused: usize,
    /// Eligible candidates newly inserted.
    pub registered: usize,
    /// Eligible candidates already represented.
    pub already_present: usize,
}

impl AutomaticRegistrationOutcome {
    /// Both policy and registration partitions reconcile exactly.
    pub fn is_conserved(&self) -> bool {
        self.produced == self.eligible + self.refused
            && self.eligible == self.registered + self.already_present
    }
}

impl AutomaticRegistrationPlan {
    /// Total candidates decided by the policy.
    pub fn considered(&self) -> usize {
        self.produced
    }

    /// Whether every input candidate reached exactly one decision.
    pub fn is_conserved(&self) -> bool {
        self.produced == self.accepted.len() + self.refused.len()
    }
}

/// Decide whether one produced candidate is safe to persist automatically.
///
/// Location, source membership, classification, anti-cheat, and DRM evidence do
/// not grant admission. Steam owns its durable application identities; local path
/// candidates need positive engine evidence at verified-or-stronger fidelity.
pub fn automatic_registration_decision(
    candidate: &CandidateTarget,
) -> AutomaticRegistrationDecision {
    if matches!(candidate.identity, CandidateIdentity::SteamAppId(_))
        && candidate.source_name == "steam"
    {
        return AutomaticRegistrationDecision::Eligible(
            AutomaticRegistrationBasis::AuthoritativePlatformIdentity,
        );
    }

    if matches!(candidate.identity, CandidateIdentity::Path(_))
        && candidate.evidence.iter().any(|finding| {
            finding.category == SignatureCategory::Engine
                && finding.fidelity >= FidelityTier::Verified
        })
    {
        return AutomaticRegistrationDecision::Eligible(
            AutomaticRegistrationBasis::VerifiedEngineEvidence,
        );
    }

    AutomaticRegistrationDecision::Refused(AutomaticRegistrationRefusal::InsufficientTitleEvidence)
}

/// Partition a discovery set through [`automatic_registration_decision`].
pub fn automatic_registration_plan(candidates: &[CandidateTarget]) -> AutomaticRegistrationPlan {
    let mut plan = AutomaticRegistrationPlan {
        produced: candidates.len(),
        ..AutomaticRegistrationPlan::default()
    };
    for candidate in candidates {
        match automatic_registration_decision(candidate) {
            AutomaticRegistrationDecision::Eligible(_) => plan.accepted.push(candidate.clone()),
            AutomaticRegistrationDecision::Refused(reason) => {
                plan.refused.push(RefusedAutomaticCandidate {
                    candidate: candidate.clone(),
                    reason,
                });
            }
        }
    }
    plan
}

/// Apply automatic admission and register only accepted candidates.
pub fn register_automatic_candidates(
    store: &mut Store,
    candidates: &[CandidateTarget],
) -> Result<AutomaticRegistrationOutcome, TargetsError> {
    let plan = automatic_registration_plan(candidates);
    let registration = register_candidates(store, &plan.accepted)?;
    Ok(AutomaticRegistrationOutcome {
        produced: plan.produced,
        eligible: plan.accepted.len(),
        refused: plan.refused.len(),
        registered: registration.registered,
        already_present: registration.already_present,
    })
}

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
    use fragcap_profile::{DetectionFinding, FidelityTier, SignatureCategory};

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

    fn verified_engine_candidate(path: &str, name: &str) -> CandidateTarget {
        let mut candidate = path_candidate(path, name);
        candidate.fidelity = FidelityTier::Verified;
        candidate.evidence.push(DetectionFinding {
            category: SignatureCategory::Engine,
            product: "Fixture Engine".to_string(),
            evidence: "FixtureEngine.dll".to_string(),
            fidelity: FidelityTier::Verified,
        });
        candidate
    }

    #[test]
    fn automatic_registration_accepts_authoritative_platform_identity_without_engine_evidence() {
        let decision = automatic_registration_decision(&steam_candidate(620, "Portal 2"));
        assert_eq!(
            decision,
            AutomaticRegistrationDecision::Eligible(
                AutomaticRegistrationBasis::AuthoritativePlatformIdentity
            )
        );
    }

    #[test]
    fn automatic_registration_accepts_only_strong_engine_evidence_for_path_candidates() {
        let strong = verified_engine_candidate("C:/Games/Foo", "Foo");
        assert_eq!(
            automatic_registration_decision(&strong),
            AutomaticRegistrationDecision::Eligible(
                AutomaticRegistrationBasis::VerifiedEngineEvidence
            )
        );

        let mut weak = strong;
        weak.evidence[0].fidelity = FidelityTier::HeuristicUnverified;
        assert_eq!(
            automatic_registration_decision(&weak),
            AutomaticRegistrationDecision::Refused(
                AutomaticRegistrationRefusal::InsufficientTitleEvidence
            )
        );

        let mut non_engine = weak;
        non_engine.evidence[0].category = SignatureCategory::AntiCheat;
        non_engine.evidence[0].fidelity = FidelityTier::Verified;
        assert_eq!(
            automatic_registration_decision(&non_engine),
            AutomaticRegistrationDecision::Refused(
                AutomaticRegistrationRefusal::InsufficientTitleEvidence
            )
        );

        let mut forged_platform = steam_candidate(42, "Not from Steam");
        forged_platform.source_name = "known-roots".to_string();
        assert_eq!(
            automatic_registration_decision(&forged_platform),
            AutomaticRegistrationDecision::Refused(
                AutomaticRegistrationRefusal::InsufficientTitleEvidence
            )
        );
    }

    #[test]
    fn location_alone_is_refused_and_batch_decisions_are_conserved() {
        let candidates = vec![
            steam_candidate(620, "Portal 2"),
            verified_engine_candidate("C:/Games/Foo", "Foo"),
            path_candidate("C:/Games/Browser Assets", "Browser Assets"),
        ];
        let plan = automatic_registration_plan(&candidates);
        assert_eq!(plan.accepted.len(), 2);
        assert_eq!(plan.refused.len(), 1);
        assert_eq!(plan.considered(), candidates.len());
        assert!(plan.is_conserved());
        assert_eq!(
            plan.refused[0].reason,
            AutomaticRegistrationRefusal::InsufficientTitleEvidence
        );

        let mut store = Store::open_in_memory().expect("store");
        let outcome = register_automatic_candidates(&mut store, &candidates).expect("register");
        assert_eq!(outcome.produced, 3);
        assert_eq!(outcome.eligible, 2);
        assert_eq!(outcome.refused, 1);
        assert_eq!(outcome.registered, 2);
        assert_eq!(outcome.already_present, 0);
        assert!(outcome.is_conserved());
        assert_eq!(store.targets().expect("targets").len(), 2);
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
