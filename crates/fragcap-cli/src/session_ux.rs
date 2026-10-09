// SPDX-License-Identifier: Apache-2.0

//! Human projections of existing native session authority, never new policy.

use fragcap::deep_capture::api::{
    ArtifactResult, ArtifactStatus, CleanupResult, CleanupStatus, DeepCaptureEvent,
    InspectabilityState as Inspectability, SessionOutcome,
};
use serde_json::Value;

use crate::display::{human_display_value, wrap_hanging};

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

fn render_exact_fields(fields: &[(String, String)], width: usize) -> String {
    let rows: Vec<Vec<String>> = fields
        .iter()
        .map(|(label, value)| vec![human_display_value(label), human_display_value(value)])
        .collect();
    let layout = crate::display::ColumnLayout::new(0, &rows);
    rows.iter()
        .map(|row| {
            let exact = matches!(row[0].as_str(), "Target:" | "Bundle:" | "Next action:")
                || row[0].starts_with("Case ")
                || row[1].contains("; written;")
                || row[1].contains("; failed;");
            format!(
                "{}\n",
                if exact {
                    layout.render_row(row)
                } else {
                    layout.render_wrapped_row(row, width.clamp(40, 80))
                }
            )
        })
        .collect()
}

/// Exact values are never reflowed internally, even when wider than the terminal.
pub(crate) fn calibration_verdict(value: &Value, width: usize) -> String {
    let scalar = |value: &Value| match value {
        Value::Null => "unavailable".to_string(),
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    };
    let mut text = String::from("\nCalibration verdict\n\n");
    let mut fields = vec![
        ("Target:".into(), scalar(&value["target"])),
        ("Target id:".into(), scalar(&value["target_id"])),
        ("Verdict:".into(), scalar(&value["verdict"])),
        ("Workflow status:".into(), scalar(&value["status"])),
        ("Reason:".into(), scalar(&value["reason"])),
    ];
    if let Some(attempt) = value.get("attempted_case") {
        fields.push(("Attempted case:".into(), scalar(&attempt["verdict"])));
        fields.push(("Attempt reason:".into(), scalar(&attempt["reason"])));
        fields.push((
            "Earliest boundary:".into(),
            scalar(&attempt["earliest_boundary"]),
        ));
        if let Some(identity) = attempt["identity"].as_object() {
            for (key, value) in identity {
                fields.push((format!("Attempt {key}:"), scalar(value)));
            }
        }
    } else {
        fields.push(("Attempted case:".into(), "not run".into()));
    }
    if let Some(current) = value.get("current_case") {
        fields.push((
            "Current applicable readiness:".into(),
            scalar(&current["verdict"]),
        ));
        fields.push(("Readiness reason:".into(), scalar(&current["reason"])));
        fields.push((
            "Deep Capture routing prerequisite:".into(),
            scalar(&current["deep_capture_routing_prerequisite_satisfied"]),
        ));
        fields.push((
            "Launch readiness:".into(),
            scalar(
                value
                    .get("launch_readiness")
                    .unwrap_or(&current["launch_readiness"]),
            ),
        ));
        fields.push((
            "Protocol readiness:".into(),
            if current["protocol"].is_null() {
                "not assessed; routing case only".into()
            } else {
                scalar(&current["protocol"])
            },
        ));
        if let Some(identity) = current.get("identity").and_then(Value::as_object) {
            for (key, value) in identity {
                fields.push((
                    if key == "target_id" {
                        "Case stored target row:".into()
                    } else {
                        format!("Case {key}:")
                    },
                    scalar(value),
                ));
            }
        }
    } else {
        fields.push((
            "Current applicable readiness:".into(),
            "unavailable; exact current case not established".into(),
        ));
    }
    if let Some(limitations) = value["limitations"].as_array() {
        for (index, limitation) in limitations.iter().enumerate() {
            fields.push((format!("Limitation {}:", index + 1), scalar(limitation)));
        }
    }
    if let Some(error) = value["terminal_error"].as_str() {
        fields.push(("Terminal error:".into(), error.into()));
    }
    fields.push((
        "Remaining protocols:".into(),
        value["remaining_protocols"]
            .as_array()
            .map(|values| {
                if values.is_empty() {
                    "none in the established case".into()
                } else {
                    values.iter().map(&scalar).collect::<Vec<_>>().join(", ")
                }
            })
            .unwrap_or_else(|| "unavailable".into()),
    ));
    if let Some(id) = value["workflow_id"].as_i64() {
        fields.push(("Workflow identity:".into(), format!("{id}; resume reuses intent and attempt history, with fresh preflight and authorization")));
        fields.push((
            "Workflow revision:".into(),
            scalar(&value["workflow_revision"]),
        ));
        fields.push(("Workflow state:".into(), scalar(&value["workflow_state"])));
    }
    if let Some(number) = value["attempt"].as_u64() {
        fields.push((
            "Attempt ordinal:".into(),
            format!(
                "{number}; {} / {}",
                scalar(&value["phase"]),
                scalar(&value["protocol"])
            ),
        ));
    }
    fields.push((
        "Next action:".into(),
        value["next_command"]
            .as_str()
            .unwrap_or("none supported by the current evidence; review the stated blocker")
            .into(),
    ));
    fields.push((
        "Action purpose:".into(),
        scalar(&value["next_command_purpose"]),
    ));
    text.push_str(&render_exact_fields(&fields, width));
    text.push_str(&wrapped("Attempted evidence and current stored readiness are independent. Session finalization, artifact writes, readable access, process completeness, and resource cleanup do not establish target compatibility.", width));
    text
}

