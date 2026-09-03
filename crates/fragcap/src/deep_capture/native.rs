// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

pub use fragcap_proxy::ClientIdentity;
use fragcap_proxy::{
    native_tls_client_config_with_identity, tls_client_config_with_roots_and_identity,
    DestinationPolicy, NativeProxyBackend as RuntimeBackend, NativeProxyConfig, ShutdownReport,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};

use fragcap_core::{Fidelity, FlowKey, FlowRegistry, Proto};

pub use fragcap_proxy::{
    CertificateStore, NativeCertificateStore, TrustController, TrustError, TrustMutation,
    TrustState, CURRENT_USER_ROOT, LOCAL_MACHINE_ROOT,
};

use super::{
    ApplicationConnectionWindow, BackendDescriptor, Budget, CleanupResult, CleanupStatus,
    CompatibilityObservation, CorrelationState, Inspectability, LoopbackEndpoint,
    ProtocolClassification, ProxyBackend, ProxyLease, ProxyRoute, SessionPlan, Stage, StageFailure,
};

/// Finite native runtime limits selected by the library consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeProxyLimits {
    pub max_connections: usize,
    pub per_connection_buffer_bytes: usize,
    pub shutdown_timeout: Duration,
}

impl Default for NativeProxyLimits {
    fn default() -> Self {
        Self {
            max_connections: 128,
            per_connection_buffer_bytes: 16 * 1024,
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// Library-owned native implementation of the Deep Capture proxy seam.
///
/// Production native implementation of the Deep Capture proxy seam.
pub struct NativeProxyAdapter {
    limits: NativeProxyLimits,
    observation_context: NativeObservationContext,
    application_artifact: Option<PathBuf>,
    proxy_lifecycle_artifact: Option<PathBuf>,
    key_log_artifact: Option<PathBuf>,
    capture_payloads: bool,
    client_identity: Option<ClientIdentity>,
    listener_reservation: Option<NativeListenerReservation>,
}

/// Exact listener ownership retained from endpoint selection through startup.
#[derive(Clone, Default)]
pub struct NativeListenerReservation {
    listener: Arc<Mutex<Option<TcpListener>>>,
}

impl NativeListenerReservation {
    /// Bind and retain one exact loopback listener for the pending plan.
    pub fn reserve(
        &self,
        address: SocketAddr,
    ) -> Result<LoopbackEndpoint, super::PreflightRefusal> {
        let requested = LoopbackEndpoint::new(address)?;
        let listener = TcpListener::bind(requested.address()).map_err(|error| {
            super::PreflightRefusal::new(
                "listener-reservation-failed",
                format!("cannot reserve {address}: {error}"),
            )
        })?;
        let endpoint = listener.local_addr().map_err(|error| {
            super::PreflightRefusal::new(
                "listener-address-failed",
                format!("cannot read reserved listener endpoint: {error}"),
            )
        })?;
        let endpoint = LoopbackEndpoint::new(endpoint)?;
        let mut slot = self.listener.lock().map_err(|_| {
            super::PreflightRefusal::new(
                "listener-reservation-poisoned",
                "the exact listener reservation owner is unavailable",
            )
        })?;
        if slot.is_some() {
            return Err(super::PreflightRefusal::new(
                "listener-already-reserved",
                "the session already owns an exact listener reservation",
            ));
        }
        *slot = Some(listener);
        Ok(endpoint)
    }

    fn take(&self, expected: SocketAddr) -> Result<TcpListener, StageFailure> {
        let mut slot = self.listener.lock().map_err(|_| {
            StageFailure::new(
                Stage::ProxyStart,
                "listener-reservation-poisoned",
                "the exact listener reservation owner is unavailable",
            )
        })?;
        let listener = slot.take().ok_or_else(|| {
            StageFailure::new(
                Stage::ProxyStart,
                "listener-reservation-missing",
                "the authorized endpoint no longer has its exact listener reservation",
            )
        })?;
        let actual = listener.local_addr().map_err(|error| {
            StageFailure::new(
                Stage::ProxyStart,
                "listener-address-failed",
                format!("cannot read reserved listener endpoint: {error}"),
            )
        })?;
        if actual != expected {
            return Err(StageFailure::new(
                Stage::ProxyStart,
                "listener-reservation-mismatch",
                format!("reserved listener {actual} does not match authorized endpoint {expected}"),
            ));
        }
        Ok(listener)
    }
}

/// Session-owned bridge between packet truth and native proxy observations.
#[derive(Clone, Debug, Default)]
pub struct NativeObservationContext {
    flow_registry: Arc<FlowRegistry>,
    controlled_process_id: Arc<AtomicU32>,
}

impl NativeObservationContext {
    /// The registry ordinary Capture must populate for this session.
    pub fn flow_registry(&self) -> Arc<FlowRegistry> {
        Arc::clone(&self.flow_registry)
    }

    /// Record the exact controlled child process after it has launched.
    pub fn record_controlled_process_id(&self, process_id: u32) {
        self.controlled_process_id
            .store(process_id, Ordering::Release);
    }

    pub(crate) fn controlled_process_id(&self) -> Option<u32> {
        match self.controlled_process_id.load(Ordering::Acquire) {
            0 => None,
            process_id => Some(process_id),
        }
    }
}

impl NativeProxyAdapter {
    pub fn new(limits: NativeProxyLimits) -> Self {
        Self {
            limits,
            observation_context: NativeObservationContext::default(),
            application_artifact: None,
            proxy_lifecycle_artifact: None,
            key_log_artifact: None,
            capture_payloads: true,
            client_identity: None,
            listener_reservation: None,
        }
    }

    pub fn limits(&self) -> NativeProxyLimits {
        self.limits
    }

    /// Use packet and process truth shared with this session's capture runner.
    pub fn with_observation_context(mut self, context: NativeObservationContext) -> Self {
        self.observation_context = context;
        self
    }

    /// Stream native application observations to this approved artifact path.
    pub fn with_application_artifact(mut self, path: impl Into<PathBuf>) -> Self {
        self.application_artifact = Some(path.into());
        self
    }

    /// Stream the native proxy resource chronology to this protected path.
    pub fn with_proxy_lifecycle_artifact(mut self, path: impl Into<PathBuf>) -> Self {
        self.proxy_lifecycle_artifact = Some(path.into());
        self
    }

    /// Stream client-facing proxy TLS secrets to this approved protected path.
    pub fn with_key_log_artifact(mut self, path: impl Into<PathBuf>) -> Self {
        self.key_log_artifact = Some(path.into());
        self
    }

    /// Select whether application body bytes may be retained.
    pub fn with_payload_capture(mut self, capture_payloads: bool) -> Self {
        self.capture_payloads = capture_payloads;
        self
    }

    /// Use one explicit operator-owned identity for upstream mutual TLS.
    pub fn with_client_identity(mut self, identity: ClientIdentity) -> Self {
        self.client_identity = Some(identity);
        self
    }

    /// Consume the exact listener retained by endpoint selection at startup.
    pub fn with_listener_reservation(mut self, reservation: NativeListenerReservation) -> Self {
        self.listener_reservation = Some(reservation);
        self
    }
}

impl Default for NativeProxyAdapter {
    fn default() -> Self {
        Self::new(NativeProxyLimits::default())
    }
}

impl ProxyBackend for NativeProxyAdapter {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            name: "fragcap-native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn start(
        &mut self,
        plan: &SessionPlan,
        budget: Budget,
    ) -> Result<Box<dyn ProxyLease>, StageFailure> {
        if plan.client_identity != self.client_identity.is_some() {
            return Err(StageFailure::new(
                Stage::ProxyStart,
                "client-identity-plan-mismatch",
                "the authorized plan and configured upstream client identity differ",
            ));
        }
        let endpoint = plan.endpoint.address();
        let protocol = fragcap_proxy::ProtocolLimits {
            capture_payloads: self.capture_payloads,
            ..fragcap_proxy::ProtocolLimits::default()
        };
        let config = NativeProxyConfig::new(
            endpoint,
            self.limits.max_connections,
            self.limits.per_connection_buffer_bytes,
            self.limits.shutdown_timeout,
        )
        .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?
        .with_protocol_limits(protocol)
        .with_session_id(plan.session_id.clone())
        .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
        let controlled_lab = plan
            .controlled
            .then(ControlledLab::start)
            .transpose()
            .map_err(|error| {
                StageFailure::new(Stage::ProxyStart, "controlled-lab-failed", error)
            })?;
        let correlation_context = self.observation_context.clone();
        let controlled = plan.controlled;
        let target_id = plan.target.id;
        let mut application_artifact = self
            .application_artifact
            .as_ref()
            .map(|path| {
                super::ApplicationArtifactLease::open_correlated(
                    path,
                    &plan.session_id,
                    4_096,
                    Arc::new(move |descriptor| {
                        let (
                            flow_id,
                            process_id,
                            process_image,
                            role,
                            attribution,
                            packet_observations,
                            packet_observations_unretained,
                            state,
                            reason,
                        ) = if controlled {
                            (
                                None,
                                correlation_context.controlled_process_id(),
                                Some("client.exe".to_string()),
                                Some("client".to_string()),
                                Some("controlled-harness".to_string()),
                                0,
                                0,
                                CorrelationState::Unavailable,
                                "controlled-harness-has-no-packet-flow".to_string(),
                            )
                        } else {
                            correlate_connection_window(
                                &correlation_context.flow_registry,
                                descriptor,
                            )
                        };
                        super::ApplicationCorrelation {
                            target_id: Some(target_id),
                            flow_id,
                            process_id,
                            process_image,
                            role,
                            attribution,
                            packet_observations,
                            packet_observations_unretained,
                            state: Some(state.as_str().to_string()),
                            reason: Some(reason),
                        }
                    }),
                )
                .map_err(|error| {
                    StageFailure::new(
                        Stage::ProxyStart,
                        "application-writer-open-failed",
                        error.to_string(),
                    )
                })
            })
            .transpose()?;
        let mut proxy_lifecycle = self
            .proxy_lifecycle_artifact
            .as_ref()
            .map(|path| {
                super::ProxyLifecycleLease::open_with_listener(
                    path,
                    &plan.session_id,
                    4_096,
                    plan.endpoint.address().to_string(),
                )
                .map_err(|error| {
                    StageFailure::new(
                        Stage::ProxyStart,
                        "proxy-lifecycle-open-failed",
                        error.to_string(),
                    )
                })
            })
            .transpose()?;
        let mut backend = RuntimeBackend::new(config);
        if let Some(reservation) = &self.listener_reservation {
            backend = backend.with_reserved_listener(reservation.take(endpoint)?);
        }
        let key_log = if plan.artifacts.key_log {
            let path = self.key_log_artifact.as_ref().ok_or_else(|| {
                StageFailure::new(
                    Stage::ProxyStart,
                    "key-log-path-missing",
                    "the authorized plan requested a key log without a protected artifact path",
                )
            })?;
            let file = super::open_sensitive_file(path).map_err(|error| {
                StageFailure::new(Stage::ProxyStart, "key-log-open-failed", error.to_string())
            })?;
            Some(Arc::new(fragcap_proxy::SessionKeyLog::new(file)))
        } else {
            None
        };
        if let Some(key_log) = &key_log {
            backend = backend.with_key_log(Arc::clone(key_log));
        }
        let mut event_sinks = Vec::new();
        if let Some(artifact) = &application_artifact {
            event_sinks.push(artifact.sink());
        }
        if let Some(lifecycle) = &proxy_lifecycle {
            event_sinks.push(lifecycle.sink());
        }
        if !event_sinks.is_empty() {
            backend = backend.with_application_event_sink(Arc::new(
                fragcap_proxy::FanoutApplicationEventSink::new(event_sinks),
            ));
        }
        if controlled_lab.is_none() && self.client_identity.is_some() {
            let tls = native_tls_client_config_with_identity(self.client_identity.as_ref())
                .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?;
            backend = backend.with_tls_client_config(tls);
        }
        if let Some(lab) = &controlled_lab {
            let mut policy = DestinationPolicy::new(endpoint);
            policy.grant_for_test(lab.http_origin);
            policy.grant_for_test(lab.https_origin);
            let mut roots = rustls::RootCertStore::empty();
            roots.add(lab.origin_certificate.clone()).map_err(|error| {
                StageFailure::new(
                    Stage::ProxyStart,
                    "controlled-root-failed",
                    error.to_string(),
                )
            })?;
            let tls =
                tls_client_config_with_roots_and_identity(roots, self.client_identity.as_ref())
                    .map_err(|error| {
                        StageFailure::new(Stage::ProxyStart, error.code, error.detail)
                    })?;
            backend = backend
                .with_destination_policy(policy)
                .with_tls_client_config(tls);
        }
        let lease = match backend.start(budget.remaining()) {
            Ok(lease) => lease,
            Err(error) => {
                if let Some(lifecycle) = proxy_lifecycle.as_mut() {
                    let _ = lifecycle.listener_failed(&error.detail);
                    let _ = lifecycle.finish();
                }
                return Err(StageFailure::new(
                    Stage::ProxyStart,
                    error.code,
                    error.detail,
                ));
            }
        };
        if let Some(lifecycle) = &proxy_lifecycle {
            lifecycle
                .listener_started(lease.endpoint().to_string())
                .map_err(|error| {
                    StageFailure::new(
                        Stage::ProxyStart,
                        "proxy-lifecycle-listener-start-failed",
                        error.to_string(),
                    )
                })?;
        }
        Ok(Box::new(NativeProxyLease {
            lease,
            controlled: plan.controlled,
            controlled_lab,
            observation_context: self.observation_context.clone(),
            application_artifact: application_artifact.take(),
            proxy_lifecycle: proxy_lifecycle.take(),
            key_log,
        }))
    }
}

struct NativeProxyLease {
    lease: fragcap_proxy::NativeProxyLease,
    controlled: bool,
    controlled_lab: Option<ControlledLab>,
    observation_context: NativeObservationContext,
    application_artifact: Option<super::ApplicationArtifactLease>,
    proxy_lifecycle: Option<super::ProxyLifecycleLease>,
    key_log: Option<Arc<fragcap_proxy::SessionKeyLog>>,
}

impl ProxyLease for NativeProxyLease {
    fn route(&self) -> Result<ProxyRoute, StageFailure> {
        let endpoint = self.lease.endpoint();
        Ok(ProxyRoute::new(
            LoopbackEndpoint::new(endpoint)
                .map_err(|error| StageFailure::new(Stage::ProxyStart, error.code, error.detail))?,
            (
                self.lease.proxy_url(),
                self.lease.capability_proof().socks5h_url(endpoint),
            ),
            self.lease.capability_proof().proxy_authorization(),
            self.lease.ca_der().to_vec(),
            self.lease.ca_sha1_thumbprint().to_string(),
            self.lease.authority_generation(),
            self.controlled_lab
                .as_ref()
                .map(|lab| (lab.http_origin, lab.https_origin)),
        ))
    }

    fn observations(
        &mut self,
        budget: Budget,
    ) -> Result<Vec<CompatibilityObservation>, StageFailure> {
        let observation = self
            .lease
            .observation(budget.remaining())
            .map_err(|error| StageFailure::new(Stage::Observe, error.code, error.detail))?;
        Ok(observation
            .application
            .into_iter()
            .map(|value| {
                let (
                    flow_id,
                    process_id,
                    process_image,
                    role,
                    attribution,
                    packet_observations,
                    packet_observations_unretained,
                    correlation_state,
                    correlation_reason,
                ) = if self.controlled {
                    (
                        None,
                        self.observation_context.controlled_process_id(),
                        Some("client.exe".to_string()),
                        Some("client".to_string()),
                        Some("controlled-harness".to_string()),
                        0,
                        0,
                        CorrelationState::Unavailable,
                        "controlled-harness-has-no-packet-flow".to_string(),
                    )
                } else {
                    correlate_native_observation(
                        &self.observation_context.flow_registry,
                        value.client_peer,
                        value.proxy_local,
                        value.connection_opened_at_ns,
                        value.connection_closed_at_ns,
                    )
                };
                let protocol = match value.protocol.as_str() {
                    "connect" | "tls" => "https".to_string(),
                    _ => value.protocol,
                };
                let inspectability = Inspectability::from_label(value.inspectability);
                let reason = value.reason;
                let classification = ProtocolClassification::from_proxy_evidence(
                    &protocol,
                    value.inspectability,
                    reason.as_deref(),
                );
                CompatibilityObservation {
                    flow_id,
                    proxy_connection_id: value.connection_id.to_string(),
                    client_peer: Some(value.client_peer),
                    proxy_local: Some(value.proxy_local),
                    observed_at: value.timestamp_ns.to_string(),
                    process_id,
                    process_image,
                    role,
                    attribution,
                    packet_observations,
                    packet_observations_unretained,
                    correlation_state,
                    correlation_reason,
                    protocol,
                    inspectability,
                    method: value.method,
                    url: value.url,
                    status: value.status,
                    reason,
                    classification,
                }
            })
            .collect())
    }

    fn stop(&mut self, budget: Budget) -> CleanupResult {
        let report = self.lease.stop(budget.remaining());
        let mut result = cleanup_result("native-proxy-listener", &report);
        if report.is_clean() {
            if let Some(artifact) = self.application_artifact.as_mut() {
                if let Err(error) = artifact.finish() {
                    result.status = CleanupStatus::Failed;
                    result.reason = format!("application writer failed: {error}");
                }
            }
            if let Some(lifecycle) = self.proxy_lifecycle.as_mut() {
                if let Err(error) = lifecycle.finish() {
                    result.status = CleanupStatus::Failed;
                    result.reason = format!("proxy lifecycle writer failed: {error}");
                }
            }
        }
        result
    }

    fn cleanup(&mut self, budget: Budget) -> Vec<CleanupResult> {
        let report = self.lease.cleanup(budget.remaining());
        let mut results = vec![cleanup_result("native-proxy-runtime", &report)];
        if let Some(key_log) = self.key_log.take() {
            let status = key_log.status();
            results.push(CleanupResult {
                resource: "tls-key-log".to_string(),
                status: if status.failure.is_none() {
                    CleanupStatus::Released
                } else {
                    CleanupStatus::Failed
                },
                reason: format!(
                    "records={}, bytes={}, flushes={}, failure={}",
                    status.records,
                    status.bytes,
                    status.flushes,
                    status.failure.as_deref().unwrap_or("none")
                ),
            });
        }
        if let Some(mut artifact) = self.application_artifact.take() {
            let status = artifact.finish();
            results.push(CleanupResult {
                resource: "application-writer".to_string(),
                status: if status.is_ok() {
                    CleanupStatus::Released
                } else {
                    CleanupStatus::Failed
                },
                reason: status.err().map_or_else(
                    || "application stream finalized".to_string(),
                    |error| error.to_string(),
                ),
            });
        }
        if let Some(mut lifecycle) = self.proxy_lifecycle.take() {
            let result = lifecycle.finish();
            results.push(CleanupResult {
                resource: "proxy-lifecycle-writer".to_string(),
                status: if result.is_ok() {
                    CleanupStatus::Released
                } else {
                    CleanupStatus::Failed
                },
                reason: result.err().map_or_else(
                    || "proxy lifecycle writer completed".to_string(),
                    |error| error.to_string(),
                ),
            });
        }
        if let Some(mut lab) = self.controlled_lab.take() {
            results.push(lab.cleanup());
        }
        results
    }
}

type NativeCorrelation = (
    Option<fragcap_core::FlowId>,
    Option<u32>,
    Option<String>,
    Option<String>,
    Option<String>,
    u64,
    u64,
    CorrelationState,
    String,
);

fn correlate_connection_window(
    registry: &FlowRegistry,
    window: &ApplicationConnectionWindow,
) -> NativeCorrelation {
    let Some(closed_at_ns) = window.closed_at_ns else {
        return (
            None,
            None,
            None,
            None,
            None,
            0,
            0,
            CorrelationState::Unavailable,
            "connection-terminal-not-observed".to_string(),
        );
    };
    correlate_native_observation(
        registry,
        window.descriptor.client_peer,
        window.descriptor.proxy_local,
        window.opened_at_ns,
        closed_at_ns,
    )
}

fn correlate_native_observation(
    registry: &FlowRegistry,
    client_peer: SocketAddr,
    proxy_local: SocketAddr,
    opened_at_ns: u64,
    closed_at_ns: u64,
) -> NativeCorrelation {
    let (local, remote) = if client_peer <= proxy_local {
        (client_peer, proxy_local)
    } else {
        (proxy_local, client_peer)
    };
    let key = FlowKey::new(Proto::Tcp, local, remote);
    let Some(summary) = registry.summary(&key) else {
        return (
            None,
            None,
            None,
            None,
            None,
            0,
            0,
            CorrelationState::Unavailable,
            "packet-flow-not-observed".to_string(),
        );
    };
    if summary.unretained_observations > 0 {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            summary.observations.len() as u64,
            summary.unretained_observations,
            CorrelationState::Unavailable,
            "packet-history-bound-exceeded".to_string(),
        );
    }
    if summary.global_unretained_observations > 0 {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            summary.observations.len() as u64,
            summary.unretained_observations,
            CorrelationState::Unavailable,
            "capture-buffer-history-incomplete".to_string(),
        );
    }
    let overlapping = summary
        .observations
        .iter()
        .filter(|observation| {
            let timestamp = observation.timestamp.as_nanos();
            timestamp >= 0
                && (timestamp as u64) >= opened_at_ns
                && (timestamp as u64) <= closed_at_ns
        })
        .collect::<Vec<_>>();
    if overlapping.is_empty() {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            0,
            0,
            CorrelationState::Unavailable,
            "packet-flow-has-no-overlapping-observation".to_string(),
        );
    }
    let has_unattributed = overlapping
        .iter()
        .any(|observation| observation.attribution.is_none());
    let mut owners = overlapping
        .iter()
        .filter_map(|observation| observation.attribution.as_ref())
        .collect::<Vec<_>>();
    owners.sort_by_key(|owner| {
        let fidelity = match owner.fidelity {
            Fidelity::Retained => 0,
            Fidelity::Live => 1,
            Fidelity::None => 2,
        };
        (
            owner.pid,
            owner.process.clone(),
            owner.role.clone(),
            fidelity,
        )
    });
    owners.dedup_by(|left, right| {
        left.pid == right.pid && left.process == right.process && left.role == right.role
    });
    let Some(attribution) = owners.first() else {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            overlapping.len() as u64,
            0,
            CorrelationState::FlowOnly,
            "packet-flow-unattributed".to_string(),
        );
    };
    if owners.len() != 1 {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            overlapping.len() as u64,
            0,
            CorrelationState::Ambiguous,
            "conflicting-packet-owners".to_string(),
        );
    }
    if has_unattributed {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            overlapping.len() as u64,
            0,
            CorrelationState::Ambiguous,
            "packet-owner-partially-unresolved".to_string(),
        );
    }
    (
        Some(summary.id),
        Some(attribution.pid),
        Some(attribution.process.to_string()),
        attribution.role.as_ref().map(|role| role.to_string()),
        Some(
            match attribution.fidelity {
                Fidelity::Live => "live",
                Fidelity::Retained => "retained",
                Fidelity::None => "none",
            }
            .to_string(),
        ),
        overlapping.len() as u64,
        0,
        CorrelationState::Matched,
        "exact-flow-and-owner".to_string(),
    )
}

