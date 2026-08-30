// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use crate::targets::{CompatibilityFact, CompatibilityFactKey, CompatibilityLaunchCase};

use super::{
    CalibrationOutcome, CalibrationPhase, CompatibilityFactCandidate, CompatibilityObservation,
    Inspectability, LaunchCase, PreflightRefusal, SessionMode,
};

/// Enforce the shipped Deep Capture compatibility prerequisites for one plan.
pub fn validate_compatibility_prerequisites(
    mode: SessionMode,
    controlled: bool,
    facts: &[CompatibilityFact],
    launch_case: LaunchCase,
) -> Result<(), PreflightRefusal> {
    if !controlled && mode != SessionMode::Capture {
        require_supported_launch_case(launch_case)?;
    }
    if mode == SessionMode::ReachabilityCalibration {
        return Ok(());
    }
    if controlled && mode == SessionMode::Capture {
        return Ok(());
    }
    let Some(stored_case) = stored_launch_case(launch_case) else {
        return Err(PreflightRefusal::new(
            "launch-case",
            "the controlled launch case requires the controlled adapter set",
        ));
    };
    let routes = facts.iter().rev().find(|fact| {
        fact.launch_case == Some(stored_case) && fact.key == CompatibilityFactKey::ProxyRouting
    });
    if !routes.is_some_and(|fact| !fact.stale && fact.value == "reached-client") {
        return Err(PreflightRefusal::new(
            "routing-prerequisite",
            format!(
                "Deep Capture requires current compatibility facts proving scoped proxy routing reaches the final client for launch case {}; run reachability calibration first",
                stored_case.as_str()
            ),
        ));
    }
    if mode == SessionMode::TlsCalibration {
        return Ok(());
    }
    require_supported_launch_case(launch_case)
}

fn require_supported_launch_case(launch_case: LaunchCase) -> Result<(), PreflightRefusal> {
    match launch_case {
        LaunchCase::SteamProtocolCold | LaunchCase::DirectExeCold => Ok(()),
        LaunchCase::SteamProtocolWarm => Err(PreflightRefusal::new(
            "launch-case",
            "Deep Capture cannot apply scoped proxy settings through an already-running Steam process; close Steam and retry so fragcap can own the cold launch",
        )),
        LaunchCase::DirectExeWarm => Err(PreflightRefusal::new(
            "launch-case",
            "Deep Capture cannot apply scoped proxy settings to an already-running direct executable; close it and retry so fragcap can own the cold launch",
        )),
        _ => Err(PreflightRefusal::new(
            "launch-case",
            format!(
                "Deep Capture does not support managed launch case {}; supported managed paths are cold Steam protocol and cold direct-executable launches whose compatibility facts prove client routing",
                launch_case.as_str()
            ),
        )),
    }
}

fn stored_launch_case(value: LaunchCase) -> Option<CompatibilityLaunchCase> {
    Some(match value {
        LaunchCase::SteamProtocolWarm => CompatibilityLaunchCase::SteamProtocolWarm,
        LaunchCase::SteamProtocolCold => CompatibilityLaunchCase::SteamProtocolCold,
        LaunchCase::DirectExeWarm => CompatibilityLaunchCase::DirectExeWarm,
        LaunchCase::DirectExeCold => CompatibilityLaunchCase::DirectExeCold,
        LaunchCase::PublisherLauncher => CompatibilityLaunchCase::PublisherLauncher,
        LaunchCase::PublisherLauncherWarm => CompatibilityLaunchCase::PublisherLauncherWarm,
        LaunchCase::PublisherLauncherGameStartCleanWarm => {
            CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm
        }
        LaunchCase::PublisherLauncherCold => CompatibilityLaunchCase::PublisherLauncherCold,
        LaunchCase::Controlled => return None,
    })
}