pub(crate) fn lifecycle_progress(event: &DeepCaptureEvent, width: usize) -> Option<String> {
    let text = match event {
        DeepCaptureEvent::Plan { .. } => "Deep Capture preflight passed; the exact authorized plan is ready.",
        DeepCaptureEvent::ProxyStarted { .. } => "Native proxy ready on the session-owned loopback listener. No target traffic or decryption is inferred.",
        DeepCaptureEvent::TrustAcquired { .. } => "Trust readiness step completed. Target CA acceptance remains unobserved until eligible traffic proves it.",
        DeepCaptureEvent::LaunchStarted { .. } => "Managed target launch started with child-scoped routing. Final-client ownership and proxy reachability remain to be observed.",
        DeepCaptureEvent::Started { .. } => "Observing the authorized session. Ctrl+C requests bounded shutdown and cleanup. The current bundle is kept at session end; history retention follows the authorized policy.",
        DeepCaptureEvent::Cleanup { .. } => return None,
        _ => return None,
    };
    Some(wrapped(text, width))
}

pub(crate) fn authorization_summary(plan: &Value, width: usize) -> String {
    let label = |pointer: &str| {
        plan.pointer(pointer)
            .and_then(Value::as_str)
            .unwrap_or("unavailable")
            .to_string()
    };
    let selection = |pointer: &str| {
        if plan.pointer(pointer).is_some_and(Value::is_string) {
            "selected"
        } else {
            "not selected"
        }
    };
    let fields = vec![
        ("Target:".into(), label("/target/name")),
        (
            "Target id:".into(),
            plan.pointer("/target/stable_id")
                .unwrap_or(&Value::Null)
                .to_string(),
        ),
        ("Launch case:".into(), label("/launch/observed_case")),
        ("Routing:".into(), label("/proxy/routing_scope")),
        ("CA trust:".into(), label("/trust/action")),
        (
            "CA store:".into(),
            if label("/trust/action") == "none" {
                "none".into()
            } else {
                label("/trust/store")
            },
        ),
        (
            "Payload retention:".into(),
            if plan
                .pointer("/capture/payload_retention")
                .and_then(Value::as_bool)
                == Some(false)
            {
                "disabled".into()
            } else {
                "enabled".into()
            },
        ),
        ("HAR:".into(), selection("/artifacts/har").into()),
        (
            "TLS key log:".into(),
            selection("/artifacts/key_log").into(),
        ),
        (
            "Client identity:".into(),
            selection("/artifacts/client_certificate").into(),
        ),
        (
            "Sensitive evidence:".into(),
            label("/artifacts/sensitivity"),
        ),
        ("Bundle:".into(), label("/artifacts/bundle")),
        (
            "Evidence retention:".into(),
            retention_label(&label("/artifacts/retention")),
        ),
        (
            "Empty container:".into(),
            "preserved; removal requires explicit purge".into(),
        ),
        (
            "Output recipient:".into(),
            label("/artifacts/output_recipient_sid"),
        ),
        (
            "Recipient proof:".into(),
            label("/artifacts/output_recipient_proof"),
        ),
    ];
    let mut text = String::from("Authorization\n\n");
    text.push_str(&wrapped(
        "Deep Capture: active target-scoped inspection. Capture remains passive.",
        width,
    ));
    text.push_str(&render_exact_fields(&fields, width));
    text.push_str(&wrapped(
        "No system-wide proxy change. Separate CA trust and sensitive-output consent remain required by the complete plan. Key logs contain secret TLS material when selected; supplied client identity is used only for upstream client authentication. Evidence is retained after external resource cleanup; review before sharing. Cleanup removes only exact session-added trust and reports unresolved recovery obligations. Review the complete canonical plan below, including deadlines, before authorizing its exact identifier.",
        width,
    ));
    text
}

