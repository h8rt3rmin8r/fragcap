// SPDX-License-Identifier: Apache-2.0

#[cfg(all(windows, feature = "direct-route"))]
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
#[cfg(all(windows, feature = "socket-table"))]
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

pub use fragcap_proxy::ClientIdentity;
use fragcap_proxy::{
    native_tls_client_config_with_identity, tls_client_config_with_roots_and_identity,
    DestinationPolicy, NativeProxyBackend as RuntimeBackend, NativeProxyConfig, ShutdownReport,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};

#[cfg(any(test, all(windows, feature = "socket-table")))]
use fragcap_attr::{socket_match_rank, SocketTable};
#[cfg(all(windows, feature = "socket-table"))]
use fragcap_attr::{IpHelperTable, SocketTableSource};
use fragcap_core::{Fidelity, FlowKey, FlowRegistry, ProcessEvent, Proto, Timestamp};
use fragcap_proxy::ConnectionDescriptor;

pub use fragcap_proxy::{
    CertificateStore, NativeCertificateStore, TrustController, TrustError, TrustMutation,
    TrustState, CURRENT_USER_ROOT, LOCAL_MACHINE_ROOT,
};

use super::{
    ApplicationConnectionWindow, BackendDescriptor, Budget, ClassificationSummary, CleanupResult,
    CleanupStatus, CompatibilityObservation, CorrelationState, Inspectability, LoopbackEndpoint,
    ObservationDrain, ProtocolClassification, ProxyBackend, ProxyLease, ProxyRoute, SessionPlan,
    Stage, StageFailure,
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
    prepared_authority: Option<PreparedNativeAuthority>,
    inspection_decider: Option<Arc<dyn fragcap_proxy::InspectionDecider>>,
}

static NEXT_PREPARED_AUTHORITY_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Process-local session authority prepared for exact plan review.
pub struct PreparedNativeAuthority {
    authority: Arc<fragcap_proxy::SessionCertificateAuthority>,
    lifetime: Duration,
}

impl std::fmt::Debug for PreparedNativeAuthority {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedNativeAuthority")
            .field("generation", &self.authority.generation())
            .field("sha1_thumbprint", &self.authority.sha1_thumbprint())
            .field("sha256_fingerprint", &self.authority.sha256_fingerprint())
            .field("lifetime", &self.lifetime)
            .finish()
    }
}

impl PreparedNativeAuthority {
    pub fn generation(&self) -> u64 {
        self.authority.generation()
    }

    pub fn der(&self) -> &[u8] {
        self.authority.der().as_ref()
    }

    pub fn not_before(&self) -> SystemTime {
        self.authority.not_before()
    }

    pub fn not_after(&self) -> SystemTime {
        self.authority.not_after()
    }

    #[cfg(windows)]
    pub fn persist_installation_key(
        &self,
        path: &std::path::Path,
    ) -> Result<(), super::PreflightRefusal> {
        self.authority
            .persist_installation_key(path)
            .map_err(|error| super::PreflightRefusal::new(error.code, error.detail))
    }

    pub fn sha1_thumbprint(&self) -> &str {
        self.authority.sha1_thumbprint()
    }

    pub fn sha256_fingerprint(&self) -> &str {
        self.authority.sha256_fingerprint()
    }

    pub fn lifetime(&self) -> Duration {
        self.lifetime
    }

    fn into_inner(self) -> Arc<fragcap_proxy::SessionCertificateAuthority> {
        self.authority
    }
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

#[cfg(all(windows, feature = "socket-table"))]
const SOCKET_INCEPTION_QUEUE_CAPACITY: usize = 128;

#[cfg(all(windows, feature = "socket-table"))]
enum SocketInceptionRequest {
    Lookup {
        descriptor: ConnectionDescriptor,
        opened_at_ns: u64,
        result: Arc<Mutex<super::ConnectionInception>>,
    },
    Flush(mpsc::Sender<()>),
}

#[cfg(all(windows, feature = "socket-table"))]
#[derive(Debug)]
struct SocketInceptionCollector {
    sender: Option<mpsc::SyncSender<SocketInceptionRequest>>,
}

#[cfg(all(windows, feature = "socket-table"))]
impl SocketInceptionCollector {
    fn new(
        mut lookup: impl FnMut(ConnectionDescriptor, u64) -> super::ConnectionInception + Send + 'static,
    ) -> Self {
        let (sender, receiver) = mpsc::sync_channel(SOCKET_INCEPTION_QUEUE_CAPACITY);
        let spawned = std::thread::Builder::new()
            .name("fragcap-socket-inception".to_string())
            .spawn(move || {
                while let Ok(request) = receiver.recv() {
                    match request {
                        SocketInceptionRequest::Lookup {
                            descriptor,
                            opened_at_ns,
                            result,
                        } => {
                            *result.lock().expect("inception result lock") =
                                lookup(descriptor, opened_at_ns);
                        }
                        SocketInceptionRequest::Flush(done) => {
                            let _ = done.send(());
                        }
                    }
                }
            });
        Self {
            sender: spawned.ok().map(|_| sender),
        }
    }

    fn request(
        &self,
        descriptor: ConnectionDescriptor,
        opened_at_ns: u64,
    ) -> Arc<Mutex<super::ConnectionInception>> {
        let result = Arc::new(Mutex::new(super::ConnectionInception {
            reason: "socket-collector-pending",
            ..Default::default()
        }));
        let Some(sender) = &self.sender else {
            result.lock().expect("inception result lock").reason = "socket-collector-unavailable";
            return result;
        };
        let request = SocketInceptionRequest::Lookup {
            descriptor,
            opened_at_ns,
            result: Arc::clone(&result),
        };
        if let Err(error) = sender.try_send(request) {
            result.lock().expect("inception result lock").reason = match error {
                mpsc::TrySendError::Full(_) => "socket-collector-saturated",
                mpsc::TrySendError::Disconnected(_) => "socket-collector-unavailable",
            };
        }
        result
    }

    fn flush(&self, budget: Duration) {
        let Some(sender) = &self.sender else { return };
        let (done_send, done_receive) = mpsc::channel();
        if sender
            .try_send(SocketInceptionRequest::Flush(done_send))
            .is_ok()
        {
            let _ = done_receive.recv_timeout(budget.min(Duration::from_millis(250)));
        }
    }
}

#[cfg(all(windows, feature = "socket-table"))]
impl Default for SocketInceptionCollector {
    fn default() -> Self {
        let mut source = IpHelperTable::default();
        Self::new(move |descriptor, opened_at_ns| match source.read() {
            Ok(table) => select_inception_socket_owner(&table, descriptor, opened_at_ns),
            Err(_) => super::ConnectionInception {
                reason: "socket-table-read-failed",
                ..Default::default()
            },
        })
    }
}

/// Session-owned bridge between packet truth and native proxy observations.
#[derive(Clone, Debug, Default)]
pub struct NativeObservationContext {
    flow_registry: Arc<FlowRegistry>,
    controlled_process_id: Arc<AtomicU32>,
    process_evidence: Arc<Mutex<Option<super::CaptureProcessEvidence>>>,
    #[cfg(all(windows, feature = "direct-route"))]
    direct_helpers: Arc<Mutex<DirectHelperRegistry>>,
    #[cfg(all(windows, feature = "direct-route"))]
    direct_proofs:
        Arc<Mutex<BTreeMap<super::direct_route::OriginalFlow, Arc<Mutex<DirectRouteProof>>>>>,
    #[cfg(all(windows, feature = "direct-route"))]
    direct_route_active: Arc<AtomicBool>,
    #[cfg(all(windows, feature = "socket-table"))]
    socket_collector: Arc<SocketInceptionCollector>,
}

#[cfg(all(windows, feature = "direct-route"))]
#[derive(Clone, Debug)]
struct DirectHelperOwner {
    flow: super::direct_route::OriginalFlow,
    opened_earliest_ns: u64,
    opened_latest_ns: u64,
}

#[cfg(all(windows, feature = "direct-route"))]
#[derive(Clone, Debug, Default)]
pub(crate) struct DirectRouteProof {
    pub proxy_connected: bool,
    pub reply_sent: bool,
    pub reply_failed: bool,
    pub client_ack: bool,
    pub first_returned_data_end: Option<u32>,
    pub returned_data_acknowledged: bool,
    pub client_to_upstream: u64,
    pub upstream_to_client: u64,
    pub client_datagrams: u64,
    pub peer_datagrams: u64,
}

#[cfg(all(windows, feature = "direct-route"))]
impl DirectRouteProof {
    fn confirms_reachability(&self, transport: Proto) -> bool {
        self.proxy_connected
            && self.reply_sent
            && !self.reply_failed
            && match transport {
                Proto::Tcp => {
                    self.client_ack
                        && self.returned_data_acknowledged
                        && self.client_to_upstream > 0
                        && self.upstream_to_client > 0
                }
                Proto::Udp => self.client_datagrams > 0 && self.peer_datagrams > 0,
            }
    }

    fn proof_kind(&self, transport: Proto) -> &'static str {
        match transport {
            Proto::Tcp if self.client_to_upstream > 0 && self.upstream_to_client > 0 => "transfer",
            Proto::Tcp => "handshake",
            Proto::Udp if self.client_to_upstream > 0 && self.upstream_to_client > 0 => "transfer",
            Proto::Udp => "datagram-response",
        }
    }
}

