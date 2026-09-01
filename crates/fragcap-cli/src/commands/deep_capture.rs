// SPDX-License-Identifier: Apache-2.0

//! `deep-capture`: explicit scoped local proxy inspection for one stored target.
//!
//! The command maps arguments and presentation onto the library-owned native
//! session. Proxy protocol, trust identity, and lifecycle policy stay below this
//! boundary.

use std::cell::RefCell;
use std::fs;
use std::io::IsTerminal;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use fragcap::deep_capture::{calibration_outcome, observation_proves_final_client_ca_acceptance};
use fragcap::deep_capture::{
    calibration_outcome_reason, terminal_calibration_outcome, CalibrationOutcome, CalibrationPhase,
    CompatibilityObservation as Observation, Inspectability,
};
#[cfg(windows)]
use fragcap::deep_capture::{CertificateStore, NativeCertificateStore, TrustMutation, TrustState};
use fragcap::targets::{
    entry_windows_clients, entry_windows_launch_entries, resolve_id, resolve_positional,
    CompatibilityEvidenceSource, CompatibilityFact, CompatibilityFactKey, CompatibilityLaunchCase,
    Selection, Store, TargetEntry,
};
#[cfg(test)]
use fragcap::FlowId;
use fragcap::{
    CaptureStats, CapturedPacket, FlowKey, FlowRegistry, InterfaceDeclaration, InterfaceId,
    LinkType, Payload, PcapngWriter, Proto, RawPacket, Sink, StopReason, Timestamp,
};
use serde_json::json;

use crate::args::Direction;
use crate::cli::{
    CaptureArgs, ControlledTargetArgs, DeepCaptureArgs, DeepCaptureCalibrationArg,
    DeepCaptureLaunchCaseArg, OfflineArgs, ScopeArg,
};
use crate::commands::{capture, target_resolve};
use crate::emit::Emitter;
use crate::events::{rfc3339_utc, Event};
use crate::exit::{CliError, Exit};
use crate::paths;

const CONTROLLED_TARGET_HANDLE: &str = "sample-target";
const CONTROLLED_TARGET_STABLE_ID: i64 = 75_000;
const CALIBRATION_LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);
const CALIBRATION_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(60);
const CALIBRATION_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
const CALIBRATION_CLEANUP_TIMEOUT: Duration = Duration::from_secs(15);
const WARM_RESTART_POLL_INTERVAL: Duration = Duration::from_millis(250);

fn calibration_phase(value: DeepCaptureCalibrationArg) -> CalibrationPhase {
    match value {
        DeepCaptureCalibrationArg::Reachability => CalibrationPhase::Reachability,
        DeepCaptureCalibrationArg::Tls => CalibrationPhase::Tls,
    }
}

fn library_launch_case(value: CompatibilityLaunchCase) -> fragcap::deep_capture::LaunchCase {
    use fragcap::deep_capture::LaunchCase;
    match value {
        CompatibilityLaunchCase::SteamProtocolWarm => LaunchCase::SteamProtocolWarm,
        CompatibilityLaunchCase::SteamProtocolCold => LaunchCase::SteamProtocolCold,
        CompatibilityLaunchCase::DirectExeWarm => LaunchCase::DirectExeWarm,
        CompatibilityLaunchCase::DirectExeCold => LaunchCase::DirectExeCold,
        CompatibilityLaunchCase::PublisherLauncher => LaunchCase::PublisherLauncher,
        CompatibilityLaunchCase::PublisherLauncherWarm => LaunchCase::PublisherLauncherWarm,
        CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm => {
            LaunchCase::PublisherLauncherGameStartCleanWarm
        }
        CompatibilityLaunchCase::PublisherLauncherCold => LaunchCase::PublisherLauncherCold,
    }
}

impl From<DeepCaptureLaunchCaseArg> for CompatibilityLaunchCase {
    fn from(value: DeepCaptureLaunchCaseArg) -> Self {
        match value {
            DeepCaptureLaunchCaseArg::SteamProtocolWarm => Self::SteamProtocolWarm,
            DeepCaptureLaunchCaseArg::SteamProtocolCold => Self::SteamProtocolCold,
            DeepCaptureLaunchCaseArg::DirectExeWarm => Self::DirectExeWarm,
            DeepCaptureLaunchCaseArg::DirectExeCold => Self::DirectExeCold,
            DeepCaptureLaunchCaseArg::PublisherLauncher => Self::PublisherLauncher,
            DeepCaptureLaunchCaseArg::PublisherLauncherWarm => Self::PublisherLauncherWarm,
            DeepCaptureLaunchCaseArg::PublisherLauncherGameStartCleanWarm => {
                Self::PublisherLauncherGameStartCleanWarm
            }
            DeepCaptureLaunchCaseArg::PublisherLauncherCold => Self::PublisherLauncherCold,
        }
    }
}

#[derive(Clone, Debug)]
struct CalibrationPlan {
    target: String,
    phase: CalibrationPhase,
    declared_launch_case: CompatibilityLaunchCase,
    observed_launch_case: CompatibilityLaunchCase,
    bundle: PathBuf,
    deadlines: CalibrationDeadlines,
}

#[derive(Clone, Copy, Debug)]
struct CalibrationDeadlines {
    launch: Duration,
    observation: Duration,
    shutdown: Duration,
    cleanup: Duration,
}

impl CalibrationDeadlines {
    fn from_args(args: &DeepCaptureArgs) -> Self {
        Self {
            launch: bounded_timeout(args.wait, CALIBRATION_LAUNCH_TIMEOUT),
            observation: bounded_timeout(args.duration, CALIBRATION_OBSERVATION_TIMEOUT),
            shutdown: CALIBRATION_SHUTDOWN_TIMEOUT,
            cleanup: CALIBRATION_CLEANUP_TIMEOUT,
        }
    }

    fn seconds(duration: Duration) -> u64 {
        u64::try_from(duration.as_millis().saturating_add(999) / 1_000).unwrap_or(u64::MAX)
    }
}

fn bounded_timeout(provided: Option<Duration>, maximum: Duration) -> Duration {
    provided.unwrap_or(maximum).min(maximum)
}

#[cfg(windows)]
fn remaining_timeout(started: Instant, total: Duration) -> Duration {
    total.saturating_sub(started.elapsed())
}

