// SPDX-License-Identifier: Apache-2.0

//! Exact original-flow ownership and one-use bridge receipts.

use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
};

use fragcap_core::Proto;

use super::{CaptureProcessEvidence, ConnectionInception};

const MAX_CLOCK_ERROR_NS: u64 = 5_000_000;
const MAX_ANCHOR_AGE_SECONDS: i128 = 10;

/// Bounded conversion between WinDivert's QPC timestamp and process-event
/// Unix time. Packet/event ordering itself stays in QPC units.
#[derive(Clone, Copy, Debug)]
pub struct QpcAnchor {
    qpc_midpoint: i64,
    unix_ns: i64,
    frequency: i64,
    error_ns: u64,
}

impl QpcAnchor {
    pub fn from_bracket(
        before_qpc: i64,
        after_qpc: i64,
        unix_ns: i64,
        frequency: i64,
    ) -> Option<Self> {
        if before_qpc < 0 || after_qpc < before_qpc || unix_ns < 0 || frequency <= 0 {
            return None;
        }
        let width = i128::from(after_qpc - before_qpc);
        let width_ns = width.checked_mul(1_000_000_000)? / i128::from(frequency);
        let error_ns = u64::try_from(width_ns / 2 + 100_000).ok()?;
        if error_ns > MAX_CLOCK_ERROR_NS {
            return None;
        }
        Some(Self {
            qpc_midpoint: before_qpc + (after_qpc - before_qpc) / 2,
            unix_ns,
            frequency,
            error_ns,
        })
    }

