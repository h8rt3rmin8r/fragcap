// SPDX-License-Identifier: Apache-2.0

//! Guided registration and bounded calibration sequencing for one exact target.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use fragcap::deep_capture::api as deep_capture_api;
use fragcap::targets::{
    AuthorTargetClientOutcome, CalibrationPauseReason, CalibrationWorkflow,
    CalibrationWorkflowCheckpoint, CalibrationWorkflowPhase, CalibrationWorkflowState,
    CalibrationWorkflowUpdateOutcome, CandidateIdentity, CandidateTarget,
    CompatibilityAddressFamily, CompatibilityLaunchCase, CompatibilityProtocol,
    CompatibilityRoutingStrategy, Discovery, DiscoveryAccount, LaunchEntry, Selection, Store,
    TargetEntry,
};
use serde_json::{json, Value};
use subtle::ConstantTimeEq;

use crate::cli::{
    CalibrateArgs, DeepCaptureArgs, DeepCaptureCalibrationArg, DeepCaptureCalibrationProtocolArg,
    DeepCaptureLaunchCaseArg, DeepCaptureProxyFamilyArg, GuidedCalibrationPauseArg,
    GuidedCalibrationProtocolArg, GuidedCalibrationRoutingArg,
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

#[derive(Clone, Copy)]
struct AttemptProgress {
    number: u64,
    phase: deep_capture_api::CalibrationPhase,
    protocol: CompatibilityProtocol,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ExactAttemptCase {
    phase: &'static str,
    launch_case: CompatibilityLaunchCase,
    proxy_backend: String,
    proxy_backend_version: String,
    routing_strategy: CompatibilityRoutingStrategy,
    address_family: CompatibilityAddressFamily,
    protocol: CompatibilityProtocol,
    fragcap_version: String,
    target_version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SequenceTargetAuthority {
    row_id: Option<i64>,
    stable_id: i64,
    handle: String,
    name: String,
    anchor: Option<String>,
    install_root: Option<String>,
    launch_entries: Option<Value>,
}

impl SequenceTargetAuthority {
    fn from_target(target: &TargetEntry) -> Self {
        Self {
            row_id: target.id,
            stable_id: target.stable_id,
            handle: target.handle.clone(),
            name: target.name.clone(),
            anchor: target.anchor.clone(),
            install_root: target.install_root.clone(),
            launch_entries: target.launch_entries.clone(),
        }
    }

    fn matches(&self, target: &TargetEntry) -> bool {
        self == &Self::from_target(target)
    }
}

impl ExactAttemptCase {
    fn from_step(step: &deep_capture_api::CalibrationProposalStep) -> Self {
        Self {
            phase: step.phase.as_str(),
            launch_case: step.case.launch_case,
            proxy_backend: step.case.proxy_backend.clone(),
            proxy_backend_version: step.case.proxy_backend_version.clone(),
            routing_strategy: step.case.routing_strategy,
            address_family: step.case.address_family,
            protocol: step.case.protocol,
            fragcap_version: step.case.fragcap_version.clone(),
            target_version: step.case.target_version.clone(),
        }
    }

    fn durable_key(&self) -> String {
        json!({
            "address_family": self.address_family.as_str(),
            "fragcap_version": self.fragcap_version,
            "launch_case": self.launch_case.as_str(),
            "phase": self.phase,
            "protocol": self.protocol.as_str(),
            "proxy_backend": self.proxy_backend,
            "proxy_backend_version": self.proxy_backend_version,
            "routing_strategy": self.routing_strategy.as_str(),
            "target_version": self.target_version,
        })
        .to_string()
    }
}

const GUIDED_CONCRETE_PROTOCOLS: [CompatibilityProtocol; 13] = [
    CompatibilityProtocol::Http1,
    CompatibilityProtocol::Https,
    CompatibilityProtocol::Http2,
    CompatibilityProtocol::WebSocket,
    CompatibilityProtocol::Sse,
    CompatibilityProtocol::Grpc,
    CompatibilityProtocol::GenericTcp,
    CompatibilityProtocol::NonHttpTls,
    CompatibilityProtocol::Socks5Tcp,
    CompatibilityProtocol::Socks5Udp,
    CompatibilityProtocol::GenericUdp,
    CompatibilityProtocol::Quic,
    CompatibilityProtocol::Http3,
];
const MAX_GUIDED_ATTEMPTS: usize = GUIDED_CONCRETE_PROTOCOLS.len() + 1;

fn guided_sequence_interrupted(interrupt: &AtomicBool) -> bool {
    interrupt.load(Ordering::Relaxed)
}

fn no_progress_reason(attempted_count: usize, inserted: bool) -> Option<&'static str> {
    if attempted_count >= MAX_GUIDED_ATTEMPTS {
        Some("attempt-bound-exhausted")
    } else if !inserted {
        Some("attempt-case-already-executed")
    } else {
        None
    }
}

const REGISTRATION_PLAN_SCHEMA: &str = "fragcap.target-registration-plan.v1";
const REGISTRATION_OPERATION: &str = "register-candidate-v1";
const REGISTRATION_PLAN_PREFIX: &str = "target-registration-v1:";
const STEAM_CLIENT_PLAN_SCHEMA: &str = "fragcap.steam-client-setup-plan.v1";
const STEAM_CLIENT_OPERATION: &str = "author-target-client-if-unchanged-v1";
const STEAM_CLIENT_PLAN_PREFIX: &str = "steam-client-setup-v1:";
const STORED_CLIENT_PLAN_SCHEMA: &str = "fragcap.stored-client-selection-plan.v1";
const STORED_CLIENT_OPERATION: &str = "select-stored-client-if-unchanged-v1";
const STORED_CLIENT_PLAN_PREFIX: &str = "stored-client-selection-v1:";

#[derive(Clone, Debug)]
struct RegistrationPlan {
    id: String,
    canonical: Value,
    canonical_json: String,
    candidate: CandidateTarget,
    discovery_account: DiscoveryAccount,
    discovery_warning_count: usize,
}

#[derive(Clone, Debug)]
struct SteamClientPlan {
    id: String,
    canonical: Value,
    canonical_json: String,
    target: TargetEntry,
    app_id: u32,
    executable: String,
    candidate: CandidateTarget,
    discovery_account: DiscoveryAccount,
    discovery_warning_count: usize,
}

#[derive(Clone, Debug)]
struct StoredClientPlan {
    id: String,
    canonical: Value,
    canonical_json: String,
    target: TargetEntry,
    launch_entry: LaunchEntry,
}

impl StoredClientPlan {
    fn new(target: TargetEntry, launch_entry: LaunchEntry, local_store: &Path) -> Self {
        let resulting_launch_entries =
            Value::Array(vec![fragcap::targets::launch_entry_value(&launch_entry)]);
        let canonical = json!({
            "local_store": crate::commands::target_resolve::resolve_store_identity(local_store).to_string_lossy(),
            "no_effects": [
                "no-process-control",
                "no-process-launch",
                "no-proxy-or-routing",
                "no-session-authorization",
                "no-system-trust-change",
            ],
            "operation": STORED_CLIENT_OPERATION,
            "proposed_executable": launch_entry.executable(),
            "resulting_launch_entries": resulting_launch_entries,
            "schema": STORED_CLIENT_PLAN_SCHEMA,
            "target": target_plan_value(&target),
        });
        let canonical_json = serde_json::to_string(&canonical)
            .expect("the stored client plan contains only serializable values");
        let digest = blake3::hash(canonical_json.as_bytes()).to_hex();
        Self {
            id: format!("{STORED_CLIENT_PLAN_PREFIX}{digest}"),
            canonical,
            canonical_json,
            target,
            launch_entry,
        }
    }

    fn emit(&self, emitter: &mut Emitter) -> Result<(), CliError> {
        emitter
            .event_checked(&Event::CalibrationStoredClientPlan {
                plan_id: self.id.clone(),
                canonical_json: self.canonical_json.clone(),
                target_id: self.target.stable_id,
                executable: self.launch_entry.executable().to_string(),
            })
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the stored client selection plan: {error}"
                ))
            })?;
        let rendered = serde_json::to_string_pretty(&self.canonical)
            .expect("the stored client plan contains only serializable values");
        emitter
            .required_human_checked(&format!(
                "Stored client selection plan\n  plan id: {}\n{}\n",
                self.id, rendered
            ))
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the stored client selection plan: {error}"
                ))
            })
    }
}

impl SteamClientPlan {
    fn new(
        target: TargetEntry,
        discovery: &Discovery,
        local_store: &Path,
    ) -> Result<Option<Self>, CliError> {
        if target.launch_entries.is_some() {
            return Ok(None);
        }
        let Some(app_id) = steam_app_id(&target) else {
            return Ok(None);
        };
        if !discovery.account.is_conserved() {
            return Err(CliError::failure(
                "discovery accounting was not conserved; refusing Steam client setup",
            ));
        }
        let candidates: Vec<_> = discovery
            .candidates
            .iter()
            .filter(|candidate| {
                candidate.identity == CandidateIdentity::SteamAppId(app_id)
                    && candidate.source_name == "steam"
            })
            .collect();
        let candidate = match candidates.as_slice() {
            [candidate] => *candidate,
            [] => {
                return Err(CliError::usage(format!(
                    "Steam client setup is unavailable: discovery returned no exact steam:{app_id} candidate; refresh Steam metadata and retry"
                )))
            }
            _ => {
                return Err(CliError::usage(format!(
                    "Steam client setup is unavailable: discovery returned {} exact steam:{app_id} candidates; resolve the conflicting Steam metadata and retry",
                    candidates.len()
                )))
            }
        };
        let Some(target_root) = target.install_root.as_deref() else {
            return Err(CliError::usage(
                "Steam client setup is unavailable: the stored target has no exact install root; refresh or re-register the target",
            ));
        };
        if candidate.install_root.as_deref() != Some(target_root) {
            return Err(CliError::usage(
                "Steam client setup is unavailable: the discovered Steam install root does not match the stored target; refresh the target authority and retry",
            ));
        }
        let Some(executable) = candidate.executable_hint.as_deref() else {
            return Err(CliError::usage(
                "Steam client setup is unavailable: Steam metadata did not provide an executable path; configure the launch declaration explicitly",
            ));
        };
        if !fragcap::targets::is_client_executable(executable) {
            return Err(CliError::usage(
                "Steam client setup is unavailable: the Steam executable hint is not one unambiguous executable path; configure the launch declaration explicitly",
            ));
        }

        let canonical = steam_client_plan_value(
            &target,
            candidate,
            discovery,
            local_store,
            app_id,
            executable,
        );
        let canonical_json = serde_json::to_string(&canonical)
            .expect("the Steam client plan contains only serializable values");
        let digest = blake3::hash(canonical_json.as_bytes()).to_hex();
        Ok(Some(Self {
            id: format!("{STEAM_CLIENT_PLAN_PREFIX}{digest}"),
            canonical,
            canonical_json,
            target,
            app_id,
            executable: executable.to_string(),
            candidate: candidate.clone(),
            discovery_account: discovery.account.clone(),
            discovery_warning_count: discovery.warnings.len(),
        }))
    }

    fn emit(&self, emitter: &mut Emitter) -> Result<(), CliError> {
        emitter
            .event_checked(&Event::CalibrationSteamClientPlan {
                plan_id: self.id.clone(),
                canonical_json: self.canonical_json.clone(),
                target_id: self.target.stable_id,
                steam_app_id: self.app_id,
                executable: self.executable.clone(),
                discovery_considered: self.discovery_account.considered,
                discovery_produced: self.discovery_account.produced,
                discovery_parse_failed: self.discovery_account.parse_failed,
                discovery_declined_by_user: self.discovery_account.declined_by_user,
                discovery_considered_not_a_game: self.discovery_account.considered_not_a_game,
                discovery_container_descended: self.discovery_account.container_descended,
                discovery_container_descent_truncated: self
                    .discovery_account
                    .container_descent_truncated,
                discovery_volume_skipped: self.discovery_account.volume_skipped,
                discovery_access_error: self.discovery_account.access_error,
                discovery_warning_count: self.discovery_warning_count as u64,
            })
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the Steam client setup plan: {error}"
                ))
            })?;
        let rendered = serde_json::to_string_pretty(&self.canonical)
            .expect("the Steam client plan contains only serializable values");
        emitter
            .required_human_checked(&format!(
                "Steam client setup plan\n  plan id: {}\n{}\n",
                self.id, rendered
            ))
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the Steam client setup plan: {error}"
                ))
            })
    }
}

impl RegistrationPlan {
    fn new(
        candidate: CandidateTarget,
        discovery: &Discovery,
        local_store: &Path,
    ) -> Result<Self, CliError> {
        if !discovery.account.is_conserved() {
            return Err(CliError::failure(
                "discovery accounting was not conserved; refusing target registration",
            ));
        }
        let canonical = registration_plan_value(&candidate, discovery, local_store)?;
        let canonical_json = serde_json::to_string(&canonical)
            .expect("the registration plan contains only serializable values");
        let digest = blake3::hash(canonical_json.as_bytes()).to_hex();
        Ok(Self {
            id: format!("{REGISTRATION_PLAN_PREFIX}{digest}"),
            canonical,
            canonical_json,
            candidate,
            discovery_account: discovery.account.clone(),
            discovery_warning_count: discovery.warnings.len(),
        })
    }

    fn emit(&self, emitter: &mut Emitter) -> Result<(), CliError> {
        emitter
            .event_checked(&Event::CalibrationRegistrationPlan {
                plan_id: self.id.clone(),
                canonical_json: self.canonical_json.clone(),
                discovery_considered: self.discovery_account.considered,
                discovery_produced: self.discovery_account.produced,
                discovery_parse_failed: self.discovery_account.parse_failed,
                discovery_declined_by_user: self.discovery_account.declined_by_user,
                discovery_considered_not_a_game: self.discovery_account.considered_not_a_game,
                discovery_container_descended: self.discovery_account.container_descended,
                discovery_container_descent_truncated: self
                    .discovery_account
                    .container_descent_truncated,
                discovery_volume_skipped: self.discovery_account.volume_skipped,
                discovery_access_error: self.discovery_account.access_error,
                discovery_warning_count: self.discovery_warning_count as u64,
            })
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the target registration plan: {error}"
                ))
            })?;
        let rendered = serde_json::to_string_pretty(&self.canonical)
            .expect("the registration plan contains only serializable values");
        emitter
            .required_human_checked(&format!(
                "Target registration plan\n  plan id: {}\n{}\n",
                self.id, rendered
            ))
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the target registration plan: {error}"
                ))
            })
    }
}

enum TargetFrontDoor {
    Ready(Box<TargetEntry>),
    Declined,
}

struct CandidateSelection {
    requested: Option<String>,
    consumed: bool,
}

impl CandidateSelection {
    fn new(requested: Option<&str>) -> Result<Self, CliError> {
        if let Some(value) = requested {
            fragcap::targets::validate_calibration_candidate_id(value)
                .map_err(|error| CliError::usage(error.to_string()))?;
        }
        Ok(Self {
            requested: requested.map(str::to_string),
            consumed: false,
        })
    }

    fn consume(&mut self, value: &str) -> Result<(), CliError> {
        if self.consumed {
            return Err(CliError::usage(
                "the supplied calibration candidate was already consumed by an earlier ambiguity",
            ));
        }
        if self.requested.as_deref() != Some(value) {
            return Err(CliError::usage(
                "the current calibration candidate does not match the supplied identity",
            ));
        }
        self.consumed = true;
        Ok(())
    }

    fn require_consumed(&self) -> Result<(), CliError> {
        if self.requested.is_some() && !self.consumed {
            return Err(CliError::usage(
                "the supplied calibration candidate was not consumed by a current ambiguous target, Steam client, or stored client choice",
            ));
        }
        Ok(())
    }

    fn has_unconsumed(&self) -> bool {
        self.requested.is_some() && !self.consumed
    }
}

