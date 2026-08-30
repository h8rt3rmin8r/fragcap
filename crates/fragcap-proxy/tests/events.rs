// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;

use fragcap_proxy::{ObservationKind, ObservationStream, PayloadState, RawObservation};

fn event(payload: &[u8]) -> RawObservation {
    RawObservation {
        version: 0,
        session_id: "test-session".into(),
        connection_id: Some(7),
        sequence: 0,
        timestamp_ns: 42,
        provenance: "controlled-lab".into(),
        kind: ObservationKind::Message,
        payload: PayloadState::Complete(payload.to_vec()),
    }
}

#[test]
fn queue_drops_oldest_preserves_order_and_marks_incomplete() {
    let mut stream = ObservationStream::new(NonZeroUsize::new(2).unwrap(), 4);
    stream.push(event(b"one"));
    stream.push(event(b"two"));
    stream.push(event(b"three"));
    let snapshot = stream.snapshot();
    assert_eq!(snapshot.occupancy, 2);
    assert_eq!(snapshot.accounting.dropped_oldest, 1);
    assert!(!snapshot.accounting.complete());
    let events = stream.drain();
    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
}

#[test]
fn payload_truncation_retains_original_length_and_conserves_counts() {
    let mut stream = ObservationStream::new(NonZeroUsize::new(2).unwrap(), 4);
    stream.push(event(b"123456"));
    let record = stream.pop().unwrap();
    assert_eq!(
        record.payload,
        PayloadState::Truncated {
            retained: b"1234".to_vec(),
            original_len: 6
        }
    );
    let accounting = stream.snapshot().accounting;
    assert_eq!(
        accounting.admitted,
        accounting.emitted + accounting.dropped_oldest
    );
    assert_eq!(accounting.truncated, 1);
    assert!(!accounting.complete());
}

#[test]
fn refused_unparsed_and_projection_gaps_are_distinct() {
    let mut stream = ObservationStream::new(NonZeroUsize::new(1).unwrap(), 0);
    stream.record_refusal();
    stream.record_unparsed();
    stream.record_projection_gap();
    let counts = stream.snapshot().accounting;
    assert_eq!(
        (counts.refused, counts.unparsed, counts.projection_gaps),
        (1, 1, 1)
    );
}