struct ControlledLab {
    http_origin: SocketAddr,
    https_origin: SocketAddr,
    origin_certificate: CertificateDer<'static>,
    shutdown: Arc<AtomicBool>,
    workers: Vec<JoinHandle<Result<(), String>>>,
}

impl ControlledLab {
    fn start() -> Result<Self, String> {
        let http =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|error| error.to_string())?;
        let https =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|error| error.to_string())?;
        http.set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        https
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let http_origin = http.local_addr().map_err(|error| error.to_string())?;
        let https_origin = https.local_addr().map_err(|error| error.to_string())?;
        let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
            .map_err(|error| error.to_string())?;
        let origin_certificate = CertificateDer::from(generated.cert);
        let private = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut tls_config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|error| error.to_string())?
            .with_no_client_auth()
            .with_single_cert(vec![origin_certificate.clone()], private.into())
            .map_err(|error| error.to_string())?;
        tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let shutdown = Arc::new(AtomicBool::new(false));
        let http_stop = Arc::clone(&shutdown);
        let http_worker = std::thread::spawn(move || serve_origin(http, http_stop, None));
        let https_stop = Arc::clone(&shutdown);
        let https_worker =
            std::thread::spawn(move || serve_origin(https, https_stop, Some(Arc::new(tls_config))));
        Ok(Self {
            http_origin,
            https_origin,
            origin_certificate,
            shutdown,
            workers: vec![http_worker, https_worker],
        })
    }

    fn cleanup(&mut self) -> CleanupResult {
        self.shutdown.store(true, Ordering::Release);
        let mut failed = Vec::new();
        for worker in self.workers.drain(..) {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => failed.push(error),
                Err(_) => failed.push("controlled origin thread panicked".to_string()),
            }
        }
        CleanupResult {
            resource: "controlled-protocol-lab".to_string(),
            status: if failed.is_empty() {
                CleanupStatus::Released
            } else {
                CleanupStatus::Failed
            },
            reason: if failed.is_empty() {
                "released".to_string()
            } else {
                failed.join("; ")
            },
        }
    }
}

