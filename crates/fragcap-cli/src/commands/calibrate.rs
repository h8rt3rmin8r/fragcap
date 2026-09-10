// SPDX-License-Identifier: Apache-2.0

//! Guided one-attempt calibration for one already registered target.

use std::path::Path;

use fragcap::deep_capture::api as deep_capture_api;
use fragcap::targets::{
    CompatibilityAddressFamily, CompatibilityLaunchCase, CompatibilityProtocol,
    CompatibilityRoutingStrategy, Store, TargetEntry,
};

use crate::cli::{
    CalibrateArgs, DeepCaptureArgs, DeepCaptureCalibrationArg, DeepCaptureCalibrationProtocolArg,
    DeepCaptureLaunchCaseArg, DeepCaptureProxyFamilyArg, GuidedCalibrationProtocolArg,
};
use crate::commands::deep_capture;
use crate::emit::Emitter;
use crate::events::Event;
use crate::exit::{CliError, Exit};
use crate::DeepCaptureAuthorizationInput;

struct Guidance {
    topology: Option<String>,
    action: &'static str,
    status: &'static str,
    observed_launch_case: Option<String>,
    selected_launch_case: Option<String>,
    reason: Option<String>,
    images: Vec<String>,
    limitations: Vec<String>,
    requested_protocols: Vec<String>,
    observed_protocols: Vec<String>,
    completed_protocols: Vec<String>,
    remaining_protocols: Vec<String>,
    next_command: Option<String>,
}

