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
    direct_bridge::{connect_authenticated_socks5, DirectTcpBridge},
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
const PACKET_FILTER: &str = "tcp or udp";

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
}

struct SharedStats {
    started: Instant,
    packets: Mutex<DirectPacketAccounting>,
    processor_progress_ms: AtomicU64,
    watchdog_released: AtomicBool,
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
            && now_ms.saturating_sub(self.processor_progress_ms.load(Ordering::Acquire)) >= 100
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

fn receive_packets(
    handle: Arc<Handle>,
    sender: SyncSender<WorkPacket>,
    selected: Arc<Mutex<BTreeSet<DirectTuple>>>,
    excluded: Arc<Mutex<BTreeSet<DirectTuple>>>,
    connecting: Arc<AtomicBool>,
    stats: Arc<SharedStats>,
) -> io::Result<()> {
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
        if !is_candidate || connecting.load(Ordering::Acquire) || is_excluded {
            if is_candidate {
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
                        connecting.store(true, Ordering::Release);
                        let association = DirectUdpAssociation::new(
                            proxy,
                            flow.tuple.remote,
                            password.as_bytes(),
                            Duration::from_millis(50),
                        );
                        connecting.store(false, Ordering::Release);
                        let association = match association {
                            Ok(association) => association,
                            Err(_) => {
                                stats.proxy_refused.fetch_add(1, Ordering::Relaxed);
                                reinject(&handle, &stats, &packet, &address)?;
                                continue;
                            }
                        };
                        let helper_open_latest = unix_now_ns()?;
                        excluded
                            .lock()
                            .expect("helper route lock")
                            .insert(association.relay_tuple()?);
                        helper_id = helper_id.saturating_add(1);
                        let now = started.elapsed().as_nanos() as u64;
                        let receipt = receipts
                            .issue(flow.clone(), now.saturating_add(10_000_000_000))
                            .map_err(|error| {
                                io::Error::other(format!("issue UDP receipt: {error:?}"))
                            })?;
                        receipts
                            .consume(receipt, &session_id, flow.tuple, helper_id, now)
                            .map_err(|error| {
                                io::Error::other(format!("consume UDP receipt: {error:?}"))
                            })?;
                        if let Err(reason) = context.register_direct_helper(
                            association.control_local()?,
                            association.control_peer()?,
                            flow.clone(),
                            helper_open_earliest,
                            helper_open_latest,
                        ) {
                            receipts.retire(receipt).map_err(|error| {
                                io::Error::other(format!("retire refused UDP receipt: {error:?}"))
                            })?;
                            stats.record_bypass(reason);
                            reinject(&handle, &stats, &packet, &address)?;
                            continue;
                        }
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
                    connecting.store(true, Ordering::Release);
                    let helper = connect_authenticated_socks5(
                        proxy,
                        flow.tuple.remote,
                        password.as_bytes(),
                        Duration::from_secs(2),
                    );
                    connecting.store(false, Ordering::Release);
                    let helper = match helper {
                        Ok(helper) => helper,
                        Err(_) => {
                            stats.proxy_refused.fetch_add(1, Ordering::Relaxed);
                            reinject(&handle, &stats, &packet, &address)?;
                            continue;
                        }
                    };
                    let helper_open_latest = unix_now_ns()?;
                    excluded
                        .lock()
                        .expect("helper route lock")
                        .insert(DirectTuple {
                            transport: Proto::Tcp,
                            local: helper.local_addr()?,
                            remote: helper.peer_addr()?,
                        });
                    helper_id = helper_id.saturating_add(1);
                    let now = started.elapsed().as_nanos() as u64;
                    let receipt = receipts
                        .issue(flow.clone(), now.saturating_add(10_000_000_000))
                        .map_err(|error| {
                            io::Error::other(format!("issue route receipt: {error:?}"))
                        })?;
                    receipts
                        .consume(receipt, &session_id, flow.tuple, helper_id, now)
                        .map_err(|error| {
                            io::Error::other(format!("consume route receipt: {error:?}"))
                        })?;
                    if let Err(reason) = context.register_direct_helper(
                        helper.local_addr()?,
                        helper.peer_addr()?,
                        flow.clone(),
                        helper_open_earliest,
                        helper_open_latest,
                    ) {
                        receipts.retire(receipt).map_err(|error| {
                            io::Error::other(format!("retire refused route receipt: {error:?}"))
                        })?;
                        stats.record_bypass(reason);
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
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
        let connecting = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(SharedStats::new());
        let receive_worker = {
            let handle = Arc::clone(&handle);
            let selected = Arc::clone(&selected);
            let excluded = Arc::clone(&excluded);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            thread::Builder::new()
                .name("fragcap-direct-receive".into())
                .spawn(move || {
                    let result = receive_packets(
                        Arc::clone(&handle),
                        sender,
                        selected,
                        excluded,
                        connecting,
                        stats,
                    );
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
        Ok(Self {
            handle: Some(handle),
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
        io::{Read, Write},
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
        os::windows::process::CommandExt,
        process::{Child, Command, Stdio},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    use fragcap_core::{ProcessEvent, Timestamp, WatcherReport};
    use fragcap_proxy::{DestinationPolicy, NativeProxyBackend, NativeProxyConfig};

    use crate::deep_capture::{CaptureProcessEvidence, StageTransition, StageTransitionKind};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn spawn_client(protocol: &str, original: std::net::SocketAddr) -> Child {
        Command::new(std::env::current_exe().unwrap())
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
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap()
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

    #[test]
    fn direct_runtime_child() {
        let Ok(protocol) = std::env::var("FRAGCAP_S200_CHILD_PROTOCOL") else {
            return;
        };
        let original: std::net::SocketAddr = std::env::var("FRAGCAP_S200_CHILD_ORIGINAL")
            .unwrap()
            .parse()
            .unwrap();
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
        assert!(!stats.stalled_packet(109));
        assert!(stats.stalled_packet(110));
        stats.processor_progress_ms.store(105, Ordering::Release);
        assert!(!stats.stalled_packet(110));
        packet_finished(&stats, PacketDisposition::Reinjected).unwrap();
        assert!(!stats.stalled_packet(1_000));
        stats.packets.lock().unwrap().release().unwrap();
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_routes_exact_generation() {
        live_tcp_case(IpAddr::V4(Ipv4Addr::LOCALHOST));
        live_tcp_case(IpAddr::V6(Ipv6Addr::LOCALHOST));
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
        let config =
            NativeProxyConfig::new(SocketAddr::new(ip, 0), 4, 1024, Duration::from_secs(2))
                .unwrap()
                .with_session_id("s200-direct-runtime")
                .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let mut child = spawn_client("tcp", original);
        let context = client_context(child.id());
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
        server.join().unwrap();
        assert_eq!(
            proxy_report
                .observation
                .protocol
                .socks_udp_associate_succeeded,
            1
        );
    }
}