fn calibration_answer_is_affirmative(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

impl CalibrationPlan {
    fn emit(&self, emitter: &mut Emitter) {
        emitter.event(&Event::DeepCaptureCalibrationPlan {
            target: self.target.clone(),
            phase: self.phase.as_str().to_string(),
            declared_launch_case: self.declared_launch_case.as_str().to_string(),
            observed_launch_case: self.observed_launch_case.as_str().to_string(),
            proxy_backend: "fragcap-native".to_string(),
            bundle: self.bundle.display().to_string(),
            trust_action: if self.phase == CalibrationPhase::Tls {
                "session-owned current-user CA trust"
            } else {
                "none"
            }
            .to_string(),
            launch_timeout_secs: CalibrationDeadlines::seconds(self.deadlines.launch),
            observation_timeout_secs: CalibrationDeadlines::seconds(self.deadlines.observation),
            shutdown_timeout_secs: CalibrationDeadlines::seconds(self.deadlines.shutdown),
            cleanup_timeout_secs: CalibrationDeadlines::seconds(self.deadlines.cleanup),
        });
        emitter.required_human(&format!(
            "Compatibility calibration plan\n  target: {}\n  phase: {}\n  launch case: {} (observed {})\n  proxy: loopback only, launch-scoped environment\n  bundle: {}\n  deadlines: launch {}s, observation {}s, shutdown {}s, cleanup {}s\n  trust action: {}\n  facts: append only directly observed rows to the selected target\n  cleanup: proxy process, listener, private CA material, and session trust if created\n  system proxy change: none\n  evidence publication: none\n",
            self.target,
            self.phase.as_str(),
            self.declared_launch_case.as_str(),
            self.observed_launch_case.as_str(),
            self.bundle.display(),
            CalibrationDeadlines::seconds(self.deadlines.launch),
            CalibrationDeadlines::seconds(self.deadlines.observation),
            CalibrationDeadlines::seconds(self.deadlines.shutdown),
            CalibrationDeadlines::seconds(self.deadlines.cleanup),
            if self.phase == CalibrationPhase::Tls {
                "session-owned current-user CA trust"
            } else {
                "none"
            }
        ));
    }
}

fn confirm_calibration(args: &DeepCaptureArgs, emitter: &mut Emitter) -> Result<bool, CliError> {
    if args.yes {
        return Ok(true);
    }
    if emitter.is_json() {
        return Err(CliError::usage(
            "JSON compatibility calibration requires --yes because it cannot prompt",
        ));
    }
    if !std::io::stdin().is_terminal() {
        return Err(CliError::usage(
            "compatibility calibration requires interactive input or --yes",
        ));
    }
    emitter.required_human("Proceed with this calibration? [y/N] ");
    emitter.flush();
    let mut answer = String::new();
    match std::io::stdin().read_line(&mut answer) {
        Ok(0) | Err(_) => Ok(false),
        Ok(_) => Ok(calibration_answer_is_affirmative(&answer)),
    }
}

fn confirm_warm_restart(
    args: &DeepCaptureArgs,
    emitter: &mut Emitter,
    prompt: &str,
) -> Result<bool, CliError> {
    if args.yes {
        return Ok(true);
    }
    if emitter.is_json() {
        return Err(CliError::usage(
            "JSON warm restart requires --yes because it cannot prompt",
        ));
    }
    if !std::io::stdin().is_terminal() {
        return Err(CliError::usage(
            "warm restart requires interactive input or --yes",
        ));
    }
    emitter.required_human(prompt);
    emitter.flush();
    let mut answer = String::new();
    match std::io::stdin().read_line(&mut answer) {
        Ok(0) | Err(_) => Ok(false),
        Ok(_) => Ok(calibration_answer_is_affirmative(&answer)),
    }
}

#[derive(Clone, Debug)]
struct WarmRestartContext {
    target: String,
    authority: WarmRestartAuthority,
    plan: fragcap::deep_capture::WarmRestartPlan,
}

#[derive(Clone, Debug, PartialEq)]
struct WarmRestartAuthority {
    stable_id: i64,
    anchor: Option<String>,
    install_root: Option<String>,
    launch_entries: Option<serde_json::Value>,
}

impl WarmRestartAuthority {
    fn from_target(target: &TargetEntry) -> Self {
        Self {
            stable_id: target.stable_id,
            anchor: target.anchor.clone(),
            install_root: target.install_root.clone(),
            launch_entries: target.launch_entries.clone(),
        }
    }
}

fn steam_restart_images(target: &TargetEntry) -> Vec<String> {
    platform_restart_images(entry_windows_clients(target))
}

fn platform_restart_images(client_images: Vec<String>) -> Vec<String> {
    let mut images = Vec::with_capacity(client_images.len() + 1);
    images.push("steam.exe".to_string());
    images.extend(client_images);
    images
}

fn warm_restart_images(
    target: &TargetEntry,
    launch_case: CompatibilityLaunchCase,
) -> Result<Vec<String>, CliError> {
    match launch_case {
        CompatibilityLaunchCase::SteamProtocolWarm => Ok(steam_restart_images(target)),
        CompatibilityLaunchCase::DirectExeWarm => Ok(entry_windows_clients(target)),
        CompatibilityLaunchCase::PublisherLauncherWarm
        | CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm => {
            match fragcap::managed_launch::prepare_managed_launch(target)
                .map_err(|error| CliError::usage(error.to_string()))?
            {
                fragcap::managed_launch::ManagedLaunch::Publisher(publisher) => Ok(publisher
                    .stages()
                    .iter()
                    .map(|stage| stage.image_name().to_string_lossy().into_owned())
                    .collect()),
                _ => Err(CliError::failure(
                    "warm publisher state did not reprepare as a publisher chain",
                )),
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn emit_restart_outcome(
    emitter: &mut Emitter,
    context: &WarmRestartContext,
    stage: &str,
    status: &str,
    cold_case: Option<fragcap::deep_capture::LaunchCase>,
    reason: &str,
) {
    emitter.event(&Event::DeepCaptureRestart {
        target: context.target.clone(),
        stage: stage.to_string(),
        status: status.to_string(),
        warm_case: context.plan.warm_case().as_str().to_string(),
        cold_case: cold_case.map(|case| case.as_str().to_string()),
        reason: reason.to_string(),
    });
}

fn run_warm_restart(
    args: &DeepCaptureArgs,
    store: &Store,
    emitter: &mut Emitter,
) -> Result<Option<WarmRestartContext>, CliError> {
    if !args.restart_warm {
        return Ok(None);
    }
    let target = resolve_target(store, args)?;
    let observed = launch_case(&target)?;
    let images = warm_restart_images(&target, observed)?;
    if images.is_empty() {
        emitter.progress("warm restart was requested, but the selected target is already cold");
        return Ok(None);
    }
    let plan = fragcap::deep_capture::WarmRestartPlan::new(
        library_launch_case(observed),
        images,
        args.wait,
    )
    .map_err(|error| CliError::usage(error.to_string()))?;
    let context = WarmRestartContext {
        target: target.handle.clone(),
        authority: WarmRestartAuthority::from_target(&target),
        plan,
    };
    emitter.event(&Event::DeepCaptureRestartPlan {
        target: context.target.clone(),
        warm_case: context.plan.warm_case().as_str().to_string(),
        images: context.plan.images().to_vec(),
        deadline_secs: CalibrationDeadlines::seconds(context.plan.deadline()),
    });
    emitter.required_human(&format!(
        "Warm-to-cold restart plan\n  target: {}\n  observed case: {}\n  declared images: {}\n  identity: image-name observation only; ownership is not proven\n  deadline: {}s\n  action: close the application through its normal Exit or Quit control\n  process control: none; fragcap will never force kill or signal it\n",
        context.target,
        context.plan.warm_case().as_str(),
        context.plan.images().join(", "),
        CalibrationDeadlines::seconds(context.plan.deadline()),
    ));
    if !confirm_warm_restart(
        args,
        emitter,
        "Wait while you close the application normally? [y/N] ",
    )? {
        emit_restart_outcome(
            emitter,
            &context,
            "wait-authorization",
            "declined",
            None,
            "operator declined the close-and-retry wait; no effects were applied",
        );
        return Err(CliError::usage(
            "warm restart was declined; no effects were applied",
        ));
    }
    emit_restart_outcome(
        emitter,
        &context,
        "waiting",
        "active",
        None,
        "operator authorized bounded observation while performing normal shutdown",
    );
    emitter.progress("Waiting for every declared image to become absent...");
    let started = Instant::now();
    loop {
        let present = match process_images_running(context.plan.images()) {
            Ok(present) => present,
            Err(error) => {
                emit_restart_outcome(
                    emitter,
                    &context,
                    "waiting",
                    "inventory-failed",
                    None,
                    "the process image snapshot could not be read",
                );
                return Err(error);
            }
        };
        if context.plan.snapshot_is_cold(&present) {
            break;
        }
        if started.elapsed() >= context.plan.deadline() {
            emit_restart_outcome(
                emitter,
                &context,
                "waiting",
                "timeout",
                None,
                "one or more declared images remained present at the deadline",
            );
            return Err(CliError::failure(format!(
                "warm restart timed out after {} seconds; fragcap did not stop any process",
                CalibrationDeadlines::seconds(context.plan.deadline())
            )));
        }
        std::thread::sleep(
            WARM_RESTART_POLL_INTERVAL
                .min(context.plan.deadline().saturating_sub(started.elapsed())),
        );
    }
    let refreshed = match resolve_target(store, args) {
        Ok(target) => target,
        Err(error) => {
            emit_restart_outcome(
                emitter,
                &context,
                "reprepare",
                "target-resolution-failed",
                None,
                "the selected target could not be resolved again after shutdown",
            );
            return Err(CliError::usage(format!(
                "warm restart reached an observed cold state, but target re-resolution failed: {error}"
            )));
        }
    };
    if WarmRestartAuthority::from_target(&refreshed) != context.authority {
        emit_restart_outcome(
            emitter,
            &context,
            "reprepare",
            "changed-target",
            None,
            "the selected target's launch authority changed after shutdown",
        );
        return Err(CliError::usage(
            "the selected target launch declaration changed during warm restart; no effects were applied",
        ));
    }
    let cold = match launch_case(&refreshed) {
        Ok(case) => library_launch_case(case),
        Err(error) => {
            emit_restart_outcome(
                emitter,
                &context,
                "reprepare",
                "launch-preparation-failed",
                None,
                "current launch facts could not be prepared after shutdown",
            );
            return Err(error);
        }
    };
    if cold != context.plan.cold_case() {
        emit_restart_outcome(
            emitter,
            &context,
            "reprepare",
            "not-cold",
            Some(cold),
            "fresh launch-state resolution did not produce the corresponding cold case",
        );
        return Err(CliError::usage(format!(
            "warm restart did not produce the expected {} state; observed {}",
            context.plan.cold_case().as_str(),
            cold.as_str()
        )));
    }
    emit_restart_outcome(
        emitter,
        &context,
        "reprepare",
        "cold-ready",
        Some(cold),
        "all declared images are absent and current target facts resolve to the cold case",
    );
    Ok(Some(context))
}

struct LibraryTargetAdapter<'a> {
    args: &'a DeepCaptureArgs,
    store: Rc<RefCell<Store>>,
    selected: Rc<RefCell<Option<TargetEntry>>>,
    selected_launch_case: Rc<RefCell<Option<CompatibilityLaunchCase>>>,
}

impl fragcap::deep_capture::TargetResolver for LibraryTargetAdapter<'_> {
    fn resolve(
        &mut self,
        _config: &fragcap::deep_capture::SessionConfig,
    ) -> Result<fragcap::deep_capture::PreparedTarget, fragcap::deep_capture::PreflightRefusal>
    {
        let target = resolve_target(&self.store.borrow(), self.args)
            .map_err(|error| library_refusal("target-resolution", error))?;
        let id = target.id.ok_or_else(|| {
            fragcap::deep_capture::PreflightRefusal::new(
                "target-id-missing",
                "resolved target has no local row id",
            )
        })?;
        let launch_case = effective_launch_case(&target, self.args.controlled_target)
            .map_err(|error| library_refusal("launch-case", error))?;
        *self.selected_launch_case.borrow_mut() = Some(launch_case);
        let prepared = fragcap::deep_capture::PreparedTarget {
            id,
            handle: target.handle.clone(),
            launch_case: library_launch_case(launch_case),
        };
        *self.selected.borrow_mut() = Some(target);
        Ok(prepared)
    }

    fn validate_compatibility(
        &mut self,
        target: &fragcap::deep_capture::PreparedTarget,
        config: &fragcap::deep_capture::SessionConfig,
    ) -> Result<(), fragcap::deep_capture::PreflightRefusal> {
        let facts = self
            .store
            .borrow()
            .compatibility_facts_for_target(target.id)
            .map_err(|error| {
                fragcap::deep_capture::PreflightRefusal::new(
                    "compatibility-read",
                    error.to_string(),
                )
            })?;
        if self.args.controlled_target {
            require_controlled_target(
                self.selected
                    .borrow()
                    .as_ref()
                    .expect("resolved target retained"),
            )
            .map_err(|error| library_refusal("controlled-target", error))?;
        }
        fragcap::deep_capture::validate_compatibility_prerequisites(
            config.mode,
            self.args.controlled_target,
            &facts,
            target.launch_case,
        )?;
        Ok(())
    }
}

fn library_refusal(code: &'static str, error: CliError) -> fragcap::deep_capture::PreflightRefusal {
    fragcap::deep_capture::PreflightRefusal::new(code, error.to_string())
}

struct LibraryEndpointAdapter;

impl fragcap::deep_capture::EndpointAllocator for LibraryEndpointAdapter {
    fn select(
        &mut self,
    ) -> Result<fragcap::deep_capture::LoopbackEndpoint, fragcap::deep_capture::PreflightRefusal>
    {
        select_loopback_port()
            .map(|port| fragcap::deep_capture::LoopbackEndpoint { port })
            .map_err(|error| library_refusal("loopback-endpoint", error))
    }
}

struct LibraryIdentifierAdapter;

impl fragcap::deep_capture::IdentifierSource for LibraryIdentifierAdapter {
    fn next_id(
        &mut self,
        kind: &'static str,
    ) -> Result<String, fragcap::deep_capture::PreflightRefusal> {
        Ok(if kind == "session" {
            session_id()
        } else {
            format!("plan-{}", session_id())
        })
    }
}

struct LibraryClockAdapter {
    started: Instant,
}

impl fragcap::deep_capture::SessionClock for LibraryClockAdapter {
    fn wall_now(&mut self) -> SystemTime {
        SystemTime::now()
    }

    fn monotonic_elapsed(&mut self) -> Duration {
        self.started.elapsed()
    }
}

#[derive(Default)]
struct LibraryRuntime {
    controlled_process_id: Option<u32>,
    process_events: Vec<String>,
    interrupted: bool,
    backend: Option<ProxyBackend>,
    trust: Option<TrustOutcome>,
    observations: Vec<Observation>,
    listen_port: Option<u16>,
    started_at: Option<SystemTime>,
}

fn library_stage_failure(
    stage: fragcap::deep_capture::Stage,
    code: &'static str,
    error: CliError,
) -> fragcap::deep_capture::StageFailure {
    fragcap::deep_capture::StageFailure::new(stage, code, error.to_string())
}

fn library_cleanup_resource(value: CleanupResource) -> fragcap::deep_capture::CleanupResult {
    fragcap::deep_capture::CleanupResult {
        resource: value.resource,
        status: match value.status.as_str() {
            "succeeded" => fragcap::deep_capture::CleanupStatus::Released,
            "not-needed" => fragcap::deep_capture::CleanupStatus::NotNeeded,
            _ => fragcap::deep_capture::CleanupStatus::Failed,
        },
        reason: value.reason,
    }
}

struct LibraryTrustAdapter {
    controlled: bool,
    runtime: Rc<RefCell<LibraryRuntime>>,
}

impl fragcap::deep_capture::TrustManager for LibraryTrustAdapter {
    fn acquire(
        &mut self,
        _plan: &fragcap::deep_capture::SessionPlan,
        route: &fragcap::deep_capture::ProxyRoute,
        _budget: fragcap::deep_capture::Budget,
    ) -> Result<Box<dyn fragcap::deep_capture::TrustLease>, fragcap::deep_capture::StageFailure>
    {
        let mut manager: Box<dyn TrustManager> = if self.controlled {
            Box::new(ControlledTrustManager)
        } else {
            if route.ca_der().is_empty() || route.ca_sha1_thumbprint().is_empty() {
                return Err(fragcap::deep_capture::StageFailure::new(
                    fragcap::deep_capture::Stage::Trust,
                    "ca-material-missing",
                    "the proxy did not expose session CA material",
                ));
            }
            platform_trust_manager(
                route.ca_der().to_vec(),
                route.ca_sha1_thumbprint().to_string(),
            )
            .map_err(|error| {
                library_stage_failure(fragcap::deep_capture::Stage::Trust, "manager", error)
            })?
        };
        let outcome = manager.ensure_trusted(true).map_err(|error| {
            library_stage_failure(fragcap::deep_capture::Stage::Trust, "acquire", error)
        })?;
        self.runtime.borrow_mut().trust = Some(outcome);
        Ok(Box::new(LibraryTrustLease { manager }))
    }
}

struct LibraryTrustLease {
    manager: Box<dyn TrustManager>,
}

impl fragcap::deep_capture::TrustLease for LibraryTrustLease {
    fn cleanup(
        &mut self,
        budget: fragcap::deep_capture::Budget,
    ) -> fragcap::deep_capture::CleanupResult {
        library_cleanup_resource(self.manager.cleanup(budget.remaining()))
    }
}

struct LibraryLaunchAdapter;

impl fragcap::deep_capture::LaunchAdapter for LibraryLaunchAdapter {
    fn launch(
        &mut self,
        _target: &fragcap::deep_capture::PreparedTarget,
        _launch_case: fragcap::deep_capture::LaunchCase,
        _route: &fragcap::deep_capture::AppliedRoute,
        _budget: fragcap::deep_capture::Budget,
    ) -> Result<Box<dyn fragcap::deep_capture::LaunchLease>, fragcap::deep_capture::StageFailure>
    {
        Ok(Box::new(LibraryLaunchLease))
    }
}

struct LibraryLaunchLease;

impl fragcap::deep_capture::LaunchLease for LibraryLaunchLease {
    fn cleanup(
        &mut self,
        _budget: fragcap::deep_capture::Budget,
    ) -> fragcap::deep_capture::CleanupResult {
        fragcap::deep_capture::CleanupResult {
            resource: "managed-launch".to_string(),
            status: fragcap::deep_capture::CleanupStatus::NotNeeded,
            reason: "ordinary Capture owns the managed launch lifetime".to_string(),
        }
    }
}

struct LibraryCaptureAdapter<'a, 'e, 'w> {
    args: &'a DeepCaptureArgs,
    emitter: Rc<RefCell<&'e mut Emitter<'w>>>,
    prepared: Option<(CaptureArgs, capture::PreparedCapture)>,
    runtime: Rc<RefCell<LibraryRuntime>>,
    observation_context: fragcap::deep_capture::NativeObservationContext,
    mode: fragcap::deep_capture::SessionMode,
}

impl fragcap::deep_capture::CaptureRunner for LibraryCaptureAdapter<'_, '_, '_> {
    fn prepare(
        &mut self,
        config: &fragcap::deep_capture::SessionConfig,
        target: &fragcap::deep_capture::PreparedTarget,
        _endpoint: fragcap::deep_capture::LoopbackEndpoint,
    ) -> Result<fragcap::deep_capture::PreparedCapture, fragcap::deep_capture::PreflightRefusal>
    {
        self.mode = config.mode;
        if !self.args.controlled_target {
            let deadlines = CalibrationDeadlines {
                launch: config.deadlines.launch,
                observation: config.deadlines.observation,
                shutdown: config.deadlines.shutdown,
                cleanup: config.deadlines.cleanup,
            };
            let mut capture_args = real_capture_args(self.args, &config.bundle, deadlines);
            if target.launch_case == fragcap::deep_capture::LaunchCase::SteamProtocolCold {
                capture_args.wait = owned_platform_wait(capture_args.wait, config.deadlines.launch);
            }
            let prepared =
                if target.launch_case == fragcap::deep_capture::LaunchCase::SteamProtocolCold {
                    capture::prepare_owned_platform(&capture_args, &mut self.emitter.borrow_mut())
                } else {
                    capture::prepare(&capture_args, &mut self.emitter.borrow_mut())
                }
                .map_err(|error| library_refusal("capture-prepare", error))?;
            self.prepared = Some((capture_args, prepared));
        }
        Ok(fragcap::deep_capture::PreparedCapture {
            token: "ordinary-capture".to_string(),
        })
    }

    fn run(
        &mut self,
        _prepared: &fragcap::deep_capture::PreparedCapture,
        route: &fragcap::deep_capture::AppliedRoute,
        budget: fragcap::deep_capture::Budget,
    ) -> Result<fragcap::deep_capture::CaptureRunResult, fragcap::deep_capture::StageFailure> {
        if self.args.controlled_target {
            let phase = match self.mode {
                fragcap::deep_capture::SessionMode::Capture => None,
                fragcap::deep_capture::SessionMode::ReachabilityCalibration => {
                    Some(CalibrationPhase::Reachability)
                }
                fragcap::deep_capture::SessionMode::TlsCalibration => Some(CalibrationPhase::Tls),
                _ => None,
            };
            let runtime = Rc::clone(&self.runtime);
            let context = self.observation_context.clone();
            run_controlled_target_harness(
                route.proxy(),
                phase,
                budget.remaining(),
                move |process_id| {
                    runtime.borrow_mut().controlled_process_id = Some(process_id);
                    context.record_controlled_process_id(process_id);
                },
            )
            .map_err(|error| {
                library_stage_failure(
                    fragcap::deep_capture::Stage::Capture,
                    "controlled-target",
                    error,
                )
            })?;
            return Ok(fragcap::deep_capture::CaptureRunResult {
                observations: Vec::new(),
                interrupted: false,
            });
        }
        let (capture_args, mut prepared) = self.prepared.take().ok_or_else(|| {
            fragcap::deep_capture::StageFailure::new(
                fragcap::deep_capture::Stage::Capture,
                "capture-not-prepared",
                "ordinary Capture preparation was not retained",
            )
        })?;
        prepared
            .with_launch_environment(route.environment())
            .map_err(|error| {
                library_stage_failure(
                    fragcap::deep_capture::Stage::Launch,
                    "launch-route-environment",
                    error,
                )
            })?;
        let (result, process_events, interrupted) = run_real_capture(
            &capture_args,
            prepared,
            self.observation_context.flow_registry(),
            &mut self.emitter.borrow_mut(),
        );
        {
            let mut runtime = self.runtime.borrow_mut();
            runtime.process_events = process_events;
            runtime.interrupted = interrupted;
        }
        result.map_err(|error| {
            library_stage_failure(fragcap::deep_capture::Stage::Capture, "capture-run", error)
        })?;
        Ok(fragcap::deep_capture::CaptureRunResult {
            observations: Vec::new(),
            interrupted,
        })
    }

    fn stop(
        &mut self,
        _budget: fragcap::deep_capture::Budget,
    ) -> fragcap::deep_capture::CleanupResult {
        fragcap::deep_capture::CleanupResult {
            resource: "capture".to_string(),
            status: fragcap::deep_capture::CleanupStatus::NotNeeded,
            reason: "ordinary Capture returned after its own bounded stop".to_string(),
        }
    }
}

struct LibraryFactAdapter {
    store: Rc<RefCell<Store>>,
    selected_launch_case: Rc<RefCell<Option<CompatibilityLaunchCase>>>,
    runtime: Rc<RefCell<LibraryRuntime>>,
    controlled: bool,
}

impl fragcap::deep_capture::CompatibilityRepository for LibraryFactAdapter {
    fn append(
        &mut self,
        target: &fragcap::deep_capture::PreparedTarget,
        fact: &fragcap::deep_capture::CompatibilityFact,
    ) -> fragcap::deep_capture::FactWriteStatus {
        let key = match CompatibilityFactKey::parse(&fact.kind) {
            Ok(key) => key,
            Err(error) => {
                return fragcap::deep_capture::FactWriteStatus::Failed {
                    code: "fact-key".to_string(),
                    detail: error.to_string(),
                }
            }
        };
        let launch_case = match *self.selected_launch_case.borrow() {
            Some(value) => value,
            None => {
                return fragcap::deep_capture::FactWriteStatus::Failed {
                    code: "launch-case".to_string(),
                    detail: "the resolved launch case is unavailable".to_string(),
                }
            }
        };
        let runtime = self.runtime.borrow();
        let backend = match runtime.backend.as_ref() {
            Some(value) => value,
            None => {
                return fragcap::deep_capture::FactWriteStatus::Failed {
                    code: "proxy-backend".to_string(),
                    detail: "the selected proxy backend is unavailable".to_string(),
                }
            }
        };
        let final_owner = fact
            .final_owner_index
            .and_then(|index| runtime.observations.get(index));
        let result = insert_fact(
            &mut self.store.borrow_mut(),
            key,
            &fact.value,
            FactContext {
                target_id: target.id,
                launch_case,
                backend,
                controlled: self.controlled,
                final_owner,
                phase: fact.phase,
            },
        );
        match result {
            Ok(()) => fragcap::deep_capture::FactWriteStatus::Appended,
            Err(error) => fragcap::deep_capture::FactWriteStatus::Failed {
                code: "fact-append".to_string(),
                detail: error.to_string(),
            },
        }
    }
}

struct LibraryArtifactAdapter<'e, 'w> {
    emitter: Rc<RefCell<&'e mut Emitter<'w>>>,
    selected: Rc<RefCell<Option<TargetEntry>>>,
    selected_launch_case: Rc<RefCell<Option<CompatibilityLaunchCase>>>,
    runtime: Rc<RefCell<LibraryRuntime>>,
    deadlines: CalibrationDeadlines,
}

impl fragcap::deep_capture::ArtifactSink for LibraryArtifactAdapter<'_, '_> {
    fn validate_destination(
        &mut self,
        path: &Path,
    ) -> Result<(), fragcap::deep_capture::PreflightRefusal> {
        validate_bundle_root(path).map_err(|error| library_refusal("bundle-destination", error))
    }

    fn prepare(
        &mut self,
        plan: &fragcap::deep_capture::SessionPlan,
    ) -> Result<(), fragcap::deep_capture::StageFailure> {
        fragcap::deep_capture::prepare_bundle(&plan.bundle)
            .and_then(|()| {
                if let Some(root) = paths::deep_capture_session_dir() {
                    crate::doctor::fix::register_session_owner(&root, &plan.bundle)?;
                }
                Ok(())
            })
            .and_then(|()| {
                fragcap::deep_capture::write_crash_prefix(&plan.bundle, &plan.session_id)
            })
            .map_err(|error| {
                fragcap::deep_capture::StageFailure::new(
                    fragcap::deep_capture::Stage::Bundle,
                    "bundle-protection-failed",
                    error.to_string(),
                )
            })
    }

    fn finalize(
        &mut self,
        bundle: &Path,
        snapshot: &fragcap::deep_capture::TerminalSnapshot,
    ) -> Vec<fragcap::deep_capture::ArtifactResult> {
        if let Err(error) = fs::create_dir_all(bundle) {
            return vec![fragcap::deep_capture::ArtifactResult {
                role: "bundle-finalization".to_string(),
                path: bundle.to_path_buf(),
                sensitivity: fragcap::deep_capture::Sensitivity::Metadata,
                required: true,
                status: fragcap::deep_capture::ArtifactStatus::Failed {
                    code: "bundle-create".to_string(),
                    detail: error.to_string(),
                },
            }];
        }
        let runtime = self.runtime.borrow();
        let target = self
            .selected
            .borrow()
            .as_ref()
            .expect("preflight retained the target")
            .clone();
        let backend = runtime.backend.clone().unwrap_or_else(|| ProxyBackend {
            name: "unavailable".to_string(),
            version: "unavailable".to_string(),
        });
        let launch_case = self
            .selected_launch_case
            .borrow()
            .unwrap_or(CompatibilityLaunchCase::SteamProtocolCold);
        let session = DeepCaptureSession {
            session_id: snapshot.session_id.clone(),
            bundle: bundle.to_path_buf(),
            target,
            target_id: snapshot.target.id,
            backend,
            launch_case,
            listen_port: runtime.listen_port.unwrap_or_default(),
            started_at: runtime.started_at.unwrap_or(snapshot.finished_at),
        };
        let trust = runtime.trust.clone().unwrap_or(TrustOutcome {
            state: "not-requested".to_string(),
            action: "none".to_string(),
            thumbprint: None,
        });
        let fact_writes: Vec<FactWriteResult> = snapshot
            .fact_writes
            .iter()
            .map(|write| FactWriteResult {
                key: write.fact.kind.clone(),
                value: write.fact.value.clone(),
                status: match write.status {
                    fragcap::deep_capture::FactWriteStatus::Appended => "performed",
                    fragcap::deep_capture::FactWriteStatus::Skipped { .. } => "skipped",
                    fragcap::deep_capture::FactWriteStatus::Failed { .. } => "failed",
                    _ => "failed",
                }
                .to_string(),
                reason: match &write.status {
                    fragcap::deep_capture::FactWriteStatus::Appended => None,
                    fragcap::deep_capture::FactWriteStatus::Skipped { reason } => {
                        Some(reason.clone())
                    }
                    fragcap::deep_capture::FactWriteStatus::Failed { detail, .. } => {
                        Some(detail.clone())
                    }
                    _ => Some("unrecognized library fact-write status".to_string()),
                },
            })
            .collect();
        let calibration = match snapshot.mode {
            fragcap::deep_capture::SessionMode::ReachabilityCalibration => {
                Some(CalibrationPhase::Reachability)
            }
            fragcap::deep_capture::SessionMode::TlsCalibration => Some(CalibrationPhase::Tls),
            fragcap::deep_capture::SessionMode::Capture => None,
            _ => None,
        };
        let outcome = calibration.map(|phase| {
            terminal_calibration_outcome(
                phase,
                &snapshot.observations,
                runtime.interrupted,
                !snapshot.failures.is_empty(),
            )
        });
        let session_state = match snapshot.outcome {
            fragcap::deep_capture::SessionOutcome::Complete => "complete",
            fragcap::deep_capture::SessionOutcome::Partial
            | fragcap::deep_capture::SessionOutcome::Interrupted => "partial",
            fragcap::deep_capture::SessionOutcome::Failed => "failed",
            _ => "failed",
        };
        let context = BundleContext {
            session: &session,
            controlled: snapshot.controlled,
            har_requested: snapshot.artifacts.har,
            key_log_requested: snapshot.artifacts.key_log,
            sensitive_retention: snapshot.artifacts.sensitive_retention,
            observations: &snapshot.observations,
            trust: &trust,
            session_state,
            controlled_process_id: runtime.controlled_process_id,
            process_events: &runtime.process_events,
            calibration,
            calibration_outcome: outcome,
            deadlines: self.deadlines,
            fact_writes: &fact_writes,
        };
        let write_result = write_bundle(&context, &mut self.emitter.borrow_mut());
        let roles = [
            (
                "pcapng",
                "capture.fcapng",
                fragcap::deep_capture::Sensitivity::Metadata,
            ),
            (
                "application-jsonl",
                "application.jsonl",
                fragcap::deep_capture::Sensitivity::Payload,
            ),
            (
                "proxy-lifecycle",
                "proxy.jsonl",
                fragcap::deep_capture::Sensitivity::Payload,
            ),
            (
                "cleanup-lifecycle",
                "cleanup.jsonl",
                fragcap::deep_capture::Sensitivity::Metadata,
            ),
            (
                "resource-journal",
                "resource-journal.jsonl",
                fragcap::deep_capture::Sensitivity::Secret,
            ),
            (
                "process-trace",
                "process-trace.jsonl",
                fragcap::deep_capture::Sensitivity::Payload,
            ),
            (
                "compatibility",
                "compatibility.json",
                fragcap::deep_capture::Sensitivity::Metadata,
            ),
            (
                "cleanup-summary",
                "cleanup.json",
                fragcap::deep_capture::Sensitivity::Metadata,
            ),
            (
                "manifest",
                "manifest.json",
                fragcap::deep_capture::Sensitivity::Metadata,
            ),
        ];
        let mut results: Vec<_> = roles
            .into_iter()
            .map(|(role, path, sensitivity)| {
                let full = bundle.join(path);
                let status = if full.is_file() {
                    fragcap::deep_capture::ArtifactStatus::Written
                } else if let Err(error) = &write_result {
                    fragcap::deep_capture::ArtifactStatus::Failed {
                        code: "bundle-write".to_string(),
                        detail: error.to_string(),
                    }
                } else {
                    fragcap::deep_capture::ArtifactStatus::Omitted {
                        reason: "artifact was not produced".to_string(),
                    }
                };
                fragcap::deep_capture::ArtifactResult {
                    role: role.to_string(),
                    path: full,
                    sensitivity,
                    required: role != "pcapng" || session_state == "complete",
                    status,
                }
            })
            .collect();
        if let Err(error) = write_result {
            results.push(fragcap::deep_capture::ArtifactResult {
                role: "bundle-finalization".to_string(),
                path: bundle.to_path_buf(),
                sensitivity: fragcap::deep_capture::Sensitivity::Metadata,
                required: true,
                status: fragcap::deep_capture::ArtifactStatus::Failed {
                    code: "bundle-write".to_string(),
                    detail: error.to_string(),
                },
            });
        }
        results
    }

    fn reconcile(
        &mut self,
        bundle: &Path,
        snapshot: &fragcap::deep_capture::TerminalSnapshot,
    ) -> Vec<fragcap::deep_capture::ArtifactResult> {
        let result = (|| -> Result<(), CliError> {
            let cleanup = cleanup_from_lifecycle(&bundle.join("cleanup.jsonl"))?;
            write_file(
                bundle.join("cleanup.json"),
                cleanup_json(&snapshot.session_id, &cleanup)?.as_bytes(),
            )?;
            let manifest_path = bundle.join("manifest.json");
            if manifest_path.is_file() {
                let mut manifest: serde_json::Value =
                    serde_json::from_slice(&fs::read(&manifest_path).map_err(|error| {
                        CliError::failure(format!(
                            "cannot read {} for reconciliation: {error}",
                            manifest_path.display()
                        ))
                    })?)
                    .map_err(|error| CliError::failure(error.to_string()))?;
                manifest["cleanup"]["status"] = json!(cleanup.status());
                let bytes = serde_json::to_vec_pretty(&manifest)
                    .map_err(|error| CliError::failure(error.to_string()))?;
                write_file(manifest_path, &bytes)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => Vec::new(),
            Err(error) => vec![fragcap::deep_capture::ArtifactResult {
                role: "lifecycle-reconciliation".to_string(),
                path: bundle.to_path_buf(),
                sensitivity: fragcap::deep_capture::Sensitivity::Metadata,
                required: true,
                status: fragcap::deep_capture::ArtifactStatus::Failed {
                    code: "lifecycle-reconciliation".to_string(),
                    detail: error.to_string(),
                },
            }],
        }
    }
}

struct LibraryEventAdapter<'a, 'e, 'w> {
    args: &'a DeepCaptureArgs,
    emitter: Rc<RefCell<&'e mut Emitter<'w>>>,
    selected: Rc<RefCell<Option<TargetEntry>>>,
    selected_launch_case: Rc<RefCell<Option<CompatibilityLaunchCase>>>,
    runtime: Rc<RefCell<LibraryRuntime>>,
    bundle: PathBuf,
}

impl fragcap::deep_capture::EventSink for LibraryEventAdapter<'_, '_, '_> {
    fn emit(
        &mut self,
        event: &fragcap::deep_capture::DeepCaptureEvent,
    ) -> Result<(), fragcap::deep_capture::StageFailure> {
        let mut emitter = self.emitter.borrow_mut();
        match event {
            fragcap::deep_capture::DeepCaptureEvent::Plan { plan, .. } => {
                {
                    let mut runtime = self.runtime.borrow_mut();
                    runtime.backend = Some(ProxyBackend {
                        name: plan.proxy_backend.name.clone(),
                        version: plan.proxy_backend.version.clone(),
                    });
                    runtime.listen_port = Some(plan.endpoint.port);
                    runtime.started_at = Some(SystemTime::now());
                }
                let target = self
                    .selected
                    .borrow()
                    .as_ref()
                    .expect("preflight retained target")
                    .clone();
                emitter.event(&Event::DeepCapturePreflight {
                    status: "ready".to_string(),
                    blockers: 0,
                    warnings: 0,
                    target: target.handle.clone(),
                    proxy_backend: plan.proxy_backend.name.clone(),
                    trust_state: "confirmation-present".to_string(),
                });
                emitter.progress("Deep Capture preflight passed");
                if let Some(phase) = self.args.calibrate.map(calibration_phase) {
                    emitter.event(&Event::DeepCaptureCalibrationPhase {
                        session_id: Some(plan.session_id.clone()),
                        phase: phase.as_str().to_string(),
                        stage: "confirmed".to_string(),
                        status: "started".to_string(),
                        reason: "operator confirmed the displayed calibration plan".to_string(),
                    });
                }
            }
            fragcap::deep_capture::DeepCaptureEvent::ProxyStarted { session_id, .. } => {
                let runtime = self.runtime.borrow();
                let backend = runtime
                    .backend
                    .as_ref()
                    .expect("proxy start retained backend");
                emitter.event(&Event::DeepCaptureProxyStarted {
                    session_id: session_id.clone(),
                    backend: backend.name.clone(),
                    version: backend.version.clone(),
                    listen_addr: "127.0.0.1".to_string(),
                    listen_port: runtime.listen_port.unwrap_or_default(),
                });
                if self.args.key_log {
                    emitter.event(&Event::DeepCaptureKeyLogReady {
                        session_id: session_id.clone(),
                        path: self.bundle.join("tls-keylog.log").display().to_string(),
                    });
                }
            }
            fragcap::deep_capture::DeepCaptureEvent::TrustAcquired { session_id, .. } => {
                let runtime = self.runtime.borrow();
                let trust = runtime.trust.clone().unwrap_or(TrustOutcome {
                    state: "not-requested".to_string(),
                    action: "none".to_string(),
                    thumbprint: None,
                });
                emitter.event(&Event::DeepCaptureTrust {
                    session_id: session_id.clone(),
                    state: trust.state,
                    action: trust.action,
                    thumbprint: trust.thumbprint,
                });
            }
            fragcap::deep_capture::DeepCaptureEvent::LaunchStarted { session_id, .. } => {
                let target = self
                    .selected
                    .borrow()
                    .as_ref()
                    .expect("preflight retained target")
                    .clone();
                emitter.event(&Event::DeepCaptureLaunch {
                    session_id: session_id.clone(),
                    launch_case: self
                        .selected_launch_case
                        .borrow()
                        .map(|value| value.as_str().to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    scoped_proxy: true,
                    target: target.handle,
                });
            }
            fragcap::deep_capture::DeepCaptureEvent::Started { .. } => {}
            fragcap::deep_capture::DeepCaptureEvent::Observation {
                session_id,
                observation,
                ..
            } => emitter.event(&Event::DeepCaptureApplication {
                session_id: session_id.clone(),
                flow_id: observation.flow_id.map(|flow_id| flow_id.to_string()),
                proxy_connection_id: observation.proxy_connection_id.clone(),
                protocol: observation.protocol.clone(),
                inspectability: observation.inspectability.as_str().to_string(),
            }),
            fragcap::deep_capture::DeepCaptureEvent::Cleanup { .. } => {}
            fragcap::deep_capture::DeepCaptureEvent::Terminal { report, .. } => {
                if let Some(phase) = self.args.calibrate.map(calibration_phase) {
                    let runtime = self.runtime.borrow();
                    let outcome = terminal_calibration_outcome(
                        phase,
                        &report.observations,
                        runtime.interrupted,
                        !report.failures.is_empty(),
                    );
                    emitter.event(&Event::DeepCaptureCalibrationPhase {
                        session_id: Some(report.session_id.clone()),
                        phase: phase.as_str().to_string(),
                        stage: "complete".to_string(),
                        status: outcome.to_string(),
                        reason: calibration_outcome_reason(phase, outcome).to_string(),
                    });
                }
                let status = match report.outcome {
                    fragcap::deep_capture::SessionOutcome::Complete => "complete",
                    fragcap::deep_capture::SessionOutcome::Partial
                    | fragcap::deep_capture::SessionOutcome::Interrupted => "partial",
                    _ => "failed",
                };
                let cleanup_status = if report.cleanup.iter().all(|result| {
                    matches!(
                        result.status,
                        fragcap::deep_capture::CleanupStatus::Released
                            | fragcap::deep_capture::CleanupStatus::NotNeeded
                    )
                }) {
                    "succeeded"
                } else {
                    "failed"
                };
                emitter.event(&Event::DeepCaptureComplete {
                    session_id: report.session_id.clone(),
                    manifest: "manifest.json".to_string(),
                    status: status.to_string(),
                    cleanup_status: cleanup_status.to_string(),
                    inspectable: report
                        .observations
                        .iter()
                        .filter(|value| value.inspectability == Inspectability::Full)
                        .count() as u64,
                    metadata_only: report
                        .observations
                        .iter()
                        .filter(|value| value.inspectability == Inspectability::MetadataOnly)
                        .count() as u64,
                    unsupported: report
                        .observations
                        .iter()
                        .filter(|value| value.inspectability == Inspectability::Unsupported)
                        .count() as u64,
                });
                if report
                    .failures
                    .iter()
                    .any(|failure| failure.stage == fragcap::deep_capture::Stage::Bundle)
                {
                    emitter.progress("Deep Capture bundle finalization failed");
                } else {
                    emitter.progress(&format!(
                        "Deep Capture bundle written to {}",
                        self.bundle.join("manifest.json").display()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Run `deep-capture` through the public library coordinator.
fn load_client_identity(
    args: &DeepCaptureArgs,
) -> Result<Option<fragcap::deep_capture::ClientIdentity>, CliError> {
    let (Some(certificate_path), Some(private_key_path)) =
        (&args.client_certificate, &args.client_private_key)
    else {
        return Ok(None);
    };
    let mut certificate = fs::read(certificate_path).map_err(|error| {
        CliError::failure(format!(
            "could not read client certificate {}: {error}",
            certificate_path.display()
        ))
    })?;
    let mut private_key = fs::read(private_key_path).map_err(|error| {
        CliError::failure(format!(
            "could not read client private key {}: {error}",
            private_key_path.display()
        ))
    })?;
    let identity = fragcap::deep_capture::ClientIdentity::from_bytes(&certificate, &private_key)
        .map_err(|error| CliError::failure(format!("client identity is invalid: {}", error.code)));
    certificate.fill(0);
    private_key.fill(0);
    identity.map(Some)
}

pub fn run(args: &DeepCaptureArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    if !args.launch {
        return Err(CliError::usage(
            "Deep Capture requires --launch so scoped proxy configuration is owned by the session",
        ));
    }
    let calibration = args.calibrate.map(calibration_phase);
    let deadlines = CalibrationDeadlines::from_args(args);
    if calibration == Some(CalibrationPhase::Reachability)
        && (args.trust_ca || args.har || args.key_log)
    {
        return Err(CliError::usage(
            "reachability calibration does not change trust or produce HAR or TLS key logs",
        ));
    }
    if calibration != Some(CalibrationPhase::Reachability) && !(args.trust_ca || args.yes) {
        return Err(CliError::usage(
            "Deep Capture HTTPS inspection requires explicit CA trust confirmation; pass --trust-ca or --yes",
        ));
    }
    if let Some(root) = paths::deep_capture_session_dir().filter(|path| path.is_dir()) {
        let root = root.canonicalize().map_err(|error| {
            CliError::failure(format!(
                "cannot inspect prior Deep Capture sessions: {error}"
            ))
        })?;
        crate::doctor::fix::recover_deep_capture_journals(&root, &mut std::io::sink()).map_err(
            |errors| {
                CliError::failure(format!(
                    "prior Deep Capture recovery is incomplete; run `fragcap doctor --fix`: {}",
                    errors.join("; ")
                ))
            },
        )?;
    }
    let mode = match calibration {
        None => fragcap::deep_capture::SessionMode::Capture,
        Some(CalibrationPhase::Reachability) => {
            fragcap::deep_capture::SessionMode::ReachabilityCalibration
        }
        Some(CalibrationPhase::Tls) => fragcap::deep_capture::SessionMode::TlsCalibration,
    };
    let target_label = args
        .selector
        .as_deref()
        .or(args.target.as_deref())
        .map(str::to_string)
        .or_else(|| args.id.map(|id| id.to_string()))
        .unwrap_or_default();
    let store = Rc::new(RefCell::new(open_local_store(args.local_db.as_deref())?));
    let restart = run_warm_restart(args, &store.borrow(), emitter)?;
    let pending_session_id = session_id();
    let bundle = bundle_root(args.bundle.as_deref(), &pending_session_id)?;
    let config = fragcap::deep_capture::SessionConfig {
        target: target_label,
        launch_case: args
            .launch_case
            .map(CompatibilityLaunchCase::from)
            .map(library_launch_case),
        mode,
        controlled: args.controlled_target,
        bundle: bundle.clone(),
        trust_ca: calibration != Some(CalibrationPhase::Reachability)
            && (args.trust_ca || args.yes),
        har: args.har,
        key_log: args.key_log,
        client_identity: args.client_certificate.is_some(),
        sensitive_retention: fragcap::deep_capture::SensitiveRetention::Retain,
        deadlines: fragcap::deep_capture::Deadlines {
            launch: deadlines.launch,
            observation: deadlines.observation,
            shutdown: deadlines.shutdown,
            cleanup: deadlines.cleanup,
        },
    };

    let selected = Rc::new(RefCell::new(None));
    let selected_launch_case = Rc::new(RefCell::new(None));
    let runtime = Rc::new(RefCell::new(LibraryRuntime::default()));
    let observation_context = fragcap::deep_capture::NativeObservationContext::default();
    let client_identity = load_client_identity(args)?;
    let emitter = Rc::new(RefCell::new(emitter));
    let mut adapters = fragcap::deep_capture::AdapterSet {
        targets: Box::new(LibraryTargetAdapter {
            args,
            store: Rc::clone(&store),
            selected: Rc::clone(&selected),
            selected_launch_case: Rc::clone(&selected_launch_case),
        }),
        endpoints: Box::new(LibraryEndpointAdapter),
        clock: Box::new(LibraryClockAdapter {
            started: Instant::now(),
        }),
        identifiers: Box::new(LibraryIdentifierAdapter),
        proxy: Box::new({
            let mut proxy = fragcap::deep_capture::NativeProxyAdapter::default()
                .with_observation_context(observation_context.clone())
                .with_application_artifact(bundle.join("application.jsonl"))
                .with_proxy_lifecycle_artifact(bundle.join("proxy.jsonl"))
                .with_key_log_artifact(bundle.join("tls-keylog.log"))
                .with_payload_capture(!args.no_payload);
            if let Some(identity) = client_identity {
                proxy = proxy.with_client_identity(identity);
            }
            proxy
        }),
        trust: Box::new(LibraryTrustAdapter {
            controlled: args.controlled_target,
            runtime: Rc::clone(&runtime),
        }),
        routing: Box::new(fragcap::deep_capture::ChildEnvironmentRouting),
        launch: Box::new(LibraryLaunchAdapter),
        capture: Box::new(LibraryCaptureAdapter {
            args,
            emitter: Rc::clone(&emitter),
            prepared: None,
            runtime: Rc::clone(&runtime),
            observation_context,
            mode,
        }),
        facts: Box::new(LibraryFactAdapter {
            store: Rc::clone(&store),
            selected_launch_case: Rc::clone(&selected_launch_case),
            runtime: Rc::clone(&runtime),
            controlled: args.controlled_target,
        }),
        artifacts: Box::new(LibraryArtifactAdapter {
            emitter: Rc::clone(&emitter),
            selected: Rc::clone(&selected),
            selected_launch_case: Rc::clone(&selected_launch_case),
            runtime: Rc::clone(&runtime),
            deadlines,
        }),
        events: Box::new(LibraryEventAdapter {
            args,
            emitter: Rc::clone(&emitter),
            selected: Rc::clone(&selected),
            selected_launch_case: Rc::clone(&selected_launch_case),
            runtime: Rc::clone(&runtime),
            bundle: bundle.clone(),
        }),
    };
    let prepared = fragcap::deep_capture::DeepCapture::preflight(config, &mut adapters)
        .map_err(cli_error_from_library_refusal)?;

    if let Some(restart) = &restart {
        emitter.borrow_mut().required_human(&format!(
            "Cold Deep Capture plan re-prepared\n  target: {}\n  launch case: {}\n  plan: {}\n  process control: none\n",
            restart.target,
            restart.plan.cold_case().as_str(),
            prepared.plan().id,
        ));
        if !confirm_warm_restart(
            args,
            &mut emitter.borrow_mut(),
            "Authorize this newly prepared cold session? [y/N] ",
        )? {
            emit_restart_outcome(
                &mut emitter.borrow_mut(),
                restart,
                "launch-authorization",
                "declined",
                Some(restart.plan.cold_case()),
                "operator declined the re-prepared cold session; no effects were applied",
            );
            return Err(CliError::usage(
                "the re-prepared cold session was declined; no effects were applied",
            ));
        }
        emit_restart_outcome(
            &mut emitter.borrow_mut(),
            restart,
            "launch-authorization",
            "authorized",
            Some(restart.plan.cold_case()),
            "operator authorized the exact re-prepared cold plan",
        );
    }

    if let Some(phase) = calibration {
        let target = selected
            .borrow()
            .as_ref()
            .expect("preflight retained target")
            .clone();
        let observed_launch_case = selected_launch_case
            .borrow()
            .expect("preflight retained launch case");
        let plan = CalibrationPlan {
            target: target.handle,
            phase,
            declared_launch_case: args
                .launch_case
                .map(CompatibilityLaunchCase::from)
                .expect("clap requires launch-case with calibration"),
            observed_launch_case,
            bundle: bundle.clone(),
            deadlines,
        };
        plan.emit(&mut emitter.borrow_mut());
        if !confirm_calibration(args, &mut emitter.borrow_mut())? {
            emitter
                .borrow_mut()
                .progress("compatibility calibration declined; no effects were applied");
            return Ok(Exit::SUCCESS);
        }
    }

    let authorization = fragcap::deep_capture::Authorization::approved(prepared.plan().id.clone());
    let report = prepared
        .into_session(adapters)
        .run_to_completion(authorization);
    if report.is_complete() {
        Ok(Exit::SUCCESS)
    } else {
        let detail = report
            .snapshot
            .failures
            .first()
            .map(|failure| failure.detail.clone())
            .unwrap_or_else(|| "Deep Capture completed with partial results".to_string());
        Err(CliError::failure(detail))
    }
}

fn cli_error_from_library_refusal(refusal: fragcap::deep_capture::PreflightRefusal) -> CliError {
    match refusal.code.as_str() {
        "launch-case-mismatch"
        | "launch-case"
        | "controlled-target"
        | "compatibility"
        | "routing-prerequisite"
        | "capture-prepare"
        | "bundle-destination"
        | "reachability-tls-options"
        | "trust-not-authorized" => CliError::usage(refusal.detail),
        _ => CliError::failure(refusal.detail),
    }
}

struct DeepCaptureSession {
    session_id: String,
    bundle: PathBuf,
    target: TargetEntry,
    target_id: i64,
    backend: ProxyBackend,
    launch_case: CompatibilityLaunchCase,
    listen_port: u16,
    started_at: SystemTime,
}

#[derive(Clone, Debug)]
struct ProxyBackend {
    name: String,
    version: String,
}

#[derive(Clone)]
struct TrustOutcome {
    state: String,
    action: String,
    thumbprint: Option<String>,
}

#[derive(Clone)]
struct CleanupResource {
    resource: String,
    status: String,
    reason: String,
}

impl CleanupResource {
    fn new(resource: &str, status: &str, reason: &str) -> Self {
        Self {
            resource: resource.to_string(),
            status: status.to_string(),
            reason: reason.to_string(),
        }
    }
}

struct CleanupReport {
    resources: Vec<CleanupResource>,
}

impl CleanupReport {
    fn new(resources: Vec<CleanupResource>) -> Self {
        Self { resources }
    }

    fn status(&self) -> &'static str {
        if self
            .resources
            .iter()
            .any(|resource| resource.status == "failed")
        {
            "failed"
        } else {
            "succeeded"
        }
    }
}

trait TrustManager {
    fn ensure_trusted(&mut self, confirmed: bool) -> Result<TrustOutcome, CliError>;
    fn cleanup(&mut self, timeout: Duration) -> CleanupResource;
}

struct ControlledTrustManager;

impl TrustManager for ControlledTrustManager {
    fn ensure_trusted(&mut self, confirmed: bool) -> Result<TrustOutcome, CliError> {
        if !confirmed {
            return Err(CliError::usage(
                "controlled Deep Capture trust was not confirmed",
            ));
        }
        Ok(TrustOutcome {
            state: "simulated-current-user".to_string(),
            action: "simulated-by-controlled-harness".to_string(),
            thumbprint: Some("controlled-thumbprint".to_string()),
        })
    }

    fn cleanup(&mut self, _timeout: Duration) -> CleanupResource {
        CleanupResource::new(
            "trust-entry",
            "not-needed",
            "controlled trust manager made no operating-system change",
        )
    }
}

#[cfg(windows)]
fn platform_trust_manager(
    der: Vec<u8>,
    thumbprint: String,
) -> Result<Box<dyn TrustManager>, CliError> {
    Ok(Box::new(WindowsCurrentUserTrustManager {
        der,
        thumbprint,
        store: NativeCertificateStore,
        installed_this_session: false,
    }))
}

#[cfg(not(windows))]
fn platform_trust_manager(
    _der: Vec<u8>,
    _thumbprint: String,
) -> Result<Box<dyn TrustManager>, CliError> {
    Err(CliError::failure(
        "Deep Capture current-user CA trust is implemented only on Windows",
    ))
}

#[cfg(windows)]
struct WindowsCurrentUserTrustManager {
    der: Vec<u8>,
    thumbprint: String,
    store: NativeCertificateStore,
    installed_this_session: bool,
}

#[cfg(windows)]
impl TrustManager for WindowsCurrentUserTrustManager {
    fn ensure_trusted(&mut self, confirmed: bool) -> Result<TrustOutcome, CliError> {
        if !confirmed {
            return Err(CliError::usage(
                "Deep Capture CA trust mutation requires explicit confirmation",
            ));
        }
        let thumbprint = self.thumbprint.clone();
        let before = self
            .store
            .observe(&self.der, &thumbprint)
            .map_err(|error| {
                CliError::failure(format!("cannot query current-user CA trust: {error}"))
            })?;
        if before == TrustState::PresentExact {
            return Ok(TrustOutcome {
                state: "current-user-trusted".to_string(),
                action: "already-trusted".to_string(),
                thumbprint: Some(thumbprint),
            });
        }
        if before == TrustState::Mismatch {
            return Err(CliError::failure(
                "the authorized CA thumbprint resolves to different certificate bytes",
            ));
        }
        let mutation = self
            .store
            .add_exact(&self.der, &thumbprint)
            .map_err(|error| {
                CliError::failure(format!("cannot install current-user CA trust: {error}"))
            })?;
        self.installed_this_session = mutation == TrustMutation::Added;
        Ok(TrustOutcome {
            state: "current-user-trusted".to_string(),
            action: if self.installed_this_session {
                "installed-for-session"
            } else {
                "already-trusted"
            }
            .to_string(),
            thumbprint: Some(thumbprint),
        })
    }

    fn cleanup(&mut self, timeout: Duration) -> CleanupResource {
        let started = Instant::now();
        if !self.installed_this_session {
            return CleanupResource::new(
                "trust-entry",
                "not-needed",
                "the session did not install a current-user trust entry",
            );
        }
        if remaining_timeout(started, timeout).is_zero() {
            return CleanupResource::new("trust-entry", "failed", "cleanup deadline expired");
        }
        match self.store.remove_exact(&self.der, &self.thumbprint) {
            Ok(TrustMutation::Removed | TrustMutation::AlreadyAbsent) => {
                self.installed_this_session = false;
                CleanupResource::new(
                    "trust-entry",
                    "succeeded",
                    "session CA removed from the current-user Root store",
                )
            }
            Ok(_) => CleanupResource::new(
                "trust-entry",
                "failed",
                "native trust cleanup returned an unexpected mutation",
            ),
            Err(error) => CleanupResource::new(
                "trust-entry",
                "failed",
                &format!("cannot remove current-user CA trust: {error}"),
            ),
        }
    }
}

fn open_local_store(flag: Option<&Path>) -> Result<Store, CliError> {
    let path = paths::local_db_path(flag)
        .or_else(paths::default_local_db_path)
        .ok_or_else(|| CliError::usage("no local store is available; pass --local-db"))?;
    if !path.is_file() {
        return Err(CliError::usage(format!(
            "the local target store {} does not exist; register a target before Deep Capture",
            path.display()
        )));
    }
    Store::open(&path).map_err(|e| CliError::failure(format!("cannot open local store: {e}")))
}

fn validate_bundle_root(path: &Path) -> Result<(), CliError> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(CliError::usage(format!(
            "the Deep Capture bundle path {} is not a directory",
            path.display()
        )));
    }
    let mut entries = fs::read_dir(path).map_err(|e| {
        CliError::failure(format!(
            "cannot inspect Deep Capture bundle directory {}: {e}",
            path.display()
        ))
    })?;
    if entries.next().is_some() {
        return Err(CliError::usage(format!(
            "the Deep Capture bundle directory {} is not empty",
            path.display()
        )));
    }
    Ok(())
}

fn resolve_target(store: &Store, args: &DeepCaptureArgs) -> Result<TargetEntry, CliError> {
    let selector = args.selector.as_deref().or(args.target.as_deref());
    let selection = match (selector, args.id) {
        (Some(selector), None) => resolve_positional(store, selector),
        (None, Some(id)) => resolve_id(store, id),
        _ => {
            return Err(CliError::usage(
                "exactly one of a target selector, --target, or --id is required",
            ))
        }
    }
    .map_err(|e| CliError::failure(e.to_string()))?;

    match selection {
        Selection::Resolved(t) => Ok(*t),
        Selection::NoMatch => Err(CliError::usage(target_resolve::no_match_message(
            store, selector,
        ))),
        Selection::Ambiguous(matches) => {
            let mut msg = format!(
                "the selector is ambiguous ({} targets match); select by handle or `--id`:",
                matches.len()
            );
            for t in &matches {
                msg.push_str(&format!("\n  {}\t{}\t{}", t.handle, t.stable_id, t.name));
            }
            Err(CliError::usage(msg))
        }
    }
}

fn select_loopback_port() -> Result<u16, CliError> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| CliError::failure(format!("cannot reserve a Deep Capture port: {e}")))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|e| CliError::failure(format!("cannot read the reserved proxy port: {e}")))
}

fn require_controlled_target(target: &TargetEntry) -> Result<(), CliError> {
    if target.handle == CONTROLLED_TARGET_HANDLE && target.stable_id == CONTROLLED_TARGET_STABLE_ID
    {
        Ok(())
    } else {
        Err(CliError::usage(
            "the controlled Deep Capture harness accepts only its reserved synthetic target",
        ))
    }
}

fn bundle_root(flag: Option<&Path>, session_id: &str) -> Result<PathBuf, CliError> {
    if let Some(path) = flag {
        return Ok(path.to_path_buf());
    }
    let root = paths::deep_capture_session_dir().ok_or_else(|| {
        CliError::usage("no Deep Capture session directory is available; pass --bundle")
    })?;
    Ok(root.join(session_id))
}

fn session_id() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("fcap-session-{secs}-{}", std::process::id())
}

fn launch_case(target: &TargetEntry) -> Result<CompatibilityLaunchCase, CliError> {
    if target
        .anchor
        .as_deref()
        .is_some_and(|anchor| anchor.starts_with("steam:"))
    {
        Ok(steam_launch_case(steam_is_running()?))
    } else {
        let launches = entry_windows_launch_entries(target);
        let publisher_declaration = (launches.len() > 1
            && launches.iter().any(|launch| launch.role().is_some()))
            || (launches.len() == 1 && launches[0].role().is_some_and(|role| role != "client"));
        if !publisher_declaration {
            let clients = entry_windows_clients(target);
            let client = match clients.as_slice() {
                [] => {
                    return Err(CliError::usage(
                        "direct Deep Capture requires one resolved Windows client executable",
                    ));
                }
                [client] => client,
                _ => {
                    return Err(CliError::usage(format!(
                        "direct Deep Capture found multiple Windows client executables: {}",
                        clients.join(", ")
                    )));
                }
            };
            return Ok(direct_launch_case(process_image_is_running(client)?));
        }
        match fragcap::managed_launch::prepare_managed_launch(target)
            .map_err(|error| CliError::usage(error.to_string()))?
        {
            fragcap::managed_launch::ManagedLaunch::Publisher(publisher) => {
                publisher_launch_case(&publisher)
            }
            fragcap::managed_launch::ManagedLaunch::Direct(_) => {
                unreachable!("publisher declarations prepare as publisher chains")
            }
            fragcap::managed_launch::ManagedLaunch::Steam(_) => {
                unreachable!("a non-Steam stored target cannot prepare a Steam launch")
            }
            fragcap::managed_launch::ManagedLaunch::Platform(_) => {
                unreachable!("stored publisher preparation never creates a platform plan")
            }
        }
    }
}

fn publisher_launch_case(
    publisher: &fragcap::managed_launch::PublisherChainLaunch,
) -> Result<CompatibilityLaunchCase, CliError> {
    let images: Vec<String> = publisher
        .stages()
        .iter()
        .map(|stage| stage.image_name().to_string_lossy().into_owned())
        .collect();
    let running = process_images_running(&images)?;
    Ok(classify_publisher_processes(&running))
}

fn classify_publisher_processes(running: &[bool]) -> CompatibilityLaunchCase {
    debug_assert!(running.len() >= 2, "publisher chains have root and client");
    if !running[0] && running.iter().skip(1).all(|present| !present) {
        return CompatibilityLaunchCase::PublisherLauncherCold;
    }
    if running[0] && !running[running.len() - 1] {
        return CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm;
    }
    CompatibilityLaunchCase::PublisherLauncherWarm
}

fn effective_launch_case(
    target: &TargetEntry,
    controlled: bool,
) -> Result<CompatibilityLaunchCase, CliError> {
    if controlled {
        Ok(CompatibilityLaunchCase::DirectExeWarm)
    } else {
        launch_case(target)
    }
}

fn steam_launch_case(steam_running: bool) -> CompatibilityLaunchCase {
    if steam_running {
        CompatibilityLaunchCase::SteamProtocolWarm
    } else {
        CompatibilityLaunchCase::SteamProtocolCold
    }
}

fn direct_launch_case(client_running: bool) -> CompatibilityLaunchCase {
    if client_running {
        CompatibilityLaunchCase::DirectExeWarm
    } else {
        CompatibilityLaunchCase::DirectExeCold
    }
}

#[cfg(windows)]
fn steam_is_running() -> Result<bool, CliError> {
    process_image_is_running("steam.exe")
}

#[cfg(windows)]
fn process_image_is_running(image: &str) -> Result<bool, CliError> {
    process_images_running(&[image.to_string()]).map(|running| running[0])
}

#[cfg(windows)]
fn process_images_running(images: &[String]) -> Result<Vec<bool>, CliError> {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let mut running = vec![false; images.len()];

    // This is a handle to a read-only process snapshot, not to any process. It
    // supplies the image names needed to distinguish a cold launch from a warm
    // protocol dispatch without requesting rights against Steam or a target.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE || snapshot == 0 {
        return Err(CliError::failure(format!(
            "cannot determine whether declared target processes are already running: {}",
            std::io::Error::last_os_error()
        )));
    }

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    // SAFETY: the snapshot is live and entry declares its platform structure size.
    if unsafe { Process32FirstW(snapshot, &mut entry) } == 0 {
        let error = std::io::Error::last_os_error();
        // SAFETY: the snapshot handle is closed exactly once before returning.
        unsafe { CloseHandle(snapshot) };
        return Err(CliError::failure(format!(
            "cannot enumerate processes while checking the declared target chain: {error}"
        )));
    }

    loop {
        let end = entry
            .szExeFile
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(entry.szExeFile.len());
        let observed = String::from_utf16_lossy(&entry.szExeFile[..end]);
        for (index, image) in images.iter().enumerate() {
            if observed.eq_ignore_ascii_case(image) {
                running[index] = true;
            }
        }
        // SAFETY: the same live snapshot and initialized entry remain valid.
        if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
            // SAFETY: read immediately after the failed enumeration call.
            let code = unsafe { GetLastError() };
            // SAFETY: the snapshot handle is closed exactly once before returning.
            unsafe { CloseHandle(snapshot) };
            if code == ERROR_NO_MORE_FILES {
                return Ok(running);
            }
            return Err(CliError::failure(format!(
                "cannot finish enumerating processes while checking the declared target chain: {}",
                std::io::Error::from_raw_os_error(code as i32)
            )));
        }
    }
}

