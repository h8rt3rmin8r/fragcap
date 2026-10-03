// SPDX-License-Identifier: Apache-2.0

//! Human projections of existing native session authority, never new policy.

use fragcap::deep_capture::api::{
    ArtifactResult, ArtifactStatus, CleanupResult, CleanupStatus, DeepCaptureEvent,
    InspectabilityState as Inspectability, SessionOutcome,
};
use serde_json::Value;

use crate::display::{display_width, human_display_value, wrap_hanging};

fn display_value(value: &str) -> String {
    human_display_value(value)
        .chars()
        .map(|character| {
            if character.is_control() {
                format!("\\u{{{:x}}}", character as u32)
            } else {
                character.to_string()
            }
        })
        .collect()
}

pub(crate) fn wrapped(text: &str, width: usize) -> String {
    text.lines()
        .map(|line| format!("{}\n", wrap_hanging(line, 0, width.clamp(40, 80))))
        .collect()
}

/// Exact values are never reflowed internally, even when wider than the terminal.
fn exact_detail(label: &str, value: &str, width: usize) -> String {
    let value = display_value(value);
    let line = format!("{label}: {value}");
    if display_width(&line) <= width.clamp(40, 80) {
        format!("{line}\n")
    } else {
        format!("{label}:\n{value}\n")
    }
}

pub(crate) fn lifecycle_progress(event: &DeepCaptureEvent, width: usize) -> Option<String> {
    let text = match event {
        DeepCaptureEvent::Plan { .. } => "Deep Capture preflight passed; the exact authorized plan is ready.",
        DeepCaptureEvent::ProxyStarted { .. } => "Native proxy ready on the session-owned loopback listener. No target traffic or decryption is inferred.",
        DeepCaptureEvent::TrustAcquired { .. } => "Trust readiness step completed. Target CA acceptance remains unobserved until eligible traffic proves it.",
        DeepCaptureEvent::LaunchStarted { .. } => "Managed target launch started with child-scoped routing. Final-client ownership and proxy reachability remain to be observed.",
        DeepCaptureEvent::Started { .. } => "Observing the authorized session. Ctrl+C requests bounded shutdown and cleanup; retained evidence is not automatically deleted.",
        DeepCaptureEvent::Cleanup { .. } => return None,
        _ => return None,
    };
    Some(wrapped(text, width))
}

pub(crate) fn authorization_summary(plan: &Value, width: usize) -> String {
    let label = |pointer: &str| {
        display_value(
            plan.pointer(pointer)
                .and_then(Value::as_str)
                .unwrap_or("unavailable"),
        )
    };
    let selection = |pointer: &str| {
        if plan.pointer(pointer).is_some_and(Value::is_string) {
            "selected"
        } else {
            "not selected"
        }
    };
    let mut text = wrapped(
        "Deep Capture: active target-scoped inspection. Capture remains passive.",
        width,
    );
    text.push_str(&exact_detail("Target", &label("/target/name"), width));
    text.push_str(&wrapped(&format!(
        "Target id: {}. Launch case: {}.\n\
         Routing: {}. No system-wide proxy change.\n\
         CA trust: {}. Only exact session-owned additions are removed during cleanup.\n\
         CA store: {}.\n\
         Separate CA trust and sensitive-output consent remain required by the complete plan; this summary does not replace authorization.\n\
         Payload retention: {}.\nHAR: {}.\nTLS key log: {}.\nClient identity: {}.\n\
         Sensitive evidence: {}. Key logs contain secret TLS material when selected; supplied client identity is used only for upstream client authentication.\n\
         Evidence is retained after external resource cleanup; review before sharing.\n\
         Cleanup stops owned capture/proxy work, removes only session-added trust, and retains exact recovery records for incomplete obligations. Run fragcap doctor --fix to review unresolved residue with explicit confirmation.\n\
         Review the complete canonical plan below, including deadlines, before authorizing its exact identifier.\n",
        plan.pointer("/target/stable_id").unwrap_or(&Value::Null),
        label("/launch/observed_case"), label("/proxy/routing_scope"), label("/trust/action"),
        if plan.pointer("/trust/action").and_then(Value::as_str) == Some("none") {
            "none".to_string()
        } else { label("/trust/store") },
        if plan.pointer("/capture/payload_retention").and_then(Value::as_bool) == Some(false) {
            "disabled"
        } else { "enabled" },
        selection("/artifacts/har"), selection("/artifacts/key_log"),
        selection("/artifacts/client_certificate"), label("/artifacts/sensitivity"),
    ), width));
    text.push_str(&exact_detail("Bundle", &label("/artifacts/bundle"), width));
    text
}