pub fn run(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    let requested_protocols = normalize_protocol_args(&args.protocol);
    let local_store_path = deep_capture::local_store_path(args.local_db.as_deref())?;
    let store = deep_capture::open_local_store(args.local_db.as_deref())?;
    let local_store_argument = quote_powershell_path(&local_store_path)?;
    let target = deep_capture::resolve_target_input(
        &store,
        args.selector.as_deref(),
        args.target.as_deref(),
        args.id,
    )?;
    if args.controlled_target {
        deep_capture::require_controlled_target(&target)?;
    }
    let snapshot = process_snapshot(args.controlled_target);
    let proposal = build_proposal(&store, &target, snapshot.clone(), &requested_protocols)?;
    if !proposal.limitations.is_empty() {
        let limitations = limitation_messages(&proposal);
        emit_guidance(
            emitter,
            &target,
            Guidance {
                topology: topology(&proposal),
                action: "refused",
                status: "refused",
                observed_launch_case: None,
                selected_launch_case: None,
                reason: Some("proposal-limitations".to_string()),
                images: readiness_images(&proposal),
                limitations: limitations.clone(),
                requested_protocols: protocol_names(&requested_protocols),
                observed_protocols: Vec::new(),
                completed_protocols: Vec::new(),
                remaining_protocols: protocol_names(&requested_protocols),
                next_command: None,
            },
        );
        return Err(CliError::usage(format!(
            "guided calibration cannot select an attempt: {}",
            limitations.join("; ")
        )));
    }

    match &proposal.readiness {
        deep_capture_api::CalibrationLaunchReadiness::OperatorAction {
            observed_case,
            cold_case,
            images,
        } => {
            let next_command = calibrate_command(
                target.stable_id,
                &local_store_argument,
                true,
                &requested_protocols,
            );
            emit_guidance(
                emitter,
                &target,
                Guidance {
                    topology: topology(&proposal),
                    action: "operator-action",
                    status: "warm",
                    observed_launch_case: Some(observed_case.as_str().to_string()),
                    selected_launch_case: Some(cold_case.as_str().to_string()),
                    reason: Some("declared-process-image-present".to_string()),
                    images: images.clone(),
                    limitations: Vec::new(),
                    requested_protocols: protocol_names(&requested_protocols),
                    observed_protocols: Vec::new(),
                    completed_protocols: Vec::new(),
                    remaining_protocols: protocol_names(&requested_protocols),
                    next_command: Some(next_command),
                },
            );
            if !args.restart_warm {
                return Ok(Exit::SUCCESS);
            }
            let mut restart_args = low_level_args(
                args,
                None,
                deep_capture_api::CalibrationPhase::Reachability,
                CompatibilityProtocol::Routing,
            )?;
            restart_args.restart_warm = true;
            deep_capture::prepare_warm_restart(&restart_args, &store, emitter)?;
        }
        deep_capture_api::CalibrationLaunchReadiness::Ready { .. } => {}
        _ => {
            emit_guidance(
                emitter,
                &target,
                Guidance {
                    topology: topology(&proposal),
                    action: "refused",
                    status: "refused",
                    observed_launch_case: None,
                    selected_launch_case: None,
                    reason: Some("launch-readiness-unavailable".to_string()),
                    images: readiness_images(&proposal),
                    limitations: Vec::new(),
                    requested_protocols: protocol_names(&requested_protocols),
                    observed_protocols: Vec::new(),
                    completed_protocols: Vec::new(),
                    remaining_protocols: protocol_names(&requested_protocols),
                    next_command: None,
                },
            );
            return Err(CliError::usage(
                "guided calibration could not determine an exact launch case",
            ));
        }
    }

    let resolver_args = low_level_args(
        args,
        None,
        deep_capture_api::CalibrationPhase::Reachability,
        CompatibilityProtocol::Routing,
    )?;
    let fresh_store = deep_capture::open_local_store(args.local_db.as_deref())?;
    let fresh_target = deep_capture::resolve_target(&fresh_store, &resolver_args)?;
    let fresh_snapshot = if args.restart_warm {
        process_snapshot(args.controlled_target)
    } else {
        snapshot
    };
    let selected = build_proposal(
        &fresh_store,
        &fresh_target,
        fresh_snapshot.clone(),
        &requested_protocols,
    )?;
    if !selected.limitations.is_empty() {
        let limitations = limitation_messages(&selected);
        emit_guidance(
            emitter,
            &fresh_target,
            Guidance {
                topology: topology(&selected),
                action: "refused",
                status: "refused",
                observed_launch_case: None,
                selected_launch_case: None,
                reason: Some("proposal-limitations".to_string()),
                images: readiness_images(&selected),
                limitations: limitations.clone(),
                requested_protocols: protocol_names(&requested_protocols),
                observed_protocols: Vec::new(),
                completed_protocols: Vec::new(),
                remaining_protocols: protocol_names(&requested_protocols),
                next_command: None,
            },
        );
        return Err(CliError::usage(format!(
            "guided calibration cannot select an attempt: {}",
            limitations.join("; ")
        )));
    }
    if selected.steps.is_empty() {
        let launch_case = ready_launch_case(&selected)?;
        let current = deep_capture::current_compatibility_case(
            launch_case,
            DeepCaptureProxyFamilyArg::Ipv4,
            CompatibilityProtocol::Routing,
        );
        deep_capture_api::validate_compatibility_prerequisites(
            deep_capture_api::SessionMode::Capture,
            args.controlled_target,
            &fresh_store
                .compatibility_facts_for_target(required_row_id(&fresh_target)?)
                .map_err(|error| CliError::failure(error.to_string()))?,
            deep_capture::library_launch_case(launch_case),
            &current,
        )
        .map_err(|refusal| CliError::usage(refusal.to_string()))?;
        let (completed_protocols, remaining_protocols) =
            coverage_from_proposal(&selected, &requested_protocols);
        let (status, reason) = if requested_protocols.is_empty() {
            ("ready", "current-routing-evidence")
        } else {
            ("requested-coverage-complete", "current-protocol-evidence")
        };
        emit_guidance(
            emitter,
            &fresh_target,
            Guidance {
                topology: topology(&selected),
                action: "ready",
                status,
                observed_launch_case: None,
                selected_launch_case: Some(launch_case.as_str().to_string()),
                reason: Some(reason.to_string()),
                images: readiness_images(&selected),
                limitations: Vec::new(),
                requested_protocols: protocol_names(&requested_protocols),
                observed_protocols: Vec::new(),
                completed_protocols: protocol_names(&completed_protocols),
                remaining_protocols: protocol_names(&remaining_protocols),
                next_command: Some(target_command(
                    "deep-capture",
                    fresh_target.stable_id,
                    &local_store_argument,
                    " --launch",
                )),
            },
        );
        return Ok(Exit::SUCCESS);
    }
    let step = &selected.steps[0];
    if !valid_guided_step(step) {
        return Err(CliError::usage(
            "guided calibration selected an unsupported phase or protocol; no session was started",
        ));
    }
    let action = match step.phase {
        deep_capture_api::CalibrationPhase::Reachability => "run-reachability",
        deep_capture_api::CalibrationPhase::Tls => "run-protocol",
    };
    let low_level = low_level_args(
        args,
        Some(launch_case_arg(step.case.launch_case)),
        step.phase,
        step.case.protocol,
    )?;
    let (completed_protocols, remaining_protocols) =
        coverage_from_proposal(&selected, &requested_protocols);
    emit_guidance(
        emitter,
        &fresh_target,
        Guidance {
            topology: topology(&selected),
            action,
            status: "selected",
            observed_launch_case: None,
            selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
            reason: Some(step.reason.as_str().to_string()),
            images: readiness_images(&selected),
            limitations: Vec::new(),
            requested_protocols: protocol_names(&requested_protocols),
            observed_protocols: Vec::new(),
            completed_protocols: protocol_names(&completed_protocols),
            remaining_protocols: protocol_names(&remaining_protocols),
            next_command: None,
        },
    );
    drop(fresh_store);
    let outcome = match deep_capture::run_with_outcome(&low_level, authorization, emitter) {
        Ok(outcome) => outcome,
        Err(error) => {
            emit_guidance(
                emitter,
                &fresh_target,
                Guidance {
                    topology: topology(&selected),
                    action,
                    status: "failed",
                    observed_launch_case: None,
                    selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                    reason: Some("delegated-session-error".to_string()),
                    images: readiness_images(&selected),
                    limitations: Vec::new(),
                    requested_protocols: protocol_names(&requested_protocols),
                    observed_protocols: Vec::new(),
                    completed_protocols: protocol_names(&completed_protocols),
                    remaining_protocols: protocol_names(&remaining_protocols),
                    next_command: None,
                },
            );
            return Err(error);
        }
    };
    let observed_protocols = deep_capture_api::observed_protocol_candidates(
        &outcome.observations,
        args.controlled_target,
    );
    let all_protocols = merge_protocols(&requested_protocols, &observed_protocols);
    let completed_store = deep_capture::open_local_store(args.local_db.as_deref())?;
    let completed_target = deep_capture::resolve_target(&completed_store, &low_level)?;
    let completed_snapshot = process_snapshot(args.controlled_target);
    let completed = build_proposal(
        &completed_store,
        &completed_target,
        completed_snapshot,
        &all_protocols,
    )?;
    let (completed_protocols, remaining_protocols) =
        coverage_from_proposal(&completed, &all_protocols);
    let (completed_status, completed_reason) = if outcome.terminal_error.is_some() {
        ("failed", "delegated-session-terminal-failure")
    } else {
        completion_outcome_for_step(
            &completed,
            step.phase,
            step.case.launch_case,
            step.case.protocol,
            &completed_protocols,
        )
    };
    let next_command = continuation_command(
        &completed,
        outcome.terminal_error.is_none(),
        completed_target.stable_id,
        &local_store_argument,
        &all_protocols,
        &remaining_protocols,
    );
    emit_guidance(
        emitter,
        &completed_target,
        Guidance {
            topology: topology(&completed),
            action,
            status: completed_status,
            observed_launch_case: None,
            selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
            reason: Some(completed_reason.to_string()),
            images: readiness_images(&completed),
            limitations: limitation_messages(&completed),
            requested_protocols: protocol_names(&requested_protocols),
            observed_protocols: protocol_names(&observed_protocols),
            completed_protocols: protocol_names(&completed_protocols),
            remaining_protocols: protocol_names(&remaining_protocols),
            next_command,
        },
    );
    match outcome.terminal_error {
        Some(error) => Err(error),
        None => Ok(Exit::SUCCESS),
    }
}

