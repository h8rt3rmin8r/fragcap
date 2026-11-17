// SPDX-License-Identifier: Apache-2.0

//! Finite, prearmed packet route for exact managed sockets.

use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use fragcap_core::{
    parse::{raw_transport_head, raw_udp_datagram},
    Proto,
};
use zeroize::Zeroizing;

use super::{
    direct_bridge::{
        connect_authenticated_socks5_staged, DirectBridgeError, DirectTcpBridge, HelperFailure,
    },
    direct_observer::{CurrentOwner, DirectSocketObserver, SocketObserverStats},
    direct_route::{
        CandidateBuffer, CandidateDecision, DirectPacketAccounting, DirectReceiptLedger,
        DirectSocketEvent, DirectTuple, OriginalFlow, PacketDisposition, RouteReceiptId,
        SocketEpoch,
    },
    direct_stack::StackError,
    direct_udp::{build_udp_reply, DirectUdpAssociation},
    native::DirectRouteProof,
    route_events::{
        RouteEvent, RouteEventKind, RouteEventLease, RouteEventSink, RouteStreamOutcome,
    },
    windivert::{sample_qpc_anchor, Address, Api, Handle, NETWORK_LAYER},
    NativeObservationContext,
};

const PACKET_QUEUE: usize = 256;
const CANDIDATE_QUEUE_LIMIT: usize = PACKET_QUEUE / 2;
const CANDIDATE_UDP_QUEUE_LIMIT: usize = CANDIDATE_QUEUE_LIMIT - CANDIDATE_QUEUE_LIMIT / 8;
const CANDIDATE_BYTES: usize = 4 * 1024 * 1024;
// An unknown-owner packet may belong to unrelated workstation traffic.
// Leave 10 ms of the physical 50 ms sentinel budget for scheduling and
// reinjection after the ownership wait. Queue stalls are measured separately;
// this deadline alone cannot guarantee physical pass-through latency.
const CANDIDATE_WAIT_NS: u64 = 40_000_000;
const SOCKET_EVENTS: usize = 4096;
const EVENT_HISTORY_NS: i64 = 300_000_000;
const REFUSED_FLOWS: usize = 4096;
const MAX_PENDING_HELPERS: usize = 16;
const MAX_PENDING_PACKETS: usize = 8;
// Selection and subsequent client packets are outbound. Returned selected
// bytes enter through the synthetic-reply path, so diverting unrelated
// inbound traffic only adds work and latency to every other connection.
const PACKET_FILTER: &str = "outbound and (tcp or udp)";
const WORKER_STALL_THRESHOLD_MS: u64 = 90;
const UDP_PROXY_CONNECT_TIMEOUT: Duration = Duration::from_millis(50);
const TCP_PROXY_CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const ACK_CLOSE_GRACE: Duration = Duration::from_secs(1);
// The socket observer and network receiver run independently. A packet already
// captured under an exact selected epoch may reach the worker just after the
// observer reports CLOSE and a different owner reuses the tuple. Keep the old
// bridge briefly so the queued packet is handled by its own route, never sent
// raw into the new owner's connection.
const SELECTED_RETIRE_GRACE: Duration = Duration::from_millis(100);
const SELECTED_HISTORY_GRACE: Duration = Duration::from_secs(1);
const MAX_ACK_WITNESSES: usize = 4096;

#[cfg(test)]
static STALL_NEXT_CANDIDATE: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static FAIL_NEXT_CLEANUP_REPORT: AtomicBool = AtomicBool::new(false);

fn unix_now_ns() -> io::Result<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| io::Error::other("system clock before Unix epoch"))?;
    u64::try_from(duration.as_nanos())
        .map_err(|_| io::Error::other("system clock beyond route timestamp range"))
}

#[cfg(test)]
fn candidate_timeline(
    received_ns: u64,
    socket_event_published_ns: Option<u64>,
    process_proof_observed_ns: Option<u64>,
    decided_ns: u64,
    outcome: &str,
) {
    // Controlled metadata-only timeline. No packet or application bytes leave
    // the candidate path, including for unrelated owners.
    eprintln!(
        "S201 candidate timing: receipt_ns={received_ns}, socket_event_published_ns={socket_event_published_ns:?}, process_proof_observed_ns={process_proof_observed_ns:?}, decision_ns={decided_ns}, held_ns={}, outcome={outcome}",
        decided_ns.saturating_sub(received_ns),
    );
}

enum WorkPacket {
    Candidate(super::direct_route::DirectPacketHead, Vec<u8>, Address, u64),
    Selected(SelectedRoute, Vec<u8>, Address),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SelectedRoute {
    epoch: SocketEpoch,
    flow_id: u64,
}

#[derive(Clone, Copy)]
struct RetiredSelection {
    retire_qpc: i64,
    bridge_until: Instant,
    history_until: Instant,
    bridge_available: bool,
}

#[derive(Default)]
struct SelectedRegistry {
    current: BTreeMap<DirectTuple, SelectedRoute>,
    retiring: BTreeMap<SelectedRoute, RetiredSelection>,
    retired_by_tuple: BTreeMap<DirectTuple, BTreeSet<SelectedRoute>>,
    // A late NETWORK packet can outlive both finite socket-event history and
    // the bridge grace. Keep a bounded session-long tuple fence so it cannot
    // fall through to unscoped raw reinjection after history is pruned.
    quarantined: BTreeMap<DirectTuple, i64>,
    queued: BTreeMap<SelectedRoute, usize>,
}

impl SelectedRegistry {
    fn enqueue(&mut self, route: SelectedRoute) {
        *self.queued.entry(route).or_default() += 1;
    }

    fn finish(&mut self, route: SelectedRoute) {
        let count = self
            .queued
            .get_mut(&route)
            .expect("selected work must have an outstanding queue receipt");
        assert!(*count > 0, "selected work queue receipt cannot underflow");
        *count -= 1;
        if *count == 0 {
            self.queued.remove(&route);
        }
    }

    fn queued(&self, route: SelectedRoute) -> usize {
        self.queued.get(&route).copied().unwrap_or(0)
    }

    fn record_retirement(
        &mut self,
        route: SelectedRoute,
        retire_qpc: i64,
        has_bridge: bool,
    ) -> bool {
        if self.retiring.len() >= SOCKET_EVENTS && !self.retiring.contains_key(&route) {
            return false;
        }
        if self.quarantined.len() >= SOCKET_EVENTS
            && !self.quarantined.contains_key(&route.epoch.tuple)
        {
            return false;
        }
        let now = Instant::now();
        if self.current.get(&route.epoch.tuple) == Some(&route) {
            self.current.remove(&route.epoch.tuple);
        }
        self.retiring.insert(
            route,
            RetiredSelection {
                retire_qpc,
                bridge_until: now + SELECTED_RETIRE_GRACE,
                history_until: now + SELECTED_HISTORY_GRACE,
                bridge_available: has_bridge,
            },
        );
        self.retired_by_tuple
            .entry(route.epoch.tuple)
            .or_default()
            .insert(route);
        self.quarantined
            .entry(route.epoch.tuple)
            .and_modify(|cutoff| *cutoff = (*cutoff).max(retire_qpc))
            .or_insert(retire_qpc);
        true
    }

    fn has_tuple(&self, tuple: DirectTuple) -> bool {
        self.current.contains_key(&tuple) || self.quarantined.contains_key(&tuple)
    }

    fn unique_lineage_route(&self, tuple: DirectTuple) -> Option<SelectedRoute> {
        let current = self.current.get(&tuple).copied();
        let retired = self.retired_by_tuple.get(&tuple);
        match (current, retired) {
            (Some(route), None) => Some(route),
            (None, Some(routes)) if routes.len() == 1 => routes.iter().next().copied(),
            _ => None,
        }
    }

    fn retired_at(
        &self,
        tuple: DirectTuple,
        epoch: SocketEpoch,
        packet_qpc: i64,
    ) -> Option<(SelectedRoute, bool)> {
        self.retired_by_tuple.get(&tuple)?.iter().find_map(|route| {
            let retirement = self.retiring.get(route)?;
            (route.epoch == epoch
                && route.epoch.contains_packet(packet_qpc)
                && packet_qpc < retirement.retire_qpc
                && Instant::now() < retirement.history_until)
                .then_some((
                    *route,
                    retirement.bridge_available && Instant::now() < retirement.bridge_until,
                ))
        })
    }

    fn exact_retired_route_at_close(
        &self,
        tuple: DirectTuple,
        retire_qpc: i64,
    ) -> Option<SelectedRoute> {
        let mut matches = self.retired_by_tuple.get(&tuple)?.iter().filter(|route| {
            self.retiring
                .get(route)
                .is_some_and(|retired| retired.retire_qpc == retire_qpc)
        });
        let route = *matches.next()?;
        matches.next().is_none().then_some(route)
    }

    fn prune_history(&mut self, now: Instant) {
        let expired: Vec<_> = self
            .retiring
            .iter()
            .filter_map(|(route, retirement)| {
                (!retirement.bridge_available
                    && self.queued(*route) == 0
                    && now >= retirement.history_until)
                    .then_some(*route)
            })
            .collect();
        for route in expired {
            self.retiring.remove(&route);
            if let Some(routes) = self.retired_by_tuple.get_mut(&route.epoch.tuple) {
                routes.remove(&route);
                if routes.is_empty() {
                    self.retired_by_tuple.remove(&route.epoch.tuple);
                }
            }
        }
    }
}

enum SelectedIngress {
    Route {
        route: SelectedRoute,
        historical: CurrentOwner,
        bridge_available: bool,
    },
    Quarantined {
        tuple: DirectTuple,
        historical: CurrentOwner,
        last_retire_qpc: i64,
    },
}

// A socket can emit a header-only TCP reset or FIN just after its CLOSE event.
// This is an unattributed control tail: losing it is counted, but it must not
// be returned to a potentially reused tuple or confused with selected data.
fn selected_post_close_control_tail(
    packet: &[u8],
    packet_qpc: i64,
    close_qpc: Option<i64>,
    historical: CurrentOwner,
    live: CurrentOwner,
) -> bool {
    if historical != CurrentOwner::Absent
        || live != CurrentOwner::Absent
        || close_qpc.is_none_or(|close| packet_qpc <= close)
    {
        return false;
    }
    let Ok(head) = raw_transport_head(packet) else {
        return false;
    };
    let Some(flags) = head.tcp_flags else {
        return false;
    };
    if head.transport != Proto::Tcp || flags & 0x05 == 0 || flags & 0x02 != 0 {
        return false;
    }
    let Some(transport) =
        packet.get(head.transport_offset..head.transport_offset + head.transport_len)
    else {
        return false;
    };
    if transport.len() < 20 {
        return false;
    }
    let header_len = usize::from(transport[12] >> 4) * 4;
    header_len >= 20 && header_len == transport.len()
}

fn selected_tcp_omission_detail(
    packet: &[u8],
    packet_qpc: i64,
    exact_close_qpc: Option<i64>,
) -> Option<&'static str> {
    let head = raw_transport_head(packet).ok()?;
    if head.transport != Proto::Tcp {
        return None;
    }
    let transport =
        packet.get(head.transport_offset..head.transport_offset + head.transport_len)?;
    if transport.len() < 20 {
        return None;
    }
    let header_len = usize::from(transport[12] >> 4) * 4;
    if header_len < 20 || header_len > transport.len() {
        return None;
    }
    let payload = transport.len() > header_len;
    Some(match (payload, exact_close_qpc) {
        (true, Some(close)) if packet_qpc < close => "selected-tcp-payload-before-close-unproven",
        (false, Some(close)) if packet_qpc < close => {
            "selected-tcp-header-only-before-close-unproven"
        }
        (true, Some(close)) if packet_qpc == close => {
            "selected-tcp-payload-close-boundary-unproven"
        }
        (false, Some(close)) if packet_qpc == close => {
            "selected-tcp-header-only-close-boundary-unproven"
        }
        (true, Some(_)) => "selected-tcp-payload-after-close-unproven",
        (false, Some(_)) => "selected-tcp-header-only-after-close-unproven",
        (true, None) => "selected-tcp-payload-close-unavailable",
        (false, None) => "selected-tcp-header-only-close-unavailable",
    })
}

fn selected_loss_reason(
    default: &'static str,
    route: SelectedRoute,
    packet: &[u8],
    packet_qpc: i64,
    historical: CurrentOwner,
    live: CurrentOwner,
    observer: &Arc<Mutex<DirectSocketObserver>>,
) -> &'static str {
    if historical != CurrentOwner::Absent || live != CurrentOwner::Absent {
        return default;
    }
    let close_qpc = observer
        .lock()
        .expect("socket observer lock")
        .closed_qpc_for_exact_epoch(route.epoch);
    if selected_post_close_control_tail(packet, packet_qpc, close_qpc, historical, live) {
        "selected-tuple-post-close-control-unattributed"
    } else if matches!(
        default,
        "selected-owner-epoch-changed-or-ambiguous"
            | "selected-packet-history-ambiguous-or-changed"
    ) {
        selected_tcp_omission_detail(packet, packet_qpc, close_qpc).unwrap_or(default)
    } else {
        default
    }
}

fn udp_unproven_bypass_reason(packet_qpc: i64, exact_close_qpc: Option<i64>) -> &'static str {
    match exact_close_qpc {
        Some(close_qpc) if packet_qpc < close_qpc => {
            "selected-udp-before-close-owner-unproven-original-reinjected"
        }
        Some(close_qpc) if packet_qpc == close_qpc => {
            "selected-udp-close-boundary-original-reinjected"
        }
        Some(_) => "selected-udp-after-close-original-reinjected",
        None => "selected-udp-close-unavailable-original-reinjected",
    }
}

fn udp_post_close_lineage_flow(
    route: SelectedRoute,
    unique_lineage: Option<SelectedRoute>,
    historical: CurrentOwner,
    live: CurrentOwner,
    packet_qpc: i64,
    exact_close_qpc: Option<i64>,
) -> Option<u64> {
    (unique_lineage == Some(route)
        && historical == CurrentOwner::Absent
        && live == CurrentOwner::Absent
        && exact_close_qpc.is_some_and(|close_qpc| packet_qpc > close_qpc))
    .then_some(route.flow_id)
}

fn quarantined_packet_is_proven_new(
    historical: CurrentOwner,
    live: CurrentOwner,
    last_retire_qpc: i64,
    packet_qpc: i64,
) -> bool {
    match (historical, live) {
        (CurrentOwner::Unique(epoch), CurrentOwner::Unique(current)) => {
            epoch == current
                && epoch.open_qpc >= last_retire_qpc
                && epoch.contains_packet(packet_qpc)
        }
        _ => false,
    }
}

struct SelectedWorkGuard {
    selected: Arc<Mutex<SelectedRegistry>>,
    route: SelectedRoute,
}

impl Drop for SelectedWorkGuard {
    fn drop(&mut self) {
        self.selected
            .lock()
            .expect("selected route lock")
            .finish(self.route);
    }
}

struct RetiredTcp {
    flow: ActiveTcp,
    retire_qpc: i64,
    bridge_until: Instant,
    reason: &'static str,
}

struct RetiredUdp {
    flow: ActiveUdp,
    retire_qpc: i64,
    bridge_until: Instant,
    reason: &'static str,
}

fn selected_packet_admission(
    selected: SelectedRoute,
    current: Option<SocketEpoch>,
    packet_qpc: i64,
) -> Result<(), &'static str> {
    if !selected.epoch.contains_packet(packet_qpc) {
        return Err("selected-packet-before-open-or-clock-ambiguous");
    }
    if current != Some(selected.epoch) {
        return Err("selected-owner-epoch-changed-or-ambiguous");
    }
    Ok(())
}

/// A selected TCP packet may outlive its socket event or worker route because
/// the WinDivert handles and worker queue are independent. Return the TCP
/// original only while the exact same owner remains live. An absent owner may
/// mean the next OPEN has not reached the socket observer yet.
fn stale_selected_can_pass_original(
    selected: SelectedRoute,
    current: CurrentOwner,
    packet_qpc: i64,
) -> bool {
    selected.epoch.contains_packet(packet_qpc) && current == CurrentOwner::Unique(selected.epoch)
}

fn fatal_packet_quarantine(
    selected: &SelectedRegistry,
    packet: &[u8],
    address: Address,
) -> (bool, Option<u64>) {
    if !address.outbound() {
        return (false, None);
    }
    let Ok(head) = raw_transport_head(packet) else {
        // With socket evidence lost, an unparseable outbound packet cannot be
        // proven unrelated to a selected tuple.
        return (true, None);
    };
    let tuple = DirectTuple {
        transport: head.transport,
        local: head.source,
        remote: head.destination,
    };
    if !selected.has_tuple(tuple) {
        return (false, None);
    }
    let flow_id = selected
        .current
        .get(&tuple)
        .map(|route| route.flow_id)
        .or_else(|| {
            let routes = selected.retired_by_tuple.get(&tuple)?;
            (routes.len() == 1).then(|| routes.iter().next().expect("one retired route").flow_id)
        });
    (true, flow_id)
}

fn selected_work_admission(
    queued: SelectedRoute,
    selected_now: Option<SelectedRoute>,
    owner_now: Option<SocketEpoch>,
    target_epoch: Option<SocketEpoch>,
    packet_qpc: i64,
) -> Result<(), &'static str> {
    if selected_now != Some(queued) || target_epoch != Some(queued.epoch) {
        return Err("selected-route-generation-retired");
    }
    selected_packet_admission(queued, owner_now, packet_qpc)
}

fn queue_candidate_with_selected_reserve(
    sender: &SyncSender<WorkPacket>,
    queued_candidates: &AtomicUsize,
    queued_udp_candidates: &AtomicUsize,
    candidate: super::direct_route::DirectPacketHead,
    packet: &[u8],
    address: Address,
    received_ns: u64,
) -> bool {
    let udp = candidate.tuple.transport == Proto::Udp;
    if udp {
        if queued_udp_candidates.load(Ordering::Acquire) >= CANDIDATE_UDP_QUEUE_LIMIT {
            return false;
        }
        queued_udp_candidates.fetch_add(1, Ordering::AcqRel);
    }
    if queued_candidates.load(Ordering::Acquire) >= CANDIDATE_QUEUE_LIMIT {
        if udp {
            queued_udp_candidates.fetch_sub(1, Ordering::AcqRel);
        }
        return false;
    }
    queued_candidates.fetch_add(1, Ordering::AcqRel);
    if sender
        .try_send(WorkPacket::Candidate(
            candidate,
            packet.to_vec(),
            address,
            received_ns,
        ))
        .is_ok()
    {
        true
    } else {
        queued_candidates.fetch_sub(1, Ordering::AcqRel);
        if udp {
            queued_udp_candidates.fetch_sub(1, Ordering::AcqRel);
        }
        false
    }
}

fn receive_work(
    receiver: &Receiver<WorkPacket>,
    queued_candidates: &AtomicUsize,
    queued_udp_candidates: &AtomicUsize,
) -> Result<WorkPacket, mpsc::RecvTimeoutError> {
    let work = receiver.recv_timeout(Duration::from_millis(1))?;
    if let WorkPacket::Candidate(head, ..) = &work {
        queued_candidates.fetch_sub(1, Ordering::AcqRel);
        if head.tuple.transport == Proto::Udp {
            queued_udp_candidates.fetch_sub(1, Ordering::AcqRel);
        }
    }
    Ok(work)
}

#[derive(Clone, Debug, Default)]
pub struct DirectRuntimeStats {
    pub packets: DirectPacketAccounting,
    pub socket_events: SocketObserverStats,
    pub admission_busy_bypassed: u64,
    pub candidate_capacity_bypassed: u64,
    pub candidate_max_queue_us: u64,
    pub candidate_max_hold_us: u64,
    pub candidate_over_50ms: u64,
    pub bypass_reasons: BTreeMap<&'static str, u64>,
    pub proxy_refused: u64,
    pub udp_relay_resets: u64,
    pub synthetic_reply_lost: u64,
    pub event_history_overflow: bool,
    pub pre_receive_loss_unmeasurable: bool,
    pub watchdog_released: bool,
    pub watchdog_release_after_progress_ms: u64,
    pub route_stream: Option<RouteStreamOutcome>,
}

fn record_route(sink: Option<&RouteEventSink>, flow_id: u64, kind: RouteEventKind) {
    if let Some(sink) = sink {
        let _ = sink.record(RouteEvent {
            flow_id,
            at_ns: unix_now_ns().unwrap_or(0),
            clock_error_ns: None,
            kind,
        });
    }
}

fn bridge_failure_reason(error: &DirectBridgeError) -> &'static str {
    match error {
        DirectBridgeError::Stack(StackError::UntrustedChecksum) => "untrusted-ip-checksum",
        DirectBridgeError::Stack(StackError::FlowMismatch) => "private-stack-flow-mismatch",
        DirectBridgeError::Stack(StackError::PacketTooLarge) => "private-stack-packet-too-large",
        DirectBridgeError::Stack(StackError::QueueFull) => "private-stack-queue-full",
        DirectBridgeError::Stack(StackError::InvalidScope) => "private-stack-invalid-scope",
        DirectBridgeError::Stack(StackError::ListenFailed) => "private-stack-listen-failed",
        DirectBridgeError::Io(_) => "proxy-helper-io-failed",
    }
}

struct SharedStats {
    started: Instant,
    packets: Mutex<DirectPacketAccounting>,
    processor_progress_ms: AtomicU64,
    watchdog_released: AtomicBool,
    watchdog_release_after_progress_ms: AtomicU64,
    stop_watchdog: AtomicBool,
    admission_busy_bypassed: AtomicU64,
    candidate_capacity_bypassed: AtomicU64,
    candidate_max_queue_us: AtomicU64,
    candidate_max_hold_us: AtomicU64,
    candidate_over_50ms: AtomicU64,
    bypass_reasons: Mutex<BTreeMap<&'static str, u64>>,
    proxy_refused: AtomicU64,
    udp_relay_resets: AtomicU64,
    synthetic_reply_lost: AtomicU64,
    event_history_overflow: AtomicBool,
}

