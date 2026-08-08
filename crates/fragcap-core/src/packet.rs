// SPDX-License-Identifier: Apache-2.0

//! Observations: timestamps, raw packets, and packets carrying whatever the
//! pipeline resolved about them.
//!
//! Constitution P-9 binds this module more tightly than any other. Nothing here
//! offers a way to alter, mask, truncate, reorder, or withhold an observation.
//! The one place the temptation arises is truncation: `orig_len` is kept
//! separate from the retained payload precisely so a shortened capture says so.

use crate::attribution::Attribution;
use crate::flow::{Direction, FlowKey};

/// The packet payload type.
///
/// Aliased rather than used directly so the choice has one name. `bytes::Bytes`
/// clones by bumping a reference count, which matters because a payload is
/// cloned from the capture thread into a bounded ring and then fanned out to
/// several sinks. See slice S02 plan decision D-1.
pub type Payload = bytes::Bytes;

/// An instant, as a count of nanoseconds since the Unix epoch.
///
/// One canonical resolution, deliberately. Specification section 12.7 stores
/// microseconds in the capture file, matching the pcapng per-interface declared
/// resolution, but that resolution is a property of the output format and
/// carrying it here would put format knowledge in a crate that constitution P-2
/// requires be platform-neutral and format-neutral.
///
/// Nanoseconds is finer than any capture backend supplies, so converting inward
/// is lossless. There is deliberately **no** conversion to microseconds on this
/// type: the single lossy conversion happens in the pcapng writer at slice S06,
/// where the declared resolution already lives, so P-9 compliance has exactly
/// one site to inspect rather than one per call.
///
/// Signed so that the difference between two timestamps is expressible in the
/// same type, which the session anchor correlation in section 12.7 needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    /// From a raw nanosecond count since the Unix epoch.
    pub const fn from_nanos(nanos: i64) -> Self {
        Timestamp(nanos)
    }

    /// From a seconds and nanoseconds pair, which is the shape a capture driver
    /// reports. Saturates rather than wrapping, because a wrapped timestamp
    /// would be a silently wrong observation.
    pub fn from_parts(secs: i64, nanos: u32) -> Self {
        Timestamp(
            secs.saturating_mul(1_000_000_000)
                .saturating_add(nanos as i64),
        )
    }

    /// The raw nanosecond count. Lossless in both directions.
    pub const fn as_nanos(self) -> i64 {
        self.0
    }

    /// Nanoseconds elapsed from `earlier` to `self`, which may be negative.
    pub const fn nanos_since(self, earlier: Timestamp) -> i64 {
        self.0 - earlier.0
    }
}

/// An observation as acquired, before anything has been resolved about it.
///
/// Produced by a [`crate::traits::PacketSource`]. Carries no attribution and no
/// flow key, because neither has been computed yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPacket {
    /// When the capture driver observed the frame. Section 12.7: the driver's
    /// timestamp is used rather than a fragcap-applied one, because it is
    /// closer to interface arrival and is therefore more accurate.
    pub ts: Timestamp,
    /// The bytes retained. May be shorter than the frame was on the wire, if a
    /// snapshot length was in effect.
    pub data: Payload,
    /// How long the frame was on the wire, before any snapshot limit.
    ///
    /// Kept separate from `data.len()` so that truncation is self-describing.
    /// The operator choosing a snapshot length is scope, which P-9 permits and
    /// which is visible in their own invocation. A record that failed to say
    /// truncation happened would not be.
    pub orig_len: u32,
}

impl RawPacket {
    pub fn new(ts: Timestamp, data: Payload, orig_len: u32) -> Self {
        RawPacket { ts, data, orig_len }
    }

    /// How many bytes were retained.
    pub fn captured_len(&self) -> usize {
        self.data.len()
    }

    /// Whether fewer bytes were retained than were on the wire.
    ///
    /// Derived from the two fields rather than stored, so it cannot disagree
    /// with them.
    pub fn is_truncated(&self) -> bool {
        (self.orig_len as usize) > self.data.len()
    }
}

/// Which of three states a packet's attribution is in.
///
/// Derived from the flow key and the attribution read together, not stored. See
/// slice S02 plan decision D-5: an explicit field would add a discriminant to a
/// per-packet struct to hold information the struct already carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributionState {
    /// No flow key, so there was nothing to attempt attribution with.
    NotAttempted,
    /// A flow key was present and attribution did not resolve. The packet is
    /// retained and marked, per constitution P-4, never dropped.
    Unresolved,
    /// Attribution resolved.
    Resolved,
}

/// An observation plus whatever the pipeline resolved about it.
///
/// Every added field is optional, because resolution can fail and a failure
/// must not discard the observation. Absent means "not resolved", never "not
/// applicable".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedPacket {
    pub ts: Timestamp,
    pub data: Payload,
    pub orig_len: u32,
    /// Populated by header parsing in slice S03.
    pub flow: Option<FlowKey>,
    /// Populated by header parsing in slice S03.
    pub direction: Option<Direction>,
    /// Populated by the socket table attributor in slice S10.
    pub attribution: Option<Attribution>,
}

impl CapturedPacket {
    /// Lift a raw packet, with nothing yet resolved.
    pub fn from_raw(raw: RawPacket) -> Self {
        CapturedPacket {
            ts: raw.ts,
            data: raw.data,
            orig_len: raw.orig_len,
            flow: None,
            direction: None,
            attribution: None,
        }
    }

