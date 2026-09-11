// SPDX-License-Identifier: Apache-2.0

//! Stable, content-derived identities for one guided-calibration candidate choice.

use serde_json::{json, Value};

use crate::{CandidateIdentity, CandidateTarget, TargetsError};

/// Versioned prefix for one calibration candidate identity.
pub const CALIBRATION_CANDIDATE_PREFIX: &str = "candidate-v1:";

/// Return the stable identity of one complete candidate authority.
pub fn calibration_candidate_id(candidate: &CandidateTarget) -> String {
    let canonical = calibration_candidate_value(candidate);
    let encoded = serde_json::to_vec(&canonical)
        .expect("the calibration candidate projection contains only serializable values");
    format!(
        "{CALIBRATION_CANDIDATE_PREFIX}{}",
        blake3::hash(&encoded).to_hex()
    )
}

/// Validate one candidate identity supplied by an operator or automation.
pub fn validate_calibration_candidate_id(value: &str) -> Result<(), TargetsError> {
    let Some(digest) = value.strip_prefix(CALIBRATION_CANDIDATE_PREFIX) else {
        return Err(TargetsError::Model(
            "calibration candidate identity has an unsupported prefix".to_string(),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(TargetsError::Model(
            "calibration candidate identity requires 64 lowercase hexadecimal characters"
                .to_string(),
        ));
    }
    Ok(())
}

/// Return the canonical reviewable value whose bytes define a candidate identity.
pub fn calibration_candidate_value(candidate: &CandidateTarget) -> Value {
    let identity = match &candidate.identity {
        CandidateIdentity::SteamAppId(app_id) => {
            json!({"kind": "steam-app-id", "value": app_id})
        }
        CandidateIdentity::Path(path) => json!({"kind": "path", "value": path}),
    };
    let mut evidence = candidate
        .evidence
        .iter()
        .map(|finding| {
            json!({
                "category": finding.category.as_str(),
                "evidence": finding.evidence,
                "fidelity": finding.fidelity.as_str(),
                "product": finding.product,
            })
        })
        .collect::<Vec<_>>();
    evidence.sort_by_key(Value::to_string);
    json!({
        "classification": candidate.classification.as_str(),
        "detection_scan": candidate.detection_scan.map(|value| value.as_str()),
        "display_name": candidate.display_name,
        "evidence": evidence,
        "executable_hint": candidate.executable_hint,
        "fidelity": candidate.fidelity.as_str(),
        "folder_name": candidate.folder_name,
        "identity": identity,
        "install_root": candidate.install_root,
        "schema": "fragcap.calibration-candidate.v1",
        "source": candidate.source_name,
    })
}

#[cfg(test)]
mod tests {
    use fragcap_profile::{DetectionFinding, FidelityTier, SignatureCategory};

    use super::*;
    use crate::{CandidateIdentity, DetectionScan, TargetClassification};

    type CandidateMutation = Box<dyn Fn(&mut CandidateTarget)>;

    fn candidate() -> CandidateTarget {
        CandidateTarget {
            identity: CandidateIdentity::SteamAppId(75000),
            display_name: "Sample Target".to_string(),
            fidelity: FidelityTier::Observed,
            classification: TargetClassification::Game,
            evidence: vec![
                DetectionFinding {
                    category: SignatureCategory::Engine,
                    product: "Fixture Engine".to_string(),
                    evidence: "Engine.dll".to_string(),
                    fidelity: FidelityTier::Verified,
                },
                DetectionFinding {
                    category: SignatureCategory::Drm,
                    product: "Fixture DRM".to_string(),
                    evidence: "drm.dll".to_string(),
                    fidelity: FidelityTier::Observed,
                },
            ],
            detection_scan: Some(DetectionScan::Complete),
            source_name: "steam".to_string(),
            install_root: Some("C:\\Games\\Sample Target".to_string()),
            folder_name: Some("Sample Target".to_string()),
            executable_hint: Some("client.exe".to_string()),
        }
    }

    #[test]
    fn candidate_identity_is_deterministic_and_strictly_parseable() {
        let id = calibration_candidate_id(&candidate());
        assert_eq!(id.len(), CALIBRATION_CANDIDATE_PREFIX.len() + 64);
        assert!(id.starts_with(CALIBRATION_CANDIDATE_PREFIX));
        assert!(id[CALIBRATION_CANDIDATE_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        assert_eq!(calibration_candidate_id(&candidate()), id);
        assert!(validate_calibration_candidate_id(&id).is_ok());
        for invalid in [
            "candidate-v1:",
            "candidate-v2:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "candidate-v1:0123456789ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef",
            "candidate-v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg",
            " candidate-v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ] {
            assert!(
                validate_calibration_candidate_id(invalid).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn every_authority_field_changes_the_candidate_identity() {
        let base = candidate();
        let base_id = calibration_candidate_id(&base);
        let mutations: Vec<CandidateMutation> = vec![
            Box::new(|value| value.identity = CandidateIdentity::SteamAppId(75001)),
            Box::new(|value| value.display_name.push_str(" Changed")),
            Box::new(|value| value.fidelity = FidelityTier::Verified),
            Box::new(|value| value.classification = TargetClassification::Tool),
            Box::new(|value| value.evidence[0].product.push_str(" Changed")),
            Box::new(|value| value.detection_scan = Some(DetectionScan::Incomplete)),
            Box::new(|value| value.source_name.push_str("-changed")),
            Box::new(|value| value.install_root = Some("D:\\Games\\Sample Target".to_string())),
            Box::new(|value| value.folder_name = Some("Changed".to_string())),
            Box::new(|value| value.executable_hint = Some("changed.exe".to_string())),
        ];
        for mutate in mutations {
            let mut changed = base.clone();
            mutate(&mut changed);
            assert_ne!(calibration_candidate_id(&changed), base_id);
        }
    }

    #[test]
    fn evidence_order_does_not_change_the_candidate_identity() {
        let first = candidate();
        let mut reversed = first.clone();
        reversed.evidence.reverse();
        assert_eq!(
            calibration_candidate_id(&first),
            calibration_candidate_id(&reversed)
        );
    }
}