#[cfg(not(windows))]
fn steam_is_running() -> Result<bool, CliError> {
    Err(CliError::usage(
        "Deep Capture managed Steam launch is only supported on Windows",
    ))
}

#[cfg(not(windows))]
fn process_image_is_running(_image: &str) -> Result<bool, CliError> {
    Err(CliError::usage(
        "Deep Capture managed direct launch is only supported on Windows",
    ))
}

#[cfg(not(windows))]
fn process_images_running(_images: &[String]) -> Result<Vec<bool>, CliError> {
    Err(CliError::usage(
        "Deep Capture managed publisher launch is only supported on Windows",
    ))
}

fn real_capture_args(
    args: &DeepCaptureArgs,
    bundle: &Path,
    deadlines: CalibrationDeadlines,
) -> CaptureArgs {
    CaptureArgs {
        selector: args.selector.clone(),
        target: args.target.clone(),
        id: args.id,
        process: None,
        path: None,
        path_regex: None,
        catalog_db: args.catalog_db.clone(),
        local_db: args.local_db.clone(),
        out: Some(bundle.join("capture.fcapng")),
        mode: None,
        sink: Vec::new(),
        duration: if args.calibrate.is_some() {
            Some(deadlines.observation)
        } else {
            args.duration
        },
        wait: if args.calibrate.is_some() {
            Some(deadlines.launch)
        } else {
            args.wait
        },
        max_packets: args.max_packets,
        max_bytes: args.max_bytes,
        roles: None,
        scope: ScopeArg::Target,
        direction: Direction::Both,
        interface: args.interface.clone(),
        loopback: true,
        no_payload: args.no_payload,
        ring: None,
        launch: true,
        offline: OfflineArgs::default(),
    }
}

