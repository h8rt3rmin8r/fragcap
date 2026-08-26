// SPDX-License-Identifier: Apache-2.0

//! `deep-capture`: explicit scoped local proxy inspection for one stored target.
//!
//! This first vertical slice is intentionally narrow. It exercises the product
//! shape and artifact contracts with a controlled target path, refuses unsafe or
//! unknown real-target paths before side effects, and keeps the proxy backend
//! behind a replaceable boundary so a native backend can replace `mitmdump`
//! later without changing the CLI contract.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use fragcap::targets::{
    resolve_id, resolve_positional, CompatibilityEvidenceSource, CompatibilityFact,
    CompatibilityFactKey, CompatibilityLaunchCase, Selection, Store, TargetEntry,
};
use fragcap::{
    CaptureStats, CapturedPacket, Fidelity, FlowId, FlowKey, FlowRegistry, InterfaceDeclaration,
    InterfaceId, LinkType, Payload, PcapngWriter, Proto, RawPacket, Sink, Timestamp,
};
use serde_json::json;

use crate::args::Direction;
use crate::cli::{
    CaptureArgs, ControlledTargetArgs, DeepCaptureArgs, DeepCaptureProxyArg, OfflineArgs, ScopeArg,
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

/// Run `deep-capture`.
pub fn run(args: &DeepCaptureArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    if !args.launch {
        return Err(CliError::usage(
            "Deep Capture requires --launch so scoped proxy configuration is owned by the session",
        ));
    }
    if !(args.trust_ca || args.yes) {
        return Err(CliError::usage(
            "Deep Capture HTTPS inspection requires explicit CA trust confirmation; pass --trust-ca \
             or --yes",
        ));
    }

    let mut store = open_local_store(args.local_db.as_deref())?;
    let target = resolve_target(&store, args)?;
    let target_id = target.id.ok_or_else(|| {
        CliError::failure("resolved target has no local row id; cannot write compatibility facts")
    })?;
    let selected_launch_case = launch_case(&target);

    let facts = store
        .compatibility_facts_for_target(target_id)
        .map_err(|e| CliError::failure(format!("cannot read Deep Capture facts: {e}")))?;
    if args.controlled_target {
        require_controlled_target(&target)?;
    } else {
        require_known_compatibility(&facts, selected_launch_case)?;
    }
    let backend = resolve_backend(args)?;
    let session = DeepCaptureSession::new(
        args,
        target,
        target_id,
        backend.descriptor().clone(),
        selected_launch_case,
    )?;
    validate_bundle_root(&session.bundle)?;

    emitter.event(&Event::DeepCapturePreflight {
        status: "ready".to_string(),
        blockers: 0,
        warnings: 0,
        target: session.target.handle.clone(),
        proxy_backend: session.backend.name.clone(),
        trust_state: "confirmation-present".to_string(),
    });
    emitter.progress("Deep Capture preflight passed");

    fs::create_dir_all(&session.bundle).map_err(|e| {
        CliError::failure(format!(
            "cannot create Deep Capture bundle {}: {e}",
            session.bundle.display()
        ))
    })?;
    let mut proxy = backend.start(args, &session.bundle, session.listen_port)?;

    emitter.event(&Event::DeepCaptureProxyStarted {
        session_id: session.session_id.clone(),
        backend: session.backend.name.clone(),
        version: session.backend.version.clone(),
        listen_addr: "127.0.0.1".to_string(),
        listen_port: session.listen_port,
    });
    emitter.progress(&format!(
        "proxy backend {} ready on 127.0.0.1:{}",
        session.backend.name, session.listen_port
    ));
    if let Some(path) = &proxy.key_log_path {
        announce_key_log(&session.session_id, path, emitter);
    }

    let mut trust_manager = match trust_manager(args, &proxy) {
        Ok(manager) => manager,
        Err(err) => {
            report_early_cleanup(&session, &mut proxy, None, emitter);
            return Err(err);
        }
    };
    let trust = match trust_manager.ensure_trusted(args.trust_ca || args.yes) {
        Ok(trust) => trust,
        Err(err) => {
            report_early_cleanup(&session, &mut proxy, Some(trust_manager.as_mut()), emitter);
            return Err(err);
        }
    };
    emitter.event(&Event::DeepCaptureTrust {
        session_id: session.session_id.clone(),
        state: trust.state.clone(),
        action: trust.action.clone(),
        thumbprint: trust.thumbprint.clone(),
    });
    emitter.progress(&format!("Deep Capture CA trust: {}", trust.action));

    emitter.event(&Event::DeepCaptureLaunch {
        session_id: session.session_id.clone(),
        launch_case: session.launch_case.as_str().to_string(),
        scoped_proxy: true,
        target: session.target.handle.clone(),
    });
    emitter.progress("managed launch prepared with scoped proxy configuration");

    let (observations, operation_failure, controlled_process_id, process_events) =
        if args.controlled_target {
            let target_result = run_controlled_target_harness(session.listen_port);
            let controlled_process_id = target_result.as_ref().ok().copied();
            let stop_result = proxy
                .stop()
                .map_err(|e| CliError::failure(format!("cannot stop controlled proxy: {e}")));
            let failure = target_result.err().or_else(|| stop_result.err());
            match read_proxy_observations(&proxy.events_path) {
                Ok(mut observations) => {
                    assign_controlled_flow_ids(&mut observations, controlled_process_id);
                    (observations, failure, controlled_process_id, Vec::new())
                }
                Err(err) => (
                    Vec::new(),
                    failure.or(Some(err)),
                    controlled_process_id,
                    Vec::new(),
                ),
            }
        } else {
            let (flow_registry, capture_result, process_events) =
                run_real_capture(args, &session.bundle, session.listen_port, emitter);
            let stop_result = proxy
                .stop()
                .map_err(|e| CliError::failure(format!("cannot stop proxy backend: {e}")));
            let failure = capture_result.err().or_else(|| stop_result.err());
            match read_proxy_observations(&proxy.events_path) {
                Ok(mut observations) => {
                    correlate_observations(&mut observations, &flow_registry);
                    (observations, failure, None, process_events)
                }
                Err(err) => (Vec::new(), failure.or(Some(err)), None, process_events),
            }
        };
    let proxy_cleanup = proxy.cleanup_process();
    let port_cleanup = proxy.cleanup_port();
    let trust_cleanup = trust_manager.cleanup();
    if args.key_log {
        proxy.retain_key_log();
    }
    let material_cleanup = proxy.cleanup_ephemeral();
    let cleanup = CleanupReport::new(vec![
        proxy_cleanup,
        port_cleanup,
        trust_cleanup,
        material_cleanup,
    ]);
    let session_state = match operation_failure.as_ref() {
        None => "complete",
        Some(_) if args.controlled_target || session.bundle.join("capture.fcapng").is_file() => {
            "partial"
        }
        Some(_) => "failed",
    };
    for observation in &observations {
        emitter.event(&Event::DeepCaptureApplication {
            session_id: session.session_id.clone(),
            flow_id: observation.flow_id.map(|flow_id| flow_id.to_string()),
            proxy_connection_id: observation.proxy_connection_id.clone(),
            protocol: observation.protocol.clone(),
            inspectability: observation.inspectability.clone(),
        });
    }

    write_bundle(
        &BundleContext {
            session: &session,
            args,
            observations: &observations,
            trust: &trust,
            cleanup: &cleanup,
            session_state,
            controlled_process_id,
            process_events: &process_events,
        },
        emitter,
    )?;
    write_compatibility_facts(
        &mut store,
        session.target_id,
        session.launch_case,
        &session.backend,
        &observations,
        args.controlled_target,
    )?;

    let manifest = "manifest.json".to_string();
    emitter.event(&Event::DeepCaptureComplete {
        session_id: session.session_id.clone(),
        manifest: manifest.clone(),
        status: session_state.to_string(),
        cleanup_status: cleanup.status().to_string(),
        inspectable: observations
            .iter()
            .filter(|o| o.inspectability == "full")
            .count() as u64,
        metadata_only: observations
            .iter()
            .filter(|o| o.inspectability == "metadata-only")
            .count() as u64,
        unsupported: observations
            .iter()
            .filter(|o| o.inspectability == "unsupported")
            .count() as u64,
    });
    emitter.progress(&format!(
        "Deep Capture bundle written to {}",
        session.bundle.join(manifest).display()
    ));

    match operation_failure {
        Some(err) => Err(err),
        None => Ok(Exit::SUCCESS),
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

impl DeepCaptureSession {
    fn new(
        args: &DeepCaptureArgs,
        target: TargetEntry,
        target_id: i64,
        backend: ProxyBackend,
        launch_case: CompatibilityLaunchCase,
    ) -> Result<Self, CliError> {
        let session_id = session_id();
        Ok(Self {
            bundle: bundle_root(args.bundle.as_deref(), &session_id)?,
            launch_case,
            target,
            target_id,
            backend,
            session_id,
            listen_port: select_loopback_port()?,
            started_at: SystemTime::now(),
        })
    }
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
}

#[derive(Clone)]
struct Observation {
    flow_id: Option<FlowId>,
    proxy_connection_id: String,
    client_peer: Option<SocketAddr>,
    proxy_local: Option<SocketAddr>,
    observed_at: String,
    process_id: Option<u32>,
    process_image: Option<String>,
    role: Option<String>,
    attribution: Option<String>,
    protocol: String,
    inspectability: String,
    method: Option<String>,
    url: Option<String>,
    status: Option<u16>,
    reason: Option<String>,
}

impl RunningProxy {
    fn stop(&mut self) -> Result<(), String> {
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
            child.wait().map_err(|err| err.to_string())?;
        }
        if let Some(thread) = self.controlled_thread.take() {
            thread
                .join()
                .map_err(|_| "controlled proxy thread panicked".to_string())??;
        }
        Ok(())
    }

    fn cleanup_process(&mut self) -> CleanupResource {
        match self.stop() {
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
        let mut failures = Vec::new();
        for path in &self.ephemeral_paths {
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

fn report_early_cleanup(
    session: &DeepCaptureSession,
    proxy: &mut RunningProxy,
    trust_manager: Option<&mut dyn TrustManager>,
    emitter: &mut Emitter,
) {
    let trust_cleanup = trust_manager.map_or_else(
        || {
            CleanupResource::new(
                "trust-entry",
                "not-attempted",
                "trust manager initialization failed before a trust change could be attempted",
            )
        },
        TrustManager::cleanup,
    );
    let cleanup = CleanupReport::new(vec![
        proxy.cleanup_process(),
        proxy.cleanup_port(),
        trust_cleanup,
        proxy.cleanup_ephemeral(),
        CleanupResource::new(
            "manifest-state",
            "not-written",
            "session initialization failed before the final manifest could be written",
        ),
    ]);
    if let Ok(content) = cleanup_json(&session.session_id, &cleanup) {
        let _ = write_file(session.bundle.join("cleanup.json"), content.as_bytes());
    }
    for resource in &cleanup.resources {
        emitter.event(&Event::DeepCaptureCleanup {
            session_id: session.session_id.clone(),
            resource: resource.resource.clone(),
            status: resource.status.clone(),
            reason: resource.reason.clone(),
        });
    }
}

trait TrustManager {
    fn ensure_trusted(&mut self, confirmed: bool) -> Result<TrustOutcome, CliError>;
    fn cleanup(&mut self) -> CleanupResource;
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

    fn cleanup(&mut self) -> CleanupResource {
        CleanupResource::new(
            "trust-entry",
            "not-needed",
            "controlled trust manager made no operating-system change",
        )
    }
}

fn trust_manager(
    args: &DeepCaptureArgs,
    proxy: &RunningProxy,
) -> Result<Box<dyn TrustManager>, CliError> {
    if args.controlled_target {
        return Ok(Box::new(ControlledTrustManager));
    }
    let ca_cert_path = proxy.ca_cert_path.clone().ok_or_else(|| {
        CliError::failure("the Deep Capture proxy did not expose session CA material")
    })?;
    platform_trust_manager(ca_cert_path)
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
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Security::Cryptography::{
            CertFreeCertificateContext, CertGetCertificateContextProperty, CryptQueryObject,
            CERT_CONTEXT, CERT_QUERY_CONTENT_FLAG_CERT, CERT_QUERY_FORMAT_FLAG_ALL,
            CERT_QUERY_OBJECT_FILE, CERT_SHA1_HASH_PROP_ID,
        };

        let path: Vec<u16> = self
            .ca_cert_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut context: *mut core::ffi::c_void = core::ptr::null_mut();
        // SAFETY: `path` is a live, null-terminated UTF-16 filename. All unused
        // out parameters are null, and `context` is a live out pointer. The
        // returned certificate context is released below on every path.
        let queried = unsafe {
            CryptQueryObject(
                CERT_QUERY_OBJECT_FILE,
                path.as_ptr().cast(),
                CERT_QUERY_CONTENT_FLAG_CERT,
                CERT_QUERY_FORMAT_FLAG_ALL,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut context,
            )
        };
        if queried == 0 || context.is_null() {
            return Err(CliError::failure(
                "Windows could not parse the Deep Capture session CA certificate",
            ));
        }

        let certificate = context.cast::<CERT_CONTEXT>();
        let mut hash = [0u8; 20];
        let mut hash_len = hash.len() as u32;
        // SAFETY: `certificate` is the live context returned above; `hash` and
        // `hash_len` are live writable buffers of the size passed to the API.
        let read = unsafe {
            CertGetCertificateContextProperty(
                certificate,
                CERT_SHA1_HASH_PROP_ID,
                hash.as_mut_ptr().cast(),
                &mut hash_len,
            )
        };
        // SAFETY: `certificate` is owned by this function and has not been freed.
        unsafe { CertFreeCertificateContext(certificate) };
        if read == 0 || hash_len != hash.len() as u32 {
            return Err(CliError::failure(
                "Windows could not read the Deep Capture session CA thumbprint",
            ));
        }
        Ok(hash.iter().map(|byte| format!("{byte:02X}")).collect())
    }

    fn certutil(&self, args: impl IntoIterator<Item = OsString>) -> Result<bool, String> {
        let certutil = find_on_path("certutil")
            .ok_or_else(|| "certutil.exe is required for current-user CA trust".to_string())?;
        let mut command = std::process::Command::new(certutil);
        command.args(args).stderr(Stdio::null());
        command_stdout_with_timeout(&mut command, Duration::from_secs(15))
            .map(|(status, _)| status.success())
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

    fn cleanup(&mut self) -> CleanupResource {
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
        match self.remove(thumbprint) {
            Ok(true) => match self.is_trusted(thumbprint) {
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

fn require_known_compatibility(
    facts: &[CompatibilityFact],
    launch_case: CompatibilityLaunchCase,
) -> Result<(), CliError> {
    let latest = |key| {
        facts
            .iter()
            .rev()
            .find(|fact| fact.launch_case == Some(launch_case) && fact.key == key)
    };
    let routes = latest(CompatibilityFactKey::ProxyRouting)
        .is_some_and(|fact| !fact.stale && fact.value == "reached-client");
    let propagates = latest(CompatibilityFactKey::ProxyPropagation)
        .is_some_and(|fact| !fact.stale && fact.value == "confirmed");
    if routes && propagates {
        Ok(())
    } else {
        Err(CliError::usage(format!(
            "Deep Capture requires current compatibility facts proving scoped proxy routing \
                 reaches the final client for launch case {}; run compatibility measurement first",
            launch_case.as_str()
        )))
    }
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

fn launch_case(target: &TargetEntry) -> CompatibilityLaunchCase {
    if target
        .anchor
        .as_deref()
        .is_some_and(|anchor| anchor.starts_with("steam:"))
    {
        CompatibilityLaunchCase::SteamProtocolWarm
    } else {
        CompatibilityLaunchCase::DirectExeWarm
    }
}

fn run_real_capture(
    args: &DeepCaptureArgs,
    bundle: &Path,
    listen_port: u16,
    emitter: &mut Emitter,
) -> (Arc<FlowRegistry>, Result<(), CliError>, Vec<String>) {
    let proxy_url = format!("http://127.0.0.1:{listen_port}");
    let _env = ScopedProxyEnv::set(&proxy_url);
    let flow_registry = Arc::new(FlowRegistry::default());
    let capture_args = CaptureArgs {
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
        duration: args.duration,
        wait: args.wait,
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
    };
    emitter.begin_event_capture();
    let result =
        capture::run_with_flow_registry(&capture_args, emitter, Arc::clone(&flow_registry))
            .and_then(|exit| {
                if exit == Exit::SUCCESS {
                    Ok(())
                } else {
                    Err(CliError::failure(format!(
                        "packet capture ended with exit code {}",
                        exit.code()
                    )))
                }
            });
    let process_events = emitter.take_captured_events();
    (flow_registry, result, process_events)
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
        inspectability: string_field(value, "inspectability")
            .unwrap_or_else(|| "unknown".to_string()),
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

fn run_controlled_target_harness(listen_port: u16) -> Result<u32, CliError> {
    let proxy_url = format!("http://127.0.0.1:{listen_port}");
    let _env = ScopedProxyEnv::set(&proxy_url);
    let executable = std::env::var_os("FRAGCAP_CONTROLLED_TARGET_EXECUTABLE")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or_else(|| CliError::failure("cannot locate the controlled target executable"))?;
    let mut child = std::process::Command::new(executable)
        .arg("__controlled-target")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| CliError::failure(format!("cannot start controlled target: {e}")))?;
    let process_id = child.id();
    let status = child
        .wait()
        .map_err(|e| CliError::failure(format!("cannot wait for controlled target: {e}")))?;
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
    for (index, request) in [
        "GET http://127.0.0.1/fragcap-controlled/http HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        "CONNECT 127.0.0.1:443 HTTP/1.1\r\nHost: 127.0.0.1:443\r\n\r\n",
        "FRAGCAP-METADATA 127.0.0.1:443 HTTP/1.1\r\n\r\n",
        "FRAGCAP-UNSUPPORTED 127.0.0.1:443 HTTP/1.1\r\n\r\n",
    ]
    .into_iter()
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
    args: &'a DeepCaptureArgs,
    observations: &'a [Observation],
    trust: &'a TrustOutcome,
    cleanup: &'a CleanupReport,
    session_state: &'a str,
    controlled_process_id: Option<u32>,
    process_events: &'a [String],
}

fn write_bundle(ctx: &BundleContext<'_>, emitter: &mut Emitter) -> Result<(), CliError> {
    let packet_truth = ctx.session.bundle.join("capture.fcapng");
    if ctx.args.controlled_target {
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
    let har_produced = ctx.args.har
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
        compatibility_json(
            &ctx.session.session_id,
            &ctx.session.target,
            &ctx.session.backend,
            ctx.session.launch_case,
            ctx.observations,
        )?
        .as_bytes(),
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
        } else if ctx.args.key_log {
            "not-produced"
        } else {
            "not-requested"
        },
        if key_log_produced {
            "requested analyzer key log retained in the session bundle"
        } else if ctx.args.key_log {
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
        } else if observation.inspectability == "unsupported" {
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
            "inspectability": observation.inspectability,
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

fn compatibility_json(
    session_id: &str,
    target: &TargetEntry,
    backend: &ProxyBackend,
    launch_case: CompatibilityLaunchCase,
    observations: &[Observation],
) -> Result<String, CliError> {
    serde_json::to_string_pretty(&json!({
        "session_id": session_id,
        "target": {
            "id": target.id,
            "handle": target.handle,
        },
        "launch_case": launch_case.as_str(),
        "proxy_backend": backend.name,
        "proxy_backend_version": backend.version,
        "observations": observations.iter().map(|o| {
            json!({
                "protocol": o.protocol,
                "inspectability": o.inspectability,
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
    } else if ctx.args.har {
        omissions.push(json!({"role":"har","reason":"no-http-semantics","severity":"info"}));
    } else {
        omissions.push(json!({"role":"har","reason":"not-requested","severity":"info"}));
    }
    if ctx.args.key_log && key_log_produced {
        artifacts.push(artifact(
            "tls-key-log",
            "tls-keylog.log",
            "analyzer-aid",
            "secret-adjacent",
            "text/plain",
            false,
        ));
    } else if ctx.args.key_log {
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
            "process_roles": if ctx.args.controlled_target { json!(["client"]) } else { json!([]) },
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

fn write_compatibility_facts(
    store: &mut Store,
    target_id: i64,
    launch_case: CompatibilityLaunchCase,
    backend: &ProxyBackend,
    observations: &[Observation],
    controlled: bool,
) -> Result<(), CliError> {
    if !observations.is_empty() {
        let reached_client = observations.iter().any(|observation| {
            observation.flow_id.is_some()
                && observation
                    .role
                    .as_deref()
                    .and_then(compatibility_owner_role)
                    == Some("client")
        });
        for (key, value) in [
            (
                CompatibilityFactKey::ProxyRouting,
                if reached_client {
                    "reached-client"
                } else {
                    "inconclusive"
                },
            ),
            (
                CompatibilityFactKey::ProxyPropagation,
                if reached_client {
                    "confirmed"
                } else {
                    "not-confirmed"
                },
            ),
            (CompatibilityFactKey::ProxyVariableTested, "HTTP_PROXY"),
            (CompatibilityFactKey::ProxyVariableTested, "HTTPS_PROXY"),
            (CompatibilityFactKey::ProxyVariableTested, "ALL_PROXY"),
            (CompatibilityFactKey::ProxyVariableTested, "NO_PROXY"),
        ] {
            insert_fact(
                store,
                target_id,
                key,
                value,
                launch_case,
                backend,
                controlled,
            )?;
        }
    }
    if observations
        .iter()
        .any(|o| o.protocol == "https" && o.inspectability == "full")
    {
        insert_fact(
            store,
            target_id,
            CompatibilityFactKey::TlsTrustBehavior,
            "accepts-local-ca",
            launch_case,
            backend,
            controlled,
        )?;
    }
    let final_roles: BTreeSet<&str> = observations
        .iter()
        .filter_map(|observation| observation.role.as_deref())
        .filter_map(compatibility_owner_role)
        .collect();
    for role in final_roles {
        insert_fact(
            store,
            target_id,
            CompatibilityFactKey::FinalSocketOwnerRole,
            role,
            launch_case,
            backend,
            controlled,
        )?;
    }
    let inspectability: BTreeSet<&str> = observations
        .iter()
        .map(|observation| observation.inspectability.as_str())
        .collect();
    for inspectability in inspectability {
        insert_fact(
            store,
            target_id,
            CompatibilityFactKey::Inspectability,
            inspectability,
            launch_case,
            backend,
            controlled,
        )?;
    }
    let protocols: BTreeSet<&str> = observations
        .iter()
        .map(|observation| observation.protocol.as_str())
        .collect();
    for protocol in protocols {
        insert_fact(
            store,
            target_id,
            CompatibilityFactKey::ProtocolBehavior,
            protocol,
            launch_case,
            backend,
            controlled,
        )?;
    }
    Ok(())
}

fn compatibility_owner_role(role: &str) -> Option<&str> {
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

fn insert_fact(
    store: &mut Store,
    target_id: i64,
    key: CompatibilityFactKey,
    value: &str,
    launch_case: CompatibilityLaunchCase,
    backend: &ProxyBackend,
    controlled: bool,
) -> Result<(), CliError> {
    let mut fact = CompatibilityFact::new(
        target_id,
        key,
        value,
        CompatibilityEvidenceSource::ObservedRun,
    )
    .map_err(|e| CliError::failure(e.to_string()))?;
    fact.launch_case = Some(launch_case);
    fact.observed_at = Some(rfc3339_utc(SystemTime::now()));
    fact.fragcap_version = Some(env!("CARGO_PKG_VERSION").to_string());
    fact.proxy_backend = Some(backend.name.clone());
    fact.proxy_backend_version = Some(backend.version.clone());
    fact.proxy_mode = Some("launch-scoped-env".to_string());
    fact.final_owner_executable = controlled.then(|| "client.exe".to_string());
    fact.final_owner_handoff = false;
    fact.note = Some("scrubbed Deep Capture MVP observation".to_string());
    store
        .insert_compatibility_fact(&fact)
        .map_err(|e| CliError::failure(format!("cannot write Deep Capture facts: {e}")))?;
    Ok(())
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
                let _ = child.wait();
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
            inspectability: "full".to_string(),
            method: Some("GET".to_string()),
            url: Some("https://127.0.0.1/controlled".to_string()),
            status: Some(200),
            reason: None,
        }
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
        let cleanup = manager.cleanup();
        assert_eq!(cleanup.resource, "trust-entry");
        assert_eq!(cleanup.status, "not-needed");
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
        let selected = CompatibilityLaunchCase::DirectExeWarm;
        assert!(require_known_compatibility(&routing_facts(selected, false), selected).is_ok());
        assert!(require_known_compatibility(
            &routing_facts(CompatibilityLaunchCase::DirectExeCold, false),
            selected,
        )
        .is_err());
        assert!(require_known_compatibility(&routing_facts(selected, true), selected).is_err());

        let mut superseded = routing_facts(selected, false);
        let mut latest = CompatibilityFact::new(
            1,
            CompatibilityFactKey::ProxyPropagation,
            "not-confirmed",
            CompatibilityEvidenceSource::ObservedRun,
        )
        .unwrap();
        latest.launch_case = Some(selected);
        superseded.push(latest);
        assert!(require_known_compatibility(&superseded, selected).is_err());
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
