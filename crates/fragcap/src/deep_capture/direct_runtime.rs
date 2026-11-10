// SPDX-License-Identifier: Apache-2.0

//! Finite, prearmed packet route for exact managed TCP sockets.

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
    time::{Duration, Instant},
};

use fragcap_core::{parse::raw_transport_head, Proto};
use zeroize::Zeroizing;

use super::{
    direct_bridge::{connect_authenticated_socks5, DirectTcpBridge},
    direct_observer::{DirectSocketObserver, SocketObserverStats},
    direct_route::{
        CandidateBuffer, CandidateDecision, DirectPacketAccounting, DirectReceiptLedger,
        DirectSocketEvent, DirectTuple, PacketDisposition,
    },
    windivert::{sample_qpc_anchor, Address, Api, Handle, NETWORK_LAYER},
    NativeObservationContext,
};

const PACKET_QUEUE: usize = 256;
const CANDIDATE_BYTES: usize = 4 * 1024 * 1024;
const CANDIDATE_WAIT_NS: u64 = 10_000_000;
const SOCKET_EVENTS: usize = 4096;
const PACKET_FILTER: &str = "tcp or udp";

enum WorkPacket {
    Candidate(super::direct_route::DirectPacketHead, Vec<u8>, Address),
    Selected(DirectTuple, Vec<u8>, Address),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DirectRuntimeStats {
    pub packets: DirectPacketAccounting,
    pub socket_events: SocketObserverStats,
    pub admission_busy_bypassed: u64,
    pub candidate_capacity_bypassed: u64,
    pub proxy_refused: u64,
    pub synthetic_reply_lost: u64,
    pub event_history_overflow: bool,
    pub pre_receive_loss_unmeasurable: bool,
}

#[derive(Default)]
struct SharedStats {
    packets: Mutex<DirectPacketAccounting>,
    admission_busy_bypassed: AtomicU64,
    candidate_capacity_bypassed: AtomicU64,
    proxy_refused: AtomicU64,
    synthetic_reply_lost: AtomicU64,
    event_history_overflow: AtomicBool,
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
        if !is_candidate || connecting.load(Ordering::Acquire) {
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

fn process_packets(
    receiver: Receiver<WorkPacket>,
    handle: Arc<Handle>,
    observer: Arc<Mutex<DirectSocketObserver>>,
    context: NativeObservationContext,
    selected: Arc<Mutex<BTreeSet<DirectTuple>>>,
    connecting: Arc<AtomicBool>,
    stats: Arc<SharedStats>,
    session_id: String,
    proxy: std::net::SocketAddr,
    password: Zeroizing<String>,
    max_flows: usize,
) -> io::Result<()> {
    let started = Instant::now();
    let mut candidates = CandidateBuffer::new(PACKET_QUEUE, CANDIDATE_BYTES, CANDIDATE_WAIT_NS)
        .expect("fixed candidate bounds");
    let mut events = Vec::<DirectSocketEvent>::new();
    let mut active = BTreeMap::<DirectTuple, ActiveTcp>::new();
    let mut receipts = DirectReceiptLedger::new(session_id.clone(), max_flows);
    let mut helper_id = 0u64;
    loop {
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
                        packet, address, ..
                    } = decision
                    {
                        reinject(&handle, &stats, &packet, &address)?;
                    }
                }
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let new_events = observer.lock().expect("socket observer lock").take_events();
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
                    packet, address, ..
                } => {
                    reinject(&handle, &stats, &packet, &address)?;
                }
                CandidateDecision::Selected {
                    flow,
                    packet,
                    address,
                } => {
                    if flow.tuple.transport != Proto::Tcp || active.len() >= max_flows {
                        reinject(&handle, &stats, &packet, &address)?;
                        continue;
                    }
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
            let replies = flow
                .bridge
                .poll(started.elapsed().as_millis() as i64)
                .map_err(|error| io::Error::other(format!("selected TCP poll: {error}")))?;
            inject_replies(&handle, &stats, replies, flow.address)?;
        }
    }
}