fn owned_platform_wait(current: Option<Duration>, launch_deadline: Duration) -> Option<Duration> {
    Some(current.map_or(launch_deadline, |value| value.min(launch_deadline)))
}

fn run_real_capture(
    capture_args: &CaptureArgs,
    prepared: capture::PreparedCapture,
    flow_registry: Arc<FlowRegistry>,
    emitter: &mut Emitter,
) -> (Result<(), CliError>, Vec<String>, bool) {
    emitter.begin_event_capture();
    let result = capture::run_prepared_with_flow_registry(
        capture_args,
        emitter,
        prepared,
        Arc::clone(&flow_registry),
    );
    let interrupted = result
        .as_ref()
        .is_ok_and(|outcome| outcome.stop_reason == Some(StopReason::Interrupt));
    let result = result.and_then(|outcome| {
        if outcome.exit == Exit::SUCCESS {
            Ok(())
        } else {
            Err(CliError::failure(format!(
                "packet capture ended with exit code {}",
                outcome.exit.code()
            )))
        }
    });
    let process_events = emitter.take_captured_events();
    (result, process_events, interrupted)
}

fn run_controlled_target_harness(
    route: &fragcap::deep_capture::ProxyRoute,
    calibration: Option<CalibrationPhase>,
    execution_timeout: Duration,
    on_started: impl FnOnce(u32),
) -> Result<u32, CliError> {
    let proxy_url = route.proxy_url();
    let executable = std::env::var_os("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or_else(|| CliError::failure("cannot locate the controlled target executable"))?;
    let mut command = std::process::Command::new(executable);
    let (http_origin, https_origin) = route
        .controlled_origins()
        .ok_or_else(|| CliError::failure("native controlled protocol lab is unavailable"))?;
    command
        .arg("__controlled-target")
        .env("HTTP_PROXY", proxy_url)
        .env("HTTPS_PROXY", proxy_url)
        .env("ALL_PROXY", proxy_url)
        .env("NO_PROXY", "")
        .env(
            "FRAGCAP_NATIVE_PROXY_ENDPOINT",
            format!("127.0.0.1:{}", route.endpoint().port),
        )
        .env(
            "FRAGCAP_NATIVE_PROXY_AUTHORIZATION",
            route.proxy_authorization(),
        )
        .env("FRAGCAP_CONTROLLED_HTTP_ORIGIN", http_origin.to_string())
        .env("FRAGCAP_CONTROLLED_HTTPS_ORIGIN", https_origin.to_string())
        .env("FRAGCAP_CONTROLLED_CA_DER", encode_hex(route.ca_der()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if calibration == Some(CalibrationPhase::Reachability) {
        command.env("FRAGCAP_CONTROLLED_REQUEST_LIMIT", "1");
    }
    let mut child = command
        .spawn()
        .map_err(|e| CliError::failure(format!("cannot start controlled target: {e}")))?;
    let process_id = child.id();
    on_started(process_id);
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < execution_timeout => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                return Err(CliError::failure(format!(
                    "controlled target exceeded its {} second combined launch and observation deadline",
                    CalibrationDeadlines::seconds(execution_timeout)
                )));
            }
            Err(err) => {
                return Err(CliError::failure(format!(
                    "cannot wait for controlled target: {err}"
                )));
            }
        }
    };
    if status.success() {
        Ok(process_id)
    } else {
        Err(CliError::failure(format!(
            "controlled target exited with status {status}"
        )))
    }
}