enum RegistrationConfirmation {
    Confirmed,
    Declined,
    Closed,
    Invalid,
    Interrupted,
}

fn resolve_or_register_target(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
    local_store_path: &Path,
    store: &mut Store,
    candidate_selection: &mut CandidateSelection,
) -> Result<TargetFrontDoor, CliError> {
    match deep_capture::select_target_input(
        store,
        args.selector.as_deref(),
        args.target.as_deref(),
        args.id,
    )? {
        Selection::Resolved(target) => return Ok(TargetFrontDoor::Ready(target)),
        Selection::Ambiguous(_) => {
            return deep_capture::resolve_target_input(
                store,
                args.selector.as_deref(),
                args.target.as_deref(),
                args.id,
            )
            .map(|target| TargetFrontDoor::Ready(Box::new(target)))
        }
        Selection::NoMatch => {}
    }

    if args.id.is_some() {
        return deep_capture::resolve_target_input(
            store,
            args.selector.as_deref(),
            args.target.as_deref(),
            args.id,
        )
        .map(|target| TargetFrontDoor::Ready(Box::new(target)));
    }
    if emitter.is_json() && !args.authorize_stdin {
        return Err(CliError::usage(
            "JSON target registration requires --authorize-stdin and the exact emitted plan identifier",
        ));
    }
    if !args.authorize_stdin && !authorization.is_terminal() {
        return Err(CliError::usage(
            "target registration requires an interactive terminal or --authorize-stdin",
        ));
    }

    let selector = args
        .selector
        .as_deref()
        .or(args.target.as_deref())
        .ok_or_else(|| {
            CliError::usage("a positional target selector or --target is required for discovery")
        })?;
    let discovery = discover_for_calibration(args, store, emitter)?;
    let candidate =
        select_discovery_candidate_with_choice(selector, &discovery, candidate_selection, emitter)?;
    let plan = RegistrationPlan::new(candidate, &discovery, local_store_path)?;
    plan.emit(emitter)?;
    let confirmation = match confirm_registration(args, authorization, emitter, &plan) {
        Ok(confirmation) => confirmation,
        Err(error) => {
            let _ = registration_outcome(
                emitter,
                &plan.id,
                "failed",
                "target registration confirmation could not be completed",
                None,
                false,
            );
            return Err(error);
        }
    };
    match confirmation {
        RegistrationConfirmation::Confirmed => {}
        RegistrationConfirmation::Declined => {
            registration_outcome(
                emitter,
                &plan.id,
                "declined",
                "operator declined target registration",
                None,
                false,
            )?;
            return Ok(TargetFrontDoor::Declined);
        }
        RegistrationConfirmation::Closed => {
            registration_outcome(
                emitter,
                &plan.id,
                "closed",
                "target registration input closed before confirmation",
                None,
                false,
            )?;
            return Ok(TargetFrontDoor::Declined);
        }
        RegistrationConfirmation::Invalid => {
            registration_outcome(
                emitter,
                &plan.id,
                "invalid",
                "exact current registration plan identifier was not supplied",
                None,
                false,
            )?;
            return Err(CliError::usage(
                "target registration confirmation did not match the exact current plan identifier; no target was registered",
            ));
        }
        RegistrationConfirmation::Interrupted => {
            registration_outcome(
                emitter,
                &plan.id,
                "interrupted",
                "interrupt requested before target registration completed",
                None,
                false,
            )?;
            return Err(CliError::failure(
                "target registration was interrupted; no target was registered",
            ));
        }
    }

    let current_discovery = match discover_for_calibration(args, store, emitter) {
        Ok(discovery) => discovery,
        Err(_) => {
            registration_outcome(
                emitter,
                &plan.id,
                "drifted",
                "discovered target authority could not be reproduced after confirmation",
                None,
                false,
            )?;
            return Err(CliError::usage(
                "the discovered target could not be reproduced after confirmation; review a fresh registration plan",
            ));
        }
    };
    let current_candidate = match revalidate_candidate(&plan.candidate, &current_discovery) {
        Ok(candidate) => candidate,
        Err(error) => {
            let mut no_selection = CandidateSelection::new(None)?;
            let _ = select_discovery_candidate_with_choice(
                selector,
                &current_discovery,
                &mut no_selection,
                emitter,
            );
            registration_outcome(
                emitter,
                &plan.id,
                "drifted",
                "discovered target selection changed after confirmation",
                None,
                false,
            )?;
            return Err(error);
        }
    };
    let current_plan =
        match RegistrationPlan::new(current_candidate, &current_discovery, local_store_path) {
            Ok(plan) => plan,
            Err(_) => {
                registration_outcome(
                    emitter,
                    &plan.id,
                    "drifted",
                    "discovery accounting could not reproduce the confirmed authority",
                    None,
                    false,
                )?;
                return Err(CliError::usage(
                "discovery accounting changed after confirmation; review a fresh registration plan",
            ));
            }
        };
    if !constant_time_equal(
        plan.canonical_json.as_bytes(),
        current_plan.canonical_json.as_bytes(),
    ) {
        let mut no_selection = CandidateSelection::new(None)?;
        let _ = select_discovery_candidate_with_choice(
            selector,
            &current_discovery,
            &mut no_selection,
            emitter,
        );
        registration_outcome(
            emitter,
            &plan.id,
            "drifted",
            "discovered target authority changed after confirmation",
            None,
            false,
        )?;
        return Err(CliError::usage(
            "the discovered target changed after confirmation; review a fresh registration plan",
        ));
    }

    let inserted = match fragcap::targets::register_candidate(store, &plan.candidate) {
        Ok(inserted) => inserted,
        Err(error) => {
            registration_outcome(
                emitter,
                &plan.id,
                "failed",
                "the shared target registration operation failed",
                None,
                false,
            )?;
            return Err(CliError::failure(error.to_string()));
        }
    };
    let target = match registered_candidate_target(store, &plan.candidate) {
        Ok(target) => target,
        Err(error) => {
            registration_outcome(
                emitter,
                &plan.id,
                "failed",
                "the registered target could not be resolved exactly",
                None,
                false,
            )?;
            return Err(error);
        }
    };
    registration_outcome(
        emitter,
        &plan.id,
        if inserted {
            "registered"
        } else {
            "already-present"
        },
        if inserted {
            "confirmed candidate registered through the shared target operation"
        } else {
            "confirmed candidate already had the same durable target identity"
        },
        Some(target.stable_id),
        true,
    )?;
    Ok(TargetFrontDoor::Ready(Box::new(target)))
}

fn discover_for_calibration(
    args: &CalibrateArgs,
    store: &mut Store,
    emitter: &mut Emitter,
) -> Result<Discovery, CliError> {
    if args.controlled_target {
        let mut account = fragcap::targets::DiscoveryAccount::default();
        account.produce();
        let drifted = std::env::var_os("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DRIFT").is_some();
        let steam_client_drift =
            std::env::var_os("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_DRIFT").is_some();
        let ambiguous =
            std::env::var_os("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_AMBIGUOUS").is_some();
        let steam_client_ambiguous =
            std::env::var_os("FRAGCAP_CONTROLLED_TARGET_STEAM_CLIENT_AMBIGUOUS").is_some();
        let duplicate =
            std::env::var_os("FRAGCAP_CONTROLLED_TARGET_REGISTRATION_DUPLICATE").is_some();
        let mut candidates = vec![CandidateTarget {
            identity: CandidateIdentity::SteamAppId(75_000),
            display_name: if drifted {
                "Changed Sample Target".to_string()
            } else {
                "Sample Target".to_string()
            },
            fidelity: fragcap::profile::FidelityTier::Observed,
            classification: fragcap::targets::TargetClassification::Game,
            evidence: Vec::new(),
            detection_scan: None,
            source_name: "steam".to_string(),
            install_root: Some("C:\\Games\\Sample Target".to_string()),
            folder_name: Some("Sample Target".to_string()),
            executable_hint: Some(if steam_client_drift {
                "changed-client.exe".to_string()
            } else {
                "client.exe".to_string()
            }),
        }];
        if ambiguous {
            account.produce();
            candidates.push(CandidateTarget {
                identity: CandidateIdentity::Path(
                    "C:\\Other Games\\Sample Target\\client.exe".to_string(),
                ),
                display_name: "Sample Target".to_string(),
                fidelity: fragcap::profile::FidelityTier::Observed,
                classification: fragcap::targets::TargetClassification::Game,
                evidence: Vec::new(),
                detection_scan: None,
                source_name: "filesystem".to_string(),
                install_root: Some("C:\\Other Games\\Sample Target".to_string()),
                folder_name: Some("Sample Target".to_string()),
                executable_hint: Some("client.exe".to_string()),
            });
        }
        if steam_client_ambiguous {
            account.produce();
            candidates.push(CandidateTarget {
                identity: CandidateIdentity::SteamAppId(75_000),
                display_name: "Conflicting Sample Target".to_string(),
                fidelity: fragcap::profile::FidelityTier::Observed,
                classification: fragcap::targets::TargetClassification::Game,
                evidence: Vec::new(),
                detection_scan: None,
                source_name: "steam".to_string(),
                install_root: Some("D:\\Games\\Sample Target".to_string()),
                folder_name: Some("Sample Target".to_string()),
                executable_hint: Some("other-client.exe".to_string()),
            });
        }
        if duplicate {
            account.produce();
            candidates.push(candidates[0].clone());
        }
        return Ok(Discovery {
            candidates,
            account,
            warnings: Vec::new(),
        });
    }

    let catalog = match crate::commands::target_resolve::ensure_catalog_store(
        args.catalog_db.as_deref(),
    ) {
        Ok(Some(path)) if path.is_file() => path,
        Ok(Some(path)) => {
            return Err(CliError::failure(format!(
                "catalog store does not exist at {}; installed-target discovery was not attempted",
                path.display()
            )))
        }
        Ok(None) => {
            let warning = "catalog store location is unavailable; installed-target discovery was not attempted";
            emitter.warn(warning);
            return Ok(Discovery {
                warnings: vec![warning.to_string()],
                ..Discovery::default()
            });
        }
        Err(message) => {
            emitter.warn(&message);
            return Ok(Discovery {
                warnings: vec![message],
                ..Discovery::default()
            });
        }
    };
    let discovery = crate::commands::targets::compose_and_discover(&catalog, store, None)?;
    for warning in &discovery.warnings {
        emitter.warn(warning);
    }
    Ok(discovery)
}

fn select_discovery_candidate_with_choice(
    selector: &str,
    discovery: &Discovery,
    selection: &mut CandidateSelection,
    emitter: &mut Emitter,
) -> Result<CandidateTarget, CliError> {
    let steam_id = selector.parse::<u32>().ok();
    let folded_selector = selector.to_lowercase();
    let matches: Vec<_> = discovery
        .candidates
        .iter()
        .filter(|candidate| {
            steam_id.is_some_and(|appid| candidate.identity == CandidateIdentity::SteamAppId(appid))
                || candidate.display_name.to_lowercase() == folded_selector
        })
        .cloned()
        .collect();
    match matches.as_slice() {
        [candidate] if !selection.has_unconsumed() => Ok(candidate.clone()),
        [_] => Err(CliError::usage(
            "the supplied calibration candidate has no current ambiguous target-registration choice",
        )),
        [] => Err(CliError::usage(format!(
            "no stored or discovered target exactly matches {selector:?}; discovery considered {}, produced {}, and reported {} warning(s)",
            discovery.account.considered,
            discovery.account.produced,
            discovery.warnings.len(),
        ))),
        _ => choose_ambiguous_candidate(
            "target-registration",
            selector,
            None,
            matches,
            discovery,
            selection,
            emitter,
        ),
    }
}

#[cfg(test)]
fn select_discovery_candidate(
    selector: &str,
    discovery: &Discovery,
) -> Result<CandidateTarget, CliError> {
    let steam_id = selector.parse::<u32>().ok();
    let folded_selector = selector.to_lowercase();
    let matches = discovery
        .candidates
        .iter()
        .filter(|candidate| {
            steam_id.is_some_and(|appid| candidate.identity == CandidateIdentity::SteamAppId(appid))
                || candidate.display_name.to_lowercase() == folded_selector
        })
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(CliError::usage(format!(
            "no stored or discovered target exactly matches {selector:?}"
        ))),
        _ => Err(CliError::usage(format!(
            "the discovered selector is ambiguous ({} exact candidates match); no target was selected",
            matches.len()
        ))),
    }
}

fn choose_ambiguous_candidate(
    scope: &str,
    selector: &str,
    target_id: Option<i64>,
    candidates: Vec<CandidateTarget>,
    discovery: &Discovery,
    selection: &mut CandidateSelection,
    emitter: &mut Emitter,
) -> Result<CandidateTarget, CliError> {
    let mut choices = candidates
        .into_iter()
        .map(|candidate| {
            let id = fragcap::targets::calibration_candidate_id(&candidate);
            (id, candidate)
        })
        .collect::<Vec<_>>();
    choices.sort_by(|left, right| left.0.cmp(&right.0));
    emit_candidate_choices(scope, selector, target_id, &choices, discovery, emitter)?;
    if choices.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(CliError::usage(
            "calibration discovery produced duplicate candidate authority; no candidate was selected",
        ));
    }
    let Some(requested) = selection.requested.as_deref() else {
        return Err(CliError::usage(format!(
            "the calibration {scope} is ambiguous; rerun with one exact --candidate identity"
        )));
    };
    let selected = choices
        .iter()
        .find(|(id, _)| id == requested)
        .map(|(_, candidate)| candidate.clone())
        .ok_or_else(|| {
            CliError::usage(format!(
                "the supplied calibration candidate is not present in the current {scope} choices"
            ))
        })?;
    let requested = requested.to_string();
    selection.consume(&requested)?;
    Ok(selected)
}

fn emit_candidate_choices(
    scope: &str,
    selector: &str,
    target_id: Option<i64>,
    choices: &[(String, CandidateTarget)],
    discovery: &Discovery,
    emitter: &mut Emitter,
) -> Result<(), CliError> {
    let projections = choices
        .iter()
        .map(|(id, candidate)| {
            json!({
                "classification": candidate.classification.as_str(),
                "display_name": candidate.display_name,
                "executable_hint": candidate.executable_hint,
                "fidelity": candidate.fidelity.as_str(),
                "id": id,
                "identity": candidate_identity(&candidate.identity),
                "install_root": candidate.install_root,
                "source": candidate.source_name,
            })
        })
        .collect::<Vec<_>>();
    emitter
        .event_checked(&Event::CalibrationChoiceRequired {
            scope: scope.to_string(),
            selector: selector.to_string(),
            target_id,
            choices: projections.clone(),
            discovery_considered: discovery.account.considered,
            discovery_produced: discovery.account.produced,
            discovery_warning_count: discovery.warnings.len() as u64,
            process_control: "none".to_string(),
        })
        .map_err(|error| CliError::usage(format!("could not write candidate choices: {error}")))?;
    let mut human = format!("Calibration choice required: scope={scope} selector={selector}\n");
    for choice in projections {
        human.push_str(&format!(
            "  {}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            choice["id"].as_str().unwrap_or("invalid-id"),
            choice["source"].as_str().unwrap_or("unknown-source"),
            choice["identity"].as_str().unwrap_or("unknown-identity"),
            choice["fidelity"].as_str().unwrap_or("unknown-fidelity"),
            choice["classification"]
                .as_str()
                .unwrap_or("unknown-classification"),
            choice["display_name"].as_str().unwrap_or("unknown-name"),
            choice["executable_hint"]
                .as_str()
                .unwrap_or("no-executable-hint"),
            choice["install_root"].as_str().unwrap_or("no-install-root"),
        ));
    }
    human.push_str("Rerun the same command with --candidate '<CANDIDATE_ID>'.\n");
    emitter
        .required_human_checked(&human)
        .map_err(|error| CliError::usage(format!("could not write candidate choices: {error}")))
}

