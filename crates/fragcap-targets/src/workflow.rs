// SPDX-License-Identifier: Apache-2.0

//! Durable progress for one explicitly resumed guided calibration workflow.

use serde_json::Value;

use fragcap_profile::FidelityTier;

use crate::{
    ClassificationSource, CompatibilityAddressFamily, CompatibilityLaunchCase,
    CompatibilityProtocol, CompatibilityRoutingStrategy, DetectionScan, TargetClassification,
    TargetEntry, TargetsError,
};

/// The checkpoint record contract understood by this build.
pub const CALIBRATION_WORKFLOW_RECORD_VERSION: i64 = 1;

/// The closed lifecycle of one guided calibration workflow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationWorkflowState {
    Ready,
    InFlight,
    Paused,
    Completed,
    Refused,
}

impl CalibrationWorkflowState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::InFlight => "in-flight",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Refused => "refused",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TargetsError> {
        match value {
            "ready" => Ok(Self::Ready),
            "in-flight" => Ok(Self::InFlight),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "refused" => Ok(Self::Refused),
            other => Err(TargetsError::Model(format!(
                "unknown calibration workflow state {other:?}"
            ))),
        }
    }
}

/// Why a workflow stopped without claiming compatibility evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationPauseReason {
    Login,
    Eula,
    Update,
    AntiCheat,
    Gameplay,
    Shutdown,
    Interrupted,
    Authorization,
    Failure,
}

impl CalibrationPauseReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Eula => "eula",
            Self::Update => "update",
            Self::AntiCheat => "anti-cheat",
            Self::Gameplay => "gameplay",
            Self::Shutdown => "shutdown",
            Self::Interrupted => "interrupted",
            Self::Authorization => "authorization",
            Self::Failure => "failure",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TargetsError> {
        match value {
            "login" => Ok(Self::Login),
            "eula" => Ok(Self::Eula),
            "update" => Ok(Self::Update),
            "anti-cheat" => Ok(Self::AntiCheat),
            "gameplay" => Ok(Self::Gameplay),
            "shutdown" => Ok(Self::Shutdown),
            "interrupted" => Ok(Self::Interrupted),
            "authorization" => Ok(Self::Authorization),
            "failure" => Ok(Self::Failure),
            other => Err(TargetsError::Model(format!(
                "unknown calibration pause reason {other:?}"
            ))),
        }
    }
}

/// The effectful calibration phase recorded only while an attempt is in flight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationWorkflowPhase {
    Reachability,
    Tls,
}

impl CalibrationWorkflowPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reachability => "reachability",
            Self::Tls => "tls",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TargetsError> {
        match value {
            "reachability" => Ok(Self::Reachability),
            "tls" => Ok(Self::Tls),
            other => Err(TargetsError::Model(format!(
                "unknown calibration workflow phase {other:?}"
            ))),
        }
    }
}

/// Complete immutable target authority bound to a workflow at creation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationTargetAuthority {
    pub stable_id: i64,
    pub handle: String,
    pub name: String,
    pub classification: TargetClassification,
    pub classification_source: ClassificationSource,
    pub fidelity: FidelityTier,
    pub provenance: Option<Value>,
    pub anchor: Option<String>,
    pub install_root: Option<String>,
    pub launch_entries: Option<Value>,
    pub evidence: Option<Value>,
    pub detection_scan: Option<DetectionScan>,
    pub folder_name: Option<String>,
    pub executable_hint: Option<String>,
}

impl CalibrationTargetAuthority {
    pub fn from_target(target: &TargetEntry) -> Self {
        Self {
            stable_id: target.stable_id,
            handle: target.handle.clone(),
            name: target.name.clone(),
            classification: target.classification,
            classification_source: target.classification_source,
            fidelity: target.fidelity,
            provenance: target.provenance.clone(),
            anchor: target.anchor.clone(),
            install_root: target.install_root.clone(),
            launch_entries: target.launch_entries.clone(),
            evidence: target.evidence.clone(),
            detection_scan: target.detection_scan,
            folder_name: target.folder_name.clone(),
            executable_hint: target.executable_hint.clone(),
        }
    }

