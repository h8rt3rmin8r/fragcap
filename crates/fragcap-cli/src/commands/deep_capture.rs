// SPDX-License-Identifier: Apache-2.0

//! `deep-capture`: explicit scoped local proxy inspection for one stored target.
//!
//! This first vertical slice is intentionally narrow. It exercises the product
//! shape and artifact contracts with a controlled target path, refuses unsafe or
//! unknown real-target paths before side effects, and keeps the proxy backend
//! behind a replaceable boundary so a native backend can replace `mitmdump`
//! later without changing the CLI contract.

use std::cell::RefCell;
use std::ffi::OsString;
use std::fs;
use std::io::{IsTerminal, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use fragcap::deep_capture::{calibration_outcome, observation_proves_final_client_ca_acceptance};
use fragcap::deep_capture::{
    calibration_outcome_reason, terminal_calibration_outcome, CalibrationOutcome, CalibrationPhase,
    CompatibilityObservation as Observation, Inspectability,
};
use fragcap::targets::{
    entry_windows_clients, resolve_id, resolve_positional, CompatibilityEvidenceSource,
    CompatibilityFact, CompatibilityFactKey, CompatibilityLaunchCase, Selection, Store,
    TargetEntry,
};
use fragcap::{
    CaptureStats, CapturedPacket, Fidelity, FlowId, FlowKey, FlowRegistry, InterfaceDeclaration,
    InterfaceId, LinkType, Payload, PcapngWriter, Proto, RawPacket, Sink, StopReason, Timestamp,
};
use serde_json::json;

use crate::args::Direction;
use crate::cli::{
    CaptureArgs, ControlledTargetArgs, DeepCaptureArgs, DeepCaptureCalibrationArg,
    DeepCaptureLaunchCaseArg, DeepCaptureProxyArg, OfflineArgs, ScopeArg,
};
use crate::commands::{capture, target_resolve};
use crate::emit::Emitter;
use crate::events::{rfc3339_utc, Event};
use crate::exit::{CliError, Exit};
use crate::paths;

const MITMDUMP_ADDON: &str = r#"
from mitmproxy import http, tcp
import json
import os
import time

EVENTS = os.environ.get("FRAGCAP_DEEP_CAPTURE_EVENTS")

def write_record(record):
    if not EVENTS:
        return
    record.setdefault("ts", time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()))
    with open(EVENTS, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")

def address(value):
    if not value:
        return None
    return {"host": value[0], "port": value[1]}

def response(flow: http.HTTPFlow):
    write_record({
        "proxy_connection_id": flow.id,
        "client_peer": address(flow.client_conn.peername),
        "proxy_local": address(flow.client_conn.sockname),
        "protocol": flow.request.scheme,
        "inspectability": "full",
        "method": flow.request.method,
        "url": flow.request.pretty_url,
        "status": flow.response.status_code if flow.response else None,
        "reason": None,
    })

def error(flow: http.HTTPFlow):
    write_record({
        "proxy_connection_id": flow.id,
        "client_peer": address(flow.client_conn.peername),
        "proxy_local": address(flow.client_conn.sockname),
        "protocol": flow.request.scheme if flow.request else "http",
        "inspectability": "metadata-only",
        "method": flow.request.method if flow.request else None,
        "url": flow.request.pretty_url if flow.request else None,
        "status": None,
        "reason": str(flow.error) if flow.error else "http flow error",
    })

def tcp_message(flow: tcp.TCPFlow):
    write_record({
        "proxy_connection_id": flow.id,
        "client_peer": address(flow.client_conn.peername),
        "proxy_local": address(flow.client_conn.sockname),
        "protocol": "non-http-tls",
        "inspectability": "metadata-only",
        "method": None,
        "url": None,
        "status": None,
        "reason": "no HTTP semantics observed",
    })
"#;
const CONTROLLED_TARGET_HANDLE: &str = "sample-target";
const CONTROLLED_TARGET_STABLE_ID: i64 = 75_000;
const CALIBRATION_LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);
const CALIBRATION_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(60);
const CALIBRATION_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
const CALIBRATION_CLEANUP_TIMEOUT: Duration = Duration::from_secs(15);

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
            proxy_backend: "mitmdump".to_string(),
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
    flow_registry: Option<Arc<FlowRegistry>>,
    controlled_process_id: Option<u32>,
    process_events: Vec<String>,
    interrupted: bool,
    backend: Option<ProxyBackend>,
    ca_cert_path: Option<PathBuf>,
    trust: Option<TrustOutcome>,
    observations: Vec<Observation>,
    listen_port: Option<u16>,
    key_log_path: Option<PathBuf>,
    started_at: Option<SystemTime>,
}

struct LibraryProxyAdapter<'a> {
    args: &'a DeepCaptureArgs,
    runtime: Rc<RefCell<LibraryRuntime>>,
}

impl fragcap::deep_capture::ProxyBackend for LibraryProxyAdapter<'_> {
    fn descriptor(&self) -> fragcap::deep_capture::BackendDescriptor {
        fragcap::deep_capture::BackendDescriptor {
            name: if self.args.controlled_target {
                "controlled"
            } else {
                "mitmdump"
            }
            .to_string(),
            version: "resolved-at-start".to_string(),
        }
    }

    fn start(
        &mut self,
        plan: &fragcap::deep_capture::SessionPlan,
        _budget: fragcap::deep_capture::Budget,
    ) -> Result<Box<dyn fragcap::deep_capture::ProxyLease>, fragcap::deep_capture::StageFailure>
    {
        fs::create_dir_all(&plan.bundle).map_err(|error| {
            fragcap::deep_capture::StageFailure::new(
                fragcap::deep_capture::Stage::ProxyStart,
                "bundle-create",
                format!(
                    "cannot create Deep Capture bundle {}: {error}",
                    plan.bundle.display()
                ),
            )
        })?;
        let backend = resolve_backend(self.args).map_err(|error| {
            library_stage_failure(fragcap::deep_capture::Stage::ProxyStart, "backend", error)
        })?;
        let descriptor = backend.descriptor().clone();
        let proxy = backend
            .start(self.args, &plan.bundle, plan.endpoint.port)
            .map_err(|error| {
                library_stage_failure(fragcap::deep_capture::Stage::ProxyStart, "start", error)
            })?;
        {
            let mut runtime = self.runtime.borrow_mut();
            runtime.backend = Some(descriptor);
            runtime.ca_cert_path = proxy.ca_cert_path.clone();
            runtime.listen_port = Some(plan.endpoint.port);
            runtime.key_log_path = proxy.key_log_path.clone();
            runtime.started_at = Some(SystemTime::now());
        }
        Ok(Box::new(LibraryProxyLease {
            proxy,
            controlled: self.args.controlled_target,
            retain_key_log: self.args.key_log,
            runtime: Rc::clone(&self.runtime),
        }))
    }
}

struct LibraryProxyLease {
    proxy: RunningProxy,
    controlled: bool,
    retain_key_log: bool,
    runtime: Rc<RefCell<LibraryRuntime>>,
}

impl fragcap::deep_capture::ProxyLease for LibraryProxyLease {
    fn observations(
        &mut self,
        _budget: fragcap::deep_capture::Budget,
    ) -> Result<Vec<Observation>, fragcap::deep_capture::StageFailure> {
        let mut observations =
            read_proxy_observations(&self.proxy.events_path).map_err(|error| {
                library_stage_failure(fragcap::deep_capture::Stage::Observe, "proxy-events", error)
            })?;
        let runtime = self.runtime.borrow();
        if self.controlled {
            assign_controlled_flow_ids(&mut observations, runtime.controlled_process_id);
        } else if let Some(registry) = &runtime.flow_registry {
            correlate_observations(&mut observations, registry);
        }
        drop(runtime);
        self.runtime.borrow_mut().observations = observations.clone();
        Ok(observations)
    }