fn revalidate_candidate(
    expected: &CandidateTarget,
    discovery: &Discovery,
) -> Result<CandidateTarget, CliError> {
    let expected_id = fragcap::targets::calibration_candidate_id(expected);
    let matches = discovery
        .candidates
        .iter()
        .filter(|candidate| fragcap::targets::calibration_candidate_id(candidate) == expected_id)
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(CliError::usage(
            "the selected calibration candidate is absent from current discovery",
        )),
        _ => Err(CliError::usage(
            "current discovery contains duplicate selected candidate authority",
        )),
    }
}

fn registration_plan_value(
    candidate: &CandidateTarget,
    discovery: &Discovery,
    local_store: &Path,
) -> Result<Value, CliError> {
    let local_store = crate::commands::target_resolve::resolve_store_identity(local_store);
    let mut evidence: Vec<_> = candidate
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
        .collect();
    evidence.sort_by_key(|value| value.to_string());
    let mut warnings = discovery.warnings.clone();
    warnings.sort();
    let predicted_stable_id = match candidate.identity {
        CandidateIdentity::SteamAppId(appid) => Some(fragcap::targets::identifier::anchored_id(
            &format!("steam:{appid}"),
        )),
        CandidateIdentity::Path(_) => None,
        CandidateIdentity::LaunchEntry(_) => None,
    };
    Ok(json!({
        "candidate": {
            "classification": candidate.classification.as_str(),
            "detection_scan": candidate.detection_scan.map(|scan| scan.as_str()),
            "display_name": candidate.display_name,
            "evidence": evidence,
            "executable_hint": candidate.executable_hint,
            "fidelity": candidate.fidelity.as_str(),
            "folder_name": candidate.folder_name,
            "identity": candidate_identity_value(&candidate.identity),
            "install_root": candidate.install_root,
            "source": candidate.source_name,
        },
        "discovery": {
            "account": {
                "access_error": discovery.account.access_error,
                "considered": discovery.account.considered,
                "considered_not_a_game": discovery.account.considered_not_a_game,
                "container_descended": discovery.account.container_descended,
                "container_descent_truncated": discovery.account.container_descent_truncated,
                "declined_by_user": discovery.account.declined_by_user,
                "parse_failed": discovery.account.parse_failed,
                "produced": discovery.account.produced,
                "volume_skipped": discovery.account.volume_skipped,
            },
            "warnings": warnings,
        },
        "local_store": local_store.to_string_lossy(),
        "operation": REGISTRATION_OPERATION,
        "predicted_stable_id": predicted_stable_id,
        "schema": REGISTRATION_PLAN_SCHEMA,
    }))
}

fn steam_client_plan_value(
    target: &TargetEntry,
    candidate: &CandidateTarget,
    discovery: &Discovery,
    local_store: &Path,
    app_id: u32,
    executable: &str,
) -> Value {
    let local_store = crate::commands::target_resolve::resolve_store_identity(local_store);
    let mut evidence: Vec<_> = candidate
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
        .collect();
    evidence.sort_by_key(|value| value.to_string());
    let mut warnings = discovery.warnings.clone();
    warnings.sort();
    json!({
        "candidate": {
            "classification": candidate.classification.as_str(),
            "detection_scan": candidate.detection_scan.map(|scan| scan.as_str()),
            "display_name": candidate.display_name,
            "evidence": evidence,
            "executable_hint": candidate.executable_hint,
            "fidelity": candidate.fidelity.as_str(),
            "folder_name": candidate.folder_name,
            "identity": candidate_identity_value(&candidate.identity),
            "install_root": candidate.install_root,
            "source": candidate.source_name,
        },
        "discovery": {
            "account": {
                "access_error": discovery.account.access_error,
                "considered": discovery.account.considered,
                "considered_not_a_game": discovery.account.considered_not_a_game,
                "container_descended": discovery.account.container_descended,
                "container_descent_truncated": discovery.account.container_descent_truncated,
                "declined_by_user": discovery.account.declined_by_user,
                "parse_failed": discovery.account.parse_failed,
                "produced": discovery.account.produced,
                "volume_skipped": discovery.account.volume_skipped,
            },
            "warnings": warnings,
        },
        "local_store": local_store.to_string_lossy(),
        "no_effects": [
            "no-process-control",
            "no-process-launch",
            "no-proxy-or-routing",
            "no-session-authorization",
            "no-system-trust-change",
        ],
        "operation": STEAM_CLIENT_OPERATION,
        "proposed_executable": executable,
        "resulting_launch_entries": fragcap::targets::resolved_client_launch(executable),
        "schema": STEAM_CLIENT_PLAN_SCHEMA,
        "steam_app_id": app_id,
        "target": target_plan_value(target),
    })
}

fn target_plan_value(target: &TargetEntry) -> Value {
    json!({
        "anchor": target.anchor,
        "classification": target.classification.as_str(),
        "classification_source": target.classification_source.as_str(),
        "detection_scan": target.detection_scan.map(|scan| scan.as_str()),
        "evidence": target.evidence,
        "executable_hint": target.executable_hint,
        "fidelity": target.fidelity.as_str(),
        "folder_name": target.folder_name,
        "handle": target.handle,
        "id": target.id,
        "install_root": target.install_root,
        "launch_entries": target.launch_entries,
        "name": target.name,
        "provenance": target.provenance,
        "stable_id": target.stable_id,
    })
}

fn steam_app_id(target: &TargetEntry) -> Option<u32> {
    let anchor = target.anchor.as_deref()?;
    if fragcap::targets::identifier::canonicalize_anchor(anchor) != anchor {
        return None;
    }
    let digits = anchor.strip_prefix("steam:")?;
    let app_id = digits.parse::<u32>().ok()?;
    (app_id > 0 && digits == app_id.to_string()).then_some(app_id)
}

fn candidate_identity(identity: &CandidateIdentity) -> String {
    match identity {
        CandidateIdentity::SteamAppId(appid) => format!("steam:{appid}"),
        CandidateIdentity::Path(path) => path.clone(),
        CandidateIdentity::LaunchEntry(entry) => {
            fragcap::targets::launch_entry_value(entry).to_string()
        }
    }
}

fn candidate_identity_value(identity: &CandidateIdentity) -> Value {
    match identity {
        CandidateIdentity::SteamAppId(appid) => json!({"kind": "steam-app-id", "value": appid}),
        CandidateIdentity::Path(path) => json!({"kind": "path", "value": path}),
        CandidateIdentity::LaunchEntry(entry) => json!({
            "kind": "stored-launch-entry",
            "value": fragcap::targets::launch_entry_value(entry),
        }),
    }
}

fn confirm_registration(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
    plan: &RegistrationPlan,
) -> Result<RegistrationConfirmation, CliError> {
    if !args.authorize_stdin {
        emitter
            .required_human_checked(&format!("Register exact target plan {}? [y/N] ", plan.id))
            .map_err(|error| {
                CliError::usage(format!(
                    "could not write the target registration prompt: {error}"
                ))
            })?;
    }
    emitter.flush().map_err(|error| {
        CliError::usage(format!(
            "could not flush the target registration plan before input: {error}"
        ))
    })?;
    if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
        return Ok(RegistrationConfirmation::Interrupted);
    }
    let response = authorization
        .read_response(&plan.id, args.authorize_stdin)
        .map_err(|error| {
            CliError::usage(format!(
                "could not read target registration confirmation: {error}"
            ))
        })?;
    if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
        return Ok(RegistrationConfirmation::Interrupted);
    }
    if args.authorize_stdin {
        if response.is_empty() {
            return Ok(RegistrationConfirmation::Closed);
        }
        return Ok(if exact_plan_response(&response, &plan.id) {
            RegistrationConfirmation::Confirmed
        } else {
            RegistrationConfirmation::Invalid
        });
    }
    let Some(line) = response.strip_suffix(b"\n") else {
        return Ok(RegistrationConfirmation::Closed);
    };
    let Ok(answer) = std::str::from_utf8(line) else {
        return Ok(RegistrationConfirmation::Invalid);
    };
    Ok(
        if answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes") {
            RegistrationConfirmation::Confirmed
        } else {
            RegistrationConfirmation::Declined
        },
    )
}

fn exact_plan_response(response: &[u8], expected: &str) -> bool {
    let Some(candidate) = response.strip_suffix(b"\n") else {
        return false;
    };
    constant_time_equal(candidate, expected.as_bytes())
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && left.ct_eq(right).unwrap_u8() == 1
}

fn registered_candidate_target(
    store: &Store,
    candidate: &CandidateTarget,
) -> Result<TargetEntry, CliError> {
    let identity_target = match &candidate.identity {
        CandidateIdentity::SteamAppId(appid) => {
            let anchor =
                fragcap::targets::identifier::canonicalize_anchor(&format!("steam:{appid}"));
            store
                .target_by_anchor(&anchor)
                .map_err(|error| CliError::failure(error.to_string()))?
                .ok_or_else(|| {
                    CliError::failure(
                        "registration completed but its exact anchored target could not be resolved",
                    )
                })
        }
        CandidateIdentity::Path(_) => {
            let root = candidate.install_root.as_deref().ok_or_else(|| {
                CliError::failure("a path candidate registration has no install-root authority")
            })?;
            let matches: Vec<_> = store
                .targets()
                .map_err(|error| CliError::failure(error.to_string()))?
                .into_iter()
                .filter(|target| target.install_root.as_deref() == Some(root))
                .collect();
            match matches.as_slice() {
                [target] => Ok(target.clone()),
                [] => Err(CliError::failure(
                    "registration completed but its exact path target could not be resolved",
                )),
                _ => Err(CliError::failure(
                    "registration produced conflicting targets for one exact install root",
                )),
            }
        }
        CandidateIdentity::LaunchEntry(_) => Err(CliError::failure(
            "a stored launch-entry choice cannot enter target registration",
        )),
    }?;
    store
        .target_by_stable_id(identity_target.stable_id)
        .map_err(|error| CliError::failure(error.to_string()))?
        .ok_or_else(|| {
            CliError::failure(
                "registration completed but its durable target identity could not be resolved",
            )
        })
}

fn registration_outcome(
    emitter: &mut Emitter,
    plan_id: &str,
    status: &str,
    reason: &str,
    target_id: Option<i64>,
    continued: bool,
) -> Result<(), CliError> {
    emitter
        .event_checked(&Event::CalibrationRegistration {
            plan_id: plan_id.to_string(),
            status: status.to_string(),
            reason: reason.to_string(),
            target_id,
            continued,
        })
        .and_then(|()| {
            emitter.required_human_checked(&format!(
                "Target registration: {status} ({reason}); target id: {}; calibration continued: {continued}\n",
                target_id.map_or_else(|| "none".to_string(), |id| id.to_string()),
            ))
        })
        .map_err(|error| {
            CliError::usage(format!(
                "could not write the target registration outcome: {error}"
            ))
        })
}

fn prepare_steam_client(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
    local_store_path: &Path,
    store: &mut Store,
    target: TargetEntry,
    candidate_selection: &mut CandidateSelection,
) -> Result<TargetFrontDoor, CliError> {
    if target.launch_entries.is_some() || steam_app_id(&target).is_none() {
        return Ok(TargetFrontDoor::Ready(Box::new(target)));
    }

    let discovery = discover_for_calibration(args, store, emitter)?;
    let app_id = steam_app_id(&target).expect("the early return checked the Steam app id");
    let current_candidates = discovery
        .candidates
        .iter()
        .filter(|candidate| {
            candidate.identity == CandidateIdentity::SteamAppId(app_id)
                && candidate.source_name == "steam"
        })
        .cloned()
        .collect::<Vec<_>>();
    let selected_candidate = match current_candidates.as_slice() {
        [candidate] if !candidate_selection.has_unconsumed() => candidate.clone(),
        [_] => {
            return Err(CliError::usage(
                "the supplied calibration candidate has no current ambiguous Steam client choice",
            ))
        }
        [] => {
            return Err(CliError::usage(format!(
                "Steam client setup is unavailable: discovery returned no exact steam:{app_id} candidate; refresh Steam metadata and retry"
            )))
        }
        _ => choose_ambiguous_candidate(
            "steam-client",
            &format!("steam:{app_id}"),
            Some(target.stable_id),
            current_candidates,
            &discovery,
            candidate_selection,
            emitter,
        )?,
    };
    let selected_discovery = discovery_with_candidate(&discovery, selected_candidate);
    let Some(plan) = SteamClientPlan::new(target.clone(), &selected_discovery, local_store_path)?
    else {
        return Ok(TargetFrontDoor::Ready(Box::new(target)));
    };
    if emitter.is_json() && !args.authorize_stdin {
        return Err(CliError::usage(
            "JSON Steam client setup requires --authorize-stdin and the exact emitted plan identifier",
        ));
    }
    if !args.authorize_stdin && !authorization.is_terminal() {
        return Err(CliError::usage(
            "Steam client setup requires an interactive terminal or --authorize-stdin",
        ));
    }

    plan.emit(emitter)?;
    let confirmation = match confirm_steam_client(args, authorization, emitter, &plan) {
        Ok(confirmation) => confirmation,
        Err(error) => {
            let _ = steam_client_outcome(
                emitter,
                &plan,
                "failed",
                "steam-client-confirmation-failed",
                false,
            );
            return Err(error);
        }
    };
    match confirmation {
        RegistrationConfirmation::Confirmed => {}
        RegistrationConfirmation::Declined => {
            steam_client_outcome(
                emitter,
                &plan,
                "declined",
                "operator-declined-socket-holder-assertion",
                false,
            )?;
            return Ok(TargetFrontDoor::Declined);
        }
        RegistrationConfirmation::Closed => {
            steam_client_outcome(emitter, &plan, "closed", "steam-client-input-closed", false)?;
            return Ok(TargetFrontDoor::Declined);
        }
        RegistrationConfirmation::Invalid => {
            steam_client_outcome(
                emitter,
                &plan,
                "invalid",
                "exact-steam-client-plan-id-not-supplied",
                false,
            )?;
            return Err(CliError::usage(
                "Steam client confirmation did not match the exact current plan identifier; no target was changed",
            ));
        }
        RegistrationConfirmation::Interrupted => {
            steam_client_outcome(
                emitter,
                &plan,
                "interrupted",
                "interrupt-before-steam-client-update",
                false,
            )?;
            return Err(CliError::failure(
                "Steam client setup was interrupted; no target was changed",
            ));
        }
    }

    let current_target = match store.target_by_stable_id(plan.target.stable_id) {
        Ok(target) => target,
        Err(error) => {
            steam_client_outcome(
                emitter,
                &plan,
                "failed",
                "target-read-failed-after-confirmation",
                false,
            )?;
            return Err(CliError::failure(error.to_string()));
        }
    };
    let Some(current_target) = current_target else {
        steam_client_outcome(
            emitter,
            &plan,
            "drifted",
            "target-missing-after-confirmation",
            false,
        )?;
        return Err(CliError::usage(
            "the target changed after Steam client confirmation; review a fresh plan",
        ));
    };
    let current_discovery = match discover_for_calibration(args, store, emitter) {
        Ok(discovery) => discovery,
        Err(error) => {
            steam_client_outcome(
                emitter,
                &plan,
                "drifted",
                "discovery-unavailable-after-confirmation",
                false,
            )?;
            return Err(error);
        }
    };
    let current_candidate = match revalidate_candidate(&plan.candidate, &current_discovery) {
        Ok(candidate) => candidate,
        Err(error) => {
            steam_client_outcome(
                emitter,
                &plan,
                "drifted",
                "steam-client-candidate-not-reproduced",
                false,
            )?;
            return Err(error);
        }
    };
    let current_selected_discovery =
        discovery_with_candidate(&current_discovery, current_candidate);
    let current_plan = match SteamClientPlan::new(
        current_target,
        &current_selected_discovery,
        local_store_path,
    ) {
        Ok(plan) => plan,
        Err(error) => {
            steam_client_outcome(
                emitter,
                &plan,
                "drifted",
                "steam-client-authority-not-reproduced",
                false,
            )?;
            return Err(error);
        }
    };
    let Some(current_plan) = current_plan else {
        steam_client_outcome(
            emitter,
            &plan,
            "drifted",
            "steam-client-authority-not-reproduced",
            false,
        )?;
        return Err(CliError::usage(
            "the Steam client authority changed after confirmation; review a fresh plan",
        ));
    };
    if !constant_time_equal(
        plan.canonical_json.as_bytes(),
        current_plan.canonical_json.as_bytes(),
    ) {
        steam_client_outcome(
            emitter,
            &plan,
            "drifted",
            "steam-client-plan-changed-after-confirmation",
            false,
        )?;
        return Err(CliError::usage(
            "the Steam client authority changed after confirmation; review a fresh plan",
        ));
    }

    let persistence = steam_client_persistence_result(
        emitter,
        &plan,
        store.author_target_client_if_unchanged(&plan.target, &plan.executable),
    )?;
    match persistence {
        AuthorTargetClientOutcome::Applied => {}
        AuthorTargetClientOutcome::Changed => {
            steam_client_outcome(
                emitter,
                &plan,
                "changed",
                "target-changed-before-conditional-update",
                false,
            )?;
            return Err(CliError::usage(
                "the target changed before the Steam client update; review a fresh plan",
            ));
        }
        AuthorTargetClientOutcome::Missing => {
            steam_client_outcome(
                emitter,
                &plan,
                "changed",
                "target-missing-before-conditional-update",
                false,
            )?;
            return Err(CliError::usage(
                "the target disappeared before the Steam client update; review a fresh plan",
            ));
        }
    }

    let updated = authored_steam_client_result(
        emitter,
        &plan,
        store.target_by_stable_id(plan.target.stable_id),
    )?;
    let mut expected = plan.target.clone();
    expected.launch_entries = Some(fragcap::targets::resolved_client_launch(&plan.executable));
    expected.fidelity = fragcap::profile::FidelityTier::Authored;
    if updated != expected {
        steam_client_outcome(
            emitter,
            &plan,
            "failed",
            "authored-target-verification-failed",
            false,
        )?;
        return Err(CliError::failure(
            "the authored Steam target did not match the confirmed result",
        ));
    }
    steam_client_outcome(emitter, &plan, "applied", "authored-client-persisted", true)?;
    Ok(TargetFrontDoor::Ready(Box::new(updated)))
}