fn serve_origin(
    listener: TcpListener,
    shutdown: Arc<AtomicBool>,
    tls: Option<Arc<rustls::ServerConfig>>,
) -> Result<(), String> {
    while !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false).map_err(|e| e.to_string())?;
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|e| e.to_string())?;
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .map_err(|e| e.to_string())?;
                if let Some(config) = &tls {
                    let connection = rustls::ServerConnection::new(Arc::clone(config))
                        .map_err(|e| e.to_string())?;
                    let mut stream = rustls::StreamOwned::new(connection, stream);
                    answer_origin(&mut stream, b"SECURE")?;
                } else {
                    let mut stream = stream;
                    answer_origin(&mut stream, b"PLAIN")?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn answer_origin(stream: &mut (impl Read + Write), body: &[u8]) -> Result<(), String> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        head.push(byte[0]);
        if head.len() > 16 * 1024 {
            return Err("controlled origin request head exceeded limit".to_string());
        }
    }
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

/// Exercise the native route from the hidden controlled child process.
#[doc(hidden)]
pub fn run_controlled_native_requests(
    endpoint: SocketAddr,
    authorization: &str,
    http_origin: SocketAddr,
    https_origin: SocketAddr,
    ca_der: Vec<u8>,
    include_tls: bool,
) -> Result<(), String> {
    let mut plain =
        TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    plain
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    write!(plain, "GET http://{http_origin}/plain HTTP/1.1\r\nHost: {http_origin}\r\nProxy-Authorization: {authorization}\r\nConnection: close\r\n\r\n").map_err(|e| e.to_string())?;
    let mut response = String::new();
    plain
        .read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    if !response.starts_with("HTTP/1.1 200") || !response.ends_with("PLAIN") {
        return Err("native HTTP controlled request returned an unexpected response".to_string());
    }
    if !include_tls {
        return Ok(());
    }
    let mut tcp =
        TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    tcp.set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    write!(tcp, "CONNECT localhost:{} HTTP/1.1\r\nHost: localhost:{}\r\nProxy-Authorization: {authorization}\r\n\r\n", https_origin.port(), https_origin.port()).map_err(|e| e.to_string())?;
    let head = read_head(&mut tcp)?;
    if head != b"HTTP/1.1 200 Connection Established\r\n\r\n" {
        return Err("native CONNECT returned an unexpected response".to_string());
    }
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(ca_der))
        .map_err(|e| e.to_string())?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let connection = rustls::ClientConnection::new(
        Arc::new(config),
        ServerName::try_from("localhost")
            .map_err(|e| e.to_string())?
            .to_owned(),
    )
    .map_err(|e| e.to_string())?;
    let mut tls = rustls::StreamOwned::new(connection, tcp);
    write!(
        tls,
        "GET /secure HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        https_origin.port()
    )
    .map_err(|e| e.to_string())?;
    let mut response = String::new();
    tls.read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    if !response.starts_with("HTTP/1.1 200") || !response.ends_with("SECURE") {
        return Err("native HTTPS controlled request returned an unexpected response".to_string());
    }
    Ok(())
}