#[cfg(test)]
fn completion_outcome(
    proposal: &deep_capture_api::CalibrationProposal,
    attempted_launch_case: CompatibilityLaunchCase,
) -> (&'static str, &'static str) {
    let ready_launch_case = match proposal.readiness {
        deep_capture_api::CalibrationLaunchReadiness::Ready { launch_case, .. } => {
            Some(launch_case)
        }
        _ => None,
    };
    completion_outcome_from_state(
        !proposal.limitations.is_empty(),
        ready_launch_case,
        proposal.steps.is_empty(),
        attempted_launch_case,
    )
}

fn completion_outcome_for_step(
    proposal: &deep_capture_api::CalibrationProposal,
    phase: deep_capture_api::CalibrationPhase,
    attempted_launch_case: CompatibilityLaunchCase,
    attempted_protocol: CompatibilityProtocol,
    completed_protocols: &[CompatibilityProtocol],
) -> (&'static str, &'static str) {
    if !proposal.limitations.is_empty() {
        return ("refused", "post-session-proposal-limitations");
    }
    let ready_launch_case = match proposal.readiness {
        deep_capture_api::CalibrationLaunchReadiness::Ready { launch_case, .. } => launch_case,
        _ => return ("not-completed", "post-session-launch-not-ready"),
    };
    if ready_launch_case != attempted_launch_case {
        return (
            "not-completed",
            "attempt-produced-no-current-positive-evidence",
        );
    }
    match phase {
        deep_capture_api::CalibrationPhase::Reachability => {
            let route_still_required = proposal
                .steps
                .first()
                .is_some_and(|step| step.phase == deep_capture_api::CalibrationPhase::Reachability);
            if route_still_required {
                (
                    "not-completed",
                    "attempt-produced-no-current-positive-evidence",
                )
            } else {
                ("completed", "current-routing-evidence-recorded")
            }
        }
        deep_capture_api::CalibrationPhase::Tls => {
            if completed_protocols.contains(&attempted_protocol) {
                ("completed", "current-protocol-evidence-recorded")
            } else {
                (
                    "not-completed",
                    "attempt-produced-no-current-positive-evidence",
                )
            }
        }
    }
}