/// Stop must be called before the native proxy lease is released.
pub struct DirectPacketRuntime {
    handle: Arc<Handle>,
    observer: Arc<Mutex<DirectSocketObserver>>,
    receive_worker: Option<JoinHandle<io::Result<()>>>,
    process_worker: Option<JoinHandle<io::Result<()>>>,
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
        let connecting = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(SharedStats::default());
        let receive_worker = {
            let handle = Arc::clone(&handle);
            let selected = Arc::clone(&selected);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            thread::Builder::new()
                .name("fragcap-direct-receive".into())
                .spawn(move || receive_packets(handle, sender, selected, connecting, stats))?
        };
        let process_worker = {
            let handle = Arc::clone(&handle);
            let observer = Arc::clone(&observer);
            let selected = Arc::clone(&selected);
            let connecting = Arc::clone(&connecting);
            let stats = Arc::clone(&stats);
            thread::Builder::new()
                .name("fragcap-direct-process".into())
                .spawn(move || {
                    process_packets(
                        receiver, handle, observer, context, selected, connecting, stats,
                        session_id, proxy, password, max_flows,
                    )
                })?
        };
        Ok(Self {
            handle,
            observer,
            receive_worker: Some(receive_worker),
            process_worker: Some(process_worker),
            stats,
        })
    }

    pub fn stop(&mut self) -> io::Result<DirectRuntimeStats> {
        if self.receive_worker.is_some() {
            self.handle.stop_receiving()?;
            self.receive_worker
                .take()
                .unwrap()
                .join()
                .map_err(|_| io::Error::other("direct receive worker panicked"))??;
        }
        if let Some(worker) = self.process_worker.take() {
            worker
                .join()
                .map_err(|_| io::Error::other("direct process worker panicked"))??;
        }
        self.observer.lock().expect("socket observer lock").stop()?;
        self.handle.stop()?;
        let packets = *self.stats.packets.lock().expect("route accounting lock");
        packets
            .release()
            .map_err(|error| io::Error::other(format!("unfinished direct packets: {error:?}")))?;
        Ok(DirectRuntimeStats {
            packets,
            socket_events: self.observer.lock().expect("socket observer lock").stats(),
            admission_busy_bypassed: self.stats.admission_busy_bypassed.load(Ordering::Relaxed),
            candidate_capacity_bypassed: self
                .stats
                .candidate_capacity_bypassed
                .load(Ordering::Relaxed),
            proxy_refused: self.stats.proxy_refused.load(Ordering::Relaxed),
            synthetic_reply_lost: self.stats.synthetic_reply_lost.load(Ordering::Relaxed),
            event_history_overflow: self.stats.event_history_overflow.load(Ordering::Relaxed),
            pre_receive_loss_unmeasurable: true,
        })
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
        net::{Ipv4Addr, TcpListener, TcpStream},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    use fragcap_core::{ProcessEvent, Timestamp, WatcherReport};
    use fragcap_proxy::{DestinationPolicy, NativeProxyBackend, NativeProxyConfig};

    use crate::deep_capture::{CaptureProcessEvidence, StageTransition, StageTransitionKind};

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_direct_runtime_routes_exact_generation() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let origin = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
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
        .with_session_id("s200-direct-runtime")
        .unwrap();
        let mut policy = DestinationPolicy::new(config.listen());
        policy.grant_for_test(original);
        let mut proxy = NativeProxyBackend::new(config).with_destination_policy(policy);
        let mut lease = proxy.start(Duration::from_secs(2)).unwrap();
        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let started = Timestamp::from_nanos(i64::try_from(now_ns - 1_000_000_000).unwrap());
        let pid = std::process::id();
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
        let mut runtime = DirectPacketRuntime::start(
            Path::new(&root),
            context,
            "s200-direct-runtime".into(),
            lease.endpoint(),
            lease.capability_proof().proxy_password(),
            4,
        )
        .unwrap();
        let mut client = TcpStream::connect_timeout(&original, Duration::from_secs(3)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        client.write_all(b"runtime-client-request").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();
        assert_eq!(response, b"runtime-origin-reply");
        let stats = runtime.stop().unwrap();
        assert!(stats.packets.consumed > 0, "selected flow was not routed");
        assert_eq!(stats.packets.lost, 0);
        assert_eq!(stats.synthetic_reply_lost, 0);
        assert!(stats.pre_receive_loss_unmeasurable);
        server.join().unwrap();
        let proxy_report = lease.cleanup(Duration::from_secs(2));
        assert_eq!(proxy_report.observation.protocol.socks_connect_succeeded, 1);
    }
}