fn stored_client_candidates(target: &TargetEntry) -> Vec<CandidateTarget> {
    if target
        .anchor
        .as_deref()
        .is_some_and(|anchor| anchor.starts_with("steam:"))
    {
        return Vec::new();
    }
    let launches = fragcap::targets::entry_windows_launch_entries(target);
    if launches.len() < 2
        || launches
            .iter()
            .any(|entry| entry.role().is_some_and(|role| role != "client"))
    {
        return Vec::new();
    }
    launches
        .into_iter()
        .map(|entry| CandidateTarget {
            identity: CandidateIdentity::LaunchEntry(entry.clone()),
            display_name: target.name.clone(),
            fidelity: target.fidelity,
            classification: target.classification,
            evidence: Vec::new(),
            detection_scan: target.detection_scan,
            source_name: "stored-launch-declaration".to_string(),
            install_root: target.install_root.clone(),
            folder_name: target.folder_name.clone(),
            executable_hint: Some(entry.executable().to_string()),
        })
        .collect()
}

fn stored_client_discovery(target: &TargetEntry) -> Discovery {
    let candidates = stored_client_candidates(target);
    let mut account = DiscoveryAccount::default();
    for _ in &candidates {
        account.produce();
    }
    Discovery {
        candidates,
        account,
        warnings: Vec::new(),
    }
}

fn prepare_stored_client(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
    local_store_path: &Path,
    store: &mut Store,
    target: TargetEntry,
    candidate_selection: &mut CandidateSelection,
) -> Result<TargetFrontDoor, CliError> {
    let discovery = stored_client_discovery(&target);
    if discovery.candidates.is_empty() {
        return Ok(TargetFrontDoor::Ready(Box::new(target)));
    }
    let selected = choose_ambiguous_candidate(
        "stored-client",
        &target.handle,
        Some(target.stable_id),
        discovery.candidates.clone(),
        &discovery,
        candidate_selection,
        emitter,
    )?;
    let launch_entry = match &selected.identity {
        CandidateIdentity::LaunchEntry(entry) => entry.clone(),
        _ => {
            return Err(CliError::failure(
                "stored client choice lost its launch authority",
            ))
        }
    };
    let plan = StoredClientPlan::new(target.clone(), launch_entry, local_store_path);
    if emitter.is_json() && !args.authorize_stdin {
        return Err(CliError::usage(
            "JSON stored client selection requires --authorize-stdin and the exact emitted plan identifier",
        ));
    }
    if !args.authorize_stdin && !authorization.is_terminal() {
        return Err(CliError::usage(
            "stored client selection requires an interactive terminal or --authorize-stdin",
        ));
    }
    plan.emit(emitter)?;
    let confirmation = match confirm_stored_client(args, authorization, emitter, &plan) {
        Ok(confirmation) => confirmation,
        Err(error) => {
            let _ = stored_client_outcome(
                emitter,
                &plan,
                "failed",
                "stored-client-confirmation-failed",
                false,
            );
            return Err(error);
        }
    };
    match confirmation {
        RegistrationConfirmation::Confirmed => {}
        RegistrationConfirmation::Declined => {
            stored_client_outcome(
                emitter,
                &plan,
                "declined",
                "operator-declined-stored-client-assertion",
                false,
            )?;
            return Ok(TargetFrontDoor::Declined);
        }
        RegistrationConfirmation::Closed => {
            stored_client_outcome(
                emitter,
                &plan,
                "closed",
                "stored-client-input-closed",
                false,
            )?;
            return Ok(TargetFrontDoor::Declined);
        }
        RegistrationConfirmation::Invalid => {
            stored_client_outcome(
                emitter,
                &plan,
                "invalid",
                "exact-stored-client-plan-id-not-supplied",
                false,
            )?;
            return Err(CliError::usage(
                "stored client confirmation did not match the exact current plan identifier; no target was changed",
            ));
        }
        RegistrationConfirmation::Interrupted => {
            stored_client_outcome(
                emitter,
                &plan,
                "interrupted",
                "interrupt-before-stored-client-update",
                false,
            )?;
            return Err(CliError::failure(
                "stored client selection was interrupted; no target was changed",
            ));
        }
    }

    let current = match store.target_by_stable_id(plan.target.stable_id) {
        Ok(Some(target)) => target,
        Ok(None) => {
            stored_client_outcome(
                emitter,
                &plan,
                "drifted",
                "target-missing-after-confirmation",
                false,
            )?;
            return Err(CliError::usage(
                "the target disappeared after stored client confirmation; review a fresh plan",
            ));
        }
        Err(error) => {
            stored_client_outcome(
                emitter,
                &plan,
                "failed",
                "target-read-failed-after-confirmation",
                false,
            )?;
            return Err(CliError::failure(error.to_string()));
        }
    };
    let requested = fragcap::targets::calibration_candidate_id(&selected);
    let current_candidates = stored_client_candidates(&current)
        .into_iter()
        .filter(|candidate| fragcap::targets::calibration_candidate_id(candidate) == requested)
        .collect::<Vec<_>>();
    let [current_candidate] = current_candidates.as_slice() else {
        stored_client_outcome(
            emitter,
            &plan,
            "drifted",
            "stored-client-candidate-not-reproduced",
            false,
        )?;
        return Err(CliError::usage(
            "the stored client choices changed after confirmation; review a fresh plan",
        ));
    };
    let current_launch_entry = match &current_candidate.identity {
        CandidateIdentity::LaunchEntry(entry) => entry.clone(),
        _ => {
            return Err(CliError::failure(
                "reproduced stored client choice lost its launch authority",
            ));
        }
    };
    let current_plan = StoredClientPlan::new(current, current_launch_entry, local_store_path);
    if !constant_time_equal(
        plan.canonical_json.as_bytes(),
        current_plan.canonical_json.as_bytes(),
    ) {
        stored_client_outcome(
            emitter,
            &plan,
            "drifted",
            "stored-client-plan-changed-after-confirmation",
            false,
        )?;
        return Err(CliError::usage(
            "the stored client authority changed after confirmation; review a fresh plan",
        ));
    }
    let persistence = stored_client_persistence_result(
        emitter,
        &plan,
        store.select_target_client_if_unchanged(&plan.target, &plan.launch_entry),
    )?;
    match persistence {
        AuthorTargetClientOutcome::Applied => {}
        AuthorTargetClientOutcome::Changed => {
            stored_client_outcome(
                emitter,
                &plan,
                "changed",
                "target-changed-before-conditional-update",
                false,
            )?;
            return Err(CliError::usage(
                "the target changed before stored client selection; review a fresh plan",
            ));
        }
        AuthorTargetClientOutcome::Missing => {
            stored_client_outcome(
                emitter,
                &plan,
                "changed",
                "target-missing-before-conditional-update",
                false,
            )?;
            return Err(CliError::usage(
                "the target disappeared before stored client selection; review a fresh plan",
            ));
        }
    }
    let updated = selected_stored_client_result(
        emitter,
        &plan,
        store.target_by_stable_id(plan.target.stable_id),
    )?;
    let mut expected = plan.target.clone();
    expected.launch_entries = Some(Value::Array(vec![fragcap::targets::launch_entry_value(
        &plan.launch_entry,
    )]));
    expected.fidelity = fragcap::profile::FidelityTier::Authored;
    if updated != expected {
        stored_client_outcome(
            emitter,
            &plan,
            "failed",
            "selected-target-verification-failed",
            false,
        )?;
        return Err(CliError::failure(
            "the selected stored client did not match the confirmed result",
        ));
    }
    stored_client_outcome(emitter, &plan, "applied", "stored-client-persisted", true)?;
    Ok(TargetFrontDoor::Ready(Box::new(updated)))
}

fn confirm_stored_client(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
    plan: &StoredClientPlan,
) -> Result<RegistrationConfirmation, CliError> {
    if !args.authorize_stdin {
        emitter
            .required_human_checked(&format!(
                "Does {} hold the target's network sockets? [y/N] ",
                plan.launch_entry.executable()
            ))
            .map_err(|error| {
                CliError::usage(format!("could not write the stored client prompt: {error}"))
            })?;
    }
    emitter.flush().map_err(|error| {
        CliError::usage(format!(
            "could not flush the stored client plan before input: {error}"
        ))
    })?;
    if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
        return Ok(RegistrationConfirmation::Interrupted);
    }
    let response = authorization
        .read_response(&plan.id, args.authorize_stdin)
        .map_err(|error| {
            CliError::usage(format!(
                "could not read stored client confirmation: {error}"
            ))
        })?;
    if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
        return Ok(RegistrationConfirmation::Interrupted);
    }
    if args.authorize_stdin {
        if response.is_empty() {
            return Ok(RegistrationConfirmation::Closed);
        }
        return Ok(if exact_plan_response(&response, &plan.id) {
            RegistrationConfirmation::Confirmed
        } else {
            RegistrationConfirmation::Invalid
        });
    }
    let Some(line) = response.strip_suffix(b"\n") else {
        return Ok(RegistrationConfirmation::Closed);
    };
    let Ok(answer) = std::str::from_utf8(line) else {
        return Ok(RegistrationConfirmation::Invalid);
    };
    Ok(
        if answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes") {
            RegistrationConfirmation::Confirmed
        } else {
            RegistrationConfirmation::Declined
        },
    )
}

fn stored_client_outcome(
    emitter: &mut Emitter,
    plan: &StoredClientPlan,
    status: &str,
    reason: &str,
    continued: bool,
) -> Result<(), CliError> {
    emitter
        .event_checked(&Event::CalibrationStoredClient {
            plan_id: plan.id.clone(),
            status: status.to_string(),
            reason: reason.to_string(),
            target_id: plan.target.stable_id,
            continued,
        })
        .and_then(|()| {
            emitter.required_human_checked(&format!(
                "Stored client selection: {status} ({reason}); target id: {}; calibration continued: {continued}\n",
                plan.target.stable_id,
            ))
        })
        .map_err(|error| {
            CliError::usage(format!(
                "could not write the stored client outcome: {error}"
            ))
        })
}

fn stored_client_persistence_result<E: std::fmt::Display>(
    emitter: &mut Emitter,
    plan: &StoredClientPlan,
    result: Result<AuthorTargetClientOutcome, E>,
) -> Result<AuthorTargetClientOutcome, CliError> {
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            stored_client_outcome(
                emitter,
                plan,
                "failed",
                "stored-client-persistence-failed",
                false,
            )?;
            Err(CliError::failure(error.to_string()))
        }
    }
}

fn selected_stored_client_result<E: std::fmt::Display>(
    emitter: &mut Emitter,
    plan: &StoredClientPlan,
    result: Result<Option<TargetEntry>, E>,
) -> Result<TargetEntry, CliError> {
    match result {
        Ok(Some(target)) => Ok(target),
        Ok(None) => {
            stored_client_outcome(
                emitter,
                plan,
                "failed",
                "selected-target-missing-after-update",
                false,
            )?;
            Err(CliError::failure(
                "the selected stored target could not be re-resolved",
            ))
        }
        Err(error) => {
            stored_client_outcome(
                emitter,
                plan,
                "failed",
                "selected-target-read-failed",
                false,
            )?;
            Err(CliError::failure(error.to_string()))
        }
    }
}

fn discovery_with_candidate(discovery: &Discovery, candidate: CandidateTarget) -> Discovery {
    Discovery {
        candidates: vec![candidate],
        account: discovery.account.clone(),
        warnings: discovery.warnings.clone(),
    }
}

fn confirm_steam_client(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
    plan: &SteamClientPlan,
) -> Result<RegistrationConfirmation, CliError> {
    if !args.authorize_stdin {
        emitter
            .required_human_checked(&format!(
                "Does {} hold the target's network sockets? [y/N] ",
                plan.executable
            ))
            .map_err(|error| {
                CliError::usage(format!("could not write the Steam client prompt: {error}"))
            })?;
    }
    emitter.flush().map_err(|error| {
        CliError::usage(format!(
            "could not flush the Steam client setup plan before input: {error}"
        ))
    })?;
    if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
        return Ok(RegistrationConfirmation::Interrupted);
    }
    let response = authorization
        .read_response(&plan.id, args.authorize_stdin)
        .map_err(|error| {
            CliError::usage(format!("could not read Steam client confirmation: {error}"))
        })?;
    if crate::orchestrator::INTERRUPT.load(Ordering::Relaxed) {
        return Ok(RegistrationConfirmation::Interrupted);
    }
    if args.authorize_stdin {
        if response.is_empty() {
            return Ok(RegistrationConfirmation::Closed);
        }
        return Ok(if exact_plan_response(&response, &plan.id) {
            RegistrationConfirmation::Confirmed
        } else {
            RegistrationConfirmation::Invalid
        });
    }
    let Some(line) = response.strip_suffix(b"\n") else {
        return Ok(RegistrationConfirmation::Closed);
    };
    let Ok(answer) = std::str::from_utf8(line) else {
        return Ok(RegistrationConfirmation::Invalid);
    };
    Ok(
        if answer.trim().eq_ignore_ascii_case("y") || answer.trim().eq_ignore_ascii_case("yes") {
            RegistrationConfirmation::Confirmed
        } else {
            RegistrationConfirmation::Declined
        },
    )
}

fn steam_client_outcome(
    emitter: &mut Emitter,
    plan: &SteamClientPlan,
    status: &str,
    reason: &str,
    continued: bool,
) -> Result<(), CliError> {
    emitter
        .event_checked(&Event::CalibrationSteamClient {
            plan_id: plan.id.clone(),
            status: status.to_string(),
            reason: reason.to_string(),
            target_id: plan.target.stable_id,
            continued,
        })
        .and_then(|()| {
            emitter.required_human_checked(&format!(
                "Steam client setup: {status} ({reason}); target id: {}; calibration continued: {continued}\n",
                plan.target.stable_id,
            ))
        })
        .map_err(|error| {
            CliError::usage(format!("could not write the Steam client outcome: {error}"))
        })
}