/// Select append-only compatibility facts from direct observations.
pub fn compatibility_fact_candidates(
    launch_case: &str,
    observations: &[CompatibilityObservation],
    controlled: bool,
    calibration: Option<CalibrationPhase>,
) -> Vec<CompatibilityFactCandidate> {
    let mut facts = Vec::new();
    let final_owner_index = observations.iter().rposition(|observation| {
        observation
            .role
            .as_deref()
            .and_then(compatibility_owner_role)
            == Some("client")
    });
    let mut push = |key, value: &str, phase| {
        facts.push(CompatibilityFactCandidate {
            key,
            value: value.to_string(),
            phase,
            final_owner_index,
        });
    };

    if let Some(phase) = calibration {
        push(CompatibilityFactKey::LaunchCase, launch_case, phase);
    }
    if !observations.is_empty() && calibration != Some(CalibrationPhase::Tls) {
        let reached_client = observations
            .iter()
            .any(observation_is_correlated_to_final_client);
        let phase = calibration.unwrap_or(CalibrationPhase::Reachability);
        push(
            CompatibilityFactKey::ProxyRouting,
            if reached_client {
                "reached-client"
            } else {
                "inconclusive"
            },
            phase,
        );
        push(
            CompatibilityFactKey::ProxyPropagation,
            if reached_client && controlled {
                "confirmed"
            } else if reached_client {
                "not-tested"
            } else {
                "not-confirmed"
            },
            phase,
        );
        for variable in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"] {
            push(CompatibilityFactKey::ProxyVariableTested, variable, phase);
        }
    }
    if calibration != Some(CalibrationPhase::Reachability)
        && observations
            .iter()
            .any(observation_proves_final_client_ca_acceptance)
    {
        push(
            CompatibilityFactKey::TlsTrustBehavior,
            "accepts-local-ca",
            calibration.unwrap_or(CalibrationPhase::Tls),
        );
    }
    let final_roles: BTreeSet<&str> = observations
        .iter()
        .filter_map(|observation| observation.role.as_deref())
        .filter_map(compatibility_owner_role)
        .collect();
    for role in final_roles {
        push(
            CompatibilityFactKey::FinalSocketOwnerRole,
            role,
            calibration.unwrap_or(CalibrationPhase::Reachability),
        );
    }
    if calibration != Some(CalibrationPhase::Reachability) {
        let phase = calibration.unwrap_or(CalibrationPhase::Tls);
        let inspectability: BTreeSet<&str> = observations
            .iter()
            .map(|observation| observation.inspectability.as_str())
            .collect();
        for value in inspectability {
            push(CompatibilityFactKey::Inspectability, value, phase);
        }
        let protocols: BTreeSet<&str> = observations
            .iter()
            .map(|observation| observation.protocol.as_str())
            .collect();
        for value in protocols {
            push(CompatibilityFactKey::ProtocolBehavior, value, phase);
        }
    }
    facts
}

