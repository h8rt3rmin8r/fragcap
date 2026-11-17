// SPDX-License-Identifier: Apache-2.0

//! Bounded passive socket-event observer for direct route candidate joins.

use std::{
    collections::{BTreeMap, VecDeque},
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use super::{
    direct_route::{DirectSocketEvent, DirectTuple, SocketEpoch, SocketEventKind},
    windivert::{
        sample_qpc_anchor, Address, Api, Handle, FLAG_RECV_ONLY, FLAG_SNIFF, SOCKET_LAYER,
    },
};

const SOCKET_FILTER: &str = "(event == CONNECT or event == CLOSE) and (tcp or udp)";
const CLOSED_EPOCH_RETENTION: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
struct ClosedEpoch {
    epoch: SocketEpoch,
    close_qpc: i64,
    closed_at: Instant,
    sequence: u64,
}

#[derive(Default)]
struct OwnerLedger {
    by_tuple: BTreeMap<DirectTuple, Vec<SocketEpoch>>,
    closed_by_tuple: BTreeMap<DirectTuple, VecDeque<ClosedEpoch>>,
    closed_order: VecDeque<(DirectTuple, u64)>,
    live_count: usize,
    history_floor_qpc: i64,
    history_lost: bool,
    next_closed_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentOwner {
    Absent,
    Unique(SocketEpoch),
    Ambiguous,
}

impl OwnerLedger {
    fn state(&self, tuple: DirectTuple) -> CurrentOwner {
        match self.by_tuple.get(&tuple).map(Vec::as_slice) {
            None | Some([]) => CurrentOwner::Absent,
            Some([epoch]) => CurrentOwner::Unique(*epoch),
            Some(_) => CurrentOwner::Ambiguous,
        }
    }

    #[cfg(test)]
    fn unique_epoch(&self, tuple: DirectTuple) -> Option<SocketEpoch> {
        match self.state(tuple) {
            CurrentOwner::Unique(epoch) => Some(epoch),
            CurrentOwner::Absent | CurrentOwner::Ambiguous => None,
        }
    }

    fn state_at_qpc(&self, tuple: DirectTuple, packet_qpc: i64) -> CurrentOwner {
        if packet_qpc <= self.history_floor_qpc || self.history_lost {
            return CurrentOwner::Ambiguous;
        }
        let mut owner = None;
        if let Some(live) = self.by_tuple.get(&tuple) {
            for epoch in live {
                if packet_qpc == epoch.open_qpc {
                    return CurrentOwner::Ambiguous;
                }
                if epoch.contains_packet(packet_qpc) {
                    if owner.is_some() {
                        return CurrentOwner::Ambiguous;
                    }
                    owner = Some(*epoch);
                }
            }
        }
        if let Some(closed) = self.closed_by_tuple.get(&tuple) {
            for interval in closed {
                if packet_qpc == interval.epoch.open_qpc || packet_qpc == interval.close_qpc {
                    return CurrentOwner::Ambiguous;
                }
                if interval.epoch.contains_packet(packet_qpc) && packet_qpc < interval.close_qpc {
                    if owner.is_some() {
                        return CurrentOwner::Ambiguous;
                    }
                    owner = Some(interval.epoch);
                }
            }
        }
        owner.map_or(CurrentOwner::Absent, CurrentOwner::Unique)
    }

    fn owner_at_qpc(&mut self, tuple: DirectTuple, packet_qpc: i64, now: Instant) -> CurrentOwner {
        self.prune_closed(now);
        self.state_at_qpc(tuple, packet_qpc)
    }

    fn prune_closed(&mut self, now: Instant) {
        while let Some((tuple, sequence)) = self.closed_order.front().copied() {
            let Some(interval) = self
                .closed_by_tuple
                .get(&tuple)
                .and_then(VecDeque::front)
                .copied()
            else {
                unreachable!("closed epoch index must match its order queue");
            };
            debug_assert_eq!(interval.sequence, sequence);
            if now.saturating_duration_since(interval.closed_at) < CLOSED_EPOCH_RETENTION {
                break;
            }
            self.closed_order.pop_front();
            self.history_floor_qpc = self.history_floor_qpc.max(interval.close_qpc);
            let intervals = self
                .closed_by_tuple
                .get_mut(&tuple)
                .expect("closed epoch index exists");
            intervals.pop_front();
            if intervals.is_empty() {
                self.closed_by_tuple.remove(&tuple);
            }
        }
    }
}

fn update_current_owner(
    owners: &mut OwnerLedger,
    event: DirectSocketEvent,
    capacity: usize,
) -> bool {
    update_current_owner_at(owners, event, capacity, Instant::now())
}

fn update_current_owner_at(
    owners: &mut OwnerLedger,
    event: DirectSocketEvent,
    capacity: usize,
    now: Instant,
) -> bool {
    owners.prune_closed(now);
    match event.kind {
        SocketEventKind::Open => {
            if owners.live_count >= capacity {
                return false;
            }
            owners
                .by_tuple
                .entry(event.tuple)
                .or_default()
                .push(SocketEpoch::from_open(event));
            owners.live_count += 1;
            true
        }
        SocketEventKind::Close => {
            let mut retained = true;
            if let Some(epochs) = owners.by_tuple.get_mut(&event.tuple) {
                let mut unique = None;
                let mut ambiguous = false;
                for (index, epoch) in epochs.iter().enumerate() {
                    if epoch.matches_event(event) {
                        if unique.is_some() {
                            ambiguous = true;
                            break;
                        }
                        unique = Some(index);
                    }
                }
                // A reused endpoint/PID with two eligible OPENs is
                // indistinguishable at CLOSE. Preserve ambiguity rather than
                // guessing which socket generation ended.
                if !ambiguous {
                    if let Some(index) = unique {
                        let epoch = epochs.remove(index);
                        owners.live_count -= 1;
                        if owners.closed_order.len() >= capacity {
                            owners.history_lost = true;
                            retained = false;
                        } else {
                            let interval = ClosedEpoch {
                                epoch,
                                close_qpc: event.at_qpc,
                                closed_at: now,
                                sequence: owners.next_closed_sequence,
                            };
                            owners.next_closed_sequence =
                                owners.next_closed_sequence.wrapping_add(1);
                            owners
                                .closed_order
                                .push_back((event.tuple, interval.sequence));
                            owners
                                .closed_by_tuple
                                .entry(event.tuple)
                                .or_default()
                                .push_back(interval);
                        }
                    }
                }
                if epochs.is_empty() {
                    owners.by_tuple.remove(&event.tuple);
                }
            }
            retained
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SocketObserverStats {
    pub received: u64,
    pub retained: u64,
    pub capacity_lost: u64,
    pub clock_lost: u64,
}

#[derive(Default)]
struct Counters {
    received: AtomicU64,
    retained: AtomicU64,
    capacity_lost: AtomicU64,
    clock_lost: AtomicU64,
}

impl Counters {
    fn evidence_lost(&self) -> bool {
        self.capacity_lost.load(Ordering::Acquire) != 0
            || self.clock_lost.load(Ordering::Acquire) != 0
    }
}

fn healthy_live_owner(
    owners: &OwnerLedger,
    counters: &Counters,
    tuple: DirectTuple,
) -> CurrentOwner {
    if owners.history_lost || counters.evidence_lost() {
        CurrentOwner::Ambiguous
    } else {
        owners.state(tuple)
    }
}

fn healthy_closed_qpc(
    owners: &mut OwnerLedger,
    counters: &Counters,
    epoch: SocketEpoch,
    now: Instant,
) -> Option<i64> {
    owners.prune_closed(now);
    if owners.history_lost || counters.evidence_lost() {
        return None;
    }
    if owners
        .by_tuple
        .get(&epoch.tuple)
        .is_some_and(|live| live.contains(&epoch))
    {
        return None;
    }
    let mut matching = owners
        .closed_by_tuple
        .get(&epoch.tuple)?
        .iter()
        .filter(|interval| interval.epoch == epoch)
        .map(|interval| interval.close_qpc);
    let close_qpc = matching.next()?;
    matching.next().is_none().then_some(close_qpc)
}

/// The observer opens before the managed root, keeps no payload, and publishes
/// only a finite, caller-drained queue of connect events.
pub struct DirectSocketObserver {
    handle: Arc<Handle>,
    events: Arc<Mutex<VecDeque<DirectSocketEvent>>>,
    current_owners: Arc<Mutex<OwnerLedger>>,
    counters: Arc<Counters>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

impl DirectSocketObserver {
    pub fn start(api: &Arc<Api>, capacity: usize) -> io::Result<Self> {
        if capacity == 0 || capacity > 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "socket-event capacity outside finite route bound",
            ));
        }
        let handle =
            Arc::new(api.open(SOCKET_FILTER, SOCKET_LAYER, FLAG_SNIFF | FLAG_RECV_ONLY)?);
        let events = Arc::new(Mutex::new(VecDeque::with_capacity(capacity)));
        let current_owners = Arc::new(Mutex::new(OwnerLedger::default()));
        let endpoint_tuples = Arc::new(Mutex::new(BTreeMap::<(u64, u32), DirectTuple>::new()));
        let counters = Arc::new(Counters::default());
        let worker_handle = Arc::clone(&handle);
        let worker_events = Arc::clone(&events);
        let worker_owners = Arc::clone(&current_owners);
        let worker_endpoint_tuples = Arc::clone(&endpoint_tuples);
        let worker_counters = Arc::clone(&counters);
        let worker = thread::Builder::new()
            .name("fragcap-direct-socket-events".into())
            .spawn(move || {
                let mut empty = [];
                loop {
                    let mut address = Address::default();
                    match worker_handle.recv(&mut empty, &mut address) {
                        Ok(_) => {
                            worker_counters.received.fetch_add(1, Ordering::Relaxed);
                            let Some(socket) = address.socket() else {
                                continue;
                            };
                            let Some((at_unix_ns, clock_error_ns)) = sample_qpc_anchor()
                                .ok()
                                .and_then(|anchor| anchor.convert(address.timestamp_qpc))
                            else {
                                worker_counters.clock_lost.fetch_add(1, Ordering::Release);
                                continue;
                            };
                            let kind = match address.event() {
                                4 => SocketEventKind::Open,
                                7 => SocketEventKind::Close,
                                _ => continue,
                            };
                            let key = (socket.endpoint_id, socket.pid);
                            let tuple = match kind {
                                SocketEventKind::Open => {
                                    let Some(tuple) = socket.tuple() else {
                                        continue;
                                    };
                                    let mut known = worker_endpoint_tuples
                                        .lock()
                                        .expect("socket endpoint ledger lock");
                                    if known.len() >= capacity && !known.contains_key(&key) {
                                        worker_counters
                                            .capacity_lost
                                            .fetch_add(1, Ordering::Release);
                                        continue;
                                    }
                                    known.insert(key, tuple);
                                    tuple
                                }
                                SocketEventKind::Close => {
                                    let remembered = worker_endpoint_tuples
                                        .lock()
                                        .expect("socket endpoint ledger lock")
                                        .remove(&key);
                                    let Some(tuple) = remembered.or_else(|| socket.tuple()) else {
                                        continue;
                                    };
                                    tuple
                                }
                            };
                            let event = DirectSocketEvent {
                                tuple,
                                endpoint_id: socket.endpoint_id,
                                pid: socket.pid,
                                at_qpc: address.timestamp_qpc,
                                at_unix_ns,
                                clock_error_ns,
                                kind,
                            };
                            if event.at_qpc <= 0 {
                                worker_counters.clock_lost.fetch_add(1, Ordering::Release);
                                continue;
                            }
                            let mut owners = worker_owners.lock().expect("socket owner map lock");
                            if !update_current_owner(&mut owners, event, capacity) {
                                worker_counters
                                    .capacity_lost
                                    .fetch_add(1, Ordering::Release);
                            }
                            let mut queue = worker_events.lock().expect("socket event queue lock");
                            if queue.len() < capacity {
                                queue.push_back(event);
                                worker_counters.retained.fetch_add(1, Ordering::Relaxed);
                            } else {
                                owners.history_lost = true;
                                worker_counters
                                    .capacity_lost
                                    .fetch_add(1, Ordering::Release);
                            }
                        }
                        Err(error) if error.raw_os_error() == Some(232) => return Ok(()),
                        Err(error) => return Err(error),
                    }
                }
            })?;
        Ok(Self {
            handle,
            events,
            current_owners,
            counters,
            worker: Some(worker),
        })
    }

    pub fn take_events(&self) -> Vec<DirectSocketEvent> {
        self.events
            .lock()
            .expect("socket event queue lock")
            .drain(..)
            .collect()
    }

    pub fn current_owner_pid(&self, tuple: DirectTuple) -> Option<u32> {
        self.current_owner_epoch(tuple).map(|owner| owner.pid)
    }

    #[cfg(test)]
    pub fn current_owner_endpoint_id(&self, tuple: DirectTuple) -> Option<u64> {
        self.current_owner_epoch(tuple)
            .map(|owner| owner.endpoint_id)
    }

    pub fn closed_qpc_for_exact_epoch(&self, epoch: SocketEpoch) -> Option<i64> {
        let mut owners = self.current_owners.lock().expect("socket owner map lock");
        healthy_closed_qpc(&mut owners, &self.counters, epoch, Instant::now())
    }

    /// Returns one live generation only while socket evidence is complete.
    /// Two overlapping OPENs or any recorded event loss withhold positive proof.
    pub fn current_owner_epoch(&self, tuple: DirectTuple) -> Option<SocketEpoch> {
        match self.current_owner_state(tuple) {
            CurrentOwner::Unique(epoch) => Some(epoch),
            CurrentOwner::Absent | CurrentOwner::Ambiguous => None,
        }
    }

    pub fn current_owner_state(&self, tuple: DirectTuple) -> CurrentOwner {
        let owners = self.current_owners.lock().expect("socket owner map lock");
        healthy_live_owner(&owners, &self.counters, tuple)
    }

    /// Returns the unique observed socket epoch containing this packet's QPC
    /// instant, including a recently closed epoch. It does not establish that
    /// an event still queued in WinDivert's separate socket handle is absent.
    pub fn owner_at_qpc(&self, tuple: DirectTuple, packet_qpc: i64) -> CurrentOwner {
        let state = self
            .current_owners
            .lock()
            .expect("socket owner map lock")
            .owner_at_qpc(tuple, packet_qpc, Instant::now());
        if self.counters.evidence_lost() {
            CurrentOwner::Ambiguous
        } else {
            state
        }
    }

    pub fn stats(&self) -> SocketObserverStats {
        SocketObserverStats {
            received: self.counters.received.load(Ordering::Relaxed),
            retained: self.counters.retained.load(Ordering::Relaxed),
            capacity_lost: self.counters.capacity_lost.load(Ordering::Relaxed),
            clock_lost: self.counters.clock_lost.load(Ordering::Relaxed),
        }
    }

    pub fn stop(&mut self) -> io::Result<()> {
        if let Some(worker) = self.worker.take() {
            self.handle.stop_receiving()?;
            worker
                .join()
                .map_err(|_| io::Error::other("socket observer worker panicked"))??;
        }
        Ok(())
    }
}

impl Drop for DirectSocketObserver {
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
        path::Path,
        time::{Duration, Instant},
    };

    #[test]
    fn live_owner_proof_stops_after_socket_evidence_loss() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = DirectSocketEvent {
            tuple,
            endpoint_id: 7,
            pid: 42,
            at_qpc: 100,
            at_unix_ns: 100,
            clock_error_ns: 0,
            kind: SocketEventKind::Open,
        };
        let mut owners = OwnerLedger::default();
        assert!(update_current_owner(&mut owners, event, 2));
        assert!(matches!(
            healthy_live_owner(&owners, &Counters::default(), tuple),
            CurrentOwner::Unique(epoch) if epoch.pid == 42
        ));

        let capacity_lost = Counters::default();
        capacity_lost.capacity_lost.store(1, Ordering::Release);
        assert_eq!(
            healthy_live_owner(&owners, &capacity_lost, tuple),
            CurrentOwner::Ambiguous
        );
        let clock_lost = Counters::default();
        clock_lost.clock_lost.store(1, Ordering::Release);
        assert_eq!(
            healthy_live_owner(&owners, &clock_lost, tuple),
            CurrentOwner::Ambiguous
        );
        owners.history_lost = true;
        assert_eq!(
            healthy_live_owner(&owners, &Counters::default(), tuple),
            CurrentOwner::Ambiguous
        );
    }

    #[test]
    fn exact_closed_qpc_requires_complete_unique_retained_evidence() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id: 7,
            pid: 42,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        let base = Instant::now();
        assert!(update_current_owner_at(
            &mut owners,
            event(100, SocketEventKind::Open),
            2,
            base,
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(200, SocketEventKind::Close),
            2,
            base,
        ));
        let epoch = SocketEpoch::from_open(event(100, SocketEventKind::Open));
        assert_eq!(
            healthy_closed_qpc(&mut owners, &Counters::default(), epoch, base),
            Some(200)
        );

        let lost = Counters::default();
        lost.clock_lost.store(1, Ordering::Release);
        assert_eq!(healthy_closed_qpc(&mut owners, &lost, epoch, base), None);
        let interval = *owners.closed_by_tuple.get(&tuple).unwrap().front().unwrap();
        owners
            .closed_by_tuple
            .get_mut(&tuple)
            .unwrap()
            .push_back(interval);
        owners.closed_order.push_back((tuple, interval.sequence));
        assert_eq!(
            healthy_closed_qpc(&mut owners, &Counters::default(), epoch, base),
            None
        );
        assert_eq!(
            healthy_closed_qpc(
                &mut owners,
                &Counters::default(),
                epoch,
                base + CLOSED_EPOCH_RETENTION,
            ),
            None
        );
    }

    #[test]
    fn historical_owner_uses_exact_open_close_interval_across_reuse() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |endpoint_id, pid, at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id,
            pid,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        let base = Instant::now();
        assert!(update_current_owner_at(
            &mut owners,
            event(7, 42, 100, SocketEventKind::Open),
            4,
            base,
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(7, 42, 200, SocketEventKind::Close),
            4,
            base,
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(8, 99, 210, SocketEventKind::Open),
            4,
            base,
        ));
        let old = owners.state_at_qpc(tuple, 150);
        assert!(matches!(old, CurrentOwner::Unique(epoch) if epoch.endpoint_id == 7));
        let new = owners.state_at_qpc(tuple, 215);
        assert!(matches!(new, CurrentOwner::Unique(epoch) if epoch.endpoint_id == 8));
        assert_eq!(owners.state_at_qpc(tuple, 205), CurrentOwner::Absent);
        assert_eq!(owners.state_at_qpc(tuple, 200), CurrentOwner::Ambiguous);
        assert_eq!(owners.state_at_qpc(tuple, 210), CurrentOwner::Ambiguous);
        assert!(
            matches!(owners.state(tuple), CurrentOwner::Unique(epoch) if epoch.endpoint_id == 8)
        );
    }

    #[test]
    fn historical_owner_refuses_overlaps_and_expired_history() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |endpoint_id, pid, at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id,
            pid,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        let base = Instant::now();
        assert!(update_current_owner_at(
            &mut owners,
            event(7, 42, 100, SocketEventKind::Open),
            4,
            base
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(8, 99, 150, SocketEventKind::Open),
            4,
            base
        ));
        assert_eq!(owners.state_at_qpc(tuple, 175), CurrentOwner::Ambiguous);
        assert!(update_current_owner_at(
            &mut owners,
            event(7, 42, 200, SocketEventKind::Close),
            4,
            base
        ));
        assert!(
            matches!(owners.state_at_qpc(tuple, 125), CurrentOwner::Unique(epoch) if epoch.endpoint_id == 7)
        );
        assert_eq!(owners.state_at_qpc(tuple, 175), CurrentOwner::Ambiguous);
        assert!(update_current_owner_at(
            &mut owners,
            event(9, 100, 300, SocketEventKind::Open),
            4,
            base + CLOSED_EPOCH_RETENTION,
        ));
        assert_eq!(owners.state_at_qpc(tuple, 125), CurrentOwner::Ambiguous);
        assert_eq!(owners.closed_order.len(), 0);
    }

    #[test]
    fn historical_owner_expires_without_another_socket_event() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id: 7,
            pid: 42,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        let base = Instant::now();
        assert!(update_current_owner_at(
            &mut owners,
            event(100, SocketEventKind::Open),
            2,
            base,
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(200, SocketEventKind::Close),
            2,
            base,
        ));
        assert!(matches!(
            owners.owner_at_qpc(tuple, 150, base + CLOSED_EPOCH_RETENTION / 2),
            CurrentOwner::Unique(_)
        ));
        assert_eq!(
            owners.owner_at_qpc(tuple, 150, base + CLOSED_EPOCH_RETENTION),
            CurrentOwner::Ambiguous
        );
        assert_eq!(owners.closed_order.len(), 0);
    }

    #[test]
    fn historical_owner_capacity_loss_never_yields_positive_proof() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |endpoint_id, at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id,
            pid: 42,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        let base = Instant::now();
        assert!(update_current_owner_at(
            &mut owners,
            event(7, 100, SocketEventKind::Open),
            1,
            base
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(7, 150, SocketEventKind::Close),
            1,
            base
        ));
        assert!(update_current_owner_at(
            &mut owners,
            event(8, 200, SocketEventKind::Open),
            1,
            base
        ));
        assert!(!update_current_owner_at(
            &mut owners,
            event(8, 250, SocketEventKind::Close),
            1,
            base
        ));
        assert!(owners.history_lost);
        assert_eq!(owners.state_at_qpc(tuple, 125), CurrentOwner::Ambiguous);
        assert_eq!(owners.state_at_qpc(tuple, 225), CurrentOwner::Ambiguous);
    }

    #[test]
    fn stale_close_cannot_remove_reused_tuple_owner() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Udp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |endpoint_id, pid, at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id,
            pid,
            at_qpc,
            at_unix_ns: 1,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        assert!(update_current_owner(
            &mut owners,
            event(7, 42, 1, SocketEventKind::Open),
            2
        ));
        assert!(update_current_owner(
            &mut owners,
            event(8, 99, 2, SocketEventKind::Open),
            2
        ));
        assert!(update_current_owner(
            &mut owners,
            event(7, 42, 3, SocketEventKind::Close),
            2
        ));
        assert_eq!(
            owners.by_tuple.get(&tuple),
            Some(&vec![SocketEpoch {
                tuple,
                endpoint_id: 8,
                pid: 99,
                open_qpc: 2,
            }])
        );
        assert!(update_current_owner(
            &mut owners,
            event(8, 99, 4, SocketEventKind::Close),
            2
        ));
        assert!(owners.by_tuple.is_empty());
        assert_eq!(owners.live_count, 0);
    }

    #[test]
    fn same_endpoint_id_reuse_is_a_distinct_generation_and_closes_normally() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Tcp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id: 7,
            pid: 42,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        assert!(update_current_owner(
            &mut owners,
            event(100, SocketEventKind::Open),
            2
        ));
        assert!(update_current_owner(
            &mut owners,
            event(150, SocketEventKind::Close),
            2
        ));
        assert!(update_current_owner(
            &mut owners,
            event(200, SocketEventKind::Open),
            2
        ));
        assert_eq!(owners.unique_epoch(tuple).unwrap().open_qpc, 200);
        assert!(update_current_owner(
            &mut owners,
            event(300, SocketEventKind::Close),
            2
        ));
        assert!(owners.unique_epoch(tuple).is_none());
        assert_eq!(owners.live_count, 0);
    }

    #[test]
    fn overlapping_open_is_ambiguous_even_with_reused_endpoint_and_pid() {
        let tuple = DirectTuple {
            transport: fragcap_core::Proto::Udp,
            local: "127.0.0.1:41001".parse().unwrap(),
            remote: "127.0.0.1:41002".parse().unwrap(),
        };
        let event = |at_qpc, kind| DirectSocketEvent {
            tuple,
            endpoint_id: 7,
            pid: 42,
            at_qpc,
            at_unix_ns: at_qpc,
            clock_error_ns: 0,
            kind,
        };
        let mut owners = OwnerLedger::default();
        assert!(update_current_owner(
            &mut owners,
            event(100, SocketEventKind::Open),
            2
        ));
        assert!(update_current_owner(
            &mut owners,
            event(200, SocketEventKind::Open),
            2
        ));
        assert!(owners.unique_epoch(tuple).is_none());
        assert!(update_current_owner(
            &mut owners,
            event(250, SocketEventKind::Close),
            2
        ));
        assert!(owners.unique_epoch(tuple).is_none());
        assert_eq!(owners.live_count, 2);
        assert!(!update_current_owner(
            &mut owners,
            event(300, SocketEventKind::Open),
            2
        ));
    }

    #[test]
    #[ignore = "requires the verified WinDivert component on a disposable Windows runner"]
    fn disposable_socket_observer_retains_exact_connect() {
        let root = std::env::var_os("WINDIVERT_VERIFIED_ROOT")
            .expect("WINDIVERT_VERIFIED_ROOT is required");
        let api = Api::load_verified(Path::new(&root)).expect("verified component");
        let mut observer = DirectSocketObserver::start(&api, 256).expect("start observer");
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let destination = listener.local_addr().unwrap();
        let mut client = TcpStream::connect_timeout(&destination, Duration::from_secs(2)).unwrap();
        let client_addr = client.local_addr().unwrap();
        let (mut server, _) = listener.accept().unwrap();
        client.write_all(b"observer").unwrap();
        let mut body = [0; 8];
        server.read_exact(&mut body).unwrap();
        assert_eq!(&body, b"observer");
        let deadline = Instant::now() + Duration::from_millis(100);
        let mut found = None;
        while Instant::now() < deadline {
            for event in observer.take_events() {
                if event.tuple.local == client_addr && event.tuple.remote == destination {
                    found = Some(event);
                }
            }
            if found.is_some() {
                break;
            }
            std::thread::yield_now();
        }
        let event = found.expect("exact socket connect event");
        assert_eq!(event.pid, std::process::id());
        assert!(event.at_unix_ns > 0);
        assert!(event.clock_error_ns <= 5_000_000);
        observer.stop().unwrap();
        assert_eq!(observer.stats().capacity_lost, 0);
        eprintln!(
            "socket observer: pid={}, endpoint_id={}, clock_error_ns={}",
            event.pid, event.endpoint_id, event.clock_error_ns,
        );
    }
}