fn steam_client_persistence_result<E: std::fmt::Display>(
    emitter: &mut Emitter,
    plan: &SteamClientPlan,
    result: Result<AuthorTargetClientOutcome, E>,
) -> Result<AuthorTargetClientOutcome, CliError> {
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            steam_client_outcome(
                emitter,
                plan,
                "failed",
                "steam-client-persistence-failed",
                false,
            )?;
            Err(CliError::failure(error.to_string()))
        }
    }
}

fn authored_steam_client_result<E: std::fmt::Display>(
    emitter: &mut Emitter,
    plan: &SteamClientPlan,
    result: Result<Option<TargetEntry>, E>,
) -> Result<TargetEntry, CliError> {
    match result {
        Ok(Some(target)) => Ok(target),
        Ok(None) => {
            steam_client_outcome(
                emitter,
                plan,
                "failed",
                "authored-target-missing-after-update",
                false,
            )?;
            Err(CliError::failure(
                "the authored Steam target could not be re-resolved",
            ))
        }
        Err(error) => {
            steam_client_outcome(
                emitter,
                plan,
                "failed",
                "authored-target-read-failed",
                false,
            )?;
            Err(CliError::failure(error.to_string()))
        }
    }
}

pub fn run(
    args: &CalibrateArgs,
    authorization: &mut dyn DeepCaptureAuthorizationInput,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    if args.resume.is_some_and(|workflow_id| workflow_id <= 0) {
        let error = CliError::usage("calibration workflow identifier must be positive");
        return Err(if emitter.is_json() {
            error
        } else {
            let command_context =
                crate::workflow_help::CalibrationCommandContext::for_calibrate(args, None);
            crate::workflow_help::actionable_error(
                error,
                crate::workflow_help::target_reference(
                    args.selector.as_deref(),
                    args.target.as_deref(),
                    args.id,
                ),
                Some(crate::workflow_help::FirstRunRefusal::Calibration),
                crate::workflow_help::WorkflowVerb::Calibrate,
                Some(&command_context),
            )
        });
    }
    let mut candidate_selection =
        CandidateSelection::new(args.candidate.as_deref()).map_err(|error| {
            if emitter.is_json() {
                error
            } else {
                let command_context =
                    crate::workflow_help::CalibrationCommandContext::for_calibrate(args, None);
                crate::workflow_help::actionable_error(
                    error,
                    crate::workflow_help::target_reference(
                        args.selector.as_deref(),
                        args.target.as_deref(),
                        args.id,
                    ),
                    Some(crate::workflow_help::FirstRunRefusal::Calibration),
                    crate::workflow_help::WorkflowVerb::Calibrate,
                    Some(&command_context),
                )
            }
        })?;
    let mut requested_protocols = normalize_protocol_args(&args.protocol);
    let local_store_path = deep_capture::local_store_path(args.local_db.as_deref())?;
    let mut store = Store::open(&local_store_path)
        .map_err(|error| CliError::failure(format!("cannot open local store: {error}")))?;
    let local_store_argument = quote_powershell_path(&local_store_path)?;
    let (target, mut workflow) = if let Some(workflow_id) = args.resume {
        let mut workflow = store
            .calibration_workflow(workflow_id)
            .map_err(|error| {
                CliError::failure(format!("cannot read calibration workflow: {error}"))
            })?
            .ok_or_else(|| {
                CliError::usage(format!(
                    "calibration workflow {workflow_id} does not exist in the selected local store"
                ))
            })?;
        let target = store
            .target_by_stable_id(workflow.target.stable_id)
            .map_err(|error| CliError::failure(format!("cannot resolve workflow target: {error}")))?
            .ok_or_else(|| {
                CliError::usage(format!(
                    "calibration workflow {workflow_id} no longer has a target"
                ))
            })?;
        if !workflow.target.matches(&target) {
            let checkpoint =
                checkpoint_from_workflow(&workflow, CalibrationWorkflowState::Refused, None);
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
            return Err(CliError::usage(format!(
                "calibration workflow {workflow_id} target authority changed; start a fresh workflow"
            )));
        }
        if workflow.state == CalibrationWorkflowState::Refused {
            return Err(CliError::usage(format!(
                "calibration workflow {workflow_id} was refused and cannot start another session"
            )));
        }
        requested_protocols = workflow.requested_protocols.clone();
        if workflow.state == CalibrationWorkflowState::InFlight {
            let checkpoint = checkpoint_from_workflow(
                &workflow,
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Interrupted),
            );
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
                workflow_status_guidance(&workflow, "interrupted", "prior-process-interrupted"),
            );
            return Ok(Exit::SUCCESS);
        }
        if workflow.state == CalibrationWorkflowState::Completed && args.pause_for.is_some() {
            return Err(CliError::usage(
                "a completed calibration workflow cannot be paused",
            ));
        }
        if let Some(pause) = args.pause_for {
            let checkpoint = checkpoint_from_workflow(
                &workflow,
                CalibrationWorkflowState::Paused,
                Some(pause_reason(pause)),
            );
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
                workflow_status_guidance(&workflow, "paused", "operator-selected-pause"),
            );
            return Ok(Exit::SUCCESS);
        }
        if workflow.state == CalibrationWorkflowState::Paused {
            let checkpoint =
                checkpoint_from_workflow(&workflow, CalibrationWorkflowState::Ready, None);
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
        }
        (target, workflow)
    } else {
        let front_door = resolve_or_register_target(
            args,
            authorization,
            emitter,
            &local_store_path,
            &mut store,
            &mut candidate_selection,
        )
        .map_err(|error| {
            if emitter.is_json() {
                error
            } else {
                let command_context =
                    crate::workflow_help::CalibrationCommandContext::for_calibrate(
                        args,
                        Some(local_store_path.clone()),
                    );
                crate::workflow_help::actionable_error(
                    error,
                    crate::workflow_help::target_reference(
                        args.selector.as_deref(),
                        args.target.as_deref(),
                        args.id,
                    ),
                    None,
                    crate::workflow_help::WorkflowVerb::Calibrate,
                    Some(&command_context),
                )
            }
        })?;
        let target = match front_door {
            TargetFrontDoor::Ready(target) => *target,
            TargetFrontDoor::Declined => return Ok(Exit::SUCCESS),
        };
        let target = match prepare_steam_client(
            args,
            authorization,
            emitter,
            &local_store_path,
            &mut store,
            target,
            &mut candidate_selection,
        )? {
            TargetFrontDoor::Ready(target) => *target,
            TargetFrontDoor::Declined => return Ok(Exit::SUCCESS),
        };
        let target = match prepare_stored_client(
            args,
            authorization,
            emitter,
            &local_store_path,
            &mut store,
            target,
            &mut candidate_selection,
        )? {
            TargetFrontDoor::Ready(target) => *target,
            TargetFrontDoor::Declined => return Ok(Exit::SUCCESS),
        };
        candidate_selection.require_consumed()?;
        let workflow = store
            .create_calibration_workflow_with_intent(
                &target,
                &requested_protocols,
                args.launch_case.map(compatibility_launch_case_arg),
                args.routing_strategy
                    .map(compatibility_routing_arg)
                    .unwrap_or(CompatibilityRoutingStrategy::ChildEnvironment),
                args.proxy_family
                    .map(compatibility_family_arg)
                    .unwrap_or(CompatibilityAddressFamily::Ipv4),
                workflow_now(),
            )
            .map_err(|error| {
                CliError::failure(format!("cannot create calibration workflow: {error}"))
            })?;
        (target, workflow)
    };
    if args.controlled_target {
        deep_capture::require_controlled_target(&target)?;
    }
    let sequence_target_authority = SequenceTargetAuthority::from_target(&target);
    let snapshot = process_snapshot(args.controlled_target);
    let proposal = build_proposal_for_workflow(
        &store,
        &target,
        snapshot.clone(),
        &requested_protocols,
        &workflow,
    )?;
    if let Some(expected) = workflow.selected_launch_case {
        if let Some(current) = proposal_cold_launch_case(&proposal) {
            if current != expected {
                let checkpoint =
                    checkpoint_from_workflow(&workflow, CalibrationWorkflowState::Refused, None);
                apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
                emit_workflow_guidance(
                    emitter,
                    &target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&proposal),
                        action: "refused",
                        status: "refused",
                        observed_launch_case: None,
                        selected_launch_case: Some(current.as_str().to_string()),
                        reason: Some("launch-case-assertion-mismatch".to_string()),
                        images: readiness_images(&proposal),
                        limitations: limitation_messages(&proposal),
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&workflow.observed_protocols),
                        completed_protocols: protocol_names(&workflow.completed_protocols),
                        remaining_protocols: protocol_names(&workflow.remaining_protocols),
                        next_command: None,
                    },
                );
                return Err(CliError::usage(format!(
                    "guided calibration launch-case assertion {} does not match current inferred case {}",
                    expected.as_str(),
                    current.as_str()
                )));
            }
        }
    }
    if workflow.state == CalibrationWorkflowState::Completed {
        let all_protocols = merge_protocols(&requested_protocols, &workflow.observed_protocols);
        let (completed_protocols, remaining_protocols) =
            coverage_from_proposal(&proposal, &all_protocols);
        emit_workflow_guidance(
            emitter,
            &target,
            &workflow,
            &local_store_argument,
            Guidance {
                topology: topology(&proposal),
                action: "ready",
                status: if remaining_protocols.is_empty() {
                    "workflow-complete"
                } else {
                    "workflow-complete-current-gap"
                },
                observed_launch_case: None,
                selected_launch_case: ready_launch_case(&proposal)
                    .ok()
                    .map(|value| value.as_str().to_string()),
                reason: Some("completed-workflow-status-check".to_string()),
                images: readiness_images(&proposal),
                limitations: limitation_messages(&proposal),
                requested_protocols: protocol_names(&requested_protocols),
                observed_protocols: protocol_names(&workflow.observed_protocols),
                completed_protocols: protocol_names(&completed_protocols),
                remaining_protocols: protocol_names(&remaining_protocols),
                next_command: None,
            },
        );
        return Ok(Exit::SUCCESS);
    }
    if !proposal.limitations.is_empty() {
        let limitations = limitation_messages(&proposal);
        let checkpoint =
            checkpoint_from_workflow(&workflow, CalibrationWorkflowState::Refused, None);
        apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
        emit_workflow_guidance(
            emitter,
            &target,
            &workflow,
            &local_store_argument,
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
                observed_protocols: protocol_names(&workflow.observed_protocols),
                completed_protocols: protocol_names(&workflow.completed_protocols),
                remaining_protocols: protocol_names(&workflow.remaining_protocols),
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
            let next_command = format!(
                "{} --restart-warm",
                calibration_resume_command(workflow.id, &local_store_argument)
            );
            let checkpoint = checkpoint_from_workflow(
                &workflow,
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Shutdown),
            );
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
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
                    observed_protocols: protocol_names(&workflow.observed_protocols),
                    completed_protocols: protocol_names(&workflow.completed_protocols),
                    remaining_protocols: protocol_names(&workflow.remaining_protocols),
                    next_command: Some(next_command),
                },
            );
            if !args.restart_warm {
                return Ok(Exit::SUCCESS);
            }
            let mut restart_args = low_level_args(
                args,
                target.stable_id,
                None,
                deep_capture_api::CalibrationPhase::Reachability,
                CompatibilityProtocol::Routing,
                workflow.address_family,
            )?;
            restart_args.restart_warm = true;
            deep_capture::prepare_warm_restart(&restart_args, &store, emitter)?;
            let checkpoint =
                checkpoint_from_workflow(&workflow, CalibrationWorkflowState::Ready, None);
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
        }
        deep_capture_api::CalibrationLaunchReadiness::Ready { .. } => {}
        _ => {
            let checkpoint =
                checkpoint_from_workflow(&workflow, CalibrationWorkflowState::Refused, None);
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
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
                    observed_protocols: protocol_names(&workflow.observed_protocols),
                    completed_protocols: protocol_names(&workflow.completed_protocols),
                    remaining_protocols: protocol_names(&workflow.remaining_protocols),
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
        target.stable_id,
        None,
        deep_capture_api::CalibrationPhase::Reachability,
        CompatibilityProtocol::Routing,
        workflow.address_family,
    )?;
    let mut observed_protocols = workflow.observed_protocols.clone();
    let mut attempted = workflow
        .attempted_case_keys
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let (mut last_completed_protocols, mut last_remaining_protocols) =
        coverage_from_proposal(&proposal, &requested_protocols);

    loop {
        if guided_sequence_interrupted(&crate::orchestrator::INTERRUPT) {
            let checkpoint = checkpoint_from_workflow(
                &workflow,
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Interrupted),
            );
            apply_workflow_checkpoint(&mut store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
                workflow_status_guidance(
                    &workflow,
                    "interrupted",
                    "sequence-interrupted-before-attempt",
                ),
            );
            return Err(CliError::usage(
                "guided calibration was interrupted before the next attempt; no later effects were applied",
            ));
        }
        let mut fresh_store = deep_capture::open_local_store(args.local_db.as_deref())?;
        let all_protocols = merge_protocols(&requested_protocols, &observed_protocols);
        let fresh_target = match deep_capture::resolve_target(&fresh_store, &resolver_args) {
            Ok(target) => target,
            Err(error) => {
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &last_completed_protocols,
                    &last_remaining_protocols,
                    CalibrationWorkflowState::Paused,
                    Some(CalibrationPauseReason::Failure),
                    None,
                );
                apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
                emit_workflow_guidance(
                    emitter,
                    &target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: None,
                        action: "refused",
                        status: "refused",
                        observed_launch_case: None,
                        selected_launch_case: None,
                        reason: Some("target-authority-unavailable".to_string()),
                        images: Vec::new(),
                        limitations: vec![error.message().to_string()],
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&last_completed_protocols),
                        remaining_protocols: protocol_names(&last_remaining_protocols),
                        next_command: None,
                    },
                );
                return Err(error);
            }
        };
        if !sequence_target_authority.matches(&fresh_target) {
            let checkpoint = progress_checkpoint(
                &workflow,
                &requested_protocols,
                &observed_protocols,
                &[],
                &all_protocols,
                CalibrationWorkflowState::Refused,
                None,
                None,
            );
            apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
                Guidance {
                    topology: None,
                    action: "refused",
                    status: "refused",
                    observed_launch_case: None,
                    selected_launch_case: None,
                    reason: Some("target-authority-drift".to_string()),
                    images: Vec::new(),
                    limitations: Vec::new(),
                    requested_protocols: protocol_names(&requested_protocols),
                    observed_protocols: protocol_names(&observed_protocols),
                    completed_protocols: Vec::new(),
                    remaining_protocols: protocol_names(&all_protocols),
                    next_command: None,
                },
            );
            return Err(CliError::usage(
                "the target authority changed during guided calibration; review a fresh sequence",
            ));
        }
        let selected = build_proposal(
            &fresh_store,
            &fresh_target,
            process_snapshot(args.controlled_target),
            &all_protocols,
            workflow.routing_strategy,
            workflow.address_family,
        )?;
        let (completed_protocols, remaining_protocols) =
            coverage_from_proposal(&selected, &all_protocols);

        if !selected.limitations.is_empty() {
            let limitations = limitation_messages(&selected);
            let checkpoint = progress_checkpoint(
                &workflow,
                &requested_protocols,
                &observed_protocols,
                &completed_protocols,
                &remaining_protocols,
                CalibrationWorkflowState::Refused,
                None,
                None,
            );
            apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &fresh_target,
                &workflow,
                &local_store_argument,
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
                    observed_protocols: protocol_names(&observed_protocols),
                    completed_protocols: protocol_names(&completed_protocols),
                    remaining_protocols: protocol_names(&remaining_protocols),
                    next_command: None,
                },
            );
            return Err(CliError::usage(format!(
                "guided calibration cannot select an attempt: {}",
                limitations.join("; ")
            )));
        }

        match &selected.readiness {
            deep_capture_api::CalibrationLaunchReadiness::OperatorAction {
                observed_case,
                cold_case,
                images,
            } => {
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &completed_protocols,
                    &remaining_protocols,
                    CalibrationWorkflowState::Paused,
                    Some(CalibrationPauseReason::Shutdown),
                    None,
                );
                apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
                emit_workflow_guidance(
                    emitter,
                    &fresh_target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&selected),
                        action: "operator-action",
                        status: "warm",
                        observed_launch_case: Some(observed_case.as_str().to_string()),
                        selected_launch_case: Some(cold_case.as_str().to_string()),
                        reason: Some("declared-process-image-present".to_string()),
                        images: images.clone(),
                        limitations: Vec::new(),
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&completed_protocols),
                        remaining_protocols: protocol_names(&remaining_protocols),
                        next_command: Some(format!(
                            "{} --restart-warm",
                            calibration_resume_command(workflow.id, &local_store_argument)
                        )),
                    },
                );
                return Ok(Exit::SUCCESS);
            }
            deep_capture_api::CalibrationLaunchReadiness::Ready { .. } => {}
            _ => {
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &completed_protocols,
                    &remaining_protocols,
                    CalibrationWorkflowState::Refused,
                    None,
                    None,
                );
                apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
                emit_workflow_guidance(
                    emitter,
                    &fresh_target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&selected),
                        action: "refused",
                        status: "refused",
                        observed_launch_case: None,
                        selected_launch_case: None,
                        reason: Some("launch-readiness-unavailable".to_string()),
                        images: readiness_images(&selected),
                        limitations: Vec::new(),
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&completed_protocols),
                        remaining_protocols: protocol_names(&remaining_protocols),
                        next_command: None,
                    },
                );
                return Err(CliError::usage(
                    "guided calibration could not determine an exact launch case",
                ));
            }
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
            let (status, reason) = if all_protocols.is_empty() {
                ("ready", "current-routing-evidence")
            } else if requested_protocols.is_empty() {
                ("observed-coverage-complete", "current-protocol-evidence")
            } else {
                ("requested-coverage-complete", "current-protocol-evidence")
            };
            let checkpoint = progress_checkpoint(
                &workflow,
                &requested_protocols,
                &observed_protocols,
                &completed_protocols,
                &remaining_protocols,
                CalibrationWorkflowState::Completed,
                None,
                None,
            );
            apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &fresh_target,
                &workflow,
                &local_store_argument,
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
                    observed_protocols: protocol_names(&observed_protocols),
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

        let step = selected.steps[0].clone();
        if !valid_guided_step(&step) {
            return Err(CliError::usage(
                "guided calibration selected an unsupported phase or protocol; no session was started",
            ));
        }
        let action = match step.phase {
            deep_capture_api::CalibrationPhase::Reachability => "run-reachability",
            deep_capture_api::CalibrationPhase::Tls => "run-protocol",
        };
        let next_command = Some(calibration_resume_command(
            workflow.id,
            &local_store_argument,
        ));
        let key = ExactAttemptCase::from_step(&step).durable_key();
        let attempted_count = workflow.attempt_ordinal as usize;
        let inserted = attempted_count < MAX_GUIDED_ATTEMPTS && attempted.insert(key.clone());
        let no_progress_reason = no_progress_reason(attempted_count, inserted);
        if let Some(no_progress_reason) = no_progress_reason {
            // The retained exact-case key can be the checkpoint of an interrupted
            // effectful attempt. Recovery authority must win over the no-repeat
            // guard so an outstanding cleanup or trust obligation is never hidden.
            deep_capture::require_prior_recovery_settled()?;
            let checkpoint = progress_checkpoint(
                &workflow,
                &requested_protocols,
                &observed_protocols,
                &completed_protocols,
                &remaining_protocols,
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Failure),
                None,
            );
            apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
            emit_workflow_guidance(
                emitter,
                &fresh_target,
                &workflow,
                &local_store_argument,
                Guidance {
                    topology: topology(&selected),
                    action,
                    status: "no-progress",
                    observed_launch_case: None,
                    selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                    reason: Some(no_progress_reason.to_string()),
                    images: readiness_images(&selected),
                    limitations: Vec::new(),
                    requested_protocols: protocol_names(&requested_protocols),
                    observed_protocols: protocol_names(&observed_protocols),
                    completed_protocols: protocol_names(&completed_protocols),
                    remaining_protocols: protocol_names(&remaining_protocols),
                    next_command,
                },
            );
            return Ok(Exit::SUCCESS);
        }
        let attempt_number = workflow.attempt_ordinal as usize + 1;
        let progress = AttemptProgress {
            number: attempt_number as u64,
            phase: step.phase,
            protocol: step.case.protocol,
        };
        let mut low_level = low_level_args(
            args,
            fresh_target.stable_id,
            Some(launch_case_arg(step.case.launch_case)),
            step.phase,
            step.case.protocol,
            workflow.address_family,
        )?;
        low_level.bundle = attempt_bundle(
            args.bundle.as_deref(),
            attempt_number,
            step.phase,
            step.case.protocol,
        )?;
        if let Some(bundle) = low_level.bundle.as_deref() {
            if let Err(error) = deep_capture::validate_bundle_root(bundle) {
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &completed_protocols,
                    &remaining_protocols,
                    CalibrationWorkflowState::Paused,
                    Some(CalibrationPauseReason::Failure),
                    None,
                );
                apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
                emit_attempt_workflow_guidance(
                    emitter,
                    &fresh_target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&selected),
                        action,
                        status: "refused",
                        observed_launch_case: None,
                        selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                        reason: Some("bundle-destination-refused".to_string()),
                        images: readiness_images(&selected),
                        limitations: vec![error.message().to_string()],
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&completed_protocols),
                        remaining_protocols: protocol_names(&remaining_protocols),
                        next_command,
                    },
                    progress,
                );
                return Err(error);
            }
        }
        let checkpoint = progress_checkpoint(
            &workflow,
            &requested_protocols,
            &observed_protocols,
            &completed_protocols,
            &remaining_protocols,
            CalibrationWorkflowState::InFlight,
            None,
            Some((progress.number, step.phase, step.case.protocol, key)),
        );
        apply_workflow_checkpoint(&mut fresh_store, &mut workflow, checkpoint)?;
        emit_attempt_workflow_guidance(
            emitter,
            &fresh_target,
            &workflow,
            &local_store_argument,
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
                observed_protocols: protocol_names(&observed_protocols),
                completed_protocols: protocol_names(&completed_protocols),
                remaining_protocols: protocol_names(&remaining_protocols),
                next_command: None,
            },
            progress,
        );
        drop(fresh_store);
        let outcome = match deep_capture::run_with_outcome(&low_level, authorization, emitter) {
            Ok(outcome) => outcome,
            Err(error) => {
                let refused = error.exit() == Exit::USAGE;
                let mut terminal_store = deep_capture::open_local_store(args.local_db.as_deref())?;
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &completed_protocols,
                    &remaining_protocols,
                    CalibrationWorkflowState::Paused,
                    Some(if refused {
                        CalibrationPauseReason::Authorization
                    } else {
                        CalibrationPauseReason::Failure
                    }),
                    None,
                );
                let checkpoint = if refused {
                    retry_current_case(&workflow, checkpoint)
                } else {
                    checkpoint
                };
                apply_workflow_checkpoint(&mut terminal_store, &mut workflow, checkpoint)?;
                emit_attempt_workflow_guidance(
                    emitter,
                    &fresh_target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&selected),
                        action,
                        status: if refused { "refused" } else { "failed" },
                        observed_launch_case: None,
                        selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                        reason: Some(
                            if refused {
                                "delegated-session-refused"
                            } else {
                                "delegated-session-error"
                            }
                            .to_string(),
                        ),
                        images: readiness_images(&selected),
                        limitations: Vec::new(),
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&completed_protocols),
                        remaining_protocols: protocol_names(&remaining_protocols),
                        next_command: None,
                    },
                    progress,
                );
                return Err(error);
            }
        };
        let newly_observed = deep_capture_api::observed_protocol_candidates(
            &outcome.observations,
            args.controlled_target,
        );
        observed_protocols = merge_protocols(&observed_protocols, &newly_observed);
        let all_protocols = merge_protocols(&requested_protocols, &observed_protocols);
        let mut completed_store = deep_capture::open_local_store(args.local_db.as_deref())?;
        let completed_target = match deep_capture::resolve_target(&completed_store, &low_level) {
            Ok(target) => target,
            Err(error) => {
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &completed_protocols,
                    &remaining_protocols,
                    CalibrationWorkflowState::Paused,
                    Some(CalibrationPauseReason::Failure),
                    None,
                );
                apply_workflow_checkpoint(&mut completed_store, &mut workflow, checkpoint)?;
                emit_attempt_workflow_guidance(
                    emitter,
                    &fresh_target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&selected),
                        action,
                        status: "failed",
                        observed_launch_case: None,
                        selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                        reason: Some("post-session-target-unavailable".to_string()),
                        images: readiness_images(&selected),
                        limitations: vec![error.message().to_string()],
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&completed_protocols),
                        remaining_protocols: protocol_names(&remaining_protocols),
                        next_command: None,
                    },
                    progress,
                );
                return Err(error);
            }
        };
        if !sequence_target_authority.matches(&completed_target) {
            let checkpoint = progress_checkpoint(
                &workflow,
                &requested_protocols,
                &observed_protocols,
                &completed_protocols,
                &remaining_protocols,
                CalibrationWorkflowState::Refused,
                None,
                None,
            );
            apply_workflow_checkpoint(&mut completed_store, &mut workflow, checkpoint)?;
            emit_attempt_workflow_guidance(
                emitter,
                &target,
                &workflow,
                &local_store_argument,
                Guidance {
                    topology: topology(&selected),
                    action,
                    status: "refused",
                    observed_launch_case: None,
                    selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                    reason: Some("target-authority-drift".to_string()),
                    images: readiness_images(&selected),
                    limitations: Vec::new(),
                    requested_protocols: protocol_names(&requested_protocols),
                    observed_protocols: protocol_names(&observed_protocols),
                    completed_protocols: protocol_names(&completed_protocols),
                    remaining_protocols: protocol_names(&remaining_protocols),
                    next_command: None,
                },
                progress,
            );
            return Err(CliError::usage(
                "the target authority changed during guided calibration; review a fresh sequence",
            ));
        }
        let completed = match build_proposal(
            &completed_store,
            &completed_target,
            process_snapshot(args.controlled_target),
            &all_protocols,
            workflow.routing_strategy,
            workflow.address_family,
        ) {
            Ok(proposal) => proposal,
            Err(error) => {
                let checkpoint = progress_checkpoint(
                    &workflow,
                    &requested_protocols,
                    &observed_protocols,
                    &completed_protocols,
                    &remaining_protocols,
                    CalibrationWorkflowState::Paused,
                    Some(CalibrationPauseReason::Failure),
                    None,
                );
                apply_workflow_checkpoint(&mut completed_store, &mut workflow, checkpoint)?;
                emit_attempt_workflow_guidance(
                    emitter,
                    &completed_target,
                    &workflow,
                    &local_store_argument,
                    Guidance {
                        topology: topology(&selected),
                        action,
                        status: "failed",
                        observed_launch_case: None,
                        selected_launch_case: Some(step.case.launch_case.as_str().to_string()),
                        reason: Some("post-session-proposal-unavailable".to_string()),
                        images: readiness_images(&selected),
                        limitations: vec![error.message().to_string()],
                        requested_protocols: protocol_names(&requested_protocols),
                        observed_protocols: protocol_names(&observed_protocols),
                        completed_protocols: protocol_names(&completed_protocols),
                        remaining_protocols: protocol_names(&remaining_protocols),
                        next_command: None,
                    },
                    progress,
                );
                return Err(error);
            }
        };
        let (completed_protocols, remaining_protocols) =
            coverage_from_proposal(&completed, &all_protocols);
        last_completed_protocols = completed_protocols.clone();
        last_remaining_protocols = remaining_protocols.clone();
        let (completed_status, completed_reason) = match outcome.disposition {
            deep_capture::RunDisposition::Declined => ("declined", "operator-declined"),
            deep_capture::RunDisposition::Interrupted => {
                ("interrupted", "delegated-session-interrupted")
            }
            deep_capture::RunDisposition::Failed => {
                ("failed", "delegated-session-terminal-failure")
            }
            deep_capture::RunDisposition::Completed => completion_outcome_for_step(
                &completed,
                step.phase,
                step.case.launch_case,
                step.case.protocol,
                &completed_protocols,
            ),
        };
        let next_command = if outcome.disposition == deep_capture::RunDisposition::Failed {
            None
        } else if remaining_protocols.is_empty() && completed.steps.is_empty() {
            Some(target_command(
                "deep-capture",
                completed_target.stable_id,
                &local_store_argument,
                " --launch",
            ))
        } else {
            Some(calibration_resume_command(
                workflow.id,
                &local_store_argument,
            ))
        };
        let (workflow_state, pause_reason) =
            terminal_workflow_state(outcome.disposition, completed_status);
        let checkpoint = progress_checkpoint(
            &workflow,
            &requested_protocols,
            &observed_protocols,
            &completed_protocols,
            &remaining_protocols,
            workflow_state,
            pause_reason,
            None,
        );
        let checkpoint = if outcome.disposition == deep_capture::RunDisposition::Declined {
            retry_current_case(&workflow, checkpoint)
        } else {
            checkpoint
        };
        apply_workflow_checkpoint(&mut completed_store, &mut workflow, checkpoint)?;
        emit_attempt_workflow_guidance(
            emitter,
            &completed_target,
            &workflow,
            &local_store_argument,
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
            progress,
        );
        if let Some(error) = outcome.terminal_error {
            return Err(error);
        }
        match outcome.disposition {
            deep_capture::RunDisposition::Completed if completed_status == "completed" => {}
            deep_capture::RunDisposition::Failed => {
                return Err(CliError::failure(
                    "guided calibration session failed without a terminal error",
                ));
            }
            _ => return Ok(Exit::SUCCESS),
        }
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
    Ok(crate::workflow_help::quote_powershell_argument(value))
}