#[derive(Default)]
pub(crate) struct SessionProgress {
    total: u64,
    classes: [u64; 6],
}

impl SessionProgress {
    pub(crate) fn observe(&mut self, class: Inspectability, width: usize) -> Option<String> {
        let index = match class {
            Inspectability::Full => 0,
            Inspectability::MetadataOnly => 1,
            Inspectability::DecryptedUnknown => 2,
            Inspectability::EncryptedOpaque => 3,
            Inspectability::PacketOnly => 4,
            Inspectability::Unavailable => 5,
            _ => 5,
        };
        self.total = self.total.saturating_add(1);
        self.classes[index] = self.classes[index].saturating_add(1);
        if self.total != 1 && !self.total.is_multiple_of(100) {
            return None;
        }
        Some(wrapped(&format!(
            "Observed application counters: observations={}, full={}, metadata-only={}, decrypted-unknown={}, encrypted-opaque={}, packet-only={}, unavailable={}. Live loss and process ownership are unavailable here; terminal artifacts reconcile them. These are observed records, not total traffic or target compatibility.\n",
            self.total, self.classes[0], self.classes[1], self.classes[2],
            self.classes[3], self.classes[4], self.classes[5],
        ), width))
    }
}

pub(crate) fn terminal_summary(
    outcome: SessionOutcome,
    complete: bool,
    artifacts: &[ArtifactResult],
    cleanup: &[CleanupResult],
    width: usize,
) -> String {
    let outcome = match outcome {
        SessionOutcome::Complete if complete => "complete",
        SessionOutcome::Complete | SessionOutcome::Partial => "partial",
        SessionOutcome::Interrupted => "interrupted",
        SessionOutcome::Failed => "failed",
        _ => "unavailable",
    };
    let mut text = wrapped(&format!("Deep Capture outcome: {outcome}. Session outcome, artifact completeness, and resource cleanup are independent.\n"), width);
    let mut retained = false;
    for artifact in artifacts {
        let status = match &artifact.status {
            ArtifactStatus::Written => "written",
            ArtifactStatus::Omitted { .. } => "omitted",
            ArtifactStatus::Failed { .. } => "failed",
            _ => "unavailable",
        };
        let present =
            !matches!(artifact.status, ArtifactStatus::Omitted { .. }) && artifact.path.is_file();
        retained |= present;
        text.push_str(&wrapped(
            &format!("Artifact {}: {status}.\n", display_value(&artifact.role),),
            width,
        ));
        text.push_str(&exact_detail(
            if present {
                "retained at"
            } else {
                "no retained file confirmed at"
            },
            &artifact.path.display().to_string(),
            width,
        ));
    }
    if !retained {
        text.push_str("No retained artifact was confirmed.\n");
    } else {
        text.push_str(&wrapped("Retained evidence may be sensitive or incomplete. External resource cleanup does not delete it; review before sharing.", width));
    }
    let mut unresolved = false;
    for result in cleanup {
        let status = match result.status {
            CleanupStatus::Released => "released",
            CleanupStatus::NotNeeded => "not-needed",
            CleanupStatus::TimedOut => "timed-out",
            CleanupStatus::Failed => "failed",
            _ => "unavailable",
        };
        unresolved |= !matches!(
            result.status,
            CleanupStatus::Released | CleanupStatus::NotNeeded
        );
        text.push_str(&exact_detail(
            &format!("Cleanup {}", display_value(&result.resource)),
            status,
            width,
        ));
        text.push_str(&wrapped(&display_value(&result.reason), width));
    }
    if cleanup.is_empty() {
        text.push_str(&wrapped(
            "No cleanup result was reported; no absence of residue is inferred.",
            width,
        ));
    }
    if unresolved {
        text.push_str(&wrapped("Unresolved owned resource obligations remain. Run fragcap doctor --fix to inspect exact recovery records and review repair with explicit confirmation. Never remove trust or files by name alone.", width));
    }
    if outcome != "complete" {
        text.push_str(&wrapped("Review retained evidence and the reported failure or interruption before retrying. A retry requires a freshly prepared and separately authorized session; retained records never authorize new effects.", width));
    }
    text
}