pub(crate) fn collection_summary(value: &Value, width: usize) -> String {
    let observed = |key: &str| {
        value
            .get(key)
            .map_or_else(|| "unavailable".into(), ToString::to_string)
    };
    let fields = [
        ("Complete:".into(), observed("complete")),
        ("Removed bundle bytes:".into(), observed("removed_bytes")),
        ("Removed bundle files:".into(), observed("removed_files")),
        ("Eligible sessions:".into(), observed("eligible_sessions")),
        ("Retained sessions:".into(), observed("retained_sessions")),
        (
            "Unresolved sessions:".into(),
            observed("unresolved_sessions"),
        ),
        ("Empty containers:".into(), observed("empty_containers")),
        ("Limitations:".into(), observed("limitations")),
    ];
    let mut text = String::from("\nSession history maintenance\n\n");
    text.push_str(&render_exact_fields(&fields, width));
    text.push_str(&wrapped("Routine collection preserves empty containers and the bundle just returned. Saved and custom evidence remains retained until explicit cleanup. Review the backlog with fragcap bundle collect.", width));
    text
}

fn retention_label(value: &str) -> String {
    match value {
        "managed-history-30-days-20-sessions-2-gib" => "Managed history: 30 days, 20 completed sessions, 2 GiB; oldest eligible contents collected first".into(),
        "retain-until-explicit-cleanup" => "Retained until explicit cleanup".into(),
        other => other.into(),
    }
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

#[cfg(test)]
pub(crate) fn terminal_summary(
    outcome: SessionOutcome,
    complete: bool,
    artifacts: &[ArtifactResult],
    cleanup: &[CleanupResult],
    width: usize,
) -> String {
    terminal_summary_with_access(outcome, complete, artifacts, cleanup, None, width)
}

pub(crate) fn access_inspection_command(bundle: &std::path::Path, sid: Option<&str>) -> String {
    let mut command = format!(
        "fragcap bundle access-inspect {}",
        crate::workflow_help::quote_powershell_argument(&bundle.display().to_string())
    );
    if let Some(sid) = sid.filter(|sid| sid.starts_with("S-")) {
        command.push_str(&format!(
            " --output-recipient {}",
            crate::workflow_help::quote_powershell_argument(sid)
        ));
    }
    command
}

pub(crate) fn terminal_summary_with_access(
    outcome: SessionOutcome,
    complete: bool,
    artifacts: &[ArtifactResult],
    cleanup: &[CleanupResult],
    access: Option<&fragcap::deep_capture::BundleAccessVerification>,
    width: usize,
) -> String {
    let outcome = match outcome {
        SessionOutcome::Complete if complete => "complete",
        SessionOutcome::Complete | SessionOutcome::Partial => "partial",
        SessionOutcome::Interrupted => "interrupted",
        SessionOutcome::Failed => "failed",
        _ => "unavailable",
    };
    let manifest = artifacts
        .iter()
        .find(|artifact| artifact.role == "manifest")
        .and_then(|artifact| read_manifest_projection(&artifact.path));
    let completeness = manifest
        .as_ref()
        .and_then(|value| value.get("state").or_else(|| value.get("completeness")))
        .and_then(Value::as_str)
        .unwrap_or("unavailable");
    let mut text = String::from("\nCaptured evidence\n\n");
    let mut fields = vec![
        ("Session finalization:".into(), outcome.into()),
        ("Evidence completeness:".into(), completeness.into()),
    ];
    if let Some(retention) = manifest
        .as_ref()
        .and_then(|value| value["sensitive_artifacts"]["retention"].as_str())
    {
        fields.push(("Evidence retention:".into(), retention_label(retention)));
        fields.push((
            "Empty container:".into(),
            "preserved; removal requires explicit purge".into(),
        ));
    }
    if let Some(access) = access {
        fields.push(("Output recipient:".into(), access.recipient_sid.clone()));
        fields.push(("Recipient proof:".into(), access.proof_kind.clone()));
        fields.push(("Output access:".into(), access.state.clone()));
    }
    let root = artifacts
        .iter()
        .find(|artifact| artifact.role == "manifest")
        .and_then(|artifact| artifact.path.parent())
        .or_else(|| {
            artifacts
                .first()
                .and_then(|artifact| artifact.path.parent())
        });
    if let Some(root) = root.filter(|root| !root.as_os_str().is_empty()) {
        fields.push(("Bundle:".into(), root.display().to_string()));
        if !access.is_some_and(|access| access.is_verified()) {
            fields.push((
                "Next command:".into(),
                access_inspection_command(root, access.map(|access| access.recipient_sid.as_str())),
            ));
        }
    }
    let mut retained = false;
    for artifact in artifacts {
        let (status, detail) = match &artifact.status {
            ArtifactStatus::Written => {
                let verified = access.is_some_and(|access| {
                    access.is_verified()
                        && access.paths.iter().any(|path| {
                            path.state == "verified"
                                && root.is_some_and(|root| root.join(&path.path) == artifact.path)
                        })
                });
                if verified {
                    retained = true;
                    let semantic = manifest
                        .as_ref()
                        .and_then(|value| value["artifacts"].as_array())
                        .and_then(|entries| {
                            entries.iter().find(|entry| entry["role"] == artifact.role)
                        })
                        .and_then(|entry| entry["completeness"].as_str())
                        .unwrap_or("unavailable");
                    (
                        "written; recipient access verified",
                        format!("completeness={semantic}"),
                    )
                } else {
                    (
                        "written; access unavailable",
                        "ordinary-recipient access unresolved".into(),
                    )
                }
            }
            ArtifactStatus::Omitted { reason } => ("omitted", reason.clone()),
            ArtifactStatus::Failed { code, detail } => ("failed", format!("{code}: {detail}")),
            _ => ("unavailable", "no artifact result".into()),
        };
        let value = if matches!(artifact.status, ArtifactStatus::Omitted { .. }) {
            format!("{status}; {detail}")
        } else {
            let path = root
                .and_then(|root| artifact.path.strip_prefix(root).ok())
                .unwrap_or(&artifact.path);
            format!("{}; {status}; {detail}", path.display())
        };
        fields.push((format!("{}:", display_value(&artifact.role)), value));
    }
    if let Some(entries) = manifest
        .as_ref()
        .and_then(|value| value["artifacts"].as_array())
    {
        for entry in entries {
            if entry["role"] == "process-trace" {
                if let Some(loss) = entry["loss"].as_object() {
                    for (name, value) in loss {
                        if value.as_u64().is_some_and(|value| value > 0) {
                            fields.push((format!("Process trace {name}:"), value.to_string()));
                        }
                    }
                }
            }
        }
    }
    if let Some(trace) = artifacts
        .iter()
        .find(|artifact| artifact.role == "process-trace")
        .and_then(|artifact| read_trace_projection(&artifact.path))
    {
        if let Some(summary) = fragcap::deep_capture::read_process_trace(&trace) {
            fields.push((
                "Process trace finalization:".into(),
                summary.finalization.into(),
            ));
            fields.push((
                "Process trace completeness:".into(),
                summary.completeness.into(),
            ));
            fields.push((
                "Process trace limitations:".into(),
                summary.limitations.to_string(),
            ));
            for line in trace
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            {
                if line["type"] == "process-trace.limitation" {
                    if let (Some(reason), Some(count)) =
                        (line["reason"].as_str(), line["count"].as_u64())
                    {
                        fields.push((format!("Process trace {reason}:"), count.to_string()));
                    }
                }
            }
        }
    }
    text.push_str(&render_exact_fields(&fields, width));
    if !retained {
        text.push_str("No retained artifact was confirmed readable.\n");
    } else {
        text.push_str(&wrapped("Retained evidence may be sensitive. Recipient access is verified independently from evidence completeness. External resource cleanup does not delete retained evidence.", width));
    }
    text.push_str("\nResource cleanup\n\n");
    let mut unresolved = false;
    let mut rows = Vec::new();
    for result in cleanup {
        let status = match result.status {
            CleanupStatus::Released => "released",
            CleanupStatus::NotNeeded => "not-needed",
            CleanupStatus::TimedOut => "timed-out",
            CleanupStatus::Failed => "failed",
            _ => "unavailable",
        };
        let failed = !matches!(
            result.status,
            CleanupStatus::Released | CleanupStatus::NotNeeded
        );
        unresolved |= failed;
        rows.push((
            format!("{}:", display_value(&result.resource)),
            status.into(),
        ));
        if failed {
            rows.push((
                format!("{} reason:", display_value(&result.resource)),
                result.reason.clone(),
            ));
        }
    }
    text.push_str(&crate::display::render_fields(0, &rows, width));
    if cleanup.is_empty() {
        text.push_str("No cleanup result was reported; absence of residue is unproven.\n");
    } else if unresolved {
        text.push_str(&wrapped("Unresolved owned resource obligations remain. Run fragcap doctor --fix to inspect exact recovery records and review repair with explicit confirmation.", width));
    }
    text
}

/// Read only the finite metadata manifest, never payload evidence or key material.
fn read_manifest_projection(path: &std::path::Path) -> Option<Value> {
    use std::io::Read;
    const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
    let file = std::fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return None;
    }
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    (value["manifest_version"] == 2 && value["artifacts"].is_array()).then_some(value)
}