fn target_command(verb: &str, target_id: i64, local_store: &str, suffix: &str) -> String {
    format!("fragcap {verb} --id {target_id} --local-db {local_store}{suffix}")
}

fn calibration_resume_command(workflow_id: i64, local_store: &str) -> String {
    format!("fragcap calibrate --resume {workflow_id} --local-db {local_store}")
}

fn workflow_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn pause_reason(value: GuidedCalibrationPauseArg) -> CalibrationPauseReason {
    match value {
        GuidedCalibrationPauseArg::Login => CalibrationPauseReason::Login,
        GuidedCalibrationPauseArg::Eula => CalibrationPauseReason::Eula,
        GuidedCalibrationPauseArg::Update => CalibrationPauseReason::Update,
        GuidedCalibrationPauseArg::AntiCheat => CalibrationPauseReason::AntiCheat,
        GuidedCalibrationPauseArg::Gameplay => CalibrationPauseReason::Gameplay,
        GuidedCalibrationPauseArg::Shutdown => CalibrationPauseReason::Shutdown,
        GuidedCalibrationPauseArg::Interrupted => CalibrationPauseReason::Interrupted,
    }
}

fn workflow_phase(value: deep_capture_api::CalibrationPhase) -> CalibrationWorkflowPhase {
    match value {
        deep_capture_api::CalibrationPhase::Reachability => CalibrationWorkflowPhase::Reachability,
        deep_capture_api::CalibrationPhase::Tls => CalibrationWorkflowPhase::Tls,
    }
}