impl SharedStats {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            packets: Mutex::new(DirectPacketAccounting::default()),
            processor_progress_ms: AtomicU64::new(1),
            watchdog_released: AtomicBool::new(false),
            watchdog_release_after_progress_ms: AtomicU64::new(0),
            stop_watchdog: AtomicBool::new(false),
            admission_busy_bypassed: AtomicU64::new(0),
            candidate_capacity_bypassed: AtomicU64::new(0),
            candidate_max_queue_us: AtomicU64::new(0),
            candidate_max_hold_us: AtomicU64::new(0),
            candidate_over_50ms: AtomicU64::new(0),
            bypass_reasons: Mutex::new(BTreeMap::new()),
            proxy_refused: AtomicU64::new(0),
            udp_relay_resets: AtomicU64::new(0),
            synthetic_reply_lost: AtomicU64::new(0),
            event_history_overflow: AtomicBool::new(false),
        }
    }

    fn stalled_packet(&self, now_ms: u64) -> bool {
        self.packets
            .lock()
            .expect("route accounting lock")
            .in_flight
            != 0
            && now_ms.saturating_sub(self.processor_progress_ms.load(Ordering::Acquire))
                >= WORKER_STALL_THRESHOLD_MS
    }

    fn processor_progress(&self) {
        self.processor_progress_ms.store(
            self.started.elapsed().as_millis() as u64 + 1,
            Ordering::Release,
        );
    }

    fn record_bypass(&self, reason: &'static str) {
        let mut reasons = self.bypass_reasons.lock().expect("bypass reason lock");
        let count = reasons.entry(reason).or_insert(0);
        *count = count.checked_add(1).expect("bounded route bypass count");
    }

    fn record_candidate_queue_delay(&self, received_ns: u64) {
        let elapsed = (self.started.elapsed().as_nanos() as u64).saturating_sub(received_ns);
        self.candidate_max_queue_us
            .fetch_max(elapsed / 1_000, Ordering::Relaxed);
    }

    fn record_candidate_terminal_delay(&self, received_ns: u64) {
        let elapsed = (self.started.elapsed().as_nanos() as u64).saturating_sub(received_ns);
        self.candidate_max_hold_us
            .fetch_max(elapsed / 1_000, Ordering::Relaxed);
        if elapsed > 50_000_000 {
            self.candidate_over_50ms.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn packet_received(stats: &SharedStats) -> io::Result<()> {
    stats
        .packets
        .lock()
        .expect("route accounting lock")
        .receive()
        .map_err(|error| io::Error::other(format!("packet receive accounting: {error:?}")))
}

fn packet_finished(stats: &SharedStats, disposition: PacketDisposition) -> io::Result<()> {
    stats
        .packets
        .lock()
        .expect("route accounting lock")
        .finish(disposition)
        .map_err(|error| io::Error::other(format!("packet terminal accounting: {error:?}")))
}

fn mark_refused_flow(
    refused: &Mutex<BTreeMap<DirectTuple, SocketEpoch>>,
    epoch: SocketEpoch,
) -> bool {
    let mut flows = refused.lock().expect("refused route lock");
    if flows.len() >= REFUSED_FLOWS && !flows.contains_key(&epoch.tuple) {
        return false;
    }
    flows.insert(epoch.tuple, epoch);
    true
}

fn reinject(
    handle: &Handle,
    stats: &SharedStats,
    packet: &[u8],
    address: &Address,
) -> io::Result<()> {
    match handle.send(packet, address) {
        Ok(sent) if sent == packet.len() => packet_finished(stats, PacketDisposition::Reinjected),
        Ok(_) => {
            packet_finished(stats, PacketDisposition::Lost)?;
            Err(io::Error::other("short packet reinjection"))
        }
        Err(error) => {
            packet_finished(stats, PacketDisposition::Lost)?;
            Err(error)
        }
    }
}

fn reinject_uninspected_udp(
    handle: &Handle,
    stats: &SharedStats,
    packet: &[u8],
    address: &Address,
    route_events: Option<&RouteEventSink>,
    flow_id: Option<u64>,
    reason: &'static str,
) -> io::Result<()> {
    let result = reinject(handle, stats, packet, address);
    if let Some(flow_id) = flow_id {
        record_route(
            route_events,
            flow_id,
            RouteEventKind::PacketOmission {
                reason: if result.is_ok() {
                    reason
                } else {
                    "selected-udp-original-reinjection-failed"
                },
                disposition: if result.is_ok() { "reinjected" } else { "lost" },
            },
        );
    }
    result
}

fn tcp_payload_end(packet: &[u8]) -> Option<u32> {
    let head = raw_transport_head(packet).ok()?;
    if head.transport != Proto::Tcp {
        return None;
    }
    let transport =
        packet.get(head.transport_offset..head.transport_offset + head.transport_len)?;
    let header_len = usize::from(transport[12] >> 4) * 4;
    let payload_len = transport.len().checked_sub(header_len)?;
    if payload_len == 0 {
        return None;
    }
    let sequence = u32::from_be_bytes(transport[4..8].try_into().ok()?);
    Some(sequence.wrapping_add(u32::try_from(payload_len).ok()?))
}

fn tcp_acknowledges_returned_data(packet: &[u8], first_data_end: u32) -> bool {
    let Ok(head) = raw_transport_head(packet) else {
        return false;
    };
    if head.transport != Proto::Tcp || !head.tcp_ack {
        return false;
    }
    let Some(transport) =
        packet.get(head.transport_offset..head.transport_offset + head.transport_len)
    else {
        return false;
    };
    let acknowledged = u32::from_be_bytes(transport[8..12].try_into().expect("validated TCP head"));
    acknowledged.wrapping_sub(first_data_end) < (1 << 31)
}

fn inject_replies(
    handle: &Handle,
    stats: &SharedStats,
    replies: Vec<Vec<u8>>,
    address: Address,
    route_events: Option<&RouteEventSink>,
    flow_id: u64,
    proof: Option<&Arc<Mutex<DirectRouteProof>>>,
) -> io::Result<()> {
    if !replies.is_empty() {
        let (if_idx, sub_if_idx) = address.network_interface();
        record_route(
            route_events,
            flow_id,
            RouteEventKind::SyntheticReplyGenerated {
                packets: replies.len() as u32,
                if_idx,
                sub_if_idx,
            },
        );
    }
    for reply in replies {
        let sent = if let Some(proof) = proof {
            let mut proof = proof.lock().expect("direct proof lock");
            let sent = handle.send(&reply, &address);
            if matches!(&sent, Ok(length) if *length == reply.len()) {
                proof.reply_sent = true;
                if proof.first_returned_data_end.is_none() {
                    proof.first_returned_data_end = tcp_payload_end(&reply);
                }
            } else {
                proof.reply_failed = true;
            }
            sent
        } else {
            handle.send(&reply, &address)
        };
        if !matches!(sent, Ok(length) if length == reply.len()) {
            record_route(
                route_events,
                flow_id,
                RouteEventKind::SyntheticReplyFailed {
                    reason: "send-failed",
                },
            );
            record_route(
                route_events,
                flow_id,
                RouteEventKind::Terminal {
                    reason: "reply-injection-failed",
                },
            );
            stats.synthetic_reply_lost.fetch_add(1, Ordering::Relaxed);
            return Err(io::Error::other("synthetic TCP reply injection failed"));
        }
        record_route(
            route_events,
            flow_id,
            RouteEventKind::SyntheticReplySent { packets: 1 },
        );
    }
    Ok(())
}

struct ReceiveRuntimeInputs {
    handle: Arc<Handle>,
    context: NativeObservationContext,
    proxy: std::net::SocketAddr,
    sender: SyncSender<WorkPacket>,
    queued_candidates: Arc<AtomicUsize>,
    queued_udp_candidates: Arc<AtomicUsize>,
    observer: Arc<Mutex<DirectSocketObserver>>,
    selected: Arc<Mutex<SelectedRegistry>>,
    ack_witnesses: Arc<Mutex<BTreeMap<DirectTuple, AckWitness>>>,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    refused: Arc<Mutex<BTreeMap<DirectTuple, SocketEpoch>>>,
    connecting: Arc<AtomicBool>,
    stats: Arc<SharedStats>,
    route_events: Option<RouteEventSink>,
}

// The proxy's exact listener is infrastructure, regardless of when socket
// ownership evidence arrives. Other unknown-owner packets remain candidates
// while a helper connects, so a concurrent game flow is not lost.
fn helper_packet_bypasses_candidate(
    destination: std::net::SocketAddr,
    proxy: std::net::SocketAddr,
    connecting: bool,
    owner_pid: Option<u32>,
    helper_pid: u32,
) -> bool {
    destination == proxy || (connecting && owner_pid == Some(helper_pid))
}

#[derive(Clone)]
struct AckWitness {
    epoch: SocketEpoch,
    flow_id: u64,
    proof: Arc<Mutex<DirectRouteProof>>,
    closed_at: Option<Instant>,
    close_qpc: Option<i64>,
}

impl AckWitness {
    fn accepts_owner(&self, current_owner: CurrentOwner, packet_qpc: i64) -> bool {
        if !self.epoch.contains_packet(packet_qpc)
            || self
                .close_qpc
                .is_some_and(|close_qpc| packet_qpc >= close_qpc)
        {
            return false;
        }
        match current_owner {
            CurrentOwner::Unique(owner) if owner == self.epoch => true,
            CurrentOwner::Unique(owner) => self
                .close_qpc
                .is_some_and(|close_qpc| packet_qpc < close_qpc && owner.open_qpc > packet_qpc),
            CurrentOwner::Absent => self
                .close_qpc
                .is_some_and(|close_qpc| packet_qpc < close_qpc),
            CurrentOwner::Ambiguous => false,
        }
    }
}

fn observe_returned_ack(
    packet: &[u8],
    tuple: DirectTuple,
    packet_qpc: i64,
    observer: &Arc<Mutex<DirectSocketObserver>>,
    witnesses: &Arc<Mutex<BTreeMap<DirectTuple, AckWitness>>>,
    route_events: Option<&RouteEventSink>,
) {
    let Some(witness) = witnesses
        .lock()
        .expect("ACK witness lock")
        .get(&tuple)
        .cloned()
    else {
        return;
    };
    let current_owner = observer
        .lock()
        .expect("socket observer lock")
        .current_owner_state(tuple);
    if !witness.accepts_owner(current_owner, packet_qpc) {
        return;
    }
    let mut proof = witness.proof.lock().expect("direct proof lock");
    let acknowledged = !proof.returned_data_acknowledged
        && proof
            .first_returned_data_end
            .is_some_and(|end| tcp_acknowledges_returned_data(packet, end));
    if !acknowledged {
        return;
    }
    proof.client_ack = true;
    proof.returned_data_acknowledged = true;
    drop(proof);
    record_route(
        route_events,
        witness.flow_id,
        RouteEventKind::ReturnedDataAcknowledged,
    );
    let mut witnesses = witnesses.lock().expect("ACK witness lock");
    if witnesses
        .get(&tuple)
        .is_some_and(|current| current.epoch == witness.epoch && current.flow_id == witness.flow_id)
    {
        witnesses.remove(&tuple);
    }
}

fn receive_packets(inputs: ReceiveRuntimeInputs) -> io::Result<()> {
    let ReceiveRuntimeInputs {
        handle,
        context,
        proxy,
        sender,
        queued_candidates,
        queued_udp_candidates,
        observer,
        selected,
        ack_witnesses,
        excluded,
        refused,
        connecting,
        stats,
        route_events,
    } = inputs;
    let mut packet = [0u8; 65_535];
    loop {
        let mut address = Address::default();
        let length = match handle.recv(&mut packet, &mut address) {
            Ok(length) => length,
            Err(error) if error.raw_os_error() == Some(232) => return Ok(()),
            Err(error) => return Err(error),
        };
        // Candidate hold time starts when WinDivert delivers the packet, not
        // when the processing worker eventually dequeues it.
        let received_ns = stats.started.elapsed().as_nanos() as u64;
        packet_received(&stats)?;
        let bytes = &packet[..length];
        let event_health = observer.lock().expect("socket observer lock").stats();
        if event_health.capacity_lost != 0
            || event_health.clock_lost != 0
            || stats.event_history_overflow.load(Ordering::Acquire)
        {
            let (quarantine, flow_id) = fatal_packet_quarantine(
                &selected.lock().expect("selected route lock"),
                bytes,
                address,
            );
            let udp = raw_transport_head(bytes)
                .ok()
                .is_some_and(|head| head.transport == Proto::Udp);
            let sent = if quarantine && udp {
                stats.record_bypass("selected-evidence-loss-udp-original-reinjected");
                // No socket history can support an old-flow claim. Return the
                // original UDP datagram unchanged, then release broad routing.
                reinject_uninspected_udp(
                    &handle,
                    &stats,
                    bytes,
                    &address,
                    route_events.as_ref(),
                    None,
                    "selected-evidence-loss-udp-original-reinjected",
                )
            } else if quarantine {
                stats.record_bypass("selected-evidence-loss-quarantined");
                if let Some(flow_id) = flow_id {
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::PacketOmission {
                            reason: "selected-evidence-loss-quarantined",
                            disposition: "lost",
                        },
                    );
                }
                packet_finished(&stats, PacketDisposition::Lost)
            } else {
                reinject(&handle, &stats, bytes, &address)
            };
            let _ = handle.stop();
            sent?;
            return Err(io::Error::other(
                "direct socket evidence was lost; broad route released",
            ));
        }
        let head = raw_transport_head(bytes).ok();
        let selected_route = head
            .as_ref()
            .filter(|_| address.outbound())
            .and_then(|head| {
                let tuple = DirectTuple {
                    transport: head.transport,
                    local: head.source,
                    remote: head.destination,
                };
                if !selected
                    .lock()
                    .expect("selected route lock")
                    .has_tuple(tuple)
                {
                    return None;
                }
                // The socket and network handles can deliver events in either
                // order. Query the packet's QPC against retained exact ownership,
                // then consult the current or briefly retired selected route.
                let historical = observer
                    .lock()
                    .expect("socket observer lock")
                    .owner_at_qpc(tuple, address.timestamp_qpc);
                let registry = selected.lock().expect("selected route lock");
                let retired = match historical {
                    CurrentOwner::Unique(epoch) => {
                        registry.retired_at(tuple, epoch, address.timestamp_qpc)
                    }
                    CurrentOwner::Absent | CurrentOwner::Ambiguous => None,
                };
                let current = registry.current.get(&tuple).copied();
                match (historical, current, retired) {
                    (CurrentOwner::Unique(epoch), Some(route), _) if route.epoch == epoch => {
                        Some(SelectedIngress::Route {
                            route,
                            historical,
                            bridge_available: true,
                        })
                    }
                    (_, _, Some((route, bridge_available))) => Some(SelectedIngress::Route {
                        route,
                        historical,
                        bridge_available,
                    }),
                    (_, Some(route), _) => Some(SelectedIngress::Route {
                        route,
                        historical,
                        bridge_available: true,
                    }),
                    _ => registry
                        .quarantined
                        .get(&tuple)
                        .copied()
                        .map(|last_retire_qpc| SelectedIngress::Quarantined {
                            tuple,
                            historical,
                            last_retire_qpc,
                        }),
                }
            });
        if let Some(SelectedIngress::Quarantined {
            tuple,
            historical,
            last_retire_qpc,
        }) = selected_route
        {
            let live = observer
                .lock()
                .expect("socket observer lock")
                .current_owner_state(tuple);
            let proven_new = quarantined_packet_is_proven_new(
                historical,
                live,
                last_retire_qpc,
                address.timestamp_qpc,
            );
            if !proven_new {
                let old_route = selected
                    .lock()
                    .expect("selected route lock")
                    .exact_retired_route_at_close(tuple, last_retire_qpc);
                let reason = old_route.map_or("selected-tuple-quarantined", |route| {
                    selected_loss_reason(
                        "selected-tuple-quarantined",
                        route,
                        bytes,
                        address.timestamp_qpc,
                        historical,
                        live,
                        &observer,
                    )
                });
                if tuple.transport == Proto::Udp {
                    let exact_close_qpc = old_route.and_then(|route| {
                        observer
                            .lock()
                            .expect("socket observer lock")
                            .closed_qpc_for_exact_epoch(route.epoch)
                    });
                    let bypass_reason =
                        udp_unproven_bypass_reason(address.timestamp_qpc, exact_close_qpc);
                    stats.record_bypass(bypass_reason);
                    let unique_lineage = selected
                        .lock()
                        .expect("selected route lock")
                        .unique_lineage_route(tuple);
                    let flow_id = old_route.and_then(|route| {
                        udp_post_close_lineage_flow(
                            route,
                            unique_lineage,
                            historical,
                            live,
                            address.timestamp_qpc,
                            exact_close_qpc,
                        )
                    });
                    // The packet owner is unresolved, so it cannot borrow the
                    // retired proxy route. Return this one datagram unchanged
                    // without claiming selected transfer or inspecting bytes.
                    reinject_uninspected_udp(
                        &handle,
                        &stats,
                        bytes,
                        &address,
                        route_events.as_ref(),
                        flow_id,
                        bypass_reason,
                    )?;
                    continue;
                }
                stats.record_bypass(reason);
                packet_finished(&stats, PacketDisposition::Lost)?;
                if reason == "selected-tuple-post-close-control-unattributed" {
                    record_route(
                        route_events.as_ref(),
                        old_route
                            .expect("classified control tail has an exact retired route")
                            .flow_id,
                        RouteEventKind::PacketOmission {
                            reason,
                            disposition: "lost",
                        },
                    );
                }
                continue;
            }
        }
        if let Some(SelectedIngress::Route {
            route,
            historical,
            bridge_available,
        }) = selected_route
        {
            let current_state = observer
                .lock()
                .expect("socket observer lock")
                .current_owner_state(route.epoch.tuple);
            let current = match current_state {
                CurrentOwner::Unique(epoch) => Some(epoch),
                CurrentOwner::Absent | CurrentOwner::Ambiguous => None,
            };
            let historical_epoch = match historical {
                CurrentOwner::Unique(epoch) => Some(epoch),
                CurrentOwner::Absent | CurrentOwner::Ambiguous => None,
            };
            if let Err(default_reason) =
                selected_packet_admission(route, historical_epoch, address.timestamp_qpc)
            {
                let reason = selected_loss_reason(
                    default_reason,
                    route,
                    bytes,
                    address.timestamp_qpc,
                    historical,
                    current_state,
                    &observer,
                );
                let pass_new_owner = current.is_some_and(|owner| {
                    owner != route.epoch && owner.contains_packet(address.timestamp_qpc)
                });
                if pass_new_owner
                    && head.as_ref().is_some_and(|head| {
                        (head.transport == Proto::Tcp && head.initial_tcp_syn)
                            || head.transport == Proto::Udp
                    })
                {
                    if let Some(candidate) = sample_qpc_anchor()
                        .ok()
                        .and_then(|clock| address.initial_packet_head(bytes, clock))
                    {
                        if queue_candidate_with_selected_reserve(
                            &sender,
                            &queued_candidates,
                            &queued_udp_candidates,
                            candidate,
                            bytes,
                            address,
                            received_ns,
                        ) {
                            continue;
                        }
                        stats
                            .candidate_capacity_bypassed
                            .fetch_add(1, Ordering::Relaxed);
                    }
                }
                if route.epoch.tuple.transport == Proto::Udp {
                    let exact_close_qpc = observer
                        .lock()
                        .expect("socket observer lock")
                        .closed_qpc_for_exact_epoch(route.epoch);
                    let bypass_reason =
                        udp_unproven_bypass_reason(address.timestamp_qpc, exact_close_qpc);
                    stats.record_bypass(bypass_reason);
                    let unique_lineage = selected
                        .lock()
                        .expect("selected route lock")
                        .unique_lineage_route(route.epoch.tuple);
                    let flow_id = udp_post_close_lineage_flow(
                        route,
                        unique_lineage,
                        historical,
                        current_state,
                        address.timestamp_qpc,
                        exact_close_qpc,
                    );
                    reinject_uninspected_udp(
                        &handle,
                        &stats,
                        bytes,
                        &address,
                        route_events.as_ref(),
                        flow_id,
                        bypass_reason,
                    )?;
                    continue;
                }
                stats.record_bypass(reason);
                let reinjected = historical == CurrentOwner::Unique(route.epoch)
                    && stale_selected_can_pass_original(
                        route,
                        current_state,
                        address.timestamp_qpc,
                    );
                if reinjected {
                    reinject(&handle, &stats, bytes, &address)?;
                } else {
                    packet_finished(&stats, PacketDisposition::Lost)?;
                }
                if !pass_new_owner {
                    record_route(
                        route_events.as_ref(),
                        route.flow_id,
                        RouteEventKind::PacketOmission {
                            reason,
                            disposition: if reinjected { "reinjected" } else { "lost" },
                        },
                    );
                }
                continue;
            }
            // The route may retire between the first lookup and enqueue. Hold
            // the registry lock through the nonblocking channel send so the
            // worker sees an exact outstanding count before retirement.
            let queued = if bridge_available {
                let mut registry = selected.lock().expect("selected route lock");
                let current = registry.current.get(&route.epoch.tuple).copied() == Some(route);
                let retired = registry
                    .retired_at(route.epoch.tuple, route.epoch, address.timestamp_qpc)
                    .is_some_and(|(matched, available)| matched == route && available);
                if current || retired {
                    registry.enqueue(route);
                    let sent = sender
                        .try_send(WorkPacket::Selected(route, bytes.to_vec(), address))
                        .is_ok();
                    if !sent {
                        registry.finish(route);
                    }
                    Some(sent)
                } else {
                    None
                }
            } else {
                None
            };
            if queued != Some(true) {
                let reason = if queued == Some(false) {
                    "selected-work-queue-saturated"
                } else {
                    "selected-retired-bridge-expired"
                };
                stats.record_bypass(reason);
                let reinjected = route.epoch.tuple.transport == Proto::Udp;
                if reinjected {
                    reinject_uninspected_udp(
                        &handle,
                        &stats,
                        bytes,
                        &address,
                        route_events.as_ref(),
                        Some(route.flow_id),
                        reason,
                    )?;
                } else {
                    packet_finished(&stats, PacketDisposition::Lost)?;
                    record_route(
                        route_events.as_ref(),
                        route.flow_id,
                        RouteEventKind::PacketOmission {
                            reason,
                            disposition: "lost",
                        },
                    );
                }
                if queued == Some(false) {
                    let _ = handle.stop();
                    return Err(io::Error::other("selected packet queue saturated"));
                }
            }
            continue;
        }
        // A managed direct target has one exact root. Once the socket table
        // names a different PID, its packet cannot be selected by this route.
        // Pass it through on the receive worker so unrelated traffic never
        // waits behind a selected flow's proxy work. Unknown owners retain
        // the bounded candidate path.
        if let (Some(root_pid), Some(head)) = (context.direct_managed_root_pid(), head.as_ref()) {
            let tuple = DirectTuple {
                transport: head.transport,
                local: head.source,
                remote: head.destination,
            };
            if observer
                .lock()
                .expect("socket observer lock")
                .current_owner_pid(tuple)
                .is_some_and(|owner_pid| owner_pid != root_pid)
            {
                stats.record_bypass("other-managed-root-pid");
                reinject(&handle, &stats, bytes, &address)?;
                continue;
            }
        }
        if address.outbound() {
            if let Some(head) = head
                .as_ref()
                .filter(|head| head.transport == Proto::Tcp && head.tcp_ack)
            {
                observe_returned_ack(
                    bytes,
                    DirectTuple {
                        transport: Proto::Tcp,
                        local: head.source,
                        remote: head.destination,
                    },
                    address.timestamp_qpc,
                    &observer,
                    &ack_witnesses,
                    route_events.as_ref(),
                );
            }
        }
        let is_candidate = address.outbound()
            && head.as_ref().is_some_and(|head| {
                (head.transport == Proto::Tcp && head.initial_tcp_syn)
                    || head.transport == Proto::Udp
            });
        let is_excluded = head.as_ref().is_some_and(|head| {
            excluded
                .lock()
                .expect("helper route lock")
                .contains(&DirectTuple {
                    transport: head.transport,
                    local: head.source,
                    remote: head.destination,
                })
        });
        let is_refused = head.as_ref().is_some_and(|head| {
            let tuple = DirectTuple {
                transport: head.transport,
                local: head.source,
                remote: head.destination,
            };
            let refused_epoch = refused
                .lock()
                .expect("refused route lock")
                .get(&tuple)
                .copied();
            refused_epoch.is_some_and(|epoch| {
                epoch.contains_packet(address.timestamp_qpc)
                    && observer
                        .lock()
                        .expect("socket observer lock")
                        .current_owner_epoch(tuple)
                        == Some(epoch)
            })
        });
        let proxy_endpoint = head.as_ref().is_some_and(|head| head.destination == proxy);
        let helper_connecting = connecting.load(Ordering::Acquire);
        let helper_bypass = head.as_ref().is_some_and(|head| {
            let owner_pid = if helper_connecting && !proxy_endpoint {
                observer
                    .lock()
                    .expect("socket observer lock")
                    .current_owner_pid(DirectTuple {
                        transport: head.transport,
                        local: head.source,
                        remote: head.destination,
                    })
            } else {
                None
            };
            helper_packet_bypasses_candidate(
                head.destination,
                proxy,
                helper_connecting,
                owner_pid,
                std::process::id(),
            )
        });
        if !is_candidate || helper_bypass || is_excluded || is_refused {
            if is_refused {
                stats.record_bypass("proxy-refused-flow");
            } else if is_excluded {
                stats.record_bypass("helper-self-excluded");
            } else if proxy_endpoint && is_candidate {
                stats.record_bypass("helper-proxy-endpoint");
            } else if helper_bypass && is_candidate {
                stats
                    .admission_busy_bypassed
                    .fetch_add(1, Ordering::Relaxed);
            }
            reinject(&handle, &stats, bytes, &address)?;
            continue;
        }
        let candidate = sample_qpc_anchor()
            .ok()
            .and_then(|clock| address.initial_packet_head(bytes, clock));
        if let Some(candidate) = candidate {
            if queue_candidate_with_selected_reserve(
                &sender,
                &queued_candidates,
                &queued_udp_candidates,
                candidate,
                bytes,
                address,
                received_ns,
            ) {
                continue;
            }
            stats
                .candidate_capacity_bypassed
                .fetch_add(1, Ordering::Relaxed);
        }
        reinject(&handle, &stats, bytes, &address)?;
    }
}

struct ActiveTcp {
    bridge: Option<DirectTcpBridge>,
    address: Address,
    flow: OriginalFlow,
    context: NativeObservationContext,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    epoch: SocketEpoch,
    receipt: RouteReceiptId,
    helper: DirectTuple,
    flow_id: u64,
    last_transfer: (u64, u64),
    proof: Arc<Mutex<DirectRouteProof>>,
}

impl Drop for ActiveTcp {
    fn drop(&mut self) {
        drop(self.bridge.take());
        let mut excluded = self.excluded.lock().expect("helper route lock");
        excluded.remove(&self.helper);
        drop(excluded);
        self.context.retire_direct_helper_registration(&self.flow);
    }
}

struct PendingTcp {
    flow: OriginalFlow,
    epoch: SocketEpoch,
    packet: Vec<u8>,
    address: Address,
    response_address: Address,
    receipt: RouteReceiptId,
    flow_id: u64,
    proof: Arc<Mutex<DirectRouteProof>>,
    result: Receiver<Result<(std::net::TcpStream, u64), HelperFailure>>,
    worker: JoinHandle<()>,
    cancelled: Arc<AtomicBool>,
    extra: Vec<(Vec<u8>, Address)>,
}

fn helper_failure_outcome(stage: &str, kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => "timeout",
        io::ErrorKind::PermissionDenied
            if matches!(stage, "method-reply" | "auth-reply" | "connect-reply") =>
        {
            "refused"
        }
        _ => "io-error",
    }
}

fn udp_association_failure_outcome(kind: io::ErrorKind) -> (&'static str, &'static str) {
    match kind {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => {
            ("timeout", "udp-association-timeout")
        }
        _ => ("io-error", "udp-association-io-failed"),
    }
}

fn hold_pending_packet(
    pending: &mut PendingTcp,
    packet: Vec<u8>,
    address: Address,
    stats: &SharedStats,
    route_events: Option<&RouteEventSink>,
) -> io::Result<()> {
    if pending.packet == packet || pending.extra.iter().any(|(held, _)| *held == packet) {
        // The first SYN is still held. A retransmission has no new payload to
        // deliver and must not start a second helper or escape to the origin.
        return packet_finished(stats, PacketDisposition::Consumed);
    }
    if pending.extra.len() < MAX_PENDING_PACKETS {
        pending.extra.push((packet, address));
    } else {
        stats.record_bypass("pending-packet-capacity-lost");
        packet_finished(stats, PacketDisposition::Lost)?;
        record_route(
            route_events,
            pending.flow_id,
            RouteEventKind::PacketOmission {
                reason: "pending-packet-capacity-lost",
                disposition: "lost",
            },
        );
    }
    Ok(())
}

fn retire_pending_tcp(
    pending: PendingTcp,
    stats: &SharedStats,
    route_events: Option<&RouteEventSink>,
    reason: &'static str,
    retired_helpers: &mut Vec<JoinHandle<()>>,
) -> io::Result<RouteReceiptId> {
    pending.cancelled.store(true, Ordering::Release);
    lose_pending_first_packet(stats, route_events, pending.flow_id, &pending.proof)?;
    for _ in pending.extra {
        packet_finished(stats, PacketDisposition::Lost)?;
        record_route(
            route_events,
            pending.flow_id,
            RouteEventKind::PacketOmission {
                reason: "pending-packet-after-owner-retired",
                disposition: "lost",
            },
        );
    }
    record_route(
        route_events,
        pending.flow_id,
        RouteEventKind::Terminal { reason },
    );
    retired_helpers.push(pending.worker);
    Ok(pending.receipt)
}

fn lose_pending_first_packet(
    stats: &SharedStats,
    route_events: Option<&RouteEventSink>,
    flow_id: u64,
    proof: &Arc<Mutex<DirectRouteProof>>,
) -> io::Result<()> {
    packet_finished(stats, PacketDisposition::Lost)?;
    proof
        .lock()
        .expect("direct proof lock")
        .selected_packet_lost = true;
    record_route(
        route_events,
        flow_id,
        RouteEventKind::SelectedPacketDisposition {
            disposition: "lost",
        },
    );
    Ok(())
}

fn reinject_selected_first_packet(
    handle: &Handle,
    stats: &SharedStats,
    packet: &[u8],
    address: &Address,
    proof: &Arc<Mutex<DirectRouteProof>>,
    route_events: Option<&RouteEventSink>,
    flow_id: u64,
) -> io::Result<()> {
    let result = reinject(handle, stats, packet, address);
    let disposition = if result.is_ok() {
        proof
            .lock()
            .expect("direct proof lock")
            .selected_packet_reinjected = true;
        "reinjected"
    } else {
        proof
            .lock()
            .expect("direct proof lock")
            .selected_packet_lost = true;
        "lost"
    };
    record_route(
        route_events,
        flow_id,
        RouteEventKind::SelectedPacketDisposition { disposition },
    );
    result
}

fn reinject_pending_selected_packet(
    handle: &Handle,
    stats: &SharedStats,
    packet: &[u8],
    address: &Address,
    route_events: Option<&RouteEventSink>,
    flow_id: u64,
) -> io::Result<()> {
    let result = reinject(handle, stats, packet, address);
    record_route(
        route_events,
        flow_id,
        RouteEventKind::PacketOmission {
            reason: "pending-packet-after-setup-failed",
            disposition: if result.is_ok() { "reinjected" } else { "lost" },
        },
    );
    result
}

fn lose_unforwarded_pending_packets(
    stats: &SharedStats,
    route_events: Option<&RouteEventSink>,
    flow_id: u64,
    count: usize,
) -> io::Result<()> {
    let mut first_error = None;
    for _ in 0..count {
        if let Err(error) = packet_finished(stats, PacketDisposition::Lost) {
            first_error.get_or_insert(error);
        } else {
            record_route(
                route_events,
                flow_id,
                RouteEventKind::PacketOmission {
                    reason: "pending-packet-after-forward-failed",
                    disposition: "lost",
                },
            );
        }
    }
    first_error.map_or(Ok(()), Err)
}

struct ActiveUdp {
    association: Option<DirectUdpAssociation>,
    address: Address,
    flow: OriginalFlow,
    context: NativeObservationContext,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    epoch: SocketEpoch,
    receipt: RouteReceiptId,
    relay: DirectTuple,
    control: DirectTuple,
    flow_id: u64,
    proof: Arc<Mutex<DirectRouteProof>>,
}

impl ActiveUdp {
    fn association(&self) -> &DirectUdpAssociation {
        self.association.as_ref().expect("active UDP association")
    }
}

impl Drop for ActiveUdp {
    fn drop(&mut self) {
        // Close the control and relay sockets before revoking live helper
        // ownership. The historical accepted window remains for artifacts.
        drop(self.association.take());
        let mut excluded = self.excluded.lock().expect("helper route lock");
        excluded.remove(&self.relay);
        excluded.remove(&self.control);
        drop(excluded);
        self.context.retire_direct_helper_registration(&self.flow);
    }
}

struct PendingUdpHelperRegistration {
    context: NativeObservationContext,
    flow: OriginalFlow,
    adopted: bool,
}

impl PendingUdpHelperRegistration {
    fn new(context: NativeObservationContext, flow: OriginalFlow) -> Self {
        Self {
            context,
            flow,
            adopted: false,
        }
    }

    fn adopt(mut self) {
        self.adopted = true;
    }
}

impl Drop for PendingUdpHelperRegistration {
    fn drop(&mut self) {
        if !self.adopted {
            self.context.retire_direct_helper_registration(&self.flow);
        }
    }
}

struct ProcessRuntimeInputs {
    receiver: Receiver<WorkPacket>,
    ready: mpsc::Sender<()>,
    queued_candidates: Arc<AtomicUsize>,
    queued_udp_candidates: Arc<AtomicUsize>,
    handle: Arc<Handle>,
    observer: Arc<Mutex<DirectSocketObserver>>,
    context: NativeObservationContext,
    selected: Arc<Mutex<SelectedRegistry>>,
    ack_witnesses: Arc<Mutex<BTreeMap<DirectTuple, AckWitness>>>,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    refused: Arc<Mutex<BTreeMap<DirectTuple, SocketEpoch>>>,
    connecting: Arc<AtomicBool>,
    stats: Arc<SharedStats>,
    session_id: String,
    proxy: std::net::SocketAddr,
    password: Zeroizing<String>,
    max_flows: usize,
    route_events: Option<RouteEventSink>,
}

struct ActiveFlows<'a> {
    tcp: &'a mut BTreeMap<DirectTuple, ActiveTcp>,
    udp: &'a mut BTreeMap<DirectTuple, ActiveUdp>,
}

struct ForwardEvidence<'a> {
    started: Instant,
    route_events: Option<&'a RouteEventSink>,
}

fn prune_socket_events(events: &mut Vec<DirectSocketEvent>, cutoff_unix_ns: i64) {
    let old_closes: BTreeMap<_, _> = events
        .iter()
        .filter(|event| {
            event.kind == super::direct_route::SocketEventKind::Close
                && event.at_unix_ns < cutoff_unix_ns
        })
        .map(|event| ((event.tuple, event.endpoint_id), event.at_qpc))
        .collect();
    events.retain(|event| {
        event.at_unix_ns >= cutoff_unix_ns
            || (event.kind == super::direct_route::SocketEventKind::Open
                && old_closes
                    .get(&(event.tuple, event.endpoint_id))
                    .is_none_or(|closed_at| *closed_at < event.at_qpc))
    });
}

#[allow(clippy::too_many_arguments)] // The exact owner fence is part of the packet route context.
fn forward_active_packet(
    tuple: DirectTuple,
    packet: &[u8],
    address: Address,
    flows: ActiveFlows<'_>,
    handle: &Handle,
    stats: &SharedStats,
    evidence: ForwardEvidence<'_>,
    observer: &Arc<Mutex<DirectSocketObserver>>,
) -> io::Result<bool> {
    let route_events = evidence.route_events;
    if tuple.transport == Proto::Udp {
        let Some(flow) = flows.udp.get_mut(&tuple) else {
            return Ok(false);
        };
        forward_udp_packet(flow, packet, stats, route_events)?;
        return Ok(true);
    }
    let Some(flow) = flows.tcp.get_mut(&tuple) else {
        return Ok(false);
    };
    let epoch = flow.epoch;
    forward_tcp_packet(
        flow,
        packet,
        address,
        handle,
        stats,
        evidence,
        true,
        Some((observer, epoch)),
    )?;
    Ok(true)
}