#[cfg(all(windows, feature = "direct-route"))]
type DirectHelperRegistry =
    BTreeMap<(&'static str, SocketAddr, SocketAddr), Vec<DirectHelperOwner>>;

#[cfg(all(windows, feature = "direct-route"))]
const DIRECT_HELPER_ACCEPT_SLACK_NS: u64 = 50_000_000;
#[cfg(all(windows, feature = "direct-route"))]
const DIRECT_PACKET_JOIN_SLACK_NS: u64 = 50_000_000;

impl NativeObservationContext {
    /// Publish only newly retained process facts during an active capture.
    /// Cloning the full event history on every ETW event would be quadratic.
    pub fn record_live_process_evidence(&self, evidence: &super::CaptureProcessEvidence) {
        let mut held = self.process_evidence.lock().expect("process evidence lock");
        let live = held.get_or_insert_with(super::CaptureProcessEvidence::default);
        live.event_limit = evidence.event_limit;
        live.launch_pid = evidence.launch_pid;
        live.launch_at = evidence.launch_at;
        live.launch_earliest_at = evidence.launch_earliest_at;
        live.launch_identity = evidence.launch_identity.clone();
        live.events_unretained = evidence.events_unretained;
        live.stage_transitions_unretained = evidence.stage_transitions_unretained;
        live.stage_path_limitations = evidence.stage_path_limitations.clone();
        live.unparseable_events = evidence.unparseable_events;
        live.rundown_ignored = evidence.rundown_ignored;
        live.watcher_ended = evidence.watcher_ended;
        live.watcher_report.clone_from(&evidence.watcher_report);
        if live.shutdown_scope.is_empty() {
            live.shutdown_scope.clone_from(&evidence.shutdown_scope);
        }
        if live.events.len() <= evidence.events.len() {
            live.events
                .extend_from_slice(&evidence.events[live.events.len()..]);
        }
        if live.stage_transitions.len() <= evidence.stage_transitions.len() {
            live.stage_transitions
                .extend_from_slice(&evidence.stage_transitions[live.stage_transitions.len()..]);
        }
        if live.managed_instances.len() <= evidence.managed_instances.len() {
            live.managed_instances
                .extend_from_slice(&evidence.managed_instances[live.managed_instances.len()..]);
        }
        live.managed_instance_limit_reached = evidence.managed_instance_limit_reached;
    }

    /// Retain already observed process events for the final connection-instance join.
    pub fn record_process_evidence(&self, evidence: &super::CaptureProcessEvidence) {
        *self.process_evidence.lock().expect("process evidence lock") = Some(evidence.clone());
    }

    /// Read the current retained generation proof for a pending direct SYN.
    /// The route samples only while a first packet is awaiting admission.
    #[cfg(all(windows, feature = "direct-route"))]
    pub(crate) fn direct_process_evidence(&self) -> Option<super::CaptureProcessEvidence> {
        self.process_evidence
            .lock()
            .expect("process evidence lock")
            .clone()
    }

    #[cfg(all(windows, feature = "direct-route"))]
    pub(crate) fn register_direct_helper(
        &self,
        transport: &'static str,
        helper_local: SocketAddr,
        proxy_local: SocketAddr,
        flow: super::direct_route::OriginalFlow,
        opened_earliest_ns: u64,
        opened_latest_ns: u64,
    ) -> Result<(), &'static str> {
        if !matches!(transport, "tcp" | "udp")
            || opened_earliest_ns > opened_latest_ns
            || helper_local == proxy_local
        {
            return Err("invalid-direct-helper-window");
        }
        let mut registry = self.direct_helpers.lock().expect("direct helper lock");
        if registry.values().map(Vec::len).sum::<usize>() >= 512 {
            return Err("direct-helper-registry-full");
        }
        let owners = registry
            .entry((transport, helper_local, proxy_local))
            .or_default();
        if owners.iter().any(|owner| {
            owner.opened_earliest_ns <= opened_latest_ns
                && opened_earliest_ns <= owner.opened_latest_ns
        }) {
            return Err("direct-helper-window-overlap");
        }
        owners.push(DirectHelperOwner {
            flow,
            opened_earliest_ns,
            opened_latest_ns,
        });
        self.direct_proof_for(&owners.last().expect("new direct helper").flow)?;
        Ok(())
    }

    #[cfg(all(windows, feature = "direct-route"))]
    pub(crate) fn direct_proof_for(
        &self,
        flow: &super::direct_route::OriginalFlow,
    ) -> Result<Arc<Mutex<DirectRouteProof>>, &'static str> {
        let mut proofs = self.direct_proofs.lock().expect("direct proof lock");
        if let Some(proof) = proofs.get(flow) {
            return Ok(Arc::clone(proof));
        }
        if proofs.len() >= 512 {
            return Err("direct-proof-registry-full");
        }
        let proof = Arc::new(Mutex::new(DirectRouteProof::default()));
        proofs.insert(flow.clone(), Arc::clone(&proof));
        Ok(proof)
    }

    #[cfg(all(windows, feature = "direct-route"))]
    pub(crate) fn set_direct_route_active(&self, active: bool) {
        if active {
            self.direct_helpers
                .lock()
                .expect("direct helper lock")
                .clear();
            self.direct_proofs
                .lock()
                .expect("direct proof lock")
                .clear();
        }
        self.direct_route_active.store(active, Ordering::Release);
    }

    #[cfg(all(test, windows, feature = "direct-route"))]
    pub(crate) fn direct_helper_flows_for_test(&self) -> Vec<super::direct_route::OriginalFlow> {
        self.direct_helpers
            .lock()
            .expect("direct helper lock")
            .values()
            .flatten()
            .map(|owner| owner.flow.clone())
            .collect()
    }

    #[cfg(all(windows, feature = "direct-route"))]
    fn direct_helper_flow(
        &self,
        descriptor: ConnectionDescriptor,
        opened_at_ns: u64,
    ) -> Result<Option<super::direct_route::OriginalFlow>, &'static str> {
        let registry = self.direct_helpers.lock().expect("direct helper lock");
        let Some(owners) = registry.get(&(
            descriptor.transport,
            descriptor.client_peer,
            descriptor.proxy_local,
        )) else {
            return Ok(None);
        };
        let mut matching = owners.iter().filter(|owner| {
            owner
                .opened_earliest_ns
                .saturating_sub(DIRECT_HELPER_ACCEPT_SLACK_NS)
                <= opened_at_ns
                && opened_at_ns
                    <= owner
                        .opened_latest_ns
                        .saturating_add(DIRECT_HELPER_ACCEPT_SLACK_NS)
        });
        let Some(owner) = matching.next() else {
            return Ok(None);
        };
        if matching.next().is_some() {
            return Err("direct-helper-window-ambiguous");
        }
        Ok(Some(owner.flow.clone()))
    }

    #[cfg(all(windows, feature = "direct-route"))]
    fn direct_helper_correlation(
        &self,
        descriptor: ConnectionDescriptor,
        opened_at_ns: u64,
        closed_at_ns: u64,
    ) -> Option<NativeCorrelation> {
        let flow = match self.direct_helper_flow(descriptor, opened_at_ns) {
            Ok(Some(flow)) => flow,
            Ok(None) => return None,
            Err(reason) => {
                return Some((
                    None,
                    None,
                    None,
                    None,
                    None,
                    0,
                    0,
                    CorrelationState::Ambiguous,
                    reason.into(),
                ));
            }
        };
        let route_proof = self
            .direct_proofs
            .lock()
            .expect("direct proof lock")
            .get(&flow)
            .map(|proof| proof.lock().expect("direct proof lock").clone());
        let route_positive = route_proof
            .as_ref()
            .is_some_and(|proof| proof.confirms_reachability(flow.tuple.transport));
        let proof_kind = route_proof
            .as_ref()
            .map(|proof| proof.proof_kind(flow.tuple.transport))
            .unwrap_or("unavailable");
        let key = FlowKey::new(flow.tuple.transport, flow.tuple.local, flow.tuple.remote);
        let Some(summary) = self.flow_registry.summary(&key) else {
            if route_positive {
                return Some((
                    None,
                    Some(flow.pid),
                    Some(flow.process_image),
                    Some("client".into()),
                    Some("direct-route-receipt".into()),
                    0,
                    0,
                    CorrelationState::Matched,
                    format!("direct-route-client-{proof_kind}-original-packet-unavailable"),
                ));
            }
            return Some((
                None,
                None,
                None,
                None,
                None,
                0,
                0,
                CorrelationState::Unavailable,
                if route_proof
                    .as_ref()
                    .is_some_and(|proof| proof.proxy_connected && proof.reply_sent)
                {
                    "direct-route-client-delivery-unproven"
                } else {
                    "direct-original-packet-not-retained"
                }
                .into(),
            ));
        };
        let window_start = flow
            .first_packet_at_ns
            .saturating_sub(DIRECT_PACKET_JOIN_SLACK_NS as i64);
        let window_end = i64::try_from(closed_at_ns).unwrap_or(i64::MAX);
        let in_window = |time: i64| time >= window_start && time <= window_end;
        let observed = summary
            .observations
            .iter()
            .filter(|observation| in_window(observation.timestamp.as_nanos()))
            .count() as u64;
        let incomplete = summary.omission_precision_lost
            || summary.unretained_observations != 0
            || summary.global_unretained_observations != 0
            || !summary.omissions.is_empty();
        let first_packet_retained = summary.observations.iter().any(|observation| {
            in_window(observation.timestamp.as_nanos())
                && observation
                    .timestamp
                    .as_nanos()
                    .abs_diff(flow.first_packet_at_ns)
                    <= DIRECT_PACKET_JOIN_SLACK_NS
                && observation.attribution.as_ref().is_some_and(|owner| {
                    owner.pid == flow.pid
                        && owner.process.as_ref() == flow.process_image
                        && owner.role.as_deref() == Some("client")
                })
        });
        let competing_owner = summary.observations.iter().any(|observation| {
            in_window(observation.timestamp.as_nanos())
                && observation.attribution.as_ref().is_some_and(|owner| {
                    owner.pid != flow.pid || owner.process.as_ref() != flow.process_image
                })
        });
        if route_positive && !competing_owner && (incomplete || !first_packet_retained) {
            return Some((
                Some(summary.id),
                Some(flow.pid),
                Some(flow.process_image),
                Some("client".into()),
                Some("direct-route-receipt".into()),
                observed,
                summary.unretained_observations,
                CorrelationState::Matched,
                if incomplete {
                    format!("direct-route-client-{proof_kind}-packet-history-incomplete")
                } else {
                    format!("direct-route-client-{proof_kind}-original-packet-unavailable")
                },
            ));
        }
        if incomplete || !first_packet_retained || competing_owner {
            return Some((
                Some(summary.id),
                None,
                None,
                None,
                None,
                observed,
                summary.unretained_observations,
                if competing_owner {
                    CorrelationState::Ambiguous
                } else {
                    CorrelationState::Unavailable
                },
                if competing_owner {
                    "direct-original-owner-conflict"
                } else {
                    "direct-original-packet-history-incomplete"
                }
                .into(),
            ));
        }
        if !route_positive {
            return Some((
                Some(summary.id),
                None,
                None,
                None,
                None,
                observed,
                summary.unretained_observations,
                CorrelationState::Unavailable,
                "direct-route-client-delivery-unproven".into(),
            ));
        }
        Some((
            Some(summary.id),
            Some(flow.pid),
            Some(flow.process_image),
            Some("client".into()),
            Some("direct-route-receipt".into()),
            observed,
            0,
            CorrelationState::Matched,
            "direct-original-flow-and-managed-process-instance".into(),
        ))
    }

    fn correlate_proxy_window(&self, window: &ApplicationConnectionWindow) -> NativeCorrelation {
        #[cfg(all(windows, feature = "direct-route"))]
        if let Some(result) = self.direct_helper_correlation(
            window.descriptor,
            window.opened_at_ns,
            window.closed_at_ns.unwrap_or(window.opened_at_ns),
        ) {
            return result;
        }
        correlate_connection_window(&self.flow_registry, window)
    }

    fn correlate_proxy_observation(
        &self,
        client_peer: SocketAddr,
        proxy_local: SocketAddr,
        opened_at_ns: u64,
        closed_at_ns: u64,
    ) -> NativeCorrelation {
        #[cfg(all(windows, feature = "direct-route"))]
        {
            let mut direct = None;
            for transport in ["tcp", "udp"] {
                if let Some(result) = self.direct_helper_correlation(
                    ConnectionDescriptor {
                        transport,
                        client_peer,
                        proxy_local,
                    },
                    opened_at_ns,
                    closed_at_ns,
                ) {
                    if direct.is_some() {
                        return (
                            None,
                            None,
                            None,
                            None,
                            None,
                            0,
                            0,
                            CorrelationState::Ambiguous,
                            "direct-helper-transport-ambiguous".into(),
                        );
                    }
                    direct = Some(result);
                }
            }
            if let Some(result) = direct {
                return result;
            }
        }
        correlate_native_observation(
            &self.flow_registry,
            client_peer,
            proxy_local,
            opened_at_ns,
            closed_at_ns,
        )
    }

    fn socket_inception(
        &self,
        descriptor: ConnectionDescriptor,
        opened_at_ns: u64,
    ) -> Arc<Mutex<super::ConnectionInception>> {
        #[cfg(all(windows, feature = "socket-table"))]
        {
            self.socket_collector.request(descriptor, opened_at_ns)
        }
        #[cfg(not(all(windows, feature = "socket-table")))]
        {
            let _ = (descriptor, opened_at_ns);
            Arc::new(Mutex::new(super::ConnectionInception {
                reason: "socket-table-unavailable",
                ..Default::default()
            }))
        }
    }

    fn finish_inception_collection(&self, budget: Duration) {
        #[cfg(all(windows, feature = "socket-table"))]
        self.socket_collector.flush(budget);
        #[cfg(not(all(windows, feature = "socket-table")))]
        let _ = budget;
    }

    fn connection_owner(
        &self,
        window: &ApplicationConnectionWindow,
    ) -> super::ConnectionOwnerEvidence {
        #[cfg(all(windows, feature = "direct-route"))]
        match self.direct_helper_flow(window.descriptor, window.opened_at_ns) {
            Ok(Some(flow)) => {
                return super::ConnectionOwnerEvidence {
                    state: "exact".into(),
                    reason: "direct-route-receipt".into(),
                    pid: Some(flow.pid),
                    process_instance_id: Some(flow.process_instance_id),
                    process_image: Some(flow.process_image),
                    role: Some("client".into()),
                    original_destination: Some(flow.tuple.remote),
                }
            }
            Err(reason) => {
                return super::ConnectionOwnerEvidence {
                    state: "ambiguous".into(),
                    reason: reason.into(),
                    ..Default::default()
                }
            }
            Ok(None) => {}
        }
        let inception = *window.inception.lock().expect("inception result lock");
        if inception.state != "exact" {
            return super::ConnectionOwnerEvidence {
                state: inception.state.into(),
                reason: inception.reason.into(),
                ..Default::default()
            };
        }
        let evidence = self.process_evidence.lock().expect("process evidence lock");
        let Some(evidence) = evidence.as_ref() else {
            return super::ConnectionOwnerEvidence {
                state: "unavailable".into(),
                reason: "process-events-unavailable".into(),
                ..Default::default()
            };
        };
        bind_inception_process(evidence, inception, window.opened_at_ns)
    }

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

