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

use crate::interface::InterfaceId;
use crate::parse::ParseReject;

/// One counter per way header parsing declined to produce a flow key, plus the
/// two outcomes that are not declines but still need saying.
///
/// Separate from [`CaptureStats`] and held by it, for the same reason
/// [`SourceStats`] is: three vocabularies flattened into one struct is three
/// vocabularies a reader has to disentangle.
///
/// The twelve rejection counters correspond one to one with
/// [`ParseReject`]'s variants, and that enumeration is closed so the
/// correspondence cannot silently gain a hole. The separation between them is
/// drawn where the remedy differs, which is what constitution P-4 is for: a
/// short header means raise the snapshot length, a malformed header means a
/// broken sender or a parser defect, an unsupported encapsulation means
/// unexpected traffic, an unsupported link type means an unexpected backend.
///
/// There is deliberately no counter for a successful parse. The count of flow
/// keys produced is the captured count less [`ParseStats::rejected`], and this
/// module's rule against stored totals applies to it as much as to the rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParseStats {
    /// The link type is not one this build parses.
    pub unsupported_link_type: u64,
    /// An Ethernet frame whose type is neither IPv4 nor IPv6, VLAN tags
    /// included.
    pub unsupported_ether_type: u64,
    /// A loopback frame whose address family value is unrecognized in either
    /// byte order.
    pub unsupported_address_family: u64,
    /// A raw IP frame whose version nibble is neither 4 nor 6.
    pub unsupported_ip_version: u64,
    /// A network header whose own fields contradict each other.
    pub malformed_network_header: u64,
    /// An IPv6 extension header chain longer than the bound, or one whose
    /// declared length would not advance the walk.
    pub extension_chain_too_long: u64,
    /// An IPv6 chain terminating in the no-next-header value, meaning a
    /// well-formed packet that legitimately carries no transport.
    pub no_next_header: u64,
    /// A transport protocol that is neither TCP nor UDP, encrypted payloads
    /// included.
    pub unsupported_transport: u64,
    /// A transport header whose own fields contradict each other.
    pub malformed_transport_header: u64,
    /// The captured bytes ended before a field the parser needed. The usual
    /// cause is a snapshot length, which is the operator's own choice.
    pub short_header: u64,
    /// A non-initial fragment whose first fragment was never recorded.
    pub unmatched_fragment: u64,
    /// Neither endpoint is on the capturing host, so there is no endpoint to
    /// put in the flow key's local position. Specification section 8.4 defines
    /// that position as the endpoint on the capturing host, so inventing one
    /// would be a false statement rather than an approximation.
    pub no_local_endpoint: u64,

    /// Both endpoints are on the capturing host, so section 12.6's rule
    /// returns both answers. A flow key was produced and the direction was
    /// left undetermined, to be resolved from the attributed process's
    /// endpoint in a later slice. Not a rejection.
    pub direction_ambiguous: u64,
    /// The fragment identity table dropped an entry to admit a newer one. Not
    /// a rejection, though it may cause an `unmatched_fragment` later.
    pub fragment_evicted: u64,
}

impl ParseStats {
    /// Everything that ended without a flow key. Computed from the twelve
    /// named counters, never stored, so it cannot drift from them.
    ///
    /// Excludes `direction_ambiguous`, which accompanies a successful parse,
    /// and `fragment_evicted`, which is a table event rather than an outcome.
    pub fn rejected(&self) -> u64 {
        self.counters()[..12]
            .iter()
            .fold(0, |a, b| a.saturating_add(*b))
    }

    /// Whether any frame failed to yield a flow key.
    pub fn rejected_anything(&self) -> bool {
        self.rejected() > 0
    }

    /// Increment the counter for one rejection cause, and only that one.
    pub(crate) fn record_reject(&mut self, reject: ParseReject) {
        let slot = match reject {
            ParseReject::UnsupportedLinkType => &mut self.unsupported_link_type,
            ParseReject::UnsupportedEtherType => &mut self.unsupported_ether_type,
            ParseReject::UnsupportedAddressFamily => &mut self.unsupported_address_family,
            ParseReject::UnsupportedIpVersion => &mut self.unsupported_ip_version,
            ParseReject::MalformedNetworkHeader => &mut self.malformed_network_header,
            ParseReject::ExtensionChainTooLong => &mut self.extension_chain_too_long,
            ParseReject::NoNextHeader => &mut self.no_next_header,
            ParseReject::UnsupportedTransport => &mut self.unsupported_transport,
            ParseReject::MalformedTransportHeader => &mut self.malformed_transport_header,
            ParseReject::ShortHeader => &mut self.short_header,
            ParseReject::UnmatchedFragment => &mut self.unmatched_fragment,
            ParseReject::NoLocalEndpoint => &mut self.no_local_endpoint,
        };
        *slot = slot.saturating_add(1);
    }