#[cfg(test)]
fn completion_outcome_from_state(
    has_limitations: bool,
    ready_launch_case: Option<CompatibilityLaunchCase>,
    has_no_steps: bool,
    attempted_launch_case: CompatibilityLaunchCase,
) -> (&'static str, &'static str) {
    if has_limitations {
        return ("refused", "post-session-proposal-limitations");
    }
    match ready_launch_case {
        Some(launch_case) if launch_case == attempted_launch_case && has_no_steps => {
            ("completed", "current-routing-evidence-recorded")
        }
        Some(_) => (
            "not-completed",
            "attempt-produced-no-current-positive-evidence",
        ),
        _ => ("not-completed", "post-session-launch-not-ready"),
    }
}

fn quote_powershell_path(path: &Path) -> Result<String, CliError> {
    let value = path.to_str().ok_or_else(|| {
        CliError::usage("the effective local store path cannot be represented in a next command")
    })?;
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '`' => quoted.push_str("``"),
            '"' => quoted.push_str("`\""),
            '$' => quoted.push_str("`$"),
            '\r' => quoted.push_str("`r"),
            '\n' => quoted.push_str("`n"),
            '\t' => quoted.push_str("`t"),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    Ok(quoted)
}

fn target_command(verb: &str, target_id: i64, local_store: &str, suffix: &str) -> String {
    format!("fragcap {verb} --id {target_id} --local-db {local_store}{suffix}")
}

fn calibrate_command(
    target_id: i64,
    local_store: &str,
    restart_warm: bool,
    protocols: &[CompatibilityProtocol],
) -> String {
    let mut suffix = if restart_warm {
        " --restart-warm".to_string()
    } else {
        String::new()
    };
    for protocol in protocols {
        suffix.push_str(" --protocol ");
        suffix.push_str(protocol.as_str());
    }
    target_command("calibrate", target_id, local_store, &suffix)
}

fn continuation_command(
    proposal: &deep_capture_api::CalibrationProposal,
    session_succeeded: bool,
    target_id: i64,
    local_store: &str,
    all_protocols: &[CompatibilityProtocol],
    remaining_protocols: &[CompatibilityProtocol],
) -> Option<String> {
    if !session_succeeded || !proposal.limitations.is_empty() {
        return None;
    }
    if matches!(
        proposal.readiness,
        deep_capture_api::CalibrationLaunchReadiness::OperatorAction { .. }
    ) {
        let protocols = if remaining_protocols.is_empty() {
            all_protocols
        } else {
            remaining_protocols
        };
        return Some(calibrate_command(target_id, local_store, true, protocols));
    }
    if !matches!(
        proposal.readiness,
        deep_capture_api::CalibrationLaunchReadiness::Ready { .. }
    ) {
        return None;
    }
    let route_still_required = proposal
        .steps
        .first()
        .is_some_and(|step| step.phase == deep_capture_api::CalibrationPhase::Reachability);
    if route_still_required {
        Some(calibrate_command(
            target_id,
            local_store,
            false,
            all_protocols,
        ))
    } else if remaining_protocols.is_empty() {
        Some(target_command(
            "deep-capture",
            target_id,
            local_store,
            " --launch",
        ))
    } else {
        Some(calibrate_command(
            target_id,
            local_store,
            false,
            remaining_protocols,
        ))
    }
}

