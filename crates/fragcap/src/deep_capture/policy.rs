// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use crate::targets::{
    latest_applicable_fact, CompatibilityCase, CompatibilityFact, CompatibilityFactKey,
    CompatibilityLaunchCase, CompatibilityProtocol,
};

use super::{
    CalibrationOutcome, CalibrationPhase, ClassificationReason, CompatibilityFactCandidate,
    CompatibilityObservation, DetectionState, InspectabilityState, LaunchCase, PreflightRefusal,
    SessionMode, TrafficFamily,
};

/// Enforce the shipped Deep Capture compatibility prerequisites for one plan.
pub fn validate_compatibility_prerequisites(
    mode: SessionMode,
    controlled: bool,
    facts: &[CompatibilityFact],
    launch_case: LaunchCase,
    current: &CompatibilityCase,
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
    let routes = latest_applicable_fact(facts, CompatibilityFactKey::ProxyRouting, current);
    if routes.is_none_or(|fact| fact.value != "reached-client") {
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
        LaunchCase::SteamProtocolCold
        | LaunchCase::DirectExeCold
        | LaunchCase::PublisherLauncherCold => Ok(()),
        LaunchCase::SteamProtocolWarm => Err(PreflightRefusal::new(
            "launch-case",
            "Deep Capture cannot apply scoped proxy settings through an already-running Steam process; close Steam and retry so fragcap can own the cold launch",
        )),
        LaunchCase::DirectExeWarm => Err(PreflightRefusal::new(
            "launch-case",
            "Deep Capture cannot apply scoped proxy settings to an already-running direct executable; close it and retry so fragcap can own the cold launch",
        )),
        LaunchCase::PublisherLauncherWarm => Err(PreflightRefusal::new(
            "launch-case",
            "Deep Capture cannot apply scoped proxy settings to an already-running publisher launcher; close the publisher chain and retry so fragcap can own the cold launch",
        )),
        LaunchCase::PublisherLauncherGameStartCleanWarm => Err(PreflightRefusal::new(
            "launch-case",
            "Deep Capture cannot apply scoped proxy settings retroactively to a publisher launcher even when the game client is not running; close the launcher and retry from a cold chain",
        )),
        _ => Err(PreflightRefusal::new(
            "launch-case",
            format!(
                "Deep Capture does not support managed launch case {}; supported managed paths are cold Steam protocol, cold direct-executable, and exact cold publisher-chain launches whose compatibility facts prove client routing",
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
    selected_protocol: Option<CompatibilityProtocol>,
) -> Vec<CompatibilityFactCandidate> {
    let mut facts = Vec::new();
    // A cold Steam launch is rooted in the exact platform process fragcap
    // created with the scoped proxy environment. A correlated final client can
    // only bind beneath that root, so reaching it proves propagation just as
    // directly as the controlled harness does. Routing remains the independent
    // reached-client fact below.
    let propagation_owned = controlled || launch_case == LaunchCase::SteamProtocolCold.as_str();
    let final_owner_index = observations.iter().rposition(|observation| {
        observation
            .role
            .as_deref()
            .and_then(compatibility_owner_role)
            == Some("client")
    });
    let mut push = |key, value: &str, phase, protocol| {
        facts.push(CompatibilityFactCandidate {
            key,
            value: value.to_string(),
            phase,
            protocol,
            final_owner_index,
        });
    };

    if let Some(phase) = calibration {
        push(
            CompatibilityFactKey::LaunchCase,
            launch_case,
            phase,
            CompatibilityProtocol::NotApplicable,
        );
    }
    if !observations.is_empty() && calibration != Some(CalibrationPhase::Tls) {
        let reached_client = observations.iter().any(|observation| {
            observation_is_correlated_to_final_client(observation)
                || (controlled
                    && observation.attribution.as_deref() == Some("controlled-harness")
                    && observation.role.as_deref() == Some("client"))
        });
        let phase = calibration.unwrap_or(CalibrationPhase::Reachability);
        push(
            CompatibilityFactKey::ProxyRouting,
            if reached_client {
                "reached-client"
            } else {
                "inconclusive"
            },
            phase,
            CompatibilityProtocol::NotApplicable,
        );
        push(
            CompatibilityFactKey::ProxyPropagation,
            if reached_client && propagation_owned {
                "confirmed"
            } else if reached_client {
                "not-tested"
            } else {
                "not-confirmed"
            },
            phase,
            CompatibilityProtocol::NotApplicable,
        );
        for variable in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"] {
            push(
                CompatibilityFactKey::ProxyVariableTested,
                variable,
                phase,
                CompatibilityProtocol::NotApplicable,
            );
        }
    }
    let accepted_protocol = observations.iter().find_map(|observation| {
        let protocol = compatibility_protocol_family(&observation.classification)?;
        if selected_protocol.is_none_or(|selected| selected == protocol)
            && (observation_proves_final_client_ca_acceptance(observation)
                || (controlled
                    && observation.attribution.as_deref() == Some("controlled-harness")
                    && observation.role.as_deref() == Some("client")
                    && classification_proves_tls(&observation.classification)))
        {
            Some(protocol)
        } else {
            None
        }
    });
    if calibration != Some(CalibrationPhase::Reachability) {
        if let Some(protocol) = accepted_protocol {
            push(
                CompatibilityFactKey::TlsTrustBehavior,
                "accepts-local-ca",
                calibration.unwrap_or(CalibrationPhase::Tls),
                protocol,
            );
        }
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
            CompatibilityProtocol::NotApplicable,
        );
    }
    if calibration != Some(CalibrationPhase::Reachability) {
        let phase = calibration.unwrap_or(CalibrationPhase::Tls);
        let inspectability: BTreeSet<(CompatibilityProtocol, &str)> = observations
            .iter()
            .filter(|observation| {
                classification_is_fact_eligible(&observation.classification)
                    && selected_protocol.is_none_or(|selected| {
                        compatibility_protocol_family(&observation.classification) == Some(selected)
                    })
            })
            .filter_map(|observation| {
                compatibility_protocol_family(&observation.classification).map(|protocol| {
                    (
                        protocol,
                        compatibility_inspectability(&observation.classification),
                    )
                })
            })
            .collect();
        for (protocol, value) in inspectability {
            push(CompatibilityFactKey::Inspectability, value, phase, protocol);
        }
        let protocols: BTreeSet<(CompatibilityProtocol, &str)> = observations
            .iter()
            .filter(|observation| {
                classification_is_fact_eligible(&observation.classification)
                    && selected_protocol.is_none_or(|selected| {
                        compatibility_protocol_family(&observation.classification) == Some(selected)
                    })
            })
            .filter_map(|observation| {
                compatibility_protocol_family(&observation.classification).map(|protocol| {
                    (
                        protocol,
                        compatibility_protocol(&observation.classification),
                    )
                })
            })
            .collect();
        for (protocol, value) in protocols {
            push(
                CompatibilityFactKey::ProtocolBehavior,
                value,
                phase,
                protocol,
            );
        }
    }
    facts
}

fn compatibility_protocol_family(
    classification: &super::ProtocolClassification,
) -> Option<CompatibilityProtocol> {
    Some(match classification.family() {
        TrafficFamily::Http1 => CompatibilityProtocol::Http1,
        TrafficFamily::Https => CompatibilityProtocol::Https,
        TrafficFamily::Http2 => CompatibilityProtocol::Http2,
        TrafficFamily::WebSocket => CompatibilityProtocol::WebSocket,
        TrafficFamily::Sse => CompatibilityProtocol::Sse,
        TrafficFamily::Grpc => CompatibilityProtocol::Grpc,
        TrafficFamily::GenericTcp => CompatibilityProtocol::GenericTcp,
        TrafficFamily::NonHttpTls => CompatibilityProtocol::NonHttpTls,
        TrafficFamily::Socks5Tcp => CompatibilityProtocol::Socks5Tcp,
        TrafficFamily::Socks5Udp => CompatibilityProtocol::Socks5Udp,
        TrafficFamily::GenericUdp => CompatibilityProtocol::GenericUdp,
        TrafficFamily::Quic => CompatibilityProtocol::Quic,
        TrafficFamily::Http3 => CompatibilityProtocol::Http3,
        TrafficFamily::Unrouted | TrafficFamily::Unknown => return None,
    })
}

fn compatibility_protocol(classification: &super::ProtocolClassification) -> &'static str {
    match classification.family() {
        TrafficFamily::Http1 | TrafficFamily::Http2 | TrafficFamily::Sse | TrafficFamily::Grpc => {
            "http"
        }
        TrafficFamily::Https | TrafficFamily::Http3 => "https",
        TrafficFamily::WebSocket => "websocket",
        TrafficFamily::NonHttpTls => "non-http-tls",
        TrafficFamily::Quic => "quic",
        TrafficFamily::Socks5Udp | TrafficFamily::GenericUdp => "udp",
        TrafficFamily::GenericTcp | TrafficFamily::Socks5Tcp => "plaintext",
        TrafficFamily::Unrouted | TrafficFamily::Unknown => "unknown",
    }
}

fn compatibility_inspectability(classification: &super::ProtocolClassification) -> &'static str {
    match classification.inspectability() {
        InspectabilityState::Full | InspectabilityState::DecryptedUnknown => "full",
        InspectabilityState::MetadataOnly | InspectabilityState::EncryptedOpaque => "metadata-only",
        InspectabilityState::PacketOnly | InspectabilityState::Unavailable => "unknown",
    }
}