    fn stop(
        &mut self,
        budget: fragcap::deep_capture::Budget,
    ) -> fragcap::deep_capture::CleanupResult {
        let result = self.proxy.stop_with_timeout(budget.remaining());
        library_cleanup(
            "proxy-shutdown",
            result.map(|()| ("succeeded", "proxy stopped within its deadline")),
        )
    }

    fn cleanup(
        &mut self,
        budget: fragcap::deep_capture::Budget,
    ) -> Vec<fragcap::deep_capture::CleanupResult> {
        if self.retain_key_log {
            self.proxy.retain_key_log();
        }
        [
            self.proxy.cleanup_process_with_timeout(budget.remaining()),
            self.proxy.cleanup_port(),
            self.proxy
                .cleanup_ephemeral_with_timeout(budget.remaining()),
        ]
        .into_iter()
        .map(library_cleanup_resource)
        .collect()
    }
}

fn library_stage_failure(
    stage: fragcap::deep_capture::Stage,
    code: &'static str,
    error: CliError,
) -> fragcap::deep_capture::StageFailure {
    fragcap::deep_capture::StageFailure::new(stage, code, error.to_string())
}

fn library_cleanup(
    resource: &str,
    result: Result<(&str, &str), String>,
) -> fragcap::deep_capture::CleanupResult {
    match result {
        Ok((status, reason)) => fragcap::deep_capture::CleanupResult {
            resource: resource.to_string(),
            status: if status == "succeeded" {
                fragcap::deep_capture::CleanupStatus::Released
            } else {
                fragcap::deep_capture::CleanupStatus::NotNeeded
            },
            reason: reason.to_string(),
        },
        Err(reason) => fragcap::deep_capture::CleanupResult {
            resource: resource.to_string(),
            status: fragcap::deep_capture::CleanupStatus::Failed,
            reason,
        },
    }
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
        _budget: fragcap::deep_capture::Budget,
    ) -> Result<Box<dyn fragcap::deep_capture::TrustLease>, fragcap::deep_capture::StageFailure>
    {
        let mut manager: Box<dyn TrustManager> = if self.controlled {
            Box::new(ControlledTrustManager)
        } else {
            let ca_cert_path = self.runtime.borrow().ca_cert_path.clone().ok_or_else(|| {
                fragcap::deep_capture::StageFailure::new(
                    fragcap::deep_capture::Stage::Trust,
                    "ca-material-missing",
                    "the proxy did not expose session CA material",
                )
            })?;
            platform_trust_manager(ca_cert_path).map_err(|error| {
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
        _endpoint: fragcap::deep_capture::LoopbackEndpoint,
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
    mode: fragcap::deep_capture::SessionMode,
}

impl fragcap::deep_capture::CaptureRunner for LibraryCaptureAdapter<'_, '_, '_> {
    fn prepare(
        &mut self,
        config: &fragcap::deep_capture::SessionConfig,
        target: &fragcap::deep_capture::PreparedTarget,
        endpoint: fragcap::deep_capture::LoopbackEndpoint,
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
            let capture_args = real_capture_args(self.args, &config.bundle, deadlines);
            let mut prepared = capture::prepare(&capture_args, &mut self.emitter.borrow_mut())
                .map_err(|error| library_refusal("capture-prepare", error))?;
            if target.launch_case == fragcap::deep_capture::LaunchCase::DirectExeCold {
                let proxy_url = format!("http://127.0.0.1:{}", endpoint.port);
                prepared
                    .with_launch_environment([
                        ("HTTP_PROXY", proxy_url.as_str()),
                        ("HTTPS_PROXY", proxy_url.as_str()),
                        ("ALL_PROXY", proxy_url.as_str()),
                        ("NO_PROXY", ""),
                    ])
                    .map_err(|error| library_refusal("direct-launch-environment", error))?;
            }
            self.prepared = Some((capture_args, prepared));
        }
        Ok(fragcap::deep_capture::PreparedCapture {
            token: "ordinary-capture".to_string(),
        })
    }

    fn run(
        &mut self,
        _prepared: &fragcap::deep_capture::PreparedCapture,
        endpoint: fragcap::deep_capture::LoopbackEndpoint,
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
            let process_id =
                run_controlled_target_harness(endpoint.port, phase, budget.remaining()).map_err(
                    |error| {
                        library_stage_failure(
                            fragcap::deep_capture::Stage::Capture,
                            "controlled-target",
                            error,
                        )
                    },
                )?;
            self.runtime.borrow_mut().controlled_process_id = Some(process_id);
            return Ok(fragcap::deep_capture::CaptureRunResult {
                observations: Vec::new(),
                interrupted: false,
            });
        }
        let (capture_args, prepared) = self.prepared.take().ok_or_else(|| {
            fragcap::deep_capture::StageFailure::new(
                fragcap::deep_capture::Stage::Capture,
                "capture-not-prepared",
                "ordinary Capture preparation was not retained",
            )
        })?;
        let (registry, result, process_events, interrupted) = run_real_capture(
            &capture_args,
            prepared,
            endpoint.port,
            &mut self.emitter.borrow_mut(),
        );
        {
            let mut runtime = self.runtime.borrow_mut();
            runtime.flow_registry = Some(registry);
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

    fn finalize(
        &mut self,
        bundle: &Path,
        snapshot: &fragcap::deep_capture::TerminalSnapshot,
    ) -> Vec<fragcap::deep_capture::ArtifactResult> {
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
            executable: None,
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
        let cleanup = CleanupReport::new(
            snapshot
                .cleanup
                .iter()
                .map(|result| CleanupResource {
                    resource: result.resource.clone(),
                    status: match result.status {
                        fragcap::deep_capture::CleanupStatus::Released => "succeeded",
                        fragcap::deep_capture::CleanupStatus::NotNeeded => "not-needed",
                        fragcap::deep_capture::CleanupStatus::TimedOut
                        | fragcap::deep_capture::CleanupStatus::Failed => "failed",
                        _ => "failed",
                    }
                    .to_string(),
                    reason: result.reason.clone(),
                })
                .collect(),
        );
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
            observations: &snapshot.observations,
            trust: &trust,
            cleanup: &cleanup,
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
                "proxy-log",
                "proxy.jsonl",
                fragcap::deep_capture::Sensitivity::Payload,
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
                "cleanup",
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
                if let Some(path) = &runtime.key_log_path {
                    announce_key_log(session_id, path, &mut emitter);
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
        deadlines: fragcap::deep_capture::Deadlines {
            launch: deadlines.launch,
            observation: deadlines.observation,
            shutdown: deadlines.shutdown,
            cleanup: deadlines.cleanup,
        },
    };

    let store = Rc::new(RefCell::new(open_local_store(args.local_db.as_deref())?));
    let selected = Rc::new(RefCell::new(None));
    let selected_launch_case = Rc::new(RefCell::new(None));
    let runtime = Rc::new(RefCell::new(LibraryRuntime::default()));
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
        proxy: Box::new(LibraryProxyAdapter {
            args,
            runtime: Rc::clone(&runtime),
        }),
        trust: Box::new(LibraryTrustAdapter {
            controlled: args.controlled_target,
            runtime: Rc::clone(&runtime),
        }),
        launch: Box::new(LibraryLaunchAdapter),
        capture: Box::new(LibraryCaptureAdapter {
            args,
            emitter: Rc::clone(&emitter),
            prepared: None,
            runtime: Rc::clone(&runtime),
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
    executable: Option<PathBuf>,
}

trait ProxyBackendAdapter {
    fn descriptor(&self) -> &ProxyBackend;
    fn start(
        &self,
        args: &DeepCaptureArgs,
        bundle: &Path,
        listen_port: u16,
    ) -> Result<RunningProxy, CliError>;
}

struct ControlledProxyBackend {
    descriptor: ProxyBackend,
}

impl ProxyBackendAdapter for ControlledProxyBackend {
    fn descriptor(&self) -> &ProxyBackend {
        &self.descriptor
    }

    fn start(
        &self,
        _args: &DeepCaptureArgs,
        bundle: &Path,
        listen_port: u16,
    ) -> Result<RunningProxy, CliError> {
        start_controlled_proxy(bundle, listen_port)
    }
}

struct MitmdumpProxyBackend {
    descriptor: ProxyBackend,
}

impl ProxyBackendAdapter for MitmdumpProxyBackend {
    fn descriptor(&self) -> &ProxyBackend {
        &self.descriptor
    }

    fn start(
        &self,
        args: &DeepCaptureArgs,
        bundle: &Path,
        listen_port: u16,
    ) -> Result<RunningProxy, CliError> {
        start_mitmdump_proxy(args, &self.descriptor, bundle, listen_port)
    }
}

struct RunningProxy {
    events_path: PathBuf,
    ca_cert_path: Option<PathBuf>,
    key_log_path: Option<PathBuf>,
    child: Option<Child>,
    started_child: bool,
    controlled_thread: Option<JoinHandle<Result<(), String>>>,
    controlled_shutdown: Option<Arc<AtomicBool>>,
    started_controlled: bool,
    listen_port: u16,
    ephemeral_paths: Vec<PathBuf>,
    stop_result: Option<Result<(), String>>,
}

impl RunningProxy {
    fn stop(&mut self) -> Result<(), String> {
        self.stop_with_timeout(CALIBRATION_SHUTDOWN_TIMEOUT)
    }

    fn stop_with_timeout(&mut self, timeout: Duration) -> Result<(), String> {
        if let Some(result) = &self.stop_result {
            return result.clone();
        }
        let result = self.stop_once(timeout);
        self.stop_result = Some(result.clone());
        result
    }

    fn stop_once(&mut self, timeout: Duration) -> Result<(), String> {
        let started = Instant::now();
        if let Some(shutdown) = &self.controlled_shutdown {
            shutdown.store(true, Ordering::Release);
        }
        if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {}
                Err(err) => return Err(err.to_string()),
            }
            child.kill().map_err(|err| err.to_string())?;
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) if started.elapsed() >= timeout => {
                        return Err(format!(
                            "proxy process did not stop within {} seconds",
                            CalibrationDeadlines::seconds(timeout)
                        ));
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                    Err(err) => return Err(err.to_string()),
                }
            }
        }
        if let Some(thread) = self.controlled_thread.take() {
            while !thread.is_finished() {
                if started.elapsed() >= timeout {
                    return Err(format!(
                        "controlled proxy did not stop within {} seconds",
                        CalibrationDeadlines::seconds(timeout)
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            thread
                .join()
                .map_err(|_| "controlled proxy thread panicked".to_string())??;
        }
        Ok(())
    }

    fn cleanup_process(&mut self) -> CleanupResource {
        self.cleanup_process_with_timeout(CALIBRATION_CLEANUP_TIMEOUT)
    }

    fn cleanup_process_with_timeout(&mut self, timeout: Duration) -> CleanupResource {
        match self.stop_with_timeout(timeout) {
            Ok(()) if self.started_child => {
                CleanupResource::new("proxy-process", "succeeded", "owned proxy child stopped")
            }
            Ok(()) if self.started_controlled => CleanupResource::new(
                "proxy-process",
                "succeeded",
                "controlled proxy adapter stopped",
            ),
            Err(err) => CleanupResource::new("proxy-process", "failed", &err),
            Ok(()) => CleanupResource::new("proxy-process", "not-needed", "no child was started"),
        }
    }

    fn cleanup_port(&self) -> CleanupResource {
        if !(self.started_child || self.started_controlled) {
            return CleanupResource::new(
                "proxy-port",
                "not-needed",
                "the controlled backend did not bind a loopback port",
            );
        }
        if loopback_port_is_open(self.listen_port) {
            CleanupResource::new(
                "proxy-port",
                "failed",
                "the session loopback port still accepts connections",
            )
        } else {
            CleanupResource::new(
                "proxy-port",
                "succeeded",
                "the session loopback port was released",
            )
        }
    }

    fn cleanup_ephemeral(&self) -> CleanupResource {
        self.cleanup_ephemeral_with_timeout(CALIBRATION_CLEANUP_TIMEOUT)
    }

    fn cleanup_ephemeral_with_timeout(&self, timeout: Duration) -> CleanupResource {
        let started = Instant::now();
        let mut failures = Vec::new();
        for path in &self.ephemeral_paths {
            if started.elapsed() >= timeout {
                failures.push(format!(
                    "cleanup deadline of {} seconds expired before {}",
                    CalibrationDeadlines::seconds(timeout),
                    path.display()
                ));
                break;
            }
            let result = if path.is_dir() {
                fs::remove_dir_all(path)
            } else if path.exists() {
                fs::remove_file(path)
            } else {
                Ok(())
            };
            if let Err(err) = result {
                failures.push(format!("{}: {err}", path.display()));
            }
        }
        if failures.is_empty() {
            CleanupResource::new(
                "proxy-private-material",
                if self.ephemeral_paths.is_empty() {
                    "not-needed"
                } else {
                    "succeeded"
                },
                "session proxy internals removed",
            )
        } else {
            CleanupResource::new("proxy-private-material", "failed", &failures.join("; "))
        }
    }

    fn retain_key_log(&mut self) {
        if let Some(key_log_path) = &self.key_log_path {
            self.ephemeral_paths.retain(|path| path != key_log_path);
        }
    }
}

impl Drop for RunningProxy {
    fn drop(&mut self) {
        let _ = self.stop();
    }
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
fn platform_trust_manager(ca_cert_path: PathBuf) -> Result<Box<dyn TrustManager>, CliError> {
    Ok(Box::new(WindowsCurrentUserTrustManager {
        ca_cert_path,
        thumbprint: None,
        installed_this_session: false,
    }))
}

#[cfg(not(windows))]
fn platform_trust_manager(_ca_cert_path: PathBuf) -> Result<Box<dyn TrustManager>, CliError> {
    Err(CliError::failure(
        "Deep Capture current-user CA trust is implemented only on Windows",
    ))
}

#[cfg(windows)]
struct WindowsCurrentUserTrustManager {
    ca_cert_path: PathBuf,
    thumbprint: Option<String>,
    installed_this_session: bool,
}

#[cfg(windows)]
impl WindowsCurrentUserTrustManager {
    fn thumbprint(&self) -> Result<String, CliError> {
        crate::windows_cert::file_thumbprint(&self.ca_cert_path).map_err(CliError::failure)
    }

    fn certutil(&self, args: impl IntoIterator<Item = OsString>) -> Result<bool, String> {
        self.certutil_with_timeout(args, CALIBRATION_CLEANUP_TIMEOUT)
    }

    fn certutil_with_timeout(
        &self,
        args: impl IntoIterator<Item = OsString>,
        timeout: Duration,
    ) -> Result<bool, String> {
        if timeout.is_zero() {
            return Err("cleanup deadline expired before certutil could run".to_string());
        }
        let certutil = find_on_path("certutil")
            .ok_or_else(|| "certutil.exe is required for current-user CA trust".to_string())?;
        let mut command = std::process::Command::new(certutil);
        command.args(args).stderr(Stdio::null());
        command_stdout_with_timeout(&mut command, timeout).map(|(status, _)| status.success())
    }

    fn is_trusted(&self, thumbprint: &str) -> Result<bool, String> {
        self.certutil(
            ["-user", "-verifystore", "Root", thumbprint]
                .into_iter()
                .map(OsString::from),
        )
    }

    fn remove(&self, thumbprint: &str) -> Result<bool, String> {
        self.certutil(
            ["-user", "-delstore", "Root", thumbprint]
                .into_iter()
                .map(OsString::from),
        )
    }

    fn is_trusted_with_timeout(&self, thumbprint: &str, timeout: Duration) -> Result<bool, String> {
        self.certutil_with_timeout(
            ["-user", "-verifystore", "Root", thumbprint]
                .into_iter()
                .map(OsString::from),
            timeout,
        )
    }

    fn remove_with_timeout(&self, thumbprint: &str, timeout: Duration) -> Result<bool, String> {
        self.certutil_with_timeout(
            ["-user", "-delstore", "Root", thumbprint]
                .into_iter()
                .map(OsString::from),
            timeout,
        )
    }
}

#[cfg(windows)]
impl TrustManager for WindowsCurrentUserTrustManager {
    fn ensure_trusted(&mut self, confirmed: bool) -> Result<TrustOutcome, CliError> {
        if !confirmed {
            return Err(CliError::usage(
                "Deep Capture CA trust mutation requires explicit confirmation",
            ));
        }
        let thumbprint = self.thumbprint()?;
        self.thumbprint = Some(thumbprint.clone());
        if self
            .is_trusted(&thumbprint)
            .map_err(|e| CliError::failure(format!("cannot query current-user CA trust: {e}")))?
        {
            return Ok(TrustOutcome {
                state: "current-user-trusted".to_string(),
                action: "already-trusted".to_string(),
                thumbprint: Some(thumbprint),
            });
        }

        let installed = self
            .certutil([
                OsString::from("-user"),
                OsString::from("-addstore"),
                OsString::from("-f"),
                OsString::from("Root"),
                self.ca_cert_path.as_os_str().to_owned(),
            ])
            .map_err(|e| CliError::failure(format!("cannot install current-user CA trust: {e}")))?;
        if !installed {
            return Err(CliError::failure(
                "certutil could not install the Deep Capture CA in the current-user Root store",
            ));
        }
        self.installed_this_session = true;
        if !self
            .is_trusted(&thumbprint)
            .map_err(|e| CliError::failure(format!("cannot verify current-user CA trust: {e}")))?
        {
            let _ = self.remove(&thumbprint);
            self.installed_this_session = false;
            return Err(CliError::failure(
                "Deep Capture CA installation could not be verified in the current-user Root store",
            ));
        }
        Ok(TrustOutcome {
            state: "current-user-trusted".to_string(),
            action: "installed-for-session".to_string(),
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
        let Some(thumbprint) = self.thumbprint.as_deref() else {
            return CleanupResource::new(
                "trust-entry",
                "failed",
                "the installed trust entry has no recorded thumbprint",
            );
        };
        match self.remove_with_timeout(thumbprint, remaining_timeout(started, timeout)) {
            Ok(true) => match self
                .is_trusted_with_timeout(thumbprint, remaining_timeout(started, timeout))
            {
                Ok(false) => {
                    self.installed_this_session = false;
                    CleanupResource::new(
                        "trust-entry",
                        "succeeded",
                        "session CA removed from the current-user Root store",
                    )
                }
                Ok(true) => CleanupResource::new(
                    "trust-entry",
                    "failed",
                    "session CA remains in the current-user Root store after removal",
                ),
                Err(err) => CleanupResource::new(
                    "trust-entry",
                    "failed",
                    &format!("cannot verify trust cleanup: {err}"),
                ),
            },
            Ok(false) => CleanupResource::new(
                "trust-entry",
                "failed",
                "certutil could not remove the session CA from the current-user Root store",
            ),
            Err(err) => CleanupResource::new("trust-entry", "failed", &err),
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

fn resolve_backend(args: &DeepCaptureArgs) -> Result<Box<dyn ProxyBackendAdapter>, CliError> {
    match args.proxy_backend {
        DeepCaptureProxyArg::Mitmdump if args.controlled_target => {
            Ok(Box::new(ControlledProxyBackend {
                descriptor: ProxyBackend {
                    name: "controlled".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    executable: None,
                },
            }))
        }
        DeepCaptureProxyArg::Mitmdump => {
            let descriptor = mitmdump_backend()?;
            Ok(Box::new(MitmdumpProxyBackend { descriptor }))
        }
    }
}

fn mitmdump_backend() -> Result<ProxyBackend, CliError> {
    mitmdump_backend_from_path(find_on_path("mitmdump"))
}

fn mitmdump_backend_from_path(path: Option<PathBuf>) -> Result<ProxyBackend, CliError> {
    let path = path.ok_or_else(|| {
        CliError::failure(
            "Deep Capture proxy backend `mitmdump` is unavailable; install it or run `fragcap \
             doctor` for readiness details",
        )
    })?;
    let version = command_stdout_with_timeout(
        std::process::Command::new(&path)
            .arg("--version")
            .stderr(std::process::Stdio::null()),
        std::time::Duration::from_secs(2),
    )
    .ok()
    .and_then(|(status, stdout)| {
        if status.success() {
            Some(
                String::from_utf8_lossy(&stdout)
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .unwrap_or("version undetermined")
                    .trim()
                    .to_string(),
            )
        } else {
            None
        }
    })
    .unwrap_or_else(|| "version undetermined".to_string());
    Ok(ProxyBackend {
        name: "mitmdump".to_string(),
        version,
        executable: Some(path),
    })
}

fn start_controlled_proxy(bundle: &Path, listen_port: u16) -> Result<RunningProxy, CliError> {
    let events_path = bundle.join("proxy-observations.jsonl");
    let listener = TcpListener::bind(("127.0.0.1", listen_port)).map_err(|e| {
        CliError::failure(format!("cannot bind controlled proxy loopback port: {e}"))
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|e| CliError::failure(format!("cannot configure controlled proxy: {e}")))?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let thread_events = events_path.clone();
    let controlled_thread = std::thread::spawn(move || {
        serve_controlled_proxy(listener, &thread_events, &thread_shutdown)
    });
    Ok(RunningProxy {
        events_path: events_path.clone(),
        ca_cert_path: None,
        key_log_path: None,
        child: None,
        started_child: false,
        controlled_thread: Some(controlled_thread),
        controlled_shutdown: Some(shutdown),
        started_controlled: true,
        listen_port,
        ephemeral_paths: vec![events_path],
        stop_result: None,
    })
}

fn start_mitmdump_proxy(
    args: &DeepCaptureArgs,
    backend: &ProxyBackend,
    bundle: &Path,
    listen_port: u16,
) -> Result<RunningProxy, CliError> {
    let events_path = bundle.join("proxy-observations.jsonl");
    let executable = backend
        .executable
        .as_ref()
        .ok_or_else(|| CliError::failure("Deep Capture proxy backend has no executable path"))?;
    let confdir = bundle.join("mitmproxy");
    fs::create_dir_all(&confdir).map_err(|e| {
        CliError::failure(format!(
            "cannot create mitmdump confdir {}: {e}",
            confdir.display()
        ))
    })?;
    let addon = bundle.join("fragcap-mitmproxy-addon.py");
    write_file(addon.clone(), MITMDUMP_ADDON.as_bytes())?;
    let stdout = fs::File::create(bundle.join("mitmdump.stdout.log"))
        .map_err(|e| CliError::failure(format!("cannot create mitmdump stdout log: {e}")))?;
    let stderr = fs::File::create(bundle.join("mitmdump.stderr.log"))
        .map_err(|e| CliError::failure(format!("cannot create mitmdump stderr log: {e}")))?;

    let key_log_path = if args.key_log {
        Some(prepare_key_log(bundle)?)
    } else {
        None
    };
    let mut command = std::process::Command::new(executable);
    command
        .arg("--listen-host")
        .arg("127.0.0.1")
        .arg("--listen-port")
        .arg(listen_port.to_string())
        .arg("--set")
        .arg(format!("confdir={}", confdir.display()))
        .arg("--set")
        .arg("termlog_verbosity=error")
        .arg("--set")
        .arg("flow_detail=0")
        .arg("-s")
        .arg(&addon)
        .env("FRAGCAP_DEEP_CAPTURE_EVENTS", &events_path)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_key_log(&mut command, key_log_path.as_deref());
    let ca_cert = confdir.join("mitmproxy-ca-cert.cer");
    let mut ephemeral_paths = vec![
        confdir,
        addon,
        bundle.join("mitmdump.stdout.log"),
        bundle.join("mitmdump.stderr.log"),
        bundle.join("proxy-observations.jsonl"),
    ];
    if let Some(path) = &key_log_path {
        ephemeral_paths.push(path.clone());
    }
    let child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            let proxy = RunningProxy {
                events_path,
                ca_cert_path: Some(ca_cert),
                key_log_path,
                child: None,
                started_child: false,
                controlled_thread: None,
                controlled_shutdown: None,
                started_controlled: false,
                listen_port,
                ephemeral_paths,
                stop_result: None,
            };
            let _ = proxy.cleanup_ephemeral();
            return Err(CliError::failure(format!(
                "cannot start mitmdump backend {}: {err}",
                executable.display()
            )));
        }
    };
    let mut proxy = RunningProxy {
        events_path,
        ca_cert_path: Some(ca_cert.clone()),
        key_log_path,
        child: Some(child),
        started_child: true,
        controlled_thread: None,
        controlled_shutdown: None,
        started_controlled: false,
        listen_port,
        ephemeral_paths,
        stop_result: None,
    };
    if let Err(err) = wait_for_proxy_ready(listen_port, Duration::from_secs(5)) {
        let _ = proxy.cleanup_process();
        let _ = proxy.cleanup_ephemeral();
        return Err(err);
    }
    if let Err(err) = wait_for_file(&ca_cert, Duration::from_secs(5)) {
        let _ = proxy.cleanup_process();
        let _ = proxy.cleanup_ephemeral();
        return Err(err);
    }
    Ok(proxy)
}

fn prepare_key_log(bundle: &Path) -> Result<PathBuf, CliError> {
    let bundle = if bundle.is_absolute() {
        bundle.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| CliError::failure(format!("cannot resolve the key-log directory: {e}")))?
            .join(bundle)
    };
    let path = bundle.join("tls-keylog.log");
    fs::File::create(&path).map_err(|e| {
        CliError::failure(format!(
            "cannot create live TLS key log {}: {e}",
            path.display()
        ))
    })?;
    Ok(path)
}

fn announce_key_log(session_id: &str, path: &Path, emitter: &mut Emitter) {
    let path = path.display().to_string();
    emitter.event(&Event::DeepCaptureKeyLogReady {
        session_id: session_id.to_string(),
        path: path.clone(),
    });
    emitter.progress(&format!("TLS key log ready for live analyzers: {path}"));
}

fn configure_key_log(command: &mut std::process::Command, key_log_path: Option<&Path>) {
    command
        .env_remove("SSLKEYLOGFILE")
        .env_remove("MITMPROXY_SSLKEYLOGFILE");
    if let Some(path) = key_log_path {
        command.env("MITMPROXY_SSLKEYLOGFILE", path);
    }
}

fn wait_for_proxy_ready(port: u16, timeout: Duration) -> Result<(), CliError> {
    let addr = ("127.0.0.1", port)
        .to_socket_addrs()
        .map_err(|e| CliError::failure(format!("cannot resolve proxy listen address: {e}")))?
        .next()
        .ok_or_else(|| CliError::failure("cannot resolve proxy listen address"))?;
    let start = Instant::now();
    while start.elapsed() < timeout {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(CliError::failure(
        "mitmdump did not become ready on 127.0.0.1 before the startup timeout",
    ))
}

fn wait_for_file(path: &Path, timeout: Duration) -> Result<(), CliError> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if path.is_file() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(CliError::failure(format!(
        "mitmdump did not create its session CA certificate {} before the startup timeout",
        path.display()
    )))
}

fn select_loopback_port() -> Result<u16, CliError> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| CliError::failure(format!("cannot reserve a Deep Capture port: {e}")))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|e| CliError::failure(format!("cannot read the reserved proxy port: {e}")))
}

fn loopback_port_is_open(port: u16) -> bool {
    let Ok(addr) = ("127.0.0.1", port).to_socket_addrs() else {
        return false;
    };
    addr.into_iter()
        .any(|addr| TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok())
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
        if process_image_is_running(client)? {
            return Err(CliError::usage(format!(
                "cannot prove a cold direct launch while a process named {client} is already \
                 running; the no-handle process snapshot cannot distinguish same-named \
                 executables by path, so close that process and retry"
            )));
        }
        Ok(CompatibilityLaunchCase::DirectExeCold)
    }
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

#[cfg(windows)]
fn steam_is_running() -> Result<bool, CliError> {
    process_image_is_running("steam.exe")
}

#[cfg(windows)]
fn process_image_is_running(image: &str) -> Result<bool, CliError> {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    // This is a handle to a read-only process snapshot, not to any process. It
    // supplies the image names needed to distinguish a cold launch from a warm
    // protocol dispatch without requesting rights against Steam or a target.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE || snapshot == 0 {
        return Err(CliError::failure(format!(
            "cannot determine whether {image} is already running: {}",
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
            "cannot enumerate processes while checking {image}: {error}"
        )));
    }

    loop {
        let end = entry
            .szExeFile
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(entry.szExeFile.len());
        if String::from_utf16_lossy(&entry.szExeFile[..end]).eq_ignore_ascii_case(image) {
            // SAFETY: the snapshot handle is closed exactly once before returning.
            unsafe { CloseHandle(snapshot) };
            return Ok(true);
        }
        // SAFETY: the same live snapshot and initialized entry remain valid.
        if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
            // SAFETY: read immediately after the failed enumeration call.
            let code = unsafe { GetLastError() };
            // SAFETY: the snapshot handle is closed exactly once before returning.
            unsafe { CloseHandle(snapshot) };
            if code == ERROR_NO_MORE_FILES {
                return Ok(false);
            }
            return Err(CliError::failure(format!(
                "cannot finish enumerating processes while checking {image}: {}",
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

fn run_real_capture(
    capture_args: &CaptureArgs,
    prepared: capture::PreparedCapture,
    listen_port: u16,
    emitter: &mut Emitter,
) -> (Arc<FlowRegistry>, Result<(), CliError>, Vec<String>, bool) {
    let proxy_url = format!("http://127.0.0.1:{listen_port}");
    let _env = ScopedProxyEnv::set(&proxy_url);
    let flow_registry = Arc::new(FlowRegistry::default());
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
    (flow_registry, result, process_events, interrupted)
}

struct ScopedProxyEnv {
    saved: Vec<(&'static str, Option<OsString>)>,
}

impl ScopedProxyEnv {
    fn set(proxy_url: &str) -> Self {
        let keys = ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"];
        let saved = keys
            .into_iter()
            .map(|key| (key, std::env::var_os(key)))
            .collect::<Vec<_>>();
        std::env::set_var("HTTP_PROXY", proxy_url);
        std::env::set_var("HTTPS_PROXY", proxy_url);
        std::env::set_var("ALL_PROXY", proxy_url);
        std::env::set_var("NO_PROXY", "");
        Self { saved }
    }
}

impl Drop for ScopedProxyEnv {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..) {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

fn read_proxy_observations(path: &Path) -> Result<Vec<Observation>, CliError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|e| {
        CliError::failure(format!(
            "cannot read proxy observations {}: {e}",
            path.display()
        ))
    })?;
    let mut observations = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line).map_err(|e| {
            CliError::failure(format!(
                "invalid proxy observation in {} on line {}: {e}",
                path.display(),
                index + 1
            ))
        })?;
        observations.push(observation_from_value(index, &value));
    }
    Ok(observations)
}

fn observation_from_value(index: usize, value: &serde_json::Value) -> Observation {
    Observation {
        flow_id: None,
        proxy_connection_id: string_field(value, "proxy_connection_id")
            .unwrap_or_else(|| format!("proxy-{:08}", index + 1)),
        client_peer: socket_addr_field(value, "client_peer"),
        proxy_local: socket_addr_field(value, "proxy_local"),
        observed_at: string_field(value, "ts").unwrap_or_else(|| rfc3339_utc(SystemTime::now())),
        process_id: None,
        process_image: None,
        role: None,
        attribution: None,
        protocol: string_field(value, "protocol").unwrap_or_else(|| "unknown".to_string()),
        inspectability: Inspectability::from_label(
            string_field(value, "inspectability")
                .as_deref()
                .unwrap_or("unknown"),
        ),
        method: string_field(value, "method"),
        url: string_field(value, "url"),
        status: value
            .get("status")
            .and_then(|v| v.as_u64())
            .and_then(|v| u16::try_from(v).ok()),
        reason: string_field(value, "reason"),
    }
}

fn socket_addr_field(value: &serde_json::Value, key: &str) -> Option<SocketAddr> {
    let endpoint = value.get(key)?;
    let host = endpoint.get("host")?.as_str()?;
    let port = endpoint
        .get("port")?
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())?;
    format!("{host}:{port}")
        .parse()
        .ok()
        .or_else(|| host.parse().ok().map(|ip| SocketAddr::new(ip, port)))
}

fn correlate_observations(observations: &mut [Observation], registry: &FlowRegistry) {
    for observation in observations {
        let (Some(client_peer), Some(proxy_local)) =
            (observation.client_peer, observation.proxy_local)
        else {
            continue;
        };
        let (local, remote) = if client_peer <= proxy_local {
            (client_peer, proxy_local)
        } else {
            (proxy_local, client_peer)
        };
        let key = FlowKey::new(Proto::Tcp, local, remote);
        observation.flow_id = registry.lookup(&key);
        if let Some(attribution) = registry.attribution(&key) {
            observation.process_id = Some(attribution.pid);
            observation.process_image = Some(attribution.process.to_string());
            observation.role = attribution.role.map(|role| role.to_string());
            observation.attribution = Some(
                match attribution.fidelity {
                    Fidelity::Live => "live",
                    Fidelity::Retained => "retained",
                    Fidelity::None => "none",
                }
                .to_string(),
            );
        }
    }
}

fn assign_controlled_flow_ids(observations: &mut [Observation], process_id: Option<u32>) {
    for (index, observation) in observations.iter_mut().enumerate() {
        observation.flow_id = FlowId::new((index + 1) as u64);
        observation.process_id = process_id;
        observation.process_image = Some("client.exe".to_string());
        observation.role = Some("client".to_string());
        observation.attribution = Some("controlled-harness".to_string());
    }
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn serve_controlled_proxy(
    listener: TcpListener,
    path: &Path,
    shutdown: &AtomicBool,
) -> Result<(), String> {
    fs::write(path, []).map_err(|e| e.to_string())?;
    let mut ordinal = 0u64;
    while ordinal < 4 && !shutdown.load(Ordering::Acquire) {
        let (mut stream, peer) = match listener.accept() {
            Ok(connection) => connection,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(err) => return Err(err.to_string()),
        };
        ordinal += 1;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        let mut request = [0u8; 2048];
        let read = stream.read(&mut request).map_err(|e| e.to_string())?;
        let request = String::from_utf8_lossy(&request[..read]);
        let first_line = request.lines().next().unwrap_or_default();
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let target = parts.next().unwrap_or_default();
        let local = stream.local_addr().ok();
        let (protocol, inspectability, method, url, status, reason) = match method {
            "GET" => ("http", "full", Some("GET"), Some(target), Some(200), None),
            "CONNECT" => (
                "https",
                "full",
                Some("CONNECT"),
                Some("https://127.0.0.1/fragcap-controlled/https"),
                Some(200),
                None,
            ),
            "FRAGCAP-METADATA" => (
                "non-http-tls",
                "metadata-only",
                None,
                None,
                None,
                Some("no HTTP semantics observed"),
            ),
            _ => (
                "udp",
                "unsupported",
                None,
                None,
                None,
                Some("protocol is outside MVP inspection scope"),
            ),
        };
        let record = json!({
            "proxy_connection_id": format!("proxy-{ordinal:04}"),
            "client_peer": {"host": peer.ip().to_string(), "port": peer.port()},
            "proxy_local": local.map(|address| json!({
                "host": address.ip().to_string(),
                "port": address.port(),
            })),
            "ts": rfc3339_utc(SystemTime::now()),
            "protocol": protocol,
            "inspectability": inspectability,
            "method": method,
            "url": url,
            "status": status,
            "reason": reason,
        });
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{record}").map_err(|e| e.to_string())?;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run_controlled_target_harness(
    listen_port: u16,
    calibration: Option<CalibrationPhase>,
    execution_timeout: Duration,
) -> Result<u32, CliError> {
    let proxy_url = format!("http://127.0.0.1:{listen_port}");
    let executable = std::env::var_os("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or_else(|| CliError::failure("cannot locate the controlled target executable"))?;
    let mut command = std::process::Command::new(executable);
    command
        .arg("__controlled-target")
        .env("HTTP_PROXY", &proxy_url)
        .env("HTTPS_PROXY", &proxy_url)
        .env("ALL_PROXY", &proxy_url)
        .env("NO_PROXY", "")
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
    let address = proxy
        .strip_prefix("http://")
        .ok_or_else(|| CliError::failure("controlled target received an invalid proxy URL"))?;
    let address: SocketAddr = address
        .parse()
        .map_err(|_| CliError::failure("controlled target received an invalid proxy endpoint"))?;
    if !address.ip().is_loopback() {
        return Err(CliError::failure(
            "controlled target proxy endpoint is not loopback",
        ));
    }
    let fail_after = std::env::var("FRAGCAP_CONTROLLED_TARGET_FAIL_AFTER")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let request_limit = std::env::var("FRAGCAP_CONTROLLED_REQUEST_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(4);
    for (index, request) in [
        "GET http://127.0.0.1/fragcap-controlled/http HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        "CONNECT 127.0.0.1:443 HTTP/1.1\r\nHost: 127.0.0.1:443\r\n\r\n",
        "FRAGCAP-METADATA 127.0.0.1:443 HTTP/1.1\r\n\r\n",
        "FRAGCAP-UNSUPPORTED 127.0.0.1:443 HTTP/1.1\r\n\r\n",
    ]
    .into_iter()
    .take(request_limit)
    .enumerate()
    {
        let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .map_err(|e| CliError::failure(format!("controlled target cannot reach proxy: {e}")))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| CliError::failure(e.to_string()))?;
        stream
            .write_all(request.as_bytes())
            .map_err(|e| CliError::failure(format!("controlled target request failed: {e}")))?;
        let mut response = [0u8; 128];
        let read = stream
            .read(&mut response)
            .map_err(|e| CliError::failure(format!("controlled proxy response failed: {e}")))?;
        if !response[..read].starts_with(b"HTTP/1.1 200") {
            return Err(CliError::failure(
                "controlled proxy returned an unexpected response",
            ));
        }
        if fail_after == Some(index + 1) {
            return Err(CliError::failure(
                "controlled target stopped at the requested test checkpoint",
            ));
        }
    }
    Ok(Exit::SUCCESS)
}

struct BundleContext<'a> {
    session: &'a DeepCaptureSession,
    controlled: bool,
    har_requested: bool,
    key_log_requested: bool,
    observations: &'a [Observation],
    trust: &'a TrustOutcome,
    cleanup: &'a CleanupReport,
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
    write_file(
        ctx.session.bundle.join("application.jsonl"),
        application_jsonl(
            &ctx.session.session_id,
            ctx.session.target_id,
            ctx.observations,
            ctx.session_state,
        )
        .as_bytes(),
    )?;
    let har_produced = ctx.har_requested
        && ctx
            .observations
            .iter()
            .any(|observation| observation.method.is_some() && observation.url.is_some());
    if har_produced {
        write_file(
            ctx.session.bundle.join("http.har"),
            har_json(ctx.observations)?.as_bytes(),
        )?;
    }
    write_file(
        ctx.session.bundle.join("proxy.jsonl"),
        proxy_jsonl(
            &ctx.session.session_id,
            &ctx.session.backend,
            ctx.session.listen_port,
            ctx.session_state,
        )
        .as_bytes(),
    )?;
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
        let _ = fs::remove_file(&key_log_path);
    }
    let mut cleanup_resources = ctx.cleanup.resources.clone();
    cleanup_resources.push(CleanupResource::new(
        "packet-capture",
        if packet_truth_produced {
            "retained"
        } else {
            "not-produced"
        },
        if packet_truth_produced {
            "packet truth retained in the session bundle"
        } else {
            "packet capture failed before packet truth was produced"
        },
    ));
    cleanup_resources.push(CleanupResource::new(
        "tls-key-log",
        if key_log_produced {
            "retained"
        } else if ctx.key_log_requested {
            "not-produced"
        } else {
            "not-requested"
        },
        if key_log_produced {
            "requested analyzer key log retained in the session bundle"
        } else if ctx.key_log_requested {
            "the proxy backend did not produce an analyzer key log"
        } else {
            "analyzer key logging was not requested"
        },
    ));
    cleanup_resources.push(CleanupResource::new(
        "bundle-artifacts",
        "retained",
        "declared session artifacts retained for the operator",
    ));
    cleanup_resources.push(CleanupResource::new(
        "manifest-state",
        "pending",
        "resource cleanup finished; final manifest write is pending",
    ));
    let mut final_cleanup = CleanupReport::new(cleanup_resources);
    write_file(
        ctx.session.bundle.join("cleanup.json"),
        cleanup_json(&ctx.session.session_id, &final_cleanup)?.as_bytes(),
    )?;
    write_file(
        ctx.session.bundle.join("manifest.json"),
        manifest_json(
            ctx,
            &final_cleanup,
            har_produced,
            key_log_produced,
            packet_truth_produced,
        )?
        .as_bytes(),
    )?;
    let manifest_state = final_cleanup
        .resources
        .iter_mut()
        .find(|resource| resource.resource == "manifest-state")
        .expect("the cleanup report always declares manifest state");
    manifest_state.status = "written".to_string();
    manifest_state.reason = "final manifest written after resource cleanup".to_string();
    write_file(
        ctx.session.bundle.join("cleanup.json"),
        cleanup_json(&ctx.session.session_id, &final_cleanup)?.as_bytes(),
    )?;

    let mut produced_artifacts = vec![
        ("application-jsonl", "sensitive"),
        ("proxy-log", "sensitive"),
        ("process-trace", "sensitive"),
        ("compatibility", "ordinary"),
        ("cleanup", "ordinary"),
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

fn artifact_path(role: &str) -> &'static str {
    match role {
        "pcapng" => "capture.fcapng",
        "application-jsonl" => "application.jsonl",
        "proxy-log" => "proxy.jsonl",
        "process-trace" => "process-trace.jsonl",
        "compatibility" => "compatibility.json",
        "cleanup" => "cleanup.json",
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

fn proxy_jsonl(
    session_id: &str,
    backend: &ProxyBackend,
    listen_port: u16,
    session_state: &str,
) -> String {
    let started = json!({
        "session_id": session_id,
        "event": "proxy.started",
        "backend": backend.name,
        "version": backend.version,
        "listen_addr": "127.0.0.1",
        "listen_port": listen_port,
    })
    .to_string();
    let stopped = json!({
        "session_id": session_id,
        "event": "proxy.stopped",
        "backend": backend.name,
        "status": session_state,
    })
    .to_string();
    format!("{started}\n{stopped}\n")
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

fn har_json(observations: &[Observation]) -> Result<String, CliError> {
    let entries: Vec<_> = observations
        .iter()
        .filter(|o| o.method.is_some() && o.url.is_some())
        .map(|observation| crate::har::Entry {
            started_at: &observation.observed_at,
            method: observation.method.as_deref().unwrap_or("GET"),
            url: observation.url.as_deref().unwrap_or("http://127.0.0.1/"),
            status: observation.status.unwrap_or(0),
        })
        .collect();
    crate::har::render(&entries).map_err(|e| CliError::failure(e.to_string()))
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
    key_log_produced: bool,
    packet_truth_produced: bool,
) -> Result<String, CliError> {
    let mut artifacts = vec![
        artifact(
            "application-jsonl",
            "application.jsonl",
            "application-events",
            "sensitive",
            "application/x-ndjson",
            true,
        ),
        artifact(
            "proxy-log",
            "proxy.jsonl",
            "proxy-events",
            "sensitive",
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
            "cleanup",
            "cleanup.json",
            "cleanup-report",
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
        artifacts.insert(
            0,
            artifact(
                "pcapng",
                "capture.fcapng",
                "packet-truth",
                "ordinary",
                "application/x-pcapng",
                true,
            ),
        );
    } else {
        omissions.push(json!({"role":"pcapng","reason":"writer-failed","severity":"error"}));
    }
    if har_produced {
        artifacts.push(artifact(
            "har",
            "http.har",
            "http-projection",
            "sensitive",
            "application/json",
            false,
        ));
    } else if ctx.har_requested {
        omissions.push(json!({"role":"har","reason":"no-http-semantics","severity":"info"}));
    } else {
        omissions.push(json!({"role":"har","reason":"not-requested","severity":"info"}));
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
    } else {
        omissions.push(json!({"role":"tls-key-log","reason":"not-requested","severity":"info"}));
    }
    serde_json::to_string_pretty(&json!({
        "manifest_version": 1,
        "session_id": ctx.session.session_id,
        "mode": "deep-capture",
        "state": ctx.session_state,
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
        "authority": authority,
        "sensitivity": sensitivity,
        "content_type": content_type,
        "required": required,
    })
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

    for (index, observation) in observations.iter().enumerate() {
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
        packet.flow_id = observation.flow_id;
        writer
            .write(&packet)
            .map_err(|e| CliError::failure(format!("cannot write controlled packet: {e}")))?;
    }
    Box::new(writer)
        .finish(&CaptureStats::default())
        .map_err(|e| CliError::failure(format!("cannot finish controlled pcapng: {e}")))
}

fn find_on_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let candidates = std::env::split_paths(&path).flat_map(|dir| {
        #[cfg(windows)]
        {
            vec![dir.join(format!("{program}.exe")), dir.join(program)]
        }
        #[cfg(not(windows))]
        {
            vec![dir.join(program)]
        }
    });
    candidates.into_iter().find(|path| path.is_file())
}

fn command_stdout_with_timeout(
    command: &mut std::process::Command,
    timeout: std::time::Duration,
) -> Result<(std::process::ExitStatus, Vec<u8>), String> {
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                if let Some(mut pipe) = child.stdout.take() {
                    pipe.read_to_end(&mut stdout)
                        .map_err(|err| err.to_string())?;
                }
                return Ok((status, stdout));
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                return Err(format!("timed out after {} ms", timeout.as_millis()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
            Err(err) => return Err(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::{Format, Verbosity};

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

    #[test]
    fn proxy_shutdown_returns_when_its_deadline_expires() {
        let thread = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(100));
            Ok(())
        });
        let mut proxy = RunningProxy {
            events_path: PathBuf::new(),
            ca_cert_path: None,
            key_log_path: None,
            child: None,
            started_child: false,
            controlled_thread: Some(thread),
            controlled_shutdown: Some(Arc::new(AtomicBool::new(false))),
            started_controlled: true,
            listen_port: 0,
            ephemeral_paths: Vec::new(),
            stop_result: None,
        };
        let started = Instant::now();
        let error = proxy
            .stop_with_timeout(Duration::from_millis(1))
            .unwrap_err();
        assert!(started.elapsed() < Duration::from_millis(50));
        assert!(error.contains("did not stop within"));
        assert_eq!(proxy.stop(), Err(error));
    }

    #[test]
    fn missing_proxy_backend_has_a_distinct_readiness_error() {
        let error = mitmdump_backend_from_path(None).unwrap_err();
        assert!(error.to_string().contains("mitmdump"));
        assert!(error.to_string().contains("unavailable"));
    }

    #[test]
    fn proxy_key_logging_is_explicitly_opt_in() {
        let mut disabled = std::process::Command::new("mitmdump");
        configure_key_log(&mut disabled, None);
        let disabled: std::collections::BTreeMap<_, _> = disabled
            .get_envs()
            .map(|(key, value)| (key.to_owned(), value.map(ToOwned::to_owned)))
            .collect();
        assert_eq!(
            disabled.get(std::ffi::OsStr::new("SSLKEYLOGFILE")),
            Some(&None)
        );
        assert_eq!(
            disabled.get(std::ffi::OsStr::new("MITMPROXY_SSLKEYLOGFILE")),
            Some(&None)
        );

        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("bundle");
        fs::create_dir(&bundle).unwrap();
        let key_log_path = prepare_key_log(&bundle).unwrap();
        assert!(key_log_path.is_absolute());
        assert_eq!(fs::metadata(&key_log_path).unwrap().len(), 0);

        let mut enabled = std::process::Command::new("mitmdump");
        configure_key_log(&mut enabled, Some(&key_log_path));
        let enabled: std::collections::BTreeMap<_, _> = enabled
            .get_envs()
            .map(|(key, value)| (key.to_owned(), value.map(ToOwned::to_owned)))
            .collect();
        assert_eq!(
            enabled
                .get(std::ffi::OsStr::new("MITMPROXY_SSLKEYLOGFILE"))
                .and_then(|value| value.as_deref()),
            Some(key_log_path.as_os_str())
        );
    }

    #[test]
    fn live_key_log_path_is_announced_before_session_completion() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("bundle");
        fs::create_dir(&bundle).unwrap();
        let key_log_path = prepare_key_log(&bundle).unwrap();
        let mut output = Vec::new();
        let mut emitter = Emitter::new(&mut output, Format::Human, Verbosity::Normal);

        announce_key_log("session-1", &key_log_path, &mut emitter);
        drop(emitter);

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("TLS key log ready for live analyzers"));
        assert!(output.contains(&key_log_path.display().to_string()));
        assert_eq!(fs::metadata(key_log_path).unwrap().len(), 0);
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
    fn real_capture_accepts_cold_steam_and_direct_launches() {
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
        for unsupported in [
            CompatibilityLaunchCase::SteamProtocolWarm,
            CompatibilityLaunchCase::DirectExeWarm,
            CompatibilityLaunchCase::PublisherLauncherCold,
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

    #[test]
    #[ignore = "local mitmdump demonstration; run explicitly when the backend is installed"]
    fn deep_capture_mitmdump_demo() {
        if find_on_path("mitmdump").is_none() {
            eprintln!("mitmdump is unavailable; demonstration skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("bundle");
        fs::create_dir_all(&bundle).unwrap();
        let args = DeepCaptureArgs {
            selector: Some("sample-target".to_string()),
            target: None,
            id: None,
            catalog_db: None,
            local_db: None,
            launch: true,
            bundle: Some(bundle.clone()),
            duration: None,
            wait: None,
            max_packets: None,
            max_bytes: None,
            interface: Vec::new(),
            no_payload: false,
            trust_ca: true,
            yes: false,
            calibrate: None,
            launch_case: None,
            har: false,
            key_log: true,
            proxy_backend: DeepCaptureProxyArg::Mitmdump,
            controlled_target: false,
        };
        let backend = MitmdumpProxyBackend {
            descriptor: mitmdump_backend().unwrap(),
        };
        let port = select_loopback_port().unwrap();
        let mut proxy = backend.start(&args, &bundle, port).unwrap();
        assert!(loopback_port_is_open(port));
        assert!(proxy
            .key_log_path
            .as_ref()
            .is_some_and(|path| path.is_file()));
        assert!(proxy
            .ca_cert_path
            .as_ref()
            .is_some_and(|path| path.is_file()));
        #[cfg(windows)]
        {
            let manager = WindowsCurrentUserTrustManager {
                ca_cert_path: proxy.ca_cert_path.clone().unwrap(),
                thumbprint: None,
                installed_this_session: false,
            };
            assert_eq!(manager.thumbprint().unwrap().len(), 40);
        }
        proxy.stop().unwrap();
        let process = proxy.cleanup_process();
        assert_eq!(process.status, "succeeded");
        let material = proxy.cleanup_ephemeral();
        assert_eq!(material.status, "succeeded");
    }
}