/// Run the hidden placeholder target used by deterministic Deep Capture tests.
pub fn run_controlled_target(_args: &ControlledTargetArgs) -> Result<Exit, CliError> {
    let proxy = std::env::var("HTTP_PROXY")
        .map_err(|_| CliError::failure("controlled target did not inherit HTTP_PROXY"))?;
    for key in ["HTTPS_PROXY", "ALL_PROXY"] {
        if std::env::var(key).as_deref() != Ok(proxy.as_str()) {
            return Err(CliError::failure(format!(
                "controlled target did not inherit {key}"
            )));
        }
    }
    if std::env::var("NO_PROXY").as_deref() != Ok("") {
        return Err(CliError::failure(
            "controlled target did not inherit the session NO_PROXY value",
        ));
    }
    let address = controlled_env_address("FRAGCAP_NATIVE_PROXY_ENDPOINT")?;
    if !address.ip().is_loopback() {
        return Err(CliError::failure(
            "controlled target proxy endpoint is not loopback",
        ));
    }
    let request_limit = std::env::var("FRAGCAP_CONTROLLED_REQUEST_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2);
    let authorization = std::env::var("FRAGCAP_NATIVE_PROXY_AUTHORIZATION")
        .map_err(|_| CliError::failure("controlled target did not inherit proxy authorization"))?;
    let http_origin = controlled_env_address("FRAGCAP_CONTROLLED_HTTP_ORIGIN")?;
    let https_origin = controlled_env_address("FRAGCAP_CONTROLLED_HTTPS_ORIGIN")?;
    let ca_der = decode_hex(
        &std::env::var("FRAGCAP_CONTROLLED_CA_DER")
            .map_err(|_| CliError::failure("controlled target did not inherit the session CA"))?,
    )?;
    fragcap::deep_capture::run_controlled_native_requests(
        address,
        &authorization,
        http_origin,
        https_origin,
        ca_der,
        request_limit > 1,
    )
    .map_err(|error| CliError::failure(format!("controlled native request failed: {error}")))?;
    if std::env::var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|checkpoint| checkpoint <= request_limit)
    {
        return Err(CliError::failure(
            "controlled target stopped at the requested test checkpoint",
        ));
    }
    Ok(Exit::SUCCESS)
}

