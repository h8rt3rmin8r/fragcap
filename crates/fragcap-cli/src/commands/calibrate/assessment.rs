// SPDX-License-Identifier: Apache-2.0

//! One pure attempted-case projection, independent from current stored readiness.

use fragcap::deep_capture::api::{
    terminal_calibration_outcome, ArtifactStatus, CalibrationOutcome, CalibrationPhase,
    CleanupStatus, CompatibilityProtocol, CorrelationState, EvidenceWindow, FactWriteResult,
    FactWriteStatus, ProxyDiagnostics, SessionOutcome, TerminalReport,
};
use fragcap::deep_capture::{CaptureProcessEvidence, StageTransitionKind};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub(crate) struct AttemptAssessment {
    pub(crate) verdict: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) earliest_boundary: &'static str,
    pub(crate) outcome: CalibrationOutcome,
    pub(crate) proxy: Option<ProxyDiagnostics>,
    pub(crate) observation_records: u64,
    pub(crate) owner_release_records: u64,
    pub(crate) unavailable_window_records: u64,
    pub(crate) matched_records: u64,
    pub(crate) flow_only_records: u64,
    pub(crate) ambiguous_records: u64,
    pub(crate) unavailable_correlation_records: u64,
    pub(crate) other_owner_records: u64,
    pub(crate) final_client_records: u64,
    pub(crate) retained_target_packets: Option<u64>,
    pub(crate) finalization: &'static str,
    pub(crate) artifact_writes: &'static str,
    pub(crate) process_completeness: &'static str,
    pub(crate) cleanup: &'static str,
    pub(crate) fact_appended: u64,
    pub(crate) fact_skipped: u64,
    pub(crate) fact_failed: u64,
    pub(crate) failed_fact_writes: Vec<FactWriteResult>,
}