fn forward_udp_packet(
    flow: &mut ActiveUdp,
    packet: &[u8],
    stats: &SharedStats,
    route_events: Option<&RouteEventSink>,
) -> io::Result<()> {
    let tuple = flow.epoch.tuple;
    let datagram = match raw_udp_datagram(packet) {
        Ok(datagram) if datagram.source == tuple.local && datagram.destination == tuple.remote => {
            datagram
        }
        _ => {
            packet_finished(stats, PacketDisposition::Lost)?;
            return Err(io::Error::other(
                "selected UDP packet invalid or tuple changed",
            ));
        }
    };
    if let Err(error) = flow.association().send(datagram.payload) {
        packet_finished(stats, PacketDisposition::Lost)?;
        return Err(io::Error::other(format!("selected UDP relay: {error}")));
    }
    packet_finished(stats, PacketDisposition::Consumed)?;
    {
        let mut proof = flow.proof.lock().expect("direct proof lock");
        proof.client_to_upstream = proof
            .client_to_upstream
            .saturating_add(datagram.payload.len() as u64);
        proof.client_datagrams = proof.client_datagrams.saturating_add(1);
    }
    record_route(
        route_events,
        flow.flow_id,
        RouteEventKind::DatagramTransfer {
            client_to_upstream: datagram.payload.len() as u64,
            upstream_to_client: 0,
            client_datagrams: 1,
            peer_datagrams: 0,
        },
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Keep the owner fence adjacent to synthetic reply injection.
fn forward_tcp_packet(
    flow: &mut ActiveTcp,
    packet: &[u8],
    address: Address,
    handle: &Handle,
    stats: &SharedStats,
    evidence: ForwardEvidence<'_>,
    allow_synthetic_reply: bool,
    reply_owner: Option<(&Arc<Mutex<DirectSocketObserver>>, SocketEpoch)>,
) -> io::Result<()> {
    let ForwardEvidence {
        started,
        route_events,
    } = evidence;
    let replies = match flow
        .bridge
        .as_mut()
        .expect("active TCP bridge")
        .ingest_diverted(
            packet,
            address.ip_checksum_valid(),
            address.tcp_checksum_valid(),
            address.outbound(),
            started.elapsed().as_millis() as i64,
        ) {
        Ok(replies) => replies,
        Err(error) => {
            record_route(
                route_events,
                flow.flow_id,
                RouteEventKind::Terminal {
                    reason: bridge_failure_reason(&error),
                },
            );
            let mut proof = flow.proof.lock().expect("direct proof lock");
            if !proof.selected_packet_consumed {
                proof.selected_packet_lost = true;
                record_route(
                    route_events,
                    flow.flow_id,
                    RouteEventKind::SelectedPacketDisposition {
                        disposition: "lost",
                    },
                );
            }
            drop(proof);
            packet_finished(stats, PacketDisposition::Lost)?;
            return Err(io::Error::other(format!("selected TCP bridge: {error}")));
        }
    };
    packet_finished(stats, PacketDisposition::Consumed)?;
    {
        let mut proof = flow.proof.lock().expect("direct proof lock");
        if !proof.selected_packet_consumed {
            proof.selected_packet_consumed = true;
            record_route(
                route_events,
                flow.flow_id,
                RouteEventKind::SelectedPacketDisposition {
                    disposition: "consumed",
                },
            );
        }
    }
    if raw_transport_head(packet).is_ok_and(|head| head.tcp_ack) {
        let returned_data_acknowledged = {
            let mut proof = flow.proof.lock().expect("direct proof lock");
            proof.client_ack = true;
            if !proof.returned_data_acknowledged
                && proof
                    .first_returned_data_end
                    .is_some_and(|end| tcp_acknowledges_returned_data(packet, end))
            {
                proof.returned_data_acknowledged = true;
                true
            } else {
                false
            }
        };
        record_route(
            route_events,
            flow.flow_id,
            RouteEventKind::ClientResponse { ack: true },
        );
        if returned_data_acknowledged {
            record_route(
                route_events,
                flow.flow_id,
                RouteEventKind::ReturnedDataAcknowledged,
            );
        }
    }
    let transferred = flow
        .bridge
        .as_ref()
        .expect("active TCP bridge")
        .transferred_bytes();
    if transferred != flow.last_transfer {
        {
            let mut proof = flow.proof.lock().expect("direct proof lock");
            proof.client_to_upstream = transferred.0;
            proof.upstream_to_client = transferred.1;
        }
        record_route(
            route_events,
            flow.flow_id,
            RouteEventKind::Transfer {
                client_to_upstream: transferred.0,
                upstream_to_client: transferred.1,
            },
        );
        flow.last_transfer = transferred;
    }
    let reply_owner_still_live = reply_owner.is_none_or(|(observer, epoch)| {
        observer
            .lock()
            .expect("socket observer lock")
            .current_owner_epoch(epoch.tuple)
            == Some(epoch)
    });
    if allow_synthetic_reply && reply_owner_still_live {
        inject_replies(
            handle,
            stats,
            replies,
            flow.address,
            route_events,
            flow.flow_id,
            Some(&flow.proof),
        )?;
    } else if !replies.is_empty() {
        // The original socket generation has closed or another owner now has
        // its tuple. A synthetic reply to the old client could enter that new
        // socket. Keep the omission explicit instead of claiming delivery.
        stats
            .synthetic_reply_lost
            .fetch_add(replies.len() as u64, Ordering::Relaxed);
        record_route(
            route_events,
            flow.flow_id,
            RouteEventKind::SyntheticReplyFailed {
                reason: "retired-owner-changed",
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn abort_retirement_history_full(
    handle: &Handle,
    stats: &SharedStats,
    route_events: Option<&RouteEventSink>,
    context: &NativeObservationContext,
    pending_tcp: &mut BTreeMap<DirectTuple, PendingTcp>,
    retired_helpers: &mut Vec<JoinHandle<()>>,
    active: &BTreeMap<DirectTuple, ActiveTcp>,
    retired_tcp: &BTreeMap<SelectedRoute, RetiredTcp>,
    active_udp: &BTreeMap<DirectTuple, ActiveUdp>,
    retired_udp: &BTreeMap<SelectedRoute, RetiredUdp>,
    additional_flow_id: Option<u64>,
    receipts: &mut DirectReceiptLedger,
    issued_receipts: &mut Vec<RouteReceiptId>,
) -> io::Error {
    stats.event_history_overflow.store(true, Ordering::Release);
    stats.record_bypass("selected-retirement-history-full");
    let stop_error = handle.stop().err();
    for pending in pending_tcp.values() {
        pending.cancelled.store(true, Ordering::Release);
    }
    let mut accounting_errors = 0usize;
    for pending in pending_tcp.values() {
        accounting_errors += usize::from(
            lose_pending_first_packet(stats, route_events, pending.flow_id, &pending.proof)
                .is_err(),
        );
        for _ in &pending.extra {
            accounting_errors +=
                usize::from(packet_finished(stats, PacketDisposition::Lost).is_err());
            record_route(
                route_events,
                pending.flow_id,
                RouteEventKind::PacketOmission {
                    reason: "selected-retirement-history-full",
                    disposition: "lost",
                },
            );
        }
    }
    let mut affected = BTreeSet::new();
    affected.extend(pending_tcp.values().map(|flow| flow.flow_id));
    affected.extend(active.values().map(|flow| flow.flow_id));
    affected.extend(retired_tcp.values().map(|flow| flow.flow.flow_id));
    affected.extend(active_udp.values().map(|flow| flow.flow_id));
    affected.extend(retired_udp.values().map(|flow| flow.flow.flow_id));
    affected.extend(additional_flow_id);
    for flow_id in affected {
        record_route(
            route_events,
            flow_id,
            RouteEventKind::Terminal {
                reason: "selected-retirement-history-full",
            },
        );
    }
    let mut helper_panics = 0usize;
    for pending in pending_tcp.values() {
        context.retire_direct_helper_registration(&pending.flow);
    }
    for (_, pending) in std::mem::take(pending_tcp) {
        helper_panics += usize::from(pending.worker.join().is_err());
    }
    for worker in retired_helpers.drain(..) {
        helper_panics += usize::from(worker.join().is_err());
    }
    let mut receipt_errors = 0usize;
    for receipt in issued_receipts.drain(..) {
        receipt_errors += usize::from(receipts.retire(receipt).is_err());
    }
    io::Error::other(format!(
        "selected route retirement history full; packet accounting errors={accounting_errors}; receipt retirement errors={receipt_errors}; helper panics={helper_panics}; handle stop error={stop_error:?}"
    ))
}

fn process_packets(inputs: ProcessRuntimeInputs) -> io::Result<()> {
    let ProcessRuntimeInputs {
        receiver,
        ready,
        queued_candidates,
        queued_udp_candidates,
        handle,
        observer,
        context,
        selected,
        ack_witnesses,
        excluded,
        refused,
        connecting,
        stats,
        session_id,
        proxy,
        password,
        max_flows,
        route_events,
    } = inputs;
    let started = Instant::now();
    let mut candidates = CandidateBuffer::new(PACKET_QUEUE, CANDIDATE_BYTES, CANDIDATE_WAIT_NS)
        .expect("fixed candidate bounds");
    let mut observed_process_version = 0;
    let mut cached_process_evidence = None::<super::CaptureProcessEvidence>;
    let mut events = Vec::<DirectSocketEvent>::new();
    #[cfg(test)]
    let mut event_published_ns = BTreeMap::<DirectTuple, u64>::new();
    #[cfg(test)]
    let mut process_proof_observed_ns = None::<u64>;
    let mut active = BTreeMap::<DirectTuple, ActiveTcp>::new();
    let mut retired_tcp = BTreeMap::<SelectedRoute, RetiredTcp>::new();
    let mut pending_tcp = BTreeMap::<DirectTuple, PendingTcp>::new();
    let mut retired_helpers = Vec::<JoinHandle<()>>::new();
    let mut active_udp = BTreeMap::<DirectTuple, ActiveUdp>::new();
    let mut retired_udp = BTreeMap::<SelectedRoute, RetiredUdp>::new();
    let mut receipts = DirectReceiptLedger::new(session_id.clone(), max_flows);
    let mut issued_receipts = Vec::<RouteReceiptId>::new();
    let mut helper_id = 0u64;
    let mut last_ack_prune = Instant::now();
    let mut last_event_prune = Instant::now();
    let mut last_retired_prune = Instant::now();
    ready
        .send(())
        .map_err(|_| io::Error::other("direct route startup was abandoned"))?;
    loop {
        stats.processor_progress();
        // A retired bridge remains available for pre-retirement packets for a
        // finite grace. Its exact outstanding queue count prevents retirement
        // while a receive-thread-selected packet is still waiting here.
        let finish_retired = if last_retired_prune.elapsed() >= Duration::from_millis(10) {
            let mut registry = selected.lock().expect("selected route lock");
            let now = Instant::now();
            let ready: Vec<_> = retired_tcp
                .iter()
                .filter_map(|(route, retired)| {
                    (now >= retired.bridge_until && registry.queued(*route) == 0).then_some(*route)
                })
                .collect();
            let ready_udp: Vec<_> = retired_udp
                .iter()
                .filter_map(|(route, retired)| {
                    (now >= retired.bridge_until && registry.queued(*route) == 0).then_some(*route)
                })
                .collect();
            for route in ready.iter().chain(ready_udp.iter()) {
                if let Some(retiring) = registry.retiring.get_mut(route) {
                    retiring.bridge_available = false;
                }
            }
            registry.prune_history(now);
            last_retired_prune = now;
            (ready, ready_udp)
        } else {
            (Vec::new(), Vec::new())
        };
        for route in finish_retired.0 {
            let retired = retired_tcp.remove(&route).expect("selected retired bridge");
            record_route(
                route_events.as_ref(),
                retired.flow.flow_id,
                RouteEventKind::Terminal {
                    reason: retired.reason,
                },
            );
            excluded
                .lock()
                .expect("helper route lock")
                .remove(&retired.flow.helper);
        }
        for route in finish_retired.1 {
            let retired = retired_udp
                .remove(&route)
                .expect("selected retired UDP bridge");
            record_route(
                route_events.as_ref(),
                retired.flow.flow_id,
                RouteEventKind::Terminal {
                    reason: retired.reason,
                },
            );
            let mut helpers = excluded.lock().expect("helper route lock");
            helpers.remove(&retired.flow.relay);
            helpers.remove(&retired.flow.control);
            drop(helpers);
            drop(retired);
        }
        let mut helper_index = 0;
        while helper_index < retired_helpers.len() {
            if retired_helpers[helper_index].is_finished() {
                retired_helpers
                    .swap_remove(helper_index)
                    .join()
                    .map_err(|_| io::Error::other("retired direct helper task panicked"))?;
            } else {
                helper_index += 1;
            }
        }
        let mut candidate_added = false;
        if last_ack_prune.elapsed() >= ACK_CLOSE_GRACE {
            ack_witnesses
                .lock()
                .expect("ACK witness lock")
                .retain(|_, witness| {
                    witness
                        .closed_at
                        .is_none_or(|closed_at| closed_at.elapsed() <= ACK_CLOSE_GRACE)
                });
            last_ack_prune = Instant::now();
        }
        match receive_work(&receiver, &queued_candidates, &queued_udp_candidates) {
            Ok(WorkPacket::Candidate(head, packet, address, received_ns)) => {
                stats.record_candidate_queue_delay(received_ns);
                #[cfg(test)]
                if STALL_NEXT_CANDIDATE.swap(false, Ordering::AcqRel) {
                    thread::sleep(Duration::from_millis(150));
                }
                if refused
                    .lock()
                    .expect("refused route lock")
                    .get(&head.tuple)
                    .copied()
                    .is_some_and(|epoch| {
                        epoch.contains_packet(head.at_qpc)
                            && observer
                                .lock()
                                .expect("socket observer lock")
                                .current_owner_epoch(head.tuple)
                                == Some(epoch)
                    })
                {
                    stats.record_bypass("proxy-refused-flow");
                    reinject(&handle, &stats, &packet, &address)?;
                    continue;
                }
                if let Err(pending) = candidates.enqueue(head, packet, address, received_ns) {
                    stats.record_candidate_terminal_delay(received_ns);
                    stats
                        .candidate_capacity_bypassed
                        .fetch_add(1, Ordering::Relaxed);
                    reinject(&handle, &stats, &pending.packet, &pending.address)?;
                } else {
                    candidate_added = true;
                }
            }
            Ok(WorkPacket::Selected(route, packet, address)) => {
                let _queued = SelectedWorkGuard {
                    selected: Arc::clone(&selected),
                    route,
                };
                let tuple = route.epoch.tuple;
                let current_route = selected
                    .lock()
                    .expect("selected route lock")
                    .current
                    .get(&tuple)
                    .copied();
                let current_owner_state = observer
                    .lock()
                    .expect("socket observer lock")
                    .current_owner_state(tuple);
                let current_owner = match current_owner_state {
                    CurrentOwner::Unique(epoch) => Some(epoch),
                    CurrentOwner::Absent | CurrentOwner::Ambiguous => None,
                };
                // A late SOCKET event can invalidate the receive thread's
                // provisional historical proof. Revalidate before bridge
                // ingress, while the queued count still retains the route.
                let historical_owner = observer
                    .lock()
                    .expect("socket observer lock")
                    .owner_at_qpc(tuple, address.timestamp_qpc);
                let target_epoch = pending_tcp
                    .get(&tuple)
                    .map(|pending| pending.epoch)
                    .or_else(|| active.get(&tuple).map(|flow| flow.epoch))
                    .or_else(|| active_udp.get(&tuple).map(|flow| flow.epoch));
                // The receive thread already proved this exact selected epoch
                // before enqueue. A later SOCKET event may have changed the
                // live owner while the packet waited in the worker queue.
                if historical_owner == CurrentOwner::Unique(route.epoch)
                    && route.epoch.contains_packet(address.timestamp_qpc)
                {
                    if current_route == Some(route) && target_epoch == Some(route.epoch) {
                        if let Some(pending) = pending_tcp.get_mut(&tuple) {
                            hold_pending_packet(
                                pending,
                                packet,
                                address,
                                &stats,
                                route_events.as_ref(),
                            )?;
                            continue;
                        }
                        if let Some(flow) = active.get_mut(&tuple) {
                            if flow.epoch == route.epoch {
                                forward_tcp_packet(
                                    flow,
                                    &packet,
                                    address,
                                    &handle,
                                    &stats,
                                    ForwardEvidence {
                                        started,
                                        route_events: route_events.as_ref(),
                                    },
                                    current_owner == Some(route.epoch),
                                    Some((&observer, route.epoch)),
                                )?;
                                continue;
                            }
                        }
                        if (current_owner == Some(route.epoch) || tuple.transport == Proto::Udp)
                            && forward_active_packet(
                                tuple,
                                &packet,
                                address,
                                ActiveFlows {
                                    tcp: &mut active,
                                    udp: &mut active_udp,
                                },
                                &handle,
                                &stats,
                                ForwardEvidence {
                                    started,
                                    route_events: route_events.as_ref(),
                                },
                                &observer,
                            )?
                        {
                            continue;
                        }
                    }
                    if let Some(retired) = retired_udp.get_mut(&route) {
                        if address.timestamp_qpc < retired.retire_qpc {
                            forward_udp_packet(
                                &mut retired.flow,
                                &packet,
                                &stats,
                                route_events.as_ref(),
                            )?;
                            continue;
                        }
                    }
                    if let Some(retired) = retired_tcp.get_mut(&route) {
                        if address.timestamp_qpc < retired.retire_qpc {
                            forward_tcp_packet(
                                &mut retired.flow,
                                &packet,
                                address,
                                &handle,
                                &stats,
                                ForwardEvidence {
                                    started,
                                    route_events: route_events.as_ref(),
                                },
                                false,
                                Some((&observer, route.epoch)),
                            )?;
                            continue;
                        }
                    }
                }
                let reason = if historical_owner != CurrentOwner::Unique(route.epoch) {
                    Some("selected-packet-history-ambiguous-or-changed")
                } else if !route.epoch.contains_packet(address.timestamp_qpc) {
                    Some("selected-packet-before-open-or-clock-ambiguous")
                } else {
                    selected_work_admission(
                        route,
                        current_route,
                        current_owner,
                        target_epoch,
                        address.timestamp_qpc,
                    )
                    .err()
                };
                if let Some(default_reason) = reason {
                    let reason = selected_loss_reason(
                        default_reason,
                        route,
                        &packet,
                        address.timestamp_qpc,
                        historical_owner,
                        current_owner_state,
                        &observer,
                    );
                    let pass_new_owner = current_owner.is_some_and(|owner| {
                        owner != route.epoch && owner.contains_packet(address.timestamp_qpc)
                    });
                    if tuple.transport == Proto::Udp {
                        let exact_close_qpc = observer
                            .lock()
                            .expect("socket observer lock")
                            .closed_qpc_for_exact_epoch(route.epoch);
                        let bypass_reason = if historical_owner == CurrentOwner::Unique(route.epoch)
                        {
                            "selected-udp-route-unavailable-original-reinjected"
                        } else {
                            udp_unproven_bypass_reason(address.timestamp_qpc, exact_close_qpc)
                        };
                        stats.record_bypass(bypass_reason);
                        let unique_lineage = selected
                            .lock()
                            .expect("selected route lock")
                            .unique_lineage_route(tuple);
                        let flow_id = if historical_owner == CurrentOwner::Unique(route.epoch) {
                            Some(route.flow_id)
                        } else {
                            udp_post_close_lineage_flow(
                                route,
                                unique_lineage,
                                historical_owner,
                                current_owner_state,
                                address.timestamp_qpc,
                                exact_close_qpc,
                            )
                        };
                        reinject_uninspected_udp(
                            &handle,
                            &stats,
                            &packet,
                            &address,
                            route_events.as_ref(),
                            flow_id,
                            bypass_reason,
                        )?;
                        continue;
                    }
                    stats.record_bypass(reason);
                    let reinjected = historical_owner == CurrentOwner::Unique(route.epoch)
                        && stale_selected_can_pass_original(
                            route,
                            current_owner_state,
                            address.timestamp_qpc,
                        );
                    if reinjected {
                        reinject(&handle, &stats, &packet, &address)?;
                    } else {
                        packet_finished(&stats, PacketDisposition::Lost)?;
                    }
                    if !pass_new_owner {
                        record_route(
                            route_events.as_ref(),
                            route.flow_id,
                            RouteEventKind::PacketOmission {
                                reason,
                                disposition: if reinjected { "reinjected" } else { "lost" },
                            },
                        );
                    }
                    continue;
                }
                if let Some(pending) = pending_tcp.get_mut(&tuple) {
                    hold_pending_packet(pending, packet, address, &stats, route_events.as_ref())?;
                    continue;
                }
                if !forward_active_packet(
                    tuple,
                    &packet,
                    address,
                    ActiveFlows {
                        tcp: &mut active,
                        udp: &mut active_udp,
                    },
                    &handle,
                    &stats,
                    ForwardEvidence {
                        started,
                        route_events: route_events.as_ref(),
                    },
                    &observer,
                )? {
                    reinject(&handle, &stats, &packet, &address)?;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let mut helper_panicked = false;
                for decision in candidates.drain() {
                    if let CandidateDecision::PassThrough {
                        reason,
                        packet,
                        address,
                        ..
                    } = decision
                    {
                        stats.record_bypass(reason);
                        reinject(&handle, &stats, &packet, &address)?;
                    }
                }
                for (_, pending) in pending_tcp {
                    pending.cancelled.store(true, Ordering::Release);
                    context.retire_direct_helper_registration(&pending.flow);
                    let PendingTcp {
                        flow,
                        packet: _,
                        extra,
                        flow_id,
                        proof,
                        worker,
                        ..
                    } = pending;
                    helper_panicked |= worker.join().is_err();
                    context.retire_direct_helper_registration(&flow);
                    lose_pending_first_packet(&stats, route_events.as_ref(), flow_id, &proof)?;
                    for _ in extra {
                        record_route(
                            route_events.as_ref(),
                            flow_id,
                            RouteEventKind::PacketOmission {
                                reason: "pending-packet-runtime-stopped",
                                disposition: "lost",
                            },
                        );
                        packet_finished(&stats, PacketDisposition::Lost)?;
                    }
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::Terminal {
                            reason: "runtime-stopped-pending-helper",
                        },
                    );
                }
                for worker in retired_helpers {
                    helper_panicked |= worker.join().is_err();
                }
                connecting.store(false, Ordering::Release);
                for receipt in issued_receipts {
                    receipts.retire(receipt).map_err(|error| {
                        io::Error::other(format!("retire route receipt: {error:?}"))
                    })?;
                }
                for flow in active.values() {
                    record_route(
                        route_events.as_ref(),
                        flow.flow_id,
                        RouteEventKind::Terminal {
                            reason: "runtime-stopped",
                        },
                    );
                }
                for retired in retired_tcp.values() {
                    record_route(
                        route_events.as_ref(),
                        retired.flow.flow_id,
                        RouteEventKind::Terminal {
                            reason: retired.reason,
                        },
                    );
                }
                for flow in active_udp.values() {
                    record_route(
                        route_events.as_ref(),
                        flow.flow_id,
                        RouteEventKind::Terminal {
                            reason: "runtime-stopped",
                        },
                    );
                }
                for retired in retired_udp.values() {
                    record_route(
                        route_events.as_ref(),
                        retired.flow.flow_id,
                        RouteEventKind::Terminal {
                            reason: retired.reason,
                        },
                    );
                }
                drop(active_udp);
                drop(retired_udp);
                return if helper_panicked {
                    Err(io::Error::other(
                        "direct helper task panicked during shutdown",
                    ))
                } else {
                    Ok(())
                };
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let new_events = observer.lock().expect("socket observer lock").take_events();
        #[cfg(test)]
        for event in &new_events {
            event_published_ns.insert(event.tuple, stats.started.elapsed().as_nanos() as u64);
        }
        // Keep exact helper socket events for prompt pass-through. The
        // candidate decision rejects this PID before process binding, so a
        // stale snapshot cannot promote proxy traffic to target ownership.
        for event in &new_events {
            if event.kind == super::direct_route::SocketEventKind::Close
                && refused
                    .lock()
                    .expect("refused route lock")
                    .get(&event.tuple)
                    .copied()
                    .is_some_and(|epoch| epoch.matches_event(*event))
            {
                refused
                    .lock()
                    .expect("refused route lock")
                    .remove(&event.tuple);
            }
            let selected_route = selected
                .lock()
                .expect("selected route lock")
                .current
                .get(&event.tuple)
                .copied();
            let Some(route) = selected_route else {
                continue;
            };
            let retired_reason = match event.kind {
                super::direct_route::SocketEventKind::Open
                    if event.at_qpc > route.epoch.open_qpc
                        && SocketEpoch::from_open(*event) != route.epoch =>
                {
                    Some("competing-socket-open")
                }
                super::direct_route::SocketEventKind::Close
                    if route.epoch.matches_event(*event) =>
                {
                    Some("socket-closed")
                }
                _ => None,
            };
            let Some(reason) = retired_reason else {
                continue;
            };
            {
                let mut witnesses = ack_witnesses.lock().expect("ACK witness lock");
                if let Some(witness) = witnesses.get_mut(&event.tuple) {
                    if witness.epoch == route.epoch {
                        if event.kind == super::direct_route::SocketEventKind::Close {
                            witness.closed_at = Some(Instant::now());
                            witness.close_qpc = Some(event.at_qpc);
                        } else {
                            // An overlapping OPEN makes return ACK ownership
                            // ambiguous until the competing socket is resolved.
                            witnesses.remove(&event.tuple);
                        }
                    }
                }
            }
            let has_bridge = active
                .get(&event.tuple)
                .is_some_and(|flow| flow.epoch == route.epoch)
                || active_udp
                    .get(&event.tuple)
                    .is_some_and(|flow| flow.epoch == route.epoch);
            if !selected
                .lock()
                .expect("selected route lock")
                .record_retirement(route, event.at_qpc, has_bridge)
            {
                return Err(abort_retirement_history_full(
                    &handle,
                    &stats,
                    route_events.as_ref(),
                    &context,
                    &mut pending_tcp,
                    &mut retired_helpers,
                    &active,
                    &retired_tcp,
                    &active_udp,
                    &retired_udp,
                    None,
                    &mut receipts,
                    &mut issued_receipts,
                ));
            }
            let retired = if let Some(pending) = pending_tcp.remove(&event.tuple) {
                debug_assert_eq!(pending.epoch, route.epoch);
                pending.cancelled.store(true, Ordering::Release);
                context.retire_direct_helper_registration(&pending.flow);
                connecting.store(!pending_tcp.is_empty(), Ordering::Release);
                Some(retire_pending_tcp(
                    pending,
                    &stats,
                    route_events.as_ref(),
                    reason,
                    &mut retired_helpers,
                )?)
            } else if let Some(flow) = active.remove(&event.tuple) {
                debug_assert_eq!(flow.epoch, route.epoch);
                debug_assert!(retired_tcp.len() < max_flows);
                context.retire_direct_helper_registration(&flow.flow);
                // The consumed receipt remains in the ledger as a tuple
                // reservation. Release that reservation at exact retirement,
                // while the bridge/proof remain briefly available for older
                // queued packets. Otherwise a new generation cannot issue its
                // own receipt until the old bridge grace elapses.
                receipts.retire(flow.receipt).map_err(|error| {
                    io::Error::other(format!("retire closed route receipt: {error:?}"))
                })?;
                issued_receipts.retain(|issued| *issued != flow.receipt);
                retired_tcp.insert(
                    route,
                    RetiredTcp {
                        flow,
                        retire_qpc: event.at_qpc,
                        bridge_until: Instant::now() + SELECTED_RETIRE_GRACE,
                        reason,
                    },
                );
                None
            } else if let Some(flow) = active_udp.remove(&event.tuple) {
                debug_assert_eq!(flow.epoch, route.epoch);
                debug_assert!(retired_udp.len() < max_flows);
                context.retire_direct_helper_registration(&flow.flow);
                retired_udp.insert(
                    route,
                    RetiredUdp {
                        flow,
                        retire_qpc: event.at_qpc,
                        bridge_until: Instant::now() + SELECTED_RETIRE_GRACE,
                        reason,
                    },
                );
                Some(
                    retired_udp
                        .get(&route)
                        .expect("retired UDP route")
                        .flow
                        .receipt,
                )
            } else {
                None
            };
            if let Some(receipt) = retired {
                receipts.retire(receipt).map_err(|error| {
                    io::Error::other(format!("retire closed route receipt: {error:?}"))
                })?;
                issued_receipts.retain(|issued| *issued != receipt);
            }
        }
        let socket_evidence_changed = !new_events.is_empty();
        events.extend(new_events);
        // Pruning scans the entire bounded history. It is not part of the
        // packet fast path: a burst of unrelated packets must not repeatedly
        // walk unchanged socket evidence while selected traffic waits behind it.
        if last_event_prune.elapsed() >= Duration::from_millis(10) || events.len() > SOCKET_EVENTS {
            let cutoff_unix_ns = i64::try_from(unix_now_ns()?)
                .map_err(|_| io::Error::other("system time outside route event history"))?
                .saturating_sub(EVENT_HISTORY_NS);
            prune_socket_events(&mut events, cutoff_unix_ns);
            last_event_prune = Instant::now();
        }
        if events.len() > SOCKET_EVENTS {
            stats.event_history_overflow.store(true, Ordering::Release);
            let _ = handle.stop();
            return Err(io::Error::other(
                "direct socket event history exceeded finite bound",
            ));
        }
        let event_loss = observer.lock().expect("socket observer lock").stats();
        let process_version = context.direct_process_evidence_version();
        let process_evidence_changed =
            candidates.len() != 0 && process_version != observed_process_version;
        if process_evidence_changed {
            observed_process_version = process_version;
            cached_process_evidence = context.direct_process_evidence();
        }
        let evidence = if event_loss.capacity_lost == 0
            && event_loss.clock_lost == 0
            && !stats.event_history_overflow.load(Ordering::Acquire)
            && candidates.len() != 0
        {
            cached_process_evidence.as_ref()
        } else {
            None
        };
        #[cfg(test)]
        if evidence.is_some() && process_proof_observed_ns.is_none() {
            process_proof_observed_ns = Some(stats.started.elapsed().as_nanos() as u64);
        }
        let now_ns = stats.started.elapsed().as_nanos() as u64;
        let decisions = if socket_evidence_changed || process_evidence_changed {
            candidates.resolve(evidence, &events, now_ns, std::process::id())
        } else if candidate_added {
            let mut decisions =
                candidates.resolve_newest(evidence, &events, now_ns, std::process::id());
            decisions.extend(candidates.expire_due(now_ns));
            decisions
        } else {
            candidates.expire_due(now_ns)
        };
        for decision in decisions {
            match decision {
                CandidateDecision::PassThrough {
                    reason,
                    packet,
                    address,
                    received_ns,
                } => {
                    stats.record_candidate_terminal_delay(received_ns);
                    #[cfg(test)]
                    if session_id == "s200-managed-chain" {
                        let publication = raw_transport_head(&packet).ok().and_then(|head| {
                            event_published_ns
                                .get(&DirectTuple {
                                    transport: head.transport,
                                    local: head.source,
                                    remote: head.destination,
                                })
                                .copied()
                        });
                        candidate_timeline(
                            received_ns,
                            publication,
                            process_proof_observed_ns,
                            stats.started.elapsed().as_nanos() as u64,
                            reason,
                        );
                    }
                    #[cfg(not(test))]
                    let _ = received_ns;
                    #[cfg(test)]
                    if session_id == "s200-managed-chain" && reason == "candidate-wait-expired" {
                        match raw_transport_head(&packet) {
                            Ok(head) if head.transport == Proto::Udp => {
                                let tuple = DirectTuple {
                                    transport: head.transport,
                                    local: head.source,
                                    remote: head.destination,
                                };
                                let matching: Vec<_> =
                                    events.iter().filter(|event| event.tuple == tuple).collect();
                                eprintln!(
                                    "managed-chain UDP candidate expired: tuple={tuple:?}, current_owner={:?}, matching_events={matching:?}, process_evidence_present={}",
                                    observer.lock().expect("socket observer lock").current_owner_endpoint_id(tuple),
                                    context.direct_process_evidence().is_some()
                                );
                            }
                            _ => {}
                        }
                    }
                    stats.record_bypass(reason);
                    reinject(&handle, &stats, &packet, &address)?;
                }
                CandidateDecision::Selected {
                    flow,
                    epoch,
                    packet,
                    address,
                    received_ns,
                } => {
                    let mut flow = *flow;
                    stats.record_candidate_terminal_delay(received_ns);
                    if !epoch.contains_packet(address.timestamp_qpc)
                        || observer
                            .lock()
                            .expect("socket observer lock")
                            .current_owner_epoch(flow.tuple)
                            != Some(epoch)
                    {
                        stats.record_bypass("candidate-owner-epoch-changed-or-ambiguous");
                        packet_finished(&stats, PacketDisposition::Lost)?;
                        continue;
                    }
                    // Exact socket and managed process ownership have already
                    // selected this target packet. Only now read its complete
                    // datagram for the passive original-packet join.
                    flow.first_packet_digest = Some(*blake3::hash(&packet).as_bytes());
                    let selected_digests = fragcap_core::parse::direct_datagram_digests(&packet);
                    #[cfg(test)]
                    if session_id == "s200-managed-chain" {
                        candidate_timeline(
                            received_ns,
                            event_published_ns.get(&flow.tuple).copied(),
                            process_proof_observed_ns,
                            stats.started.elapsed().as_nanos() as u64,
                            "selected",
                        );
                    }
                    #[cfg(not(test))]
                    let _ = received_ns;
                    if refused
                        .lock()
                        .expect("refused route lock")
                        .get(&flow.tuple)
                        .copied()
                        == Some(epoch)
                    {
                        stats.record_bypass("proxy-refused-flow");
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
                    if let Some(pending) = pending_tcp.get_mut(&flow.tuple) {
                        if pending.epoch == epoch {
                            hold_pending_packet(
                                pending,
                                packet,
                                address,
                                &stats,
                                route_events.as_ref(),
                            )?;
                            continue;
                        }
                    }
                    if let Some(stale) = pending_tcp.remove(&flow.tuple) {
                        stale.cancelled.store(true, Ordering::Release);
                        context.retire_direct_helper_registration(&stale.flow);
                        let stale_route = SelectedRoute {
                            epoch: stale.epoch,
                            flow_id: stale.flow_id,
                        };
                        if selected
                            .lock()
                            .expect("selected route lock")
                            .current
                            .get(&flow.tuple)
                            .copied()
                            == Some(stale_route)
                        {
                            selected
                                .lock()
                                .expect("selected route lock")
                                .current
                                .remove(&flow.tuple);
                        }
                        let retired_receipt = retire_pending_tcp(
                            stale,
                            &stats,
                            route_events.as_ref(),
                            "owner-epoch-replaced-before-close-delivery",
                            &mut retired_helpers,
                        )?;
                        receipts.retire(retired_receipt).map_err(|error| {
                            io::Error::other(format!("retire replaced helper receipt: {error:?}"))
                        })?;
                        issued_receipts.retain(|issued| *issued != retired_receipt);
                        connecting.store(!pending_tcp.is_empty(), Ordering::Release);
                    }
                    let active_epoch = if flow.tuple.transport == Proto::Udp {
                        active_udp.get(&flow.tuple).map(|existing| existing.epoch)
                    } else {
                        active.get(&flow.tuple).map(|existing| existing.epoch)
                    };
                    if let Some(existing_epoch) = active_epoch {
                        if existing_epoch == epoch {
                            let forwarded = forward_active_packet(
                                flow.tuple,
                                &packet,
                                address,
                                ActiveFlows {
                                    tcp: &mut active,
                                    udp: &mut active_udp,
                                },
                                &handle,
                                &stats,
                                ForwardEvidence {
                                    started,
                                    route_events: route_events.as_ref(),
                                },
                                &observer,
                            )?;
                            if !forwarded {
                                reinject(&handle, &stats, &packet, &address)?;
                            }
                            continue;
                        }
                        if let Some(stale) = active.remove(&flow.tuple) {
                            debug_assert_eq!(stale.epoch, existing_epoch);
                            let route = SelectedRoute {
                                epoch: stale.epoch,
                                flow_id: stale.flow_id,
                            };
                            if !selected
                                .lock()
                                .expect("selected route lock")
                                .record_retirement(route, epoch.open_qpc, true)
                            {
                                // This candidate belongs to the new epoch and
                                // has not received a route ID yet. Settle its
                                // packet exactly here; do not attach it to the
                                // retired route's per-flow evidence.
                                packet_finished(&stats, PacketDisposition::Lost)?;
                                stats.record_bypass(
                                    "selected-retirement-history-full-current-candidate",
                                );
                                return Err(abort_retirement_history_full(
                                    &handle,
                                    &stats,
                                    route_events.as_ref(),
                                    &context,
                                    &mut pending_tcp,
                                    &mut retired_helpers,
                                    &active,
                                    &retired_tcp,
                                    &active_udp,
                                    &retired_udp,
                                    Some(stale.flow_id),
                                    &mut receipts,
                                    &mut issued_receipts,
                                ));
                            }
                            context.retire_direct_helper_registration(&stale.flow);
                            if let Some(witness) = ack_witnesses
                                .lock()
                                .expect("ACK witness lock")
                                .get_mut(&flow.tuple)
                            {
                                if witness.epoch == stale.epoch {
                                    witness.closed_at = Some(Instant::now());
                                    witness.close_qpc = Some(epoch.open_qpc);
                                }
                            }
                            receipts.retire(stale.receipt).map_err(|error| {
                                io::Error::other(format!("retire replaced TCP receipt: {error:?}"))
                            })?;
                            issued_receipts.retain(|issued| *issued != stale.receipt);
                            retired_tcp.insert(
                                route,
                                RetiredTcp {
                                    flow: stale,
                                    retire_qpc: epoch.open_qpc,
                                    bridge_until: Instant::now() + SELECTED_RETIRE_GRACE,
                                    reason: "owner-epoch-replaced-before-close-delivery",
                                },
                            );
                        }
                        if let Some(stale) = active_udp.remove(&flow.tuple) {
                            debug_assert_eq!(stale.epoch, existing_epoch);
                            let route = SelectedRoute {
                                epoch: stale.epoch,
                                flow_id: stale.flow_id,
                            };
                            if !selected
                                .lock()
                                .expect("selected route lock")
                                .record_retirement(route, epoch.open_qpc, true)
                            {
                                reinject(&handle, &stats, &packet, &address)?;
                                return Err(abort_retirement_history_full(
                                    &handle,
                                    &stats,
                                    route_events.as_ref(),
                                    &context,
                                    &mut pending_tcp,
                                    &mut retired_helpers,
                                    &active,
                                    &retired_tcp,
                                    &active_udp,
                                    &retired_udp,
                                    Some(stale.flow_id),
                                    &mut receipts,
                                    &mut issued_receipts,
                                ));
                            }
                            context.retire_direct_helper_registration(&stale.flow);
                            ack_witnesses
                                .lock()
                                .expect("ACK witness lock")
                                .remove(&flow.tuple);
                            receipts.retire(stale.receipt).map_err(|error| {
                                io::Error::other(format!("retire replaced UDP receipt: {error:?}"))
                            })?;
                            issued_receipts.retain(|issued| *issued != stale.receipt);
                            retired_udp.insert(
                                route,
                                RetiredUdp {
                                    flow: stale,
                                    retire_qpc: epoch.open_qpc,
                                    bridge_until: Instant::now() + SELECTED_RETIRE_GRACE,
                                    reason: "owner-epoch-replaced-before-close-delivery",
                                },
                            );
                        }
                        if selected
                            .lock()
                            .expect("selected route lock")
                            .current
                            .get(&flow.tuple)
                            .copied()
                            .is_some_and(|route| route.epoch == existing_epoch)
                        {
                            selected
                                .lock()
                                .expect("selected route lock")
                                .current
                                .remove(&flow.tuple);
                        }
                    }
                    if active
                        .len()
                        .saturating_add(retired_tcp.len())
                        .saturating_add(active_udp.len())
                        .saturating_add(retired_udp.len())
                        .saturating_add(pending_tcp.len())
                        >= max_flows
                        || (flow.tuple.transport == Proto::Tcp
                            && pending_tcp.len().saturating_add(retired_helpers.len())
                                >= MAX_PENDING_HELPERS)
                    {
                        stats.record_bypass("direct-flow-capacity");
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
                    if flow.tuple.transport == Proto::Udp {
                        let datagram = match raw_udp_datagram(&packet) {
                            Ok(datagram)
                                if datagram.source == flow.tuple.local
                                    && datagram.destination == flow.tuple.remote =>
                            {
                                datagram
                            }
                            _ => {
                                reinject(&handle, &stats, &packet, &address)?;
                                continue;
                            }
                        };
                        let helper_open_earliest = match unix_now_ns() {
                            Ok(value) => value,
                            Err(error) => {
                                stats.record_bypass("direct-clock-unavailable");
                                reinject(&handle, &stats, &packet, &address)?;
                                return Err(error);
                            }
                        };
                        helper_id = helper_id.saturating_add(1);
                        let proof = match context.direct_proof_for(&flow) {
                            Ok(value) => value,
                            Err(reason) => {
                                stats.record_bypass(reason);
                                reinject(&handle, &stats, &packet, &address)?;
                                return Err(io::Error::other(reason));
                            }
                        };
                        proof
                            .lock()
                            .expect("direct proof lock")
                            .first_packet_digests = selected_digests;
                        let (if_idx, sub_if_idx) = address.network_interface();
                        record_route(
                            route_events.as_ref(),
                            helper_id,
                            RouteEventKind::Selected {
                                protocol: "udp",
                                local: flow.tuple.local,
                                remote: flow.tuple.remote,
                                pid: flow.pid,
                                process_instance_id: flow.process_instance_id.clone(),
                                endpoint_id: flow.endpoint_id,
                                first_packet_at_ns: flow.first_packet_at_ns,
                                first_packet_clock_error_ns: flow.first_packet_clock_error_ns,
                                tcp_flags: None,
                                packet_flags: address.flags,
                                ip_checksum_valid: address.ip_checksum_valid(),
                                if_idx,
                                sub_if_idx,
                                tcp_checksum_valid: false,
                            },
                        );
                        let response_address = match address.response_address(flow.tuple.remote) {
                            Ok(value) => value,
                            Err(_) => {
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::Terminal {
                                        reason: "reply-interface-unavailable",
                                    },
                                );
                                stats.record_bypass("reply-interface-unavailable");
                                let marked = mark_refused_flow(&refused, epoch);
                                reinject_selected_first_packet(
                                    &handle,
                                    &stats,
                                    &packet,
                                    &address,
                                    &proof,
                                    route_events.as_ref(),
                                    helper_id,
                                )?;
                                if !marked {
                                    return Err(io::Error::other("refused route ledger full"));
                                }
                                continue;
                            }
                        };
                        record_route(
                            route_events.as_ref(),
                            helper_id,
                            RouteEventKind::ProxyAttempt,
                        );
                        let now = started.elapsed().as_nanos() as u64;
                        let receipt = match receipts
                            .issue(flow.clone(), now.saturating_add(10_000_000_000))
                        {
                            Ok(receipt) => receipt,
                            Err(error) => {
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::Terminal {
                                        reason: "udp-receipt-issue-failed",
                                    },
                                );
                                reinject_selected_first_packet(
                                    &handle,
                                    &stats,
                                    &packet,
                                    &address,
                                    &proof,
                                    route_events.as_ref(),
                                    helper_id,
                                )?;
                                return Err(io::Error::other(format!(
                                    "issue UDP receipt: {error:?}"
                                )));
                            }
                        };
                        let pending_registration =
                            PendingUdpHelperRegistration::new(context.clone(), flow.clone());
                        connecting.store(true, Ordering::Release);
                        let association_started = Instant::now();
                        let association = DirectUdpAssociation::new_with_hook(
                            proxy,
                            flow.tuple.remote,
                            password.as_bytes(),
                            UDP_PROXY_CONNECT_TIMEOUT,
                            |stream| {
                                receipts
                                    .consume(receipt, &session_id, flow.tuple, helper_id, now)
                                    .map_err(|error| {
                                        io::Error::other(format!("consume UDP receipt: {error:?}"))
                                    })?;
                                proof.lock().expect("direct proof lock").receipt_consumed = true;
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::ReceiptConsumed,
                                );
                                context
                                    .register_direct_helper(
                                        "tcp",
                                        stream.local_addr()?,
                                        stream.peer_addr()?,
                                        flow.clone(),
                                        helper_open_earliest,
                                        unix_now_ns()?,
                                    )
                                    .map_err(io::Error::other)
                            },
                        );
                        connecting.store(false, Ordering::Release);
                        let association = match association {
                            Ok(association) => association,
                            Err(error) => {
                                drop(pending_registration);
                                let (outcome, terminal) =
                                    udp_association_failure_outcome(error.kind());
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::HelperOutcome {
                                        stage: "udp-association",
                                        elapsed_ms: association_started.elapsed().as_millis()
                                            as u64,
                                        outcome,
                                        error_kind: Some(format!("{:?}", error.kind())),
                                    },
                                );
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::Terminal { reason: terminal },
                                );
                                let retired = receipts.retire(receipt).map_err(|error| {
                                    io::Error::other(format!(
                                        "retire refused UDP receipt: {error:?}"
                                    ))
                                });
                                stats.record_bypass("udp-association-unavailable");
                                let marked = mark_refused_flow(&refused, epoch);
                                let settled = reinject_selected_first_packet(
                                    &handle,
                                    &stats,
                                    &packet,
                                    &address,
                                    &proof,
                                    route_events.as_ref(),
                                    helper_id,
                                );
                                settled?;
                                retired?;
                                if !marked {
                                    return Err(io::Error::other("refused route ledger full"));
                                }
                                continue;
                            }
                        };
                        let setup = (|| -> Result<_, (&'static str, io::Error)> {
                            let relay = association
                                .relay_tuple()
                                .map_err(|error| ("udp-relay-tuple-unavailable", error))?;
                            let control = association
                                .control_tuple()
                                .map_err(|error| ("udp-control-tuple-unavailable", error))?;
                            proof.lock().expect("direct proof lock").proxy_connected = true;
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::ProxyConnected {
                                    helper_local: control.local,
                                    proxy_local: control.remote,
                                },
                            );
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::UdpRelayAssociated {
                                    relay_local: relay.local,
                                    relay_remote: relay.remote,
                                },
                            );
                            let registered_at = unix_now_ns()
                                .map_err(|error| ("udp-helper-clock-unavailable", error))?;
                            context
                                .register_direct_helper(
                                    "udp",
                                    relay.local,
                                    relay.remote,
                                    flow.clone(),
                                    registered_at,
                                    registered_at.saturating_add(10_000_000_000),
                                )
                                .map_err(|reason| {
                                    ("udp-helper-registration-failed", io::Error::other(reason))
                                })?;
                            Ok((relay, control))
                        })();
                        let (relay, control) = match setup {
                            Ok(value) => value,
                            Err((reason, error)) => {
                                drop(association);
                                drop(pending_registration);
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::Terminal { reason },
                                );
                                let retired = receipts.retire(receipt).map_err(|failure| {
                                    io::Error::other(format!(
                                        "retire failed UDP setup receipt: {failure:?}"
                                    ))
                                });
                                let settled = reinject_selected_first_packet(
                                    &handle,
                                    &stats,
                                    &packet,
                                    &address,
                                    &proof,
                                    route_events.as_ref(),
                                    helper_id,
                                );
                                settled?;
                                retired?;
                                return Err(error);
                            }
                        };
                        {
                            let mut helpers = excluded.lock().expect("helper route lock");
                            helpers.insert(relay);
                            helpers.insert(control);
                        }
                        issued_receipts.push(receipt);
                        if let Err(error) = association.send(datagram.payload) {
                            drop(association);
                            drop(pending_registration);
                            let mut helpers = excluded.lock().expect("helper route lock");
                            helpers.remove(&relay);
                            helpers.remove(&control);
                            drop(helpers);
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::Terminal {
                                    reason: "first-datagram-send-failed",
                                },
                            );
                            proof
                                .lock()
                                .expect("direct proof lock")
                                .selected_packet_lost = true;
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::SelectedPacketDisposition {
                                    disposition: "lost",
                                },
                            );
                            packet_finished(&stats, PacketDisposition::Lost)?;
                            return Err(io::Error::other(format!(
                                "first selected UDP packet: {error}"
                            )));
                        }
                        if let Err(error) = packet_finished(&stats, PacketDisposition::Consumed) {
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::Terminal {
                                    reason: "udp-packet-accounting-failed",
                                },
                            );
                            drop(association);
                            drop(pending_registration);
                            let mut helpers = excluded.lock().expect("helper route lock");
                            helpers.remove(&relay);
                            helpers.remove(&control);
                            return Err(error);
                        }
                        proof
                            .lock()
                            .expect("direct proof lock")
                            .selected_packet_consumed = true;
                        record_route(
                            route_events.as_ref(),
                            helper_id,
                            RouteEventKind::SelectedPacketDisposition {
                                disposition: "consumed",
                            },
                        );
                        {
                            let mut proof = proof.lock().expect("direct proof lock");
                            proof.client_to_upstream = datagram.payload.len() as u64;
                            proof.client_datagrams = 1;
                        }
                        record_route(
                            route_events.as_ref(),
                            helper_id,
                            RouteEventKind::DatagramTransfer {
                                client_to_upstream: datagram.payload.len() as u64,
                                upstream_to_client: 0,
                                client_datagrams: 1,
                                peer_datagrams: 0,
                            },
                        );
                        selected
                            .lock()
                            .expect("selected route lock")
                            .current
                            .insert(
                                flow.tuple,
                                SelectedRoute {
                                    epoch,
                                    flow_id: helper_id,
                                },
                            );
                        active_udp.insert(
                            flow.tuple,
                            ActiveUdp {
                                association: Some(association),
                                address: response_address,
                                flow: flow.clone(),
                                context: context.clone(),
                                excluded: Arc::clone(&excluded),
                                epoch,
                                receipt,
                                relay,
                                control,
                                flow_id: helper_id,
                                proof,
                            },
                        );
                        pending_registration.adopt();
                        continue;
                    }
                    if flow.tuple.transport != Proto::Tcp {
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
                    let helper_open_earliest = match unix_now_ns() {
                        Ok(value) => value,
                        Err(error) => {
                            stats.record_bypass("direct-clock-unavailable");
                            reinject(&handle, &stats, &packet, &address)?;
                            return Err(error);
                        }
                    };
                    helper_id = helper_id.saturating_add(1);
                    let proof = match context.direct_proof_for(&flow) {
                        Ok(value) => value,
                        Err(reason) => {
                            stats.record_bypass(reason);
                            reinject(&handle, &stats, &packet, &address)?;
                            return Err(io::Error::other(reason));
                        }
                    };
                    proof
                        .lock()
                        .expect("direct proof lock")
                        .first_packet_digests = selected_digests;
                    let (if_idx, sub_if_idx) = address.network_interface();
                    record_route(
                        route_events.as_ref(),
                        helper_id,
                        RouteEventKind::Selected {
                            protocol: "tcp",
                            local: flow.tuple.local,
                            remote: flow.tuple.remote,
                            pid: flow.pid,
                            process_instance_id: flow.process_instance_id.clone(),
                            endpoint_id: flow.endpoint_id,
                            first_packet_at_ns: flow.first_packet_at_ns,
                            first_packet_clock_error_ns: flow.first_packet_clock_error_ns,
                            tcp_flags: raw_transport_head(&packet)
                                .ok()
                                .and_then(|head| head.tcp_flags),
                            packet_flags: address.flags,
                            ip_checksum_valid: address.ip_checksum_valid(),
                            if_idx,
                            sub_if_idx,
                            tcp_checksum_valid: address.tcp_checksum_valid(),
                        },
                    );
                    let response_address = match address.response_address(flow.tuple.remote) {
                        Ok(value) => value,
                        Err(_) => {
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::Terminal {
                                    reason: "reply-interface-unavailable",
                                },
                            );
                            stats.record_bypass("reply-interface-unavailable");
                            let marked = mark_refused_flow(&refused, epoch);
                            reinject_selected_first_packet(
                                &handle,
                                &stats,
                                &packet,
                                &address,
                                &proof,
                                route_events.as_ref(),
                                helper_id,
                            )?;
                            if !marked {
                                return Err(io::Error::other("refused route ledger full"));
                            }
                            continue;
                        }
                    };
                    record_route(
                        route_events.as_ref(),
                        helper_id,
                        RouteEventKind::ProxyAttempt,
                    );
                    let now = started.elapsed().as_nanos() as u64;
                    let receipt =
                        match receipts.issue(flow.clone(), now.saturating_add(10_000_000_000)) {
                            Ok(receipt) => receipt,
                            Err(error) => {
                                record_route(
                                    route_events.as_ref(),
                                    helper_id,
                                    RouteEventKind::Terminal {
                                        reason: "tcp-receipt-issue-failed",
                                    },
                                );
                                reinject_selected_first_packet(
                                    &handle,
                                    &stats,
                                    &packet,
                                    &address,
                                    &proof,
                                    route_events.as_ref(),
                                    helper_id,
                                )?;
                                return Err(io::Error::other(format!(
                                    "issue route receipt: {error:?}"
                                )));
                            }
                        };
                    let (result_sender, result_receiver) = mpsc::sync_channel(1);
                    let task_context = context.clone();
                    let task_flow = flow.clone();
                    let task_password = password.clone();
                    let cancelled = Arc::new(AtomicBool::new(false));
                    let worker_cancelled = Arc::clone(&cancelled);
                    let worker = match thread::Builder::new()
                        .name("fragcap-direct-helper".into())
                        .spawn(move || {
                            let began = Instant::now();
                            let outcome = connect_authenticated_socks5_staged(
                                proxy,
                                task_flow.tuple.remote,
                                task_password.as_bytes(),
                                TCP_PROXY_CONNECT_TIMEOUT,
                                |stream| {
                                    if worker_cancelled.load(Ordering::Acquire) {
                                        return Err(io::Error::new(
                                            io::ErrorKind::Interrupted,
                                            "socket owner retired before helper registration",
                                        ));
                                    }
                                    let registered = task_context
                                        .register_direct_helper(
                                            "tcp",
                                            stream.local_addr()?,
                                            stream.peer_addr()?,
                                            task_flow.clone(),
                                            helper_open_earliest,
                                            unix_now_ns()?,
                                        )
                                        .map_err(io::Error::other);
                                    if worker_cancelled.load(Ordering::Acquire) {
                                        task_context.retire_direct_helper_registration(&task_flow);
                                        Err(io::Error::new(
                                            io::ErrorKind::Interrupted,
                                            "socket owner retired during helper registration",
                                        ))
                                    } else {
                                        registered
                                    }
                                },
                            )
                            .map(|stream| (stream, began.elapsed().as_millis() as u64));
                            let _ = result_sender.send(outcome);
                        }) {
                        Ok(worker) => worker,
                        Err(error) => {
                            record_route(
                                route_events.as_ref(),
                                helper_id,
                                RouteEventKind::Terminal {
                                    reason: "tcp-helper-spawn-failed",
                                },
                            );
                            let retired = receipts.retire(receipt).map_err(|receipt_error| {
                                io::Error::other(format!(
                                    "retire unstarted helper receipt: {receipt_error:?}"
                                ))
                            });
                            let settled = reinject_selected_first_packet(
                                &handle,
                                &stats,
                                &packet,
                                &address,
                                &proof,
                                route_events.as_ref(),
                                helper_id,
                            );
                            settled?;
                            retired?;
                            return Err(error);
                        }
                    };
                    issued_receipts.push(receipt);
                    selected
                        .lock()
                        .expect("selected route lock")
                        .current
                        .insert(
                            flow.tuple,
                            SelectedRoute {
                                epoch,
                                flow_id: helper_id,
                            },
                        );
                    pending_tcp.insert(
                        flow.tuple,
                        PendingTcp {
                            flow,
                            epoch,
                            packet,
                            address,
                            response_address,
                            receipt,
                            flow_id: helper_id,
                            proof,
                            result: result_receiver,
                            worker,
                            cancelled,
                            extra: Vec::new(),
                        },
                    );
                    connecting.store(true, Ordering::Release);
                }
            }
        }
        let ready: Vec<_> = pending_tcp
            .iter()
            .filter_map(|(tuple, pending)| match pending.result.try_recv() {
                Ok(outcome) => Some((*tuple, outcome)),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => Some((
                    *tuple,
                    Err(HelperFailure {
                        stage: "complete",
                        elapsed_ms: 0,
                        error: io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "helper task ended without outcome",
                        ),
                    }),
                )),
            })
            .collect();
        for (tuple, outcome) in ready {
            let pending = pending_tcp.remove(&tuple).expect("ready helper exists");
            connecting.store(!pending_tcp.is_empty(), Ordering::Release);
            let outcome = if pending.worker.join().is_err() {
                Err(HelperFailure {
                    stage: "complete",
                    elapsed_ms: 0,
                    error: io::Error::new(io::ErrorKind::BrokenPipe, "direct helper task panicked"),
                })
            } else {
                outcome
            };
            let PendingTcp {
                flow,
                epoch,
                packet,
                address,
                response_address,
                receipt,
                flow_id,
                proof,
                extra,
                ..
            } = pending;
            if observer
                .lock()
                .expect("socket observer lock")
                .current_owner_epoch(tuple)
                != Some(epoch)
            {
                match &outcome {
                    Ok((_, elapsed_ms)) => record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::HelperOutcome {
                            stage: "complete",
                            elapsed_ms: *elapsed_ms,
                            outcome: "success",
                            error_kind: None,
                        },
                    ),
                    Err(failure) => record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::HelperOutcome {
                            stage: failure.stage,
                            elapsed_ms: failure.elapsed_ms,
                            outcome: helper_failure_outcome(failure.stage, failure.error.kind()),
                            error_kind: Some(format!("{:?}", failure.error.kind())),
                        },
                    ),
                }
                record_route(
                    route_events.as_ref(),
                    flow_id,
                    RouteEventKind::Terminal {
                        reason: "pending-helper-owner-changed",
                    },
                );
                context.retire_direct_helper_registration(&flow);
                if selected
                    .lock()
                    .expect("selected route lock")
                    .current
                    .get(&tuple)
                    .copied()
                    == Some(SelectedRoute { epoch, flow_id })
                {
                    selected
                        .lock()
                        .expect("selected route lock")
                        .current
                        .remove(&tuple);
                }
                let settled =
                    lose_pending_first_packet(&stats, route_events.as_ref(), flow_id, &proof);
                let mut extra_error = None;
                for _ in extra {
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::PacketOmission {
                            reason: "pending-packet-after-owner-retired",
                            disposition: "lost",
                        },
                    );
                    if let Err(error) = packet_finished(&stats, PacketDisposition::Lost) {
                        extra_error.get_or_insert(error);
                    }
                }
                let retired = receipts.retire(receipt).map_err(|error| {
                    io::Error::other(format!("retire stale helper receipt: {error:?}"))
                });
                issued_receipts.retain(|issued| *issued != receipt);
                settled?;
                if let Some(error) = extra_error {
                    return Err(error);
                }
                retired?;
                continue;
            }
            let (helper, elapsed_ms) = match outcome {
                Ok(value) => value,
                Err(failure) => {
                    let kind = failure.error.kind();
                    let outcome = helper_failure_outcome(failure.stage, kind);
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::HelperOutcome {
                            stage: failure.stage,
                            elapsed_ms: failure.elapsed_ms,
                            outcome,
                            error_kind: Some(format!("{kind:?}")),
                        },
                    );
                    if outcome == "refused" {
                        record_route(
                            route_events.as_ref(),
                            flow_id,
                            RouteEventKind::ProxyRefused {
                                reason: "socks5-connect-refused",
                            },
                        );
                        stats.proxy_refused.fetch_add(1, Ordering::Relaxed);
                    }
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::Terminal {
                            reason: match outcome {
                                "timeout" => "helper-timeout",
                                "refused" => "proxy-refused",
                                _ => "helper-io-failed",
                            },
                        },
                    );
                    context.retire_direct_helper_registration(&flow);
                    if selected
                        .lock()
                        .expect("selected route lock")
                        .current
                        .get(&tuple)
                        .copied()
                        == Some(SelectedRoute { epoch, flow_id })
                    {
                        selected
                            .lock()
                            .expect("selected route lock")
                            .current
                            .remove(&tuple);
                    }
                    let marked = mark_refused_flow(&refused, epoch);
                    let settled = reinject_selected_first_packet(
                        &handle,
                        &stats,
                        &packet,
                        &address,
                        &proof,
                        route_events.as_ref(),
                        flow_id,
                    );
                    let mut extra_error = None;
                    for (held, held_address) in extra {
                        if let Err(error) = reinject_pending_selected_packet(
                            &handle,
                            &stats,
                            &held,
                            &held_address,
                            route_events.as_ref(),
                            flow_id,
                        ) {
                            extra_error.get_or_insert(error);
                        }
                    }
                    let retired = receipts.retire(receipt).map_err(|error| {
                        io::Error::other(format!("retire failed helper receipt: {error:?}"))
                    });
                    issued_receipts.retain(|issued| *issued != receipt);
                    settled?;
                    if let Some(error) = extra_error {
                        return Err(error);
                    }
                    retired?;
                    if !marked {
                        return Err(io::Error::other("failed helper ledger full"));
                    }
                    continue;
                }
            };
            record_route(
                route_events.as_ref(),
                flow_id,
                RouteEventKind::HelperOutcome {
                    stage: "complete",
                    elapsed_ms,
                    outcome: "success",
                    error_kind: None,
                },
            );
            let setup = (|| -> Result<_, (&'static str, io::Error)> {
                receipts
                    .consume(
                        receipt,
                        &session_id,
                        tuple,
                        flow_id,
                        started.elapsed().as_nanos() as u64,
                    )
                    .map_err(|error| {
                        (
                            "tcp-receipt-consume-failed",
                            io::Error::other(format!("consume route receipt: {error:?}")),
                        )
                    })?;
                proof.lock().expect("direct proof lock").receipt_consumed = true;
                record_route(
                    route_events.as_ref(),
                    flow_id,
                    RouteEventKind::ReceiptConsumed,
                );
                let helper_tuple = DirectTuple {
                    transport: Proto::Tcp,
                    local: helper
                        .local_addr()
                        .map_err(|error| ("tcp-helper-local-unavailable", error))?,
                    remote: helper
                        .peer_addr()
                        .map_err(|error| ("tcp-helper-peer-unavailable", error))?,
                };
                proof.lock().expect("direct proof lock").proxy_connected = true;
                record_route(
                    route_events.as_ref(),
                    flow_id,
                    RouteEventKind::ProxyConnected {
                        helper_local: helper_tuple.local,
                        proxy_local: helper_tuple.remote,
                    },
                );
                let bridge = DirectTcpBridge::new(tuple, helper)
                    .map_err(|reason| ("tcp-bridge-setup-failed", io::Error::other(reason)))?;
                Ok((helper_tuple, bridge))
            })();
            let (helper_tuple, bridge) = match setup {
                Ok(value) => value,
                Err((reason, error)) => {
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::Terminal { reason },
                    );
                    context.retire_direct_helper_registration(&flow);
                    let retired = receipts.retire(receipt).map_err(|failure| {
                        io::Error::other(format!("retire failed TCP setup receipt: {failure:?}"))
                    });
                    issued_receipts.retain(|issued| *issued != receipt);
                    let mut registry = selected.lock().expect("selected route lock");
                    if registry.current.get(&tuple).copied()
                        == Some(SelectedRoute { epoch, flow_id })
                    {
                        registry.current.remove(&tuple);
                    }
                    drop(registry);
                    let marked = mark_refused_flow(&refused, epoch);
                    let settled = reinject_selected_first_packet(
                        &handle,
                        &stats,
                        &packet,
                        &address,
                        &proof,
                        route_events.as_ref(),
                        flow_id,
                    );
                    let mut extra_error = None;
                    for (held, held_address) in extra {
                        if let Err(failure) = reinject_pending_selected_packet(
                            &handle,
                            &stats,
                            &held,
                            &held_address,
                            route_events.as_ref(),
                            flow_id,
                        ) {
                            extra_error.get_or_insert(failure);
                        }
                    }
                    settled?;
                    if let Some(failure) = extra_error {
                        return Err(failure);
                    }
                    retired?;
                    if !marked {
                        return Err(io::Error::other("failed TCP setup ledger full"));
                    }
                    return Err(error);
                }
            };
            excluded
                .lock()
                .expect("helper route lock")
                .insert(helper_tuple);
            active.insert(
                tuple,
                ActiveTcp {
                    bridge: Some(bridge),
                    address: response_address,
                    flow: flow.clone(),
                    context: context.clone(),
                    excluded: Arc::clone(&excluded),
                    epoch,
                    receipt,
                    helper: helper_tuple,
                    flow_id,
                    last_transfer: (0, 0),
                    proof: Arc::clone(&proof),
                },
            );
            {
                let mut witnesses = ack_witnesses.lock().expect("ACK witness lock");
                if witnesses.len() < MAX_ACK_WITNESSES || witnesses.contains_key(&tuple) {
                    witnesses.insert(
                        tuple,
                        AckWitness {
                            epoch,
                            flow_id,
                            proof,
                            closed_at: None,
                            close_qpc: None,
                        },
                    );
                } else {
                    record_route(
                        route_events.as_ref(),
                        flow_id,
                        RouteEventKind::Terminal {
                            reason: "ack-witness-capacity",
                        },
                    );
                }
            }
            let mut held_packets = extra.into_iter();
            let first_result = forward_active_packet(
                tuple,
                &packet,
                address,
                ActiveFlows {
                    tcp: &mut active,
                    udp: &mut active_udp,
                },
                &handle,
                &stats,
                ForwardEvidence {
                    started,
                    route_events: route_events.as_ref(),
                },
                &observer,
            );
            match first_result {
                Ok(true) => {}
                Ok(false) => {
                    lose_unforwarded_pending_packets(
                        &stats,
                        route_events.as_ref(),
                        flow_id,
                        1 + held_packets.len(),
                    )?;
                    return Err(io::Error::other("selected TCP bridge missing after setup"));
                }
                Err(error) => {
                    lose_unforwarded_pending_packets(
                        &stats,
                        route_events.as_ref(),
                        flow_id,
                        held_packets.len(),
                    )?;
                    return Err(error);
                }
            }
            while let Some((held, held_address)) = held_packets.next() {
                match forward_active_packet(
                    tuple,
                    &held,
                    held_address,
                    ActiveFlows {
                        tcp: &mut active,
                        udp: &mut active_udp,
                    },
                    &handle,
                    &stats,
                    ForwardEvidence {
                        started,
                        route_events: route_events.as_ref(),
                    },
                    &observer,
                ) {
                    Ok(true) => {}
                    Ok(false) => {
                        lose_unforwarded_pending_packets(
                            &stats,
                            route_events.as_ref(),
                            flow_id,
                            1 + held_packets.len(),
                        )?;
                        return Err(io::Error::other("selected TCP bridge missing during drain"));
                    }
                    Err(error) => {
                        lose_unforwarded_pending_packets(
                            &stats,
                            route_events.as_ref(),
                            flow_id,
                            held_packets.len(),
                        )?;
                        return Err(error);
                    }
                }
            }
        }
        for flow in active.values_mut() {
            stats.processor_progress();
            let replies = flow
                .bridge
                .as_mut()
                .expect("active TCP bridge")
                .poll(started.elapsed().as_millis() as i64)
                .map_err(|error| io::Error::other(format!("selected TCP poll: {error}")))?;
            let transferred = flow
                .bridge
                .as_ref()
                .expect("active TCP bridge")
                .transferred_bytes();
            if transferred != flow.last_transfer {
                {
                    let mut proof = flow.proof.lock().expect("direct proof lock");
                    proof.client_to_upstream = transferred.0;
                    proof.upstream_to_client = transferred.1;
                }
                record_route(
                    route_events.as_ref(),
                    flow.flow_id,
                    RouteEventKind::Transfer {
                        client_to_upstream: transferred.0,
                        upstream_to_client: transferred.1,
                    },
                );
                flow.last_transfer = transferred;
            }
            if observer
                .lock()
                .expect("socket observer lock")
                .current_owner_epoch(flow.epoch.tuple)
                == Some(flow.epoch)
            {
                inject_replies(
                    &handle,
                    &stats,
                    replies,
                    flow.address,
                    route_events.as_ref(),
                    flow.flow_id,
                    Some(&flow.proof),
                )?;
            } else if !replies.is_empty() {
                stats
                    .synthetic_reply_lost
                    .fetch_add(replies.len() as u64, Ordering::Relaxed);
                record_route(
                    route_events.as_ref(),
                    flow.flow_id,
                    RouteEventKind::SyntheticReplyFailed {
                        reason: "retired-owner-changed",
                    },
                );
            }
        }
        for (tuple, flow) in &active_udp {
            stats.processor_progress();
            for _ in 0..8 {
                let payload = match flow.association().poll_reply() {
                    Ok(Some(payload)) => payload,
                    Ok(None) => break,
                    Err(error) if error.kind() == io::ErrorKind::ConnectionReset => {
                        // Windows reports an ICMP port-unreachable on a connected
                        // UDP socket as WSAECONNRESET. No reply packet was
                        // delivered; a completed QUIC peer can cause this on
                        // close. Keep the association and count the signal.
                        stats.udp_relay_resets.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                    Err(error) => return Err(error),
                };
                let reply = build_udp_reply(*tuple, &payload)?;
                if observer
                    .lock()
                    .expect("socket observer lock")
                    .current_owner_epoch(*tuple)
                    != Some(flow.epoch)
                {
                    stats.synthetic_reply_lost.fetch_add(1, Ordering::Relaxed);
                    record_route(
                        route_events.as_ref(),
                        flow.flow_id,
                        RouteEventKind::SyntheticReplyFailed {
                            reason: "retired-owner-changed",
                        },
                    );
                    continue;
                }
                inject_replies(
                    &handle,
                    &stats,
                    vec![reply],
                    flow.address,
                    route_events.as_ref(),
                    flow.flow_id,
                    Some(&flow.proof),
                )?;
                {
                    let mut proof = flow.proof.lock().expect("direct proof lock");
                    proof.upstream_to_client = proof
                        .upstream_to_client
                        .saturating_add(payload.len() as u64);
                    proof.peer_datagrams = proof.peer_datagrams.saturating_add(1);
                }
                record_route(
                    route_events.as_ref(),
                    flow.flow_id,
                    RouteEventKind::DatagramTransfer {
                        client_to_upstream: 0,
                        upstream_to_client: payload.len() as u64,
                        client_datagrams: 0,
                        peer_datagrams: 1,
                    },
                );
            }
        }
    }
}