fn controlled_env_address(key: &str) -> Result<SocketAddr, CliError> {
    std::env::var(key)
        .map_err(|_| CliError::failure(format!("controlled target did not inherit {key}")))?
        .parse()
        .map_err(|_| CliError::failure(format!("controlled target received invalid {key}")))
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, CliError> {
    if value.len() & 1 == 1 {
        return Err(CliError::failure(
            "controlled target received invalid CA material",
        ));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|text| u8::from_str_radix(text, 16).ok())
                .ok_or_else(|| CliError::failure("controlled target received invalid CA material"))
        })
        .collect()
}

struct BundleContext<'a> {
    session: &'a DeepCaptureSession,
    controlled: bool,
    har_requested: bool,
    key_log_requested: bool,
    sensitive_retention: fragcap::deep_capture::SensitiveRetention,
    observations: &'a [Observation],
    trust: &'a TrustOutcome,
    session_state: &'a str,
    controlled_process_id: Option<u32>,
    process_events: &'a [String],
    calibration: Option<CalibrationPhase>,
    calibration_outcome: Option<CalibrationOutcome>,
    deadlines: CalibrationDeadlines,
    fact_writes: &'a [FactWriteResult],
}

fn write_bundle(ctx: &BundleContext<'_>, emitter: &mut Emitter) -> Result<(), CliError> {
    let packet_truth = ctx.session.bundle.join("capture.fcapng");
    if ctx.controlled {
        write_controlled_pcapng(&packet_truth, ctx.observations)?;
    } else if !packet_truth.is_file() && ctx.session_state == "complete" {
        return Err(CliError::failure(format!(
            "packet capture did not produce {}",
            packet_truth.display()
        )));
    }
    let packet_truth_produced = packet_truth.is_file();
    let application = ctx.session.bundle.join("application.jsonl");
    if !application.is_file() {
        write_file(
            application.clone(),
            application_jsonl(
                &ctx.session.session_id,
                ctx.session.target_id,
                ctx.observations,
                ctx.session_state,
            )
            .as_bytes(),
        )?;
    }
    let mut har_produced = false;
    let mut har_partial = false;
    let mut har_omission_reason = "not-requested";
    if ctx.har_requested {
        match fragcap::deep_capture::project_application_har(&application) {
            Ok(projection) if projection.standard_entries + projection.partial_entries > 0 => {
                har_partial = projection.partial_entries > 0;
                match projection.publish(&ctx.session.bundle.join("http.har")) {
                    Ok(_) => har_produced = true,
                    Err(_) => har_omission_reason = "writer-failed",
                }
            }
            Ok(_) => har_omission_reason = "no-http-semantics",
            Err(_) => har_omission_reason = "projection-failed",
        }
    }
    write_file(
        ctx.session.bundle.join("process-trace.jsonl"),
        process_trace_jsonl(
            &ctx.session.session_id,
            ctx.controlled_process_id,
            ctx.process_events,
        )
        .as_bytes(),
    )?;
    write_file(
        ctx.session.bundle.join("compatibility.json"),
        compatibility_json(ctx)?.as_bytes(),
    )?;
    let key_log_path = ctx.session.bundle.join("tls-keylog.log");
    let key_log_produced = key_log_path
        .metadata()
        .is_ok_and(|metadata| metadata.len() > 0);
    if key_log_path.is_file() && !key_log_produced {
        fs::remove_file(&key_log_path).map_err(|error| {
            CliError::failure(format!(
                "empty TLS key-log cleanup failed for {}: {error}",
                key_log_path.display()
            ))
        })?;
    }
    let final_cleanup = cleanup_from_lifecycle(&ctx.session.bundle.join("cleanup.jsonl"))?;
    write_file(
        ctx.session.bundle.join("cleanup.json"),
        cleanup_json(&ctx.session.session_id, &final_cleanup)?.as_bytes(),
    )?;
    let manifest = manifest_json(
        ctx,
        &final_cleanup,
        har_produced,
        har_partial,
        har_omission_reason,
        key_log_produced,
        packet_truth_produced,
    )?;
    fragcap::deep_capture::publish_final(&ctx.session.bundle, manifest.as_bytes())
        .map_err(|error| CliError::failure(format!("cannot publish final manifest: {error}")))?;

    let mut produced_artifacts = vec![
        ("application-jsonl", "sensitive"),
        ("proxy-lifecycle", "sensitive"),
        ("cleanup-lifecycle", "ordinary"),
        ("resource-journal", "secret-adjacent"),
        ("process-trace", "sensitive"),
        ("compatibility", "ordinary"),
        ("cleanup-summary", "ordinary"),
        ("manifest", "ordinary"),
    ];
    if packet_truth_produced {
        produced_artifacts.insert(0, ("pcapng", "ordinary"));
    }
    for (role, sensitivity) in produced_artifacts {
        emitter.event(&Event::DeepCaptureBundle {
            session_id: ctx.session.session_id.clone(),
            role: role.to_string(),
            path: artifact_path(role).to_string(),
            sensitivity: sensitivity.to_string(),
            required: true,
        });
    }
    if har_produced {
        emitter.event(&Event::DeepCaptureBundle {
            session_id: ctx.session.session_id.clone(),
            role: "har".to_string(),
            path: "http.har".to_string(),
            sensitivity: "sensitive".to_string(),
            required: false,
        });
    }
    if key_log_produced {
        emitter.event(&Event::DeepCaptureBundle {
            session_id: ctx.session.session_id.clone(),
            role: "tls-key-log".to_string(),
            path: "tls-keylog.log".to_string(),
            sensitivity: "secret-adjacent".to_string(),
            required: false,
        });
    }
    for resource in &final_cleanup.resources {
        emitter.event(&Event::DeepCaptureCleanup {
            session_id: ctx.session.session_id.clone(),
            resource: resource.resource.clone(),
            status: resource.status.clone(),
            reason: resource.reason.clone(),
        });
    }
    Ok(())
}

fn cleanup_from_lifecycle(path: &Path) -> Result<CleanupReport, CliError> {
    let prefix = fragcap::deep_capture::read_lifecycle_prefix(path).map_err(|error| {
        CliError::failure(format!(
            "cannot read cleanup chronology {}: {error}",
            path.display()
        ))
    })?;
    if prefix.status != fragcap::deep_capture::LifecycleStreamStatus::Complete {
        return Err(CliError::failure(format!(
            "cleanup chronology {} has no reconciling trailer",
            path.display()
        )));
    }
    let mut resources = std::collections::BTreeMap::new();
    for record in prefix
        .records
        .iter()
        .filter(|record| record["type"] == "cleanup.result")
    {
        let fields = &record["fields"];
        let Some(resource) = fields["resource_id"].as_str() else {
            continue;
        };
        let state = fields["state"].as_str().unwrap_or("failed");
        let status = match state {
            "released" | "not-applied" => "succeeded",
            "retained" => "retained",
            "timed-out" => "timed-out",
            _ => "failed",
        };
        resources.insert(
            resource.to_string(),
            CleanupResource::new(
                resource,
                status,
                fields["detail"]
                    .as_str()
                    .unwrap_or("cleanup result lacked detail"),
            ),
        );
    }
    for record in prefix
        .records
        .iter()
        .filter(|record| record["type"] == "cleanup.adapter-result")
    {
        let fields = &record["fields"];
        let Some(resource) = fields["resource_id"].as_str() else {
            continue;
        };
        resources.insert(
            resource.to_string(),
            CleanupResource::new(
                resource,
                fields["status"].as_str().unwrap_or("failed"),
                fields["reason"]
                    .as_str()
                    .unwrap_or("cleanup result lacked reason"),
            ),
        );
    }
    if resources.is_empty() {
        return Err(CliError::failure(
            "cleanup chronology contains no terminal results",
        ));
    }
    Ok(CleanupReport::new(resources.into_values().collect()))
}

fn artifact_path(role: &str) -> &'static str {
    match role {
        "pcapng" => "capture.fcapng",
        "application-jsonl" => "application.jsonl",
        "proxy-lifecycle" => "proxy.jsonl",
        "cleanup-lifecycle" => "cleanup.jsonl",
        "resource-journal" => "resource-journal.jsonl",
        "process-trace" => "process-trace.jsonl",
        "compatibility" => "compatibility.json",
        "cleanup-summary" => "cleanup.json",
        "manifest" => "manifest.json",
        _ => "",
    }
}

fn write_file(path: PathBuf, bytes: &[u8]) -> Result<(), CliError> {
    fs::write(&path, bytes)
        .map_err(|e| CliError::failure(format!("cannot write {}: {e}", path.display())))
}

fn application_jsonl(
    session_id: &str,
    target_id: i64,
    observations: &[Observation],
    writer_status: &str,
) -> String {
    let mut out = String::new();
    out.push_str(
        &json!({
            "type": "application.header",
            "session_id": session_id,
            "manifest_version": 1,
        })
        .to_string(),
    );
    out.push('\n');
    for observation in observations {
        let flow_id = observation.flow_id.map(|flow_id| flow_id.to_string());
        let has_http = observation.method.is_some() && observation.url.is_some();
        let record_type = if has_http {
            "application.http"
        } else if observation.inspectability == Inspectability::Unsupported {
            "application.unsupported"
        } else {
            "application.metadata"
        };
        let correlation_reason = observation.flow_id.is_none().then_some(
            "flow correlation unavailable: proxy endpoints were absent or not present in packet truth",
        );
        let reason = match (observation.reason.as_deref(), correlation_reason) {
            (Some(observation), Some(correlation)) => Some(format!("{observation}; {correlation}")),
            (Some(observation), None) => Some(observation.to_string()),
            (None, Some(correlation)) => Some(correlation.to_string()),
            (None, None) => None,
        };
        let line = json!({
            "type": record_type,
            "session_id": session_id,
            "target_id": target_id,
            "flow_id": flow_id,
            "proxy_connection_id": observation.proxy_connection_id,
            "started_at": observation.observed_at,
            "ended_at": observation.observed_at,
            "direction": "outbound",
            "protocol": observation.protocol,
            "inspectability": observation.inspectability.as_str(),
            "process_id": observation.process_id,
            "process_image": observation.process_image,
            "role": observation.role,
            "attribution": observation.attribution.as_deref().unwrap_or_else(|| if observation.flow_id.is_some() { "packet-flow-only" } else { "proxy-only" }),
            "packet_observations": observation.packet_observations,
            "packet_observations_unretained": observation.packet_observations_unretained,
            "correlation_state": observation.correlation_state.as_str(),
            "correlation_reason": observation.correlation_reason,
            "http": has_http.then(|| json!({
                "method": observation.method,
                "url": observation.url,
                "status": observation.status,
            })),
            "reason": reason,
        });
        out.push_str(&line.to_string());
        out.push('\n');
    }
    out.push_str(
        &json!({
            "type": "application.trailer",
            "session_id": session_id,
            "records": observations.len(),
            "writer_status": writer_status,
        })
        .to_string(),
    );
    out.push('\n');
    out
}

