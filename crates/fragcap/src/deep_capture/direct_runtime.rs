// SPDX-License-Identifier: Apache-2.0

//! Finite, prearmed packet route for exact managed sockets.

use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
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
    direct_bridge::{connect_authenticated_socks5_with_hook, DirectTcpBridge},
    direct_observer::{DirectSocketObserver, SocketObserverStats},
    direct_route::{
        CandidateBuffer, CandidateDecision, DirectPacketAccounting, DirectReceiptLedger,
        DirectSocketEvent, DirectTuple, PacketDisposition, RouteReceiptId,
    },
    direct_udp::{build_udp_reply, DirectUdpAssociation},
    windivert::{sample_qpc_anchor, Address, Api, Handle, NETWORK_LAYER},
    NativeObservationContext,
};

const PACKET_QUEUE: usize = 256;
const CANDIDATE_BYTES: usize = 4 * 1024 * 1024;
const CANDIDATE_WAIT_NS: u64 = 10_000_000;
const SOCKET_EVENTS: usize = 4096;
const REFUSED_FLOWS: usize = 4096;
const PACKET_FILTER: &str = "tcp or udp";
const WORKER_STALL_THRESHOLD_MS: u64 = 90;
const PROXY_CONNECT_TIMEOUT: Duration = Duration::from_millis(50);

#[cfg(test)]
static STALL_NEXT_CANDIDATE: AtomicBool = AtomicBool::new(false);

fn unix_now_ns() -> io::Result<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| io::Error::other("system clock before Unix epoch"))?;
    u64::try_from(duration.as_nanos())
        .map_err(|_| io::Error::other("system clock beyond route timestamp range"))
}

enum WorkPacket {
    Candidate(super::direct_route::DirectPacketHead, Vec<u8>, Address),
    Selected(DirectTuple, Vec<u8>, Address),
}