#[cfg(all(test, windows, feature = "direct-route"))]
mod direct_helper_tests {
    use super::*;
    use crate::deep_capture::direct_route::{DirectTuple, OriginalFlow};
    use fragcap_proxy::InspectionDecider;

    #[test]
    fn empty_handshake_stays_unproven_and_datagram_response_keeps_packet_absence() {
        let flow = OriginalFlow {
            tuple: DirectTuple {
                transport: Proto::Tcp,
                local: "192.0.2.10:49152".parse().unwrap(),
                remote: "198.51.100.20:443".parse().unwrap(),
            },
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 2,
            endpoint_id: 7,
            pid: 42,
            process_instance_id: "process-42-100".into(),
            process_image: "client.exe".into(),
        };
        let helper: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: helper,
            proxy_local: proxy,
        };
        let context = NativeObservationContext::default();
        context
            .register_direct_helper("tcp", helper, proxy, flow.clone(), 100, 200)
            .unwrap();
        let window = ApplicationConnectionWindow {
            descriptor,
            opened_at_ns: 150,
            closed_at_ns: Some(300),
            inception: Arc::new(Mutex::new(super::super::ConnectionInception::default())),
        };
        let proof = context.direct_proof_for(&flow).unwrap();
        {
            let mut proof = proof.lock().unwrap();
            proof.proxy_connected = true;
            proof.reply_sent = true;
            proof.client_ack = true;
        }
        let handshake = context.correlate_proxy_window(&window);
        assert_eq!(handshake.0, None);
        assert_eq!(handshake.5, 0);
        assert_eq!(handshake.7, CorrelationState::Unavailable);
        assert_eq!(handshake.8, "direct-route-client-delivery-unproven");

        let packet_only = NativeObservationContext::default();
        packet_only
            .register_direct_helper("tcp", helper, proxy, flow.clone(), 100, 200)
            .unwrap();
        packet_only.flow_registry.observe_at(
            FlowKey::new(flow.tuple.transport, flow.tuple.local, flow.tuple.remote),
            Timestamp::from_nanos(101),
            Some(
                &fragcap_core::Attribution::new(42, "client.exe", Fidelity::Live)
                    .with_role("client"),
            ),
        );
        let no_delivery = packet_only.correlate_proxy_window(&window);
        assert_eq!(no_delivery.7, CorrelationState::Unavailable);
        assert_eq!(no_delivery.8, "direct-route-client-delivery-unproven");

        let udp = OriginalFlow {
            tuple: DirectTuple {
                transport: Proto::Udp,
                ..flow.tuple
            },
            ..flow
        };
        let udp_context = NativeObservationContext::default();
        udp_context
            .register_direct_helper("udp", helper, proxy, udp.clone(), 100, 200)
            .unwrap();
        let udp_proof = udp_context.direct_proof_for(&udp).unwrap();
        {
            let mut proof = udp_proof.lock().unwrap();
            proof.proxy_connected = true;
            proof.reply_sent = true;
            proof.client_datagrams = 1;
            proof.peer_datagrams = 1;
        }
        let udp_window = ApplicationConnectionWindow {
            descriptor: ConnectionDescriptor {
                transport: "udp",
                ..descriptor
            },
            ..window
        };
        let response = udp_context.correlate_proxy_window(&udp_window);
        assert_eq!(response.5, 0);
        assert_eq!(response.7, CorrelationState::Matched);
        assert_eq!(
            response.8,
            "direct-route-client-datagram-response-original-packet-unavailable"
        );
    }

    #[test]
    fn helper_receipt_restores_original_owner_only_for_exact_accept_window() {
        let context = NativeObservationContext::default();
        let helper: SocketAddr = "127.0.0.1:31001".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:31002".parse().unwrap();
        let flow = OriginalFlow {
            tuple: DirectTuple {
                transport: Proto::Tcp,
                local: "127.0.0.1:31003".parse().unwrap(),
                remote: "127.0.0.1:31004".parse().unwrap(),
            },
            first_packet_at_ns: 120,
            first_packet_clock_error_ns: 2,
            endpoint_id: 7,
            pid: 42,
            process_instance_id: "process-42-100".into(),
            process_image: "client.exe".into(),
        };
        context
            .register_direct_helper("tcp", helper, proxy, flow.clone(), 100, 200)
            .unwrap();
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: helper,
            proxy_local: proxy,
        };
        assert_eq!(
            context.direct_helper_flow(descriptor, 150).unwrap(),
            Some(flow.clone())
        );
        let window = ApplicationConnectionWindow {
            descriptor,
            opened_at_ns: 150,
            closed_at_ns: Some(300),
            inception: Arc::new(Mutex::new(super::super::ConnectionInception::default())),
        };
        let owner = context.connection_owner(&window);
        assert_eq!(owner.state, "exact");
        assert_eq!(owner.reason, "direct-route-receipt");
        assert_eq!(owner.pid, Some(42));
        assert_eq!(owner.process_image.as_deref(), Some("client.exe"));
        assert_eq!(
            owner.original_destination,
            Some("127.0.0.1:31004".parse().unwrap())
        );
        let decision = ExactGameDecider::new(context.clone(), vec!["client".into()]).decide(
            fragcap_proxy::InspectionRequest {
                connection_id: 1,
                client_peer: helper,
                proxy_local: proxy,
                opened_at_ns: 150,
                transport: "tcp",
            },
            Duration::from_millis(5),
        );
        assert_eq!(decision.action, fragcap_proxy::InspectionAction::Inspect);
        assert_eq!(
            decision.owner_proof,
            "direct-original-socket-and-managed-process-instance"
        );
        let missing_packet = context.correlate_proxy_window(&window);
        assert_eq!(missing_packet.7, CorrelationState::Unavailable);
        assert_eq!(missing_packet.8, "direct-original-packet-not-retained");
        let proof = context.direct_proof_for(&flow).unwrap();
        {
            let mut proof = proof.lock().unwrap();
            proof.proxy_connected = true;
            proof.reply_sent = true;
            proof.client_to_upstream = 5;
            proof.upstream_to_client = 8;
        }
        assert_eq!(
            context.correlate_proxy_window(&window).7,
            CorrelationState::Unavailable
        );
        proof.lock().unwrap().client_ack = true;
        assert_eq!(
            context.correlate_proxy_window(&window).7,
            CorrelationState::Unavailable
        );
        proof.lock().unwrap().returned_data_acknowledged = true;
        let without_packet = context.correlate_proxy_window(&window);
        assert_eq!(without_packet.0, None);
        assert_eq!(without_packet.5, 0);
        assert_eq!(without_packet.7, CorrelationState::Matched);
        assert_eq!(
            without_packet.8,
            "direct-route-client-transfer-original-packet-unavailable"
        );
        context.flow_registry.observe_at(
            FlowKey::new(flow.tuple.transport, flow.tuple.local, flow.tuple.remote),
            Timestamp::from_nanos(121),
            Some(
                &fragcap_core::Attribution::new(42, "client.exe", Fidelity::Live)
                    .with_role("client"),
            ),
        );
        let original = context.correlate_proxy_window(&window);
        assert_eq!(original.1, Some(42));
        assert_eq!(original.2.as_deref(), Some("client.exe"));
        assert_eq!(original.3.as_deref(), Some("client"));
        assert_eq!(original.5, 1);
        assert_eq!(original.7, CorrelationState::Matched);
        assert_eq!(
            context.correlate_proxy_observation(helper, proxy, 150, 300),
            original
        );
        context.flow_registry.observe_at(
            FlowKey::new(flow.tuple.transport, flow.tuple.local, flow.tuple.remote),
            Timestamp::from_nanos(122),
            Some(&fragcap_core::Attribution::new(
                99,
                "other.exe",
                Fidelity::Live,
            )),
        );
        let contested = context.correlate_proxy_window(&window);
        assert_eq!(contested.7, CorrelationState::Ambiguous);
        assert_eq!(contested.8, "direct-original-owner-conflict");
        assert_eq!(contested.1, None);
        assert_eq!(
            context.direct_helper_flow(descriptor, 100_000_000).unwrap(),
            None
        );
        assert_eq!(
            context
                .direct_helper_flow(
                    ConnectionDescriptor {
                        client_peer: "127.0.0.1:31005".parse().unwrap(),
                        ..descriptor
                    },
                    150
                )
                .unwrap(),
            None
        );
        let udp_flow = OriginalFlow {
            tuple: DirectTuple {
                transport: Proto::Udp,
                ..flow.tuple
            },
            ..flow.clone()
        };
        context
            .register_direct_helper("udp", helper, proxy, udp_flow.clone(), 100, 200)
            .unwrap();
        let udp_descriptor = ConnectionDescriptor {
            transport: "udp",
            ..descriptor
        };
        assert_eq!(
            context.direct_helper_flow(udp_descriptor, 150).unwrap(),
            Some(udp_flow)
        );
        assert_eq!(
            context
                .correlate_proxy_observation(helper, proxy, 150, 300)
                .7,
            CorrelationState::Ambiguous
        );
        let udp_decision = ExactGameDecider::new(context.clone(), vec!["client".into()]).decide(
            fragcap_proxy::InspectionRequest {
                connection_id: 2,
                client_peer: helper,
                proxy_local: proxy,
                opened_at_ns: 150,
                transport: "udp",
            },
            Duration::from_millis(5),
        );
        assert_eq!(
            udp_decision.action,
            fragcap_proxy::InspectionAction::Inspect
        );
        assert_eq!(
            context.register_direct_helper("tcp", helper, proxy, flow, 150, 250),
            Err("direct-helper-window-overlap")
        );
        context.set_direct_route_active(true);
        assert_eq!(context.direct_helper_flow(descriptor, 150).unwrap(), None);
        context.set_direct_route_active(false);
    }
}

/// Exact game owner resolution for an opted-in native inspection scope.
#[derive(Clone, Debug)]
pub struct ExactGameDecider {
    context: NativeObservationContext,
    game_roles: Arc<[String]>,
    requested_scope: super::RequestedInspectionScope,
}

impl ExactGameDecider {
    pub fn new(context: NativeObservationContext, game_roles: Vec<String>) -> Self {
        Self {
            context,
            game_roles: Arc::from(game_roles),
            requested_scope: super::RequestedInspectionScope::ExactGame,
        }
    }

    /// Preserve an auto request after the plan resolves it to exact game.
    pub fn with_auto_request(mut self) -> Self {
        self.requested_scope = super::RequestedInspectionScope::Auto;
        self
    }
}