fn process_trace_jsonl(
    session_id: &str,
    controlled_process_id: Option<u32>,
    captured_events: &[String],
) -> String {
    if let Some(process_id) = controlled_process_id {
        return json!({
            "session_id": session_id,
            "event": "controlled-harness.exited",
            "pid": process_id,
            "process": "client.exe",
            "role": "client",
            "reason": "deterministic placeholder child completed"
        })
        .to_string()
            + "\n";
    }
    let mut output = String::new();
    for line in captured_events {
        let Ok(mut event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(kind) = event.get("event").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if !matches!(kind, "stage.matched" | "stage.exited") {
            continue;
        }
        if let Some(object) = event.as_object_mut() {
            object.insert("session_id".to_string(), json!(session_id));
        }
        output.push_str(&event.to_string());
        output.push('\n');
    }
    if output.is_empty() {
        output.push_str(
            &json!({
            "session_id": session_id,
            "event": "process-trace.unavailable",
            "pid": serde_json::Value::Null,
            "process": serde_json::Value::Null,
            "role": "unknown",
            "reason": "no stage lifecycle event was observed; packet attribution remains authoritative"
        })
            .to_string(),
        );
        output.push('\n');
    }
    output
}

fn compatibility_json(ctx: &BundleContext<'_>) -> Result<String, CliError> {
    serde_json::to_string_pretty(&json!({
        "session_id": ctx.session.session_id,
        "target": {
            "id": ctx.session.target.id,
            "handle": ctx.session.target.handle,
        },
        "launch_case": ctx.session.launch_case.as_str(),
        "proxy_backend": ctx.session.backend.name,
        "proxy_backend_version": ctx.session.backend.version,
        "calibration": ctx.calibration.map(|phase| json!({
            "phase": phase.as_str(),
            "outcome": ctx.calibration_outcome.map(|outcome| outcome.to_string()),
            "trust_action": if phase == CalibrationPhase::Tls {
                "session-owned-current-user-ca"
            } else {
                "none"
            },
            "proxy_mode": "launch-scoped-env",
            "system_proxy_change": false,
            "published": false,
            "deadlines_seconds": {
                "launch": CalibrationDeadlines::seconds(ctx.deadlines.launch),
                "observation": CalibrationDeadlines::seconds(ctx.deadlines.observation),
                "shutdown": CalibrationDeadlines::seconds(ctx.deadlines.shutdown),
                "cleanup": CalibrationDeadlines::seconds(ctx.deadlines.cleanup),
            },
        })),
        "fact_writes": ctx.fact_writes.iter().map(|write| json!({
            "key": write.key,
            "value": write.value,
            "status": write.status,
            "reason": write.reason,
        })).collect::<Vec<_>>(),
        "observations": ctx.observations.iter().map(|o| {
            json!({
                "flow_id": o.flow_id.map(|id| id.to_string()),
                "proxy_connection_id": o.proxy_connection_id,
                "process_id": o.process_id,
                "process_image": o.process_image,
                "role": o.role,
                "attribution": o.attribution,
                "packet_observations": o.packet_observations,
                "packet_observations_unretained": o.packet_observations_unretained,
                "correlation_state": o.correlation_state.as_str(),
                "correlation_reason": o.correlation_reason,
                "protocol": o.protocol,
                "inspectability": o.inspectability.as_str(),
            })
        }).collect::<Vec<_>>(),
    }))
    .map_err(|e| CliError::failure(e.to_string()))
}

fn cleanup_json(session_id: &str, cleanup: &CleanupReport) -> Result<String, CliError> {
    serde_json::to_string_pretty(&json!({
        "session_id": session_id,
        "status": cleanup.status(),
        "resources": cleanup.resources.iter().map(|resource| json!({
            "resource": resource.resource,
            "status": resource.status,
            "reason": resource.reason,
        })).collect::<Vec<_>>(),
    }))
    .map_err(|e| CliError::failure(e.to_string()))
}

fn manifest_json(
    ctx: &BundleContext<'_>,
    cleanup: &CleanupReport,
    har_produced: bool,
    har_partial: bool,
    har_omission_reason: &str,
    key_log_produced: bool,
    packet_truth_produced: bool,
) -> Result<String, CliError> {
    let key_log_cleanup_failed = cleanup
        .resources
        .iter()
        .any(|resource| resource.resource == "tls-key-log" && resource.status == "failed");
    let key_log_state = match (
        ctx.key_log_requested,
        key_log_produced,
        key_log_cleanup_failed,
    ) {
        (false, _, _) => "not-requested",
        (true, false, true) => "failed",
        (true, false, false) => "empty",
        (true, true, true) => "partial",
        (true, true, false) => "retained",
    };
    let mut application_artifact = artifact(
        "application-jsonl",
        "application.jsonl",
        "application-events",
        "sensitive",
        "application/x-ndjson",
        true,
    );
    apply_application_truth(
        &mut application_artifact,
        &ctx.session.bundle.join("application.jsonl"),
    );
    let mut artifacts = vec![
        application_artifact,
        artifact(
            "proxy-lifecycle",
            "proxy.jsonl",
            "proxy-lifecycle-events",
            "sensitive",
            "application/x-ndjson",
            true,
        ),
        artifact(
            "cleanup-lifecycle",
            "cleanup.jsonl",
            "cleanup-lifecycle-events",
            "ordinary",
            "application/x-ndjson",
            true,
        ),
        artifact(
            "resource-journal",
            "resource-journal.jsonl",
            "resource-ownership-journal",
            "secret-adjacent",
            "application/x-ndjson",
            true,
        ),
        artifact(
            "process-trace",
            "process-trace.jsonl",
            "process-events",
            "sensitive",
            "application/x-ndjson",
            true,
        ),
        artifact(
            "compatibility",
            "compatibility.json",
            "compatibility-updates",
            "ordinary",
            "application/json",
            true,
        ),
        artifact(
            "cleanup-summary",
            "cleanup.json",
            "cleanup-projection",
            "ordinary",
            "application/json",
            true,
        ),
        artifact(
            "manifest",
            "manifest.json",
            "bundle-index",
            "ordinary",
            "application/json",
            true,
        ),
    ];
    let mut omissions = Vec::new();
    if packet_truth_produced {
        let mut packet_artifact = artifact(
            "pcapng",
            "capture.fcapng",
            "packet-truth",
            "ordinary",
            "application/x-pcapng",
            true,
        );
        packet_artifact["loss"] = json!({"state":"reported-in-artifact"});
        artifacts.insert(0, packet_artifact);
    } else {
        omissions.push(json!({"role":"pcapng","reason":"writer-failed","severity":"error"}));
        artifacts.push(omitted_artifact(
            "pcapng",
            "packet-truth",
            "ordinary",
            "application/x-pcapng",
            true,
            "writer-failed",
        ));
    }
    if har_produced {
        let mut har = artifact(
            "har",
            "http.har",
            "http-projection",
            "sensitive",
            "application/json",
            false,
        );
        if har_partial {
            har["completeness"] = json!("partial");
        }
        artifacts.push(har);
    } else if ctx.har_requested {
        omissions.push(json!({"role":"har","reason":har_omission_reason,"severity":if har_omission_reason == "no-http-semantics" {"info"} else {"error"}}));
        artifacts.push(omitted_artifact(
            "har",
            "http-projection",
            "sensitive",
            "application/json",
            false,
            har_omission_reason,
        ));
    } else {
        omissions.push(json!({"role":"har","reason":"not-requested","severity":"info"}));
        artifacts.push(omitted_artifact(
            "har",
            "http-projection",
            "sensitive",
            "application/json",
            false,
            "not-requested",
        ));
    }
    if ctx.key_log_requested && key_log_produced {
        artifacts.push(artifact(
            "tls-key-log",
            "tls-keylog.log",
            "analyzer-aid",
            "secret-adjacent",
            "text/plain",
            false,
        ));
    } else if ctx.key_log_requested {
        omissions.push(json!({"role":"tls-key-log","reason":"not-produced","severity":"warn"}));
        artifacts.push(omitted_artifact(
            "tls-key-log",
            "analyzer-aid",
            "secret-adjacent",
            "text/plain",
            false,
            "writer-failed",
        ));
    } else {
        omissions.push(json!({"role":"tls-key-log","reason":"not-requested","severity":"info"}));
        artifacts.push(omitted_artifact(
            "tls-key-log",
            "analyzer-aid",
            "secret-adjacent",
            "text/plain",
            false,
            "not-requested",
        ));
    }
    let manifest_state = if ctx.session_state == "complete"
        && (artifacts.iter().any(|artifact| {
            matches!(
                artifact["completeness"].as_str(),
                Some("partial" | "truncated" | "failed" | "pending")
            ) || (artifact["required"] == true && artifact["completeness"] == "omitted")
        }) || omissions
            .iter()
            .any(|omission| omission["severity"] == "error"))
    {
        "partial"
    } else {
        ctx.session_state
    };
    serde_json::to_string_pretty(&json!({
        "$schema": "https://fragcap.dev/schema/deep-capture-manifest.v2.json",
        "manifest_version": 2,
        "product": {
            "name": "fragcap",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "session_id": ctx.session.session_id,
        "mode": "deep-capture",
        "state": manifest_state,
        "target": {
            "id": ctx.session.target.id,
            "stable_id": ctx.session.target.stable_id,
            "handle": ctx.session.target.handle,
        },
        "started_at": rfc3339_utc(ctx.session.started_at),
        "stopped_at": rfc3339_utc(SystemTime::now()),
        "proxy": {
            "backend": ctx.session.backend.name,
            "version": ctx.session.backend.version,
            "mode": "launch-scoped-env",
            "listen_addr": "127.0.0.1",
            "listen_port": ctx.session.listen_port,
        },
        "trust": {
            "state": ctx.trust.state,
            "action": ctx.trust.action,
            "thumbprint": ctx.trust.thumbprint,
        },
        "launch": {
            "case": ctx.session.launch_case.as_str(),
            "scoped_proxy": true,
        },
        "artifacts": artifacts,
        "omissions": omissions,
        "correlation": {
            "flow_ids": ctx.observations
                .iter()
                .filter_map(|observation| observation.flow_id.map(|flow_id| flow_id.to_string()))
                .collect::<Vec<_>>(),
            "process_roles": if ctx.controlled { json!(["client"]) } else { json!([]) },
        },
        "cleanup": {
            "status": cleanup.status(),
            "report": "cleanup.json",
            "updated_at": rfc3339_utc(SystemTime::now()),
        },
        "sensitive_artifacts": {
            "retention": ctx.sensitive_retention.as_str(),
            "cleanup_command": "fragcap bundle cleanup <bundle> --yes",
            "journal": ".sensitive-actions.jsonl",
            "tls_key_log": {
                "requested": ctx.key_log_requested,
                "state": key_log_state,
            },
        },
    }))
    .map_err(|e| CliError::failure(e.to_string()))
}

fn artifact(
    role: &str,
    path: &str,
    authority: &str,
    sensitivity: &str,
    content_type: &str,
    required: bool,
) -> serde_json::Value {
    json!({
        "role": role,
        "path": path,
        "authority": authority_contract(authority),
        "sensitivity": sensitivity,
        "content_type": content_type,
        "required": required,
        "finalization": "complete",
        "completeness": "complete",
        "loss": {"state": "none"},
        "correlation": if role == "application-jsonl" || role == "har" || role == "pcapng" {
            json!({"state": "partial", "reason": "reported-by-correlation-accounting"})
        } else {
            json!({"state": "not-applicable"})
        },
    })
}

fn omitted_artifact(
    role: &str,
    authority: &str,
    sensitivity: &str,
    content_type: &str,
    required: bool,
    reason: &str,
) -> serde_json::Value {
    json!({
        "role": role,
        "authority": authority_contract(authority),
        "sensitivity": sensitivity,
        "content_type": content_type,
        "required": required,
        "finalization": "complete",
        "completeness": "omitted",
        "omission_reason": reason,
        "loss": {"state":"not-applicable"},
        "correlation": {"state":"not-applicable"},
    })
}

fn authority_contract(authority: &str) -> serde_json::Value {
    let (kind, source_role) = match authority {
        "packet-truth" | "application-events" | "proxy-lifecycle-events" => {
            ("primary-evidence", None)
        }
        "http-projection" => ("derived-projection", Some("application-jsonl")),
        "cleanup-projection" => ("derived-projection", Some("cleanup-lifecycle")),
        "bundle-index" => ("bundle-index", None),
        "analyzer-aid" => ("analyzer-aid", None),
        _ => ("operational-record", None),
    };
    json!({"kind":kind,"owner":authority,"source_role":source_role})
}

fn apply_application_truth(artifact: &mut serde_json::Value, path: &Path) {
    let Ok(prefix) = fragcap::deep_capture::read_application_prefix(path) else {
        artifact["finalization"] = json!("failed");
        artifact["completeness"] = json!("failed");
        artifact["loss"] = json!({"state":"unknown"});
        artifact["correlation"] =
            json!({"state":"unavailable","reason":"application-stream-unreadable"});
        return;
    };
    let finalization = match prefix.status {
        fragcap::deep_capture::ApplicationStreamStatus::Complete => "complete",
        fragcap::deep_capture::ApplicationStreamStatus::Incomplete => "incomplete",
        fragcap::deep_capture::ApplicationStreamStatus::UnknownVersion => "failed",
    };
    let trailer = prefix
        .records
        .last()
        .filter(|record| record["type"] == "application.trailer");
    let dropped = trailer
        .and_then(|record| record["dropped_records"].as_u64())
        .unwrap_or(0);
    let truncated = trailer
        .map(|record| {
            record["body_bytes_truncated"].as_u64().unwrap_or(0)
                + record["streaming_bytes_truncated"].as_u64().unwrap_or(0)
        })
        .unwrap_or(0);
    let unretained_connections = trailer
        .and_then(|record| record["correlation_connections_unretained"].as_u64())
        .unwrap_or(0);
    let correlations: Vec<_> = prefix
        .records
        .iter()
        .filter(|record| record["type"] == "application.correlation")
        .collect();
    let correlation_state = application_correlation_state(&correlations, unretained_connections);
    artifact["finalization"] = json!(finalization);
    artifact["completeness"] = json!(application_completeness(
        finalization,
        dropped,
        truncated,
        unretained_connections,
    ));
    artifact["loss"] = if dropped == 0 && truncated == 0 && unretained_connections == 0 {
        json!({"state":"none"})
    } else {
        json!({"state":"observed","dropped_records":dropped,"truncated_bytes":truncated,"unretained_connections":unretained_connections})
    };
    artifact["correlation"] = json!({"state":correlation_state,"records":correlations.len()});
}

fn application_correlation_state(
    correlations: &[&serde_json::Value],
    unretained_connections: u64,
) -> &'static str {
    if unretained_connections > 0 {
        "partial"
    } else if correlations.is_empty() {
        "unavailable"
    } else if correlations
        .iter()
        .all(|record| record["correlation_state"] == "matched")
    {
        "complete"
    } else {
        "partial"
    }
}

fn application_completeness(
    finalization: &str,
    dropped: u64,
    truncated: u64,
    unretained_connections: u64,
) -> &'static str {
    if finalization != "complete" {
        "partial"
    } else if truncated > 0 {
        "truncated"
    } else if dropped > 0 || unretained_connections > 0 {
        "partial"
    } else {
        "complete"
    }
}

struct FactWriteResult {
    key: String,
    value: String,
    status: String,
    reason: Option<String>,
}

fn insert_fact(
    store: &mut Store,
    key: CompatibilityFactKey,
    value: &str,
    ctx: FactContext<'_>,
) -> Result<(), CliError> {
    let mut fact = CompatibilityFact::new(
        ctx.target_id,
        key,
        value,
        CompatibilityEvidenceSource::ObservedRun,
    )
    .map_err(|e| CliError::failure(e.to_string()))?;
    fact.launch_case = Some(ctx.launch_case);
    fact.observed_at = Some(rfc3339_utc(SystemTime::now()));
    fact.fragcap_version = Some(env!("CARGO_PKG_VERSION").to_string());
    fact.proxy_backend = Some(ctx.backend.name.clone());
    fact.proxy_backend_version = Some(ctx.backend.version.clone());
    fact.proxy_mode = Some("launch-scoped-env".to_string());
    fact.final_owner_executable = ctx
        .final_owner
        .and_then(|owner| owner.process_image.clone());
    fact.final_owner_handoff = ctx
        .final_owner
        .and_then(|owner| owner.attribution.as_deref())
        .is_some_and(|attribution| attribution.contains("handoff"));
    fact.note = Some(format!(
        "scrubbed Deep Capture {} observation{}",
        ctx.phase.as_str(),
        if ctx.controlled {
            " from controlled target"
        } else {
            ""
        }
    ));
    store
        .insert_compatibility_fact(&fact)
        .map_err(|e| CliError::failure(format!("cannot write Deep Capture facts: {e}")))?;
    Ok(())
}

struct FactContext<'a> {
    target_id: i64,
    launch_case: CompatibilityLaunchCase,
    backend: &'a ProxyBackend,
    controlled: bool,
    final_owner: Option<&'a Observation>,
    phase: CalibrationPhase,
}