/// Stop must be called before the native proxy lease is released.
pub struct DirectPacketRuntime {
    handle: Option<Arc<Handle>>,
    context: NativeObservationContext,
    observer: Arc<Mutex<DirectSocketObserver>>,
    receive_worker: Option<JoinHandle<io::Result<()>>>,
    process_worker: Option<JoinHandle<io::Result<()>>>,
    watchdog_worker: Option<JoinHandle<()>>,
    stats: Arc<SharedStats>,
    route_events: Option<RouteEventLease>,
}

impl DirectPacketRuntime {
    #[cfg(test)]
    pub fn start(
        component: &Path,
        context: NativeObservationContext,
        session_id: String,
        proxy: std::net::SocketAddr,
        password: Zeroizing<String>,
        max_flows: usize,
    ) -> io::Result<Self> {
        Self::start_with_artifact(
            component, context, session_id, proxy, password, max_flows, None,
        )
    }

    pub fn start_with_artifact(
        component: &Path,
        context: NativeObservationContext,
        session_id: String,
        proxy: std::net::SocketAddr,
        password: Zeroizing<String>,
        max_flows: usize,
        route_path: Option<&Path>,
    ) -> io::Result<Self> {
        if max_flows == 0 || max_flows > 256 || !proxy.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid direct route bounds",
            ));
        }
        let route_events = route_path
            .map(|path| RouteEventLease::open(path, session_id.clone(), 1024))
            .transpose()?;
        let route_sink = route_events.as_ref().map(RouteEventLease::sink);
        let api = Api::load_verified(component)?;
        let observer = Arc::new(Mutex::new(DirectSocketObserver::start(
            &api,
            SOCKET_EVENTS,
        )?));
        let handle = Arc::new(api.open(PACKET_FILTER, NETWORK_LAYER, 0)?);
        handle.set_queue(4096, 100, 16 * 1024 * 1024)?;
        let (sender, receiver) = mpsc::sync_channel(PACKET_QUEUE);
        let (process_ready_tx, process_ready_rx) = mpsc::channel();
        let queued_candidates = Arc::new(AtomicUsize::new(0));
        let queued_udp_candidates = Arc::new(AtomicUsize::new(0));
        let selected = Arc::new(Mutex::new(SelectedRegistry::default()));
        let ack_witnesses = Arc::new(Mutex::new(BTreeMap::new()));
        let excluded = Arc::new(Mutex::new(BTreeSet::new()));
        let refused = Arc::new(Mutex::new(BTreeMap::new()));
        let connecting = Arc::new(AtomicBool::new(false));
        let route_context = context.clone();
        let stats = Arc::new(SharedStats::new());
        let receive_worker = {
            let receive_route_sink = route_sink.clone();
            let receive_context = route_context.clone();
            let queued_candidates = Arc::clone(&queued_candidates);
            let queued_udp_candidates = Arc::clone(&queued_udp_candidates);
            let handle = Arc::clone(&handle);
            let observer = Arc::clone(&observer);
            let selected = Arc::clone(&selected);
            let ack_witnesses = Arc::clone(&ack_witnesses);
            let excluded = Arc::clone(&excluded);
            let refused = Arc::clone(&refused);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            thread::Builder::new()
                .name("fragcap-direct-receive".into())
                .spawn(move || {
                    let result = receive_packets(ReceiveRuntimeInputs {
                        handle: Arc::clone(&handle),
                        context: receive_context,
                        proxy,
                        sender,
                        queued_candidates,
                        queued_udp_candidates,
                        observer,
                        selected,
                        ack_witnesses,
                        excluded,
                        refused,
                        connecting,
                        stats,
                        route_events: receive_route_sink,
                    });
                    if result.is_err() {
                        let _ = handle.stop();
                    }
                    result
                })?
        };
        let process_worker = {
            let process_handle = Arc::clone(&handle);
            let queued_candidates = Arc::clone(&queued_candidates);
            let queued_udp_candidates = Arc::clone(&queued_udp_candidates);
            let observer = Arc::clone(&observer);
            let selected = Arc::clone(&selected);
            let ack_witnesses = Arc::clone(&ack_witnesses);
            let excluded = Arc::clone(&excluded);
            let refused = Arc::clone(&refused);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            let launched = thread::Builder::new()
                .name("fragcap-direct-process".into())
                .spawn(move || {
                    let result = process_packets(ProcessRuntimeInputs {
                        receiver,
                        ready: process_ready_tx,
                        queued_candidates,
                        queued_udp_candidates,
                        handle: Arc::clone(&process_handle),
                        observer,
                        context,
                        selected,
                        ack_witnesses,
                        excluded,
                        refused,
                        connecting,
                        stats,
                        session_id,
                        proxy,
                        password,
                        max_flows,
                        route_events: route_sink,
                    });
                    if result.is_err() {
                        let _ = process_handle.stop();
                    }
                    result
                });
            match launched {
                Ok(worker) => worker,
                Err(error) => {
                    let _ = handle.stop_receiving();
                    let _ = receive_worker.join();
                    return Err(error);
                }
            }
        };
        if process_ready_rx
            .recv_timeout(Duration::from_secs(2))
            .is_err()
        {
            let _ = handle.stop();
            let _ = receive_worker.join();
            let _ = process_worker.join();
            return Err(io::Error::other(
                "direct packet worker did not become ready",
            ));
        }
        let watchdog_worker = {
            let watchdog_handle = Arc::clone(&handle);
            let watchdog_stats = Arc::clone(&stats);
            let launched = thread::Builder::new()
                .name("fragcap-direct-watchdog".into())
                .spawn(move || {
                    while !watchdog_stats.stop_watchdog.load(Ordering::Acquire) {
                        let now = watchdog_stats.started.elapsed().as_millis() as u64 + 1;
                        if watchdog_stats.stalled_packet(now) {
                            watchdog_stats.watchdog_release_after_progress_ms.store(
                                now.saturating_sub(
                                    watchdog_stats.processor_progress_ms.load(Ordering::Acquire),
                                ),
                                Ordering::Release,
                            );
                            watchdog_stats
                                .watchdog_released
                                .store(true, Ordering::Release);
                            let _ = watchdog_handle.stop();
                            return;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                });
            match launched {
                Ok(worker) => worker,
                Err(error) => {
                    let _ = handle.stop();
                    let _ = receive_worker.join();
                    let _ = process_worker.join();
                    return Err(error);
                }
            }
        };
        route_context.set_direct_route_active(true);
        Ok(Self {
            handle: Some(handle),
            context: route_context,
            observer,
            receive_worker: Some(receive_worker),
            process_worker: Some(process_worker),
            watchdog_worker: Some(watchdog_worker),
            stats,
            route_events,
        })
    }

    pub fn stop(&mut self) -> io::Result<DirectRuntimeStats> {
        let was_running = self.receive_worker.is_some() || self.process_worker.is_some();
        let mut first_error = None;
        if self.receive_worker.is_some() {
            let handle = self.handle.as_ref().expect("running route handle");
            if let Err(error) = handle.stop_receiving() {
                // A worker or watchdog may already have shut the handle down.
                // Keep the worker's causal error instead of masking it with
                // the resulting broken-pipe status from shutdown.
                if error.raw_os_error() != Some(232) {
                    first_error = Some(error);
                }
                let _ = handle.stop();
            }
            if let Err(error) = self
                .receive_worker
                .take()
                .unwrap()
                .join()
                .map_err(|_| io::Error::other("direct receive worker panicked"))
                .and_then(|result| result)
            {
                first_error.get_or_insert(error);
            }
        }
        if let Some(worker) = self.process_worker.take() {
            if let Err(error) = worker
                .join()
                .map_err(|_| io::Error::other("direct process worker panicked"))
                .and_then(|result| result)
            {
                first_error.get_or_insert(error);
            }
        }
        self.context.set_direct_route_active(false);
        self.stats.stop_watchdog.store(true, Ordering::Release);
        if let Some(worker) = self.watchdog_worker.take() {
            if worker.join().is_err() {
                first_error.get_or_insert(io::Error::other("direct watchdog panicked"));
            }
        }
        if self.stats.watchdog_released.load(Ordering::Acquire) {
            first_error.get_or_insert(io::Error::other(
                "watchdog released broad handle after worker stall",
            ));
        }
        if let Err(error) = self.observer.lock().expect("socket observer lock").stop() {
            first_error.get_or_insert(error);
        }
        if was_running && !self.stats.watchdog_released.load(Ordering::Acquire) {
            if let Err(error) = self.handle.as_ref().expect("running route handle").stop() {
                first_error.get_or_insert(error);
            }
        }
        if let Some(handle) = self.handle.take() {
            match Arc::try_unwrap(handle) {
                Ok(handle) => {
                    if let Err(error) = handle.close() {
                        first_error.get_or_insert(error);
                    }
                }
                Err(handle) => {
                    first_error.get_or_insert(io::Error::other(
                        "direct packet handle still shared at cleanup",
                    ));
                    drop(handle);
                }
            }
        }
        #[cfg(test)]
        if FAIL_NEXT_CLEANUP_REPORT.swap(false, Ordering::AcqRel) {
            first_error.get_or_insert(io::Error::other("injected direct cleanup uncertainty"));
        }
        let packets = {
            let mut accounting = self.stats.packets.lock().expect("route accounting lock");
            match accounting.lose_unfinished() {
                Ok(0) => {}
                Ok(unfinished) => {
                    first_error.get_or_insert(io::Error::other(format!(
                        "{unfinished} delivered packets lost after worker exit"
                    )));
                }
                Err(error) => {
                    first_error.get_or_insert(io::Error::other(format!(
                        "fatal packet accounting overflow: {error:?}"
                    )));
                }
            }
            *accounting
        };
        if let Err(error) = packets.release() {
            first_error.get_or_insert(io::Error::other(format!(
                "unfinished direct packets: {error:?}"
            )));
        }
        let route_stream = match self.route_events.as_mut() {
            Some(lease) => match lease.finish() {
                Ok(outcome) => Some(outcome),
                Err(error) => {
                    first_error.get_or_insert(io::Error::other(format!("route stream: {error}")));
                    Some(lease.accounting())
                }
            },
            None => None,
        };
        let report = DirectRuntimeStats {
            packets,
            socket_events: self.observer.lock().expect("socket observer lock").stats(),
            admission_busy_bypassed: self.stats.admission_busy_bypassed.load(Ordering::Relaxed),
            candidate_capacity_bypassed: self
                .stats
                .candidate_capacity_bypassed
                .load(Ordering::Relaxed),
            candidate_max_queue_us: self.stats.candidate_max_queue_us.load(Ordering::Relaxed),
            candidate_max_hold_us: self.stats.candidate_max_hold_us.load(Ordering::Relaxed),
            candidate_over_50ms: self.stats.candidate_over_50ms.load(Ordering::Relaxed),
            bypass_reasons: self
                .stats
                .bypass_reasons
                .lock()
                .expect("bypass reason lock")
                .clone(),
            proxy_refused: self.stats.proxy_refused.load(Ordering::Relaxed),
            udp_relay_resets: self.stats.udp_relay_resets.load(Ordering::Relaxed),
            synthetic_reply_lost: self.stats.synthetic_reply_lost.load(Ordering::Relaxed),
            event_history_overflow: self.stats.event_history_overflow.load(Ordering::Relaxed),
            pre_receive_loss_unmeasurable: true,
            watchdog_released: self.stats.watchdog_released.load(Ordering::Relaxed),
            watchdog_release_after_progress_ms: self
                .stats
                .watchdog_release_after_progress_ms
                .load(Ordering::Relaxed),
            route_stream,
        };
        if let Some(error) = first_error {
            Err(io::Error::other(format!(
                "direct route cleanup: {error}; counts: {report:?}"
            )))
        } else {
            Ok(report)
        }
    }
}