impl fragcap_proxy::InspectionDecider for ExactGameDecider {
    fn requested_scope(&self) -> &'static str {
        self.requested_scope.as_str()
    }

    fn decide(
        &self,
        request: fragcap_proxy::InspectionRequest,
        budget: Duration,
    ) -> fragcap_proxy::InspectionDecision {
        use fragcap_proxy::{InspectionDecision, InspectionReason};

        let started = Instant::now();
        let descriptor = ConnectionDescriptor {
            transport: request.transport,
            client_peer: request.client_peer,
            proxy_local: request.proxy_local,
        };
        #[cfg(all(windows, feature = "direct-route"))]
        loop {
            match self
                .context
                .direct_helper_flow(descriptor, request.opened_at_ns)
            {
                Ok(Some(_)) => {
                    let decision = if self.game_roles.iter().any(|role| role == "client") {
                        InspectionDecision::verified_direct_route(elapsed_millis(started))
                    } else {
                        InspectionDecision::uninspected(
                            InspectionReason::UnprovenRole,
                            elapsed_millis(started),
                        )
                    };
                    return decision.with_requested_scope(self.requested_scope.as_str());
                }
                Err(_) => {
                    return InspectionDecision::uninspected(
                        InspectionReason::AmbiguousOwner,
                        elapsed_millis(started),
                    )
                    .with_requested_scope(self.requested_scope.as_str())
                }
                Ok(None) => {
                    if !self.context.direct_route_active.load(Ordering::Acquire)
                        || started.elapsed() >= budget.min(Duration::from_millis(10))
                    {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        let result = self
            .context
            .socket_inception(descriptor, request.opened_at_ns);
        let inception = loop {
            let current = *result.lock().expect("inception result lock");
            if current.reason != "socket-collector-pending" {
                break current;
            }
            if started.elapsed() >= budget {
                return InspectionDecision::uninspected(
                    InspectionReason::PendingTimeout,
                    elapsed_millis(started),
                )
                .with_requested_scope(self.requested_scope.as_str());
            }
            std::thread::sleep(
                Duration::from_millis(1).min(budget.saturating_sub(started.elapsed())),
            );
        };
        let decision = if inception.state != "exact" {
            match inception.reason {
                "socket-collector-saturated" => InspectionReason::QueueSaturated,
                "multiple-client-socket-owners" => InspectionReason::AmbiguousOwner,
                "client-socket-not-in-table" | "socket-created-after-accept" => {
                    InspectionReason::StaleSocket
                }
                _ => InspectionReason::TelemetryUnavailable,
            }
        } else {
            let held = self
                .context
                .process_evidence
                .lock()
                .expect("process evidence lock");
            let Some(evidence) = held.as_ref() else {
                return InspectionDecision::uninspected(
                    InspectionReason::TelemetryUnavailable,
                    elapsed_millis(started),
                )
                .with_requested_scope(self.requested_scope.as_str());
            };
            exact_game_owner_reason(evidence, inception, request.opened_at_ns, &self.game_roles)
        };
        let result = if decision == InspectionReason::VerifiedGame {
            InspectionDecision::verified_game(elapsed_millis(started))
        } else {
            InspectionDecision::uninspected(decision, elapsed_millis(started))
        };
        result.with_requested_scope(self.requested_scope.as_str())
    }
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

fn exact_game_owner_reason(
    evidence: &super::CaptureProcessEvidence,
    inception: super::ConnectionInception,
    opened_at_ns: u64,
    game_roles: &[String],
) -> fragcap_proxy::InspectionReason {
    use fragcap_proxy::InspectionReason;

    let owner = bind_inception_process(evidence, inception, opened_at_ns);
    if owner.state != "exact" {
        return match owner.reason.as_str() {
            "socket-owner-generation-changed-before-accept" => InspectionReason::ChangedGeneration,
            "socket-created-after-accept" => InspectionReason::StaleSocket,
            _ => InspectionReason::TelemetryUnavailable,
        };
    }
    let Some(pid) = owner.pid else {
        return InspectionReason::TelemetryUnavailable;
    };
    let Some(root) = evidence.launch_identity.as_ref() else {
        return InspectionReason::TelemetryUnavailable;
    };
    let Some(instance) = evidence.managed_instances.iter().find(|instance| {
        instance.identity.pid == pid
            && owner
                .process_instance_id
                .as_deref()
                .is_some_and(|id| id == format!("process-{pid}-{}", instance.event_at.as_nanos()))
            && (instance.identity.created_unix_nanos() - i128::from(instance.event_at.as_nanos()))
                .abs()
                <= 1_000_000_000
            && instance.identity.same_user_session(root)
            && evidence.shutdown_scope.iter().any(|path| {
                path.to_string_lossy()
                    .eq_ignore_ascii_case(&instance.identity.path.to_string_lossy())
            })
    }) else {
        return InspectionReason::TelemetryUnavailable;
    };
    let _ = instance;
    match owner.role.as_deref() {
        Some(role) if game_roles.iter().any(|allowed| allowed == role) => {
            InspectionReason::VerifiedGame
        }
        Some(_) => InspectionReason::KnownMiddleware,
        None => InspectionReason::UnprovenRole,
    }
}

impl NativeProxyAdapter {
    pub fn generate_installation_authority(
        generation: u64,
        now: SystemTime,
        lifetime: Duration,
    ) -> Result<PreparedNativeAuthority, super::PreflightRefusal> {
        let authority =
            fragcap_proxy::SessionCertificateAuthority::generate(generation, now, lifetime)
                .map_err(|error| super::PreflightRefusal::new(error.code, error.detail))?;
        Ok(PreparedNativeAuthority {
            authority: Arc::new(authority),
            lifetime,
        })
    }

    #[cfg(windows)]
    pub fn prepare_installation_directories(
        root: &std::path::Path,
        pki: &std::path::Path,
    ) -> Result<(), super::PreflightRefusal> {
        fragcap_proxy::SessionCertificateAuthority::prepare_installation_directories(root, pki)
            .map_err(|error| super::PreflightRefusal::new(error.code, error.detail))
    }

    #[cfg(windows)]
    pub fn load_installation_authority(
        generation: u64,
        certificate_der: &[u8],
        key_path: &std::path::Path,
    ) -> Result<PreparedNativeAuthority, super::PreflightRefusal> {
        let authority = fragcap_proxy::SessionCertificateAuthority::load_protected_installation(
            generation,
            certificate_der,
            key_path,
        )
        .map_err(|error| super::PreflightRefusal::new(error.code, error.detail))?;
        let lifetime = authority
            .not_after()
            .duration_since(SystemTime::now())
            .unwrap_or_default()
            .min(Duration::from_secs(24 * 60 * 60));
        Ok(PreparedNativeAuthority {
            authority: Arc::new(authority),
            lifetime,
        })
    }

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
            prepared_authority: None,
            inspection_decider: None,
        }
    }

    /// Generate one session CA in memory before exact plan authorization.
    pub fn prepare_authority(
        now: SystemTime,
        lifetime: Duration,
    ) -> Result<PreparedNativeAuthority, super::PreflightRefusal> {
        let generation = NEXT_PREPARED_AUTHORITY_GENERATION.fetch_add(1, Ordering::Relaxed);
        let authority =
            fragcap_proxy::SessionCertificateAuthority::generate(generation, now, lifetime)
                .map_err(|error| super::PreflightRefusal::new(error.code, error.detail))?;
        Ok(PreparedNativeAuthority {
            authority: Arc::new(authority),
            lifetime,
        })
    }

    pub fn limits(&self) -> NativeProxyLimits {
        self.limits
    }

    /// Use packet and process truth shared with this session's capture runner.
    pub fn with_observation_context(mut self, context: NativeObservationContext) -> Self {
        self.observation_context = context;
        self
    }

    pub fn with_inspection_decider(
        mut self,
        decider: Arc<dyn fragcap_proxy::InspectionDecider>,
    ) -> Self {
        self.inspection_decider = Some(decider);
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

    /// Consume the exact authority whose public identity was authorized.
    pub fn with_prepared_authority(mut self, authority: PreparedNativeAuthority) -> Self {
        self.prepared_authority = Some(authority);
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
        let inception_context = self.observation_context.clone();
        let controlled = plan.controlled;
        let target_id = plan.target.id;
        let mut application_artifact = self
            .application_artifact
            .as_ref()
            .map(|path| {
                super::ApplicationArtifactLease::open_correlated_with_inception(
                    path,
                    &plan.session_id,
                    super::DEFAULT_APPLICATION_EVENT_QUEUE_CAPACITY,
                    Arc::new(move |descriptor, at| {
                        inception_context.socket_inception(descriptor, at)
                    }),
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
                            correlation_context.correlate_proxy_window(descriptor)
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
                            connection_owner: Some(
                                correlation_context.connection_owner(descriptor),
                            ),
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
        if let Some(decider) = &self.inspection_decider {
            backend = backend.with_inspection_decider(Arc::clone(decider));
        }
        if let Some(authority) = self.prepared_authority.take() {
            backend = backend.with_prepared_authority(authority.into_inner());
        }
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
            observations_lost: 0,
            classification_loss_accounted: 0,
            diagnostics: None,
            diagnostics_appended: false,
            observation_ended_at_ns: None,
            application_classification_summary: None,
            stop_authority: None,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeStopAuthority {
    clean: bool,
    listener_released: bool,
    incomplete_tasks: u64,
    residue: bool,
}

impl NativeStopAuthority {
    fn from_report(report: &ShutdownReport) -> Self {
        Self {
            clean: report.is_clean(),
            listener_released: report.listener_released,
            incomplete_tasks: report.incomplete_tasks,
            residue: report.residue,
        }
    }
}

fn observation_drain_is_complete(
    stop: Option<NativeStopAuthority>,
    state: fragcap_proxy::LifecycleState,
    live_connections: usize,
    connection_tasks_current: u64,
) -> bool {
    stop.is_some_and(|status| status.clean)
        && state == fragcap_proxy::LifecycleState::Stopped
        && live_connections == 0
        && connection_tasks_current == 0
}

struct NativeProxyLease {
    lease: fragcap_proxy::NativeProxyLease,
    controlled: bool,
    controlled_lab: Option<ControlledLab>,
    observation_context: NativeObservationContext,
    application_artifact: Option<super::ApplicationArtifactLease>,
    proxy_lifecycle: Option<super::ProxyLifecycleLease>,
    key_log: Option<Arc<fragcap_proxy::SessionKeyLog>>,
    observations_lost: u64,
    classification_loss_accounted: u64,
    application_classification_summary: Option<ClassificationSummary>,
    stop_authority: Option<NativeStopAuthority>,
    diagnostics: Option<super::ProxyDiagnostics>,
    diagnostics_appended: bool,
    observation_ended_at_ns: Option<u64>,
}

impl NativeProxyLease {
    fn append_connection_diagnostics(&mut self) -> std::io::Result<()> {
        if self.diagnostics_appended {
            return Ok(());
        }
        let (Some(lifecycle), Some(diagnostics)) = (&self.proxy_lifecycle, &self.diagnostics)
        else {
            return Ok(());
        };
        self.diagnostics_appended = true;
        for record in &diagnostics.connections {
            let mut value = serde_json::json!({
                "schema_version": record.schema_version,
                "connection_id": record.connection_id,
                "terminal": record.terminal,
                "category": record.cause.map(|cause| cause.as_str()),
                "code": record.code,
                "stages": record.stages.iter().map(|stage| serde_json::json!({
                    "stage": stage.stage,
                    "reached_ms": stage.reached_ms,
                    "completed_ms": stage.completed_ms,
                })).collect::<Vec<_>>(),
                "last_reached": record.last_reached,
                "last_completed": record.last_completed,
                "boundary": record.boundary,
                "tls_alert": record.tls_alert,
                "transport_category": record.transport_category,
                "os_error_code": record.os_error_code,
                "header_timeout_phase": record.header_timeout_phase,
                "completed_exchanges": record.completed_exchanges,
                "inspection_scope": record.inspection_scope,
                "inspection_action": record.inspection_action,
                "inspection_reason": record.inspection_reason,
                "inspection_decided_after_ms": record.inspection_decided_after_ms,
                "association_inspection_scope": record.association_inspection_scope,
                "association_inspection_action": record.association_inspection_action,
                "association_inspection_reason": record.association_inspection_reason,
                "association_inspection_decided_after_ms": record.association_inspection_decided_after_ms,
                "relay_client_bytes": record.relay_client_bytes,
                "relay_upstream_bytes": record.relay_upstream_bytes,
                "relay_client_datagrams": record.relay_client_datagrams,
                "relay_upstream_datagrams": record.relay_upstream_datagrams,
            });
            value["inspection_requested_scope"] =
                serde_json::json!(record.inspection_requested_scope);
            value["inspection_policy_id"] = serde_json::json!(record.inspection_policy_id);
            value["inspection_owner_proof"] = serde_json::json!(record.inspection_owner_proof);
            value["association_inspection_requested_scope"] =
                serde_json::json!(record.association_inspection_requested_scope);
            value["association_inspection_policy_id"] =
                serde_json::json!(record.association_inspection_policy_id);
            value["association_inspection_owner_proof"] =
                serde_json::json!(record.association_inspection_owner_proof);
            lifecycle.connection_diagnostic(value)?;
        }
        lifecycle.diagnostic_accounting(serde_json::json!({
            "schema_version": 1,
            "retained": diagnostics.connections.len(),
            "lost": diagnostics.connection_details_lost,
            "relay_client_bytes": diagnostics.relay_client_bytes,
            "relay_upstream_bytes": diagnostics.relay_upstream_bytes,
            "relay_client_datagrams": diagnostics.relay_client_datagrams,
            "relay_upstream_datagrams": diagnostics.relay_upstream_datagrams,
            "unavailable": diagnostics.connection_details_unavailable,
        }))
    }

    fn account_final_classification_loss(&mut self) {
        if let Some(artifact) = self.application_artifact.as_ref() {
            let newly_lost = self
                .observations_lost
                .saturating_sub(self.classification_loss_accounted);
            artifact.add_classification_loss(newly_lost);
            self.classification_loss_accounted = self
                .classification_loss_accounted
                .saturating_add(newly_lost);
        }
    }

    fn collect_observation_drain(
        &mut self,
        budget: Budget,
    ) -> Result<super::PhaseObservationDrain, StageFailure> {
        let observation = self
            .lease
            .observation(budget.remaining())
            .map_err(|error| StageFailure::new(Stage::Observe, error.code, error.detail))?;
        let stop = self.stop_authority;
        let complete = observation_drain_is_complete(
            stop,
            observation.state,
            observation.live_connections,
            observation.resources.connection_tasks_current,
        );
        let incomplete_detail = (!complete).then(|| {
            let (stop_observed, stop_clean, listener_released, incomplete_tasks, residue) = stop
                .map(|status| {
                    (
                        true,
                        status.clean,
                        status.listener_released,
                        status.incomplete_tasks,
                        status.residue,
                    )
                })
                .unwrap_or((false, false, false, 0, false));
            format!(
                "native proxy observation drain incomplete: stop_observed={stop_observed}, stop_clean={stop_clean}, listener_released={listener_released}, incomplete_tasks={incomplete_tasks}, residue={residue}, state={:?}, live_connections={}, connection_tasks_current={}",
                observation.state,
                observation.live_connections,
                observation.resources.connection_tasks_current,
            )
        });
        self.observations_lost = observation.protocol.observations_dropped_oldest;
        self.diagnostics = complete.then(|| super::ProxyDiagnostics::from_runtime(&observation));
        let observations = self.map_observations(observation.application);
        if complete {
            Ok(super::PhaseObservationDrain::complete(observations))
        } else {
            Ok(super::PhaseObservationDrain::incomplete(
                observations,
                "observation-drain-incomplete",
                incomplete_detail.expect("incomplete detail exists for an incomplete drain"),
            ))
        }
    }

    fn map_observations(
        &self,
        observations: Vec<fragcap_proxy::ProxyObservation>,
    ) -> Vec<super::PhaseQualifiedObservation> {
        let managed_windows = self
            .observation_context
            .process_evidence
            .lock()
            .expect("process evidence lock")
            .as_ref()
            .and_then(ManagedClientWindows::from_evidence);
        observations
            .into_iter()
            .map(|value| {
                let coarse_window = native_evidence_window(
                    value.evidence_observed_at_ns,
                    self.observation_ended_at_ns,
                );
                let evidence_window = if coarse_window == super::EvidenceWindow::Observation {
                    managed_windows.map_or(coarse_window, |windows| {
                        let opened = windows.classify(Some(value.connection_opened_at_ns));
                        if opened == super::EvidenceWindow::Observation {
                            windows.classify(value.evidence_observed_at_ns)
                        } else {
                            opened
                        }
                    })
                } else {
                    coarse_window
                };
                // Correlation authority for an observation-phase result cannot
                // be supplied solely by packets captured during owner release.
                let correlation_closed_at_ns =
                    if evidence_window == super::EvidenceWindow::Observation {
                        self.observation_ended_at_ns
                            .map_or(value.connection_closed_at_ns, |cutoff| {
                                value.connection_closed_at_ns.min(cutoff)
                            })
                    } else {
                        value.connection_closed_at_ns
                    };
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
                    self.observation_context.correlate_proxy_observation(
                        value.client_peer,
                        value.proxy_local,
                        value.connection_opened_at_ns,
                        correlation_closed_at_ns,
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
                super::PhaseQualifiedObservation {
                    evidence_window,
                    observation: CompatibilityObservation {
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
                    },
                }
            })
            .collect()
    }
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
        self.collect_observation_drain(budget)
            .map(|drain| drain.into_raw().into_parts().0)
    }

    fn drain_observations(&mut self, budget: Budget) -> Result<ObservationDrain, StageFailure> {
        self.collect_observation_drain(budget)
            .map(super::PhaseObservationDrain::into_raw)
    }

    fn drain_phase_observations(
        &mut self,
        budget: Budget,
    ) -> Option<Result<super::PhaseObservationDrain, StageFailure>> {
        Some(self.collect_observation_drain(budget))
    }

    fn observations_lost(&self) -> u64 {
        self.observations_lost
    }

    fn application_classification_summary(&self) -> Option<ClassificationSummary> {
        self.application_classification_summary.clone()
    }

    fn end_observation_window(&mut self, ended_at: SystemTime) {
        self.observation_ended_at_ns = ended_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .and_then(|elapsed| elapsed.as_nanos().try_into().ok());
    }

    fn terminal_diagnostics(&self) -> Option<super::ProxyDiagnostics> {
        self.diagnostics.clone()
    }

    fn stop(&mut self, budget: Budget) -> CleanupResult {
        let report = self.lease.stop(budget.remaining());
        self.diagnostics = report
            .is_clean()
            .then(|| super::ProxyDiagnostics::from_runtime(&report.observation));
        self.stop_authority = Some(NativeStopAuthority::from_report(&report));
        self.observations_lost = report.observation.protocol.observations_dropped_oldest;
        let mut result = cleanup_result("native-proxy-listener", &report);
        self.account_final_classification_loss();
        if report.is_clean() {
            self.observation_context
                .finish_inception_collection(budget.remaining());
            if let Some(artifact) = self.application_artifact.as_mut() {
                if let Err(error) = artifact.finish() {
                    result.status = CleanupStatus::Failed;
                    result.reason = format!("application writer failed: {error}");
                } else {
                    self.application_classification_summary = artifact.classification_summary();
                }
            }
            let diagnostic_result = self.append_connection_diagnostics();
            if let Err(error) = diagnostic_result {
                result.status = CleanupStatus::Failed;
                result.reason = format!("proxy diagnostic write failed: {error}");
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
        self.diagnostics = report
            .is_clean()
            .then(|| super::ProxyDiagnostics::from_runtime(&report.observation));
        self.observations_lost = report.observation.protocol.observations_dropped_oldest;
        let mut results = vec![cleanup_result("native-proxy-runtime", &report)];
        self.account_final_classification_loss();
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
            self.observation_context
                .finish_inception_collection(budget.remaining());
            let status = artifact.finish();
            if status.is_ok() {
                self.application_classification_summary = artifact.classification_summary();
            }
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
        let diagnostic_result = self.append_connection_diagnostics();
        if let Some(mut lifecycle) = self.proxy_lifecycle.take() {
            let result = lifecycle.finish();
            results.push(CleanupResult {
                resource: "proxy-lifecycle-writer".to_string(),
                status: if result.is_ok() && diagnostic_result.is_ok() {
                    CleanupStatus::Released
                } else {
                    CleanupStatus::Failed
                },
                reason: diagnostic_result
                    .err()
                    .or_else(|| result.err())
                    .map_or_else(
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

#[derive(Clone, Copy)]
struct ManagedClientWindows {
    first_match: Option<u64>,
    eligible_match: Option<u64>,
    eligible_exit: Option<u64>,
}

impl ManagedClientWindows {
    fn from_evidence(evidence: &super::CaptureProcessEvidence) -> Option<Self> {
        if evidence.shutdown_scope.is_empty() {
            return None;
        }
        let mut matches: Vec<(u32, u64)> = evidence
            .stage_transitions
            .iter()
            .filter(|transition| {
                transition.role == "client"
                    && transition.kind == super::StageTransitionKind::Matched
            })
            .filter_map(|transition| {
                u64::try_from(transition.at.as_nanos())
                    .ok()
                    .map(|at| (transition.pid, at))
            })
            .collect();
        matches.sort_by_key(|(_, at)| *at);
        let first_match = matches.first().map(|(_, at)| *at);
        let eligible = if evidence.stop_reason.as_deref() == Some("handoffincomplete")
            || evidence.stage_transitions_unretained > 0
            || evidence.watcher_ended
        {
            None
        } else {
            matches.last().copied()
        };
        let eligible_exit = eligible.and_then(|(pid, started)| {
            evidence
                .stage_transitions
                .iter()
                .filter(|transition| {
                    transition.role == "client"
                        && transition.kind == super::StageTransitionKind::Exited
                        && transition.pid == pid
                })
                .filter_map(|transition| u64::try_from(transition.at.as_nanos()).ok())
                .filter(|at| *at >= started)
                .min()
        });
        Some(Self {
            first_match,
            eligible_match: eligible.map(|(_, at)| at),
            eligible_exit,
        })
    }

    fn classify(self, timestamp_ns: Option<u64>) -> super::EvidenceWindow {
        let Some(at) = timestamp_ns else {
            return super::EvidenceWindow::Unavailable;
        };
        if self.first_match.is_none_or(|first| at < first) {
            return super::EvidenceWindow::PreClient;
        }
        if self.eligible_match.is_none_or(|eligible| at < eligible) {
            return super::EvidenceWindow::ProvisionalHandoff;
        }
        if self.eligible_exit.is_some_and(|exit| at >= exit) {
            return super::EvidenceWindow::OwnerRelease;
        }
        super::EvidenceWindow::Observation
    }
}

fn native_evidence_window(
    timestamp_ns: Option<u64>,
    observation_ended_at_ns: Option<u64>,
) -> super::EvidenceWindow {
    match (timestamp_ns, observation_ended_at_ns) {
        (Some(at), Some(cutoff)) if at <= cutoff => super::EvidenceWindow::Observation,
        (Some(_), Some(_)) => super::EvidenceWindow::OwnerRelease,
        _ => super::EvidenceWindow::Unavailable,
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

#[cfg(any(test, all(windows, feature = "socket-table")))]
fn select_inception_socket_owner(
    table: &SocketTable,
    descriptor: ConnectionDescriptor,
    opened_at_ns: u64,
) -> super::ConnectionInception {
    let Ok(opened) = i64::try_from(opened_at_ns) else {
        return super::ConnectionInception {
            reason: "connection-time-unrepresentable",
            ..Default::default()
        };
    };
    let proto = match descriptor.transport {
        "tcp" => Proto::Tcp,
        "udp" => Proto::Udp,
        _ => {
            return super::ConnectionInception {
                reason: "client-socket-transport-unsupported",
                ..Default::default()
            }
        }
    };
    let key = FlowKey::new(proto, descriptor.client_peer, descriptor.proxy_local);
    let mut best: Option<(&fragcap_attr::SocketTableEntry, fragcap_attr::MatchRank)> = None;
    let mut ambiguous = false;
    for entry in table.entries() {
        if entry.created.is_none_or(|at| at.as_nanos() > opened) {
            continue;
        }
        let Some(rank) = socket_match_rank(entry, &key) else {
            continue;
        };
        match best {
            None => {
                best = Some((entry, rank));
                ambiguous = false;
            }
            Some((_, previous)) if rank < previous => {
                best = Some((entry, rank));
                ambiguous = false;
            }
            Some((_, previous)) if rank == previous => ambiguous = true,
            _ => {}
        }
    }
    let Some((owner, _)) = best else {
        return super::ConnectionInception {
            reason: "client-socket-not-in-table",
            ..Default::default()
        };
    };
    if ambiguous {
        return super::ConnectionInception {
            state: "ambiguous",
            reason: "multiple-client-socket-owners",
            pid: None,
            socket_created_at_ns: None,
        };
    }
    super::ConnectionInception {
        state: "exact",
        reason: "ip-helper-client-socket",
        pid: Some(owner.pid),
        socket_created_at_ns: owner.created.map(Timestamp::as_nanos),
    }
}

fn active_process_start(
    events: &[ProcessEvent],
    pid: u32,
    at: Timestamp,
) -> Option<(u32, String, Timestamp)> {
    let mut latest: Option<(u32, &str, Timestamp)> = None;
    let mut tied = false;
    for event in events {
        let ProcessEvent::Started {
            pid: found,
            parent,
            image,
            at: created,
            ..
        } = event
        else {
            continue;
        };
        if *found != pid || *created > at {
            continue;
        }
        match latest {
            None => {
                latest = Some((*parent, image, *created));
                tied = false;
            }
            Some((_, _, previous)) if *created > previous => {
                latest = Some((*parent, image, *created));
                tied = false;
            }
            Some((_, _, previous)) if *created == previous => tied = true,
            _ => {}
        }
    }
    let latest = latest?;
    if tied
        || events.iter().any(|event| {
            matches!(event, ProcessEvent::Exited { pid: exited, at: ended }
            if *exited == pid && *ended >= latest.2 && *ended <= at)
        })
    {
        return None;
    }
    Some((latest.0, latest.1.to_string(), latest.2))
}

pub(super) fn bind_inception_process(
    evidence: &super::CaptureProcessEvidence,
    inception: super::ConnectionInception,
    opened_at_ns: u64,
) -> super::ConnectionOwnerEvidence {
    let unavailable = |reason: &str| super::ConnectionOwnerEvidence {
        state: "unavailable".into(),
        reason: reason.into(),
        ..Default::default()
    };
    if evidence.events_unretained > 0
        || evidence.unparseable_events > 0
        || evidence.watcher_ended
        || evidence
            .watcher_report
            .as_ref()
            .is_none_or(|report| report.lost_anything())
    {
        return unavailable("process-event-authority-incomplete");
    }
    let Some(((pid, socket_created_ns), (root_pid, launch_at))) = inception
        .pid
        .zip(inception.socket_created_at_ns)
        .zip(evidence.launch_pid.zip(evidence.launch_at))
    else {
        return unavailable("socket-or-launch-instance-unavailable");
    };
    let Ok(opened_ns) = i64::try_from(opened_at_ns) else {
        return unavailable("connection-time-invalid");
    };
    if socket_created_ns > opened_ns {
        return unavailable("socket-created-after-accept");
    }
    let opened = Timestamp::from_nanos(opened_ns);
    let Some((_, image, started)) = active_process_start(
        &evidence.events,
        pid,
        Timestamp::from_nanos(socket_created_ns),
    ) else {
        return unavailable("socket-owner-process-instance-unavailable");
    };
    if active_process_start(&evidence.events, pid, opened)
        .is_none_or(|(_, _, current)| current != started)
    {
        return unavailable("socket-owner-generation-changed-before-accept");
    }
    let mut current_pid = pid;
    let mut current_at = started;
    let mut proven = false;
    for _ in 0..=64 {
        let Some((parent, _, generation)) =
            active_process_start(&evidence.events, current_pid, current_at)
        else {
            break;
        };
        if current_pid == root_pid {
            proven = generation.nanos_since(launch_at).abs() <= 1_000_000_000;
            break;
        }
        if parent == current_pid || parent == 0 {
            break;
        }
        current_pid = parent;
        current_at = generation;
    }
    if !proven {
        return unavailable("socket-owner-managed-ancestry-unproven");
    }
    let role = evidence
        .stage_transitions
        .iter()
        .find(|transition| {
            transition.pid == pid
                && transition.at >= started
                && transition.at <= opened
                && transition.kind == super::StageTransitionKind::Matched
        })
        .map(|transition| transition.role.clone());
    super::ConnectionOwnerEvidence {
        state: "exact".into(),
        reason: "socket-and-process-generation-proved".into(),
        pid: Some(pid),
        process_instance_id: Some(format!("process-{pid}-{}", started.as_nanos())),
        process_image: Some(image),
        role,
        original_destination: None,
    }
}

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
    if summary.omission_precision_lost {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            summary.observations.len() as u64,
            summary.unretained_observations,
            CorrelationState::Unavailable,
            "packet-omission-history-bound-exceeded".to_string(),
        );
    }
    if let Some(omission) = summary.omissions.iter().find(|omission| {
        omission.cause == fragcap_core::FlowOmissionCause::Unknown
            || i128::from(omission.first.as_nanos()) <= i128::from(closed_at_ns)
                && i128::from(omission.last.as_nanos()) >= i128::from(opened_at_ns)
    }) {
        let reason = match omission.cause {
            fragcap_core::FlowOmissionCause::ScopeWithheld => "packet-scope-withheld",
            fragcap_core::FlowOmissionCause::Waiting => "packet-waiting-withheld",
            fragcap_core::FlowOmissionCause::OutOfWindow => "packet-out-of-window",
            fragcap_core::FlowOmissionCause::ScopeUnresolved => "packet-scope-unresolved",
            fragcap_core::FlowOmissionCause::DirectionWithheld => "packet-direction-withheld",
            fragcap_core::FlowOmissionCause::DirectionUnknown => "packet-direction-unresolved",
            fragcap_core::FlowOmissionCause::VolumeReached => "packet-volume-reached",
            fragcap_core::FlowOmissionCause::HistoryCapacity => "packet-history-bound-exceeded",
            fragcap_core::FlowOmissionCause::Unknown => "packet-omission-unlocalized",
        };
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            summary.observations.len() as u64,
            summary.unretained_observations,
            CorrelationState::Unavailable,
            reason.to_string(),
        );
    }
    if summary.unretained_observations > 0 && summary.omissions.is_empty() {
        return (
            Some(summary.id),
            None,
            None,
            None,
            None,
            summary.observations.len() as u64,
            summary.unretained_observations,
            CorrelationState::Unavailable,
            "packet-omission-unlocalized".to_string(),
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
        summary.unretained_observations,
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

    #[cfg(windows)]
    #[test]
    fn exact_game_owner_requires_instance_path_account_role_and_generation() {
        use fragcap_proxy::InspectionReason;

        let identity = crate::managed_shutdown::ProcessIdentity::current().unwrap();
        let started = i64::try_from(identity.created_unix_nanos()).unwrap();
        let pid = identity.pid;
        let mut evidence = super::super::CaptureProcessEvidence {
            launch_pid: Some(pid),
            launch_at: Some(Timestamp::from_nanos(started)),
            launch_identity: Some(identity.clone()),
            shutdown_scope: vec![identity.path.clone()],
            watcher_report: Some(fragcap_core::WatcherReport::default()),
            managed_instances: vec![super::super::ManagedProcessInstance {
                event_at: Timestamp::from_nanos(started),
                identity: identity.clone(),
            }],
            ..Default::default()
        };
        evidence.events.push(ProcessEvent::started(
            pid,
            0,
            identity.path.to_string_lossy(),
            "controlled",
            Timestamp::from_nanos(started),
        ));
        evidence
            .stage_transitions
            .push(super::super::StageTransition {
                kind: super::super::StageTransitionKind::Matched,
                pid,
                role: "client".to_string(),
                stage: Some("client".to_string()),
                at: Timestamp::from_nanos(started),
            });
        let inception = super::super::ConnectionInception {
            state: "exact",
            reason: "controlled-client-socket",
            pid: Some(pid),
            socket_created_at_ns: Some(started + 1),
        };
        let opened = u64::try_from(started + 2).unwrap();
        let game_roles = ["client".to_string()];
        assert_eq!(
            exact_game_owner_reason(&evidence, inception, opened, &game_roles),
            InspectionReason::VerifiedGame
        );
        evidence.stage_transitions[0].role = "launcher".to_string();
        assert_eq!(
            exact_game_owner_reason(&evidence, inception, opened, &game_roles),
            InspectionReason::KnownMiddleware
        );
        evidence.stage_transitions[0].role = "client".to_string();
        evidence.managed_instances[0].identity.path = PathBuf::from("C:\\Other\\same.exe");
        assert_eq!(
            exact_game_owner_reason(&evidence, inception, opened, &game_roles),
            InspectionReason::TelemetryUnavailable
        );
        evidence.managed_instances[0].identity.path = identity.path.clone();
        evidence.managed_instances[0].identity.user_sid = "unrelated-user".to_string();
        assert_eq!(
            exact_game_owner_reason(&evidence, inception, opened, &game_roles),
            InspectionReason::TelemetryUnavailable
        );
        evidence.managed_instances[0].identity.user_sid = identity.user_sid.clone();
        evidence.events.push(ProcessEvent::Exited {
            pid,
            at: Timestamp::from_nanos(started + 2),
        });
        assert_eq!(
            exact_game_owner_reason(&evidence, inception, opened + 1, &game_roles),
            InspectionReason::ChangedGeneration
        );
    }

    #[cfg(all(windows, feature = "socket-table"))]
    #[test]
    fn inception_collector_never_waits_for_socket_enumeration_on_accept() {
        let (entered_send, entered_receive) = mpsc::channel();
        let (release_send, release_receive) = mpsc::channel();
        let collector = SocketInceptionCollector::new(move |_, _| {
            let _ = entered_send.send(());
            let _ = release_receive.recv();
            super::super::ConnectionInception {
                state: "exact",
                reason: "test-socket-owner",
                pid: Some(71),
                socket_created_at_ns: Some(12),
            }
        });
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: "127.0.0.1:41000".parse().unwrap(),
            proxy_local: "127.0.0.1:42000".parse().unwrap(),
        };
        let first = collector.request(descriptor, 15);
        entered_receive
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        for _ in 0..SOCKET_INCEPTION_QUEUE_CAPACITY {
            let pending = collector.request(descriptor, 15);
            assert_eq!(pending.lock().unwrap().reason, "socket-collector-pending");
        }
        let saturated = collector.request(descriptor, 15);
        assert_eq!(
            saturated.lock().unwrap().reason,
            "socket-collector-saturated"
        );
        release_send.send(()).unwrap();
        entered_receive
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(first.lock().unwrap().pid, Some(71));
    }

    #[test]
    fn connection_inception_selects_only_one_exact_client_socket() {
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: client,
            proxy_local: proxy,
        };
        let entry = fragcap_attr::SocketTableEntry::tcp(client, proxy, 71)
            .created_at(Timestamp::from_nanos(12));
        let table = SocketTable::new(
            Timestamp::from_nanos(15),
            vec![
                fragcap_attr::SocketTableEntry::tcp(proxy, client, 9)
                    .created_at(Timestamp::from_nanos(10)),
                entry,
            ],
        );
        let exact = select_inception_socket_owner(&table, descriptor, 15);
        assert_eq!(
            (exact.state, exact.pid, exact.socket_created_at_ns),
            ("exact", Some(71), Some(12))
        );

        let too_early = select_inception_socket_owner(&table, descriptor, 11);
        assert_eq!(
            (too_early.state, too_early.reason),
            ("unavailable", "client-socket-not-in-table")
        );
        let ambiguous = SocketTable::new(
            Timestamp::from_nanos(15),
            vec![
                entry,
                fragcap_attr::SocketTableEntry::tcp(client, proxy, 72)
                    .created_at(Timestamp::from_nanos(13)),
            ],
        );
        assert_eq!(
            select_inception_socket_owner(&ambiguous, descriptor, 15).state,
            "ambiguous"
        );

        let association = ConnectionDescriptor {
            transport: "udp",
            client_peer: client,
            proxy_local: proxy,
        };
        let udp_table = SocketTable::new(
            Timestamp::from_nanos(15),
            vec![
                fragcap_attr::SocketTableEntry::udp(client, 73)
                    .created_at(Timestamp::from_nanos(12)),
                fragcap_attr::SocketTableEntry::udp(proxy, 9).created_at(Timestamp::from_nanos(10)),
            ],
        );
        assert_eq!(
            select_inception_socket_owner(&udp_table, association, 15).pid,
            Some(73)
        );
        assert_eq!(
            select_inception_socket_owner(&udp_table, association, 11).state,
            "unavailable"
        );

        let wildcard_v4 = fragcap_attr::SocketTableEntry::udp("0.0.0.0:41000".parse().unwrap(), 74)
            .created_at(Timestamp::from_nanos(12));
        let wildcard_v6 = fragcap_attr::SocketTableEntry::udp("[::]:41000".parse().unwrap(), 75)
            .created_at(Timestamp::from_nanos(12));
        let only_wildcard = SocketTable::new(Timestamp::from_nanos(15), vec![wildcard_v4]);
        assert_eq!(
            select_inception_socket_owner(&only_wildcard, association, 15).pid,
            Some(74)
        );
        let only_dual_stack = SocketTable::new(Timestamp::from_nanos(15), vec![wildcard_v6]);
        assert_eq!(
            select_inception_socket_owner(&only_dual_stack, association, 15).pid,
            Some(75)
        );
        let exact_and_wildcard = SocketTable::new(
            Timestamp::from_nanos(15),
            vec![
                wildcard_v4,
                fragcap_attr::SocketTableEntry::udp(client, 73)
                    .created_at(Timestamp::from_nanos(12)),
            ],
        );
        assert_eq!(
            select_inception_socket_owner(&exact_and_wildcard, association, 15).pid,
            Some(73)
        );
        let ambiguous_wildcard = SocketTable::new(
            Timestamp::from_nanos(15),
            vec![
                wildcard_v4,
                fragcap_attr::SocketTableEntry::udp("0.0.0.0:41000".parse().unwrap(), 76)
                    .created_at(Timestamp::from_nanos(13)),
            ],
        );
        assert_eq!(
            select_inception_socket_owner(&ambiguous_wildcard, association, 15).state,
            "ambiguous"
        );
        let ipv6_association = ConnectionDescriptor {
            client_peer: "[::1]:41000".parse().unwrap(),
            proxy_local: "[::1]:42000".parse().unwrap(),
            ..association
        };
        assert_eq!(
            select_inception_socket_owner(&only_dual_stack, ipv6_association, 15).pid,
            Some(75)
        );
    }

    #[test]
    fn connection_owner_requires_exact_process_generation_and_managed_ancestry() {
        let mut evidence = super::super::CaptureProcessEvidence {
            launch_pid: Some(10),
            launch_at: Some(Timestamp::from_nanos(1)),
            watcher_report: Some(fragcap_core::WatcherReport::default()),
            ..Default::default()
        };
        evidence.events.extend([
            ProcessEvent::started(10, 1, "steam.exe", "steam", Timestamp::from_nanos(1)),
            ProcessEvent::started(20, 10, "dispatch.exe", "dispatch", Timestamp::from_nanos(2)),
            ProcessEvent::started(
                30,
                20,
                "publisher.exe",
                "publisher",
                Timestamp::from_nanos(3),
            ),
            ProcessEvent::started(71, 30, "helper.exe", "helper", Timestamp::from_nanos(4)),
            ProcessEvent::started(72, 99, "helper.exe", "unrelated", Timestamp::from_nanos(4)),
        ]);
        let exact = super::super::ConnectionInception {
            state: "exact",
            reason: "ip-helper-client-socket",
            pid: Some(71),
            socket_created_at_ns: Some(12),
        };
        let owner = bind_inception_process(&evidence, exact, 15);
        assert_eq!(
            (
                owner.state.as_str(),
                owner.pid,
                owner.process_instance_id.as_deref()
            ),
            ("exact", Some(71), Some("process-71-4"))
        );
        let unrelated = bind_inception_process(
            &evidence,
            super::super::ConnectionInception {
                pid: Some(72),
                ..exact
            },
            15,
        );
        assert_eq!(
            (unrelated.state.as_str(), unrelated.reason.as_str()),
            ("unavailable", "socket-owner-managed-ancestry-unproven")
        );
        evidence.events.push(ProcessEvent::Exited {
            pid: 71,
            at: Timestamp::from_nanos(13),
        });
        assert_eq!(
            bind_inception_process(&evidence, exact, 15).reason,
            "socket-owner-generation-changed-before-accept"
        );
        evidence.events_unretained = 1;
        assert_eq!(
            bind_inception_process(&evidence, exact, 15).reason,
            "process-event-authority-incomplete"
        );
    }

    #[test]
    fn semantic_result_timestamp_has_exact_cutoff_and_unknown_is_ineligible() {
        assert_eq!(
            native_evidence_window(Some(100), Some(100)),
            super::super::EvidenceWindow::Observation
        );
        assert_eq!(
            native_evidence_window(Some(101), Some(100)),
            super::super::EvidenceWindow::OwnerRelease
        );
        assert_eq!(
            native_evidence_window(None, Some(100)),
            super::super::EvidenceWindow::Unavailable
        );
        assert_eq!(
            native_evidence_window(Some(99), None),
            super::super::EvidenceWindow::Unavailable
        );
    }

    #[test]
    fn observation_drain_requires_clean_stop_and_every_terminal_predicate() {
        let clean_stop = NativeStopAuthority {
            clean: true,
            listener_released: true,
            incomplete_tasks: 0,
            residue: false,
        };
        assert!(observation_drain_is_complete(
            Some(clean_stop),
            fragcap_proxy::LifecycleState::Stopped,
            0,
            0,
        ));

        let owner_thread_failure = NativeStopAuthority {
            clean: false,
            listener_released: false,
            incomplete_tasks: 0,
            residue: true,
        };
        for (stop, state, live_connections, connection_tasks_current) in [
            (None, fragcap_proxy::LifecycleState::Stopped, 0, 0),
            (
                Some(owner_thread_failure),
                fragcap_proxy::LifecycleState::Stopped,
                0,
                0,
            ),
            (
                Some(clean_stop),
                fragcap_proxy::LifecycleState::Stopping,
                0,
                0,
            ),
            (
                Some(clean_stop),
                fragcap_proxy::LifecycleState::Stopped,
                1,
                0,
            ),
            (
                Some(clean_stop),
                fragcap_proxy::LifecycleState::Stopped,
                0,
                1,
            ),
        ] {
            assert!(!observation_drain_is_complete(
                stop,
                state,
                live_connections,
                connection_tasks_current,
            ));
        }
    }

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
    fn launcher_pair_connection_ids_keep_distinct_non_client_owners() {
        let registry = FlowRegistry::default();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let pair = [
            (1_u64, "127.0.0.1:41000", 77_u32, "launcher.exe", "platform"),
            (2_u64, "127.0.0.1:41001", 88_u32, "updater.exe", "publisher"),
        ];
        for (_, peer, pid, image, role) in pair {
            let client: SocketAddr = peer.parse().unwrap();
            registry.observe_at(
                FlowKey::new(Proto::Tcp, client, proxy),
                Timestamp::from_nanos(20),
                Some(&Attribution::new(pid, image, Fidelity::Live).with_role(role)),
            );
        }
        let joined: Vec<_> = pair
            .into_iter()
            .map(|(id, peer, _, _, _)| {
                let client = peer.parse().unwrap();
                (
                    id,
                    correlate_native_observation(&registry, client, proxy, 10, 30),
                )
            })
            .collect();
        assert_eq!(joined[0].0, 1);
        assert_eq!((joined[0].1).1, Some(77));
        assert_eq!((joined[0].1).3.as_deref(), Some("platform"));
        assert_eq!(joined[1].0, 2);
        assert_eq!((joined[1].1).1, Some(88));
        assert_eq!((joined[1].1).3.as_deref(), Some("publisher"));
        assert!(joined.iter().all(|(_, value)| {
            value.7 == CorrelationState::Matched && value.3.as_deref() != Some("client")
        }));
        let missing = correlate_native_observation(
            &registry,
            "127.0.0.1:41002".parse().unwrap(),
            proxy,
            10,
            30,
        );
        assert_eq!(missing.7, CorrelationState::Unavailable);
        assert!(missing.1.is_none());
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
    fn early_scope_withholding_does_not_veto_later_exact_connection() {
        use fragcap_core::FlowOmissionCause;
        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        registry.mark_unretained_at(
            key,
            Timestamp::from_nanos(10),
            FlowOmissionCause::ScopeWithheld,
        );
        registry.observe_at(
            key,
            Timestamp::from_nanos(30),
            Some(&Attribution::new(7, "selected.exe", Fidelity::Live).with_role("client")),
        );
        let later = correlate_native_observation(&registry, client, proxy, 25, 40);
        assert_eq!(later.1, Some(7));
        assert_eq!(later.7, CorrelationState::Matched);
        assert_eq!(later.6, 1);
        let overlapping = correlate_native_observation(&registry, client, proxy, 5, 40);
        assert_eq!(overlapping.7, CorrelationState::Unavailable);
        assert_eq!(overlapping.8, "packet-scope-withheld");
    }

    #[test]
    fn bounded_omission_runs_refuse_a_join_without_losing_the_packet_count() {
        use fragcap_core::FlowOmissionCause;
        let registry = FlowRegistry::default();
        let client: SocketAddr = "127.0.0.1:41000".parse().unwrap();
        let proxy: SocketAddr = "127.0.0.1:42000".parse().unwrap();
        let key = FlowKey::new(Proto::Tcp, client, proxy);
        for timestamp in 0..65 {
            registry.mark_unretained_at(
                key,
                Timestamp::from_nanos(timestamp),
                if timestamp % 2 == 0 {
                    FlowOmissionCause::ScopeWithheld
                } else {
                    FlowOmissionCause::HistoryCapacity
                },
            );
        }
        registry.observe_at(
            key,
            Timestamp::from_nanos(100),
            Some(&Attribution::new(7, "selected.exe", Fidelity::Live)),
        );
        let summary = registry.summary(&key).unwrap();
        assert_eq!(summary.unretained_observations, 65);
        assert!(summary.omission_precision_lost);
        assert!(summary.omissions.is_empty());
        let result = correlate_native_observation(&registry, client, proxy, 90, 110);
        assert_eq!(result.6, 65);
        assert_eq!(result.7, CorrelationState::Unavailable);
        assert_eq!(result.8, "packet-omission-history-bound-exceeded");
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
                inception: Arc::new(Mutex::new(super::super::ConnectionInception::default())),
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

    fn loopback_packet_owner(
        client: SocketAddr,
        proxy: SocketAddr,
        timestamp: Timestamp,
        role: &str,
    ) -> Attribution {
        use fragcap_core::{FlowAttributor, StageId};

        let inner = crate::SocketTableAttributor::new(
            Box::new(crate::DeclaredTable::once(crate::SocketTable::new(
                timestamp,
                vec![
                    crate::SocketTableEntry::tcp(proxy, client, 9),
                    crate::SocketTableEntry::tcp(client, proxy, 7),
                ],
            ))),
            Box::new(crate::DeclaredNames::from([
                (7, "selected.exe"),
                (9, "fragcap.exe"),
            ])),
            Arc::new(crate::TestClock::at(timestamp)),
            crate::AttributorConfig::default(),
        );
        inner.refresh().unwrap();
        let stamper = crate::session::RoleStampingAttributor::new(Arc::new(inner));
        stamper
            .publisher()
            .publish(vec![(7, Some(Arc::from(role)), Some(StageId::new(role)))]);
        let (local, remote) = if client <= proxy {
            (client, proxy)
        } else {
            (proxy, client)
        };
        stamper
            .resolve(&FlowKey::new(Proto::Tcp, local, remote), timestamp)
            .unwrap()
    }

    #[test]
    fn canonical_loopback_owner_reaches_native_join_in_both_port_orders_and_families() {
        for ip in ["127.0.0.1", "::1"] {
            for (client_port, proxy_port) in [(41000, 42000), (42000, 41000)] {
                let client = SocketAddr::new(ip.parse().unwrap(), client_port);
                let proxy = SocketAddr::new(ip.parse().unwrap(), proxy_port);
                let at = Timestamp::from_nanos(20);
                let owner = loopback_packet_owner(client, proxy, at, "client");
                let registry = FlowRegistry::default();
                let (local, remote) = if client <= proxy {
                    (client, proxy)
                } else {
                    (proxy, client)
                };
                registry.observe_at(FlowKey::new(Proto::Tcp, local, remote), at, Some(&owner));

                let result = correlate_native_observation(&registry, client, proxy, 10, 30);
                assert_eq!(result.1, Some(7));
                assert_eq!(result.2.as_deref(), Some("selected.exe"));
                assert_eq!(result.3.as_deref(), Some("client"));
                assert_eq!(result.5, 1);
                assert_eq!(result.7, CorrelationState::Matched);
                assert_eq!(result.8, "exact-flow-and-owner");
            }
        }
    }

    // The proxy and HTTP origin below use real finite sockets. Packet ownership
    // is deliberately supplied by declared socket tables, not live Npcap. Keep
    // controlled=false so neither final join nor facts use the harness shortcut.
    fn native_join_test_lease(
        ip: std::net::IpAddr,
        origin: SocketAddr,
        path: &std::path::Path,
    ) -> NativeProxyLease {
        let context = NativeObservationContext::default();
        let registry = context.flow_registry();
        let application = super::super::ApplicationArtifactLease::open_correlated(
            path,
            "s169-native-join",
            super::super::DEFAULT_APPLICATION_EVENT_QUEUE_CAPACITY,
            Arc::new(move |window| {
                let result = correlate_connection_window(&registry, window);
                super::super::ApplicationCorrelation {
                    target_id: Some(1),
                    flow_id: result.0,
                    process_id: result.1,
                    process_image: result.2,
                    role: result.3,
                    attribution: result.4,
                    packet_observations: result.5,
                    packet_observations_unretained: result.6,
                    state: Some(result.7.as_str().to_string()),
                    reason: Some(result.8),
                    connection_owner: None,
                }
            }),
        )
        .unwrap();
        let config =
            NativeProxyConfig::new(SocketAddr::new(ip, 0), 4, 16 * 1024, Duration::from_secs(3))
                .unwrap()
                .with_session_id("s169-native-join")
                .unwrap();
        let mut policy = DestinationPolicy::new(SocketAddr::new(ip, 0));
        policy.grant_for_test(origin);
        let runtime = RuntimeBackend::new(config)
            .with_destination_policy(policy)
            .with_application_event_sink(application.sink())
            .start(Duration::from_secs(3))
            .unwrap();
        NativeProxyLease {
            lease: runtime,
            controlled: false,
            controlled_lab: None,
            observation_context: context,
            application_artifact: Some(application),
            proxy_lifecycle: None,
            key_log: None,
            observations_lost: 0,
            classification_loss_accounted: 0,
            application_classification_summary: None,
            stop_authority: None,
            diagnostics: None,
            diagnostics_appended: false,
            observation_ended_at_ns: None,
        }
    }

    #[test]
    fn final_proxy_classification_loss_is_accounted_once_across_stop_and_cleanup() {
        let directory = tempfile::tempdir().unwrap();
        let ip = "127.0.0.1".parse().unwrap();
        let mut lease = native_join_test_lease(
            ip,
            SocketAddr::new(ip, 9),
            &directory.path().join("application.jsonl"),
        );

        // A failed stop may return a loss total before cleanup finalizes the
        // writer. Repeated totals must not inflate the final classification.
        lease.observations_lost = 2;
        lease.account_final_classification_loss();
        lease.account_final_classification_loss();
        lease.observations_lost = 3;
        lease.account_final_classification_loss();
        lease.cleanup(Budget::new(Duration::from_secs(3)));
        assert_eq!(
            lease
                .application_classification_summary
                .unwrap()
                .unclassified_lost,
            3
        );
    }

    #[test]
    fn real_http_exchange_reconciles_packet_owner_application_and_facts() {
        use crate::targets::CompatibilityFactKey;
        use std::sync::mpsc;
        use std::time::Instant;

        for ip in ["127.0.0.1", "::1"] {
            for role in [Some("client"), Some("platform"), None] {
                let ip = ip.parse().unwrap();
                let directory = tempfile::tempdir().unwrap();
                let path = directory.path().join("application.jsonl");
                let origin = TcpListener::bind(SocketAddr::new(ip, 0)).unwrap();
                let origin_address = origin.local_addr().unwrap();
                origin.set_nonblocking(true).unwrap();
                let (reached_sender, reached_receiver) = mpsc::sync_channel(1);
                let (release_sender, release_receiver) = mpsc::sync_channel(1);
                let worker = std::thread::spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(3);
                    let mut stream = loop {
                        match origin.accept() {
                            Ok((stream, _)) => break stream,
                            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                assert!(Instant::now() < deadline, "origin accept timed out");
                                std::thread::sleep(Duration::from_millis(5));
                            }
                            Err(error) => panic!("origin accept failed: {error}"),
                        }
                    };
                    // Windows accepts can inherit the listener's nonblocking
                    // mode. Switch the connected stream to bounded blocking I/O
                    // before reading, independently of request arrival timing.
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let mut head = Vec::new();
                    while !head.ends_with(b"\r\n\r\n") {
                        assert!(head.len() < 4096, "origin request exceeded its bound");
                        let mut byte = [0];
                        stream.read_exact(&mut byte).unwrap();
                        head.push(byte[0]);
                    }
                    reached_sender.send(()).unwrap();
                    release_receiver
                        .recv_timeout(Duration::from_secs(3))
                        .unwrap();
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nS169")
                        .unwrap();
                });
                let mut lease = native_join_test_lease(ip, origin_address, &path);
                let proxy = lease.lease.endpoint();
                let mut client = TcpStream::connect(proxy).unwrap();
                client
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                client
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let client_address = client.local_addr().unwrap();
                write!(
                    client,
                    "GET http://{origin_address}/s169 HTTP/1.1\r\nHost: {origin_address}\r\nProxy-Authorization: {}\r\nConnection: close\r\n\r\n",
                    lease.lease.capability_proof().proxy_authorization().as_str(),
                )
                .unwrap();
                reached_receiver
                    .recv_timeout(Duration::from_secs(3))
                    .unwrap();
                // The accepted request is waiting at the origin, so this exact
                // observation lies within the real proxy connection lifetime.
                if let Some(role) = role {
                    let at = Timestamp::from_nanos(
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                    );
                    let owner = loopback_packet_owner(client_address, proxy, at, role);
                    let (local, remote) = if client_address <= proxy {
                        (client_address, proxy)
                    } else {
                        (proxy, client_address)
                    };
                    lease.observation_context.flow_registry.observe_at(
                        FlowKey::new(Proto::Tcp, local, remote),
                        at,
                        Some(&owner),
                    );
                }
                release_sender.send(()).unwrap();
                let mut response = Vec::new();
                client.read_to_end(&mut response).unwrap();
                assert!(response.starts_with(b"HTTP/1.1 200"));
                assert!(response.ends_with(b"S169"));
                worker.join().unwrap();
                lease.end_observation_window(SystemTime::now());
                assert_eq!(
                    lease.stop(Budget::new(Duration::from_secs(3))).status,
                    CleanupStatus::Released
                );
                let (qualified, status) = lease
                    .drain_phase_observations(Budget::new(Duration::from_secs(1)))
                    .unwrap()
                    .unwrap()
                    .into_parts();
                assert_eq!(status, super::super::ObservationDrainStatus::Complete);
                assert!(qualified.iter().all(
                    |value| value.evidence_window == super::super::EvidenceWindow::Observation
                ));
                let windows = qualified
                    .iter()
                    .map(|value| value.evidence_window)
                    .collect::<Vec<_>>();
                let observations = qualified
                    .into_iter()
                    .map(|value| value.observation)
                    .collect::<Vec<_>>();
                assert_eq!(observations.len(), 1);
                let observation = &observations[0];
                assert_eq!(observation.status, Some(200));
                assert_eq!(observation.client_peer, Some(client_address));
                assert_eq!(observation.proxy_local, Some(proxy));
                let expected_state = if role.is_some() {
                    "matched"
                } else {
                    "unavailable"
                };
                let expected_reason = if role.is_some() {
                    "exact-flow-and-owner"
                } else {
                    "packet-flow-not-observed"
                };
                assert_eq!(observation.correlation_state.as_str(), expected_state);
                assert_eq!(observation.correlation_reason, expected_reason);
                assert_eq!(observation.role.as_deref(), role);
                assert_eq!(observation.packet_observations, u64::from(role.is_some()));
                let facts = super::super::compatibility_fact_candidates_in_windows(
                    super::super::LaunchCase::SteamProtocolCold.as_str(),
                    &observations,
                    &windows,
                    false,
                    Some(super::super::CalibrationPhase::Reachability),
                    None,
                );
                for (key, expected) in [
                    (
                        CompatibilityFactKey::ProxyRouting,
                        if role == Some("client") {
                            "reached-client"
                        } else {
                            "inconclusive"
                        },
                    ),
                    (
                        CompatibilityFactKey::ProxyPropagation,
                        if role == Some("client") {
                            "confirmed"
                        } else {
                            "not-confirmed"
                        },
                    ),
                ] {
                    assert_eq!(
                        facts.iter().find(|fact| fact.key == key).unwrap().value,
                        expected
                    );
                }
                let records = std::fs::read_to_string(&path)
                    .unwrap()
                    .lines()
                    .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                    .collect::<Vec<_>>();
                let trailer = records.last().unwrap();
                assert_eq!(trailer["type"], "application.trailer");
                assert_eq!(trailer["writer_status"], "complete");
                assert_eq!(trailer["dropped_records"], 0);
                assert_eq!(trailer["writer_failures"], 0);
                assert_eq!(trailer["accepted_records"], trailer["written_records"]);
                assert_eq!(trailer["body_bytes_observed"], 4);
                assert_eq!(trailer["body_bytes_retained"], 4);
                let correlations = records
                    .iter()
                    .filter(|record| record["type"] == "application.correlation")
                    .collect::<Vec<_>>();
                assert_eq!(correlations.len(), 1);
                let correlation = correlations[0];
                assert_eq!(correlation["correlation_state"], expected_state);
                assert_eq!(correlation["correlation_reason"], expected_reason);
                assert_eq!(correlation["process_id"].as_u64(), role.map(|_| 7));
                assert_eq!(correlation["role"].as_str(), role);
                assert_eq!(
                    correlation["packet_observations"],
                    observation.packet_observations
                );
                assert_eq!(correlation["packet_observations_unretained"], 0);
                assert_eq!(
                    correlation["flow_id"].as_str(),
                    observation.flow_id.map(|id| id.to_string()).as_deref()
                );
                assert!(lease
                    .cleanup(Budget::new(Duration::from_secs(3)))
                    .iter()
                    .all(|result| result.status == CleanupStatus::Released));
            }
        }
    }
}