fn write_controlled_pcapng(path: &Path, observations: &[Observation]) -> Result<(), CliError> {
    let file = fs::File::create(path).map_err(|e| {
        CliError::failure(format!(
            "cannot create controlled packet truth {}: {e}",
            path.display()
        ))
    })?;
    let mut writer = PcapngWriter::new(file)
        .map_err(|e| CliError::failure(format!("cannot start controlled pcapng: {e}")))?;
    writer
        .declare_interface(&InterfaceDeclaration::new(
            LinkType::ETHERNET,
            65_535,
            "controlled-loopback",
        ))
        .map_err(|e| CliError::failure(format!("cannot declare controlled interface: {e}")))?;

    for (index, _observation) in observations.iter().enumerate() {
        let ordinal = u16::try_from(index + 1).expect("the controlled corpus has four records");
        let raw = RawPacket::new(
            Timestamp::from_parts(1, u32::from(ordinal) * 1_000),
            Payload::from(vec![0u8; 60]),
            60,
        );
        let mut packet = CapturedPacket::from_raw(raw, InterfaceId::default());
        let endpoint_a: SocketAddr = format!("127.0.0.1:{}", 8_000 + ordinal)
            .parse()
            .expect("controlled endpoint parses");
        let endpoint_b: SocketAddr = format!("127.0.0.1:{}", 40_000 + ordinal)
            .parse()
            .expect("controlled endpoint parses");
        packet.flow = Some(FlowKey::new(Proto::Tcp, endpoint_a, endpoint_b));
        packet.flow_id = fragcap::FlowId::new(index as u64 + 1);
        writer
            .write(&packet)
            .map_err(|e| CliError::failure(format!("cannot write controlled packet: {e}")))?;
    }
    Box::new(writer)
        .finish(&CaptureStats::default())
        .map_err(|e| CliError::failure(format!("cannot finish controlled pcapng: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publisher_process_inventory_keeps_cold_and_warm_states_distinct() {
        assert_eq!(
            classify_publisher_processes(&[false, false, false]),
            CompatibilityLaunchCase::PublisherLauncherCold
        );
        assert_eq!(
            classify_publisher_processes(&[true, false, false]),
            CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm
        );
        assert_eq!(
            classify_publisher_processes(&[true, true, false]),
            CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm
        );
        assert_eq!(
            classify_publisher_processes(&[false, true, false]),
            CompatibilityLaunchCase::PublisherLauncherWarm
        );
        assert_eq!(
            classify_publisher_processes(&[false, false, true]),
            CompatibilityLaunchCase::PublisherLauncherWarm
        );
    }

    #[test]
    fn direct_process_inventory_returns_typed_warm_and_cold_states() {
        assert_eq!(
            direct_launch_case(true),
            CompatibilityLaunchCase::DirectExeWarm
        );
        assert_eq!(
            direct_launch_case(false),
            CompatibilityLaunchCase::DirectExeCold
        );
    }

    #[test]
    fn steam_restart_waits_for_platform_and_declared_client_images() {
        assert_eq!(
            platform_restart_images(vec!["Game.exe".to_string()]),
            ["steam.exe", "Game.exe"]
        );
    }

    #[test]
    fn changed_launch_authority_is_not_the_same_target_plan() {
        let original = WarmRestartAuthority {
            stable_id: 1,
            anchor: Some("steam:1".to_string()),
            install_root: Some("C:\\game".to_string()),
            launch_entries: Some(serde_json::json!([
                {"os": "windows", "executable": "Game.exe"}
            ])),
        };

        let mut changed = original.clone();
        changed.install_root = Some("C:\\changed".to_string());
        assert_ne!(changed, original);

        let mut changed = original.clone();
        changed.launch_entries = Some(serde_json::json!([
            {"os": "windows", "executable": "Changed.exe"}
        ]));
        assert_ne!(changed, original);
    }

    fn observation() -> Observation {
        Observation {
            flow_id: FlowId::new(1),
            proxy_connection_id: "proxy-test".to_string(),
            client_peer: None,
            proxy_local: None,
            observed_at: "2026-01-01T00:00:00Z".to_string(),
            process_id: None,
            process_image: None,
            role: None,
            attribution: None,
            packet_observations: 1,
            packet_observations_unretained: 0,
            correlation_state: fragcap::deep_capture::CorrelationState::FlowOnly,
            correlation_reason: "test-flow".to_string(),
            protocol: "https".to_string(),
            inspectability: Inspectability::Full,
            method: Some("GET".to_string()),
            url: Some("https://127.0.0.1/controlled".to_string()),
            status: Some(200),
            reason: None,
        }
    }

    #[test]
    fn calibration_classification_keeps_outcomes_distinct() {
        let mut client = observation();
        client.role = Some("client".to_string());
        assert_eq!(
            calibration_outcome(CalibrationPhase::Reachability, &[client.clone()]),
            CalibrationOutcome::ReachedClient
        );

        let mut launcher = client.clone();
        launcher.role = Some("launcher".to_string());
        assert_eq!(
            calibration_outcome(CalibrationPhase::Reachability, &[launcher.clone()]),
            CalibrationOutcome::LauncherOnly
        );
        let mut platform = client.clone();
        platform.role = Some("platform".to_string());
        assert_eq!(
            calibration_outcome(CalibrationPhase::Reachability, &[platform.clone()]),
            CalibrationOutcome::LauncherOnly,
            "owned platform traffic does not prove final-client routing"
        );

        let mut escaped = client.clone();
        escaped.flow_id = None;
        escaped.role = None;
        escaped.reason = Some("escaped-tree".to_string());
        assert_eq!(
            calibration_outcome(CalibrationPhase::Reachability, &[escaped]),
            CalibrationOutcome::EscapedTree
        );
        assert_eq!(
            calibration_outcome(CalibrationPhase::Reachability, &[]),
            CalibrationOutcome::Inconclusive
        );
        let mut proxy_not_reached = observation();
        proxy_not_reached.flow_id = None;
        proxy_not_reached.role = None;
        proxy_not_reached.reason = Some("proxy-not-reached".to_string());
        assert_eq!(
            calibration_outcome(CalibrationPhase::Reachability, &[proxy_not_reached]),
            CalibrationOutcome::ProxyNotReached
        );

        assert_eq!(
            calibration_outcome(CalibrationPhase::Tls, &[client.clone()]),
            CalibrationOutcome::LocalCaAccepted
        );
        assert!(!observation_proves_final_client_ca_acceptance(&launcher));
        assert!(!observation_proves_final_client_ca_acceptance(&platform));
        assert_eq!(
            calibration_outcome(CalibrationPhase::Tls, &[launcher]),
            CalibrationOutcome::UnknownTrust
        );
        let mut pinned = client.clone();
        pinned.inspectability = Inspectability::Inconclusive;
        pinned.reason = Some("certificate-pinned".to_string());
        assert_eq!(
            calibration_outcome(CalibrationPhase::Tls, &[pinned]),
            CalibrationOutcome::CertificatePinned
        );
        let mut metadata = client.clone();
        metadata.inspectability = Inspectability::MetadataOnly;
        metadata.protocol = "non-http-tls".to_string();
        assert_eq!(
            calibration_outcome(CalibrationPhase::Tls, &[metadata]),
            CalibrationOutcome::MetadataOnly
        );
        let mut unsupported = client;
        unsupported.inspectability = Inspectability::Unsupported;
        unsupported.protocol = "quic".to_string();
        assert_eq!(
            calibration_outcome(CalibrationPhase::Tls, &[unsupported]),
            CalibrationOutcome::UnsupportedProtocol
        );
    }

    #[test]
    fn calibration_confirmation_is_explicit_and_defaults_to_no() {
        assert!(calibration_answer_is_affirmative("y"));
        assert!(calibration_answer_is_affirmative(" YES \r\n"));
        assert!(!calibration_answer_is_affirmative(""));
        assert!(!calibration_answer_is_affirmative("no"));
        assert!(!calibration_answer_is_affirmative("later"));
    }

    #[test]
    fn calibration_deadlines_preserve_shorter_values_and_cap_longer_ones() {
        assert_eq!(
            bounded_timeout(Some(Duration::from_secs(7)), CALIBRATION_LAUNCH_TIMEOUT),
            Duration::from_secs(7)
        );
        assert_eq!(
            bounded_timeout(Some(Duration::from_secs(90)), CALIBRATION_LAUNCH_TIMEOUT),
            CALIBRATION_LAUNCH_TIMEOUT
        );
        assert_eq!(
            bounded_timeout(None, CALIBRATION_OBSERVATION_TIMEOUT),
            CALIBRATION_OBSERVATION_TIMEOUT
        );
    }

    #[test]
    fn owned_platform_capture_inherits_the_finite_session_launch_deadline() {
        let deadline = Duration::from_secs(17);
        assert_eq!(owned_platform_wait(None, deadline), Some(deadline));
        assert_eq!(
            owned_platform_wait(Some(Duration::from_secs(3)), deadline),
            Some(Duration::from_secs(3)),
            "an explicit shorter operator deadline is retained"
        );
        assert_eq!(
            owned_platform_wait(Some(Duration::from_secs(30)), deadline),
            Some(deadline),
            "an operator value cannot exceed the session launch deadline"
        );
    }

    #[test]
    fn terminal_calibration_outcome_distinguishes_interrupt_and_failure() {
        assert_eq!(
            terminal_calibration_outcome(CalibrationPhase::Tls, &[], true, false),
            CalibrationOutcome::Interrupted
        );
        assert_eq!(
            terminal_calibration_outcome(CalibrationPhase::Tls, &[], false, true),
            CalibrationOutcome::Failed
        );
        assert_eq!(
            terminal_calibration_outcome(CalibrationPhase::Tls, &[], true, true),
            CalibrationOutcome::Interrupted
        );
    }

    #[test]
    fn real_application_records_do_not_invent_process_identity() {
        let output = application_jsonl("session", 1, &[observation()], "complete");
        let value: serde_json::Value =
            serde_json::from_str(output.lines().nth(1).unwrap()).unwrap();
        assert!(value["process_id"].is_null());
        assert!(value["process_image"].is_null());
        assert!(value["role"].is_null());
        assert_eq!(value["attribution"], "packet-flow-only");
    }

    #[test]
    fn application_stream_has_contract_header_and_trailer() {
        let output = application_jsonl("session", 1, &[observation()], "complete");
        let records: Vec<serde_json::Value> = output
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records[0]["type"], "application.header");
        assert_eq!(records[1]["flow_id"], "flow-00000001");
        assert_eq!(records[2]["type"], "application.trailer");
        assert_eq!(records[2]["records"], 1);
    }

    #[test]
    fn unretained_connection_history_keeps_application_truth_partial() {
        let matched = json!({"correlation_state":"matched"});
        let correlations = vec![&matched];
        assert_eq!(application_correlation_state(&correlations, 1), "partial");
        assert_eq!(application_completeness("complete", 0, 0, 1), "partial");
    }

    #[test]
    fn absent_packet_flow_correlation_is_explicit() {
        let mut observation = observation();
        observation.flow_id = None;
        let output = application_jsonl("session", 1, &[observation], "complete");
        let value: serde_json::Value =
            serde_json::from_str(output.lines().nth(1).unwrap()).unwrap();
        assert!(value["flow_id"].is_null());
        assert!(value["reason"]
            .as_str()
            .unwrap()
            .contains("flow correlation unavailable"));
    }

    #[test]
    fn real_process_sidecar_reports_unavailable_instead_of_a_placeholder_process() {
        let output = process_trace_jsonl("session", None, &[]);
        let value: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(value["event"], "process-trace.unavailable");
        assert!(value["pid"].is_null());
        assert!(value["process"].is_null());
    }

    #[test]
    fn real_process_sidecar_copies_observed_stage_lifecycle() {
        let event = Event::StageMatched {
            role: "client".to_string(),
            pid: 7,
            process: "client.exe".to_string(),
        }
        .render(UNIX_EPOCH);
        let output = process_trace_jsonl("session", None, &[event]);
        let value: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(value["event"], "stage.matched");
        assert_eq!(value["session_id"], "session");
        assert_eq!(value["pid"], 7);
    }

    #[test]
    fn controlled_trust_manager_never_claims_an_os_mutation() {
        let mut manager = ControlledTrustManager;
        assert!(manager.ensure_trusted(false).is_err());
        let trust = manager.ensure_trusted(true).unwrap();
        assert_eq!(trust.state, "simulated-current-user");
        let cleanup = manager.cleanup(CALIBRATION_CLEANUP_TIMEOUT);
        assert_eq!(cleanup.resource, "trust-entry");
        assert_eq!(cleanup.status, "not-needed");
    }

    fn routing_facts(launch_case: CompatibilityLaunchCase, stale: bool) -> Vec<CompatibilityFact> {
        [
            (CompatibilityFactKey::ProxyRouting, "reached-client"),
            (CompatibilityFactKey::ProxyPropagation, "confirmed"),
        ]
        .into_iter()
        .map(|(key, value)| {
            let mut fact =
                CompatibilityFact::new(1, key, value, CompatibilityEvidenceSource::UserConfirmed)
                    .unwrap();
            fact.launch_case = Some(launch_case);
            fact.stale = stale;
            fact
        })
        .collect()
    }

    #[test]
    fn compatibility_preflight_requires_the_exact_current_launch_case() {
        let selected = CompatibilityLaunchCase::SteamProtocolCold;
        let validate = |facts: &[CompatibilityFact]| {
            fragcap::deep_capture::validate_compatibility_prerequisites(
                fragcap::deep_capture::SessionMode::TlsCalibration,
                false,
                facts,
                library_launch_case(selected),
            )
        };
        assert!(validate(&routing_facts(selected, false)).is_ok());
        assert!(validate(&routing_facts(
            CompatibilityLaunchCase::DirectExeCold,
            false
        ),)
        .is_err());
        assert!(validate(&routing_facts(selected, true)).is_err());

        let mut superseded = routing_facts(selected, false);
        let mut latest = CompatibilityFact::new(
            1,
            CompatibilityFactKey::ProxyRouting,
            "inconclusive",
            CompatibilityEvidenceSource::ObservedRun,
        )
        .unwrap();
        latest.launch_case = Some(selected);
        superseded.push(latest);
        assert!(validate(&superseded).is_err());
    }

    #[test]
    fn steam_launch_state_selects_the_exact_compatibility_case() {
        assert_eq!(
            steam_launch_case(false),
            CompatibilityLaunchCase::SteamProtocolCold
        );
        assert_eq!(
            steam_launch_case(true),
            CompatibilityLaunchCase::SteamProtocolWarm
        );
    }

    #[test]
    fn compatibility_policy_accepts_exact_cold_managed_launches() {
        let validate = |launch_case| {
            fragcap::deep_capture::validate_compatibility_prerequisites(
                fragcap::deep_capture::SessionMode::Capture,
                false,
                &routing_facts(launch_case, false),
                library_launch_case(launch_case),
            )
        };
        assert!(validate(CompatibilityLaunchCase::SteamProtocolCold).is_ok());
        assert!(validate(CompatibilityLaunchCase::DirectExeCold).is_ok());
        assert!(validate(CompatibilityLaunchCase::PublisherLauncherCold).is_ok());
        for unsupported in [
            CompatibilityLaunchCase::SteamProtocolWarm,
            CompatibilityLaunchCase::DirectExeWarm,
            CompatibilityLaunchCase::PublisherLauncherWarm,
            CompatibilityLaunchCase::PublisherLauncherGameStartCleanWarm,
        ] {
            assert!(
                validate(unsupported).is_err(),
                "{} must be refused before side effects",
                unsupported.as_str()
            );
        }
    }

    #[test]
    fn real_calibration_accepts_a_cold_direct_executable() {
        for mode in [
            fragcap::deep_capture::SessionMode::ReachabilityCalibration,
            fragcap::deep_capture::SessionMode::TlsCalibration,
        ] {
            let result = fragcap::deep_capture::validate_compatibility_prerequisites(
                mode,
                false,
                &routing_facts(CompatibilityLaunchCase::DirectExeCold, false),
                fragcap::deep_capture::LaunchCase::DirectExeCold,
            );
            assert!(
                result.is_ok(),
                "{mode:?} must accept a cold direct executable"
            );
        }
    }
}