fn normalize_protocol_args(values: &[GuidedCalibrationProtocolArg]) -> Vec<CompatibilityProtocol> {
    let mut protocols = values
        .iter()
        .copied()
        .map(guided_protocol)
        .collect::<Vec<_>>();
    protocols.sort_by_key(|protocol| protocol.as_str());
    protocols.dedup();
    protocols
}

fn guided_protocol(value: GuidedCalibrationProtocolArg) -> CompatibilityProtocol {
    match value {
        GuidedCalibrationProtocolArg::Http1 => CompatibilityProtocol::Http1,
        GuidedCalibrationProtocolArg::Https => CompatibilityProtocol::Https,
        GuidedCalibrationProtocolArg::Http2 => CompatibilityProtocol::Http2,
        GuidedCalibrationProtocolArg::Websocket => CompatibilityProtocol::WebSocket,
        GuidedCalibrationProtocolArg::Sse => CompatibilityProtocol::Sse,
        GuidedCalibrationProtocolArg::Grpc => CompatibilityProtocol::Grpc,
        GuidedCalibrationProtocolArg::GenericTcp => CompatibilityProtocol::GenericTcp,
        GuidedCalibrationProtocolArg::NonHttpTls => CompatibilityProtocol::NonHttpTls,
        GuidedCalibrationProtocolArg::Socks5Tcp => CompatibilityProtocol::Socks5Tcp,
        GuidedCalibrationProtocolArg::Socks5Udp => CompatibilityProtocol::Socks5Udp,
        GuidedCalibrationProtocolArg::GenericUdp => CompatibilityProtocol::GenericUdp,
        GuidedCalibrationProtocolArg::Quic => CompatibilityProtocol::Quic,
        GuidedCalibrationProtocolArg::Http3 => CompatibilityProtocol::Http3,
    }
}

fn merge_protocols(
    left: &[CompatibilityProtocol],
    right: &[CompatibilityProtocol],
) -> Vec<CompatibilityProtocol> {
    let mut protocols = left.iter().chain(right).copied().collect::<Vec<_>>();
    protocols.sort_by_key(|protocol| protocol.as_str());
    protocols.dedup();
    protocols
}

fn protocol_names(protocols: &[CompatibilityProtocol]) -> Vec<String> {
    protocols
        .iter()
        .map(|protocol| protocol.as_str().to_string())
        .collect()
}

fn coverage_from_proposal(
    proposal: &deep_capture_api::CalibrationProposal,
    requested: &[CompatibilityProtocol],
) -> (Vec<CompatibilityProtocol>, Vec<CompatibilityProtocol>) {
    if !proposal.limitations.is_empty()
        || !matches!(
            &proposal.readiness,
            deep_capture_api::CalibrationLaunchReadiness::Ready { .. }
        )
    {
        return (Vec::new(), merge_protocols(&[], requested));
    }
    let mut remaining = proposal
        .steps
        .iter()
        .filter(|step| step.phase == deep_capture_api::CalibrationPhase::Tls)
        .map(|step| step.case.protocol)
        .chain(
            proposal
                .deferred_protocols
                .iter()
                .map(|deferred| deferred.protocol),
        )
        .collect::<Vec<_>>();
    remaining.sort_by_key(|protocol| protocol.as_str());
    remaining.dedup();
    let completed = requested
        .iter()
        .copied()
        .filter(|protocol| !remaining.contains(protocol))
        .collect();
    (completed, remaining)
}

fn valid_guided_step(step: &deep_capture_api::CalibrationProposalStep) -> bool {
    match step.phase {
        deep_capture_api::CalibrationPhase::Reachability => {
            step.case.protocol == CompatibilityProtocol::Routing
        }
        deep_capture_api::CalibrationPhase::Tls => !matches!(
            step.case.protocol,
            CompatibilityProtocol::Routing | CompatibilityProtocol::NotApplicable
        ),
    }
}