    /// Fold another parser's counters into these, for a run whose capture
    /// threads each own a parser.
    pub fn absorb(&mut self, other: ParseStats) {
        let mut fields: [&mut u64; 14] = [
            &mut self.unsupported_link_type,
            &mut self.unsupported_ether_type,
            &mut self.unsupported_address_family,
            &mut self.unsupported_ip_version,
            &mut self.malformed_network_header,
            &mut self.extension_chain_too_long,
            &mut self.no_next_header,
            &mut self.unsupported_transport,
            &mut self.malformed_transport_header,
            &mut self.short_header,
            &mut self.unmatched_fragment,
            &mut self.no_local_endpoint,
            &mut self.direction_ambiguous,
            &mut self.fragment_evicted,
        ];
        for (slot, value) in fields.iter_mut().zip(other.counters()) {
            **slot = slot.saturating_add(value);
        }
    }

    /// Every counter in a fixed order, the twelve rejections first.
    ///
    /// Exists so a test can diff the whole set and assert that exactly one
    /// counter moved. Asserting only that the expected counter advanced would
    /// pass a parser that advances three.
    pub(crate) fn counters(&self) -> [u64; 14] {
        [
            self.unsupported_link_type,
            self.unsupported_ether_type,
            self.unsupported_address_family,
            self.unsupported_ip_version,
            self.malformed_network_header,
            self.extension_chain_too_long,
            self.no_next_header,
            self.unsupported_transport,
            self.malformed_transport_header,
            self.short_header,
            self.unmatched_fragment,
            self.no_local_endpoint,
            self.direction_ambiguous,
            self.fragment_evicted,
        ]
    }
}

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    /// Each capture backend's own report, unaltered, one entry per interface.
    ///
    /// Per-interface rather than capture-wide because each handle has its own
    /// driver buffer, so `kernel_dropped` was always a per-interface quantity;
    /// there was simply never a second interface to reveal it. Folding them
    /// would tell an operator that a driver buffer is undersized without saying
    /// which, and choosing the remedy is the entire reason constitution P-4
    /// separates counters by cause.
    ///
    /// The capture-wide view is [`CaptureStats::source`], a method rather than
    /// a field, so it cannot drift from the entries it sums. That follows this
    /// module's standing rule against stored totals.
    ///
    /// **A recorded deviation**, found while planning slice S09 rather than
    /// before it. Promoted to specification section 29.
    pub sources: Vec<(InterfaceId, SourceStats)>,
    /// What header parsing concluded, per slice S03.
    ///
    /// No field of this contributes to any drop total. A packet that produced
    /// no flow key is retained and marked, per P-4, and reporting it as loss
    /// would be a P-9 problem rather than only an arithmetic one.
    pub parse: ParseStats,
}

impl CaptureStats {
    /// Every backend's report summed into one, for the capture-wide view.
    ///
    /// Computed, never stored, so it cannot disagree with the per-interface
    /// entries. Each backend reports `u32` counts which widen to `u64` here
    /// before summing, so a long capture across several busy interfaces cannot
    /// total to a number smaller than one of its parts.
    pub fn source(&self) -> SourceStats {
        self.sources
            .iter()
            .fold(SourceStats::default(), |acc, (_, s)| SourceStats {
                received: acc.received.saturating_add(s.received),
                kernel_dropped: acc.kernel_dropped.saturating_add(s.kernel_dropped),
                interface_dropped: acc.interface_dropped.saturating_add(s.interface_dropped),
            })
    }

    /// One interface's report, for an operator deciding which driver buffer to
    /// enlarge.
    pub fn source_for(&self, id: InterfaceId) -> Option<SourceStats> {
        self.sources.iter().find(|(i, _)| *i == id).map(|(_, s)| *s)
    }

    /// Record a backend's report against its interface, replacing any earlier
    /// one. The backend's counts are cumulative from the start of its run, so
    /// the latest report supersedes rather than adds to the previous.
    pub fn set_source(&mut self, id: InterfaceId, stats: SourceStats) {
        match self.sources.iter_mut().find(|(i, _)| *i == id) {
            Some((_, slot)) => *slot = stats,
            None => self.sources.push((id, stats)),
        }
    }