fn read_head(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut head = Vec::new();
    let mut byte = [0_u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        head.push(byte[0]);
        if head.len() > 16 * 1024 {
            return Err("proxy response head exceeded limit".to_string());
        }
    }
    Ok(head)
}

fn cleanup_result(resource: &str, report: &ShutdownReport) -> CleanupResult {
    let status = if report.is_clean() {
        CleanupStatus::Released
    } else if report.residue || report.incomplete_tasks > 0 {
        CleanupStatus::TimedOut
    } else {
        CleanupStatus::Failed
    };
    CleanupResult {
        resource: resource.to_string(),
        status,
        reason: format!(
            "accepted={}, completed={}, failed={}, forced={}, incomplete={}, failures={}",
            report.observation.accepted_connections,
            report.observation.completed_connections,
            report.observation.failed_connections,
            report.observation.forced_connections,
            report.incomplete_tasks,
            report.observation.failures.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener_reservation_owns_the_selected_socket_until_consumed() {
        let reservation = NativeListenerReservation::default();
        let endpoint = reservation.reserve("127.0.0.1:0".parse().unwrap()).unwrap();
        assert!(TcpListener::bind(endpoint.address()).is_err());

        let listener = reservation.take(endpoint.address()).unwrap();
        assert_eq!(listener.local_addr().unwrap(), endpoint.address());
    }
    use fragcap_core::{Attribution, Timestamp};

    #[test]
    fn native_observation_correlation_restores_packet_and_process_truth() {
        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        registry.observe(
            key,
            Some(&Attribution::new(77, "game.exe", Fidelity::Live).with_role("client")),
        );

        let correlated = correlate_native_observation(&registry, client, proxy, 0, u64::MAX);
        assert_eq!(correlated.0, registry.lookup(&key));
        assert_eq!(correlated.1, Some(77));
        assert_eq!(correlated.2.as_deref(), Some("game.exe"));
        assert_eq!(correlated.3.as_deref(), Some("client"));
        assert_eq!(correlated.4.as_deref(), Some("live"));
    }

    #[test]
    fn mixed_live_and_retained_evidence_reports_the_weaker_fidelity() {
        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        registry.observe_at(
            key,
            Timestamp::from_nanos(10),
            Some(&Attribution::new(77, "game.exe", Fidelity::Live).with_role("client")),
        );
        registry.observe_at(
            key,
            Timestamp::from_nanos(20),
            Some(&Attribution::new(77, "game.exe", Fidelity::Retained).with_role("client")),
        );

        let correlated = correlate_native_observation(&registry, client, proxy, 0, 30);
        assert_eq!(correlated.4.as_deref(), Some("retained"));
    }

    #[test]
    fn unmatched_native_observation_does_not_invent_identity() {
        let registry = FlowRegistry::default();
        let correlated = correlate_native_observation(
            &registry,
            "127.0.0.1:41000".parse().unwrap(),
            "127.0.0.1:42000".parse().unwrap(),
            0,
            u64::MAX,
        );
        assert_eq!(
            correlated,
            (
                None,
                None,
                None,
                None,
                None,
                0,
                0,
                CorrelationState::Unavailable,
                "packet-flow-not-observed".to_string(),
            )
        );
    }

    #[test]
    fn reused_endpoint_is_reconciled_only_inside_the_connection_window() {
        use fragcap_core::Timestamp;

        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        registry.observe_at(
            key,
            Timestamp::from_nanos(10),
            Some(&Attribution::new(11, "old.exe", Fidelity::Live)),
        );
        registry.observe_at(
            key,
            Timestamp::from_nanos(30),
            Some(&Attribution::new(22, "new.exe", Fidelity::Live)),
        );

        let old = correlate_native_observation(&registry, client, proxy, 5, 20);
        let new = correlate_native_observation(&registry, client, proxy, 25, 40);

        assert_eq!(old.1, Some(11));
        assert_eq!(new.1, Some(22));
        assert_eq!(old.7, CorrelationState::Matched);
        assert_eq!(new.7, CorrelationState::Matched);
    }

    #[test]
    fn conflicting_owners_in_one_window_are_reported_as_ambiguous() {
        use fragcap_core::Timestamp;

        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        for (timestamp, pid) in [(10, 11), (20, 22)] {
            registry.observe_at(
                key,
                Timestamp::from_nanos(timestamp),
                Some(&Attribution::new(pid, format!("{pid}.exe"), Fidelity::Live)),
            );
        }

        let result = correlate_native_observation(&registry, client, proxy, 5, 25);
        assert_eq!(result.7, CorrelationState::Ambiguous);
        assert_eq!(result.8, "conflicting-packet-owners");
        assert!(result.1.is_none());
    }

    #[test]
    fn bounded_history_loss_prevents_a_confident_join() {
        use fragcap_core::Timestamp;

        let registry = FlowRegistry::with_history_limit(1);
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        let owner = Attribution::new(11, "game.exe", Fidelity::Live);
        registry.observe_at(key, Timestamp::from_nanos(10), Some(&owner));
        registry.observe_at(key, Timestamp::from_nanos(20), Some(&owner));

        let result = correlate_native_observation(&registry, client, proxy, 5, 25);
        assert_eq!(result.5, 1);
        assert_eq!(result.6, 1);
        assert_eq!(result.7, CorrelationState::Unavailable);
        assert_eq!(result.8, "packet-history-bound-exceeded");
    }

    #[test]
    fn global_buffer_loss_is_uncertainty_not_per_flow_loss() {
        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        registry.observe_at(
            FlowKey::new(Proto::Tcp, client, proxy),
            Timestamp::from_nanos(10),
            Some(&Attribution::new(11, "game.exe", Fidelity::Live)),
        );
        registry.mark_globally_unretained();

        let result = correlate_native_observation(&registry, client, proxy, 5, 25);
        assert_eq!(result.6, 0);
        assert_eq!(result.7, CorrelationState::Unavailable);
        assert_eq!(result.8, "capture-buffer-history-incomplete");
    }

    #[test]
    fn missing_connection_terminal_refuses_correlation() {
        use fragcap_core::Timestamp;

        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        registry.observe_at(
            FlowKey::new(Proto::Tcp, client, proxy),
            Timestamp::from_nanos(15),
            Some(&Attribution::new(11, "game.exe", Fidelity::Live)),
        );
        let result = correlate_connection_window(
            &registry,
            &ApplicationConnectionWindow {
                descriptor: fragcap_proxy::ConnectionDescriptor {
                    transport: "tcp",
                    client_peer: client,
                    proxy_local: proxy,
                },
                opened_at_ns: 10,
                closed_at_ns: None,
            },
        );

        assert_eq!(result.7, CorrelationState::Unavailable);
        assert_eq!(result.8, "connection-terminal-not-observed");
        assert!(result.0.is_none());
        assert!(result.1.is_none());
    }

    #[test]
    fn ipv6_endpoint_order_and_observation_permutation_do_not_change_the_join() {
        use fragcap_core::Timestamp;

        let client: SocketAddr = "[::1]:41000".parse().unwrap();
        let proxy: SocketAddr = "[::1]:42000".parse().unwrap();
        let owner = Attribution::new(77, "game.exe", Fidelity::Retained).with_role("client");
        let mut results = Vec::new();
        for observations in [[(10, &owner), (20, &owner)], [(20, &owner), (10, &owner)]] {
            let registry = FlowRegistry::default();
            let key = FlowKey::new(Proto::Tcp, client, proxy);
            for (timestamp, attribution) in observations {
                registry.observe_at(key, Timestamp::from_nanos(timestamp), Some(attribution));
            }
            results.push(correlate_native_observation(
                &registry, proxy, client, 5, 25,
            ));
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0].4.as_deref(), Some("retained"));
        assert_eq!(results[0].7, CorrelationState::Matched);
    }

    #[test]
    fn controlled_process_identity_is_the_explicit_child() {
        let context = NativeObservationContext::default();
        assert_eq!(context.controlled_process_id(), None);
        context.record_controlled_process_id(4242);
        assert_eq!(context.controlled_process_id(), Some(4242));
    }
}