impl Drop for DirectPacketRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udp_association_error_does_not_claim_proxy_refusal() {
        assert_eq!(
            udp_association_failure_outcome(io::ErrorKind::PermissionDenied),
            ("io-error", "udp-association-io-failed")
        );
        assert_eq!(
            udp_association_failure_outcome(io::ErrorKind::TimedOut),
            ("timeout", "udp-association-timeout")
        );
    }

    #[test]
    fn failed_pending_forward_settles_only_unprocessed_suffix() {
        let stats = SharedStats::new();
        for _ in 0..3 {
            packet_received(&stats).unwrap();
        }
        // The failed forward has already taken its own terminal disposition.
        packet_finished(&stats, PacketDisposition::Lost).unwrap();
        lose_unforwarded_pending_packets(&stats, None, 7, 2).unwrap();
        let accounting = *stats.packets.lock().unwrap();
        assert_eq!(accounting.received, 3);
        assert_eq!(accounting.lost, 3);
        assert_eq!(accounting.in_flight, 0);
        accounting.release().unwrap();
    }

    #[test]
    fn active_tcp_drop_revokes_live_helper_authority() {
        use fragcap_proxy::ConnectionDescriptor;
        use std::net::{TcpListener, TcpStream};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = listener.local_addr().unwrap();
        let stream = TcpStream::connect(proxy).unwrap();
        let (peer, _) = listener.accept().unwrap();
        let helper = stream.local_addr().unwrap();
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let flow = OriginalFlow {
            tuple,
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 0,
            endpoint_id: 7,
            pid: 42,
            process_instance_id: "process-42-100".into(),
            process_image: "game.exe".into(),
            process_role: "client".into(),
            first_packet_digest: None,
        };
        let context = NativeObservationContext::default();
        context
            .register_direct_helper("tcp", helper, proxy, flow.clone(), 100, 200)
            .unwrap();
        let descriptor = ConnectionDescriptor {
            transport: "tcp",
            client_peer: helper,
            proxy_local: proxy,
        };
        let receipt = DirectReceiptLedger::new("session".into(), 1)
            .issue(flow.clone(), 300)
            .unwrap();
        let excluded = Arc::new(Mutex::new(BTreeSet::from([DirectTuple {
            transport: Proto::Tcp,
            local: helper,
            remote: proxy,
        }])));
        let active = ActiveTcp {
            bridge: Some(DirectTcpBridge::new(tuple, stream).unwrap()),
            address: Address::default(),
            flow: flow.clone(),
            context: context.clone(),
            excluded: Arc::clone(&excluded),
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            receipt,
            helper: DirectTuple {
                transport: Proto::Tcp,
                local: helper,
                remote: proxy,
            },
            flow_id: 1,
            last_transfer: (0, 0),
            proof: Arc::new(Mutex::new(DirectRouteProof::default())),
        };
        assert_eq!(context.direct_helper_flow(descriptor, 150), Ok(Some(flow)));
        drop(active);
        assert_eq!(context.direct_helper_flow(descriptor, 150), Ok(None));
        assert!(excluded.lock().unwrap().is_empty());
        drop(peer);
    }

    #[test]
    fn selected_work_captured_before_close_cannot_enter_reused_tuple_bridge() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let a = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let b = SelectedRoute {
            epoch: SocketEpoch {
                open_qpc: 200,
                ..a.epoch
            },
            flow_id: 2,
        };
        assert!(selected_work_admission(a, Some(a), Some(a.epoch), Some(a.epoch), 125).is_ok());
        assert_eq!(
            selected_work_admission(a, None, None, None, 125),
            Err("selected-route-generation-retired")
        );
        assert_eq!(
            selected_work_admission(a, Some(b), Some(b.epoch), Some(b.epoch), 125),
            Err("selected-route-generation-retired")
        );
        assert_eq!(
            selected_work_admission(a, Some(b), Some(b.epoch), Some(b.epoch), 225),
            Err("selected-route-generation-retired")
        );
        assert_eq!(
            selected_packet_admission(b, Some(b.epoch), 199),
            Err("selected-packet-before-open-or-clock-ambiguous")
        );
        assert!(selected_work_admission(b, Some(b), Some(b.epoch), Some(b.epoch), 225).is_ok());
        assert_eq!(
            selected_packet_admission(b, None, 225),
            Err("selected-owner-epoch-changed-or-ambiguous")
        );
    }

    #[test]
    fn queued_udp_packet_predating_close_keeps_only_exact_generation() {
        let tuple = DirectTuple {
            transport: Proto::Udp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let old = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let new = SelectedRoute {
            epoch: SocketEpoch {
                endpoint_id: 8,
                pid: 43,
                open_qpc: 200,
                ..old.epoch
            },
            flow_id: 2,
        };
        let mut routes = SelectedRegistry::default();
        routes.current.insert(tuple, old);
        routes.enqueue(old);
        assert!(routes.record_retirement(old, 200, true));
        routes.current.insert(tuple, new);
        assert_eq!(routes.queued(old), 1);
        assert_eq!(routes.retired_at(tuple, old.epoch, 199), Some((old, true)));
        assert_eq!(routes.retired_at(tuple, old.epoch, 200), None);
        assert_eq!(routes.retired_at(tuple, new.epoch, 199), None);
        assert_eq!(routes.current.get(&tuple), Some(&new));
        assert!(routes.has_tuple(tuple));
        routes.finish(old);
        assert_eq!(routes.queued(old), 0);
    }

    #[test]
    fn udp_unproven_reason_requires_exact_close_qpc() {
        assert_eq!(
            udp_unproven_bypass_reason(199, Some(200)),
            "selected-udp-before-close-owner-unproven-original-reinjected"
        );
        assert_eq!(
            udp_unproven_bypass_reason(200, Some(200)),
            "selected-udp-close-boundary-original-reinjected"
        );
        assert_eq!(
            udp_unproven_bypass_reason(201, Some(200)),
            "selected-udp-after-close-original-reinjected"
        );
        assert_eq!(
            udp_unproven_bypass_reason(201, None),
            "selected-udp-close-unavailable-original-reinjected"
        );
        let route = SelectedRoute {
            epoch: SocketEpoch {
                tuple: DirectTuple {
                    transport: Proto::Udp,
                    local: "192.0.2.10:49152".parse().unwrap(),
                    remote: "198.51.100.20:443".parse().unwrap(),
                },
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let absent = CurrentOwner::Absent;
        assert_eq!(
            udp_post_close_lineage_flow(route, Some(route), absent, absent, 201, Some(200)),
            Some(1)
        );
        assert_eq!(
            udp_post_close_lineage_flow(route, Some(route), absent, absent, 200, Some(200)),
            None
        );
        assert_eq!(
            udp_post_close_lineage_flow(route, None, absent, absent, 201, Some(200)),
            None
        );
        assert_eq!(
            udp_post_close_lineage_flow(
                route,
                Some(route),
                absent,
                CurrentOwner::Unique(SocketEpoch {
                    endpoint_id: 8,
                    open_qpc: 200,
                    ..route.epoch
                }),
                201,
                Some(200),
            ),
            None
        );
        assert_eq!(
            udp_post_close_lineage_flow(route, Some(route), absent, absent, 201, None),
            None
        );
    }

    #[test]
    fn failed_udp_association_after_control_connect_retires_helper() {
        use fragcap_proxy::ConnectionDescriptor;
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            drop(stream); // The control helper connected, but SOCKS negotiation fails.
        });
        let flow = OriginalFlow {
            tuple: DirectTuple {
                transport: Proto::Udp,
                local: "192.0.2.10:49152".parse().unwrap(),
                remote: "198.51.100.20:443".parse().unwrap(),
            },
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 0,
            endpoint_id: 7,
            pid: 42,
            process_instance_id: "process-42-100".into(),
            process_image: "game.exe".into(),
            process_role: "client".into(),
            first_packet_digest: None,
        };
        let context = NativeObservationContext::default();
        let pending = PendingUdpHelperRegistration::new(context.clone(), flow.clone());
        let mut descriptor = None;
        let result = DirectUdpAssociation::new_with_hook(
            proxy,
            flow.tuple.remote,
            b"test-password",
            Duration::from_millis(250),
            |stream| {
                let helper = stream.local_addr()?;
                let proxy_local = stream.peer_addr()?;
                context
                    .register_direct_helper("tcp", helper, proxy_local, flow.clone(), 100, 200)
                    .map_err(io::Error::other)?;
                descriptor = Some(ConnectionDescriptor {
                    transport: "tcp",
                    client_peer: helper,
                    proxy_local,
                });
                Ok(())
            },
        );
        assert!(result.is_err());
        server.join().unwrap();
        let descriptor = descriptor.expect("control helper must register before SOCKS fails");
        assert_eq!(context.direct_helper_flow(descriptor, 150), Ok(Some(flow)));
        drop(pending); // Constructor failure exits before an ActiveUdp owns the helper.
        assert_eq!(context.direct_helper_flow(descriptor, 150), Ok(None));
    }

    #[test]
    fn queued_old_generation_survives_close_and_reused_open_until_drained() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let old = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let new = SelectedRoute {
            epoch: SocketEpoch {
                endpoint_id: 8,
                pid: 43,
                open_qpc: 200,
                ..old.epoch
            },
            flow_id: 2,
        };
        let registry = Arc::new(Mutex::new(SelectedRegistry::default()));
        {
            let mut routes = registry.lock().unwrap();
            routes.current.insert(tuple, old);
            routes.enqueue(old); // receive worker has already proved old ownership
            assert!(routes.record_retirement(old, 200, true));
            routes.current.insert(tuple, new);
            assert_eq!(routes.current.get(&tuple), Some(&new));
            assert_eq!(routes.retired_at(tuple, old.epoch, 199), Some((old, true)));
            assert_eq!(routes.retired_at(tuple, old.epoch, 200), None);
            assert_eq!(routes.retired_at(tuple, new.epoch, 199), None);
            assert_eq!(routes.queued(old), 1);
        }
        // The already queued receipt keeps its own generation even if socket
        // history expires before the worker drains it.
        let guard = SelectedWorkGuard {
            selected: Arc::clone(&registry),
            route: old,
        };
        assert_eq!(registry.lock().unwrap().queued(old), 1);
        drop(guard);
        let mut routes = registry.lock().unwrap();
        assert_eq!(routes.queued(old), 0);
        routes.retiring.get_mut(&old).unwrap().bridge_available = false;
        routes.prune_history(Instant::now() + SELECTED_HISTORY_GRACE);
        assert!(!routes.retiring.contains_key(&old));
        assert_eq!(routes.current.get(&tuple), Some(&new));
        assert!(routes.has_tuple(tuple));
        assert_eq!(routes.quarantined.get(&tuple), Some(&200));
        assert!(!quarantined_packet_is_proven_new(
            CurrentOwner::Absent,
            CurrentOwner::Absent,
            200,
            150
        ));
        assert!(!quarantined_packet_is_proven_new(
            CurrentOwner::Ambiguous,
            CurrentOwner::Unique(new.epoch),
            200,
            250
        ));
        assert!(!quarantined_packet_is_proven_new(
            CurrentOwner::Unique(old.epoch),
            CurrentOwner::Unique(new.epoch),
            200,
            150
        ));
        let newer = SocketEpoch {
            open_qpc: 300,
            ..new.epoch
        };
        assert!(quarantined_packet_is_proven_new(
            CurrentOwner::Unique(newer),
            CurrentOwner::Unique(newer),
            200,
            301
        ));
        // A competing OPEN retires the old route at the new route's own OPEN
        // QPC, so equality is the ordinary safe reuse boundary.
        assert!(quarantined_packet_is_proven_new(
            CurrentOwner::Unique(newer),
            CurrentOwner::Unique(newer),
            300,
            301
        ));
        assert!(!quarantined_packet_is_proven_new(
            CurrentOwner::Unique(newer),
            CurrentOwner::Unique(newer),
            200,
            300
        ));
    }

    #[test]
    fn selected_retirement_history_capacity_refuses_without_displacing_queued_work() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let old = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let mut routes = SelectedRegistry::default();
        routes.current.insert(tuple, old);
        routes.enqueue(old);
        for flow_id in 2..=SOCKET_EVENTS as u64 + 1 {
            let route = SelectedRoute { flow_id, ..old };
            assert!(routes.record_retirement(route, 200, false));
        }
        assert!(!routes.record_retirement(old, 200, true));
        assert_eq!(routes.current.get(&tuple), Some(&old));
        assert_eq!(routes.queued(old), 1);
    }

    #[test]
    fn post_close_control_tail_classification_requires_exact_close_and_no_payload() {
        let mut packet = vec![0u8; 40];
        packet[0] = 0x45;
        packet[2..4].copy_from_slice(&40u16.to_be_bytes());
        packet[8] = 64;
        packet[9] = 6;
        packet[12..16].copy_from_slice(&[192, 0, 2, 10]);
        packet[16..20].copy_from_slice(&[198, 51, 100, 20]);
        packet[20..22].copy_from_slice(&49152u16.to_be_bytes());
        packet[22..24].copy_from_slice(&443u16.to_be_bytes());
        packet[32] = 0x50;
        packet[33] = 0x14; // RST+ACK, no payload.
        let absent = CurrentOwner::Absent;
        assert!(selected_post_close_control_tail(
            &packet,
            166,
            Some(100),
            absent,
            absent
        ));
        packet[33] = 0x11; // FIN+ACK, no payload.
        assert!(selected_post_close_control_tail(
            &packet,
            166,
            Some(100),
            absent,
            absent
        ));
        assert!(!selected_post_close_control_tail(
            &packet,
            100,
            Some(100),
            absent,
            absent
        ));
        assert!(!selected_post_close_control_tail(
            &packet, 166, None, absent, absent
        ));
        assert!(!selected_post_close_control_tail(
            &packet,
            166,
            Some(100),
            CurrentOwner::Ambiguous,
            absent
        ));
        packet[33] = 0x10; // An ACK alone is not the observed terminal control.
        assert!(!selected_post_close_control_tail(
            &packet,
            166,
            Some(100),
            absent,
            absent
        ));
        packet[33] = 0x14;
        packet[2..4].copy_from_slice(&41u16.to_be_bytes());
        packet.push(1); // Even a reset carrying one byte is selected data loss.
        assert!(!selected_post_close_control_tail(
            &packet,
            166,
            Some(100),
            absent,
            absent
        ));
    }

    #[test]
    fn selected_tcp_omission_labels_payload_and_exact_close_relation() {
        let mut packet = vec![0u8; 40];
        packet[0] = 0x45;
        packet[2..4].copy_from_slice(&40u16.to_be_bytes());
        packet[8] = 64;
        packet[9] = 6;
        packet[12..16].copy_from_slice(&[192, 0, 2, 10]);
        packet[16..20].copy_from_slice(&[198, 51, 100, 20]);
        packet[20..22].copy_from_slice(&49152u16.to_be_bytes());
        packet[22..24].copy_from_slice(&443u16.to_be_bytes());
        packet[32] = 0x50;
        packet[33] = 0x10; // Header-only ACK, not a FIN/RST control tail.
        for (packet_qpc, close_qpc, expected) in [
            (
                199,
                Some(200),
                "selected-tcp-header-only-before-close-unproven",
            ),
            (
                200,
                Some(200),
                "selected-tcp-header-only-close-boundary-unproven",
            ),
            (
                201,
                Some(200),
                "selected-tcp-header-only-after-close-unproven",
            ),
            (201, None, "selected-tcp-header-only-close-unavailable"),
        ] {
            assert_eq!(
                selected_tcp_omission_detail(&packet, packet_qpc, close_qpc),
                Some(expected)
            );
        }
        packet[2..4].copy_from_slice(&41u16.to_be_bytes());
        packet.push(7);
        for (packet_qpc, close_qpc, expected) in [
            (199, Some(200), "selected-tcp-payload-before-close-unproven"),
            (
                200,
                Some(200),
                "selected-tcp-payload-close-boundary-unproven",
            ),
            (201, Some(200), "selected-tcp-payload-after-close-unproven"),
            (201, None, "selected-tcp-payload-close-unavailable"),
        ] {
            assert_eq!(
                selected_tcp_omission_detail(&packet, packet_qpc, close_qpc),
                Some(expected)
            );
        }
    }

    #[test]
    fn quarantine_control_tail_requires_one_exact_retired_route() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let route = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let mut registry = SelectedRegistry::default();
        assert!(registry.record_retirement(route, 200, false));
        assert_eq!(
            registry.exact_retired_route_at_close(tuple, 200),
            Some(route)
        );
        assert_eq!(registry.exact_retired_route_at_close(tuple, 201), None);
        let competing = SelectedRoute {
            flow_id: 2,
            epoch: SocketEpoch {
                endpoint_id: 8,
                open_qpc: 150,
                ..route.epoch
            },
        };
        assert!(registry.record_retirement(competing, 200, false));
        assert_eq!(registry.exact_retired_route_at_close(tuple, 200), None);
    }

    #[test]
    fn selected_fin_ack_after_close_cannot_enter_reused_tuple() {
        let mut fin_ack = vec![0u8; 40];
        fin_ack[0] = 0x45;
        fin_ack[2..4].copy_from_slice(&40u16.to_be_bytes());
        fin_ack[8] = 64;
        fin_ack[9] = 6;
        fin_ack[12..16].copy_from_slice(&[192, 0, 2, 10]);
        fin_ack[16..20].copy_from_slice(&[198, 51, 100, 20]);
        fin_ack[20..22].copy_from_slice(&49152u16.to_be_bytes());
        fin_ack[22..24].copy_from_slice(&443u16.to_be_bytes());
        fin_ack[32] = 0x50;
        fin_ack[33] = 0x11;
        let head = raw_transport_head(&fin_ack).unwrap();
        assert_eq!(head.tcp_flags, Some(0x11));
        assert_eq!(head.transport_len, 20);
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: head.source,
            remote: head.destination,
        };
        let a = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let b = SocketEpoch {
            open_qpc: 200,
            ..a.epoch
        };
        // CLOSE may be delivered on the socket handle before this already
        // captured control packet is drained from the network handle. Its
        // original datagram must not be returned to an absent or new owner.
        assert!(selected_packet_admission(a, None, 150).is_err());
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Absent,
            150
        ));
        assert!(selected_work_admission(a, None, None, None, 150).is_err());
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Absent,
            150
        ));
        // Numeric endpoint/PID reuse still cannot send old A bytes through B.
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Unique(b),
            150
        ));
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Unique(b),
            225
        ));
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Ambiguous,
            150
        ));
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Absent,
            100
        ));
        assert!(!stale_selected_can_pass_original(
            a,
            CurrentOwner::Absent,
            0
        ));
        let mut registry = SelectedRegistry::default();
        registry.current.insert(tuple, a);
        let outbound = Address {
            flags: 1 << 17,
            ..Address::default()
        };
        assert_eq!(
            fatal_packet_quarantine(&registry, &fin_ack, outbound),
            (true, Some(a.flow_id))
        );
        assert!(registry.record_retirement(a, 200, false));
        registry.retiring.get_mut(&a).unwrap().bridge_available = false;
        registry.prune_history(Instant::now() + SELECTED_HISTORY_GRACE);
        assert_eq!(
            fatal_packet_quarantine(&registry, &fin_ack, outbound),
            (true, None)
        );
        assert_eq!(
            fatal_packet_quarantine(&registry, b"bad", outbound),
            (true, None)
        );
        assert_eq!(
            fatal_packet_quarantine(&SelectedRegistry::default(), &fin_ack, outbound),
            (false, None)
        );
    }

    #[test]
    fn closed_pending_helper_retires_receipt_and_frees_same_tuple() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let flow = OriginalFlow {
            tuple,
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 0,
            endpoint_id: 7,
            pid: 42,
            process_instance_id: "process-42-1".into(),
            process_image: "game.exe".into(),
            process_role: "client".into(),
            first_packet_digest: None,
        };
        let mut receipts = DirectReceiptLedger::new("test".into(), 2);
        let a_receipt = receipts.issue(flow.clone(), 1_000).unwrap();
        let stats = SharedStats::new();
        packet_received(&stats).unwrap();
        packet_received(&stats).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let worker = thread::spawn(move || {
            while !worker_cancelled.load(Ordering::Acquire) {
                thread::yield_now();
            }
        });
        let (_sender, result) = mpsc::channel();
        let pending = PendingTcp {
            flow: flow.clone(),
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 90,
            },
            packet: vec![1],
            address: Address::default(),
            response_address: Address::default(),
            receipt: a_receipt,
            flow_id: 1,
            proof: Arc::new(Mutex::new(DirectRouteProof::default())),
            result,
            worker,
            cancelled: Arc::clone(&cancelled),
            extra: vec![(vec![2], Address::default())],
        };
        let mut retired = Vec::new();
        let retired_receipt =
            retire_pending_tcp(pending, &stats, None, "socket-closed", &mut retired).unwrap();
        assert_eq!(retired_receipt, a_receipt);
        receipts.retire(retired_receipt).unwrap();
        assert!(cancelled.load(Ordering::Acquire));
        let mut b = flow;
        b.endpoint_id = 7; // Numeric endpoint reuse is allowed after the old CLOSE.
        b.first_packet_at_ns = 200;
        let b_receipt = receipts.issue(b, 1_000).unwrap();
        assert_ne!(a_receipt, b_receipt);
        assert!(matches!(
            receipts.consume(a_receipt, "test", tuple, 1, 300),
            Err(crate::deep_capture::direct_route::ReceiptError::Unknown)
        ));
        assert_eq!(retired.len(), 1);
        retired.pop().unwrap().join().unwrap();
        let accounting = *stats.packets.lock().unwrap();
        assert_eq!(accounting.received, 2);
        assert_eq!(accounting.lost, 2);
        accounting.release().unwrap();
    }

    #[test]
    fn retired_bridge_does_not_hold_tuple_receipt_against_next_generation() {
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let old = OriginalFlow {
            tuple,
            first_packet_at_ns: 100,
            first_packet_clock_error_ns: 0,
            endpoint_id: 7,
            pid: 42,
            process_instance_id: "process-42-1".into(),
            process_image: "game.exe".into(),
            process_role: "client".into(),
            first_packet_digest: None,
        };
        let mut receipts = DirectReceiptLedger::new("test".into(), 2);
        let old_id = receipts.issue(old.clone(), 1_000).unwrap();
        receipts.consume(old_id, "test", tuple, 1, 150).unwrap();
        let route = SelectedRoute {
            epoch: SocketEpoch {
                tuple,
                endpoint_id: 7,
                pid: 42,
                open_qpc: 100,
            },
            flow_id: 1,
        };
        let mut registry = SelectedRegistry::default();
        registry.current.insert(tuple, route);
        registry.enqueue(route);
        assert!(registry.record_retirement(route, 200, true));
        receipts.retire(old_id).unwrap();
        let mut next = old;
        next.first_packet_at_ns = 200;
        next.endpoint_id = 8;
        next.pid = 43;
        assert!(receipts.issue(next, 1_000).is_ok());
        assert_eq!(
            registry.retired_at(tuple, route.epoch, 150),
            Some((route, true))
        );
        assert_eq!(registry.queued(route), 1);
    }

    #[test]
    fn pending_helper_keeps_concurrent_unknown_game_syns_and_bypasses_exact_proxy() {
        use crate::deep_capture::direct_route::{DirectPacketHead, SocketEventKind};

        let proxy: SocketAddr = "127.0.0.1:1080".parse().unwrap();
        let other_loopback: SocketAddr = "127.0.0.1:1081".parse().unwrap();
        let remote: SocketAddr = "198.51.100.20:443".parse().unwrap();
        let helper_pid = 99;
        assert!(helper_packet_bypasses_candidate(
            proxy, proxy, true, None, helper_pid
        ));
        assert!(helper_packet_bypasses_candidate(
            proxy, proxy, false, None, helper_pid
        ));
        assert!(helper_packet_bypasses_candidate(
            remote,
            proxy,
            true,
            Some(helper_pid),
            helper_pid,
        ));
        assert!(!helper_packet_bypasses_candidate(
            remote, proxy, true, None, helper_pid
        ));
        assert!(!helper_packet_bypasses_candidate(
            remote,
            proxy,
            true,
            Some(12),
            helper_pid,
        ));
        assert!(!helper_packet_bypasses_candidate(
            other_loopback,
            proxy,
            true,
            None,
            helper_pid,
        ));

        let evidence = CaptureProcessEvidence {
            events: vec![
                ProcessEvent::started(10, 1, "platform.exe", "", Timestamp::from_nanos(1000)),
                ProcessEvent::started(12, 10, "client.exe", "", Timestamp::from_nanos(1200)),
            ],
            launch_pid: Some(10),
            launch_at: Some(Timestamp::from_nanos(1000)),
            stage_transitions: vec![StageTransition {
                kind: StageTransitionKind::Matched,
                pid: 12,
                role: "client".into(),
                stage: None,
                at: Timestamp::from_nanos(1220),
            }],
            watcher_report: Some(WatcherReport::default()),
            ..Default::default()
        };
        let mut candidates = CandidateBuffer::new(2, 128, CANDIDATE_WAIT_NS).unwrap();
        let mut socket_events = Vec::new();
        for (index, port) in [40000u16, 40001u16].into_iter().enumerate() {
            assert!(!helper_packet_bypasses_candidate(
                remote, proxy, true, None, helper_pid
            ));
            let tuple = DirectTuple {
                transport: Proto::Tcp,
                local: SocketAddr::from(([127, 0, 0, 1], port)),
                remote,
            };
            candidates
                .enqueue(
                    DirectPacketHead {
                        tuple,
                        at_qpc: 101,
                        at_unix_ns: 1300,
                        clock_error_ns: 1,
                        initial: true,
                        packet_digest: Some([index as u8; 32]),
                    },
                    vec![index as u8 + 1],
                    port,
                    0,
                )
                .ok()
                .expect("unknown owner must enter the candidate buffer");
            socket_events.push(DirectSocketEvent {
                tuple,
                endpoint_id: index as u64 + 1,
                pid: 12,
                at_qpc: 100,
                at_unix_ns: 1250,
                clock_error_ns: 1000,
                kind: SocketEventKind::Open,
            });
        }
        assert!(candidates
            .resolve(None, &[], 1_000_000, helper_pid)
            .is_empty());
        assert_eq!(candidates.len(), 2);
        let decisions = candidates.resolve(Some(&evidence), &socket_events, 2_000_000, helper_pid);
        assert_eq!(decisions.len(), 2);
        for (index, decision) in decisions.into_iter().enumerate() {
            let CandidateDecision::Selected { flow, address, .. } = decision else {
                panic!("both exact game sockets must be selected")
            };
            assert_eq!(flow.pid, 12);
            assert_eq!(flow.endpoint_id, index as u64 + 1);
            assert_eq!(address, [40000u16, 40001u16][index]);
        }
        assert_eq!(candidates.len(), 0);
    }

    #[test]
    fn handshake_ack_does_not_acknowledge_returned_payload() {
        fn packet(sequence: u32, acknowledgment: u32, flags: u8, payload: &[u8]) -> Vec<u8> {
            let mut packet = vec![0u8; 40 + payload.len()];
            packet[0] = 0x45;
            let packet_len = u16::try_from(packet.len()).unwrap();
            packet[2..4].copy_from_slice(&packet_len.to_be_bytes());
            packet[8] = 64;
            packet[9] = 6;
            packet[12..16].copy_from_slice(&[192, 0, 2, 10]);
            packet[16..20].copy_from_slice(&[198, 51, 100, 20]);
            packet[20..22].copy_from_slice(&40000u16.to_be_bytes());
            packet[22..24].copy_from_slice(&443u16.to_be_bytes());
            packet[24..28].copy_from_slice(&sequence.to_be_bytes());
            packet[28..32].copy_from_slice(&acknowledgment.to_be_bytes());
            packet[32] = 0x50;
            packet[33] = flags;
            packet[40..].copy_from_slice(payload);
            packet
        }
        let response = packet(1000, 0, 0x18, b"reply");
        let end = tcp_payload_end(&response).unwrap();
        assert_eq!(end, 1005);
        assert!(!tcp_acknowledges_returned_data(
            &packet(2000, 1000, 0x10, b""),
            end
        ));
        assert!(tcp_acknowledges_returned_data(
            &packet(2000, 1005, 0x10, b""),
            end
        ));
        assert!(!tcp_acknowledges_returned_data(
            &packet(2000, 1005, 0x00, b""),
            end
        ));
        let wrapping = tcp_payload_end(&packet(u32::MAX - 2, 0, 0x18, b"wrap")).unwrap();
        assert_eq!(wrapping, 1);
        assert!(tcp_acknowledges_returned_data(
            &packet(2000, 1, 0x10, b""),
            wrapping
        ));
    }

    #[test]
    fn returned_ack_witness_uses_capture_qpc_across_close_and_reuse() {
        let now = Instant::now();
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let epoch = SocketEpoch {
            tuple,
            endpoint_id: 7,
            pid: 42,
            open_qpc: 100,
        };
        let reused = SocketEpoch {
            open_qpc: 201,
            ..epoch
        };
        let witness = AckWitness {
            epoch,
            flow_id: 1,
            proof: Arc::new(Mutex::new(DirectRouteProof::default())),
            closed_at: Some(now),
            close_qpc: Some(200),
        };
        assert!(witness.accepts_owner(CurrentOwner::Unique(epoch), 150));
        assert!(witness.accepts_owner(CurrentOwner::Absent, 150));
        assert!(witness.accepts_owner(CurrentOwner::Unique(reused), 150));
        assert!(!witness.accepts_owner(CurrentOwner::Ambiguous, 150));
        assert!(!witness.accepts_owner(CurrentOwner::Unique(reused), 200));
        assert!(!witness.accepts_owner(CurrentOwner::Unique(reused), 210));
        assert!(!witness.accepts_owner(CurrentOwner::Unique(reused), 100));
    }
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
        num::NonZeroUsize,
        os::windows::process::CommandExt,
        process::{Child, Command, Stdio},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    use base64::Engine;
    use bytes::Buf;
    use fragcap_core::{ProcessEvent, Timestamp, WatcherReport};
    use fragcap_proxy::{
        ApplicationEvent, ApplicationEventSink, ApplicationSinkAccounting, DestinationAuthority,
        DestinationPolicy, EventDisposition, InspectionAction, InspectionDecider,
        InspectionDecision, InspectionReason, InspectionRequest, LeafCache, NativeProxyBackend,
        NativeProxyConfig, ProtocolLimits,
    };
    use h3_quinn::Connection;
    use hyper::{Request, Response};
    use quinn::Endpoint;

    use crate::deep_capture::{
        CaptureProcessEvidence, ExactGameDecider, StageTransition, StageTransitionKind,
    };

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    #[test]
    fn socket_history_retains_live_owner_and_expires_closed_churn() {
        use crate::deep_capture::direct_route::SocketEventKind;

        let tuple = DirectTuple {
            transport: Proto::Udp,
            local: "127.0.0.1:40123".parse().unwrap(),
            remote: "127.0.0.1:40124".parse().unwrap(),
        };
        let event = |endpoint_id, at_qpc, at_unix_ns, kind| DirectSocketEvent {
            tuple,
            endpoint_id,
            pid: 42,
            at_qpc,
            at_unix_ns,
            clock_error_ns: 0,
            kind,
        };
        let mut events = Vec::new();
        for endpoint_id in 1..=5_000 {
            events.push(event(endpoint_id, 1, 1, SocketEventKind::Open));
            events.push(event(endpoint_id, 2, 2, SocketEventKind::Close));
        }
        // The same endpoint identifier may be reused after its old CLOSE.
        events.push(event(1, 3, 3, SocketEventKind::Open));
        // A recent CLOSE must remain long enough to disambiguate a held packet.
        events.push(event(5_001, 4, 1, SocketEventKind::Open));
        events.push(event(5_001, 5, 25, SocketEventKind::Close));
        prune_socket_events(&mut events, 20);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].endpoint_id, 1);
        assert_eq!(events[0].at_qpc, 3);
        prune_socket_events(&mut events, 30);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].endpoint_id, 1);
    }

    struct UninspectedDirectOwner;

    #[derive(Default)]
    struct RetiredApplicationSink(AtomicU64);

    impl ApplicationEventSink for RetiredApplicationSink {
        fn try_emit(&self, _event: ApplicationEvent) -> EventDisposition {
            self.0.fetch_add(1, Ordering::Relaxed);
            EventDisposition::Retired
        }

        fn accounting(&self) -> ApplicationSinkAccounting {
            let attempted = self.0.load(Ordering::Relaxed);
            ApplicationSinkAccounting {
                attempted_events: attempted,
                dropped_events: attempted,
                ..ApplicationSinkAccounting::default()
            }
        }
    }

    impl InspectionDecider for UninspectedDirectOwner {
        fn decide(&self, _request: InspectionRequest, _budget: Duration) -> InspectionDecision {
            let mut decision = InspectionDecision::verified_direct_route(0);
            decision.action = InspectionAction::Uninspected;
            decision.reason = InspectionReason::KnownMiddleware;
            decision
        }
    }

    fn spawn_client(protocol: &str, original: std::net::SocketAddr) -> Child {
        spawn_client_with_ca(protocol, original, None)
    }

    fn spawn_client_with_ca(
        protocol: &str,
        original: std::net::SocketAddr,
        ca: Option<&[u8]>,
    ) -> Child {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "deep_capture::direct_runtime::tests::direct_runtime_child",
                "--nocapture",
            ])
            .env("FRAGCAP_S200_CHILD_PROTOCOL", protocol)
            .env("FRAGCAP_S200_CHILD_ORIGINAL", original.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);
        if let Some(ca) = ca {
            command.env(
                "FRAGCAP_S200_CHILD_CA",
                base64::engine::general_purpose::STANDARD.encode(ca),
            );
        }
        command.spawn().unwrap()
    }

    fn release_client(child: &mut Child) {
        child.stdin.as_mut().unwrap().write_all(b"g").unwrap();
    }

    fn finish_client(mut child: Child) {
        drop(child.stdin.take());
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn client_context(pid: u32) -> NativeObservationContext {
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let started = Timestamp::from_nanos(i64::try_from(now_ns - 1_000_000_000).unwrap());
        let context = NativeObservationContext::default();
        context.record_process_evidence(&CaptureProcessEvidence {
            events: vec![ProcessEvent::started(pid, 1, "client.exe", "", started)],
            launch_pid: Some(pid),
            launch_at: Some(started),
            stage_transitions: vec![StageTransition {
                kind: StageTransitionKind::Matched,
                pid,
                role: "client".into(),
                stage: None,
                at: started,
            }],
            watcher_report: Some(WatcherReport::default()),
            ..Default::default()
        });
        context
    }

    fn paired_client_context(first: u32, second: u32) -> NativeObservationContext {
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let started = Timestamp::from_nanos(i64::try_from(now_ns - 1_000_000_000).unwrap());
        let context = NativeObservationContext::default();
        context.record_process_evidence(&CaptureProcessEvidence {
            events: vec![
                ProcessEvent::started(first, 1, "client.exe", "", started),
                ProcessEvent::started(second, first, "client.exe", "", started),
            ],
            launch_pid: Some(first),
            launch_at: Some(started),
            stage_transitions: [first, second]
                .into_iter()
                .map(|pid| StageTransition {
                    kind: StageTransitionKind::Matched,
                    pid,
                    role: "client".into(),
                    stage: None,
                    at: started,
                })
                .collect(),
            watcher_report: Some(WatcherReport::default()),
            ..Default::default()
        });
        context
    }

    fn read_chain_pid(reader: &mut impl BufRead, role: &str) -> u32 {
        let marker = format!("S200_{role}=");
        let mut line = String::new();
        for _ in 0..20 {
            line.clear();
            assert_ne!(
                reader.read_line(&mut line).unwrap(),
                0,
                "missing {role} pid"
            );
            if let Some((_, pid)) = line.trim().split_once(&marker) {
                return pid.parse().unwrap();
            }
        }
        panic!("missing {role} pid in child output");
    }

    fn chain_evidence(platform: u32, publisher: u32, client: u32) -> CaptureProcessEvidence {
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let started = Timestamp::from_nanos(i64::try_from(now_ns - 1_000_000_000).unwrap());
        CaptureProcessEvidence {
            events: vec![
                ProcessEvent::started(platform, std::process::id(), "platform.exe", "", started),
                ProcessEvent::started(publisher, platform, "publisher.exe", "", started),
                ProcessEvent::started(client, publisher, "client.exe", "", started),
            ],
            launch_pid: Some(platform),
            launch_at: Some(started),
            stage_transitions: [
                (platform, "platform"),
                (publisher, "publisher"),
                (client, "client"),
            ]
            .into_iter()
            .map(|(pid, role)| StageTransition {
                kind: StageTransitionKind::Matched,
                pid,
                role: role.into(),
                stage: None,
                at: started,
            })
            .collect(),
            watcher_report: Some(WatcherReport::default()),
            ..Default::default()
        }
    }

    #[test]
    fn direct_runtime_child() {
        let Ok(protocol) = std::env::var("FRAGCAP_S200_CHILD_PROTOCOL") else {
            return;
        };
        let original: std::net::SocketAddr = std::env::var("FRAGCAP_S200_CHILD_ORIGINAL")
            .unwrap()
            .parse()
            .unwrap();
        if let Ok(role) = std::env::var("FRAGCAP_S200_CHAIN_ROLE") {
            if role == "platform" || role == "publisher" {
                println!("S200_{}={}", role.to_uppercase(), std::process::id());
                std::io::stdout().flush().unwrap();
                if role == "publisher" {
                    let mut gate = [0u8; 1];
                    std::io::stdin().read_exact(&mut gate).unwrap();
                    assert_eq!(gate, *b"p");
                    let proxy: SocketAddr = std::env::var("FRAGCAP_S200_CHILD_PROXY")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let authorization = std::env::var("FRAGCAP_S200_CHILD_PROXY_AUTH").unwrap();
                    let mut connection = TcpStream::connect(proxy).unwrap();
                    connection
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    connection
                        .set_write_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    write!(
                        connection,
                        "GET http://{original}/publisher HTTP/1.1\r\nHost: {original}\r\nProxy-Authorization: {authorization}\r\nConnection: close\r\n\r\n"
                    )
                    .unwrap();
                    let mut response = Vec::new();
                    connection.read_to_end(&mut response).unwrap();
                    assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"));
                    assert!(response.ends_with(b"PUBLISHER"));
                }
                let next_role = if role == "platform" {
                    "publisher"
                } else {
                    "final"
                };
                let mut next = Command::new(std::env::current_exe().unwrap());
                next.args([
                    "--exact",
                    "deep_capture::direct_runtime::tests::direct_runtime_child",
                    "--nocapture",
                ])
                .env("FRAGCAP_S200_CHAIN_ROLE", next_role)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .creation_flags(CREATE_NO_WINDOW);
                let mut child = next.spawn().unwrap();
                assert!(child.wait().unwrap().success());
                return;
            }
            assert_eq!(role, "final");
            println!("S200_CLIENT={}", std::process::id());
            std::io::stdout().flush().unwrap();
        }
        let mut gate = [0u8; 1];
        std::io::stdin().read_exact(&mut gate).unwrap();
        assert_eq!(gate, *b"g");
        if protocol == "tcp" || protocol == "tcp-roundtrip" {
            let mut client = TcpStream::connect_timeout(&original, Duration::from_secs(3)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            client.write_all(b"runtime-client-request").unwrap();
            if protocol == "tcp-roundtrip" {
                let mut response = [0u8; 20];
                client.read_exact(&mut response).unwrap();
                assert_eq!(&response, b"runtime-origin-reply");
                client.write_all(b"runtime-client-finish").unwrap();
            }
            client.shutdown(std::net::Shutdown::Write).unwrap();
            let mut response = Vec::new();
            client.read_to_end(&mut response).unwrap();
            assert_eq!(
                response,
                if protocol == "tcp" {
                    b"runtime-origin-reply".as_slice()
                } else {
                    b"".as_slice()
                }
            );
            if let Ok(peer) = std::env::var("FRAGCAP_S200_CHAIN_UDP_ORIGINAL") {
                let peer: SocketAddr = peer.parse().unwrap();
                let udp = UdpSocket::bind(SocketAddr::new(peer.ip(), 0)).unwrap();
                udp.connect(peer).unwrap();
                udp.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                udp.send(b"chain-udp-request").unwrap();
                let mut reply = [0u8; 64];
                let length = udp.recv(&mut reply).unwrap();
                assert_eq!(&reply[..length], b"chain-udp-reply");
            }
        } else if protocol == "sentinel-tcp" {
            let mut client = TcpStream::connect_timeout(&original, Duration::from_secs(3)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            client.write_all(b"sentinel-request").unwrap();
            client.shutdown(std::net::Shutdown::Write).unwrap();
            let mut response = Vec::new();
            client.read_to_end(&mut response).unwrap();
            assert_eq!(response, b"sentinel-reply");
        } else if protocol == "udp" {
            let client = UdpSocket::bind(SocketAddr::new(original.ip(), 0)).unwrap();
            client.connect(original).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            client.send(b"runtime-udp-request").unwrap();
            let mut response = [0u8; 128];
            let length = client.recv(&mut response).unwrap();
            assert_eq!(&response[..length], b"runtime-udp-reply");
        } else if protocol == "udp-burst" {
            let client = UdpSocket::bind(SocketAddr::new(original.ip(), 0)).unwrap();
            client.connect(original).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            for ordinal in 0..3 {
                client.send(format!("burst-{ordinal}").as_bytes()).unwrap();
            }
            let mut received = BTreeSet::new();
            for _ in 0..3 {
                let mut response = [0u8; 64];
                let length = client.recv(&mut response).unwrap();
                received.insert(String::from_utf8(response[..length].to_vec()).unwrap());
            }
            assert_eq!(
                received,
                BTreeSet::from(["burst-0".into(), "burst-1".into(), "burst-2".into()])
            );
        } else if protocol == "udp-reuse" {
            let client = UdpSocket::bind(SocketAddr::new(original.ip(), 0)).unwrap();
            let local = client.local_addr().unwrap();
            client.connect(original).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            client.send(b"first-socket").unwrap();
            let mut response = [0u8; 128];
            let length = client.recv(&mut response).unwrap();
            assert_eq!(&response[..length], b"first-reply");
            drop(client);
            thread::sleep(Duration::from_millis(100));
            let next = UdpSocket::bind(local).unwrap();
            next.connect(original).unwrap();
            next.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            next.send(b"second-socket").unwrap();
            let length = next.recv(&mut response).unwrap();
            assert_eq!(&response[..length], b"second-reply");
        } else if protocol == "sentinel-udp" {
            let client = UdpSocket::bind(SocketAddr::new(original.ip(), 0)).unwrap();
            client.connect(original).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            client.send(b"sentinel-udp-request").unwrap();
            let mut response = [0u8; 128];
            let length = client.recv(&mut response).unwrap();
            assert_eq!(&response[..length], b"sentinel-udp-reply");
        } else if protocol == "http" {
            let mut client = TcpStream::connect_timeout(&original, Duration::from_secs(3)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            write!(
                client,
                "GET /direct HTTP/1.1\r\nHost: {original}\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
            let mut response = Vec::new();
            client.read_to_end(&mut response).unwrap();
            assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"));
            assert!(response.ends_with(b"DIRECT"));
        } else if protocol == "tls" || protocol == "tls-dns" {
            let ca = base64::engine::general_purpose::STANDARD
                .decode(std::env::var("FRAGCAP_S200_CHILD_CA").unwrap())
                .unwrap();
            let mut roots = rustls::RootCertStore::empty();
            roots
                .add(rustls::pki_types::CertificateDer::from(ca))
                .unwrap();
            let mut config = fragcap_proxy::tls_client_config_with_roots(roots).unwrap();
            Arc::make_mut(&mut config).alpn_protocols = vec![b"http/1.1".to_vec()];
            let server_name = if protocol == "tls-dns" {
                rustls::pki_types::ServerName::try_from("localhost")
                    .unwrap()
                    .to_owned()
            } else {
                rustls::pki_types::ServerName::IpAddress(original.ip().into())
            };
            let connection = rustls::ClientConnection::new(config, server_name).unwrap();
            let tcp = TcpStream::connect_timeout(&original, Duration::from_secs(3)).unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut client = rustls::StreamOwned::new(connection, tcp);
            write!(
                client,
                "GET /direct-tls HTTP/1.1\r\nHost: {original}\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
            const RESPONSE: &[u8] =
                b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nTLS-DIRECT";
            let mut response = vec![0u8; RESPONSE.len()];
            client.read_exact(&mut response).unwrap();
            assert_eq!(response, RESPONSE);
        } else if protocol == "h3" {
            let ca = base64::engine::general_purpose::STANDARD
                .decode(std::env::var("FRAGCAP_S200_CHILD_CA").unwrap())
                .unwrap();
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                tokio::time::timeout(Duration::from_secs(8), async move {
                    let mut roots = rustls::RootCertStore::empty();
                    roots.add(ca.into()).unwrap();
                    let provider = Arc::new(rustls::crypto::ring::default_provider());
                    let tls = Arc::new(
                        rustls::ClientConfig::builder_with_provider(provider)
                            .with_protocol_versions(&[&rustls::version::TLS13])
                            .unwrap()
                            .with_root_certificates(roots)
                            .with_no_client_auth(),
                    );
                    let limits = ProtocolLimits {
                        max_concurrent_streams: 4,
                        max_socks_udp_datagram_bytes: 4096,
                        idle_timeout: Duration::from_secs(5),
                        ..ProtocolLimits::default()
                    };
                    let config = fragcap_proxy::build_quic_client_config(&tls, &limits).unwrap();
                    let mut endpoint = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
                    endpoint.set_default_client_config(config);
                    let connection = endpoint
                        .connect(original, "127.0.0.1")
                        .unwrap()
                        .await
                        .unwrap();
                    let (mut driver, mut sender) =
                        h3::client::new(Connection::new(connection.clone()))
                            .await
                            .unwrap();
                    let driver_task = tokio::spawn(async move {
                        std::future::poll_fn(|cx| driver.poll_close(cx)).await
                    });
                    let mut stream = sender
                        .send_request(
                            Request::get("https://127.0.0.1/direct-quic")
                                .body(())
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    stream.finish().await.unwrap();
                    let response = stream.recv_response().await.unwrap();
                    assert_eq!(response.status(), 200);
                    let mut body = Vec::new();
                    while let Some(mut chunk) = stream.recv_data().await.unwrap() {
                        while chunk.has_remaining() {
                            body.extend_from_slice(chunk.chunk());
                            let length = chunk.chunk().len();
                            chunk.advance(length);
                        }
                    }
                    assert_eq!(body, b"H3-DIRECT");
                    connection.close(0_u32.into(), b"complete");
                    endpoint.wait_idle().await;
                    driver_task.abort();
                })
                .await
                .unwrap();
            });
        } else {
            panic!("unknown client protocol");
        }
    }

    #[test]
    fn watchdog_requires_unfinished_packet_and_stalled_processor() {
        let stats = SharedStats::new();
        assert!(!stats.stalled_packet(1_000));
        packet_received(&stats).unwrap();
        stats.processor_progress_ms.store(10, Ordering::Release);
        assert!(!stats.stalled_packet(99));
        assert!(stats.stalled_packet(100));
        stats.processor_progress_ms.store(105, Ordering::Release);
        assert!(!stats.stalled_packet(110));
        packet_finished(&stats, PacketDisposition::Reinjected).unwrap();
        assert!(!stats.stalled_packet(1_000));
        stats.packets.lock().unwrap().release().unwrap();
    }

    #[test]
    fn candidate_flood_reserves_work_queue_space_for_selected_packets() {
        let (sender, receiver) = mpsc::sync_channel(PACKET_QUEUE);
        let queued_candidates = AtomicUsize::new(0);
        let queued_udp_candidates = AtomicUsize::new(0);
        let tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let head = super::super::direct_route::DirectPacketHead {
            tuple,
            at_qpc: 1,
            at_unix_ns: 1,
            clock_error_ns: 0,
            initial: true,
            packet_digest: Some([7; 32]),
        };
        for index in 0..1_000 {
            let _ = queue_candidate_with_selected_reserve(
                &sender,
                &queued_candidates,
                &queued_udp_candidates,
                head,
                b"candidate",
                Address::default(),
                index,
            );
        }
        assert_eq!(
            queued_candidates.load(Ordering::Acquire),
            CANDIDATE_QUEUE_LIMIT
        );
        assert!(sender
            .try_send(WorkPacket::Selected(
                SelectedRoute {
                    epoch: SocketEpoch {
                        tuple,
                        endpoint_id: 7,
                        pid: 42,
                        open_qpc: 1,
                    },
                    flow_id: 1,
                },
                b"selected".to_vec(),
                Address::default(),
            ))
            .is_ok());
        for _ in 0..CANDIDATE_QUEUE_LIMIT {
            assert!(matches!(
                receive_work(&receiver, &queued_candidates, &queued_udp_candidates),
                Ok(WorkPacket::Candidate(..))
            ));
        }
        assert_eq!(queued_candidates.load(Ordering::Acquire), 0);
        assert_eq!(queued_udp_candidates.load(Ordering::Acquire), 0);
        assert!(matches!(
            receive_work(&receiver, &queued_candidates, &queued_udp_candidates),
            Ok(WorkPacket::Selected(..))
        ));
    }

    #[test]
    fn udp_ingress_burst_cannot_consume_tcp_syn_reserve() {
        let (sender, receiver) = mpsc::sync_channel(PACKET_QUEUE);
        let queued_candidates = AtomicUsize::new(0);
        let queued_udp_candidates = AtomicUsize::new(0);
        let tcp_tuple = DirectTuple {
            transport: Proto::Tcp,
            local: "192.0.2.10:49152".parse().unwrap(),
            remote: "198.51.100.20:443".parse().unwrap(),
        };
        let udp_tuple = DirectTuple {
            transport: Proto::Udp,
            ..tcp_tuple
        };
        let mut head = super::super::direct_route::DirectPacketHead {
            tuple: udp_tuple,
            at_qpc: 1,
            at_unix_ns: 1,
            clock_error_ns: 0,
            initial: true,
            packet_digest: Some([7; 32]),
        };
        let accepted = (0..1_000)
            .filter(|index| {
                queue_candidate_with_selected_reserve(
                    &sender,
                    &queued_candidates,
                    &queued_udp_candidates,
                    head,
                    b"udp",
                    Address::default(),
                    *index,
                )
            })
            .count();
        assert_eq!(accepted, CANDIDATE_UDP_QUEUE_LIMIT);
        assert_eq!(queued_udp_candidates.load(Ordering::Acquire), accepted);
        head.tuple = tcp_tuple;
        assert!(queue_candidate_with_selected_reserve(
            &sender,
            &queued_candidates,
            &queued_udp_candidates,
            head,
            b"target-syn",
            Address::default(),
            1_001,
        ));
        assert_eq!(queued_candidates.load(Ordering::Acquire), accepted + 1);
        for _ in 0..accepted {
            assert!(matches!(
                receive_work(&receiver, &queued_candidates, &queued_udp_candidates),
                Ok(WorkPacket::Candidate(head, ..)) if head.tuple.transport == Proto::Udp
            ));
        }
        assert!(matches!(
            receive_work(&receiver, &queued_candidates, &queued_udp_candidates),
            Ok(WorkPacket::Candidate(head, ..)) if head.tuple == tcp_tuple
        ));
        assert_eq!(queued_candidates.load(Ordering::Acquire), 0);
        assert_eq!(queued_udp_candidates.load(Ordering::Acquire), 0);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_stalled_worker_releases_broad_handle() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let mut child = spawn_client("tcp", original);
        let context = client_context(child.id());
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(2),
        )
        .unwrap()
        .with_session_id("s200-stalled-worker")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut lease = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_inspection_decider(Arc::new(ExactGameDecider::new(
                context.clone(),
                vec!["client".into()],
            )))
            .start(Duration::from_secs(2))
            .unwrap();
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-stalled-worker".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        STALL_NEXT_CANDIDATE.store(true, Ordering::Release);
        release_client(&mut child);
        let deadline = Instant::now() + Duration::from_secs(2);
        while !runtime.stats.watchdog_released.load(Ordering::Acquire) {
            assert!(
                Instant::now() < deadline,
                "watchdog did not release stalled handle"
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            runtime
                .stats
                .watchdog_release_after_progress_ms
                .load(Ordering::Acquire)
                <= 100,
            "watchdog exceeded its controlled release bound"
        );
        let error = runtime.stop().unwrap_err();
        assert!(error.to_string().contains("watchdog_released: true"));
        runtime.stats.packets.lock().unwrap().release().unwrap();
        let _ = child.kill();
        let _ = child.wait();
        drop(origin);
        lease.cleanup(Duration::from_secs(2));

        let sentinel = TcpListener::bind("127.0.0.1:0").unwrap();
        let sentinel_address = sentinel.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = sentinel.accept().unwrap();
            let mut bytes = [0; 4];
            stream.read_exact(&mut bytes).unwrap();
            assert_eq!(&bytes, b"post");
            stream.write_all(b"pass").unwrap();
        });
        let mut client =
            TcpStream::connect_timeout(&sentinel_address, Duration::from_secs(2)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client.write_all(b"post").unwrap();
        let mut reply = [0; 4];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(&reply, b"pass");
        server.join().unwrap();
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_cold_platform_publisher_final_chain() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let udp_origin = UdpSocket::bind("127.0.0.1:0").unwrap();
        udp_origin
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let original_udp = udp_origin.local_addr().unwrap();
        let udp_server = thread::spawn(move || {
            let mut payload = [0u8; 64];
            let (length, peer) = udp_origin.recv_from(&mut payload).unwrap();
            assert_eq!(&payload[..length], b"chain-udp-request");
            udp_origin.send_to(b"chain-udp-reply", peer).unwrap();
        });
        let server = thread::spawn(move || {
            let mut selected = 0;
            let mut publisher = 0;
            for _ in 0..3 {
                let (mut stream, _) = origin.accept().unwrap();
                let mut first = [0u8; 1];
                stream.read_exact(&mut first).unwrap();
                if first == *b"r" {
                    let mut body = vec![first[0]];
                    stream.read_to_end(&mut body).unwrap();
                    assert_eq!(body, b"runtime-client-request");
                    selected += 1;
                    stream.write_all(b"runtime-origin-reply").unwrap();
                } else if first == *b"s" {
                    let mut body = vec![first[0]];
                    stream.read_to_end(&mut body).unwrap();
                    assert_eq!(body, b"sentinel-request");
                    stream.write_all(b"sentinel-reply").unwrap();
                } else {
                    let mut head = vec![first[0]];
                    while !head.ends_with(b"\r\n\r\n") {
                        let mut byte = [0u8; 1];
                        stream.read_exact(&mut byte).unwrap();
                        head.push(byte[0]);
                    }
                    assert!(String::from_utf8_lossy(&head).contains("/publisher"));
                    publisher += 1;
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nPUBLISHER")
                        .unwrap();
                }
            }
            (selected, publisher)
        });
        let context = NativeObservationContext::default();
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(3),
        )
        .unwrap()
        .with_session_id("s200-managed-chain")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        policy.grant_for_test(original_udp);
        let mut proxy = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_inspection_decider(Arc::new(ExactGameDecider::new(
                context.clone(),
                vec!["client".into()],
            )))
            .start(Duration::from_secs(3))
            .unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "deep_capture::direct_runtime::tests::direct_runtime_child",
                "--nocapture",
            ])
            .env("FRAGCAP_S200_CHAIN_ROLE", "platform")
            .env("FRAGCAP_S200_CHILD_PROTOCOL", "tcp")
            .env("FRAGCAP_S200_CHILD_ORIGINAL", original.to_string())
            .env("FRAGCAP_S200_CHAIN_UDP_ORIGINAL", original_udp.to_string())
            .env("FRAGCAP_S200_CHILD_PROXY", proxy.endpoint().to_string())
            .env(
                "FRAGCAP_S200_CHILD_PROXY_AUTH",
                proxy.capability_proof().proxy_authorization().as_str(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW);
        let mut chain = command.spawn().unwrap();
        let mut unrelated = spawn_client("sentinel-tcp", original);
        let mut output = BufReader::new(chain.stdout.take().unwrap());
        let platform = read_chain_pid(&mut output, "PLATFORM");
        let publisher = read_chain_pid(&mut output, "PUBLISHER");
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context.clone(),
            "s200-managed-chain".into(),
            proxy.endpoint(),
            proxy.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        chain.stdin.as_mut().unwrap().write_all(b"p").unwrap();
        let client = read_chain_pid(&mut output, "CLIENT");
        assert_ne!(platform, publisher);
        assert_ne!(publisher, client);
        assert_ne!(platform, client);
        context.record_process_evidence(&chain_evidence(platform, publisher, client));
        // The unrelated process shares the exact original destination and is
        // released beside the final client, not after the route has stopped.
        release_client(&mut unrelated);
        chain.stdin.as_mut().unwrap().write_all(b"g").unwrap();
        finish_client(unrelated);
        drop(chain.stdin.take());
        let outcome = chain.wait_with_output().unwrap();
        assert!(
            outcome.status.success(),
            "{}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        let bridged = context.direct_helper_flows_for_test();
        let packet_counts = *runtime.stats.packets.lock().unwrap();
        let bypass = runtime.stats.bypass_reasons.lock().unwrap().clone();
        let socket_counts = runtime.observer.lock().unwrap().stats();
        assert_eq!(
            bridged.len(),
            3,
            "only the final client may enter the bridge: owners={bridged:?}, packets={:?}, bypass={:?}, socket={:?}",
            packet_counts,
            bypass,
            socket_counts,
        );
        for flow in &bridged {
            assert_eq!(flow.pid, client);
            assert!(flow
                .process_instance_id
                .starts_with(&format!("process-{client}-")));
            assert_ne!(flow.tuple.local, flow.tuple.remote);
            assert_ne!(flow.endpoint_id, 0);
        }
        assert_eq!(
            bridged
                .iter()
                .filter(|flow| flow.tuple.transport == Proto::Tcp && flow.tuple.remote == original)
                .count(),
            1
        );
        assert_eq!(
            bridged
                .iter()
                .filter(
                    |flow| flow.tuple.transport == Proto::Udp && flow.tuple.remote == original_udp
                )
                .count(),
            2
        );
        let stats = runtime.stop().unwrap();
        assert!(stats.packets.consumed > 0);
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(server.join().unwrap(), (1, 1));
        udp_server.join().unwrap();
        let report = proxy.cleanup(Duration::from_secs(3));
        assert_eq!(
            report.observation.protocol.socks_connect_succeeded, 1,
            "managed chain: protocol={:?}, route={stats:?}, owners={bridged:?}",
            report.observation.protocol
        );
        assert_eq!(report.observation.protocol.socks_udp_associate_succeeded, 1);
        // The test payload has no recognized application protocol, so its
        // bytes are retained with unknown provenance rather than plaintext.
        assert_eq!(
            report.observation.protocol.socks_tcp_opaque
                + report.observation.protocol.socks_tcp_undetermined,
            1
        );
        assert_eq!(report.observation.protocol.generic_streams_unknown, 1);
        assert_eq!(report.observation.protocol.generic_streams_plain, 0);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_cancel_then_restart_releases_route() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = origin.accept().unwrap();
            let mut body = Vec::new();
            stream.read_to_end(&mut body).unwrap();
            assert_eq!(body, b"sentinel-request");
            stream.write_all(b"sentinel-reply").unwrap();
        });
        let mut child = spawn_client("sentinel-tcp", original);
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            client_context(child.id()),
            "s200-cancel-before-client".into(),
            "127.0.0.1:9".parse().unwrap(),
            Zeroizing::new("unused-capability".to_string()),
            4,
        )
        .unwrap();
        let stopped = runtime.stop().unwrap();
        assert_eq!(stopped.packets.lost, 0);
        release_client(&mut child);
        finish_client(child);
        server.join().unwrap();
        live_tcp_case(IpAddr::V4(Ipv4Addr::LOCALHOST), false);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_cleanup_failure_reports_uncertainty_and_releases_handle() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            NativeObservationContext::default(),
            "s200-cleanup-fault".into(),
            "127.0.0.1:9".parse().unwrap(),
            Zeroizing::new("unused-capability".to_string()),
            4,
        )
        .unwrap();
        FAIL_NEXT_CLEANUP_REPORT.store(true, Ordering::Release);
        let error = runtime.stop().unwrap_err().to_string();
        assert!(
            error.contains("injected direct cleanup uncertainty"),
            "{error}"
        );
        assert!(error.contains("lost: 0"), "{error}");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = [0; 4];
            stream.read_exact(&mut bytes).unwrap();
            assert_eq!(&bytes, b"ping");
            stream.write_all(b"pong").unwrap();
        });
        let mut client = TcpStream::connect_timeout(&address, Duration::from_secs(2)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client.write_all(b"ping").unwrap();
        let mut reply = [0; 4];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(&reply, b"pong");
        server.join().unwrap();
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_driver_open_and_receive_failures_release_handle() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        for stage in ["open", "receive"] {
            if stage == "open" {
                super::super::windivert::fail_next_network_open();
            } else {
                super::super::windivert::fail_next_network_recv();
            }
            let started = DirectPacketRuntime::start(
                Path::new(&root),
                NativeObservationContext::default(),
                format!("s200-driver-{stage}-fault"),
                "127.0.0.1:9".parse().unwrap(),
                Zeroizing::new("unused-capability".to_string()),
                4,
            );
            let error = match started {
                Ok(mut runtime) => runtime.stop().unwrap_err().to_string(),
                Err(error) => error.to_string(),
            };
            assert!(
                error.contains(&format!("network {stage} failure")),
                "{error}"
            );

            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                stream.write_all(&byte).unwrap();
            });
            let mut client = TcpStream::connect_timeout(&address, Duration::from_secs(2)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            client.write_all(b"x").unwrap();
            let mut reply = [0];
            client.read_exact(&mut reply).unwrap();
            assert_eq!(&reply, b"x");
            server.join().unwrap();
        }
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_send_failure_releases_broad_handle_and_counts_packet() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
        origin
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client.connect(origin.local_addr().unwrap()).unwrap();
        client
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let tuple = DirectTuple {
            transport: Proto::Udp,
            local: client.local_addr().unwrap(),
            remote: origin.local_addr().unwrap(),
        };
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            NativeObservationContext::default(),
            "s200-send-failure".into(),
            "127.0.0.1:9".parse().unwrap(),
            Zeroizing::new("unused-capability".to_string()),
            4,
        )
        .unwrap();
        super::super::windivert::fail_next_send_for(tuple);
        client.send(b"drop-this-datagram").unwrap();
        let mut reply = [0u8; 64];
        assert!(client.recv(&mut reply).is_err());
        let error = runtime.stop().unwrap_err().to_string();
        assert!(error.contains("injected WinDivert send failure"), "{error}");
        let lost = error
            .split("lost: ")
            .nth(1)
            .and_then(|tail| tail.split(',').next())
            .and_then(|count| count.parse::<u64>().ok())
            .expect("loss count in direct route cleanup error");
        assert!(lost >= 1, "{error}");
        assert!(error.contains("in_flight: 0"), "{error}");
        client.send(b"after-handle-release").unwrap();
        let mut received = [0u8; 64];
        let (length, from) = origin.recv_from(&mut received).unwrap();
        assert_eq!(&received[..length], b"after-handle-release");
        origin.send_to(b"reply", from).unwrap();
        assert_eq!(client.recv(&mut reply).unwrap(), 5);
        assert_eq!(&reply[..5], b"reply");
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_proxy_connect_error_reinjects_selected_first_packet() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = origin.accept().unwrap();
            let mut body = Vec::new();
            stream.read_to_end(&mut body).unwrap();
            assert_eq!(body, b"runtime-client-request");
            stream.write_all(b"runtime-origin-reply").unwrap();
        });
        let dead_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_proxy = dead_listener.local_addr().unwrap();
        drop(dead_listener);
        let mut child = spawn_client("tcp", original);
        let context = client_context(child.id());
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-proxy-refusal".into(),
            dead_proxy,
            Zeroizing::new("unusable-capability".to_string()),
            4,
        )
        .unwrap();
        release_client(&mut child);
        finish_client(child);
        let stats = runtime.stop().unwrap();
        server.join().unwrap();
        // A refused local TCP connect is an OS I/O failure, not a SOCKS reply
        // from a running proxy. The original packet still returns to Windows.
        assert_eq!(stats.proxy_refused, 0);
        assert_eq!(stats.packets.consumed, 0);
        assert_eq!(stats.packets.lost, 0);
        assert!(stats.packets.reinjected > 0);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_delayed_socks_helper_keeps_packet_worker_live() {
        struct OwnedClient(Option<Child>);
        impl Drop for OwnedClient {
            fn drop(&mut self) {
                if let Some(mut child) = self.0.take() {
                    if child.try_wait().is_ok_and(|status| status.is_none()) {
                        let _ = child.kill();
                    }
                    let _ = child.wait();
                }
            }
        }
        struct OwnedRoutePath(std::path::PathBuf);
        impl Drop for OwnedRoutePath {
            fn drop(&mut self) {
                if self.0.exists() {
                    if let Err(error) = std::fs::remove_file(&self.0) {
                        eprintln!("S211 route fixture cleanup failed: {error}");
                    }
                }
            }
        }
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = origin.accept().unwrap();
            let mut body = Vec::new();
            stream.read_to_end(&mut body).unwrap();
            assert_eq!(body, b"runtime-client-request");
            stream.write_all(b"runtime-origin-reply").unwrap();
        });
        let sentinel = TcpListener::bind("127.0.0.1:0").unwrap();
        let sentinel_address = sentinel.local_addr().unwrap();
        let sentinel_server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = sentinel.accept().unwrap();
                let mut request = [0; 8];
                stream.read_exact(&mut request).unwrap();
                assert_eq!(&request, b"sentinel");
                stream.write_all(b"sentinel").unwrap();
            }
        });
        let exchange_sentinel = || {
            let started = Instant::now();
            let mut stream = TcpStream::connect(sentinel_address).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream.write_all(b"sentinel").unwrap();
            let mut response = [0; 8];
            stream.read_exact(&mut response).unwrap();
            assert_eq!(&response, b"sentinel");
            started.elapsed()
        };
        let baseline = exchange_sentinel();

        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy_address = proxy.local_addr().unwrap();
        let (waiting_tx, waiting_rx) = mpsc::channel();
        let proxy_server = thread::spawn(move || {
            let (mut stream, _) = proxy.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut method = [0; 3];
            stream.read_exact(&mut method).unwrap();
            assert_eq!(method, [5, 1, 2]);
            stream.write_all(&[5, 2]).unwrap();
            let mut auth = [0; 2];
            stream.read_exact(&mut auth).unwrap();
            let mut identity = vec![0; auth[1] as usize];
            stream.read_exact(&mut identity).unwrap();
            let mut password_length = [0; 1];
            stream.read_exact(&mut password_length).unwrap();
            let mut password = vec![0; password_length[0] as usize];
            stream.read_exact(&mut password).unwrap();
            assert_eq!(password, b"s211-test-capability");
            stream.write_all(&[1, 0]).unwrap();
            let mut request = [0; 10];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(&request[..4], &[5, 1, 0, 1]);
            waiting_tx.send(()).unwrap();
            thread::sleep(Duration::from_millis(80));
            let mut upstream = TcpStream::connect(original).unwrap();
            stream
                .write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 80])
                .unwrap();
            let mut body = Vec::new();
            stream.read_to_end(&mut body).unwrap();
            upstream.write_all(&body).unwrap();
            upstream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut reply = Vec::new();
            upstream.read_to_end(&mut reply).unwrap();
            stream.write_all(&reply).unwrap();
        });

        let mut child = OwnedClient(Some(spawn_client("tcp", original)));
        let context = client_context(child.0.as_ref().unwrap().id());
        let route_path = OwnedRoutePath(std::env::temp_dir().join(format!(
            "s211-delayed-helper-{}-{}.jsonl",
            std::process::id(),
            unix_now_ns().unwrap()
        )));
        let mut runtime = DirectPacketRuntime::start_with_artifact(
            Path::new(&root),
            context,
            "s211-delayed-helper".into(),
            proxy_address,
            Zeroizing::new("s211-test-capability".to_string()),
            4,
            Some(&route_path.0),
        )
        .unwrap();
        release_client(child.0.as_mut().unwrap());
        waiting_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let active = exchange_sentinel();
        assert!(active <= Duration::from_millis(50).max(baseline * 4));
        finish_client(child.0.take().unwrap());
        let stats = runtime.stop().unwrap();
        proxy_server.join().unwrap();
        server.join().unwrap();
        sentinel_server.join().unwrap();
        assert!(!stats.watchdog_released);
        assert_eq!(stats.proxy_refused, 0);
        assert_eq!(stats.packets.lost, 0);
        assert!(stats.packets.consumed > 0);
        let route = super::super::route_events::read_route_prefix(&route_path.0).unwrap();
        assert!(route.complete());
        let helper = route
            .events
            .iter()
            .find(|event| event.kind == "helper-outcome")
            .expect("helper outcome");
        assert_eq!(helper.fields["outcome"], "success");
        assert_eq!(helper.fields["stage"], "complete");
        assert!(helper.fields["elapsed_ms"].as_u64().unwrap() >= 60);
        std::fs::remove_file(&route_path.0).unwrap();
        assert!(!route_path.0.exists());
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_proxy_auth_failure_reinjects_selected_first_packet() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = origin.accept().unwrap();
            let mut body = Vec::new();
            stream.read_to_end(&mut body).unwrap();
            assert_eq!(body, b"runtime-client-request");
            stream.write_all(b"runtime-origin-reply").unwrap();
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(2),
        )
        .unwrap()
        .with_session_id("s200-proxy-auth-refusal")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut lease = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .start(Duration::from_secs(2))
            .unwrap();
        let mut child = spawn_client("tcp", original);
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            client_context(child.id()),
            "s200-proxy-auth-refusal".into(),
            lease.endpoint(),
            Zeroizing::new("wrong-capability".to_string()),
            4,
        )
        .unwrap();
        release_client(&mut child);
        finish_client(child);
        let stats = runtime.stop().unwrap();
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(2));
        assert!(report.observation.protocol.socks_auth_refused > 0);
        assert!(stats.proxy_refused > 0);
        assert_eq!(stats.packets.consumed, 0);
        assert_eq!(stats.packets.lost, 0);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_routes_exact_generation() {
        live_tcp_case(IpAddr::V4(Ipv4Addr::LOCALHOST), false);
        live_tcp_case(IpAddr::V6(Ipv6Addr::LOCALHOST), false);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_writer_retirement_preserves_direct_route_and_counts_loss() {
        live_tcp_case(IpAddr::V4(Ipv4Addr::LOCALHOST), true);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_routes_two_parallel_clients() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = origin.accept().unwrap();
                let mut body = Vec::new();
                stream.read_to_end(&mut body).unwrap();
                assert_eq!(body, b"runtime-client-request");
                stream.write_all(b"runtime-origin-reply").unwrap();
            }
        });
        let mut first = spawn_client("tcp", original);
        let mut second = spawn_client("tcp", original);
        let context = paired_client_context(first.id(), second.id());
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(3),
        )
        .unwrap()
        .with_session_id("s200-direct-parallel")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut lease = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_inspection_decider(Arc::new(ExactGameDecider::new(
                context.clone(),
                vec!["client".into()],
            )))
            .start(Duration::from_secs(3))
            .unwrap();
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-parallel".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        release_client(&mut first);
        release_client(&mut second);
        finish_client(first);
        finish_client(second);
        let stats = runtime.stop().unwrap();
        assert!(stats.packets.consumed > 0);
        assert_eq!(stats.packets.lost, 0);
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(3));
        assert_eq!(report.observation.protocol.socks_connect_succeeded, 2);
    }

    fn live_tcp_case(ip: IpAddr, writer_retired: bool) {
        let origin = TcpListener::bind(SocketAddr::new(ip, 0)).unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            let mut selected = 0;
            let mut unrelated = 0;
            for _ in 0..4 {
                let (mut stream, _) = origin.accept().unwrap();
                let mut body = Vec::new();
                stream.read_to_end(&mut body).unwrap();
                if body == b"runtime-client-request" {
                    selected += 1;
                    stream.write_all(b"runtime-origin-reply").unwrap();
                } else {
                    assert_eq!(body, b"sentinel-request");
                    unrelated += 1;
                    stream.write_all(b"sentinel-reply").unwrap();
                }
            }
            (selected, unrelated)
        });
        live_tcp_case_for_origin(ip, original, writer_retired, None);
        assert_eq!(server.join().unwrap(), (1, 3));
    }

    fn live_tcp_case_for_origin(
        ip: IpAddr,
        original: SocketAddr,
        writer_retired: bool,
        route_path: Option<&Path>,
    ) {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let mut before = spawn_client("sentinel-tcp", original);
        release_client(&mut before);
        finish_client(before);
        let mut child = spawn_client(
            if original.ip().is_loopback() {
                "tcp"
            } else {
                "tcp-roundtrip"
            },
            original,
        );
        let context = client_context(child.id());
        let proof_context = context.clone();
        let config =
            NativeProxyConfig::new(SocketAddr::new(ip, 0), 4, 1024, Duration::from_secs(2))
                .unwrap()
                .with_session_id("s200-direct-runtime")
                .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_inspection_decider(Arc::new(ExactGameDecider::new(
                context.clone(),
                vec!["client".into()],
            )));
        if writer_retired {
            proxy = proxy.with_application_event_sink(Arc::new(RetiredApplicationSink::default()));
        }
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let mut runtime = DirectPacketRuntime::start_with_artifact(
            Path::new(&root),
            context,
            "s200-direct-runtime".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
            route_path,
        )
        .unwrap();
        let mut during = spawn_client("sentinel-tcp", original);
        release_client(&mut during);
        let selected_started = Instant::now();
        release_client(&mut child);
        finish_client(during);
        let child_output = child.wait_with_output().unwrap();
        let selected_elapsed_ms = selected_started.elapsed().as_millis();
        let ack_deadline = Instant::now() + Duration::from_millis(750);
        let mut returned_data_acknowledged = false;
        while !original.ip().is_loopback() && Instant::now() < ack_deadline {
            returned_data_acknowledged = proof_context
                .direct_helper_flows_for_test()
                .iter()
                .filter(|flow| flow.tuple.remote == original)
                .any(|flow| {
                    proof_context
                        .direct_proof_for(flow)
                        .is_ok_and(|proof| proof.lock().unwrap().returned_data_acknowledged)
                });
            if returned_data_acknowledged {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        let stop_result = runtime.stop();
        eprintln!("direct runtime probe: family={ip}, stop={stop_result:?}");
        eprintln!("selected route transaction elapsed_ms={selected_elapsed_ms}");
        let stats = stop_result.unwrap();
        assert!(stats.packets.consumed > 0, "selected flow was not routed");
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(stats.synthetic_reply_lost, 0);
        assert!(stats.pre_receive_loss_unmeasurable);
        let mut after = spawn_client("sentinel-tcp", original);
        release_client(&mut after);
        finish_client(after);
        let mut route_kinds = Vec::new();
        if let Some(path) = route_path {
            let prefix = crate::deep_capture::route_events::read_route_prefix(path).unwrap();
            assert!(prefix.complete(), "route stream incomplete");
            route_kinds = prefix
                .events
                .iter()
                .map(|event| event.kind.clone())
                .collect();
        }
        let proxy_report = lease.cleanup(Duration::from_secs(2));
        assert!(
            child_output.status.success(),
            "child failed: {}; stats={stats:?}; proxy={:?}; route={route_kinds:?}",
            String::from_utf8_lossy(&child_output.stderr),
            proxy_report.observation.protocol,
        );
        if !original.ip().is_loopback() {
            assert!(
                returned_data_acknowledged,
                "client received the reply but the route did not observe its data ACK: {:?}",
                proof_context
                    .direct_helper_flows_for_test()
                    .iter()
                    .filter(|flow| flow.tuple.remote == original)
                    .filter_map(|flow| proof_context.direct_proof_for(flow).ok())
                    .map(|proof| proof.lock().unwrap().clone())
                    .collect::<Vec<_>>()
            );
        }
        if route_path.is_some() {
            assert!(
                route_kinds
                    .iter()
                    .any(|kind| kind == "returned-data-acknowledged"),
                "route kinds: {route_kinds:?}"
            );
        }
        assert_eq!(
            proxy_report.observation.protocol.socks_connect_succeeded,
            1,
            "direct route: protocol={:?}, route={stats:?}, owners={:?}",
            proxy_report.observation.protocol,
            proof_context.direct_helper_flows_for_test()
        );
        assert_eq!(
            proxy_report.observation.protocol.socks_tcp_opaque
                + proxy_report.observation.protocol.socks_tcp_undetermined,
            1
        );
        assert_eq!(proxy_report.observation.protocol.generic_streams_unknown, 1);
        assert_eq!(proxy_report.observation.protocol.generic_streams_plain, 0);
        if writer_retired {
            assert!(proxy_report.observation.protocol.application_events_dropped > 0);
        }
    }

    #[test]
    #[ignore = "requires elevated WinDivert and controlled non-loopback IPv4/IPv6 peers"]
    fn controlled_direct_runtime_routes_external_peers() {
        let ipv4 = std::env::var("FRAGCAP_S204_EXTERNAL_IPV4")
            .expect("FRAGCAP_S204_EXTERNAL_IPV4 is required for controlled acceptance");
        let ipv6 = std::env::var("FRAGCAP_S204_EXTERNAL_IPV6")
            .expect("FRAGCAP_S204_EXTERNAL_IPV6 is required for controlled acceptance");
        for (variable, ip) in [
            (
                "FRAGCAP_S204_EXTERNAL_IPV4",
                IpAddr::V4(Ipv4Addr::LOCALHOST),
            ),
            (
                "FRAGCAP_S204_EXTERNAL_IPV6",
                IpAddr::V6(Ipv6Addr::LOCALHOST),
            ),
        ] {
            let original: SocketAddr = match variable {
                "FRAGCAP_S204_EXTERNAL_IPV4" => &ipv4,
                _ => &ipv6,
            }
            .parse()
            .unwrap();
            assert!(
                !original.ip().is_loopback(),
                "external peer must not be loopback"
            );
            assert_eq!(original.is_ipv4(), ip.is_ipv4(), "peer family mismatch");
            for diagnostics in [true, false] {
                let evidence = tempfile::tempdir().unwrap();
                let route = evidence.path().join("route.jsonl");
                live_tcp_case_for_origin(
                    ip,
                    original,
                    false,
                    diagnostics.then_some(route.as_path()),
                );
            }
        }
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_classifies_http_tls_and_opaque() {
        live_inspected_tcp_case(false, false, false);
        live_inspected_tcp_case(true, false, false);
        live_inspected_tcp_case(true, false, true);
        live_inspected_tcp_case(true, true, false);
    }

    fn live_inspected_tcp_case(tls: bool, opaque: bool, hostname: bool) {
        use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

        eprintln!("direct TCP protocol case: tls={tls}, opaque={opaque}, hostname={hostname}");

        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let (origin_config, origin_root) = if tls {
            let generated = rcgen::generate_simple_self_signed(vec![if hostname {
                "localhost"
            } else {
                "127.0.0.1"
            }
            .to_string()])
            .unwrap();
            let certificate = CertificateDer::from(generated.cert);
            let key = PrivatePkcs8KeyDer::from(generated.signing_key.serialize_der());
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            let mut config = rustls::ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(vec![certificate.clone()], key.into())
                .unwrap();
            config.alpn_protocols = vec![b"http/1.1".to_vec()];
            (Some(Arc::new(config)), Some(certificate))
        } else {
            (None, None)
        };
        let server = thread::spawn(move || {
            origin.set_nonblocking(true).unwrap();
            let accept_deadline = Instant::now() + Duration::from_secs(5);
            let stream = loop {
                match origin.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < accept_deadline, "origin accept timed out");
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("origin accept failed: {error}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut stream: Box<dyn ReadWrite> = match origin_config {
                Some(config) => Box::new(rustls::StreamOwned::new(
                    rustls::ServerConnection::new(config).unwrap(),
                    stream,
                )),
                None => Box::new(stream),
            };
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                head.push(byte[0]);
            }
            assert!(head.starts_with(if tls {
                b"GET /direct-tls HTTP/1.1\r\n"
            } else {
                b"GET /direct HTTP/1.1\r\n"
            }));
            if tls {
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nTLS-DIRECT").unwrap();
            } else {
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nDIRECT",
                    )
                    .unwrap();
            }
            stream.flush().unwrap();
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(3),
        )
        .unwrap()
        .with_session_id("s200-direct-inspected-runtime")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let authority = Arc::new(
            fragcap_proxy::SessionCertificateAuthority::generate(
                1,
                SystemTime::now(),
                Duration::from_secs(3600),
            )
            .unwrap(),
        );
        let mut child = spawn_client_with_ca(
            if tls {
                if hostname {
                    "tls-dns"
                } else {
                    "tls"
                }
            } else {
                "http"
            },
            original,
            if opaque {
                origin_root.as_ref().map(|root| root.as_ref())
            } else {
                tls.then_some(authority.der().as_ref())
            },
        );
        let context = client_context(child.id());
        let decider: Arc<dyn InspectionDecider> = if opaque {
            Arc::new(UninspectedDirectOwner)
        } else {
            Arc::new(ExactGameDecider::new(
                context.clone(),
                vec!["client".into()],
            ))
        };
        let mut proxy = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_prepared_authority(authority)
            .with_inspection_decider(decider);
        if let Some(certificate) = origin_root {
            let mut roots = rustls::RootCertStore::empty();
            roots.add(certificate).unwrap();
            proxy = proxy.with_tls_client_config(
                fragcap_proxy::tls_client_config_with_roots(roots).unwrap(),
            );
        }
        let mut lease = proxy.start(Duration::from_secs(3)).unwrap();
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-inspected-runtime".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        release_client(&mut child);
        drop(child.stdin.take());
        let child_output = child.wait_with_output().unwrap();
        let stats = runtime.stop().unwrap();
        eprintln!("S211 classified route stats: {stats:?}");
        let report = lease.cleanup(Duration::from_secs(3));
        eprintln!(
            "S211 classified proxy protocol: {:?}",
            report.observation.protocol
        );
        eprintln!(
            "S211 classified proxy connections: {:?}",
            report.observation.connection_diagnostics
        );
        let origin_error = server.join().err().map(|panic| {
            panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    panic
                        .downcast_ref::<&str>()
                        .map(|message| (*message).into())
                })
                .unwrap_or_else(|| "non-string panic".into())
        });
        assert!(
            child_output.status.success(),
            "child failed: {}; origin error={origin_error:?}; direct received={}, consumed={}, reinjected={}, lost={}, bypass={:?}; proxy SOCKS connected={}, TLS={}, opaque={}, undetermined={}, client TLS={}, upstream TLS={}",
            String::from_utf8_lossy(&child_output.stderr),
            stats.packets.received,
            stats.packets.consumed,
            stats.packets.reinjected,
            stats.packets.lost,
            stats.bypass_reasons,
            report.observation.protocol.socks_connect_succeeded,
            report.observation.protocol.socks_tls,
            report.observation.protocol.socks_tcp_opaque,
            report.observation.protocol.socks_tcp_undetermined,
            report.observation.protocol.client_tls_completed,
            report.observation.protocol.upstream_tls_completed,
        );
        assert!(origin_error.is_none(), "origin failed: {origin_error:?}");
        assert!(stats.packets.consumed > 0);
        assert_eq!(
            stats.packets.lost,
            stats
                .bypass_reasons
                .get("selected-tuple-post-close-control-unattributed")
                .copied()
                .unwrap_or(0),
            "only exact post-CLOSE header-only control tails may be lost: {:?}",
            stats.bypass_reasons
        );
        assert_eq!(report.observation.protocol.socks_connect_succeeded, 1);
        assert_eq!(
            report.observation.protocol.http1_exchanges_completed,
            u64::from(!opaque)
        );
        if opaque {
            assert_eq!(report.observation.protocol.generic_streams_tls_opaque, 1);
            assert_eq!(report.observation.protocol.client_tls_completed, 0);
            assert_eq!(report.observation.protocol.upstream_tls_completed, 0);
        } else if tls {
            assert_eq!(report.observation.protocol.client_tls_completed, 1);
            assert_eq!(report.observation.protocol.upstream_tls_completed, 1);
        }
    }

    trait ReadWrite: Read + Write {}
    impl<T: Read + Write> ReadWrite for T {}

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_inspects_http3_from_udp_socket() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let limits = ProtocolLimits {
            max_concurrent_streams: 4,
            max_socks_udp_datagram_bytes: 4096,
            idle_timeout: Duration::from_secs(5),
            ..ProtocolLimits::default()
        };
        let origin_ca = fragcap_proxy::SessionCertificateAuthority::generate(
            91,
            SystemTime::now(),
            Duration::from_secs(3600),
        )
        .unwrap();
        let mut cache = LeafCache::new(
            NonZeroUsize::new(4).unwrap(),
            NonZeroUsize::new(128 * 1024).unwrap(),
            Duration::from_secs(1800),
            91,
        )
        .unwrap();
        let authority = DestinationAuthority::parse("127.0.0.1:443").unwrap();
        let origin_config = fragcap_proxy::build_quic_server_config(
            &authority, &origin_ca, &mut cache, None, &limits,
        )
        .unwrap();
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (original, server_task) = async_runtime.block_on(async move {
            let server = Endpoint::server(origin_config, "127.0.0.1:0".parse().unwrap()).unwrap();
            let original = server.local_addr().unwrap();
            let task = tokio::spawn(async move {
                let incoming = tokio::time::timeout(Duration::from_secs(8), server.accept())
                    .await
                    .unwrap()
                    .unwrap();
                eprintln!("direct HTTP3 origin accepted QUIC");
                let connection = incoming.await.unwrap();
                let mut h3 = h3::server::Connection::new(Connection::new(connection))
                    .await
                    .unwrap();
                let resolver = h3.accept().await.unwrap().unwrap();
                let (request, mut stream) = resolver.resolve_request().await.unwrap();
                eprintln!(
                    "direct HTTP3 origin received {} {}",
                    request.method(),
                    request.uri()
                );
                assert_eq!(request.method(), "GET");
                assert_eq!(request.uri().path(), "/direct-quic");
                stream
                    .send_response(Response::builder().status(200).body(()).unwrap())
                    .await
                    .unwrap();
                stream
                    .send_data(bytes::Bytes::from_static(b"H3-DIRECT"))
                    .await
                    .unwrap();
                stream.finish().await.unwrap();
                let _ = h3.accept().await;
                server.wait_idle().await;
            });
            (original, task)
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(3),
        )
        .unwrap()
        .with_session_id("s200-direct-http3")
        .unwrap()
        .with_protocol_limits(limits);
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut upstream_roots = rustls::RootCertStore::empty();
        upstream_roots.add(origin_ca.der().clone()).unwrap();
        let session_ca = Arc::new(
            fragcap_proxy::SessionCertificateAuthority::generate(
                92,
                SystemTime::now(),
                Duration::from_secs(3600),
            )
            .unwrap(),
        );
        let mut child = spawn_client_with_ca("h3", original, Some(session_ca.der().as_ref()));
        let context = client_context(child.id());
        let mut proxy = NativeProxyBackend::new(config)
            .with_destination_policy(policy)
            .with_prepared_authority(session_ca)
            .with_tls_client_config(
                fragcap_proxy::tls_client_config_with_roots(upstream_roots).unwrap(),
            )
            .with_inspection_decider(Arc::new(ExactGameDecider::new(
                context.clone(),
                vec!["client".into()],
            )))
            .start(Duration::from_secs(3))
            .unwrap();
        let mut route = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-http3".into(),
            proxy.endpoint(),
            proxy.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        release_client(&mut child);
        drop(child.stdin.take());
        let output = child.wait_with_output().unwrap();
        if !output.status.success() {
            let stopped = route.stop();
            let report = proxy.cleanup(Duration::from_secs(3));
            eprintln!(
                "direct HTTP3 failure: route={stopped:?}, protocol={:?}, diagnostics={:?}, origin_finished={}",
                report.observation.protocol, report.observation.connection_diagnostics, server_task.is_finished()
            );
            panic!("child failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        async_runtime.block_on(server_task).unwrap();
        let stats = route.stop().unwrap();
        assert!(stats.packets.consumed > 0);
        assert_eq!(stats.packets.lost, 0);
        let report = proxy.cleanup(Duration::from_secs(3));
        assert_eq!(report.observation.protocol.socks_udp_associate_succeeded, 1);
        assert_eq!(report.observation.protocol.quic_pairs_started, 1);
        assert_eq!(report.observation.protocol.http3_streams_completed, 1);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_routes_udp_datagram() {
        live_udp_case(IpAddr::V4(Ipv4Addr::LOCALHOST));
        live_udp_case(IpAddr::V6(Ipv6Addr::LOCALHOST));
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_selected_udp_burst_reuses_one_association() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
        origin
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                let mut body = [0u8; 64];
                let (length, peer) = origin.recv_from(&mut body).unwrap();
                origin.send_to(&body[..length], peer).unwrap();
            }
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(3),
        )
        .unwrap()
        .with_session_id("s200-direct-udp-burst")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(3)).unwrap();
        let mut child = spawn_client("udp-burst", original);
        let context = client_context(child.id());
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-udp-burst".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        release_client(&mut child);
        finish_client(child);
        let stats = runtime.stop().unwrap();
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(3));
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(report.observation.protocol.socks_udp_associate_succeeded, 1);
        assert_eq!(report.observation.protocol.socks_udp_client_forwarded, 3);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_selected_udp_endpoint_reuse_starts_new_association() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = UdpSocket::bind("127.0.0.1:0").unwrap();
        origin
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let original = origin.local_addr().unwrap();
        let server = thread::spawn(move || {
            for (request, reply) in [
                (b"first-socket".as_slice(), b"first-reply".as_slice()),
                (b"second-socket".as_slice(), b"second-reply".as_slice()),
            ] {
                let mut body = [0u8; 128];
                let (length, peer) = origin.recv_from(&mut body).unwrap();
                assert_eq!(&body[..length], request);
                origin.send_to(reply, peer).unwrap();
            }
        });
        let config = NativeProxyConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            4,
            1024,
            Duration::from_secs(3),
        )
        .unwrap()
        .with_session_id("s200-direct-udp-reuse")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(3)).unwrap();
        let mut child = spawn_client("udp-reuse", original);
        let context = client_context(child.id());
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context.clone(),
            "s200-direct-udp-reuse".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        release_client(&mut child);
        finish_client(child);
        let stats = runtime.stop().unwrap();
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(3));
        let owners = context.direct_helper_flows_for_test();
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(
            report.observation.protocol.socks_udp_associate_succeeded, 2,
            "route={stats:?}; owners={owners:?}"
        );
        let endpoint_ids: BTreeSet<_> = owners
            .iter()
            .filter(|owner| owner.tuple.transport == Proto::Udp)
            .map(|owner| owner.endpoint_id)
            .collect();
        assert_eq!(endpoint_ids.len(), 2);
    }

    fn live_udp_case(ip: IpAddr) {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = UdpSocket::bind(SocketAddr::new(ip, 0)).unwrap();
        origin
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let original = origin.local_addr().unwrap();
        let sentinel = UdpSocket::bind(SocketAddr::new(ip, 0)).unwrap();
        sentinel
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let sentinel_address = sentinel.local_addr().unwrap();
        let sentinel_server = thread::spawn(move || {
            for _ in 0..3 {
                let mut body = [0u8; 128];
                let (length, peer) = sentinel.recv_from(&mut body).unwrap();
                assert_eq!(&body[..length], b"sentinel-udp-request");
                sentinel.send_to(b"sentinel-udp-reply", peer).unwrap();
            }
        });
        let mut before = spawn_client("sentinel-udp", sentinel_address);
        release_client(&mut before);
        finish_client(before);
        let server = thread::spawn(move || {
            let mut body = [0u8; 128];
            let (length, peer) = origin.recv_from(&mut body).unwrap();
            assert_eq!(&body[..length], b"runtime-udp-request");
            origin.send_to(b"runtime-udp-reply", peer).unwrap();
        });
        let config =
            NativeProxyConfig::new(SocketAddr::new(ip, 0), 4, 1024, Duration::from_secs(2))
                .unwrap()
                .with_session_id("s200-direct-udp-runtime")
                .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let mut child = spawn_client("udp", original);
        let context = client_context(child.id());
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-udp-runtime".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        let mut during = spawn_client("sentinel-udp", sentinel_address);
        release_client(&mut during);
        finish_client(during);
        release_client(&mut child);
        let output = child.wait_with_output().unwrap();
        let stop_result = runtime.stop();
        let proxy_report = lease.cleanup(Duration::from_secs(2));
        eprintln!(
            "direct UDP runtime probe: family={ip}, child={:?}, stderr={}, stop={stop_result:?}, proxy={:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            proxy_report.observation.protocol
        );
        assert!(output.status.success());
        let stats = stop_result.unwrap();
        assert!(
            stats.packets.consumed > 0,
            "selected UDP flow was not routed"
        );
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(stats.synthetic_reply_lost, 0);
        let mut after = spawn_client("sentinel-udp", sentinel_address);
        release_client(&mut after);
        finish_client(after);
        sentinel_server.join().unwrap();
        server.join().unwrap();
        assert_eq!(
            proxy_report
                .observation
                .protocol
                .socks_udp_associate_succeeded,
            1
        );
        assert_eq!(proxy_report.observation.accepted_connections, 1);
    }
}