impl AttemptAssessment {
    pub(crate) fn from_report(
        report: &TerminalReport,
        phase: CalibrationPhase,
        protocol: CompatibilityProtocol,
        process: Option<&CaptureProcessEvidence>,
        retained_target_packets: Option<u64>,
    ) -> Self {
        let snapshot = &report.snapshot;
        let interrupted = snapshot.outcome == SessionOutcome::Interrupted;
        let failed = !snapshot.failures.is_empty();
        let outcome = terminal_calibration_outcome(
            phase,
            protocol,
            &snapshot.observations,
            interrupted,
            failed,
        );
        let finalization = match snapshot.outcome {
            SessionOutcome::Complete if report.is_complete() => "complete",
            SessionOutcome::Complete | SessionOutcome::Partial => "partial",
            SessionOutcome::Interrupted => "interrupted",
            SessionOutcome::Failed => "failed",
            _ => "unavailable",
        };
        let artifact_writes = if report.artifacts.is_empty() {
            "unavailable"
        } else if report.artifacts.iter().any(|artifact| {
            artifact.required && !matches!(artifact.status, ArtifactStatus::Written)
        }) {
            "partial"
        } else {
            "required-write-results-complete"
        };
        let cleanup = if snapshot.cleanup.is_empty() {
            "unavailable"
        } else if snapshot.cleanup.iter().all(|result| {
            matches!(
                result.status,
                CleanupStatus::Released | CleanupStatus::NotNeeded
            )
        }) {
            "released"
        } else {
            "unresolved"
        };
        let process_completeness = match process {
            None => "unavailable",
            Some(evidence)
                if evidence.events_unretained > 0
                    || evidence.stage_transitions_unretained > 0
                    || evidence.unparseable_events > 0
                    || evidence.watcher_report.as_ref().is_some_and(|report| {
                        report.events_lost > 0 || report.buffers_lost > 0
                    }) =>
            {
                "partial"
            }
            // Watcher termination alone cannot establish trace/manifest completeness.
            // Their finalized artifact projection owns that fact.
            Some(_) => "unavailable",
        };
        let mut value = Self {
            verdict: "inconclusive",
            reason: "evidence-inconclusive",
            earliest_boundary: "unavailable",
            outcome,
            proxy: snapshot.proxy_diagnostics.as_deref().cloned(),
            observation_records: 0,
            owner_release_records: 0,
            unavailable_window_records: 0,
            matched_records: 0,
            flow_only_records: 0,
            ambiguous_records: 0,
            unavailable_correlation_records: 0,
            other_owner_records: 0,
            final_client_records: 0,
            retained_target_packets,
            finalization,
            artifact_writes,
            process_completeness,
            cleanup,
            fact_appended: 0,
            fact_skipped: 0,
            fact_failed: 0,
            failed_fact_writes: Vec::new(),
        };
        for observation in &snapshot.observations {
            match observation.evidence_window {
                EvidenceWindow::Observation => value.observation_records += 1,
                EvidenceWindow::OwnerRelease => value.owner_release_records += 1,
                _ => value.unavailable_window_records += 1,
            }
            match observation.correlation_state {
                CorrelationState::Matched => value.matched_records += 1,
                CorrelationState::FlowOnly => value.flow_only_records += 1,
                CorrelationState::Ambiguous => value.ambiguous_records += 1,
                CorrelationState::Unavailable => value.unavailable_correlation_records += 1,
            }
            if observation.evidence_window == EvidenceWindow::Observation
                && (fragcap::deep_capture::observation_is_correlated_to_final_client(observation)
                    || snapshot.controlled
                        && observation.role.as_deref() == Some("client")
                        && observation.attribution.as_deref() == Some("controlled-harness"))
            {
                value.final_client_records += 1;
            } else if observation
                .role
                .as_deref()
                .is_some_and(|role| !matches!(role, "client" | "target" | "unknown"))
            {
                value.other_owner_records += 1;
            }
        }
        for write in &snapshot.fact_writes {
            match write.status {
                FactWriteStatus::Appended => value.fact_appended += 1,
                FactWriteStatus::Skipped { .. } => value.fact_skipped += 1,
                FactWriteStatus::Failed { .. } => {
                    value.fact_failed += 1;
                    value.failed_fact_writes.push(write.clone());
                }
                _ => {}
            }
        }
        value.earliest_boundary = earliest_boundary(
            process,
            value.final_client_records > 0,
            snapshot.target.launch_case,
        );
        let (verdict, reason) = if interrupted {
            ("interrupted", "operator-interrupted")
        } else if failed || value.fact_failed > 0 || finalization != "complete" {
            ("failed", "session-or-reporting-failed")
        } else if matches!(
            outcome,
            CalibrationOutcome::ReachedClient | CalibrationOutcome::LocalCaAccepted
        ) {
            ("calibrated", "eligible-final-client-evidence")
        } else if value
            .proxy
            .as_ref()
            .is_some_and(|proxy| proxy.accepted_connections > 0)
            && value.final_client_records == 0
        {
            ("inconclusive", "final-client-correlation-missing")
        } else if value.owner_release_records > 0 && value.observation_records == 0 {
            ("inconclusive", "observation-window-evidence-missing")
        } else {
            ("inconclusive", "evidence-inconclusive")
        };
        value.verdict = verdict;
        value.reason = reason;
        value
    }

    pub(crate) fn json(&self) -> Value {
        let proxy = self.proxy.as_ref().map(|proxy| json!({
            "accepted_connections": proxy.accepted_connections,
            "authenticated_connections": proxy.authenticated_connections,
            "authentication_refused": proxy.authentication_refused,
            "saturated_connections": proxy.saturated_connections,
            "completed_connections": proxy.completed_connections,
            "failed_connections": proxy.failed_connections,
            "forced_connections": proxy.forced_connections,
            "live_connections": proxy.live_connections,
            "incomplete_connections": proxy.incomplete_connections,
            "connections_reconcile": proxy.connections_reconcile(),
            "terminal_details_reconcile": proxy.terminal_details_reconcile(),
            "causes_reconcile": proxy.causes_reconcile(),
            "http1_exchanges_completed": proxy.http1_exchanges_completed,
            "http2_streams_completed": proxy.http2_streams_completed,
            "http3_streams_completed": proxy.http3_streams_completed,
            "response_heads": proxy.response_heads,
            "connection_details_lost": proxy.connection_details_lost,
            "connection_details_unavailable": proxy.connection_details_unavailable,
            "failure_details_lost": proxy.failure_details_lost,
            "observations_lost": proxy.observations_lost,
            "causes": {
                "authentication": proxy.causes.authentication, "protocol": proxy.causes.protocol,
                "transport": proxy.causes.transport, "upstream": proxy.causes.upstream,
                "timeout": proxy.causes.timeout, "cancelled": proxy.causes.cancelled,
                "unavailable": proxy.causes.unavailable,
            },
            "connections": proxy.connections.iter().map(|record| json!({
                "connection_id": record.connection_id, "terminal": record.terminal,
                "category": record.cause.map(|cause| cause.as_str()), "code": record.code,
            })).collect::<Vec<_>>(),
        }));
        json!({
            "schema_version": 1, "verdict": self.verdict, "reason": self.reason,
            "outcome": self.outcome.to_string(), "earliest_boundary": self.earliest_boundary,
            "proxy": proxy, "retained_target_packets": self.retained_target_packets,
            "evidence_windows": {
                "observation_records": self.observation_records, "owner_release_records": self.owner_release_records,
                "unavailable_records": self.unavailable_window_records,
            },
            "ownership": {
                "matched_records": self.matched_records, "flow_only_records": self.flow_only_records,
                "ambiguous_records": self.ambiguous_records, "unavailable_records": self.unavailable_correlation_records,
                "other_owner_records": self.other_owner_records, "eligible_final_client_records": self.final_client_records,
                "population": "retained-observation-records",
            },
            "finalization": self.finalization, "artifact_writes": self.artifact_writes,
            "artifact_access": "not-assessed", "process_completeness": self.process_completeness,
            "cleanup": self.cleanup,
            "fact_writes": { "appended": self.fact_appended, "skipped": self.fact_skipped, "failed": self.fact_failed },
            "fact_write_results": self.failed_fact_writes.iter().filter_map(|write| {
                if let FactWriteStatus::Failed { code, detail } = &write.status {
                    Some(json!({ "kind": write.fact.kind, "status": "failed", "code": code, "detail": detail }))
                } else { None }
            }).collect::<Vec<_>>(),
        })
    }
}