    pub fn captured_len(&self) -> usize {
        self.data.len()
    }

    pub fn is_truncated(&self) -> bool {
        (self.orig_len as usize) > self.data.len()
    }

    /// Which of the three attribution states this packet is in.
    ///
    /// The mapping is pinned by a test, because it is the contract that lets a
    /// consumer tell "we never looked" from "we looked and found nothing", and
    /// P-4 requires an unattributed packet be marked rather than merely absent
    /// from the attributed count.
    pub fn attribution_state(&self) -> AttributionState {
        match (self.flow.is_some(), self.attribution.is_some()) {
            (_, true) => AttributionState::Resolved,
            (true, false) => AttributionState::Unresolved,
            (false, false) => AttributionState::NotAttempted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribution::Attribution;
    use crate::flow::{FlowKey, Proto};
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn key() -> FlowKey {
        FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
    }

    // Timestamp fidelity. Supports FR-011.
    #[test]
    fn nanoseconds_round_trip_without_loss() {
        for n in [0_i64, 1, -1, 1_234_567_891, i64::MAX, i64::MIN] {
            assert_eq!(Timestamp::from_nanos(n).as_nanos(), n);
        }
    }

    #[test]
    fn a_sub_microsecond_value_is_not_rounded_away() {
        // If a resolution conversion ever crept into this type, this is the
        // assertion that would catch it.
        let t = Timestamp::from_parts(1, 999);
        assert_eq!(t.as_nanos(), 1_000_000_999);
    }

    #[test]
    fn parts_saturate_rather_than_wrap() {
        let t = Timestamp::from_parts(i64::MAX, 999_999_999);
        assert_eq!(t.as_nanos(), i64::MAX);
    }

    #[test]
    fn a_difference_may_be_negative() {
        let a = Timestamp::from_nanos(10);
        let b = Timestamp::from_nanos(25);
        assert_eq!(a.nanos_since(b), -15);
    }

    // V-5. Truncation is self-describing.
    #[test]
    fn a_truncated_packet_reports_its_original_length() {
        let p = RawPacket::new(
            Timestamp::from_nanos(0),
            Payload::from_static(&[1, 2, 3]),
            1514,
        );
        assert_eq!(p.captured_len(), 3);
        assert_eq!(p.orig_len, 1514);
        assert!(p.is_truncated());
    }

    #[test]
    fn an_untruncated_packet_says_so() {
        let p = RawPacket::new(
            Timestamp::from_nanos(0),
            Payload::from_static(&[1, 2, 3]),
            3,
        );
        assert!(!p.is_truncated());
    }

    #[test]
    fn truncation_survives_being_lifted_into_a_captured_packet() {
        let raw = RawPacket::new(
            Timestamp::from_nanos(7),
            Payload::from_static(&[9; 64]),
            1514,
        );
        let cap = CapturedPacket::from_raw(raw);
        assert_eq!(cap.orig_len, 1514);
        assert_eq!(cap.captured_len(), 64);
        assert!(cap.is_truncated());
    }

    // FR-029, P-9. There is no operation that makes a truncated packet claim it
    // was whole. `is_truncated` is derived, so the only way to change the answer
    // is to change the observation itself, which no method here does.
    #[test]
    fn nothing_can_make_a_truncated_packet_claim_completeness() {
        let p = RawPacket::new(
            Timestamp::from_nanos(0),
            Payload::from_static(&[0; 10]),
            1514,
        );
        assert!(p.is_truncated());
        let clone = p.clone();
        assert!(clone.is_truncated(), "cloning must not launder truncation");
        assert_eq!(clone.orig_len, p.orig_len);
    }

    // V-9. The three states, each constructed and read back.
    #[test]
    fn no_flow_key_reads_as_never_attempted() {
        let p =
            CapturedPacket::from_raw(RawPacket::new(Timestamp::from_nanos(0), Payload::new(), 0));
        assert_eq!(p.attribution_state(), AttributionState::NotAttempted);
    }

    #[test]
    fn a_flow_key_without_attribution_reads_as_unresolved() {
        let mut p =
            CapturedPacket::from_raw(RawPacket::new(Timestamp::from_nanos(0), Payload::new(), 0));
        p.flow = Some(key());
        assert_eq!(p.attribution_state(), AttributionState::Unresolved);
    }

    #[test]
    fn a_flow_key_with_attribution_reads_as_resolved() {
        let mut p =
            CapturedPacket::from_raw(RawPacket::new(Timestamp::from_nanos(0), Payload::new(), 0));
        p.flow = Some(key());
        p.attribution = Some(Attribution::new(4242, "eso64.exe"));
        assert_eq!(p.attribution_state(), AttributionState::Resolved);
    }

    #[test]
    fn unresolved_is_distinguishable_from_never_attempted() {
        // The distinction P-4 needs: a packet retained and marked, versus a
        // packet nobody could have attributed.
        let mut attempted =
            CapturedPacket::from_raw(RawPacket::new(Timestamp::from_nanos(0), Payload::new(), 0));
        attempted.flow = Some(key());
        let not_attempted =
            CapturedPacket::from_raw(RawPacket::new(Timestamp::from_nanos(0), Payload::new(), 0));
        assert_ne!(
            attempted.attribution_state(),
            not_attempted.attribution_state()
        );
    }
}