/// Classify retained observations without inventing evidence from silence.
pub fn calibration_outcome(
    phase: CalibrationPhase,
    observations: &[CompatibilityObservation],
) -> CalibrationOutcome {
    match phase {
        CalibrationPhase::Reachability => {
            if observations
                .iter()
                .any(observation_is_correlated_to_final_client)
            {
                CalibrationOutcome::ReachedClient
            } else if observations.iter().any(|observation| {
                matches!(
                    observation
                        .role
                        .as_deref()
                        .and_then(compatibility_owner_role),
                    Some("launcher" | "platform" | "platform-service")
                )
            }) {
                CalibrationOutcome::LauncherOnly
            } else if observations
                .iter()
                .any(|observation| observation.reason.as_deref() == Some("escaped-tree"))
            {
                CalibrationOutcome::EscapedTree
            } else if observations
                .iter()
                .any(|observation| observation.reason.as_deref() == Some("no-relevant-traffic"))
            {
                CalibrationOutcome::NoRelevantTraffic
            } else if observations
                .iter()
                .any(|observation| observation.reason.as_deref() == Some("proxy-not-reached"))
            {
                CalibrationOutcome::ProxyNotReached
            } else if observations.is_empty() {
                CalibrationOutcome::Inconclusive
            } else if observations
                .iter()
                .all(|observation| observation.inspectability == Inspectability::Unsupported)
            {
                CalibrationOutcome::UnsupportedProtocol
            } else {
                CalibrationOutcome::Inconclusive
            }
        }
        CalibrationPhase::Tls => {
            if observations
                .iter()
                .any(observation_proves_final_client_ca_acceptance)
            {
                CalibrationOutcome::LocalCaAccepted
            } else if observations
                .iter()
                .any(|observation| observation.reason.as_deref() == Some("certificate-pinned"))
            {
                CalibrationOutcome::CertificatePinned
            } else if observations
                .iter()
                .any(|observation| observation.reason.as_deref() == Some("proxy-not-reached"))
            {
                CalibrationOutcome::ProxyNotReached
            } else if observations.is_empty() {
                CalibrationOutcome::Inconclusive
            } else if observations
                .iter()
                .all(|observation| observation.inspectability == Inspectability::Unsupported)
            {
                CalibrationOutcome::UnsupportedProtocol
            } else if observations
                .iter()
                .any(|observation| observation.inspectability == Inspectability::MetadataOnly)
            {
                CalibrationOutcome::MetadataOnly
            } else {
                CalibrationOutcome::UnknownTrust
            }
        }
    }
}

/// Whether the observation is packet-correlated to the final client role.
pub fn observation_is_correlated_to_final_client(observation: &CompatibilityObservation) -> bool {
    observation.flow_id.is_some()
        && observation
            .role
            .as_deref()
            .and_then(compatibility_owner_role)
            == Some("client")
}

/// Whether the observation directly demonstrates final-client CA acceptance.
pub fn observation_proves_final_client_ca_acceptance(
    observation: &CompatibilityObservation,
) -> bool {
    observation_is_correlated_to_final_client(observation)
        && observation.protocol == "https"
        && observation.inspectability == Inspectability::Full
}

/// Fold interruption and operation failure into the evidence-based outcome.
pub fn terminal_calibration_outcome(
    phase: CalibrationPhase,
    observations: &[CompatibilityObservation],
    interrupted: bool,
    failed: bool,
) -> CalibrationOutcome {
    if interrupted {
        CalibrationOutcome::Interrupted
    } else if failed {
        CalibrationOutcome::Failed
    } else {
        calibration_outcome(phase, observations)
    }
}

/// Stable human reason for one calibration outcome.
pub fn calibration_outcome_reason(
    phase: CalibrationPhase,
    outcome: CalibrationOutcome,
) -> &'static str {
    match (phase, outcome) {
        (_, CalibrationOutcome::Failed) => {
            "the calibration operation did not complete successfully"
        }
        (_, CalibrationOutcome::Interrupted) => "the operator interrupted the calibration",
        (CalibrationPhase::Reachability, CalibrationOutcome::ReachedClient) => {
            "proxy traffic correlated to the final client"
        }
        (CalibrationPhase::Tls, CalibrationOutcome::LocalCaAccepted) => {
            "HTTPS application semantics were observed through the session CA"
        }
        (_, CalibrationOutcome::ProxyNotReached) => {
            "no proxy observation was available before the phase ended"
        }
        (_, CalibrationOutcome::CertificatePinned) => {
            "the backend supplied explicit certificate-pinning evidence"
        }
        _ => "the phase retained only the observations supporting this outcome",
    }
}

/// Normalize an observed process role for compatibility evidence.
pub fn compatibility_owner_role(role: &str) -> Option<&str> {
    match role {
        "target" | "client" => Some("client"),
        "launcher" => Some("launcher"),
        "platform" => Some("platform"),
        "platform-service" => Some("platform-service"),
        "helper" => Some("helper"),
        "proxy" => Some("proxy"),
        "wrapper" => Some("wrapper"),
        "unknown" => Some("unknown"),
        _ => None,
    }
}