/// Classify retained observations without inventing evidence from silence.
pub fn calibration_outcome(
    phase: CalibrationPhase,
    observations: &[CompatibilityObservation],
) -> CalibrationOutcome {
    match phase {
        CalibrationPhase::Reachability => {
            if observations.iter().any(|observation| {
                observation_is_correlated_to_final_client(observation)
                    || controlled_harness_client(observation)
            }) {
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
            } else if observations.iter().all(|observation| {
                observation.classification.detection() == DetectionState::Unsupported
            }) {
                CalibrationOutcome::UnsupportedProtocol
            } else {
                CalibrationOutcome::Inconclusive
            }
        }
        CalibrationPhase::Tls => {
            if observations.iter().any(|observation| {
                observation_proves_final_client_ca_acceptance(observation)
                    || (controlled_harness_client(observation)
                        && classification_proves_tls(&observation.classification))
            }) {
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
            } else if observations.iter().all(|observation| {
                observation.classification.detection() == DetectionState::Unsupported
            }) {
                CalibrationOutcome::UnsupportedProtocol
            } else if observations.iter().any(|observation| {
                observation.classification.inspectability() == InspectabilityState::MetadataOnly
            }) {
                CalibrationOutcome::MetadataOnly
            } else {
                CalibrationOutcome::UnknownTrust
            }
        }
    }
}