    pub fn matches(&self, target: &TargetEntry) -> bool {
        self == &Self::from_target(target)
    }
}

/// One validated durable checkpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationWorkflow {
    pub id: i64,
    pub record_version: i64,
    pub target_id: i64,
    pub target: CalibrationTargetAuthority,
    pub selected_launch_case: Option<CompatibilityLaunchCase>,
    pub routing_strategy: CompatibilityRoutingStrategy,
    pub address_family: CompatibilityAddressFamily,
    pub requested_protocols: Vec<CompatibilityProtocol>,
    pub observed_protocols: Vec<CompatibilityProtocol>,
    pub completed_protocols: Vec<CompatibilityProtocol>,
    pub remaining_protocols: Vec<CompatibilityProtocol>,
    pub attempted_case_keys: Vec<String>,
    pub attempt_ordinal: u64,
    pub attempt_phase: Option<CalibrationWorkflowPhase>,
    pub attempt_protocol: Option<CompatibilityProtocol>,
    pub attempt_key: Option<String>,
    pub state: CalibrationWorkflowState,
    pub pause_reason: Option<CalibrationPauseReason>,
    pub revision: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Replacement values for one revision-checked checkpoint update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationWorkflowCheckpoint {
    pub requested_protocols: Vec<CompatibilityProtocol>,
    pub observed_protocols: Vec<CompatibilityProtocol>,
    pub completed_protocols: Vec<CompatibilityProtocol>,
    pub remaining_protocols: Vec<CompatibilityProtocol>,
    pub attempted_case_keys: Vec<String>,
    pub attempt_ordinal: u64,
    pub attempt_phase: Option<CalibrationWorkflowPhase>,
    pub attempt_protocol: Option<CompatibilityProtocol>,
    pub attempt_key: Option<String>,
    pub state: CalibrationWorkflowState,
    pub pause_reason: Option<CalibrationPauseReason>,
}

/// Outcome of one revision-checked checkpoint replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CalibrationWorkflowUpdateOutcome {
    Applied(Box<CalibrationWorkflow>),
    Changed,
    Missing,
}

impl CalibrationWorkflowCheckpoint {
    pub fn validate(&self) -> Result<(), TargetsError> {
        canonical_protocols(&self.requested_protocols)?;
        canonical_protocols(&self.observed_protocols)?;
        canonical_protocols(&self.completed_protocols)?;
        canonical_protocols(&self.remaining_protocols)?;
        canonical_case_keys(&self.attempted_case_keys)?;
        if self.attempt_ordinal > 14 {
            return Err(TargetsError::Model(
                "calibration workflow attempt ordinal exceeds 14".to_string(),
            ));
        }
        let attempted_count = u64::try_from(self.attempted_case_keys.len()).map_err(|_| {
            TargetsError::Model("calibration attempted case count overflowed".to_string())
        })?;
        if attempted_count > self.attempt_ordinal {
            return Err(TargetsError::Model(
                "calibration attempted case history exceeds its allocated ordinal".to_string(),
            ));
        }
        let attempt_fields = [
            self.attempt_phase.is_some(),
            self.attempt_protocol.is_some(),
            self.attempt_key.is_some(),
        ];
        if attempt_fields.iter().any(|present| *present)
            && !attempt_fields.iter().all(|present| *present)
        {
            return Err(TargetsError::Model(
                "calibration in-flight attempt fields are incomplete".to_string(),
            ));
        }
        let has_attempt = attempt_fields[0];
        if (self.state == CalibrationWorkflowState::InFlight) != has_attempt {
            return Err(TargetsError::Model(
                "only an in-flight calibration workflow carries an attempt".to_string(),
            ));
        }
        if self.state == CalibrationWorkflowState::InFlight && self.attempt_ordinal == 0 {
            return Err(TargetsError::Model(
                "an in-flight calibration workflow requires a positive attempt ordinal".to_string(),
            ));
        }
        if (self.state == CalibrationWorkflowState::Paused) != self.pause_reason.is_some() {
            return Err(TargetsError::Model(
                "only a paused calibration workflow carries a pause reason".to_string(),
            ));
        }
        match (self.attempt_phase, self.attempt_protocol) {
            (
                Some(CalibrationWorkflowPhase::Reachability),
                Some(CompatibilityProtocol::Routing),
            ) => {}
            (Some(CalibrationWorkflowPhase::Tls), Some(protocol))
                if is_concrete_protocol(protocol) => {}
            (None, None) => {}
            _ => {
                return Err(TargetsError::Model(
                    "calibration workflow phase and protocol are inconsistent".to_string(),
                ))
            }
        }
        if let Some(attempt_key) = &self.attempt_key {
            validate_case_key(attempt_key)?;
            if !self.attempted_case_keys.contains(attempt_key) {
                return Err(TargetsError::Model(
                    "in-flight calibration case is absent from attempted history".to_string(),
                ));
            }
        }
        Ok(())
    }
}

