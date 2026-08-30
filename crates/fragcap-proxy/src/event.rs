// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
use std::num::NonZeroUsize;

pub const OBSERVATION_VERSION: u16 = 1;

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationKind {
    Lifecycle,
    Connection,
    Dns,
    Tcp,
    Tls,
    Http,
    Stream,
    Message,
    Refusal,
    Error,
    Loss,
    Unknown(String),
    Malformed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayloadState {
    Absent,
    Complete(Vec<u8>),
    Truncated {
        retained: Vec<u8>,
        original_len: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawObservation {
    pub version: u16,
    pub session_id: String,
    pub connection_id: Option<u64>,
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub provenance: String,
    pub kind: ObservationKind,
    pub payload: PayloadState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObservationAccounting {
    pub admitted: u64,
    pub emitted: u64,
    pub dropped_oldest: u64,
    pub truncated: u64,
    pub refused: u64,
    pub unparsed: u64,
    pub projection_gaps: u64,
}

impl ObservationAccounting {
    pub fn complete(self) -> bool {
        self.dropped_oldest == 0
            && self.truncated == 0
            && self.refused == 0
            && self.unparsed == 0
            && self.projection_gaps == 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationSnapshot {
    pub capacity: usize,
    pub occupancy: usize,
    pub accounting: ObservationAccounting,
}

/// Bounded raw event stream. Full queues evict the oldest event.
pub struct ObservationStream {
    capacity: NonZeroUsize,
    max_payload_bytes: usize,
    next_sequence: u64,
    queue: VecDeque<RawObservation>,
    accounting: ObservationAccounting,
}

impl ObservationStream {
    pub fn new(capacity: NonZeroUsize, max_payload_bytes: usize) -> Self {
        Self {
            capacity,
            max_payload_bytes,
            next_sequence: 1,
            queue: VecDeque::with_capacity(capacity.get()),
            accounting: ObservationAccounting::default(),
        }
    }

    pub fn push(&mut self, mut observation: RawObservation) {
        self.accounting.admitted = self.accounting.admitted.saturating_add(1);
        observation.version = OBSERVATION_VERSION;
        observation.sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        let replacement = match &observation.payload {
            PayloadState::Complete(payload) if payload.len() > self.max_payload_bytes => {
                Some((payload[..self.max_payload_bytes].to_vec(), payload.len()))
            }
            PayloadState::Truncated {
                retained,
                original_len,
            } if retained.len() > self.max_payload_bytes => Some((
                retained[..self.max_payload_bytes].to_vec(),
                (*original_len).max(retained.len()),
            )),
            _ => None,
        };
        if let Some((retained, original_len)) = replacement {
            observation.payload = PayloadState::Truncated {
                retained,
                original_len,
            };
            self.accounting.truncated = self.accounting.truncated.saturating_add(1);
        }
        if self.queue.len() == self.capacity.get() {
            self.queue.pop_front();
            self.accounting.dropped_oldest = self.accounting.dropped_oldest.saturating_add(1);
        }
        self.queue.push_back(observation);
    }

    pub fn record_refusal(&mut self) {
        self.accounting.refused = self.accounting.refused.saturating_add(1);
    }

    pub fn record_unparsed(&mut self) {
        self.accounting.unparsed = self.accounting.unparsed.saturating_add(1);
    }

    pub fn record_projection_gap(&mut self) {
        self.accounting.projection_gaps = self.accounting.projection_gaps.saturating_add(1);
    }

    pub fn pop(&mut self) -> Option<RawObservation> {
        let item = self.queue.pop_front();
        if item.is_some() {
            self.accounting.emitted = self.accounting.emitted.saturating_add(1);
        }
        item
    }

    pub fn drain(&mut self) -> Vec<RawObservation> {
        let drained: Vec<_> = self.queue.drain(..).collect();
        self.accounting.emitted = self.accounting.emitted.saturating_add(drained.len() as u64);
        drained
    }

    pub fn snapshot(&self) -> ObservationSnapshot {
        ObservationSnapshot {
            capacity: self.capacity.get(),
            occupancy: self.queue.len(),
            accounting: self.accounting,
        }
    }
}