fn controlled_harness_client(observation: &CompatibilityObservation) -> bool {
    observation.attribution.as_deref() == Some("controlled-harness")
        && observation.role.as_deref() == Some("client")
}

fn classification_is_fact_eligible(classification: &super::ProtocolClassification) -> bool {
    classification.detection() == DetectionState::Identified
        && !matches!(
            classification.family(),
            TrafficFamily::Unknown | TrafficFamily::Unrouted
        )
        && !matches!(
            classification.inspectability(),
            InspectabilityState::Unavailable | InspectabilityState::PacketOnly
        )
        && matches!(
            classification.reason(),
            None | Some(ClassificationReason::EncryptedOpaque)
        )
}

fn classification_proves_tls(classification: &super::ProtocolClassification) -> bool {
    classification.detection() == DetectionState::Identified
        && classification.family() == TrafficFamily::Https
        && classification.inspectability() == InspectabilityState::Full
        && classification.reason().is_none()
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
        && classification_proves_tls(&observation.classification)
}

/// Fold interruption and operation failure into the evidence-based outcome.
pub fn terminal_calibration_outcome(
    phase: CalibrationPhase,
    selected_protocol: CompatibilityProtocol,
    observations: &[CompatibilityObservation],
    interrupted: bool,
    failed: bool,
) -> CalibrationOutcome {
    if interrupted {
        CalibrationOutcome::Interrupted
    } else if failed {
        CalibrationOutcome::Failed
    } else {
        match phase {
            CalibrationPhase::Reachability => calibration_outcome(phase, observations),
            CalibrationPhase::Tls => {
                let matching = observations
                    .iter()
                    .filter(|observation| {
                        compatibility_protocol_family(&observation.classification)
                            == Some(selected_protocol)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                calibration_outcome(phase, &matching)
            }
        }
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

#[cfg(test)]
mod launch_case_tests {
    use super::super::{
        DetectionState, Inspectability, InspectabilityState, ProtocolClassification, TrafficFamily,
    };
    use super::*;

    fn correlated_client_observation() -> CompatibilityObservation {
        CompatibilityObservation {
            flow_id: crate::FlowId::new(1),
            proxy_connection_id: "proxy-1".into(),
            client_peer: None,
            proxy_local: None,
            observed_at: "1970-01-01T00:00:01Z".into(),
            process_id: Some(42),
            process_image: Some("game.exe".into()),
            role: Some("client".into()),
            attribution: Some("live".into()),
            packet_observations: 1,
            packet_observations_unretained: 0,
            correlation_state: super::super::CorrelationState::Matched,
            correlation_reason: "exact-flow-and-owner".into(),
            protocol: "http".into(),
            inspectability: Inspectability::MetadataOnly,
            method: Some("GET".into()),
            url: Some("http://example.invalid/".into()),
            status: Some(200),
            reason: Some("owned final-client flow".into()),
            classification: ProtocolClassification::new(
                TrafficFamily::Http1,
                DetectionState::Identified,
                InspectabilityState::MetadataOnly,
                None,
            )
            .unwrap(),
        }
    }

    #[test]
    fn processing_failures_never_promote_positive_compatibility_facts() {
        for reason in [
            ClassificationReason::ParserFailed,
            ClassificationReason::Truncated,
            ClassificationReason::WriterFailed,
        ] {
            let mut observation = correlated_client_observation();
            observation.classification = if reason == ClassificationReason::ParserFailed {
                ProtocolClassification::new(
                    TrafficFamily::Http1,
                    DetectionState::Failed,
                    InspectabilityState::MetadataOnly,
                    Some(reason),
                )
                .unwrap()
            } else {
                ProtocolClassification::new(
                    TrafficFamily::Http1,
                    DetectionState::Identified,
                    InspectabilityState::Full,
                    Some(reason),
                )
                .unwrap()
            };
            let facts = compatibility_fact_candidates(
                LaunchCase::DirectExeCold.as_str(),
                &[observation],
                false,
                Some(CalibrationPhase::Tls),
                Some(CompatibilityProtocol::Http1),
            );
            assert!(!facts.iter().any(|fact| {
                matches!(
                    fact.key,
                    CompatibilityFactKey::ProtocolBehavior
                        | CompatibilityFactKey::Inspectability
                        | CompatibilityFactKey::TlsTrustBehavior
                )
            }));
        }
    }

    #[test]
    fn selected_protocol_never_promotes_a_different_identified_family() {
        let facts = compatibility_fact_candidates(
            LaunchCase::DirectExeCold.as_str(),
            &[correlated_client_observation()],
            false,
            Some(CalibrationPhase::Tls),
            Some(CompatibilityProtocol::Http2),
        );
        assert!(!facts.iter().any(|fact| {
            matches!(
                fact.key,
                CompatibilityFactKey::ProtocolBehavior
                    | CompatibilityFactKey::Inspectability
                    | CompatibilityFactKey::TlsTrustBehavior
            )
        }));
    }

    #[test]
    fn exact_cold_publisher_chain_is_supported() {
        assert!(require_supported_launch_case(LaunchCase::PublisherLauncherCold).is_ok());
    }

    #[test]
    fn cold_steam_owned_client_confirms_propagation_without_conflating_routing() {
        let facts = compatibility_fact_candidates(
            LaunchCase::SteamProtocolCold.as_str(),
            &[correlated_client_observation()],
            false,
            Some(CalibrationPhase::Reachability),
            Some(CompatibilityProtocol::Routing),
        );
        let value = |key| {
            facts
                .iter()
                .find(|fact| fact.key == key)
                .map(|fact| fact.value.as_str())
        };

        assert_eq!(
            value(CompatibilityFactKey::ProxyRouting),
            Some("reached-client")
        );
        assert_eq!(
            value(CompatibilityFactKey::ProxyPropagation),
            Some("confirmed")
        );
    }

    #[test]
    fn unowned_correlated_client_does_not_claim_propagation() {
        let facts = compatibility_fact_candidates(
            LaunchCase::PublisherLauncherCold.as_str(),
            &[correlated_client_observation()],
            false,
            Some(CalibrationPhase::Reachability),
            Some(CompatibilityProtocol::Routing),
        );

        assert_eq!(
            facts
                .iter()
                .find(|fact| fact.key == CompatibilityFactKey::ProxyPropagation)
                .map(|fact| fact.value.as_str()),
            Some("not-tested")
        );
    }

    #[test]
    fn both_warm_publisher_states_remain_distinct_refusals() {
        let warm = require_supported_launch_case(LaunchCase::PublisherLauncherWarm).unwrap_err();
        let clean_game =
            require_supported_launch_case(LaunchCase::PublisherLauncherGameStartCleanWarm)
                .unwrap_err();
        assert_eq!(warm.code, "launch-case");
        assert_eq!(clean_game.code, "launch-case");
        assert_ne!(warm.detail, clean_game.detail);
        assert!(warm.detail.contains("already-running publisher launcher"));
        assert!(clean_game.detail.contains("game client is not running"));
    }
}