pub(crate) fn canonical_case_keys(case_keys: &[String]) -> Result<Vec<String>, TargetsError> {
    for key in case_keys {
        validate_case_key(key)?;
    }
    let mut canonical = case_keys.to_vec();
    canonical.sort();
    canonical.dedup();
    if canonical.len() > 14 {
        return Err(TargetsError::Model(
            "calibration attempted case history exceeds 14".to_string(),
        ));
    }
    Ok(canonical)
}

fn validate_case_key(key: &str) -> Result<(), TargetsError> {
    if key.is_empty() || key.len() > 2048 || key.chars().any(char::is_control) {
        return Err(TargetsError::Model(
            "calibration attempted case key is invalid".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn encode_case_keys(case_keys: &[String]) -> Result<String, TargetsError> {
    let canonical = canonical_case_keys(case_keys)?;
    if canonical != case_keys {
        return Err(TargetsError::Model(
            "calibration attempted case history is not canonical".to_string(),
        ));
    }
    serde_json::to_string(&canonical).map_err(|error| TargetsError::Model(error.to_string()))
}

pub(crate) fn decode_case_keys(value: &str) -> Result<Vec<String>, TargetsError> {
    let keys: Vec<String> = serde_json::from_str(value).map_err(|error| {
        TargetsError::Model(format!(
            "invalid calibration attempted case history: {error}"
        ))
    })?;
    let canonical = canonical_case_keys(&keys)?;
    if encode_case_keys(&canonical)? != value {
        return Err(TargetsError::Model(
            "calibration attempted case history is not canonical".to_string(),
        ));
    }
    Ok(canonical)
}

pub(crate) fn canonical_protocols(
    protocols: &[CompatibilityProtocol],
) -> Result<Vec<CompatibilityProtocol>, TargetsError> {
    if let Some(protocol) = protocols
        .iter()
        .find(|value| !is_concrete_protocol(**value))
    {
        return Err(TargetsError::Model(format!(
            "calibration workflow protocol {:?} is not concrete",
            protocol.as_str()
        )));
    }
    let mut canonical = protocols.to_vec();
    canonical.sort_by_key(|protocol| protocol.as_str());
    canonical.dedup();
    Ok(canonical)
}

pub(crate) fn encode_protocols(
    protocols: &[CompatibilityProtocol],
) -> Result<String, TargetsError> {
    let canonical = canonical_protocols(protocols)?;
    if canonical != protocols {
        return Err(TargetsError::Model(
            "calibration workflow protocol set is not canonical".to_string(),
        ));
    }
    serde_json::to_string(
        &canonical
            .iter()
            .map(|protocol| protocol.as_str())
            .collect::<Vec<_>>(),
    )
    .map_err(|error| TargetsError::Model(error.to_string()))
}

pub(crate) fn decode_protocols(value: &str) -> Result<Vec<CompatibilityProtocol>, TargetsError> {
    let tokens: Vec<String> = serde_json::from_str(value).map_err(|error| {
        TargetsError::Model(format!(
            "invalid calibration workflow protocol set: {error}"
        ))
    })?;
    let protocols = tokens
        .iter()
        .map(|token| CompatibilityProtocol::parse(token))
        .collect::<Result<Vec<_>, _>>()?;
    let canonical = canonical_protocols(&protocols)?;
    let encoded = encode_protocols(&canonical)?;
    if encoded != value {
        return Err(TargetsError::Model(
            "calibration workflow protocol set is not canonical".to_string(),
        ));
    }
    Ok(canonical)
}

fn is_concrete_protocol(protocol: CompatibilityProtocol) -> bool {
    !matches!(
        protocol,
        CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompatibilityProtocol;

    #[test]
    fn calibration_authority_detects_drift_in_every_enrichment_field() {
        let target = TargetEntry {
            id: Some(1),
            stable_id: 145,
            handle: "guided_target".to_string(),
            name: "Guided Target".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: Some(serde_json::json!({"source":"operator"})),
            anchor: Some("path:c:/games/guided".to_string()),
            launch_entries: Some(serde_json::json!([{"path":"guided.exe"}])),
            install_root: Some("C:\\Games\\Guided".to_string()),
            evidence: Some(serde_json::json!({"verified":true})),
            detection_scan: Some(DetectionScan::Complete),
            folder_name: Some("Guided".to_string()),
            executable_hint: Some("guided.exe".to_string()),
        };
        let authority = CalibrationTargetAuthority::from_target(&target);
        assert!(authority.matches(&target));

        let mut drift = target.clone();
        drift.classification = TargetClassification::Tool;
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.classification_source = ClassificationSource::Platform;
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.fidelity = FidelityTier::Verified;
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.provenance = Some(serde_json::json!({"source":"refresh"}));
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.evidence = Some(serde_json::json!({"verified":false}));
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.detection_scan = Some(DetectionScan::Incomplete);
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.folder_name = Some("Changed".to_string());
        assert!(!authority.matches(&drift));
        let mut drift = target.clone();
        drift.executable_hint = Some("changed.exe".to_string());
        assert!(!authority.matches(&drift));
    }

    #[test]
    fn calibration_workflow_closed_tokens_round_trip() {
        for state in [
            CalibrationWorkflowState::Ready,
            CalibrationWorkflowState::InFlight,
            CalibrationWorkflowState::Paused,
            CalibrationWorkflowState::Completed,
            CalibrationWorkflowState::Refused,
        ] {
            assert_eq!(
                CalibrationWorkflowState::parse(state.as_str()).unwrap(),
                state
            );
        }
        for reason in [
            CalibrationPauseReason::Login,
            CalibrationPauseReason::Eula,
            CalibrationPauseReason::Update,
            CalibrationPauseReason::AntiCheat,
            CalibrationPauseReason::Gameplay,
            CalibrationPauseReason::Shutdown,
            CalibrationPauseReason::Interrupted,
            CalibrationPauseReason::Authorization,
            CalibrationPauseReason::Failure,
        ] {
            assert_eq!(
                CalibrationPauseReason::parse(reason.as_str()).unwrap(),
                reason
            );
        }
        assert!(CalibrationWorkflowState::parse("active").is_err());
        assert!(CalibrationPauseReason::parse("maintenance").is_err());
    }

    #[test]
    fn calibration_workflow_protocol_sets_are_canonical_and_concrete() {
        let protocols = canonical_protocols(&[
            CompatibilityProtocol::Https,
            CompatibilityProtocol::Http1,
            CompatibilityProtocol::Https,
        ])
        .unwrap();
        assert_eq!(
            protocols,
            vec![CompatibilityProtocol::Http1, CompatibilityProtocol::Https]
        );
        let encoded = encode_protocols(&protocols).unwrap();
        assert_eq!(encoded, r#"["http1","https"]"#);
        assert_eq!(decode_protocols(&encoded).unwrap(), protocols);
        assert!(canonical_protocols(&[CompatibilityProtocol::Routing]).is_err());
        assert!(decode_protocols(r#"["https","http1"]"#).is_err());
        assert!(decode_protocols(r#"["routing"]"#).is_err());

        let keys = canonical_case_keys(&["tls-b".to_string(), "tls-a".to_string()]).unwrap();
        assert_eq!(keys, vec!["tls-a", "tls-b"]);
        let encoded_keys = encode_case_keys(&keys).unwrap();
        assert_eq!(encoded_keys, r#"["tls-a","tls-b"]"#);
        assert_eq!(decode_case_keys(&encoded_keys).unwrap(), keys);
        assert!(decode_case_keys(r#"["tls-b","tls-a"]"#).is_err());
    }
}