fn read_trace_projection(path: &std::path::Path) -> Option<String> {
    use std::io::Read;
    const MAX_TRACE_BYTES: u64 = 4 * 1024 * 1024;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_TRACE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_TRACE_BYTES {
        return None;
    }
    String::from_utf8(bytes).ok()
}

pub(crate) fn artifact_process_completeness(artifacts: &[ArtifactResult]) -> &'static str {
    artifacts
        .iter()
        .find(|artifact| artifact.role == "manifest")
        .and_then(|artifact| read_manifest_projection(&artifact.path))
        .and_then(|manifest| manifest["artifacts"].as_array().cloned())
        .and_then(|entries| {
            entries
                .into_iter()
                .find(|entry| entry["role"] == "process-trace")
        })
        .and_then(|entry| entry["completeness"].as_str().map(str::to_owned))
        .map_or("unavailable", |value| match value.as_str() {
            "complete" => "complete",
            "partial" => "partial",
            _ => "unavailable",
        })
}

/// Typed connection totals never substitute for application or ownership evidence.
pub(crate) fn attempt_diagnosis(value: &Value, width: usize) -> String {
    let count = |pointer: &str| {
        value
            .pointer(pointer)
            .filter(|value| !value.is_null())
            .map_or_else(|| "unavailable".into(), Value::to_string)
    };
    let mut fields = vec![
        (
            "Attempt result:".into(),
            value["verdict"].as_str().unwrap_or("unavailable").into(),
        ),
        (
            "Reason:".into(),
            value["reason"].as_str().unwrap_or("unavailable").into(),
        ),
        (
            "Earliest boundary:".into(),
            value["earliest_boundary"]
                .as_str()
                .unwrap_or("unavailable")
                .into(),
        ),
        (
            "Target packets retained:".into(),
            count("/retained_target_packets"),
        ),
    ];
    for (label, key) in [
        ("Connections accepted", "accepted_connections"),
        ("Connections completed", "completed_connections"),
        ("Connections failed", "failed_connections"),
        ("Connections forced", "forced_connections"),
        ("Connections live", "live_connections"),
        ("Connections incomplete", "incomplete_connections"),
        ("HTTP/1 exchanges completed", "http1_exchanges_completed"),
        ("HTTP/2 streams completed", "http2_streams_completed"),
        ("HTTP/3 streams completed", "http3_streams_completed"),
        ("Response heads", "response_heads"),
        ("Connection detail loss", "connection_details_lost"),
        (
            "Connection detail unavailable",
            "connection_details_unavailable",
        ),
        ("Failure detail loss", "failure_details_lost"),
        ("Observation loss", "observations_lost"),
    ] {
        fields.push((format!("{label}:"), count(&format!("/proxy/{key}"))));
    }
    for cause in [
        "authentication",
        "protocol",
        "transport",
        "upstream",
        "timeout",
        "cancelled",
        "unavailable",
    ] {
        fields.push((
            format!("Cause {cause}:"),
            count(&format!("/proxy/causes/{cause}")),
        ));
    }
    for window in [
        "observation_records",
        "owner_release_records",
        "unavailable_records",
    ] {
        fields.push((
            format!("Window {window}:"),
            count(&format!("/evidence_windows/{window}")),
        ));
    }
    for ownership in [
        "matched_records",
        "flow_only_records",
        "ambiguous_records",
        "unavailable_records",
        "other_owner_records",
        "eligible_final_client_records",
    ] {
        fields.push((
            format!("Ownership {ownership}:"),
            count(&format!("/ownership/{ownership}")),
        ));
    }
    for key in [
        "finalization",
        "artifact_writes",
        "artifact_access",
        "process_completeness",
        "cleanup",
    ] {
        fields.push((
            format!("{key}:"),
            value[key].as_str().unwrap_or("unavailable").into(),
        ));
    }
    if let Some(reasons) = value.pointer("/correlation_evidence/reasons") {
        for (label, reason) in [
            ("Exact flow and owner", "exact-flow-and-owner"),
            ("Flow never observed", "packet-flow-not-observed"),
            (
                "History withheld or bounded",
                "packet-history-bound-exceeded",
            ),
            (
                "Capture buffer history incomplete",
                "capture-buffer-history-incomplete",
            ),
            (
                "Flow outside observation window",
                "packet-flow-has-no-overlapping-observation",
            ),
            ("Flow owner unavailable", "packet-flow-unattributed"),
            (
                "Flow owner partly unresolved",
                "packet-owner-partially-unresolved",
            ),
            ("Conflicting packet owners", "conflicting-packet-owners"),
            (
                "Connection ending unavailable",
                "connection-terminal-not-observed",
            ),
            ("Other correlation reason", "other"),
        ] {
            if let Some(total) = reasons[reason].as_u64().filter(|total| *total > 0) {
                fields.push((format!("{label}:"), total.to_string()));
            }
        }
    }
    fields.push((
        "Fact writes:".into(),
        format!(
            "appended={}, skipped={}, failed={}",
            count("/fact_writes/appended"),
            count("/fact_writes/skipped"),
            count("/fact_writes/failed")
        ),
    ));
    if let Some(results) = value["fact_write_results"].as_array() {
        for (index, result) in results.iter().enumerate() {
            fields.push((
                format!("Failed fact {}:", index + 1),
                format!(
                    "{}; {}: {}",
                    result["kind"].as_str().unwrap_or("unavailable"),
                    result["code"].as_str().unwrap_or("unavailable"),
                    result["detail"].as_str().unwrap_or("unavailable")
                ),
            ));
        }
    }
    let mut text = String::from("\nObserved case evidence\n\n");
    text.push_str(&render_exact_fields(&fields, width));
    if let Some(connections) = value
        .pointer("/proxy/connections")
        .and_then(Value::as_array)
    {
        let rows: Vec<Vec<String>> = connections
            .iter()
            .map(|record| {
                vec![
                    record["connection_id"].to_string(),
                    record["terminal"].as_str().unwrap_or("unavailable").into(),
                    record["category"].as_str().unwrap_or("none").into(),
                    record["code"].as_str().unwrap_or("none").into(),
                ]
            })
            .collect();
        if !rows.is_empty() {
            text.push_str("\nConnection endings (retained bounded details)\n");
            let layout = crate::display::ColumnLayout::new(0, &rows);
            for row in &rows {
                text.push_str(&layout.render_row(row));
                text.push('\n');
            }
        }
    }
    text.push_str(&wrapped("Connection endings and application exchanges count different populations. Ownership counters describe retained records; late owner-release records cannot satisfy the attempted observation case. Unknown or lost details remain explicit.", width));
    if value["reason"] == "final-client-correlation-missing" {
        text.push_str(&wrapped("The attempt has missing packet-flow correlation or unresolved final-client ownership. Run fragcap doctor to inspect Npcap loopback readiness, then inspect the retained application and process evidence for the correlation reasons above. Doctor checks acquisition prerequisites; it does not prove ownership or compatibility. No gameplay remedy or measurement retry is established by this evidence.", width));
    }
    text
}