    pub fn convert(self, qpc: i64) -> Option<(i64, u64)> {
        let delta = i128::from(qpc) - i128::from(self.qpc_midpoint);
        if delta.abs() > i128::from(self.frequency) * MAX_ANCHOR_AGE_SECONDS {
            return None;
        }
        let delta_ns = delta.checked_mul(1_000_000_000)? / i128::from(self.frequency);
        let unix_ns = i64::try_from(i128::from(self.unix_ns) + delta_ns).ok()?;
        (unix_ns >= 0).then_some((unix_ns, self.error_ns))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DirectTuple {
    pub transport: Proto,
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocketEventKind {
    Open,
    Close,
}

#[derive(Clone, Copy, Debug)]
pub struct DirectSocketEvent {
    pub tuple: DirectTuple,
    pub endpoint_id: u64,
    pub pid: u32,
    pub at_qpc: i64,
    pub at_unix_ns: i64,
    pub clock_error_ns: u64,
    pub kind: SocketEventKind,
}

#[derive(Clone, Copy, Debug)]
pub struct DirectPacketHead {
    pub tuple: DirectTuple,
    pub at_qpc: i64,
    pub at_unix_ns: i64,
    pub clock_error_ns: u64,
    pub initial: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OriginalFlow {
    pub tuple: DirectTuple,
    pub endpoint_id: u64,
    pub pid: u32,
    pub process_instance_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectAdmission {
    Selected(OriginalFlow),
    Bypassed(&'static str),
    Ambiguous(&'static str),
    Unavailable(&'static str),
}

pub fn select_direct_flow(
    evidence: &CaptureProcessEvidence,
    events: &[DirectSocketEvent],
    packet: DirectPacketHead,
) -> DirectAdmission {
    let matching: Vec<_> = events
        .iter()
        .filter(|event| event.tuple == packet.tuple)
        .collect();
    if matching.is_empty() {
        return DirectAdmission::Unavailable("socket-event-missing");
    }
    let eligible: Vec<_> = matching
        .iter()
        .copied()
        .filter(|event| {
            event.at_qpc <= packet.at_qpc
                && event.kind == SocketEventKind::Open
                && !matching.iter().any(|other| {
                    other.kind == SocketEventKind::Close
                        && other.endpoint_id == event.endpoint_id
                        && other.at_qpc >= event.at_qpc
                        && other.at_qpc <= packet.at_qpc
                })
        })
        .collect();
    if eligible.is_empty() {
        return DirectAdmission::Bypassed("first-packet-before-owner-event");
    }
    if eligible.len() != 1 {
        return DirectAdmission::Ambiguous("competing-socket-events");
    }
    let event = eligible[0];
    if event.clock_error_ns > MAX_CLOCK_ERROR_NS
        || packet.clock_error_ns > MAX_CLOCK_ERROR_NS
        || event.at_unix_ns < 0
        || packet.at_unix_ns < 0
        || i128::from(event.at_unix_ns) - i128::from(packet.at_unix_ns)
            > i128::from(event.clock_error_ns) + i128::from(packet.clock_error_ns)
    {
        return DirectAdmission::Unavailable("socket-clock-uncertain");
    }
    if !packet.initial {
        return DirectAdmission::Bypassed("initial-packet-missed");
    }
    let inception = ConnectionInception {
        state: "exact",
        reason: "windivert-socket-event",
        pid: Some(event.pid),
        socket_created_at_ns: Some(event.at_unix_ns),
    };
    let observed_at_ns = packet.at_unix_ns.max(event.at_unix_ns) as u64;
    let owner = super::native::bind_inception_process(evidence, inception, observed_at_ns);
    if owner.reason == "socket-owner-managed-ancestry-unproven" {
        return DirectAdmission::Bypassed("owner-not-final-client");
    }
    if owner.state != "exact" {
        return DirectAdmission::Unavailable("process-generation-unproven");
    }
    if owner.role.as_deref() != Some("client") {
        return DirectAdmission::Bypassed("owner-not-final-client");
    }
    let Some(process_instance_id) = owner.process_instance_id else {
        return DirectAdmission::Unavailable("process-instance-missing");
    };
    DirectAdmission::Selected(OriginalFlow {
        tuple: packet.tuple,
        endpoint_id: event.endpoint_id,
        pid: event.pid,
        process_instance_id,
    })
}

/// One held first packet. The payload remains opaque until exact selection;
/// a bypass returns these original bytes for unmodified reinjection.
pub struct PendingCandidate<A> {
    pub head: DirectPacketHead,
    pub packet: Vec<u8>,
    pub address: A,
    deadline_ns: u64,
}

pub enum CandidateDecision<A> {
    Selected {
        flow: OriginalFlow,
        packet: Vec<u8>,
        address: A,
    },
    PassThrough {
        reason: &'static str,
        packet: Vec<u8>,
        address: A,
    },
}

pub struct CandidateBuffer<A> {
    pending: VecDeque<PendingCandidate<A>>,
    max_packets: usize,
    max_bytes: usize,
    held_bytes: usize,
    wait_ns: u64,
}

impl<A> CandidateBuffer<A> {
    pub fn new(max_packets: usize, max_bytes: usize, wait_ns: u64) -> Option<Self> {
        if max_packets == 0
            || max_packets > 256
            || max_bytes == 0
            || max_bytes > 4 * 1024 * 1024
            || wait_ns == 0
            || wait_ns > 10_000_000
        {
            return None;
        }
        Some(Self {
            pending: VecDeque::with_capacity(max_packets),
            max_packets,
            max_bytes,
            held_bytes: 0,
            wait_ns,
        })
    }

    pub fn enqueue(
        &mut self,
        head: DirectPacketHead,
        packet: Vec<u8>,
        address: A,
        now_ns: u64,
    ) -> Result<(), PendingCandidate<A>> {
        let candidate = PendingCandidate {
            head,
            deadline_ns: now_ns.saturating_add(self.wait_ns),
            packet,
            address,
        };
        if candidate.packet.is_empty()
            || candidate.packet.len() > 65_535
            || self.pending.len() >= self.max_packets
            || self
                .held_bytes
                .checked_add(candidate.packet.len())
                .is_none_or(|total| total > self.max_bytes)
        {
            return Err(candidate);
        }
        self.held_bytes += candidate.packet.len();
        self.pending.push_back(candidate);
        Ok(())
    }

    pub fn resolve(
        &mut self,
        evidence: Option<&CaptureProcessEvidence>,
        events: &[DirectSocketEvent],
        now_ns: u64,
    ) -> Vec<CandidateDecision<A>> {
        enum Status {
            Wait,
            Selected(OriginalFlow),
            Pass(&'static str),
        }
        let mut resolved = Vec::new();
        let count = self.pending.len();
        for _ in 0..count {
            let candidate = self.pending.pop_front().expect("fixed pending length");
            let decision = if now_ns >= candidate.deadline_ns {
                Some(DirectAdmission::Bypassed("candidate-wait-expired"))
            } else {
                evidence.map(|evidence| select_direct_flow(evidence, events, candidate.head))
            };
            let status = match decision {
                Some(DirectAdmission::Selected(flow)) => Status::Selected(flow),
                Some(DirectAdmission::Bypassed(reason) | DirectAdmission::Ambiguous(reason)) => {
                    Status::Pass(reason)
                }
                Some(DirectAdmission::Unavailable(
                    "socket-event-missing" | "process-generation-unproven",
                ))
                | None => Status::Wait,
                Some(DirectAdmission::Unavailable(reason)) => Status::Pass(reason),
            };
            match status {
                Status::Wait => self.pending.push_back(candidate),
                Status::Selected(flow) => {
                    self.held_bytes -= candidate.packet.len();
                    resolved.push(CandidateDecision::Selected {
                        flow,
                        packet: candidate.packet,
                        address: candidate.address,
                    });
                }
                Status::Pass(reason) => {
                    self.held_bytes -= candidate.packet.len();
                    resolved.push(CandidateDecision::PassThrough {
                        reason,
                        packet: candidate.packet,
                        address: candidate.address,
                    });
                }
            }
        }
        resolved
    }

    pub fn drain(&mut self) -> Vec<CandidateDecision<A>> {
        self.held_bytes = 0;
        self.pending
            .drain(..)
            .map(|candidate| CandidateDecision::PassThrough {
                reason: "route-stopped",
                packet: candidate.packet,
                address: candidate.address,
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RouteReceiptId([u8; 32]);

impl std::fmt::Debug for RouteReceiptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RouteReceiptId([redacted])")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiptError {
    Full,
    Duplicate,
    TupleInUse,
    Entropy,
    Unknown,
    Expired,
    Mismatch,
    Replayed,
    HelperAlreadyBound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketDisposition {
    Reinjected,
    Consumed,
    Lost,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectPacketAccounting {
    pub received: u64,
    pub reinjected: u64,
    pub consumed: u64,
    pub lost: u64,
    pub in_flight: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingError {
    CounterOverflow,
    NoPacketInFlight,
    UnfinishedPackets,
}

impl DirectPacketAccounting {
    pub fn receive(&mut self) -> Result<(), AccountingError> {
        let received = self
            .received
            .checked_add(1)
            .ok_or(AccountingError::CounterOverflow)?;
        let in_flight = self
            .in_flight
            .checked_add(1)
            .ok_or(AccountingError::CounterOverflow)?;
        self.received = received;
        self.in_flight = in_flight;
        Ok(())
    }

    pub fn finish(&mut self, disposition: PacketDisposition) -> Result<(), AccountingError> {
        if self.in_flight == 0 {
            return Err(AccountingError::NoPacketInFlight);
        }
        let count = match disposition {
            PacketDisposition::Reinjected => &mut self.reinjected,
            PacketDisposition::Consumed => &mut self.consumed,
            PacketDisposition::Lost => &mut self.lost,
        };
        *count = count
            .checked_add(1)
            .ok_or(AccountingError::CounterOverflow)?;
        self.in_flight -= 1;
        Ok(())
    }

    /// After both route workers have ended, queued delivered packets that
    /// never reached a terminal action are explicit fatal-path losses.
    pub fn lose_unfinished(&mut self) -> Result<u64, AccountingError> {
        let unfinished = self.in_flight;
        self.lost = self
            .lost
            .checked_add(unfinished)
            .ok_or(AccountingError::CounterOverflow)?;
        self.in_flight = 0;
        Ok(unfinished)
    }

    pub fn release(&self) -> Result<(), AccountingError> {
        let settled = self
            .reinjected
            .checked_add(self.consumed)
            .and_then(|sum| sum.checked_add(self.lost))
            .ok_or(AccountingError::CounterOverflow)?;
        if self.in_flight == 0 && self.received == settled {
            Ok(())
        } else {
            Err(AccountingError::UnfinishedPackets)
        }
    }
}

struct ReceiptRecord {
    flow: OriginalFlow,
    expires_at: u64,
    helper_connection_id: Option<u64>,
}

pub struct DirectReceiptLedger {
    session_id: String,
    capacity: usize,
    receipts: BTreeMap<[u8; 32], ReceiptRecord>,
}

impl DirectReceiptLedger {
    pub fn new(session_id: String, capacity: usize) -> Self {
        Self {
            session_id,
            capacity,
            receipts: BTreeMap::new(),
        }
    }

    pub fn issue(
        &mut self,
        flow: OriginalFlow,
        expires_at: u64,
    ) -> Result<RouteReceiptId, ReceiptError> {
        if self.receipts.values().any(|record| record.flow == flow) {
            return Err(ReceiptError::Duplicate);
        }
        if self
            .receipts
            .values()
            .any(|record| record.flow.tuple == flow.tuple)
        {
            return Err(ReceiptError::TupleInUse);
        }
        if self.receipts.len() >= self.capacity {
            return Err(ReceiptError::Full);
        }
        let mut bytes = [0; 32];
        for _ in 0..3 {
            getrandom::fill(&mut bytes).map_err(|_| ReceiptError::Entropy)?;
            if let std::collections::btree_map::Entry::Vacant(entry) = self.receipts.entry(bytes) {
                entry.insert(ReceiptRecord {
                    flow,
                    expires_at,
                    helper_connection_id: None,
                });
                return Ok(RouteReceiptId(bytes));
            }
        }
        Err(ReceiptError::Entropy)
    }

    pub fn consume(
        &mut self,
        id: RouteReceiptId,
        session_id: &str,
        tuple: DirectTuple,
        helper_connection_id: u64,
        now: u64,
    ) -> Result<&OriginalFlow, ReceiptError> {
        if session_id != self.session_id {
            return Err(ReceiptError::Mismatch);
        }
        let record = self.receipts.get(&id.0).ok_or(ReceiptError::Unknown)?;
        if record.flow.tuple != tuple {
            return Err(ReceiptError::Mismatch);
        }
        if record.expires_at < now {
            return Err(ReceiptError::Expired);
        }
        if record.helper_connection_id.is_some() {
            return Err(ReceiptError::Replayed);
        }
        if self
            .receipts
            .values()
            .any(|record| record.helper_connection_id == Some(helper_connection_id))
        {
            return Err(ReceiptError::HelperAlreadyBound);
        }
        let record = self.receipts.get_mut(&id.0).ok_or(ReceiptError::Unknown)?;
        record.helper_connection_id = Some(helper_connection_id);
        Ok(&record.flow)
    }

    pub fn retire(&mut self, id: RouteReceiptId) -> Result<OriginalFlow, ReceiptError> {
        self.receipts
            .remove(&id.0)
            .map(|record| record.flow)
            .ok_or(ReceiptError::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep_capture::{StageTransition, StageTransitionKind};
    use fragcap_core::{ProcessEvent, Timestamp, WatcherReport};

    #[test]
    fn qpc_conversion_bounds_sample_error_and_age() {
        let anchor =
            QpcAnchor::from_bracket(1_000_000, 1_000_100, 1_700_000_000, 10_000_000).unwrap();
        assert_eq!(anchor.convert(1_010_050), Some((1_701_000_000, 105_000)));
        assert_eq!(anchor.convert(101_000_051), None);
        assert!(QpcAnchor::from_bracket(0, 100_000, 1_700_000_000, 10_000_000).is_none());
        assert!(QpcAnchor::from_bracket(100, 99, 1_700_000_000, 10_000_000).is_none());
    }

    fn tuple(port: u16) -> DirectTuple {
        DirectTuple {
            transport: Proto::Tcp,
            local: SocketAddr::from(([127, 0, 0, 1], port)),
            remote: SocketAddr::from(([127, 0, 0, 1], 23000)),
        }
    }

    fn evidence() -> CaptureProcessEvidence {
        CaptureProcessEvidence {
            events: vec![
                ProcessEvent::started(10, 1, "platform.exe", "", Timestamp::from_nanos(1000)),
                ProcessEvent::started(11, 10, "publisher.exe", "", Timestamp::from_nanos(1100)),
                ProcessEvent::started(12, 11, "client.exe", "", Timestamp::from_nanos(1200)),
                ProcessEvent::started(13, 1, "unrelated.exe", "", Timestamp::from_nanos(1210)),
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
        }
    }

    fn event(pid: u32, port: u16) -> DirectSocketEvent {
        DirectSocketEvent {
            tuple: tuple(port),
            endpoint_id: 99,
            pid,
            at_qpc: 100,
            at_unix_ns: 1250,
            clock_error_ns: 1000,
            kind: SocketEventKind::Open,
        }
    }

    fn packet(port: u16) -> DirectPacketHead {
        DirectPacketHead {
            tuple: tuple(port),
            at_qpc: 101,
            at_unix_ns: 1300,
            clock_error_ns: 1,
            initial: true,
        }
    }

    #[test]
    fn candidate_wait_selects_only_inside_ten_millisecond_bound() {
        let mut buffer = CandidateBuffer::new(2, 128, 10_000_000).unwrap();
        buffer
            .enqueue(packet(40000), b"first".to_vec(), 7u32, 0)
            .ok()
            .expect("first candidate admitted");
        assert!(buffer.resolve(Some(&evidence()), &[], 5_000_000).is_empty());
        assert_eq!(buffer.len(), 1);
        let decisions = buffer.resolve(Some(&evidence()), &[event(12, 40000)], 9_999_999);
        assert_eq!(decisions.len(), 1);
        let CandidateDecision::Selected {
            flow,
            packet: selected_packet,
            address,
        } = &decisions[0]
        else {
            panic!("exact client must select before deadline")
        };
        assert_eq!(flow.pid, 12);
        assert_eq!(selected_packet, b"first");
        assert_eq!(*address, 7);
        assert_eq!(buffer.len(), 0);

        buffer
            .enqueue(packet(40000), b"late".to_vec(), 8, 0)
            .ok()
            .expect("second candidate admitted");
        let decisions = buffer.resolve(Some(&evidence()), &[event(12, 40000)], 10_000_000);
        assert!(matches!(
            &decisions[0],
            CandidateDecision::PassThrough {
                reason: "candidate-wait-expired",
                packet,
                address: 8
            } if packet == b"late"
        ));
    }

    #[test]
    fn unrelated_candidate_returns_original_packet_and_capacity_is_finite() {
        let mut buffer = CandidateBuffer::new(1, 8, 10_000_000).unwrap();
        buffer
            .enqueue(packet(40000), b"opaque".to_vec(), 1u32, 0)
            .ok()
            .expect("opaque candidate admitted");
        let overflow = buffer
            .enqueue(packet(40001), b"other".to_vec(), 2, 0)
            .err()
            .expect("bounded candidate queue");
        assert_eq!(overflow.packet, b"other");
        assert_eq!(overflow.address, 2);
        let decisions = buffer.resolve(Some(&evidence()), &[event(13, 40000)], 100);
        assert!(matches!(
            &decisions[0],
            CandidateDecision::PassThrough {
                reason: "owner-not-final-client",
                packet,
                address: 1
            } if packet == b"opaque"
        ));
        buffer
            .enqueue(packet(40001), b"stop".to_vec(), 3, 100)
            .ok()
            .expect("stop candidate admitted");
        let drained = buffer.drain();
        assert!(matches!(
            &drained[0],
            CandidateDecision::PassThrough {
                reason: "route-stopped",
                packet,
                address: 3
            } if packet == b"stop"
        ));
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn final_client_requires_exact_generation_and_initial_packet() {
        let exact = select_direct_flow(&evidence(), &[event(12, 40000)], packet(40000));
        let DirectAdmission::Selected(flow) = exact else {
            panic!("expected selected flow")
        };
        assert_eq!(flow.pid, 12);
        assert_eq!(flow.endpoint_id, 99);
        assert_eq!(flow.tuple, tuple(40000));
        assert!(matches!(
            select_direct_flow(&evidence(), &[event(13, 40000)], packet(40000)),
            DirectAdmission::Bypassed("owner-not-final-client")
        ));
        assert!(matches!(
            select_direct_flow(
                &evidence(),
                &[event(12, 40000)],
                DirectPacketHead {
                    initial: false,
                    ..packet(40000)
                }
            ),
            DirectAdmission::Bypassed("initial-packet-missed")
        ));
    }

    #[test]
    fn same_destination_unrelated_and_raced_events_never_select() {
        assert!(matches!(
            select_direct_flow(&evidence(), &[event(12, 40000)], packet(40001)),
            DirectAdmission::Unavailable("socket-event-missing")
        ));
        let mut late = event(12, 40000);
        late.at_qpc = 102;
        assert!(matches!(
            select_direct_flow(&evidence(), &[late], packet(40000)),
            DirectAdmission::Bypassed("first-packet-before-owner-event")
        ));
        assert!(matches!(
            select_direct_flow(
                &evidence(),
                &[event(12, 40000), event(13, 40000)],
                packet(40000)
            ),
            DirectAdmission::Ambiguous("competing-socket-events")
        ));
    }

    #[test]
    fn closed_endpoint_cannot_own_reused_tuple() {
        let old = event(13, 40000);
        let mut close = old;
        close.kind = SocketEventKind::Close;
        close.at_qpc = 100;
        let mut fresh = event(12, 40000);
        fresh.endpoint_id = 100;
        fresh.at_qpc = 101;
        let mut first = packet(40000);
        first.at_qpc = 102;
        assert!(matches!(
            select_direct_flow(&evidence(), &[old, close, fresh], first),
            DirectAdmission::Selected(OriginalFlow {
                endpoint_id: 100,
                pid: 12,
                ..
            })
        ));
        assert!(matches!(
            select_direct_flow(&evidence(), &[old, close], first),
            DirectAdmission::Bypassed("first-packet-before-owner-event")
        ));
    }

    #[test]
    fn reused_pid_and_uncertain_packet_clock_never_select() {
        let mut reused = evidence();
        reused.events.push(ProcessEvent::Exited {
            pid: 12,
            at: Timestamp::from_nanos(1270),
        });
        reused.events.push(ProcessEvent::started(
            12,
            1,
            "unrelated.exe",
            "",
            Timestamp::from_nanos(1280),
        ));
        assert!(matches!(
            select_direct_flow(&reused, &[event(12, 40000)], packet(40000)),
            DirectAdmission::Unavailable("process-generation-unproven")
        ));
        let uncertain = DirectPacketHead {
            clock_error_ns: MAX_CLOCK_ERROR_NS + 1,
            ..packet(40000)
        };
        assert!(matches!(
            select_direct_flow(&evidence(), &[event(12, 40000)], uncertain),
            DirectAdmission::Unavailable("socket-clock-uncertain")
        ));
    }

    #[test]
    fn receipt_is_session_and_tuple_bound_and_one_use() {
        let DirectAdmission::Selected(flow) =
            select_direct_flow(&evidence(), &[event(12, 40000)], packet(40000))
        else {
            panic!("expected selected flow")
        };
        let mut ledger = DirectReceiptLedger::new("session-a".into(), 2);
        let id = ledger.issue(flow.clone(), 500).unwrap();
        assert_eq!(
            ledger.issue(flow.clone(), 500),
            Err(ReceiptError::Duplicate)
        );
        assert!(matches!(
            ledger.consume(id, "session-b", flow.tuple, 3, 100),
            Err(ReceiptError::Mismatch)
        ));
        assert!(matches!(
            ledger.consume(id, "session-a", tuple(40001), 3, 100),
            Err(ReceiptError::Mismatch)
        ));
        assert_eq!(
            ledger.consume(id, "session-a", flow.tuple, 3, 100),
            Ok(&flow)
        );
        assert!(matches!(
            ledger.consume(id, "session-a", flow.tuple, 4, 101),
            Err(ReceiptError::Replayed)
        ));
    }

    #[test]
    fn receipts_cannot_bind_competing_generations_or_one_helper_twice() {
        let DirectAdmission::Selected(flow) =
            select_direct_flow(&evidence(), &[event(12, 40000)], packet(40000))
        else {
            panic!("expected selected flow")
        };
        let mut ledger = DirectReceiptLedger::new("session-a".into(), 3);
        let first = ledger.issue(flow.clone(), 500).unwrap();
        let mut reused = flow.clone();
        reused.endpoint_id += 1;
        reused.process_instance_id.push_str("-reused");
        assert_eq!(
            ledger.issue(reused.clone(), 500),
            Err(ReceiptError::TupleInUse)
        );
        let second_flow = OriginalFlow {
            tuple: tuple(40001),
            ..flow.clone()
        };
        let second = ledger.issue(second_flow.clone(), 500).unwrap();
        assert_eq!(
            ledger.consume(first, "session-a", flow.tuple, 7, 100),
            Ok(&flow)
        );
        assert!(matches!(
            ledger.consume(second, "session-a", second_flow.tuple, 7, 100),
            Err(ReceiptError::HelperAlreadyBound)
        ));
        ledger.retire(first).unwrap();
        assert!(ledger.issue(reused, 500).is_ok());
    }

    #[test]
    fn every_received_packet_has_one_exclusive_terminal_disposition() {
        let mut counts = DirectPacketAccounting::default();
        for _ in 0..3 {
            counts.receive().unwrap();
        }
        assert_eq!(counts.release(), Err(AccountingError::UnfinishedPackets));
        counts.finish(PacketDisposition::Reinjected).unwrap();
        counts.finish(PacketDisposition::Consumed).unwrap();
        counts.finish(PacketDisposition::Lost).unwrap();
        assert_eq!(
            counts.finish(PacketDisposition::Lost),
            Err(AccountingError::NoPacketInFlight)
        );
        assert_eq!(counts.release(), Ok(()));
        assert_eq!(
            counts.received,
            counts.reinjected + counts.consumed + counts.lost
        );
        counts.receive().unwrap();
        counts.receive().unwrap();
        assert_eq!(counts.lose_unfinished(), Ok(2));
        assert_eq!(counts.lost, 3);
        assert_eq!(counts.release(), Ok(()));
    }
}
