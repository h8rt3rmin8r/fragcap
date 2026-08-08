// SPDX-License-Identifier: Apache-2.0

//! Counters.
//!
//! Constitution P-4 is the reason this module has the shape it does: every
//! discarded packet is counted in a named counter and surfaced, and adding a
//! discard path without a counter is a defect rather than an oversight.
//!
//! Two rules follow, and both are structural rather than documented.
//!
//! One counter per cause, named as specification section 12.4 names them. A
//! single `dropped` field would satisfy the letter of P-4 and defeat its
//! purpose, because the remedy differs by cause: kernel drops mean an
//! undersized driver buffer, buffer drops mean a slow sink, sink drops mean a
//! slow consumer downstream of fragcap.
//!
//! No stored totals. Every aggregate is a method over the named counters, so a
//! total cannot drift from its parts.

/// What a capture backend reports about itself.
///
/// These are another component's observations, which fragcap relays rather than
/// owns. They are kept in their own type so that fragcap's accounting is never
/// folded into them: doing so would alter what that component said, which
/// constitution P-9 forbids just as much as altering a packet would.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceStats {
    /// Frames the backend saw.
    pub received: u64,
    /// Frames the capture driver dropped before fragcap, per specification
    /// section 12.4. Indicates an undersized driver buffer.
    pub kernel_dropped: u64,
    /// Frames the interface dropped before the driver saw them.
    pub interface_dropped: u64,
}

impl SourceStats {
    /// Everything lost before fragcap could see it. Computed, not stored.
    pub fn total_dropped(&self) -> u64 {
        self.kernel_dropped.saturating_add(self.interface_dropped)
    }
}

/// What fragcap's own pipeline counted.
///
/// Holds the backend's report by value rather than merging its fields, so an
/// operator can tell where loss happened and therefore what to change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CaptureStats {
    /// Packets fragcap accepted from the source.
    pub packets_captured: u64,
    /// Packets that resolved to a process.
    pub packets_attributed: u64,
    /// Packets retained and marked because attribution did not resolve, per
    /// P-4. Never dropped for this reason.
    pub packets_unattributed: u64,
    /// Dropped by fragcap's bounded buffer, per specification section 12.4.
    /// Indicates a slow sink.
    pub buffer_dropped: u64,
    /// Dropped by a sink that could not accept, per specification section 12.4.
    /// Indicates a slow consumer downstream of fragcap.
    pub sink_dropped: u64,
    /// Packets that passed while a filter was being narrowed, per the summary
    /// output in specification section 13.
    pub filter_gaps: u64,
    /// The backend's own report, unaltered.
    pub source: SourceStats,
}

impl CaptureStats {
    /// What fragcap itself discarded. Computed, not stored.
    pub fn fragcap_dropped(&self) -> u64 {
        self.buffer_dropped.saturating_add(self.sink_dropped)
    }

    /// Everything lost anywhere, by fragcap or before it. Computed, not stored,
    /// so it cannot disagree with the named counters it sums.
    pub fn total_dropped(&self) -> u64 {
        self.fragcap_dropped()
            .saturating_add(self.source.total_dropped())
    }

    /// Whether anything was lost at all. The question an operator asks first.
    pub fn lost_anything(&self) -> bool {
        self.total_dropped() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CaptureStats {
        CaptureStats {
            packets_captured: 184_229,
            packets_attributed: 183_901,
            packets_unattributed: 328,
            buffer_dropped: 5,
            sink_dropped: 2,
            filter_gaps: 2,
            source: SourceStats {
                received: 184_240,
                kernel_dropped: 11,
                interface_dropped: 3,
            },
        }
    }

    // V-6. An individual cause is assertable on its own, which is the property
    // P-4 actually needs. A single aggregate would pass a total assertion and
    // still leave an operator unable to choose a remedy.
    #[test]
    fn each_discard_cause_is_assertable_alone() {
        let s = sample();
        assert_eq!(s.buffer_dropped, 5);
        assert_eq!(s.sink_dropped, 2);
        assert_eq!(s.source.kernel_dropped, 11);
        assert_eq!(s.source.interface_dropped, 3);
    }

    #[test]
    fn totals_are_computed_from_the_named_counters() {
        let s = sample();
        assert_eq!(s.fragcap_dropped(), 7);
        assert_eq!(s.source.total_dropped(), 14);
        assert_eq!(s.total_dropped(), 21);
    }

    #[test]
    fn changing_one_cause_changes_the_total() {
        // The assertion that a stored total would fail: it would not move.
        let mut s = sample();
        let before = s.total_dropped();
        s.sink_dropped += 100;
        assert_eq!(s.total_dropped(), before + 100);
    }

    #[test]
    fn backend_counts_are_not_folded_into_fragcap_counts() {
        let s = sample();
        assert_ne!(
            s.fragcap_dropped(),
            s.total_dropped(),
            "fragcap's own drops must exclude the backend's"
        );
        assert_eq!(s.source.received, 184_240);
    }

    #[test]
    fn a_clean_capture_reports_no_loss() {
        assert!(!CaptureStats::default().lost_anything());
    }

    #[test]
    fn any_single_cause_makes_the_capture_lossy() {
        for set in [
            |s: &mut CaptureStats| s.buffer_dropped = 1,
            |s: &mut CaptureStats| s.sink_dropped = 1,
            |s: &mut CaptureStats| s.source.kernel_dropped = 1,
            |s: &mut CaptureStats| s.source.interface_dropped = 1,
        ] {
            let mut s = CaptureStats::default();
            set(&mut s);
            assert!(s.lost_anything(), "each cause alone must register as loss");
        }
    }

    #[test]
    fn unattributed_packets_are_counted_not_dropped() {
        // P-4: unattributed packets are retained and marked. They appear in
        // their own counter and in none of the drop counters.
        let s = sample();
        assert_eq!(s.packets_unattributed, 328);
        assert_eq!(
            s.packets_attributed + s.packets_unattributed,
            s.packets_captured
        );
    }
}