fn checkpoint_from_workflow(
    workflow: &CalibrationWorkflow,
    state: CalibrationWorkflowState,
    pause_reason: Option<CalibrationPauseReason>,
) -> CalibrationWorkflowCheckpoint {
    CalibrationWorkflowCheckpoint {
        requested_protocols: workflow.requested_protocols.clone(),
        observed_protocols: workflow.observed_protocols.clone(),
        completed_protocols: workflow.completed_protocols.clone(),
        remaining_protocols: workflow.remaining_protocols.clone(),
        attempted_case_keys: workflow.attempted_case_keys.clone(),
        attempt_ordinal: workflow.attempt_ordinal,
        attempt_phase: None,
        attempt_protocol: None,
        attempt_key: None,
        state,
        pause_reason,
    }
}

#[allow(clippy::too_many_arguments)]
fn progress_checkpoint(
    workflow: &CalibrationWorkflow,
    requested_protocols: &[CompatibilityProtocol],
    observed_protocols: &[CompatibilityProtocol],
    completed_protocols: &[CompatibilityProtocol],
    remaining_protocols: &[CompatibilityProtocol],
    state: CalibrationWorkflowState,
    pause_reason: Option<CalibrationPauseReason>,
    attempt: Option<(
        u64,
        deep_capture_api::CalibrationPhase,
        CompatibilityProtocol,
        String,
    )>,
) -> CalibrationWorkflowCheckpoint {
    let mut attempted_case_keys = workflow.attempted_case_keys.clone();
    if let Some((_, _, _, key)) = &attempt {
        attempted_case_keys.push(key.clone());
        attempted_case_keys.sort();
        attempted_case_keys.dedup();
    }
    CalibrationWorkflowCheckpoint {
        requested_protocols: requested_protocols.to_vec(),
        observed_protocols: observed_protocols.to_vec(),
        completed_protocols: completed_protocols.to_vec(),
        remaining_protocols: remaining_protocols.to_vec(),
        attempted_case_keys,
        attempt_ordinal: attempt
            .as_ref()
            .map(|(ordinal, _, _, _)| *ordinal)
            .unwrap_or(workflow.attempt_ordinal),
        attempt_phase: attempt
            .as_ref()
            .map(|(_, phase, _, _)| workflow_phase(*phase)),
        attempt_protocol: attempt.as_ref().map(|(_, _, protocol, _)| *protocol),
        attempt_key: attempt.map(|(_, _, _, key)| key),
        state,
        pause_reason,
    }
}

fn apply_workflow_checkpoint(
    store: &mut Store,
    workflow: &mut CalibrationWorkflow,
    checkpoint: CalibrationWorkflowCheckpoint,
) -> Result<(), CliError> {
    match store
        .update_calibration_workflow(workflow.id, workflow.revision, &checkpoint, workflow_now())
        .map_err(|error| {
            CliError::failure(format!("cannot update calibration workflow: {error}"))
        })? {
        CalibrationWorkflowUpdateOutcome::Applied(updated) => {
            *workflow = *updated;
            Ok(())
        }
        CalibrationWorkflowUpdateOutcome::Changed => Err(CliError::usage(format!(
            "calibration workflow {} changed concurrently; review its current state",
            workflow.id
        ))),
        CalibrationWorkflowUpdateOutcome::Missing => Err(CliError::usage(format!(
            "calibration workflow {} was removed before its checkpoint could be updated",
            workflow.id
        ))),
    }
}

fn retry_current_case(
    workflow: &CalibrationWorkflow,
    mut checkpoint: CalibrationWorkflowCheckpoint,
) -> CalibrationWorkflowCheckpoint {
    if let Some(attempt_key) = &workflow.attempt_key {
        checkpoint
            .attempted_case_keys
            .retain(|value| value != attempt_key);
    }
    checkpoint
}

fn terminal_workflow_state(
    disposition: deep_capture::RunDisposition,
    completed_status: &str,
) -> (CalibrationWorkflowState, Option<CalibrationPauseReason>) {
    match disposition {
        deep_capture::RunDisposition::Declined => (
            CalibrationWorkflowState::Paused,
            Some(CalibrationPauseReason::Authorization),
        ),
        deep_capture::RunDisposition::Interrupted => (
            CalibrationWorkflowState::Paused,
            Some(CalibrationPauseReason::Interrupted),
        ),
        deep_capture::RunDisposition::Failed => (
            CalibrationWorkflowState::Paused,
            Some(CalibrationPauseReason::Failure),
        ),
        deep_capture::RunDisposition::Completed if completed_status == "completed" => {
            (CalibrationWorkflowState::Ready, None)
        }
        deep_capture::RunDisposition::Completed => (
            CalibrationWorkflowState::Paused,
            Some(CalibrationPauseReason::Gameplay),
        ),
    }
}

fn workflow_status_guidance(
    workflow: &CalibrationWorkflow,
    status: &'static str,
    reason: &'static str,
) -> Guidance {
    Guidance {
        topology: None,
        action: "operator-action",
        status,
        observed_launch_case: None,
        selected_launch_case: None,
        reason: Some(reason.to_string()),
        images: Vec::new(),
        limitations: Vec::new(),
        requested_protocols: protocol_names(&workflow.requested_protocols),
        observed_protocols: protocol_names(&workflow.observed_protocols),
        completed_protocols: protocol_names(&workflow.completed_protocols),
        remaining_protocols: protocol_names(&workflow.remaining_protocols),
        next_command: None,
    }
}

#[cfg(test)]
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

#[cfg(test)]
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
    routing_strategy: CompatibilityRoutingStrategy,
    address_family: CompatibilityAddressFamily,
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
    .with_routing_strategy(routing_strategy)
    .with_address_family(address_family)
    .with_protocol_candidates(protocols.iter().copied())
    .with_facts(facts);
    Ok(deep_capture_api::propose_calibration(&request))
}

fn build_proposal_for_workflow(
    store: &Store,
    target: &TargetEntry,
    snapshot: deep_capture_api::CalibrationProcessSnapshot,
    protocols: &[CompatibilityProtocol],
    workflow: &CalibrationWorkflow,
) -> Result<deep_capture_api::CalibrationProposal, CliError> {
    build_proposal(
        store,
        target,
        snapshot,
        protocols,
        workflow.routing_strategy,
        workflow.address_family,
    )
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
    target_stable_id: i64,
    launch_case: Option<DeepCaptureLaunchCaseArg>,
    phase: deep_capture_api::CalibrationPhase,
    protocol: CompatibilityProtocol,
    address_family: CompatibilityAddressFamily,
) -> Result<DeepCaptureArgs, CliError> {
    let calibration = match phase {
        deep_capture_api::CalibrationPhase::Reachability => DeepCaptureCalibrationArg::Reachability,
        deep_capture_api::CalibrationPhase::Tls => DeepCaptureCalibrationArg::Tls,
    };
    let calibration_protocol = protocol_arg(protocol)?;
    Ok(DeepCaptureArgs {
        selector: None,
        target: None,
        id: Some(target_stable_id),
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
        proxy_family: proxy_family_arg(address_family),
        proxy_bypass: Vec::new(),
        controlled_target: args.controlled_target,
    })
}