#[cfg(test)]
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
    #[test]
    fn managed_retention_consequences_are_visible_and_unknown_maintenance_is_not_zero() {
        let plan = serde_json::json!({"artifacts":{"retention":"managed-history-30-days-20-sessions-2-gib"}});
        for width in [40, 60, 80] {
            let text = super::authorization_summary(&plan, width)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            for required in [
                "30 days",
                "20 completed sessions",
                "2 GiB",
                "explicit purge",
            ] {
                assert!(text.contains(required), "missing {required}: {text}");
            }
            let text = super::collection_summary(
                &serde_json::json!({"complete":false,"limitations":["registry unreadable"]}),
                width,
            )
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
            assert!(text.contains("unavailable"));
            assert!(text.contains("registry unreadable"));
        }
    }

    #[test]
    fn producer_readability_cannot_replace_required_recipient_proof() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("capture.fcapng");
        std::fs::write(&path, b"synthetic").unwrap();
        let artifacts = [ArtifactResult {
            role: "capture".into(),
            path,
            sensitivity: fragcap::deep_capture::Sensitivity::Payload,
            required: true,
            status: ArtifactStatus::Written,
        }];
        let mut access = fragcap::deep_capture::BundleAccessVerification {
            recipient_sid: "S-1-5-21-1-2-3-1001".into(),
            proof_kind: "synthetic-proof".into(),
            state: "unresolved".into(),
            paths: vec![fragcap::deep_capture::AccessPathVerification {
                path: "capture.fcapng".into(),
                state: "unresolved".into(),
                reason: "denied".into(),
            }],
            limitations: vec![],
        };
        let unresolved = terminal_summary_with_access(
            SessionOutcome::Complete,
            true,
            &artifacts,
            &[],
            Some(&access),
            80,
        );
        assert!(unresolved.contains("ordinary-recipient access unresolved"));
        assert!(!unresolved.contains("recipient access verified"));
        assert!(unresolved.contains("fragcap bundle access-inspect"));
        assert!(unresolved.contains("--output-recipient"));
        assert!(unresolved.contains("S-1-5-21-1-2-3-1001"));
        access.state = "verified".into();
        access.paths[0].state = "verified".into();
        let verified = terminal_summary_with_access(
            SessionOutcome::Complete,
            true,
            &artifacts,
            &[],
            Some(&access),
            80,
        );
        assert!(verified.contains("recipient access verified"));
        assert!(verified.contains("S-1-5-21-1-2-3-1001"));
    }
    use super::*;
    use crate::display::display_width;
    use fragcap::deep_capture::api::{ArtifactStatus, CleanupStatus, Sensitivity};
    use serde_json::json;

    #[test]
    fn artifact_summary_reports_partial_semantics_and_exact_access_retry() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("manifest.json");
        std::fs::write(&manifest, r#"{"manifest_version":2,"state":"partial","artifacts":[{"role":"process-trace","completeness":"partial","loss":{"limitations":3437}}]}"#).unwrap();
        let trace = dir.path().join("process-trace.jsonl");
        std::fs::write(&trace, "controlled trace\n").unwrap();
        let artifacts = vec![
            ArtifactResult {
                role: "manifest".into(),
                path: manifest,
                sensitivity: Sensitivity::Metadata,
                required: true,
                status: ArtifactStatus::Written,
            },
            ArtifactResult {
                role: "process-trace".into(),
                path: trace,
                sensitivity: Sensitivity::Payload,
                required: true,
                status: ArtifactStatus::Written,
            },
            ArtifactResult {
                role: "har".into(),
                path: dir.path().join("http.har"),
                sensitivity: Sensitivity::Payload,
                required: false,
                status: ArtifactStatus::Omitted {
                    reason: "artifact was not requested".into(),
                },
            },
        ];
        let text = terminal_summary(SessionOutcome::Complete, true, &artifacts, &[], 80);
        assert!(text.contains("Session finalization:"));
        assert!(text.contains("partial"));
        assert!(text.contains("process-trace.jsonl"));
        assert_eq!(text.matches("Bundle:").count(), 1);
        assert_eq!(text.matches(&dir.path().display().to_string()).count(), 2);
        assert!(text.contains("fragcap bundle access-inspect"));
        assert!(!text.contains("http.har"));
        assert!(!text.contains("Deep Capture outcome: complete"));
    }

    #[test]
    fn synthetic_transcript_separates_partial_process_six_endings_and_seven_exchanges() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("manifest.json");
        std::fs::write(&manifest_path, json!({"manifest_version":2,"state":"partial","artifacts":[{"role":"process-trace","completeness":"partial"}]}).to_string()).unwrap();
        let trailer = json!({"type":"process-trace.trailer","finalization":"complete","completeness":"partial","records":4,"process_instances":2,"flow_owner_intervals":0,"limitations":3437,"events_lost":0,"unparseable_events":0,"buffers_lost":0,"rundown_ignored":0,"events_unretained":0,"stage_transitions_unretained":0,"unresolved_flow_owners":0});
        let trace_path = dir.path().join("process-trace.jsonl");
        std::fs::write(&trace_path, format!("{}\n{}\n{}\n{}\n", json!({"type":"process-trace.header"}), json!({"type":"process-trace.limitation","reason":"packet-evidence-unretained","count":3435}), json!({"type":"process-trace.limitation","reason":"process-exit-unobserved","count":2}), trailer)).unwrap();
        let artifacts = [
            ArtifactResult {
                role: "manifest".into(),
                path: manifest_path,
                sensitivity: Sensitivity::Metadata,
                required: true,
                status: ArtifactStatus::Written,
            },
            ArtifactResult {
                role: "process-trace".into(),
                path: trace_path,
                sensitivity: Sensitivity::Payload,
                required: true,
                status: ArtifactStatus::Written,
            },
        ];
        let value = json!({"verdict":"inconclusive","reason":"final-client-correlation-missing","earliest_boundary":"final-client-correlation","proxy":{"accepted_connections":6,"completed_connections":0,"failed_connections":6,"http1_exchanges_completed":7,"causes":{"protocol":5,"timeout":1}},"evidence_windows":{"observation_records":0,"owner_release_records":7,"unavailable_records":0},"ownership":{"eligible_final_client_records":0},"cleanup":"released","fact_writes":{"appended":1,"skipped":0,"failed":0}});
        let text = format!(
            "{}{}",
            attempt_diagnosis(&value, 80),
            terminal_summary(
                SessionOutcome::Complete,
                true,
                &artifacts,
                &[CleanupResult {
                    resource: "native-proxy-listener".into(),
                    status: CleanupStatus::Released,
                    reason: "private implementation text".into()
                }],
                80
            )
        );
        let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for expected in [
            "Connections accepted: 6",
            "Connections completed: 0",
            "Connections failed: 6",
            "HTTP/1 exchanges completed: 7",
            "Cause protocol: 5",
            "Cause timeout: 1",
            "Window owner_release_records: 7",
            "Process trace limitations: 3437",
            "Process trace packet-evidence-unretained: 3435",
            "Process trace process-exit-unobserved: 2",
            "Evidence completeness: partial",
            "Session finalization: complete",
        ] {
            assert!(words.contains(expected), "missing {expected}: {text}");
        }
        assert_eq!(text.matches("Bundle:").count(), 1);
        assert_eq!(text.matches(&dir.path().display().to_string()).count(), 2);
        assert!(text.contains("fragcap bundle access-inspect"));
        assert!(!text.contains("private implementation text"));
        assert!(words.contains("missing packet-flow correlation"));
        assert!(words.contains("fragcap doctor"));
        assert!(words.contains("No gameplay remedy"));
        assert_eq!(artifact_process_completeness(&artifacts), "partial");
        assert!(!text.contains("classification failed=0"));
    }

    #[test]
    fn artifact_access_failure_does_not_claim_readable_evidence() {
        let artifacts = [ArtifactResult {
            role: "proxy-lifecycle".into(),
            path: "missing-proxy.jsonl".into(),
            sensitivity: Sensitivity::Payload,
            required: true,
            status: ArtifactStatus::Written,
        }];
        let text = terminal_summary(SessionOutcome::Complete, true, &artifacts, &[], 80);
        assert!(text.contains("access unavailable"));
        assert!(text.contains("ordinary-recipient access unresolved"));
    }

    #[test]
    fn correlation_reason_counts_keep_measured_anchors_and_supported_guidance_at_narrow_widths() {
        let value = json!({
            "reason": "final-client-correlation-missing",
            "correlation_evidence": {"reasons": {
                "packet-flow-not-observed": 6,
                "packet-history-bound-exceeded": 2,
                "capture-buffer-history-incomplete": 1,
                "exact-flow-and-owner": 0,
            }},
        });
        for width in [20, 40, 80, 160] {
            let text = attempt_diagnosis(&value, width);
            let rows: Vec<_> = text.lines().filter(|line| line.contains(':')).collect();
            let widest = rows
                .iter()
                .map(|row| display_width(&row[..=row.find(':').unwrap()]))
                .max()
                .unwrap();
            for (label, count) in [
                ("Flow never observed:", "6"),
                ("History withheld or bounded:", "2"),
                ("Capture buffer history incomplete:", "1"),
            ] {
                let row = rows.iter().find(|row| row.starts_with(label)).unwrap();
                assert_eq!(row.trim_end().chars().last().unwrap().to_string(), count);
                assert_eq!(row.find(count).unwrap(), widest + 4);
            }
            assert!(!text.contains("Exact flow and owner:"));
            let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(words.contains("fragcap doctor"));
            assert!(words.contains("does not prove ownership or compatibility"));
            assert!(words.contains("No gameplay remedy or measurement retry"));
        }
    }

    #[test]
    fn authorization_fields_use_four_space_actual_label_anchors() {
        let text = authorization_summary(&plan(false, false), 80);
        let rows: Vec<_> = text
            .lines()
            .filter(|line| {
                line.starts_with("Target:")
                    || line.starts_with("Launch case:")
                    || line.starts_with("Payload retention:")
            })
            .collect();
        assert_eq!(rows.len(), 3);
        let anchors: Vec<_> = rows
            .iter()
            .map(|line| {
                let colon = line.find(':').unwrap();
                colon + 1 + line[colon + 1..].chars().take_while(|c| *c == ' ').count()
            })
            .collect();
        assert!(anchors.iter().all(|anchor| *anchor == anchors[0]));
        let longest = text
            .lines()
            .filter(|line| line.contains(':'))
            .max_by_key(|line| line.find(':').unwrap())
            .unwrap();
        assert_eq!(
            longest
                .split_once(':')
                .unwrap()
                .1
                .chars()
                .take_while(|c| *c == ' ')
                .count(),
            4
        );
    }

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
                let field = |label: &str| {
                    text.lines()
                        .find_map(|line| line.strip_prefix(label).map(str::trim))
                        .unwrap()
                };
                assert_ne!(field("CA trust:") == "none", trust);
                assert_eq!(field("HAR:") == "selected", sensitive);
                assert_eq!(field("TLS key log:") == "selected", sensitive);
                assert_eq!(field("Client identity:") == "selected", sensitive);
                assert_eq!(field("Payload retention:"), "disabled");
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
            assert_eq!(
                text.lines()
                    .find_map(|line| line.strip_prefix("Session finalization:").map(str::trim)),
                Some(expected)
            );
            assert!(
                text.contains(&dir.path().display().to_string())
                    && text.contains("application.jsonl")
                    && text.contains("retained")
            );
            assert!(text.contains("exact-ca-thumbprint") && text.contains("failed"));
            assert!(text.contains("fragcap doctor --fix") && text.contains("confirmation"));
        }
        let released = [CleanupResult {
            resource: "proxy".into(),
            status: CleanupStatus::Released,
            reason: "owned proxy stopped".into(),
        }];
        let text = terminal_summary(SessionOutcome::Complete, false, &[], &released, 80);
        assert_eq!(
            text.lines()
                .find_map(|line| line.strip_prefix("Session finalization:").map(str::trim)),
            Some("partial")
        );
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
            assert!(text
                .lines()
                .filter(|line| !line.contains(':'))
                .all(|line| display_width(line) <= width));
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
        assert!(
            text.contains("C:/controlled  evidence")
                && text.contains("long exact bundle directory")
        );
    }
}