    /// Fold another capture thread's counters into these.
    ///
    /// Every counter here is a count of events, so several capture threads'
    /// counts add. `buffer_dropped` and `sink_dropped` are deliberately not
    /// touched: the buffer and the sinks are capture-wide, the output side owns
    /// both counters, and adding them per thread would multiply one eviction by
    /// the number of interfaces running.
    pub fn absorb(&mut self, other: CaptureStats) {
        self.packets_captured = self.packets_captured.saturating_add(other.packets_captured);
        self.packets_attributed = self
            .packets_attributed
            .saturating_add(other.packets_attributed);
        self.packets_unattributed = self
            .packets_unattributed
            .saturating_add(other.packets_unattributed);
        self.filter_gaps = self.filter_gaps.saturating_add(other.filter_gaps);
        self.parse.absorb(other.parse);
        for (id, stats) in other.sources {
            self.set_source(id, stats);
        }
    }

    /// What fragcap itself discarded. Computed, not stored.
    pub fn fragcap_dropped(&self) -> u64 {
        self.buffer_dropped.saturating_add(self.sink_dropped)
    }

    /// Everything lost anywhere, by fragcap or before it. Computed, not stored,
    /// so it cannot disagree with the named counters it sums.
    pub fn total_dropped(&self) -> u64 {
        self.fragcap_dropped()
            .saturating_add(self.source().total_dropped())
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
            sources: vec![(
                InterfaceId::default(),
                SourceStats {
                    received: 184_240,
                    kernel_dropped: 11,
                    interface_dropped: 3,
                },
            )],
            parse: ParseStats {
                short_header: 4,
                unsupported_transport: 12,
                no_local_endpoint: 1,
                direction_ambiguous: 6,
                ..ParseStats::default()
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
        assert_eq!(s.source().kernel_dropped, 11);
        assert_eq!(s.source().interface_dropped, 3);
    }

    #[test]
    fn totals_are_computed_from_the_named_counters() {
        let s = sample();
        assert_eq!(s.fragcap_dropped(), 7);
        assert_eq!(s.source().total_dropped(), 14);
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
        assert_eq!(s.source().received, 184_240);
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
            |s: &mut CaptureStats| {
                s.set_source(
                    InterfaceId::default(),
                    SourceStats {
                        kernel_dropped: 1,
                        ..SourceStats::default()
                    },
                )
            },
            |s: &mut CaptureStats| {
                s.set_source(
                    InterfaceId::default(),
                    SourceStats {
                        interface_dropped: 1,
                        ..SourceStats::default()
                    },
                )
            },
        ] {
            let mut s = CaptureStats::default();
            set(&mut s);
            assert!(s.lost_anything(), "each cause alone must register as loss");
        }
    }

    // FR-036 and SC-008. No parse outcome is a drop, so no parse counter may
    // reach either drop total. Asserted at the type level, before any parser
    // exists to get it wrong.
    // T015, FR-029. The capture-wide view is a sum of its parts and cannot
    // drift from them, and one interface's report is readable on its own, which
    // is what lets an operator tell which driver buffer is undersized.
    #[test]
    fn the_capture_wide_source_view_is_the_sum_of_its_interfaces() {
        let mut s = CaptureStats::default();
        s.set_source(
            InterfaceId::new(0),
            SourceStats {
                received: 100,
                kernel_dropped: 7,
                interface_dropped: 1,
            },
        );
        s.set_source(
            InterfaceId::new(1),
            SourceStats {
                received: 50,
                kernel_dropped: 2,
                interface_dropped: 0,
            },
        );
        assert_eq!(s.source().received, 150);
        assert_eq!(s.source().kernel_dropped, 9);
        assert_eq!(s.source().interface_dropped, 1);
    }

    #[test]
    fn one_interfaces_report_is_readable_on_its_own() {
        let mut s = CaptureStats::default();
        s.set_source(
            InterfaceId::new(0),
            SourceStats {
                kernel_dropped: 7,
                ..SourceStats::default()
            },
        );
        s.set_source(InterfaceId::new(1), SourceStats::default());
        assert_eq!(s.source_for(InterfaceId::new(0)).unwrap().kernel_dropped, 7);
        assert_eq!(s.source_for(InterfaceId::new(1)).unwrap().kernel_dropped, 0);
        assert_eq!(s.source_for(InterfaceId::new(9)), None);
    }