fn attempt_bundle(
    base: Option<&Path>,
    attempt: usize,
    phase: deep_capture_api::CalibrationPhase,
    protocol: CompatibilityProtocol,
) -> Result<Option<PathBuf>, CliError> {
    let Some(base) = base else {
        return Ok(None);
    };
    if attempt == 1 {
        return Ok(Some(base.to_path_buf()));
    }
    let file_name = base.file_name().ok_or_else(|| {
        CliError::usage(format!(
            "the guided calibration bundle path {} cannot identify a sibling destination",
            base.display()
        ))
    })?;
    let mut sibling = file_name.to_os_string();
    sibling.push(format!(
        "-attempt-{attempt:02}-{}-{}",
        phase.as_str(),
        protocol.as_str()
    ));
    Ok(Some(
        base.parent().unwrap_or_else(|| Path::new("")).join(sibling),
    ))
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

fn compatibility_launch_case_arg(value: DeepCaptureLaunchCaseArg) -> CompatibilityLaunchCase {
    match value {
        DeepCaptureLaunchCaseArg::SteamProtocolWarm => CompatibilityLaunchCase::SteamProtocolWarm,
        DeepCaptureLaunchCaseArg::SteamProtocolCold => CompatibilityLaunchCase::SteamProtocolCold,
        DeepCaptureLaunchCaseArg::DirectExeWarm => CompatibilityLaunchCase::DirectExeWarm,
        DeepCaptureLaunchCaseArg::DirectExeCold => CompatibilityLaunchCase::DirectExeCold,
        DeepCaptureLaunchCaseArg::PublisherLauncher => CompatibilityLaunchCase::PublisherLauncher,
        DeepCaptureLaunchCaseArg::PublisherLauncherWarm => {
            CompatibilityLaunchCase::PublisherLauncherWarm
        }
        DeepCaptureLaunchCaseArg::PublisherLauncherGameStartCleanWarm => {
            CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm
        }
        DeepCaptureLaunchCaseArg::PublisherLauncherCold => {
            CompatibilityLaunchCase::PublisherLauncherCold
        }
    }
}

fn compatibility_routing_arg(value: GuidedCalibrationRoutingArg) -> CompatibilityRoutingStrategy {
    match value {
        GuidedCalibrationRoutingArg::ChildEnvironment => {
            CompatibilityRoutingStrategy::ChildEnvironment
        }
        GuidedCalibrationRoutingArg::CommandArguments => {
            CompatibilityRoutingStrategy::CommandArguments
        }
        GuidedCalibrationRoutingArg::TargetConfiguration => {
            CompatibilityRoutingStrategy::TargetConfiguration
        }
        GuidedCalibrationRoutingArg::HttpProxy => CompatibilityRoutingStrategy::HttpProxy,
        GuidedCalibrationRoutingArg::Socks => CompatibilityRoutingStrategy::Socks,
        GuidedCalibrationRoutingArg::ProtocolSpecific => {
            CompatibilityRoutingStrategy::ProtocolSpecific
        }
    }
}

fn compatibility_family_arg(value: DeepCaptureProxyFamilyArg) -> CompatibilityAddressFamily {
    match value {
        DeepCaptureProxyFamilyArg::Ipv4 => CompatibilityAddressFamily::Ipv4,
        DeepCaptureProxyFamilyArg::Ipv6 => CompatibilityAddressFamily::Ipv6,
    }
}

fn proxy_family_arg(value: CompatibilityAddressFamily) -> DeepCaptureProxyFamilyArg {
    match value {
        CompatibilityAddressFamily::Ipv4 => DeepCaptureProxyFamilyArg::Ipv4,
        CompatibilityAddressFamily::Ipv6 => DeepCaptureProxyFamilyArg::Ipv6,
    }
}

fn proposal_cold_launch_case(
    proposal: &deep_capture_api::CalibrationProposal,
) -> Option<CompatibilityLaunchCase> {
    match proposal.readiness {
        deep_capture_api::CalibrationLaunchReadiness::Ready { launch_case, .. } => {
            Some(launch_case)
        }
        deep_capture_api::CalibrationLaunchReadiness::OperatorAction { cold_case, .. } => {
            Some(cold_case)
        }
        _ => None,
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

fn emit_workflow_guidance(
    emitter: &mut Emitter,
    target: &TargetEntry,
    workflow: &CalibrationWorkflow,
    local_store: &str,
    guidance: Guidance,
) {
    emit_guidance_with_attempt(
        emitter,
        target,
        guidance,
        None,
        Some((workflow, local_store)),
    );
}

fn emit_attempt_workflow_guidance(
    emitter: &mut Emitter,
    target: &TargetEntry,
    workflow: &CalibrationWorkflow,
    local_store: &str,
    guidance: Guidance,
    attempt: AttemptProgress,
) {
    emit_guidance_with_attempt(
        emitter,
        target,
        guidance,
        Some(attempt),
        Some((workflow, local_store)),
    );
}

fn emit_guidance_with_attempt(
    emitter: &mut Emitter,
    target: &TargetEntry,
    guidance: Guidance,
    attempt: Option<AttemptProgress>,
    workflow: Option<(&CalibrationWorkflow, &str)>,
) {
    let resume_command = workflow
        .map(|(workflow, local_store)| calibration_resume_command(workflow.id, local_store));
    emitter.event(&Event::CalibrationGuidance {
        target_id: target.stable_id,
        target: target.handle.clone(),
        topology: guidance.topology.clone(),
        action: guidance.action.to_string(),
        status: guidance.status.to_string(),
        observed_launch_case: guidance.observed_launch_case.clone(),
        selected_launch_case: guidance.selected_launch_case.clone(),
        launch_case_assertion: Box::new(
            workflow
                .and_then(|(value, _)| value.selected_launch_case)
                .map(|value| value.as_str().to_string()),
        ),
        routing_strategy: workflow
            .map(|(value, _)| value.routing_strategy.as_str().to_string())
            .unwrap_or_else(|| "unavailable".to_string()),
        address_family: workflow
            .map(|(value, _)| value.address_family.as_str().to_string())
            .unwrap_or_else(|| "unavailable".to_string()),
        reason: guidance.reason.clone(),
        images: guidance.images.clone(),
        limitations: guidance.limitations.clone(),
        requested_protocols: guidance.requested_protocols.clone(),
        observed_protocols: guidance.observed_protocols.clone(),
        completed_protocols: guidance.completed_protocols.clone(),
        remaining_protocols: guidance.remaining_protocols.clone(),
        process_control: "none".to_string(),
        next_command: Box::new(guidance.next_command.clone()),
        attempt: attempt.map(|value| value.number),
        maximum_attempts: attempt.map(|_| MAX_GUIDED_ATTEMPTS as u64),
        phase: attempt.map(|value| value.phase.as_str().to_string()),
        protocol: attempt.map(|value| value.protocol.as_str().to_string()),
        workflow_id: workflow.map(|(value, _)| value.id),
        workflow_revision: workflow.map(|(value, _)| value.revision),
        workflow_state: Box::new(workflow.map(|(value, _)| value.state.as_str().to_string())),
        pause_reason: Box::new(
            workflow
                .and_then(|(value, _)| value.pause_reason)
                .map(|value| value.as_str().to_string()),
        ),
        resume_command: Box::new(resume_command.clone()),
    });
    emitter.progress(&format!(
        "Calibration guidance: target={} id={} topology={} action={} status={} observed_launch_case={} selected_launch_case={} launch_case_assertion={} routing_strategy={} address_family={} reason={} images={} limitations={} requested_protocols={} observed_protocols={} completed_protocols={} remaining_protocols={} attempt={} maximum_attempts={} phase={} protocol={} workflow_id={} workflow_revision={} workflow_state={} pause_reason={} process_control=none next_command={} resume_command={}",
        target.handle,
        target.stable_id,
        guidance.topology.as_deref().unwrap_or("unavailable"),
        guidance.action,
        guidance.status,
        guidance.observed_launch_case.as_deref().unwrap_or("none"),
        guidance.selected_launch_case.as_deref().unwrap_or("none"),
        workflow.and_then(|(value, _)| value.selected_launch_case).map(|value| value.as_str()).unwrap_or("none"),
        workflow.map(|(value, _)| value.routing_strategy.as_str()).unwrap_or("unavailable"),
        workflow.map(|(value, _)| value.address_family.as_str()).unwrap_or("unavailable"),
        guidance.reason.as_deref().unwrap_or("none"),
        if guidance.images.is_empty() { "none".to_string() } else { guidance.images.join(",") },
        if guidance.limitations.is_empty() { "none".to_string() } else { guidance.limitations.join("; ") },
        if guidance.requested_protocols.is_empty() { "none".to_string() } else { guidance.requested_protocols.join(",") },
        if guidance.observed_protocols.is_empty() { "none".to_string() } else { guidance.observed_protocols.join(",") },
        if guidance.completed_protocols.is_empty() { "none".to_string() } else { guidance.completed_protocols.join(",") },
        if guidance.remaining_protocols.is_empty() { "none".to_string() } else { guidance.remaining_protocols.join(",") },
        attempt.map(|value| value.number.to_string()).unwrap_or_else(|| "none".to_string()),
        attempt.map(|_| MAX_GUIDED_ATTEMPTS.to_string()).unwrap_or_else(|| "none".to_string()),
        attempt.map(|value| value.phase.as_str()).unwrap_or("none"),
        attempt.map(|value| value.protocol.as_str()).unwrap_or("none"),
        workflow.map(|(value, _)| value.id.to_string()).unwrap_or_else(|| "none".to_string()),
        workflow.map(|(value, _)| value.revision.to_string()).unwrap_or_else(|| "none".to_string()),
        workflow.map(|(value, _)| value.state.as_str()).unwrap_or("none"),
        workflow.and_then(|(value, _)| value.pause_reason).map(|value| value.as_str()).unwrap_or("none"),
        guidance.next_command.as_deref().unwrap_or("none"),
        resume_command.as_deref().unwrap_or("none"),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap::profile::FidelityTier;
    use fragcap::targets::{
        resolved_client_launch, ClassificationSource, DiscoveryAccount, TargetClassification,
    };

    fn candidate(identity: CandidateIdentity, name: &str) -> CandidateTarget {
        CandidateTarget {
            identity,
            display_name: name.to_string(),
            fidelity: FidelityTier::Observed,
            classification: TargetClassification::Game,
            evidence: Vec::new(),
            detection_scan: None,
            source_name: "steam".to_string(),
            install_root: Some(format!("C:\\Games\\{name}")),
            folder_name: Some(name.to_string()),
            executable_hint: Some("client.exe".to_string()),
        }
    }

    fn discovery(candidates: Vec<CandidateTarget>) -> Discovery {
        Discovery {
            account: DiscoveryAccount {
                considered: candidates.len() as u64,
                produced: candidates.len() as u64,
                ..DiscoveryAccount::default()
            },
            candidates,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn stored_row_resolution_remains_authoritative_before_discovery() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("local.db")).unwrap();
        let entry = TargetEntry {
            id: None,
            stable_id: 75_000,
            handle: "stored-target".to_string(),
            name: "Stored Target".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(resolved_client_launch("stored.exe")),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };
        store.insert_target(&entry).unwrap();
        store
            .write_listing_snapshot(&[(entry.stable_id, &entry.handle)])
            .unwrap();
        let selection = deep_capture::select_target_input(&store, Some("1"), None, None).unwrap();
        let Selection::Resolved(target) = selection else {
            panic!("the stored listing row must resolve before discovery")
        };
        assert_eq!(target.stable_id, entry.stable_id);
    }

    #[test]
    fn exact_candidate_selection_distinguishes_identity_name_ambiguity_and_miss() {
        let portal = candidate(CandidateIdentity::SteamAppId(620), "Portal 2");
        let other = candidate(CandidateIdentity::SteamAppId(400), "Portal");
        assert_eq!(
            select_discovery_candidate("620", &discovery(vec![portal.clone(), other.clone()]))
                .unwrap(),
            portal
        );
        assert_eq!(
            select_discovery_candidate("portal", &discovery(vec![portal.clone(), other.clone()]))
                .unwrap(),
            other
        );
        let unicode = candidate(CandidateIdentity::SteamAppId(999), "Élan");
        assert_eq!(
            select_discovery_candidate("élan", &discovery(vec![unicode.clone()])).unwrap(),
            unicode
        );
        assert!(
            select_discovery_candidate("port", &discovery(vec![portal.clone()]))
                .unwrap_err()
                .message()
                .contains("exactly matches")
        );
        let duplicate = candidate(
            CandidateIdentity::Path("D:\\Games\\Portal".to_string()),
            "Portal 2",
        );
        let error = select_discovery_candidate("portal 2", &discovery(vec![portal, duplicate]))
            .unwrap_err();
        assert!(error.message().contains("2 exact candidates"));
        assert!(error.message().contains("no target was selected"));
    }

    #[test]
    fn registration_plan_is_deterministic_domain_separated_and_field_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("local.db");
        let first = candidate(CandidateIdentity::SteamAppId(620), "Portal 2");
        let observed = discovery(vec![first.clone()]);
        let same = RegistrationPlan::new(first.clone(), &observed, &store).unwrap();
        assert_eq!(
            same.id,
            RegistrationPlan::new(first.clone(), &observed, &store)
                .unwrap()
                .id
        );
        assert!(same.id.starts_with(REGISTRATION_PLAN_PREFIX));
        assert_eq!(same.canonical["discovery"]["account"]["considered"], 1);
        assert_ne!(
            same.id,
            RegistrationPlan::new(
                candidate(CandidateIdentity::SteamAppId(620), "Portal Two"),
                &observed,
                &store,
            )
            .unwrap()
            .id
        );
        assert_ne!(
            same.id,
            RegistrationPlan::new(first.clone(), &observed, &dir.path().join("other.db"))
                .unwrap()
                .id
        );
        let incomplete = Discovery {
            account: DiscoveryAccount {
                considered: 2,
                produced: 1,
                access_error: 1,
                ..DiscoveryAccount::default()
            },
            candidates: vec![first],
            warnings: vec!["one root was inaccessible".to_string()],
        };
        assert_ne!(
            same.id,
            RegistrationPlan::new(incomplete.candidates[0].clone(), &incomplete, &store,)
                .unwrap()
                .id
        );
    }

    #[test]
    fn exact_registration_input_requires_the_identifier_and_one_lf() {
        let id = "target-registration-v1:abc";
        assert!(exact_plan_response(format!("{id}\n").as_bytes(), id));
        assert!(!exact_plan_response(id.as_bytes(), id));
        assert!(!exact_plan_response(format!("{id}\r\n").as_bytes(), id));
        assert!(!exact_plan_response(b"target-registration-v1:def\n", id));
    }

    fn steam_target(launch_entries: Option<Value>) -> TargetEntry {
        TargetEntry {
            id: Some(7),
            stable_id: fragcap::targets::identifier::anchored_id("steam:620"),
            handle: "portal_2".to_string(),
            name: "Portal 2".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::Platform,
            fidelity: FidelityTier::Observed,
            provenance: Some(json!({"source": "steam"})),
            anchor: Some("steam:620".to_string()),
            launch_entries,
            install_root: Some(r"C:\Games\Portal 2".to_string()),
            evidence: None,
            detection_scan: None,
            folder_name: Some("Portal 2".to_string()),
            executable_hint: Some("portal2.exe".to_string()),
        }
    }

    #[test]
    fn steam_client_plan_is_domain_separated_complete_and_field_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        let target = steam_target(None);
        let candidate = candidate(CandidateIdentity::SteamAppId(620), "Portal 2");
        let observed = discovery(vec![candidate]);
        let plan = SteamClientPlan::new(target.clone(), &observed, dir.path())
            .unwrap()
            .unwrap();
        assert!(plan.id.starts_with(STEAM_CLIENT_PLAN_PREFIX));
        assert_eq!(plan.canonical["target"]["stable_id"], target.stable_id);
        assert_eq!(plan.canonical["steam_app_id"], 620);
        assert_eq!(plan.canonical["proposed_executable"], "client.exe");
        assert_eq!(plan.canonical["no_effects"].as_array().unwrap().len(), 5);
        let mut changed_target = target;
        changed_target.name = "Portal Two".to_string();
        assert_ne!(
            plan.id,
            SteamClientPlan::new(changed_target, &observed, dir.path())
                .unwrap()
                .unwrap()
                .id
        );
    }

    #[test]
    fn steam_client_plan_requires_absence_exact_identity_root_and_safe_image() {
        let dir = tempfile::tempdir().unwrap();
        let valid = candidate(CandidateIdentity::SteamAppId(620), "Portal 2");
        assert!(SteamClientPlan::new(
            steam_target(Some(resolved_client_launch("portal2.exe"))),
            &discovery(vec![valid.clone()]),
            dir.path(),
        )
        .unwrap()
        .is_none());
        let ambiguity = SteamClientPlan::new(
            steam_target(None),
            &discovery(vec![
                valid.clone(),
                candidate(CandidateIdentity::SteamAppId(620), "Portal 2"),
            ]),
            dir.path(),
        )
        .unwrap_err();
        assert!(ambiguity.message().contains("2 exact steam:620 candidates"));
        let mut wrong_root = valid.clone();
        wrong_root.install_root = Some(r"D:\Games\Portal 2".to_string());
        let wrong_root =
            SteamClientPlan::new(steam_target(None), &discovery(vec![wrong_root]), dir.path())
                .unwrap_err();
        assert!(wrong_root.message().contains("install root does not match"));
        for invalid in [
            "",
            " client.exe",
            "../client.exe",
            "client.exe --flag",
            "game.exe --helper.exe",
            "game --helper.exe",
            "bin/client name.exe",
            "https://client.exe",
            "%command%",
            "client.dll",
        ] {
            assert!(
                !fragcap::targets::is_client_executable(invalid),
                "accepted {invalid:?}"
            );
        }
        assert!(fragcap::targets::is_client_executable("bin/client.EXE"));
    }

    #[test]
    fn steam_client_persistence_failure_emits_one_terminal_outcome() {
        let dir = tempfile::tempdir().unwrap();
        let target = steam_target(None);
        let observed = discovery(vec![candidate(
            CandidateIdentity::SteamAppId(620),
            "Portal 2",
        )]);
        let plan = SteamClientPlan::new(target, &observed, dir.path())
            .unwrap()
            .unwrap();
        let mut output = Vec::new();
        let mut emitter = crate::emit::Emitter::new(
            &mut output,
            crate::emit::Format::Json,
            crate::emit::Verbosity::Normal,
        );
        let error = steam_client_persistence_result(
            &mut emitter,
            &plan,
            Err::<AuthorTargetClientOutcome, _>("database is locked"),
        )
        .unwrap_err();
        assert_eq!(error.message(), "database is locked");
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output.matches("calibration.steam_client").count(), 1);
        assert!(output.contains("\"status\":\"failed\""));
        assert!(output.contains("steam-client-persistence-failed"));
        assert!(output.contains("\"continued\":false"));
    }

    #[test]
    fn authored_steam_client_read_failure_and_absence_are_terminal() {
        let dir = tempfile::tempdir().unwrap();
        let observed = discovery(vec![candidate(
            CandidateIdentity::SteamAppId(620),
            "Portal 2",
        )]);
        let plan = SteamClientPlan::new(steam_target(None), &observed, dir.path())
            .unwrap()
            .unwrap();
        for (result, reason) in [
            (
                Err::<Option<TargetEntry>, _>("database is locked"),
                "authored-target-read-failed",
            ),
            (Ok(None), "authored-target-missing-after-update"),
        ] {
            let mut output = Vec::new();
            {
                let mut emitter = crate::emit::Emitter::new(
                    &mut output,
                    crate::emit::Format::Json,
                    crate::emit::Verbosity::Normal,
                );
                assert!(authored_steam_client_result(&mut emitter, &plan, result).is_err());
            }
            let output = String::from_utf8(output).unwrap();
            assert_eq!(output.matches("calibration.steam_client").count(), 1);
            assert!(output.contains("\"status\":\"failed\""));
            assert!(output.contains(reason));
            assert!(output.contains("\"continued\":false"));
        }
    }

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
            CompatibilityRoutingStrategy::ChildEnvironment,
            CompatibilityAddressFamily::Ipv4,
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
            CompatibilityRoutingStrategy::ChildEnvironment,
            CompatibilityAddressFamily::Ipv4,
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

    #[test]
    fn supported_attempt_bound_is_one_reachability_plus_every_concrete_protocol() {
        assert_eq!(GUIDED_CONCRETE_PROTOCOLS.len(), 13);
        assert_eq!(MAX_GUIDED_ATTEMPTS, 14);
        assert_eq!(
            no_progress_reason(MAX_GUIDED_ATTEMPTS, false),
            Some("attempt-bound-exhausted")
        );
        assert_eq!(
            no_progress_reason(1, false),
            Some("attempt-case-already-executed")
        );
        assert_eq!(no_progress_reason(1, true), None);

        let interrupt = AtomicBool::new(false);
        assert!(!guided_sequence_interrupted(&interrupt));
        interrupt.store(true, Ordering::Relaxed);
        assert!(guided_sequence_interrupted(&interrupt));
    }

    #[test]
    fn terminal_workflow_state_keeps_operator_and_failure_boundaries_distinct() {
        assert_eq!(
            terminal_workflow_state(deep_capture::RunDisposition::Completed, "completed"),
            (CalibrationWorkflowState::Ready, None)
        );
        assert_eq!(
            terminal_workflow_state(deep_capture::RunDisposition::Completed, "partial"),
            (
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Gameplay)
            )
        );
        assert_eq!(
            terminal_workflow_state(deep_capture::RunDisposition::Declined, "declined"),
            (
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Authorization)
            )
        );
        assert_eq!(
            terminal_workflow_state(deep_capture::RunDisposition::Interrupted, "interrupted"),
            (
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Interrupted)
            )
        );
        assert_eq!(
            terminal_workflow_state(deep_capture::RunDisposition::Failed, "failed"),
            (
                CalibrationWorkflowState::Paused,
                Some(CalibrationPauseReason::Failure)
            )
        );
    }

    #[test]
    fn explicit_attempt_bundles_preserve_the_first_path_and_use_safe_siblings() {
        let base = Path::new("C:\\evidence\\calibration.bundle");
        assert_eq!(
            attempt_bundle(
                Some(base),
                1,
                deep_capture_api::CalibrationPhase::Reachability,
                CompatibilityProtocol::Routing
            )
            .unwrap(),
            Some(base.to_path_buf())
        );
        assert_eq!(
            attempt_bundle(
                Some(base),
                2,
                deep_capture_api::CalibrationPhase::Tls,
                CompatibilityProtocol::Https
            )
            .unwrap(),
            Some(PathBuf::from(
                "C:\\evidence\\calibration.bundle-attempt-02-tls-https"
            ))
        );
        assert_eq!(
            attempt_bundle(
                None,
                2,
                deep_capture_api::CalibrationPhase::Tls,
                CompatibilityProtocol::Https
            )
            .unwrap(),
            None
        );
        assert!(attempt_bundle(
            Some(Path::new("/")),
            2,
            deep_capture_api::CalibrationPhase::Tls,
            CompatibilityProtocol::Https
        )
        .is_err());
    }

    #[test]
    fn exact_attempt_identity_refuses_a_second_insertion() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path().join("local.db")).unwrap();
        let mut target = TargetEntry {
            id: None,
            stable_id: 90_003,
            handle: "attempt-fixture".to_string(),
            name: "Attempt Fixture".to_string(),
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
            deep_capture_api::CalibrationProcessSnapshot::complete(Vec::<String>::new()),
            &[],
            CompatibilityRoutingStrategy::ChildEnvironment,
            CompatibilityAddressFamily::Ipv4,
        )
        .unwrap();
        let key = ExactAttemptCase::from_step(&proposal.steps[0]);
        let mut attempted = HashSet::new();
        assert!(attempted.insert(key.clone()));
        assert!(!attempted.insert(key));
        assert_eq!(attempted.len(), 1);
    }

    #[test]
    fn sequence_target_authority_detects_launch_and_install_drift() {
        let target = steam_target(Some(resolved_client_launch("client.exe")));
        let authority = SequenceTargetAuthority::from_target(&target);
        assert!(authority.matches(&target));

        let mut changed_launch = target.clone();
        changed_launch.launch_entries = Some(resolved_client_launch("other.exe"));
        assert!(!authority.matches(&changed_launch));

        let mut changed_root = target.clone();
        changed_root.install_root = Some("C:\\Other".to_string());
        assert!(!authority.matches(&changed_root));
    }
}