fn earliest_boundary(
    process: Option<&CaptureProcessEvidence>,
    final_client: bool,
    launch_case: fragcap::deep_capture::api::LaunchCase,
) -> &'static str {
    let Some(process) = process else {
        return "unavailable";
    };
    let steam = matches!(
        launch_case,
        fragcap::deep_capture::api::LaunchCase::SteamProtocolCold
            | fragcap::deep_capture::api::LaunchCase::SteamProtocolWarm
    );
    let matched = |role| {
        process.stage_transitions.iter().any(|transition| {
            transition.kind == StageTransitionKind::Matched && transition.role == role
        })
    };
    if process.launch_pid.is_none() {
        "managed-launch"
    } else if steam && !matched("platform") {
        "owned-platform-binding"
    } else if steam && process.stop_reason.as_deref() == Some("platformdispatchfailed") {
        "title-dispatch"
    } else if !matched("client") && !matched("target") {
        "final-client-acquisition"
    } else if !final_client {
        "final-client-correlation"
    } else {
        "protocol-observation"
    }
}

/// Stored exact-case evidence remains separate from this invocation's process state.
#[derive(Clone, Debug)]
pub(crate) struct StoredCaseAssessment {
    pub(crate) target_id: i64,
    pub(crate) case: fragcap::deep_capture::api::CompatibilityCase,
    pub(crate) verdict: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) routing: fragcap::deep_capture::api::CalibrationEvidenceAssessment,
    pub(crate) protocol: Option<fragcap::deep_capture::api::CalibrationEvidenceAssessment>,
    pub(crate) deep_capture_may_proceed: bool,
}

impl StoredCaseAssessment {
    pub(crate) fn from_facts(
        target_id: i64,
        case: &fragcap::deep_capture::api::CompatibilityCase,
        facts: &[fragcap::deep_capture::api::StoredCompatibilityFact],
    ) -> Self {
        use fragcap::deep_capture::api::{
            assess_calibration_evidence, CalibrationEvidenceAssessment, CompatibilityFactKey,
        };
        let routing = assess_calibration_evidence(
            facts,
            Some(target_id),
            CompatibilityFactKey::ProxyRouting,
            case,
            "reached-client",
        );
        let protocol = (!matches!(
            case.protocol,
            CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable
        ))
        .then(|| {
            assess_calibration_evidence(
                facts,
                Some(target_id),
                CompatibilityFactKey::Inspectability,
                case,
                "full",
            )
        });
        let needs = match routing {
            CalibrationEvidenceAssessment::Needs(reason) => Some(reason),
            CalibrationEvidenceAssessment::Positive => protocol.and_then(|result| match result {
                CalibrationEvidenceAssessment::Positive => None,
                CalibrationEvidenceAssessment::Needs(reason) => Some(reason),
            }),
        };
        let (verdict, reason) = match needs {
            None => ("calibrated", "current-exact-case-evidence"),
            Some(reason) => (
                match reason {
                    fragcap::deep_capture::api::CalibrationProposalReason::Conflict => {
                        "inconclusive"
                    }
                    fragcap::deep_capture::api::CalibrationProposalReason::Stale => "stale",
                    fragcap::deep_capture::api::CalibrationProposalReason::ContextMismatch => {
                        "inapplicable"
                    }
                    _ => "incomplete",
                },
                reason.as_str(),
            ),
        };
        Self {
            target_id,
            case: case.clone(),
            verdict,
            reason,
            routing,
            protocol,
            // This is the shipped stored routing prerequisite. Fresh launch,
            // authorization and other preflight requirements remain independent.
            deep_capture_may_proceed: routing == CalibrationEvidenceAssessment::Positive,
        }
    }