fn build_proposal(
    store: &Store,
    target: &TargetEntry,
    snapshot: deep_capture_api::CalibrationProcessSnapshot,
    protocols: &[CompatibilityProtocol],
) -> Result<deep_capture_api::CalibrationProposal, CliError> {
    let facts = store
        .compatibility_facts_for_target(required_row_id(target)?)
        .map_err(|error| CliError::failure(error.to_string()))?;
    let request = deep_capture_api::CalibrationProposalRequest::new(
        target.clone(),
        "fragcap-native",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION"),
    )
    .with_process_snapshot(snapshot)
    .with_routing_strategy(CompatibilityRoutingStrategy::ChildEnvironment)
    .with_address_family(CompatibilityAddressFamily::Ipv4)
    .with_protocol_candidates(protocols.iter().copied())
    .with_facts(facts);
    Ok(deep_capture_api::propose_calibration(&request))
}

fn required_row_id(target: &TargetEntry) -> Result<i64, CliError> {
    target
        .id
        .ok_or_else(|| CliError::failure("resolved target has no local store row identifier"))
}

fn process_snapshot(controlled: bool) -> deep_capture_api::CalibrationProcessSnapshot {
    if controlled {
        // The reserved harness has no ambient target process. Its executable is
        // selected only after authorization by the controlled target adapter.
        return deep_capture_api::CalibrationProcessSnapshot::complete(Vec::<String>::new());
    }
    match deep_capture::process_image_snapshot() {
        Ok(images) => deep_capture_api::CalibrationProcessSnapshot::complete(images),
        Err(error) => deep_capture_api::CalibrationProcessSnapshot::unavailable(error.message()),
    }
}

fn low_level_args(
    args: &CalibrateArgs,
    launch_case: Option<DeepCaptureLaunchCaseArg>,
    phase: deep_capture_api::CalibrationPhase,
    protocol: CompatibilityProtocol,
) -> Result<DeepCaptureArgs, CliError> {
    let calibration = match phase {
        deep_capture_api::CalibrationPhase::Reachability => DeepCaptureCalibrationArg::Reachability,
        deep_capture_api::CalibrationPhase::Tls => DeepCaptureCalibrationArg::Tls,
    };
    let calibration_protocol = protocol_arg(protocol)?;
    Ok(DeepCaptureArgs {
        selector: args.selector.clone(),
        target: args.target.clone(),
        id: args.id,
        catalog_db: args.catalog_db.clone(),
        local_db: args.local_db.clone(),
        launch: true,
        bundle: args.bundle.clone(),
        duration: args.duration,
        wait: args.wait,
        max_packets: args.max_packets,
        max_bytes: args.max_bytes,
        interface: args.interface.clone(),
        no_payload: args.no_payload,
        authorize_stdin: args.authorize_stdin,
        legacy_trust_ca: false,
        legacy_yes: false,
        restart_warm: false,
        calibrate: Some(calibration),
        calibration_protocol: Some(calibration_protocol),
        launch_case,
        har: false,
        key_log: false,
        client_certificate: None,
        client_private_key: None,
        proxy_family: DeepCaptureProxyFamilyArg::Ipv4,
        proxy_bypass: Vec::new(),
        controlled_target: args.controlled_target,
    })
}

fn protocol_arg(
    value: CompatibilityProtocol,
) -> Result<DeepCaptureCalibrationProtocolArg, CliError> {
    Ok(match value {
        CompatibilityProtocol::Routing => DeepCaptureCalibrationProtocolArg::Routing,
        CompatibilityProtocol::Http1 => DeepCaptureCalibrationProtocolArg::Http1,
        CompatibilityProtocol::Https => DeepCaptureCalibrationProtocolArg::Https,
        CompatibilityProtocol::Http2 => DeepCaptureCalibrationProtocolArg::Http2,
        CompatibilityProtocol::WebSocket => DeepCaptureCalibrationProtocolArg::Websocket,
        CompatibilityProtocol::Sse => DeepCaptureCalibrationProtocolArg::Sse,
        CompatibilityProtocol::Grpc => DeepCaptureCalibrationProtocolArg::Grpc,
        CompatibilityProtocol::GenericTcp => DeepCaptureCalibrationProtocolArg::GenericTcp,
        CompatibilityProtocol::NonHttpTls => DeepCaptureCalibrationProtocolArg::NonHttpTls,
        CompatibilityProtocol::Socks5Tcp => DeepCaptureCalibrationProtocolArg::Socks5Tcp,
        CompatibilityProtocol::Socks5Udp => DeepCaptureCalibrationProtocolArg::Socks5Udp,
        CompatibilityProtocol::GenericUdp => DeepCaptureCalibrationProtocolArg::GenericUdp,
        CompatibilityProtocol::Quic => DeepCaptureCalibrationProtocolArg::Quic,
        CompatibilityProtocol::Http3 => DeepCaptureCalibrationProtocolArg::Http3,
        CompatibilityProtocol::NotApplicable => {
            return Err(CliError::usage(
                "guided calibration selected no concrete protocol",
            ))
        }
    })
}