/// A causal calibration projection from the session's retained authorities.
/// Unknown counters remain unavailable; artifact creation is never reachability.
pub(crate) fn calibration_diagnosis(
    snapshot: &fragcap::deep_capture::api::TerminalSnapshot,
    process: Option<&fragcap::deep_capture::CaptureProcessEvidence>,
    retained_target_packets: Option<u64>,
    width: usize,
) -> String {
    use fragcap::deep_capture::api::{FactWriteStatus, RouteVerificationState};

    let route_reached = snapshot
        .route_verification
        .as_ref()
        .is_some_and(|route| route.state == RouteVerificationState::ReachedSocketOwner);
    let stage = calibration_stage(snapshot.target.launch_case, process, route_reached);
    let proxy_accepted = snapshot
        .cleanup
        .iter()
        .find(|result| result.resource == "native-proxy-listener")
        .and_then(|result| result.reason.strip_prefix("accepted="))
        .and_then(|tail| tail.split(',').next())
        .and_then(|value| value.parse::<u64>().ok());
    let fact_writes = if snapshot.fact_writes.is_empty() {
        "none reported".to_string()
    } else {
        snapshot
            .fact_writes
            .iter()
            .map(|write| {
                let status = match &write.status {
                    FactWriteStatus::Appended => "appended",
                    FactWriteStatus::Skipped { .. } => "skipped",
                    FactWriteStatus::Failed { .. } => "failed",
                    _ => "unavailable",
                };
                format!("{}={} ({status})", write.fact.kind, write.fact.value)
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    wrapped(&format!(
        "Calibration diagnosis: earliest known stage: {stage}. Target packets retained: {}. Proxy connections accepted: {}. Compatibility facts: {fact_writes}. Route verification: {}. These values do not infer later stages from written artifacts.",
        retained_target_packets.map_or_else(|| "unavailable".to_string(), |count| count.to_string()),
        proxy_accepted.map_or_else(|| "unavailable".to_string(), |count| count.to_string()),
        snapshot.route_verification.as_ref().map_or("unavailable", |route| route.state.as_str()),
    ), width)
}

fn calibration_stage(
    launch_case: fragcap::deep_capture::api::LaunchCase,
    process: Option<&fragcap::deep_capture::CaptureProcessEvidence>,
    route_reached: bool,
) -> &'static str {
    let platform = process.is_some_and(|evidence| {
        evidence.stage_transitions.iter().any(|transition| {
            transition.kind == fragcap::deep_capture::StageTransitionKind::Matched
                && transition.role == "platform"
        })
    });
    let client = process.is_some_and(|evidence| {
        evidence.stage_transitions.iter().any(|transition| {
            transition.kind == fragcap::deep_capture::StageTransitionKind::Matched
                && transition.role == "client"
        })
    });
    let stop = process.and_then(|evidence| evidence.stop_reason.as_deref());
    let steam = matches!(
        launch_case,
        fragcap::deep_capture::api::LaunchCase::SteamProtocolCold
            | fragcap::deep_capture::api::LaunchCase::SteamProtocolWarm
    );
    if process.is_none() {
        "unavailable"
    } else if process.is_some_and(|evidence| evidence.launch_pid.is_none()) {
        if steam {
            "managed platform launch"
        } else {
            "managed client launch"
        }
    } else if steam && !platform {
        "owned platform binding"
    } else if steam && stop == Some("platformdispatchfailed") {
        "title dispatch"
    } else if !client {
        "final client acquisition"
    } else if route_reached {
        "later protocol observation"
    } else {
        "proxy reachability"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::display_width;
    use fragcap::deep_capture::api::{ArtifactStatus, CleanupStatus, Sensitivity};
    use serde_json::json;

    #[test]
    fn calibration_stage_uses_only_proven_launch_and_client_edges() {
        use fragcap::deep_capture::api::LaunchCase;
        use fragcap::deep_capture::{CaptureProcessEvidence, StageTransition, StageTransitionKind};
        use fragcap::Timestamp;

        let mut evidence = CaptureProcessEvidence::default();
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, Some(&evidence), false),
            "managed platform launch"
        );
        evidence.launch_pid = Some(41);
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, Some(&evidence), false),
            "owned platform binding"
        );
        assert_eq!(
            super::calibration_stage(LaunchCase::DirectExeCold, Some(&evidence), false),
            "final client acquisition"
        );
        let matched = |role: &str| StageTransition {
            kind: StageTransitionKind::Matched,
            pid: 41,
            role: role.into(),
            stage: None,
            at: Timestamp::from_nanos(1),
        };
        evidence.stage_transitions.push(matched("platform"));
        evidence.stop_reason = Some("platformdispatchfailed".into());
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, Some(&evidence), false),
            "title dispatch"
        );
        evidence.stop_reason = None;
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, Some(&evidence), false),
            "final client acquisition"
        );
        evidence.stage_transitions.push(matched("client"));
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, Some(&evidence), false),
            "proxy reachability"
        );
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, Some(&evidence), true),
            "later protocol observation"
        );
        assert_eq!(
            super::calibration_stage(LaunchCase::SteamProtocolCold, None, false),
            "unavailable"
        );
    }

    fn plan(trust: bool, sensitive: bool) -> Value {
        json!({
            "mode":"capture",
            "target":{"name":"界 Game", "stable_id":75000},
            "launch":{"observed_case":"direct-exe-cold"},
            "proxy":{"routing_scope":"managed child environment only", "system_proxy_change":false},
            "trust":{"action":if trust {"ensure exact session CA"} else {"none"}},
            "capture":{"payload_retention":false},
            "artifacts":{
                "bundle":"C:/local evidence/界",
                "sensitivity":"bundle may contain plaintext application traffic and credentials",
                "har":if sensitive {Some("capture.har")} else {None},
                "key_log":if sensitive {Some("tls-keylog.log")} else {None},
                "client_certificate":if sensitive {Some("client.pem")} else {None}
            }
        })
    }

    #[test]
    fn authorization_consequences_preserve_selected_and_unselected_authority() {
        for trust in [false, true] {
            for sensitive in [false, true] {
                let value = plan(trust, sensitive);
                let before = value.clone();
                let text = authorization_summary(&value, 80);
                assert_eq!(value, before);
                assert!(text.contains("active target-scoped inspection"));
                assert!(text.contains("Capture remains passive"));
                assert!(text.contains("界 Game") && text.contains("75000"));
                assert!(text.contains("managed child environment only"));
                assert_ne!(text.contains("CA trust: none"), trust);
                assert!(text.contains("HAR: selected") == sensitive);
                assert!(text.contains("TLS key log: selected") == sensitive);
                assert!(text.contains("Client identity: selected") == sensitive);
                assert!(text.contains("Payload retention: disabled"));
                assert!(text.contains("retained") && text.contains("credentials"));
                assert!(text.contains("Separate") && text.contains("consent"));
            }
        }
    }

    #[test]
    fn observed_counters_are_fixed_sampled_and_do_not_claim_ownership() {
        let mut progress = SessionProgress::default();
        let first = progress
            .observe(Inspectability::EncryptedOpaque, 80)
            .unwrap();
        assert!(first.contains("observations=1") && first.contains("encrypted-opaque=1"));
        assert!(first.contains("ownership") && first.contains("unavailable"));
        assert!(!first.contains("target inspected") && !first.contains("decryption succeeded"));
        for _ in 2..100 {
            assert!(progress.observe(Inspectability::Full, 80).is_none());
        }
        let hundred = progress.observe(Inspectability::MetadataOnly, 80).unwrap();
        assert!(hundred.contains("observations=100"));
        assert!(hundred.contains("full=98") && hundred.contains("metadata-only=1"));
        assert!(hundred.contains("encrypted-opaque=1"));
        assert_eq!(progress.classes.iter().sum::<u64>(), progress.total);
    }

    #[test]
    fn terminal_states_separate_retained_evidence_and_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("application.jsonl");
        std::fs::write(&path, b"controlled evidence\n").unwrap();
        let artifact = ArtifactResult {
            role: "application-jsonl".into(),
            path: path.clone(),
            sensitivity: Sensitivity::Payload,
            required: true,
            status: ArtifactStatus::Written,
        };
        for (outcome, expected) in [
            (SessionOutcome::Complete, "complete"),
            (SessionOutcome::Partial, "partial"),
            (SessionOutcome::Interrupted, "interrupted"),
            (SessionOutcome::Failed, "failed"),
        ] {
            let cleanup = [CleanupResult {
                resource: "exact-ca-thumbprint".into(),
                status: CleanupStatus::Failed,
                reason: "controlled cleanup failure".into(),
            }];
            let text = terminal_summary(
                outcome,
                outcome == SessionOutcome::Complete,
                std::slice::from_ref(&artifact),
                &cleanup,
                80,
            );
            assert!(text.contains(&format!("outcome: {expected}")));
            assert!(text.contains(&path.display().to_string()) && text.contains("retained"));
            assert!(text.contains("exact-ca-thumbprint") && text.contains("failed"));
            assert!(text.contains("fragcap doctor --fix") && text.contains("confirmation"));
        }
        let released = [CleanupResult {
            resource: "proxy".into(),
            status: CleanupStatus::Released,
            reason: "owned proxy stopped".into(),
        }];
        let text = terminal_summary(SessionOutcome::Complete, false, &[], &released, 80);
        assert!(text.contains("outcome: partial"));
        assert!(text.contains("No retained artifact was confirmed"));
        assert!(!text.contains("fragcap doctor --fix"));
    }

    #[test]
    fn missing_and_failed_artifacts_never_become_complete_evidence() {
        let artifacts = [
            ArtifactResult {
                role: "manifest".into(),
                path: "missing-control-file".into(),
                sensitivity: Sensitivity::Metadata,
                required: true,
                status: ArtifactStatus::Written,
            },
            ArtifactResult {
                role: "application-jsonl".into(),
                path: "missing-application".into(),
                sensitivity: Sensitivity::Payload,
                required: true,
                status: ArtifactStatus::Failed {
                    code: "controlled".into(),
                    detail: "controlled failure".into(),
                },
            },
        ];
        let cleanup = [CleanupResult {
            resource: "routing".into(),
            status: CleanupStatus::TimedOut,
            reason: "deadline expired".into(),
        }];
        let text = terminal_summary(SessionOutcome::Partial, false, &artifacts, &cleanup, 80);
        assert!(text.contains("No retained artifact was confirmed"));
        assert!(text.contains("application-jsonl") && text.contains("failed"));
        assert!(text.contains("routing") && text.contains("timed-out"));
    }

    #[test]
    fn narrow_unicode_summary_wraps_without_truncating_values() {
        for width in 40..=80 {
            let text = authorization_summary(&plan(true, true), width);
            assert!(text.contains("界 Game"));
            assert!(text.contains("C:/local evidence/界"));
            assert!(text.lines().all(|line| display_width(line) <= width));
        }
    }

    #[test]
    fn lifecycle_progress_reports_only_observed_stage_authority() {
        let events = [
            DeepCaptureEvent::ProxyStarted {
                sequence: 1,
                session_id: "controlled".into(),
            },
            DeepCaptureEvent::TrustAcquired {
                sequence: 2,
                session_id: "controlled".into(),
            },
            DeepCaptureEvent::LaunchStarted {
                sequence: 3,
                session_id: "controlled".into(),
            },
            DeepCaptureEvent::Started {
                sequence: 4,
                session_id: "controlled".into(),
            },
        ];
        let labels = [
            "Native proxy ready",
            "Trust readiness",
            "Managed target launch",
            "Observing",
        ];
        for (event, label) in events.iter().zip(labels) {
            let text = lifecycle_progress(event, 40).unwrap();
            assert!(text.contains(label));
            assert!(!text.contains("target inspected") && !text.contains("decryption succeeded"));
            assert!(text.lines().all(|line| display_width(line) <= 40));
        }
        assert!(lifecycle_progress(
            &DeepCaptureEvent::Cleanup {
                sequence: 5,
                session_id: "controlled".into(),
                result: CleanupResult {
                    resource: "proxy".into(),
                    status: CleanupStatus::Released,
                    reason: "stopped".into(),
                },
            },
            40
        )
        .is_none());
    }

    #[test]
    fn all_inspection_classes_and_saturation_remain_explicit() {
        let mut progress = SessionProgress::default();
        for class in [
            Inspectability::Full,
            Inspectability::MetadataOnly,
            Inspectability::DecryptedUnknown,
            Inspectability::EncryptedOpaque,
            Inspectability::PacketOnly,
            Inspectability::Unavailable,
        ] {
            progress.observe(class, 80);
        }
        assert_eq!(progress.classes, [1; 6]);
        progress.total = u64::MAX;
        progress.classes[0] = u64::MAX;
        progress.observe(Inspectability::Full, 80);
        assert_eq!(progress.total, u64::MAX);
        assert_eq!(progress.classes[0], u64::MAX);
    }

    #[test]
    fn consequence_values_cannot_inject_terminal_layout_or_control_sequences() {
        let mut value = plan(true, false);
        value["target"]["name"] = json!("line\n\t\u{1b}[31m");
        let text = authorization_summary(&value, 80);
        assert!(text.contains("line\\n\\t\\u{1b}[31m"));
        assert!(!text.contains('\u{1b}') && !text.contains('\t'));
    }

    #[test]
    fn exact_names_and_paths_preserve_repeated_spaces_at_narrow_widths() {
        let mut value = plan(true, false);
        let name = "A  controlled title with a long exact name";
        let bundle = "C:/controlled  evidence/long exact bundle directory";
        value["target"]["name"] = json!(name);
        value["artifacts"]["bundle"] = json!(bundle);
        let text = authorization_summary(&value, 40);
        assert!(text.contains(name) && text.contains(bundle));
        let artifact = ArtifactResult {
            role: "application-jsonl".into(),
            path: bundle.into(),
            sensitivity: Sensitivity::Payload,
            required: true,
            status: ArtifactStatus::Written,
        };
        let text = terminal_summary(SessionOutcome::Partial, false, &[artifact], &[], 40);
        assert!(text.contains(&std::path::Path::new(bundle).display().to_string()));
    }
}