    #[test]
    fn changing_one_interfaces_report_changes_the_capture_wide_view() {
        // The assertion a stored total would fail: it would not move.
        let mut s = CaptureStats::default();
        s.set_source(InterfaceId::new(0), SourceStats::default());
        let before = s.source().kernel_dropped;
        s.set_source(
            InterfaceId::new(0),
            SourceStats {
                kernel_dropped: 12,
                ..SourceStats::default()
            },
        );
        assert_eq!(s.source().kernel_dropped, before + 12);
    }

    // SC-007. The backend's counts and fragcap's own must stay separable, so
    // that an operator can tell an undersized driver buffer from a slow sink.
    #[test]
    fn a_driver_count_never_reaches_a_fragcap_counter() {
        let mut s = CaptureStats::default();
        s.set_source(
            InterfaceId::new(0),
            SourceStats {
                received: 900,
                kernel_dropped: 40,
                interface_dropped: 5,
            },
        );
        assert_eq!(s.fragcap_dropped(), 0, "no driver count is fragcap's own");
        assert_eq!(s.total_dropped(), 45);

        s.buffer_dropped = 3;
        assert_eq!(s.fragcap_dropped(), 3);
        assert_eq!(
            s.source().kernel_dropped,
            40,
            "a fragcap counter must not alter what the backend reported"
        );
    }

    #[test]
    fn no_parse_counter_contributes_to_any_drop_total() {
        let mut s = CaptureStats::default();
        for i in 0..s.parse.counters().len() {
            let mut p = ParseStats::default();
            // Advance one counter at a time by writing through the same field
            // order `counters` reports, so a field added without being wired
            // in here shows up as a mismatch below rather than passing.
            set_nth(&mut p, i);
            s.parse = p;
            assert_eq!(
                s.fragcap_dropped(),
                0,
                "parse counter {i} reached fragcap's drop total"
            );
            assert_eq!(
                s.total_dropped(),
                0,
                "parse counter {i} reached the overall drop total"
            );
            assert!(!s.lost_anything(), "a parse outcome is not loss");
        }
    }

    fn set_nth(p: &mut ParseStats, n: usize) {
        let fields: [&mut u64; 14] = [
            &mut p.unsupported_link_type,
            &mut p.unsupported_ether_type,
            &mut p.unsupported_address_family,
            &mut p.unsupported_ip_version,
            &mut p.malformed_network_header,
            &mut p.extension_chain_too_long,
            &mut p.no_next_header,
            &mut p.unsupported_transport,
            &mut p.malformed_transport_header,
            &mut p.short_header,
            &mut p.unmatched_fragment,
            &mut p.no_local_endpoint,
            &mut p.direction_ambiguous,
            &mut p.fragment_evicted,
        ];
        *fields[n] = 1;
    }

    #[test]
    fn the_parse_total_counts_rejections_and_nothing_else() {
        let mut p = ParseStats {
            short_header: 3,
            no_local_endpoint: 2,
            ..ParseStats::default()
        };
        assert_eq!(p.rejected(), 5);
        assert!(p.rejected_anything());

        // Neither of these ended without a flow key, so neither is a
        // rejection, and folding them in would overstate the failure rate.
        p.direction_ambiguous = 100;
        p.fragment_evicted = 100;
        assert_eq!(p.rejected(), 5);
    }

    #[test]
    fn an_ambiguous_direction_alone_is_not_a_rejection() {
        let p = ParseStats {
            direction_ambiguous: 9,
            ..ParseStats::default()
        };
        assert_eq!(p.rejected(), 0);
        assert!(!p.rejected_anything());
    }

    #[test]
    fn every_reject_variant_maps_to_a_distinct_counter() {
        // The correspondence P-4 depends on. If two variants ever shared a
        // slot, one cause would be invisible.
        use crate::parse::ParseReject::*;
        let variants = [
            UnsupportedLinkType,
            UnsupportedEtherType,
            UnsupportedAddressFamily,
            UnsupportedIpVersion,
            MalformedNetworkHeader,
            ExtensionChainTooLong,
            NoNextHeader,
            UnsupportedTransport,
            MalformedTransportHeader,
            ShortHeader,
            UnmatchedFragment,
            NoLocalEndpoint,
        ];
        for (i, variant) in variants.into_iter().enumerate() {
            let mut p = ParseStats::default();
            p.record_reject(variant);
            let moved: Vec<usize> = p
                .counters()
                .iter()
                .enumerate()
                .filter(|(_, c)| **c != 0)
                .map(|(n, _)| n)
                .collect();
            assert_eq!(
                moved,
                vec![i],
                "{} moved counters {moved:?}, expected only {i}",
                variant.as_str()
            );
            assert_eq!(p.rejected(), 1);
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