fn launch_case_arg(value: CompatibilityLaunchCase) -> DeepCaptureLaunchCaseArg {
    match value {
        CompatibilityLaunchCase::SteamProtocolWarm => DeepCaptureLaunchCaseArg::SteamProtocolWarm,
        CompatibilityLaunchCase::SteamProtocolCold => DeepCaptureLaunchCaseArg::SteamProtocolCold,
        CompatibilityLaunchCase::DirectExeWarm => DeepCaptureLaunchCaseArg::DirectExeWarm,
        CompatibilityLaunchCase::DirectExeCold => DeepCaptureLaunchCaseArg::DirectExeCold,
        CompatibilityLaunchCase::PublisherLauncher => DeepCaptureLaunchCaseArg::PublisherLauncher,
        CompatibilityLaunchCase::PublisherLauncherWarm => {
            DeepCaptureLaunchCaseArg::PublisherLauncherWarm
        }
        CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm => {
            DeepCaptureLaunchCaseArg::PublisherLauncherGameStartCleanWarm
        }
        CompatibilityLaunchCase::PublisherLauncherCold => {
            DeepCaptureLaunchCaseArg::PublisherLauncherCold
        }
    }
}

fn ready_launch_case(
    proposal: &deep_capture_api::CalibrationProposal,
) -> Result<CompatibilityLaunchCase, CliError> {
    match proposal.readiness {
        deep_capture_api::CalibrationLaunchReadiness::Ready { launch_case, .. } => Ok(launch_case),
        _ => Err(CliError::usage(
            "guided calibration did not retain a ready launch case",
        )),
    }
}

fn topology(proposal: &deep_capture_api::CalibrationProposal) -> Option<String> {
    proposal.topology.map(|value| value.as_str().to_string())
}

fn readiness_images(proposal: &deep_capture_api::CalibrationProposal) -> Vec<String> {
    match &proposal.readiness {
        deep_capture_api::CalibrationLaunchReadiness::Ready { images, .. }
        | deep_capture_api::CalibrationLaunchReadiness::OperatorAction { images, .. } => {
            images.clone()
        }
        _ => Vec::new(),
    }
}

fn limitation_messages(proposal: &deep_capture_api::CalibrationProposal) -> Vec<String> {
    proposal
        .limitations
        .iter()
        .map(|limitation| {
            let mut message = format!("{}: {}", limitation.kind.as_str(), limitation.detail);
            if !limitation.candidates.is_empty() {
                message.push_str(&format!(
                    " (candidates: {})",
                    limitation.candidates.join(", ")
                ));
            }
            message
        })
        .collect()
}