    pub(crate) fn json(&self) -> Value {
        use fragcap::deep_capture::api::CalibrationEvidenceAssessment;
        let state = |assessment| match assessment {
            CalibrationEvidenceAssessment::Positive => {
                json!({"status":"calibrated", "reason":"current-exact-case-evidence"})
            }
            CalibrationEvidenceAssessment::Needs(reason) => {
                json!({"status":"incomplete", "reason":reason.as_str()})
            }
        };
        json!({
            "verdict": self.verdict, "reason": self.reason,
            "routing": state(self.routing), "protocol": self.protocol.map(state),
            "deep_capture_routing_prerequisite_satisfied": self.deep_capture_may_proceed,
            "launch_readiness": "independent-fresh-preflight-required",
            "identity": {
                "target_id": self.target_id, "launch_case": self.case.launch_case.as_str(),
                "routing_strategy": self.case.routing_strategy.as_str(), "address_family": self.case.address_family.as_str(),
                "proxy_backend": self.case.proxy_backend, "proxy_backend_version": self.case.proxy_backend_version,
                "fragcap_version": self.case.fragcap_version, "target_version": self.case.target_version,
                "protocol": self.case.protocol.as_str(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap::deep_capture::api::*;
    use fragcap::targets::{CompatibilityEvidenceSource, CompatibilityFactKey};

    fn case(protocol: CompatibilityProtocol) -> CompatibilityCase {
        CompatibilityCase {
            launch_case: CompatibilityLaunchCase::DirectExeCold,
            proxy_backend: "native".into(),
            proxy_backend_version: "1".into(),
            routing_strategy: CompatibilityRoutingStrategy::ChildEnvironment,
            address_family: CompatibilityAddressFamily::Ipv4,
            protocol,
            fragcap_version: "controlled-version".into(),
            target_version: Some("build1".into()),
        }
    }

    fn fact(
        key: CompatibilityFactKey,
        value: &str,
        current: &CompatibilityCase,
    ) -> StoredCompatibilityFact {
        let mut fact =
            StoredCompatibilityFact::new(7, key, value, CompatibilityEvidenceSource::ObservedRun)
                .unwrap();
        fact.launch_case = Some(current.launch_case);
        fact.proxy_backend = Some(current.proxy_backend.clone());
        fact.proxy_backend_version = Some(current.proxy_backend_version.clone());
        fact.routing_strategy = Some(current.routing_strategy);
        fact.address_family = Some(current.address_family);
        fact.protocol = Some(if key == CompatibilityFactKey::ProxyRouting {
            CompatibilityProtocol::NotApplicable
        } else {
            current.protocol
        });
        fact.fragcap_version = Some(current.fragcap_version.clone());
        fact.target_version = current.target_version.clone();
        fact
    }

    fn report() -> TerminalReport {
        TerminalReport {
            snapshot: TerminalSnapshot {
                session_id: "controlled-attempt".into(),
                plan_id: PlanId::new("controlled-plan"),
                target: PreparedTarget {
                    id: 7,
                    handle: "synthetic-target".into(),
                    launch_case: LaunchCase::DirectExeCold,
                },
                mode: SessionMode::ReachabilityCalibration,
                controlled: false,
                artifacts: ArtifactRequests {
                    har: false,
                    key_log: false,
                    sensitive_retention: SensitiveRetention::Retain,
                },
                outcome: SessionOutcome::Complete,
                lifecycle_transitions: Vec::new(),
                observations: Vec::new(),
                classification_records_lost: 0,
                application_classification_summary: None,
                proxy_diagnostics: Some(Box::new(ProxyDiagnostics {
                    accepted_connections: 6,
                    authenticated_connections: 1,
                    failed_connections: 6,
                    http1_exchanges_completed: 7,
                    response_heads: 7,
                    connection_details_unavailable: 6,
                    causes: ProxyCauseCounts {
                        protocol: 5,
                        timeout: 1,
                        ..ProxyCauseCounts::default()
                    },
                    ..ProxyDiagnostics::default()
                })),
                evidence_windows: EvidenceWindows::default(),
                route_verification: None,
                failures: Vec::new(),
                fact_writes: Vec::new(),
                cleanup: vec![CleanupResult {
                    resource: "native-proxy-listener".into(),
                    status: CleanupStatus::Released,
                    reason: "not a diagnostic input".into(),
                }],
                deadlines: Deadlines::default(),
                finished_at: std::time::SystemTime::UNIX_EPOCH,
            },
            artifacts: Vec::new(),
            event_failures: Vec::new(),
        }
    }

    fn observation(window: EvidenceWindow, owner: Option<&str>) -> CompatibilityObservation {
        CompatibilityObservation {
            evidence_window: window,
            flow_id: None,
            proxy_connection_id: "1".into(),
            client_peer: None,
            proxy_local: None,
            observed_at: "42".into(),
            process_id: None,
            process_image: None,
            role: owner.map(str::to_string),
            attribution: None,
            packet_observations: 0,
            packet_observations_unretained: 0,
            correlation_state: CorrelationState::Unavailable,
            correlation_reason: "packet-flow-not-observed".into(),
            protocol: "http".into(),
            inspectability: Inspectability::Full,
            method: Some("GET".into()),
            url: None,
            status: Some(200),
            reason: None,
            classification: ProtocolClassification::new(
                TrafficFamily::Http1,
                DetectionState::Identified,
                InspectabilityState::Full,
                None,
            )
            .unwrap(),
        }
    }

    #[test]
    fn exchange_success_missing_ownership_partial_process_and_cleanup_are_separate() {
        let mut report = report();
        report.snapshot.observations = vec![observation(EvidenceWindow::Observation, None); 7];
        let process = CaptureProcessEvidence {
            events_unretained: 3,
            ..CaptureProcessEvidence::default()
        };
        let assessment = AttemptAssessment::from_report(
            &report,
            CalibrationPhase::Reachability,
            CompatibilityProtocol::Routing,
            Some(&process),
            Some(0),
        );
        assert_eq!(assessment.verdict, "inconclusive");
        assert_eq!(assessment.reason, "final-client-correlation-missing");
        assert_eq!(assessment.unavailable_correlation_records, 7);
        assert_eq!(assessment.process_completeness, "partial");
        assert_eq!(assessment.cleanup, "released");
        let value = assessment.json();
        assert_eq!(value["proxy"]["http1_exchanges_completed"], 7);
        assert_eq!(value["proxy"]["failed_connections"], 6);
        assert_eq!(value["proxy"]["causes"]["protocol"], 5);
        assert_eq!(value["proxy"]["causes"]["timeout"], 1);
        assert_eq!(value["proxy"]["connections_reconcile"], true);
        assert_eq!(value["proxy"]["terminal_details_reconcile"], true);
        assert_eq!(value["proxy"]["causes_reconcile"], true);
        assert!(!value.to_string().contains("not a diagnostic input"));
    }

    #[test]
    fn late_only_and_mixed_ownership_never_authorize_an_earlier_attempt() {
        let mut report = report();
        let mut late = observation(EvidenceWindow::OwnerRelease, Some("client"));
        late.attribution = Some("controlled-harness".into());
        report.snapshot.controlled = true;
        report.snapshot.observations = vec![
            late,
            observation(EvidenceWindow::Observation, Some("launcher")),
        ];
        let assessment = AttemptAssessment::from_report(
            &report,
            CalibrationPhase::Reachability,
            CompatibilityProtocol::Routing,
            None,
            None,
        );
        assert_eq!(assessment.final_client_records, 0);
        assert_eq!(assessment.owner_release_records, 1);
        assert_eq!(assessment.other_owner_records, 1);
        assert_ne!(assessment.verdict, "calibrated");
    }

    #[test]
    fn current_exact_facts_survive_unrelated_attempt_and_all_identity_mismatches_are_inapplicable()
    {
        let current = case(CompatibilityProtocol::Https);
        let facts = vec![
            fact(
                CompatibilityFactKey::ProxyRouting,
                "reached-client",
                &current,
            ),
            fact(CompatibilityFactKey::Inspectability, "full", &current),
        ];
        assert_eq!(
            StoredCaseAssessment::from_facts(7, &current, &facts).verdict,
            "calibrated"
        );
        let attempt = AttemptAssessment::from_report(
            &report(),
            CalibrationPhase::Reachability,
            CompatibilityProtocol::Routing,
            None,
            None,
        );
        assert_eq!(attempt.verdict, "inconclusive");
        assert_eq!(
            StoredCaseAssessment::from_facts(7, &current, &facts).verdict,
            "calibrated"
        );
        for dimension in 0..8 {
            let mut changed = current.clone();
            match dimension {
                0 => changed.launch_case = CompatibilityLaunchCase::SteamProtocolCold,
                1 => changed.proxy_backend = "other".into(),
                2 => changed.proxy_backend_version = "2".into(),
                3 => changed.routing_strategy = CompatibilityRoutingStrategy::TargetConfiguration,
                4 => changed.address_family = CompatibilityAddressFamily::Ipv6,
                5 => changed.fragcap_version = "different".into(),
                6 => changed.target_version = Some("build2".into()),
                _ => changed.protocol = CompatibilityProtocol::Http2,
            }
            assert_ne!(
                StoredCaseAssessment::from_facts(7, &changed, &facts).verdict,
                "calibrated",
                "dimension {dimension}"
            );
        }
        assert_ne!(
            StoredCaseAssessment::from_facts(8, &current, &facts).verdict,
            "calibrated"
        );
    }

    #[test]
    fn stale_and_conflicting_current_fact_values_cannot_look_ready() {
        let current = case(CompatibilityProtocol::Routing);
        let positive = fact(
            CompatibilityFactKey::ProxyRouting,
            "reached-client",
            &current,
        );
        let mut stale = positive.clone();
        stale.stale = true;
        assert_eq!(
            StoredCaseAssessment::from_facts(7, &current, &[stale]).verdict,
            "stale"
        );
        let negative = fact(CompatibilityFactKey::ProxyRouting, "inconclusive", &current);
        let conflict = StoredCaseAssessment::from_facts(7, &current, &[positive, negative]);
        assert_eq!(conflict.verdict, "inconclusive");
        assert_eq!(conflict.reason, "conflict");
        assert!(!conflict.deep_capture_may_proceed);
    }

    #[test]
    fn interrupted_attempt_and_unresolved_cleanup_do_not_erase_other_populations() {
        let mut report = report();
        report.snapshot.outcome = SessionOutcome::Interrupted;
        report.snapshot.cleanup[0].status = CleanupStatus::TimedOut;
        let assessment = AttemptAssessment::from_report(
            &report,
            CalibrationPhase::Reachability,
            CompatibilityProtocol::Routing,
            None,
            None,
        );
        assert_eq!(assessment.verdict, "interrupted");
        assert_eq!(assessment.cleanup, "unresolved");
        assert_eq!(
            assessment.proxy.as_ref().unwrap().http1_exchanges_completed,
            7
        );
    }

    #[test]
    fn failed_fact_results_preserve_exact_detail_and_independent_status_counts() {
        let mut report = report();
        let candidate = fragcap::deep_capture::api::CompatibilityFact {
            kind: "proxy-routing".into(),
            value: "inconclusive".into(),
            evidence: "controlled".into(),
            phase: CalibrationPhase::Reachability,
            protocol: CompatibilityProtocol::Routing,
            final_owner_index: None,
        };
        report.snapshot.fact_writes = vec![
            FactWriteResult {
                fact: candidate.clone(),
                status: FactWriteStatus::Appended,
            },
            FactWriteResult {
                fact: candidate.clone(),
                status: FactWriteStatus::Skipped {
                    reason: "no target".into(),
                },
            },
            FactWriteResult {
                fact: candidate,
                status: FactWriteStatus::Failed {
                    code: "fact-store-failed".into(),
                    detail: "controlled database write failed".into(),
                },
            },
        ];
        let assessment = AttemptAssessment::from_report(
            &report,
            CalibrationPhase::Reachability,
            CompatibilityProtocol::Routing,
            None,
            None,
        );
        let value = assessment.json();
        assert_eq!(
            value["fact_writes"],
            json!({ "appended": 1, "skipped": 1, "failed": 1 })
        );
        assert_eq!(
            value["fact_write_results"],
            json!([{
                "kind": "proxy-routing", "status": "failed", "code": "fact-store-failed", "detail": "controlled database write failed",
            }])
        );
        assert_eq!(assessment.verdict, "failed");
    }
}