#[derive(Clone, Debug, Default)]
pub struct DirectRuntimeStats {
    pub packets: DirectPacketAccounting,
    pub socket_events: SocketObserverStats,
    pub admission_busy_bypassed: u64,
    pub candidate_capacity_bypassed: u64,
    pub bypass_reasons: BTreeMap<&'static str, u64>,
    pub proxy_refused: u64,
    pub synthetic_reply_lost: u64,
    pub event_history_overflow: bool,
    pub pre_receive_loss_unmeasurable: bool,
    pub watchdog_released: bool,
    pub watchdog_release_after_progress_ms: u64,
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
    bypass_reasons: Mutex<BTreeMap<&'static str, u64>>,
    proxy_refused: AtomicU64,
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
            bypass_reasons: Mutex::new(BTreeMap::new()),
            proxy_refused: AtomicU64::new(0),
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

fn mark_refused_flow(refused: &Mutex<BTreeSet<DirectTuple>>, tuple: DirectTuple) -> bool {
    let mut flows = refused.lock().expect("refused route lock");
    if flows.len() >= REFUSED_FLOWS && !flows.contains(&tuple) {
        return false;
    }
    flows.insert(tuple);
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

fn inject_replies(
    handle: &Handle,
    stats: &SharedStats,
    replies: Vec<Vec<u8>>,
    address: Address,
) -> io::Result<()> {
    let address = address.response_address();
    for reply in replies {
        if !matches!(handle.send(&reply, &address), Ok(sent) if sent == reply.len()) {
            stats.synthetic_reply_lost.fetch_add(1, Ordering::Relaxed);
            return Err(io::Error::other("synthetic TCP reply injection failed"));
        }
    }
    Ok(())
}

struct ReceiveRuntimeInputs {
    handle: Arc<Handle>,
    sender: SyncSender<WorkPacket>,
    observer: Arc<Mutex<DirectSocketObserver>>,
    selected: Arc<Mutex<BTreeSet<DirectTuple>>>,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    refused: Arc<Mutex<BTreeSet<DirectTuple>>>,
    connecting: Arc<AtomicBool>,
    stats: Arc<SharedStats>,
}

fn receive_packets(inputs: ReceiveRuntimeInputs) -> io::Result<()> {
    let ReceiveRuntimeInputs {
        handle,
        sender,
        observer,
        selected,
        excluded,
        refused,
        connecting,
        stats,
    } = inputs;
    let mut packet = [0u8; 65_535];
    loop {
        let mut address = Address::default();
        let length = match handle.recv(&mut packet, &mut address) {
            Ok(length) => length,
            Err(error) if error.raw_os_error() == Some(232) => return Ok(()),
            Err(error) => return Err(error),
        };
        packet_received(&stats)?;
        let bytes = &packet[..length];
        let head = raw_transport_head(bytes).ok();
        let selected_tuple = head.as_ref().and_then(|head| {
            let tuple = DirectTuple {
                transport: head.transport,
                local: head.source,
                remote: head.destination,
            };
            selected
                .lock()
                .expect("selected route lock")
                .contains(&tuple)
                .then_some(tuple)
        });
        if let Some(tuple) = selected_tuple {
            if sender
                .try_send(WorkPacket::Selected(tuple, bytes.to_vec(), address))
                .is_err()
            {
                packet_finished(&stats, PacketDisposition::Lost)?;
                let _ = handle.stop();
                return Err(io::Error::other("selected packet queue saturated"));
            }
            continue;
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
            refused
                .lock()
                .expect("refused route lock")
                .contains(&DirectTuple {
                    transport: head.transport,
                    local: head.source,
                    remote: head.destination,
                })
        });
        let busy_unowned = connecting.load(Ordering::Acquire)
            && head.as_ref().is_none_or(|head| {
                let tuple = DirectTuple {
                    transport: head.transport,
                    local: head.source,
                    remote: head.destination,
                };
                observer
                    .lock()
                    .expect("socket observer lock")
                    .current_owner_pid(tuple)
                    .is_none_or(|pid| pid == std::process::id())
            });
        if !is_candidate || busy_unowned || is_excluded || is_refused {
            if is_refused {
                stats.record_bypass("proxy-refused-flow");
            } else if is_candidate {
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
            if sender
                .try_send(WorkPacket::Candidate(candidate, bytes.to_vec(), address))
                .is_ok()
            {
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
    bridge: DirectTcpBridge,
    address: Address,
}

struct ActiveUdp {
    association: DirectUdpAssociation,
    address: Address,
}

struct ProcessRuntimeInputs {
    receiver: Receiver<WorkPacket>,
    handle: Arc<Handle>,
    observer: Arc<Mutex<DirectSocketObserver>>,
    context: NativeObservationContext,
    selected: Arc<Mutex<BTreeSet<DirectTuple>>>,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    refused: Arc<Mutex<BTreeSet<DirectTuple>>>,
    connecting: Arc<AtomicBool>,
    stats: Arc<SharedStats>,
    session_id: String,
    proxy: std::net::SocketAddr,
    password: Zeroizing<String>,
    max_flows: usize,
}

fn process_packets(inputs: ProcessRuntimeInputs) -> io::Result<()> {
    let ProcessRuntimeInputs {
        receiver,
        handle,
        observer,
        context,
        selected,
        excluded,
        refused,
        connecting,
        stats,
        session_id,
        proxy,
        password,
        max_flows,
    } = inputs;
    let started = Instant::now();
    let mut candidates = CandidateBuffer::new(PACKET_QUEUE, CANDIDATE_BYTES, CANDIDATE_WAIT_NS)
        .expect("fixed candidate bounds");
    let mut events = Vec::<DirectSocketEvent>::new();
    let mut active = BTreeMap::<DirectTuple, ActiveTcp>::new();
    let mut active_udp = BTreeMap::<DirectTuple, ActiveUdp>::new();
    let mut receipts = DirectReceiptLedger::new(session_id.clone(), max_flows);
    let mut issued_receipts = Vec::<RouteReceiptId>::new();
    let mut helper_id = 0u64;
    loop {
        stats.processor_progress();
        match receiver.recv_timeout(Duration::from_millis(1)) {
            Ok(WorkPacket::Candidate(head, packet, address)) => {
                #[cfg(test)]
                if STALL_NEXT_CANDIDATE.swap(false, Ordering::AcqRel) {
                    thread::sleep(Duration::from_millis(150));
                }
                if refused
                    .lock()
                    .expect("refused route lock")
                    .contains(&head.tuple)
                {
                    stats.record_bypass("proxy-refused-flow");
                    reinject(&handle, &stats, &packet, &address)?;
                    continue;
                }
                let now_ns = started.elapsed().as_nanos() as u64;
                if let Err(pending) = candidates.enqueue(head, packet, address, now_ns) {
                    stats
                        .candidate_capacity_bypassed
                        .fetch_add(1, Ordering::Relaxed);
                    reinject(&handle, &stats, &pending.packet, &pending.address)?;
                }
            }
            Ok(WorkPacket::Selected(tuple, packet, address)) => {
                if tuple.transport == Proto::Udp {
                    let Some(flow) = active_udp.get_mut(&tuple) else {
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    };
                    let datagram = match raw_udp_datagram(&packet) {
                        Ok(datagram)
                            if datagram.source == tuple.local
                                && datagram.destination == tuple.remote =>
                        {
                            datagram
                        }
                        _ => {
                            packet_finished(&stats, PacketDisposition::Lost)?;
                            return Err(io::Error::other(
                                "selected UDP packet invalid or tuple changed",
                            ));
                        }
                    };
                    if let Err(error) = flow.association.send(datagram.payload) {
                        packet_finished(&stats, PacketDisposition::Lost)?;
                        return Err(io::Error::other(format!("selected UDP relay: {error}")));
                    }
                    packet_finished(&stats, PacketDisposition::Consumed)?;
                    flow.address = address;
                    continue;
                }
                let Some(flow) = active.get_mut(&tuple) else {
                    reinject(&handle, &stats, &packet, &address)?;
                    continue;
                };
                let now_ms = started.elapsed().as_millis() as i64;
                let replies =
                    match flow
                        .bridge
                        .ingest_diverted(&packet, address.ip_checksum_valid(), now_ms)
                    {
                        Ok(replies) => replies,
                        Err(error) => {
                            packet_finished(&stats, PacketDisposition::Lost)?;
                            return Err(io::Error::other(format!("selected TCP bridge: {error}")));
                        }
                    };
                packet_finished(&stats, PacketDisposition::Consumed)?;
                flow.address = address;
                inject_replies(&handle, &stats, replies, address)?;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                for decision in candidates.drain() {
                    if let CandidateDecision::PassThrough {
                        reason,
                        packet,
                        address,
                    } = decision
                    {
                        stats.record_bypass(reason);
                        reinject(&handle, &stats, &packet, &address)?;
                    }
                }
                for receipt in issued_receipts {
                    receipts.retire(receipt).map_err(|error| {
                        io::Error::other(format!("retire route receipt: {error:?}"))
                    })?;
                }
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let mut new_events = observer.lock().expect("socket observer lock").take_events();
        // The native proxy and its upstream sockets share fragcap's PID.
        // They cannot be promoted to target ownership even if a test or
        // stale process snapshot happens to name this process as a client.
        new_events.retain(|event| event.pid != std::process::id());
        if events.len().saturating_add(new_events.len()) > SOCKET_EVENTS {
            stats.event_history_overflow.store(true, Ordering::Release);
        } else {
            events.extend(new_events);
        }
        let event_loss = observer.lock().expect("socket observer lock").stats();
        let evidence = if event_loss.capacity_lost == 0
            && event_loss.clock_lost == 0
            && !stats.event_history_overflow.load(Ordering::Acquire)
            && candidates.len() != 0
        {
            context.direct_process_evidence()
        } else {
            None
        };
        let decisions = candidates.resolve(
            evidence.as_ref(),
            &events,
            started.elapsed().as_nanos() as u64,
        );
        for decision in decisions {
            match decision {
                CandidateDecision::PassThrough {
                    reason,
                    packet,
                    address,
                } => {
                    stats.record_bypass(reason);
                    reinject(&handle, &stats, &packet, &address)?;
                }
                CandidateDecision::Selected {
                    flow,
                    packet,
                    address,
                } => {
                    if refused
                        .lock()
                        .expect("refused route lock")
                        .contains(&flow.tuple)
                    {
                        stats.record_bypass("proxy-refused-flow");
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
                    if active.len().saturating_add(active_udp.len()) >= max_flows {
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
                        let helper_open_earliest = unix_now_ns()?;
                        helper_id = helper_id.saturating_add(1);
                        let now = started.elapsed().as_nanos() as u64;
                        let receipt = receipts
                            .issue(flow.clone(), now.saturating_add(10_000_000_000))
                            .map_err(|error| {
                                io::Error::other(format!("issue UDP receipt: {error:?}"))
                            })?;
                        connecting.store(true, Ordering::Release);
                        let association = DirectUdpAssociation::new_with_hook(
                            proxy,
                            flow.tuple.remote,
                            password.as_bytes(),
                            PROXY_CONNECT_TIMEOUT,
                            |stream| {
                                receipts
                                    .consume(receipt, &session_id, flow.tuple, helper_id, now)
                                    .map_err(|error| {
                                        io::Error::other(format!("consume UDP receipt: {error:?}"))
                                    })?;
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
                            Err(_) => {
                                receipts.retire(receipt).map_err(|error| {
                                    io::Error::other(format!(
                                        "retire refused UDP receipt: {error:?}"
                                    ))
                                })?;
                                stats.proxy_refused.fetch_add(1, Ordering::Relaxed);
                                if !mark_refused_flow(&refused, flow.tuple) {
                                    reinject(&handle, &stats, &packet, &address)?;
                                    return Err(io::Error::other("refused route ledger full"));
                                }
                                reinject(&handle, &stats, &packet, &address)?;
                                continue;
                            }
                        };
                        let relay = association.relay_tuple()?;
                        let registered_at = unix_now_ns()?;
                        context
                            .register_direct_helper(
                                "udp",
                                relay.local,
                                relay.remote,
                                flow.clone(),
                                registered_at,
                                registered_at.saturating_add(10_000_000_000),
                            )
                            .map_err(io::Error::other)?;
                        excluded.lock().expect("helper route lock").insert(relay);
                        issued_receipts.push(receipt);
                        if let Err(error) = association.send(datagram.payload) {
                            packet_finished(&stats, PacketDisposition::Lost)?;
                            return Err(io::Error::other(format!(
                                "first selected UDP packet: {error}"
                            )));
                        }
                        selected
                            .lock()
                            .expect("selected route lock")
                            .insert(flow.tuple);
                        packet_finished(&stats, PacketDisposition::Consumed)?;
                        active_udp.insert(
                            flow.tuple,
                            ActiveUdp {
                                association,
                                address,
                            },
                        );
                        continue;
                    }
                    if flow.tuple.transport != Proto::Tcp {
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
                    let helper_open_earliest = unix_now_ns()?;
                    helper_id = helper_id.saturating_add(1);
                    let now = started.elapsed().as_nanos() as u64;
                    let receipt = receipts
                        .issue(flow.clone(), now.saturating_add(10_000_000_000))
                        .map_err(|error| {
                            io::Error::other(format!("issue route receipt: {error:?}"))
                        })?;
                    connecting.store(true, Ordering::Release);
                    let helper = connect_authenticated_socks5_with_hook(
                        proxy,
                        flow.tuple.remote,
                        password.as_bytes(),
                        PROXY_CONNECT_TIMEOUT,
                        |stream| {
                            receipts
                                .consume(receipt, &session_id, flow.tuple, helper_id, now)
                                .map_err(|error| {
                                    io::Error::other(format!("consume route receipt: {error:?}"))
                                })?;
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
                    let helper = match helper {
                        Ok(helper) => helper,
                        Err(_) => {
                            receipts.retire(receipt).map_err(|error| {
                                io::Error::other(format!("retire refused route receipt: {error:?}"))
                            })?;
                            stats.proxy_refused.fetch_add(1, Ordering::Relaxed);
                            if !mark_refused_flow(&refused, flow.tuple) {
                                reinject(&handle, &stats, &packet, &address)?;
                                return Err(io::Error::other("refused route ledger full"));
                            }
                            reinject(&handle, &stats, &packet, &address)?;
                            continue;
                        }
                    };
                    excluded
                        .lock()
                        .expect("helper route lock")
                        .insert(DirectTuple {
                            transport: Proto::Tcp,
                            local: helper.local_addr()?,
                            remote: helper.peer_addr()?,
                        });
                    issued_receipts.push(receipt);
                    let mut bridge =
                        DirectTcpBridge::new(flow.tuple, helper).map_err(io::Error::other)?;
                    selected
                        .lock()
                        .expect("selected route lock")
                        .insert(flow.tuple);
                    let replies = match bridge.ingest_diverted(
                        &packet,
                        address.ip_checksum_valid(),
                        started.elapsed().as_millis() as i64,
                    ) {
                        Ok(replies) => replies,
                        Err(error) => {
                            packet_finished(&stats, PacketDisposition::Lost)?;
                            return Err(io::Error::other(format!(
                                "first selected TCP packet: {error}"
                            )));
                        }
                    };
                    packet_finished(&stats, PacketDisposition::Consumed)?;
                    active.insert(flow.tuple, ActiveTcp { bridge, address });
                    inject_replies(&handle, &stats, replies, address)?;
                }
            }
        }
        for flow in active.values_mut() {
            stats.processor_progress();
            let replies = flow
                .bridge
                .poll(started.elapsed().as_millis() as i64)
                .map_err(|error| io::Error::other(format!("selected TCP poll: {error}")))?;
            inject_replies(&handle, &stats, replies, flow.address)?;
        }
        for (tuple, flow) in &active_udp {
            stats.processor_progress();
            for _ in 0..8 {
                let Some(payload) = flow.association.poll_reply()? else {
                    break;
                };
                let reply = build_udp_reply(*tuple, &payload)?;
                inject_replies(&handle, &stats, vec![reply], flow.address)?;
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
}

impl DirectPacketRuntime {
    pub fn start(
        component: &Path,
        context: NativeObservationContext,
        session_id: String,
        proxy: std::net::SocketAddr,
        password: Zeroizing<String>,
        max_flows: usize,
    ) -> io::Result<Self> {
        if max_flows == 0 || max_flows > 256 || !proxy.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid direct route bounds",
            ));
        }
        let api = Api::load_verified(component)?;
        let observer = Arc::new(Mutex::new(DirectSocketObserver::start(
            &api,
            SOCKET_EVENTS,
        )?));
        let handle = Arc::new(api.open(PACKET_FILTER, NETWORK_LAYER, 0)?);
        handle.set_queue(4096, 100, 16 * 1024 * 1024)?;
        let (sender, receiver) = mpsc::sync_channel(PACKET_QUEUE);
        let selected = Arc::new(Mutex::new(BTreeSet::new()));
        let excluded = Arc::new(Mutex::new(BTreeSet::new()));
        let refused = Arc::new(Mutex::new(BTreeSet::new()));
        let connecting = Arc::new(AtomicBool::new(false));
        let route_context = context.clone();
        let stats = Arc::new(SharedStats::new());
        let receive_worker = {
            let handle = Arc::clone(&handle);
            let observer = Arc::clone(&observer);
            let selected = Arc::clone(&selected);
            let excluded = Arc::clone(&excluded);
            let refused = Arc::clone(&refused);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            thread::Builder::new()
                .name("fragcap-direct-receive".into())
                .spawn(move || {
                    let result = receive_packets(ReceiveRuntimeInputs {
                        handle: Arc::clone(&handle),
                        sender,
                        observer,
                        selected,
                        excluded,
                        refused,
                        connecting,
                        stats,
                    });
                    if result.is_err() {
                        let _ = handle.stop();
                    }
                    result
                })?
        };
        let process_worker = {
            let process_handle = Arc::clone(&handle);
            let observer = Arc::clone(&observer);
            let selected = Arc::clone(&selected);
            let excluded = Arc::clone(&excluded);
            let refused = Arc::clone(&refused);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            let launched = thread::Builder::new()
                .name("fragcap-direct-process".into())
                .spawn(move || {
                    let result = process_packets(ProcessRuntimeInputs {
                        receiver,
                        handle: Arc::clone(&process_handle),
                        observer,
                        context,
                        selected,
                        excluded,
                        refused,
                        connecting,
                        stats,
                        session_id,
                        proxy,
                        password,
                        max_flows,
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
        })
    }

    pub fn stop(&mut self) -> io::Result<DirectRuntimeStats> {
        let was_running = self.receive_worker.is_some() || self.process_worker.is_some();
        let mut first_error = None;
        if self.receive_worker.is_some() {
            let handle = self.handle.as_ref().expect("running route handle");
            if let Err(error) = handle.stop_receiving() {
                first_error = Some(error);
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
        let report = DirectRuntimeStats {
            packets,
            socket_events: self.observer.lock().expect("socket observer lock").stats(),
            admission_busy_bypassed: self.stats.admission_busy_bypassed.load(Ordering::Relaxed),
            candidate_capacity_bypassed: self
                .stats
                .candidate_capacity_bypassed
                .load(Ordering::Relaxed),
            bypass_reasons: self
                .stats
                .bypass_reasons
                .lock()
                .expect("bypass reason lock")
                .clone(),
            proxy_refused: self.stats.proxy_refused.load(Ordering::Relaxed),
            synthetic_reply_lost: self.stats.synthetic_reply_lost.load(Ordering::Relaxed),
            event_history_overflow: self.stats.event_history_overflow.load(Ordering::Relaxed),
            pre_receive_loss_unmeasurable: true,
            watchdog_released: self.stats.watchdog_released.load(Ordering::Relaxed),
            watchdog_release_after_progress_ms: self
                .stats
                .watchdog_release_after_progress_ms
                .load(Ordering::Relaxed),
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
        DestinationAuthority, DestinationPolicy, InspectionAction, InspectionDecider,
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

    struct UninspectedDirectOwner;

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
        if protocol == "tcp" {
            let mut client = TcpStream::connect_timeout(&original, Duration::from_secs(3)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            client.write_all(b"runtime-client-request").unwrap();
            client.shutdown(std::net::Shutdown::Write).unwrap();
            let mut response = Vec::new();
            client.read_to_end(&mut response).unwrap();
            assert_eq!(response, b"runtime-origin-reply");
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
        } else if protocol == "tls" {
            let ca = base64::engine::general_purpose::STANDARD
                .decode(std::env::var("FRAGCAP_S200_CHILD_CA").unwrap())
                .unwrap();
            let mut roots = rustls::RootCertStore::empty();
            roots
                .add(rustls::pki_types::CertificateDer::from(ca))
                .unwrap();
            let mut config = fragcap_proxy::tls_client_config_with_roots(roots).unwrap();
            Arc::make_mut(&mut config).alpn_protocols = vec![b"http/1.1".to_vec()];
            let server_name = rustls::pki_types::ServerName::IpAddress(original.ip().into());
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
        let server = thread::spawn(move || {
            let mut selected = 0;
            let mut publisher = 0;
            for _ in 0..2 {
                let (mut stream, _) = origin.accept().unwrap();
                let mut first = [0u8; 1];
                stream.read_exact(&mut first).unwrap();
                if first == *b"r" {
                    let mut body = vec![first[0]];
                    stream.read_to_end(&mut body).unwrap();
                    assert_eq!(body, b"runtime-client-request");
                    selected += 1;
                    stream.write_all(b"runtime-origin-reply").unwrap();
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
        chain.stdin.as_mut().unwrap().write_all(b"g").unwrap();
        drop(chain.stdin.take());
        let outcome = chain.wait_with_output().unwrap();
        assert!(
            outcome.status.success(),
            "{}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        let bridged = context.direct_helper_flows_for_test();
        assert_eq!(
            bridged.len(),
            1,
            "only the final client may enter the bridge"
        );
        assert_eq!(bridged[0].pid, client);
        assert!(bridged[0]
            .process_instance_id
            .starts_with(&format!("process-{client}-")));
        assert_eq!(bridged[0].tuple.transport, Proto::Tcp);
        assert_eq!(bridged[0].tuple.remote, original);
        assert_ne!(bridged[0].tuple.local, original);
        assert_ne!(bridged[0].endpoint_id, 0);
        let stats = runtime.stop().unwrap();
        assert!(stats.packets.consumed > 0);
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(server.join().unwrap(), (1, 1));
        let report = proxy.cleanup(Duration::from_secs(3));
        assert_eq!(report.observation.protocol.socks_connect_succeeded, 1);
        assert_eq!(report.observation.protocol.generic_streams_plain, 1);
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
        live_tcp_case(IpAddr::V4(Ipv4Addr::LOCALHOST));
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
        assert!(error.contains("lost: 1"), "{error}");
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
    fn disposable_proxy_refusal_reinjects_selected_first_packet() {
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
        assert!(stats.proxy_refused > 0);
        assert_eq!(stats.packets.consumed, 0);
        assert_eq!(stats.packets.lost, 0);
        assert!(stats.packets.reinjected > 0);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_routes_exact_generation() {
        live_tcp_case(IpAddr::V4(Ipv4Addr::LOCALHOST));
        live_tcp_case(IpAddr::V6(Ipv6Addr::LOCALHOST));
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

    fn live_tcp_case(ip: IpAddr) {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
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
        let mut before = spawn_client("sentinel-tcp", original);
        release_client(&mut before);
        finish_client(before);
        let mut child = spawn_client("tcp", original);
        let context = client_context(child.id());
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
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-runtime".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        let mut during = spawn_client("sentinel-tcp", original);
        release_client(&mut during);
        release_client(&mut child);
        finish_client(during);
        finish_client(child);
        let stop_result = runtime.stop();
        eprintln!("direct runtime probe: family={ip}, stop={stop_result:?}");
        let stats = stop_result.unwrap();
        assert!(stats.packets.consumed > 0, "selected flow was not routed");
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(stats.synthetic_reply_lost, 0);
        assert!(stats.pre_receive_loss_unmeasurable);
        let mut after = spawn_client("sentinel-tcp", original);
        release_client(&mut after);
        finish_client(after);
        assert_eq!(server.join().unwrap(), (1, 3));
        let proxy_report = lease.cleanup(Duration::from_secs(2));
        assert_eq!(proxy_report.observation.protocol.socks_connect_succeeded, 1);
        assert_eq!(proxy_report.observation.protocol.generic_streams_plain, 1);
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_classifies_http_tls_and_opaque() {
        live_inspected_tcp_case(false, false);
        live_inspected_tcp_case(true, false);
        live_inspected_tcp_case(true, true);
    }

    fn live_inspected_tcp_case(tls: bool, opaque: bool) {
        use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind("127.0.0.1:0").unwrap();
        let original = origin.local_addr().unwrap();
        let (origin_config, origin_root) = if tls {
            let generated =
                rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_string()]).unwrap();
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
            let (stream, _) = origin.accept().unwrap();
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
            if tls { "tls" } else { "http" },
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
        finish_client(child);
        let stats = runtime.stop().unwrap();
        assert!(stats.packets.consumed > 0);
        assert_eq!(stats.packets.lost, 0);
        server.join().unwrap();
        let report = lease.cleanup(Duration::from_secs(3));
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
