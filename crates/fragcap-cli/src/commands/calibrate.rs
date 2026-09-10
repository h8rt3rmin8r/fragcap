// SPDX-License-Identifier: Apache-2.0

//! Guided reachability calibration for one already registered target.

use std::path::Path;

use fragcap::deep_capture::api as deep_capture_api;
use fragcap::targets::{
    CompatibilityAddressFamily, CompatibilityLaunchCase, CompatibilityProtocol,
    CompatibilityRoutingStrategy, Store, TargetEntry,
};

use crate::cli::{
    CalibrateArgs, DeepCaptureArgs, DeepCaptureCalibrationArg, DeepCaptureCalibrationProtocolArg,
    DeepCaptureLaunchCaseArg, DeepCaptureProxyFamilyArg,
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
    next_command: Option<String>,
}

pub fn run(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
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
    let proposal = build_proposal(&store, &target, snapshot.clone())?;
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
            let next_command = target_command(
                "calibrate",
                target.stable_id,
                &local_store_argument,
                " --restart-warm",
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
                    next_command: Some(next_command),
                },
            );
            if !args.restart_warm {
                return Ok(Exit::SUCCESS);
            }
            let mut restart_args = low_level_args(args, None);
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
                    next_command: None,
                },
            );
            return Err(CliError::usage(
                "guided calibration could not determine an exact launch case",
            ));
        }
    }

    let mut low_level = low_level_args(args, None);
    let fresh_store = deep_capture::open_local_store(args.local_db.as_deref())?;
    let fresh_target = deep_capture::resolve_target(&fresh_store, &low_level)?;
    let fresh_snapshot = if args.restart_warm {
        process_snapshot(args.controlled_target)
    } else {
        snapshot
    };
    let selected = build_proposal(&fresh_store, &fresh_target, fresh_snapshot.clone())?;
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
        emit_guidance(
            emitter,
            &fresh_target,
            Guidance {
                topology: topology(&selected),
                action: "ready",
                status: "ready",
                observed_launch_case: None,
                selected_launch_case: Some(launch_case.as_str().to_string()),
                reason: Some("current-routing-evidence".to_string()),
                images: readiness_images(&selected),
                limitations: Vec::new(),
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
    if selected.steps.len() != 1 {
        return Err(CliError::usage(
            "guided calibration selected more than one attempt; no session was started",
        ));
    }
    let step = &selected.steps[0];
    if step.phase != deep_capture_api::CalibrationPhase::Reachability
        || step.case.protocol != CompatibilityProtocol::Routing
    {
        return Err(CliError::usage(
            "guided calibration selected an unsupported phase or protocol; no session was started",
        ));
    }
    low_level.launch_case = Some(launch_case_arg(step.case.launch_case));
    emit_guidance(
        emitter,
        &fresh_target,
        Guidance {
            topology: topology(&selected),
            action: "run-reachability",
            status: "selected",
            observed_launch_case: None,
            selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
            reason: Some(step.reason.as_str().to_string()),
            images: readiness_images(&selected),
            limitations: Vec::new(),
            next_command: None,
        },
    );
    drop(fresh_store);
    let exit = match deep_capture::run(&low_level, authorization, emitter) {
        Ok(exit) => exit,
        Err(error) => {
            emit_guidance(
                emitter,
                &fresh_target,
                Guidance {
                    topology: topology(&selected),
                    action: "run-reachability",
                    status: "failed",
                    observed_launch_case: None,
                    selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                    reason: Some("delegated-session-error".to_string()),
                    images: readiness_images(&selected),
                    limitations: Vec::new(),
                    next_command: None,
                },
            );
            return Err(error);
        }
    };
    if exit != Exit::SUCCESS {
        emit_guidance(
            emitter,
            &fresh_target,
            Guidance {
                topology: topology(&selected),
                action: "run-reachability",
                status: "failed",
                observed_launch_case: None,
                selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                reason: Some("delegated-session-nonzero".to_string()),
                images: readiness_images(&selected),
                limitations: Vec::new(),
                next_command: None,
            },
        );
        return Ok(exit);
    }

    let completed_store = deep_capture::open_local_store(args.local_db.as_deref())?;
    let completed_target = deep_capture::resolve_target(&completed_store, &low_level)?;
    let completed = build_proposal(&completed_store, &completed_target, fresh_snapshot)?;
    let completed_status = if completed.steps.is_empty() {
        "completed"
    } else {
        "not-completed"
    };
    emit_guidance(
        emitter,
        &completed_target,
        Guidance {
            topology: topology(&completed),
            action: "run-reachability",
            status: completed_status,
            observed_launch_case: None,
            selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
            reason: Some(if completed.steps.is_empty() {
                "current-routing-evidence-recorded".to_string()
            } else {
                "attempt-produced-no-current-positive-evidence".to_string()
            }),
            images: readiness_images(&completed),
            limitations: limitation_messages(&completed),
            next_command: Some(target_command(
                "calibrate",
                completed_target.stable_id,
                &local_store_argument,
                "",
            )),
        },
    );
    Ok(Exit::SUCCESS)
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

fn build_proposal(
    store: &Store,
    target: &TargetEntry,
    snapshot: deep_capture_api::CalibrationProcessSnapshot,
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
) -> DeepCaptureArgs {
    DeepCaptureArgs {
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
        calibrate: Some(DeepCaptureCalibrationArg::Reachability),
        calibration_protocol: Some(DeepCaptureCalibrationProtocolArg::Routing),
        launch_case,
        har: false,
        key_log: false,
        client_certificate: None,
        client_private_key: None,
        proxy_family: DeepCaptureProxyFamilyArg::Ipv4,
        proxy_bypass: Vec::new(),
        controlled_target: args.controlled_target,
    }
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
        process_control: "none".to_string(),
        next_command: guidance.next_command.clone(),
    });
    emitter.progress(&format!(
        "Calibration guidance: target={} id={} topology={} action={} status={} observed_launch_case={} selected_launch_case={} reason={} images={} limitations={} process_control=none next_command={}",
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
        )
        .unwrap();
        assert!(proposal.steps.is_empty());
        assert_eq!(
            limitation_messages(&proposal),
            ["process-inventory-unavailable: snapshot failed (candidates: fixture.exe)"]
        );
    }
}