fn emit_guidance(emitter: &mut Emitter, target: &TargetEntry, guidance: Guidance) {
    emitter.event(&Event::CalibrationGuidance {
        target_id: target.stable_id,
        target: target.handle.clone(),
        topology: guidance.topology.clone(),
        action: guidance.action.to_string(),
        status: guidance.status.to_string(),
        observed_launch_case: guidance.observed_launch_case.clone(),
        selected_launch_case: guidance.selected_launch_case.clone(),
        reason: guidance.reason.clone(),
        images: guidance.images.clone(),
        limitations: guidance.limitations.clone(),
        requested_protocols: guidance.requested_protocols.clone(),
        observed_protocols: guidance.observed_protocols.clone(),
        completed_protocols: guidance.completed_protocols.clone(),
        remaining_protocols: guidance.remaining_protocols.clone(),
        process_control: "none".to_string(),
        next_command: guidance.next_command.clone(),
    });
    emitter.progress(&format!(
        "Calibration guidance: target={} id={} topology={} action={} status={} observed_launch_case={} selected_launch_case={} reason={} images={} limitations={} requested_protocols={} observed_protocols={} completed_protocols={} remaining_protocols={} process_control=none next_command={}",
        target.handle,
        target.stable_id,
        guidance.topology.as_deref().unwrap_or("unavailable"),
        guidance.action,
        guidance.status,
        guidance.observed_launch_case.as_deref().unwrap_or("none"),
        guidance.selected_launch_case.as_deref().unwrap_or("none"),
        guidance.reason.as_deref().unwrap_or("none"),
        if guidance.images.is_empty() { "none".to_string() } else { guidance.images.join(",") },
        if guidance.limitations.is_empty() { "none".to_string() } else { guidance.limitations.join("; ") },
        if guidance.requested_protocols.is_empty() { "none".to_string() } else { guidance.requested_protocols.join(",") },
        if guidance.observed_protocols.is_empty() { "none".to_string() } else { guidance.observed_protocols.join(",") },
        if guidance.completed_protocols.is_empty() { "none".to_string() } else { guidance.completed_protocols.join(",") },
        if guidance.remaining_protocols.is_empty() { "none".to_string() } else { guidance.remaining_protocols.join(",") },
        guidance.next_command.as_deref().unwrap_or("none"),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap::profile::FidelityTier;
    use fragcap::targets::{resolved_client_launch, ClassificationSource, TargetClassification};

    #[test]
    fn unavailable_process_inventory_remains_a_typed_no_step_limitation() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("local.db")).unwrap();
        let mut target = TargetEntry {
            id: None,
            stable_id: 90_001,
            handle: "inventory-fixture".to_string(),
            name: "Inventory Fixture".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(resolved_client_launch("fixture.exe")),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };
        target.id = Some(store.insert_target(&target).unwrap());
        let proposal = build_proposal(
            &store,
            &target,
            deep_capture_api::CalibrationProcessSnapshot::unavailable("snapshot failed"),
            &[],
        )
        .unwrap();
        assert!(proposal.steps.is_empty());
        assert_eq!(
            limitation_messages(&proposal),
            ["process-inventory-unavailable: snapshot failed (candidates: fixture.exe)"]
        );
        assert_eq!(
            completion_outcome(&proposal, CompatibilityLaunchCase::DirectExeCold),
            ("refused", "post-session-proposal-limitations")
        );
        assert_eq!(
            coverage_from_proposal(&proposal, &[CompatibilityProtocol::Https]),
            (Vec::new(), vec![CompatibilityProtocol::Https])
        );
    }

    #[test]
    fn completion_requires_ready_state_for_the_attempted_launch_case() {
        assert_eq!(
            completion_outcome_from_state(
                false,
                Some(CompatibilityLaunchCase::DirectExeCold),
                true,
                CompatibilityLaunchCase::DirectExeCold,
            ),
            ("completed", "current-routing-evidence-recorded")
        );
        assert_eq!(
            completion_outcome_from_state(
                false,
                Some(CompatibilityLaunchCase::DirectExeCold),
                true,
                CompatibilityLaunchCase::SteamProtocolCold,
            ),
            (
                "not-completed",
                "attempt-produced-no-current-positive-evidence"
            )
        );
        assert_eq!(
            completion_outcome_from_state(
                false,
                None,
                true,
                CompatibilityLaunchCase::DirectExeCold,
            ),
            ("not-completed", "post-session-launch-not-ready")
        );
    }

    #[test]
    fn warm_post_session_proposal_requires_restart_in_continuation() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("local.db")).unwrap();
        let mut target = TargetEntry {
            id: None,
            stable_id: 90_002,
            handle: "warm-fixture".to_string(),
            name: "Warm Fixture".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(resolved_client_launch("fixture.exe")),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };
        target.id = Some(store.insert_target(&target).unwrap());
        let proposal = build_proposal(
            &store,
            &target,
            deep_capture_api::CalibrationProcessSnapshot::complete(["fixture.exe"]),
            &[CompatibilityProtocol::Https],
        )
        .unwrap();

        assert!(matches!(
            proposal.readiness,
            deep_capture_api::CalibrationLaunchReadiness::OperatorAction { .. }
        ));
        assert_eq!(
            continuation_command(
                &proposal,
                true,
                target.stable_id,
                "local.db",
                &[CompatibilityProtocol::Https],
                &[CompatibilityProtocol::Https],
            )
            .as_deref(),
            Some(
                "fragcap calibrate --id 90002 --local-db local.db --restart-warm --protocol https"
            )
        );
    }
}
