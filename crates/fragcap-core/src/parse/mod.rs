// SPDX-License-Identifier: Apache-2.0

//! Header parsing: a byte slice and a link type in, a [`FlowKey`] and a
//! [`Direction`] out, per specification sections 12.5 and 12.6.
//!
//! This is the first module in `fragcap-core` that computes rather than
//! declares. It is arithmetic over a borrowed slice: no I/O, no clock, no
//! platform call, and no allocation.
//!
//! # What it refuses to do
//!
//! Three refusals define the module more than the parsing does.
//!
//! **It never guesses a port.** A transport header truncated before both port
//! fields yields no flow key, counted as a short header. A defaulted port
//! produces a key that looks resolvable and is not, which is the confident
//! wrong answer constitution P-9 exists to prevent.
//!
//! **It never guesses a direction.** Section 12.6's rule returns two answers
//! for loopback traffic and none for a packet with no local endpoint. Both are
//! reported as what they are, with separate counters, because the remedies
//! differ and because section 12.6 resolves the first from the attributed
//! process's endpoint in a later slice, which is only possible if this one
//! says so.
//!
//! **It never reassembles.** Section 12.5 refuses it, on the grounds that
//! reassembling during capture would destroy the on-wire fidelity that makes
//! the capture worth taking. Non-initial fragments are attributed from a
//! bounded memory of what their first fragment said, and a fragment that
//! cannot be matched gets no key rather than an invented one.
//!
//! # Accounting
//!
//! Every path that ends without a flow key advances exactly one counter in
//! [`ParseStats`], and [`ParseReject`] is closed so that adding a path without
//! a counter does not compile. No parse outcome is a drop: a packet with no
//! flow key is retained and marked by the caller, per constitution P-4.

mod direction;
mod fragment;
mod ip;
mod link;
mod transport;

use std::net::SocketAddr;

use crate::flow::{Direction, FlowKey};
use crate::link::LinkType;
use crate::packet::CapturedPacket;
use crate::stats::ParseStats;

use fragment::{FragmentPorts, FragmentTable};
use ip::FragmentRole as Role;

pub use direction::InterfaceAddrs;

/// Why a frame produced no flow key.
///
/// Closed rather than `#[non_exhaustive]`, unlike the error enums in
/// [`crate::error`]. Those are extended by later slices adding failure modes.
/// This is the complete set of ways this parser declines, and closing it is
/// what makes "add a decline path, add a counter" a compile error rather than
/// a review note.
///
/// The variants are separated exactly where the remedy differs. That is the
/// point of constitution P-4: an operator seeing one aggregate cannot choose
/// what to change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParseReject {
    /// The link type is not one this build parses. Indicates an unexpected
    /// capture backend.
    UnsupportedLinkType,
    /// An Ethernet frame whose type is neither IPv4 nor IPv6. VLAN tagged
    /// frames land here, per FR-009. Indicates unexpected traffic.
    UnsupportedEtherType,
    /// A BSD loopback frame whose address family value is unrecognized in
    /// either byte order.
    UnsupportedAddressFamily,
    /// A raw IP frame whose version nibble is neither 4 nor 6.
    UnsupportedIpVersion,
    /// A network header whose own fields contradict each other. Indicates a
    /// broken sender or a defect in this parser, never a snapshot length.
    MalformedNetworkHeader,
    /// An IPv6 extension header chain longer than the bound, or one whose
    /// declared length would not advance the walk.
    ExtensionChainTooLong,
    /// An IPv6 chain terminating in the no-next-header value: a well-formed
    /// packet that legitimately carries no transport. Distinct from
    /// [`ParseReject::UnsupportedTransport`], which is a transport fragcap
    /// declined to read.
    NoNextHeader,
    /// A transport protocol other than TCP or UDP. Encrypted payloads land
    /// here: their ports exist and cannot be read.
    UnsupportedTransport,
    /// A transport header whose own fields contradict each other.
    MalformedTransportHeader,
    /// The captured bytes ended before a field the parser needed. The usual
    /// cause is a snapshot length, which is the operator's own choice and is
    /// visible in their own invocation.
    ShortHeader,
    /// A non-initial fragment whose first fragment was never recorded, either
    /// because it was not captured, arrived out of order, or was evicted.
    UnmatchedFragment,
    /// Neither endpoint is on the capturing host. Specification section 8.4
    /// defines a flow key's local field as the endpoint on the capturing host,
    /// and there is not one, so no key is produced rather than a false one.
    /// Indicates a stale interface address set, or traffic that was never
    /// this host's.
    NoLocalEndpoint,
}

impl ParseReject {
    /// The counter name this cause advances, for diagnostics and output.
    pub fn as_str(&self) -> &'static str {
        match self {
            ParseReject::UnsupportedLinkType => "unsupported_link_type",
            ParseReject::UnsupportedEtherType => "unsupported_ether_type",
            ParseReject::UnsupportedAddressFamily => "unsupported_address_family",
            ParseReject::UnsupportedIpVersion => "unsupported_ip_version",
            ParseReject::MalformedNetworkHeader => "malformed_network_header",
            ParseReject::ExtensionChainTooLong => "extension_chain_too_long",
            ParseReject::NoNextHeader => "no_next_header",
            ParseReject::UnsupportedTransport => "unsupported_transport",
            ParseReject::MalformedTransportHeader => "malformed_transport_header",
            ParseReject::ShortHeader => "short_header",
            ParseReject::UnmatchedFragment => "unmatched_fragment",
            ParseReject::NoLocalEndpoint => "no_local_endpoint",
        }
    }
}

/// What the parser concluded about one frame.
///
/// `Copy`, and borrowing nothing from the frame, so the caller may forward or
/// drop the frame the instant this returns.
///
/// The direction is optional inside `Parsed` rather than being a third
/// top-level variant, because an undetermined direction is a successful parse
/// with one field unresolved, not a failure. Which of the two undetermined
/// cases occurred is in the counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Parsed {
        flow: FlowKey,
        direction: Option<Direction>,
    },
    Rejected(ParseReject),
}

impl ParseOutcome {
    pub fn flow(&self) -> Option<FlowKey> {
        match self {
            ParseOutcome::Parsed { flow, .. } => Some(*flow),
            ParseOutcome::Rejected(_) => None,
        }
    }

    pub fn direction(&self) -> Option<Direction> {
        match self {
            ParseOutcome::Parsed { direction, .. } => *direction,
            ParseOutcome::Rejected(_) => None,
        }
    }

    pub fn reject(&self) -> Option<ParseReject> {
        match self {
            ParseOutcome::Parsed { .. } => None,
            ParseOutcome::Rejected(r) => Some(*r),
        }
    }
}

/// Turns frames into flow keys, and says why when it cannot.
///
/// Owned by the caller and driven through `&mut self`. The fragment identity
/// table and the counters are per-capture state, so a free function would make
/// every call site thread three arguments; interior mutability would buy
/// sharing nobody needs, since the capture thread owns exactly one of these
/// and does not share it.
///
/// Deliberately not `Clone`. Two parsers each holding half the first fragments
/// would each miss half the subsequent ones, and the symptom would be an
/// intermittently low attribution rate rather than an obvious failure.
pub struct HeaderParser {
    addrs: InterfaceAddrs,
    fragments: FragmentTable,
    stats: ParseStats,
}

impl HeaderParser {
    pub fn new(addrs: InterfaceAddrs) -> Self {
        HeaderParser {
            addrs,
            fragments: FragmentTable::default(),
            stats: ParseStats::default(),
        }
    }

    /// Replace the interface address set.
    ///
    /// Wholesale rather than incremental, so a stale address has no path by
    /// which to survive a refresh. Deciding when to call this, and obtaining
    /// the addresses, is platform work owned by S09 and S13.
    pub fn set_interface_addrs(&mut self, addrs: InterfaceAddrs) {
        self.addrs = addrs;
    }

    pub fn interface_addrs(&self) -> &InterfaceAddrs {
        &self.addrs
    }

    pub fn stats(&self) -> &ParseStats {
        &self.stats
    }

    /// Parse one frame.
    ///
    /// Never modifies the frame, never reads past it, never allocates, and
    /// always terminates.
    pub fn parse(&mut self, link: LinkType, frame: &[u8]) -> ParseOutcome {
        match self.resolve(link, frame) {
            Ok(outcome) => outcome,
            Err(reject) => {
                self.stats.record_reject(reject);
                ParseOutcome::Rejected(reject)
            }
        }
    }

    /// Parse a captured packet's bytes and write the result onto it.
    ///
    /// Exists because writing `flow` and `direction` from an outcome is the
    /// call site both S04 and S08 need, and two copies of it are two chances
    /// to set one field and forget the other.
    pub fn apply(&mut self, link: LinkType, packet: &mut CapturedPacket) -> ParseOutcome {
        let outcome = self.parse(link, &packet.data);
        packet.flow = outcome.flow();
        packet.direction = outcome.direction();
        outcome
    }

    /// The five validation stages, in the order that fixes which counter fires
    /// when a frame is wrong in more than one way. A frame wrong at more than
    /// one layer is counted at the first.
    fn resolve(&mut self, link: LinkType, frame: &[u8]) -> Result<ParseOutcome, ParseReject> {
        let (net_proto, offset) = link::dispatch(link, frame)?;
        let net_bytes = frame.get(offset..).ok_or(ParseReject::ShortHeader)?;
        let net = ip::parse(net_proto, net_bytes)?;

        let ports = match net.fragment {
            // Not fragmented, or the initial fragment, which carries the
            // transport header.
            None | Some((_, Role::Initial)) => {
                let transport_bytes = net_bytes
                    .get(net.transport_offset..)
                    .ok_or(ParseReject::ShortHeader)?;
                let (proto, src_port, dst_port) = transport::ports(net.proto, transport_bytes)?;
                let ports = FragmentPorts {
                    proto,
                    src_port,
                    dst_port,
                };
                // Recorded only after the transport parsed. An initial
                // fragment whose header could not be read has no ports to
                // remember, per FR-021a, and its later fragments are honestly
                // unmatched rather than matched against a guess.
                if let Some((key, Role::Initial)) = net.fragment {
                    if self.fragments.record(key, ports) {
                        self.stats.fragment_evicted = self.stats.fragment_evicted.saturating_add(1);
                    }
                }
                ports
            }
            // A later fragment. Its transport header is in the first one.
            Some((key, Role::Subsequent)) => self
                .fragments
                .lookup(&key)
                .ok_or(ParseReject::UnmatchedFragment)?,
            // The last fragment. Looking it up also forgets it, so the entry
            // does not outlive the datagram.
            Some((key, Role::Last)) => self
                .fragments
                .take(&key)
                .ok_or(ParseReject::UnmatchedFragment)?,
        };

        let src = SocketAddr::new(net.src, ports.src_port);
        let dst = SocketAddr::new(net.dst, ports.dst_port);
        // Recomputed for every fragment rather than inherited, per FR-022a:
        // every fragment carries the full address pair, and the address set
        // may have changed since the first one was seen.
        let locality = direction::resolve(&self.addrs, src, dst)?;
        if locality.ambiguous {
            self.stats.direction_ambiguous = self.stats.direction_ambiguous.saturating_add(1);
        }

        Ok(ParseOutcome::Parsed {
            flow: FlowKey::new(ports.proto, locality.local, locality.remote),
            direction: locality.direction,
        })
    }
}

/// Frame builders for tests.
///
/// Each takes fields and returns bytes, so a test reads as the packet it
/// describes rather than as a byte array with a comment.
#[cfg(test)]
pub(crate) mod testframe {
    use std::net::{Ipv4Addr, Ipv6Addr};

    /// Append bytes to a frame under construction.
    pub trait WithPayload {
        fn with_payload(self, bytes: &[u8]) -> Vec<u8>;
    }

    impl WithPayload for Vec<u8> {
        fn with_payload(mut self, bytes: &[u8]) -> Vec<u8> {
            self.extend_from_slice(bytes);
            self
        }
    }

    pub fn ethernet(ether_type: u16, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(14 + payload.len());
        out.extend_from_slice(&[0x02, 0, 0, 0, 0, 1]);
        out.extend_from_slice(&[0x02, 0, 0, 0, 0, 2]);
        out.extend_from_slice(&ether_type.to_be_bytes());
        out.extend_from_slice(payload);
        out
    }

    pub fn loopback_raw(family: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + payload.len());
        out.extend_from_slice(family);
        out.extend_from_slice(payload);
        out
    }

    pub struct V4 {
        pub src: Ipv4Addr,
        pub dst: Ipv4Addr,
        pub proto: u8,
        pub ident: u16,
        pub frag_offset: u16,
        pub more_fragments: bool,
        /// Option words, four octets each, beyond the fixed header.
        pub option_words: u8,
    }

    impl Default for V4 {
        fn default() -> Self {
            V4 {
                src: Ipv4Addr::new(192, 0, 2, 10),
                dst: Ipv4Addr::new(198, 51, 100, 5),
                proto: 6,
                ident: 0,
                frag_offset: 0,
                more_fragments: false,
                option_words: 0,
            }
        }
    }

    pub fn ipv4(h: V4) -> Vec<u8> {
        let header_words = 5 + h.option_words;
        let header_len = header_words as usize * 4;
        let mut out = vec![0u8; header_len];
        out[0] = 0x40 | header_words;
        out[2..4].copy_from_slice(&(header_len as u16).to_be_bytes());
        out[4..6].copy_from_slice(&h.ident.to_be_bytes());
        let flags = if h.more_fragments { 0x2000 } else { 0 };
        out[6..8].copy_from_slice(&(flags | (h.frag_offset & 0x1fff)).to_be_bytes());
        out[8] = 64;
        out[9] = h.proto;
        out[12..16].copy_from_slice(&h.src.octets());
        out[16..20].copy_from_slice(&h.dst.octets());
        out
    }

    pub struct V6 {
        pub src: Ipv6Addr,
        pub dst: Ipv6Addr,
        /// The fixed header's next header field.
        pub next: u8,
    }

    impl Default for V6 {
        fn default() -> Self {
            V6 {
                src: Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x10),
                dst: Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 5),
                next: 6,
            }
        }
    }

    pub fn ipv6(h: V6) -> Vec<u8> {
        let mut out = vec![0u8; 40];
        out[0] = 0x60;
        out[6] = h.next;
        out[7] = 64;
        out[8..24].copy_from_slice(&h.src.octets());
        out[24..40].copy_from_slice(&h.dst.octets());
        out
    }

    /// A one-unit extension header: eight octets, whatever the encoding.
    ///
    /// The option headers count eight-octet units excluding the first, and the
    /// authentication header counts four-octet units excluding two, so a
    /// length byte of zero means eight octets under both.
    pub fn extension(next: u8) -> Vec<u8> {
        let mut out = vec![0u8; 8];
        out[0] = next;
        out
    }

    pub fn fragment_ext(next: u8, offset: u16, more: bool, ident: u32) -> Vec<u8> {
        let mut out = vec![0u8; 8];
        out[0] = next;
        let flags = u16::from(more);
        out[2..4].copy_from_slice(&((offset << 3) | flags).to_be_bytes());
        out[4..8].copy_from_slice(&ident.to_be_bytes());
        out
    }

    pub fn tcp(src_port: u16, dst_port: u16) -> Vec<u8> {
        let mut out = vec![0u8; 20];
        out[0..2].copy_from_slice(&src_port.to_be_bytes());
        out[2..4].copy_from_slice(&dst_port.to_be_bytes());
        out[12] = 0x50;
        out
    }

    pub fn udp(src_port: u16, dst_port: u16, declared_len: u16) -> Vec<u8> {
        let mut out = vec![0u8; 8];
        out[0..2].copy_from_slice(&src_port.to_be_bytes());
        out[2..4].copy_from_slice(&dst_port.to_be_bytes());
        out[4..6].copy_from_slice(&declared_len.to_be_bytes());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::testframe::{self as f, WithPayload};
    use super::*;
    use crate::flow::Proto;
    use crate::packet::{Payload, RawPacket, Timestamp};
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    const LOCAL_V4: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 10);
    const PEER_V4: Ipv4Addr = Ipv4Addr::new(198, 51, 100, 5);
    const OTHER_V4: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 7);
    const ETHERTYPE_IPV4: u16 = 0x0800;
    const ETHERTYPE_IPV6: u16 = 0x86dd;
    const IPPROTO_TCP: u8 = 6;
    const IPPROTO_UDP: u8 = 17;

    fn local_v6() -> Ipv6Addr {
        Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x10)
    }

    fn parser_with(addrs: &[IpAddr]) -> HeaderParser {
        HeaderParser::new(InterfaceAddrs::new(addrs.iter().copied()))
    }

    fn local_parser() -> HeaderParser {
        parser_with(&[IpAddr::V4(LOCAL_V4), IpAddr::V6(local_v6())])
    }

    /// An Ethernet frame carrying IPv4 and the transport bytes given.
    fn eth_v4(header: f::V4, transport: &[u8]) -> Vec<u8> {
        f::ethernet(ETHERTYPE_IPV4, &f::ipv4(header).with_payload(transport))
    }

    /// Parse one frame and report which counters moved, so a test can assert
    /// that exactly one did.
    ///
    /// This is the harness that makes the rejection tests mean anything.
    /// Asserting only that the expected counter advanced would pass a parser
    /// that advances three, and P-4's whole purpose is that an operator can
    /// read one cause off the statistics and act on it.
    fn parse_counting(
        parser: &mut HeaderParser,
        link: LinkType,
        frame: &[u8],
    ) -> (ParseOutcome, Vec<(usize, u64)>) {
        let before = parser.stats().counters();
        let outcome = parser.parse(link, frame);
        let after = parser.stats().counters();
        let moved = before
            .iter()
            .zip(after.iter())
            .enumerate()
            .filter(|(_, (b, a))| a != b)
            .map(|(i, (b, a))| (i, a - b))
            .collect();
        (outcome, moved)
    }

    /// Assert a frame is rejected for exactly one reason, and that exactly one
    /// counter moved by exactly one.
    fn assert_rejected(parser: &mut HeaderParser, link: LinkType, frame: &[u8], want: ParseReject) {
        let (outcome, moved) = parse_counting(parser, link, frame);
        assert_eq!(
            outcome,
            ParseOutcome::Rejected(want),
            "wrong rejection cause"
        );
        assert_eq!(
            moved.len(),
            1,
            "{} moved {} counters, expected exactly one",
            want.as_str(),
            moved.len()
        );
        assert_eq!(moved[0].1, 1, "a counter moved by more than one");
    }

    // ---- User Story 1: a conversation gets an identity -------------------

    #[test]
    fn an_ethernet_ipv4_tcp_frame_from_the_local_host_is_outbound() {
        let mut p = local_parser();
        let frame = eth_v4(
            f::V4 {
                src: LOCAL_V4,
                dst: PEER_V4,
                proto: IPPROTO_TCP,
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        let outcome = p.parse(LinkType::ETHERNET, &frame);
        assert_eq!(
            outcome,
            ParseOutcome::Parsed {
                flow: FlowKey::new(
                    Proto::Tcp,
                    SocketAddr::new(IpAddr::V4(LOCAL_V4), 51000),
                    SocketAddr::new(IpAddr::V4(PEER_V4), 443),
                ),
                direction: Some(Direction::Outbound),
            }
        );
        assert_eq!(p.stats().rejected(), 0);
    }

    #[test]
    fn the_same_frame_inbound_swaps_the_positions_but_not_the_key() {
        let mut p = local_parser();
        let outbound = p
            .parse(
                LinkType::ETHERNET,
                &eth_v4(
                    f::V4 {
                        src: LOCAL_V4,
                        dst: PEER_V4,
                        proto: IPPROTO_TCP,
                        ..f::V4::default()
                    },
                    &f::tcp(51000, 443),
                ),
            )
            .flow()
            .expect("outbound parses");
        let inbound = p
            .parse(
                LinkType::ETHERNET,
                &eth_v4(
                    f::V4 {
                        src: PEER_V4,
                        dst: LOCAL_V4,
                        proto: IPPROTO_TCP,
                        ..f::V4::default()
                    },
                    &f::tcp(443, 51000),
                ),
            )
            .flow()
            .expect("inbound parses");
        assert_eq!(outbound, inbound, "one conversation is one key");
    }

    // SC-005.
    #[test]
    fn both_directions_of_a_conversation_are_one_map_entry() {
        let mut p = local_parser();
        let mut seen: HashMap<FlowKey, u32> = HashMap::new();
        for (src, dst, sp, dp) in [
            (LOCAL_V4, PEER_V4, 51000, 443),
            (PEER_V4, LOCAL_V4, 443, 51000),
        ] {
            let flow = p
                .parse(
                    LinkType::ETHERNET,
                    &eth_v4(
                        f::V4 {
                            src,
                            dst,
                            proto: IPPROTO_TCP,
                            ..f::V4::default()
                        },
                        &f::tcp(sp, dp),
                    ),
                )
                .flow()
                .expect("both halves parse");
            *seen.entry(flow).or_insert(0) += 1;
        }
        assert_eq!(seen.len(), 1);
        assert_eq!(seen.values().next(), Some(&2));
    }

    #[test]
    fn both_halves_of_a_loopback_conversation_are_one_map_entry() {
        let loop_v4 = Ipv4Addr::LOCALHOST;
        let mut p = parser_with(&[IpAddr::V4(loop_v4)]);
        let mut seen: HashMap<FlowKey, u32> = HashMap::new();
        for (sp, dp) in [(51000u16, 8080u16), (8080, 51000)] {
            let flow = p
                .parse(
                    LinkType::ETHERNET,
                    &eth_v4(
                        f::V4 {
                            src: loop_v4,
                            dst: loop_v4,
                            proto: IPPROTO_TCP,
                            ..f::V4::default()
                        },
                        &f::tcp(sp, dp),
                    ),
                )
                .flow()
                .expect("a loopback frame still gets a key");
            *seen.entry(flow).or_insert(0) += 1;
        }
        assert_eq!(seen.len(), 1, "the ordering rule must be direction-blind");
        assert_eq!(p.stats().direction_ambiguous, 2);
    }

    #[test]
    fn an_ethernet_ipv6_udp_frame_parses() {
        let mut p = local_parser();
        let peer: Ipv6Addr = "2001:db8::5".parse().expect("test address must parse");
        let frame = f::ethernet(
            ETHERTYPE_IPV6,
            &f::ipv6(f::V6 {
                src: local_v6(),
                dst: peer,
                next: IPPROTO_UDP,
            })
            .with_payload(&f::udp(30000, 5055, 8)),
        );
        assert_eq!(
            p.parse(LinkType::ETHERNET, &frame),
            ParseOutcome::Parsed {
                flow: FlowKey::new(
                    Proto::Udp,
                    SocketAddr::new(IpAddr::V6(local_v6()), 30000),
                    SocketAddr::new(IpAddr::V6(peer), 5055),
                ),
                direction: Some(Direction::Outbound),
            }
        );
    }

    #[test]
    fn a_raw_ip_frame_yields_the_same_key_as_its_ethernet_equivalent() {
        let mut p = local_parser();
        let header = f::V4 {
            src: LOCAL_V4,
            dst: PEER_V4,
            proto: IPPROTO_UDP,
            ..f::V4::default()
        };
        let ip = f::ipv4(header).with_payload(&f::udp(30000, 5055, 8));
        let via_raw = p.parse(LinkType::RAW, &ip).flow();
        let via_eth = p
            .parse(LinkType::ETHERNET, &f::ethernet(ETHERTYPE_IPV4, &ip))
            .flow();
        assert_eq!(via_raw, via_eth);
        assert!(via_raw.is_some());
    }

    #[test]
    fn a_bsd_loopback_frame_parses_past_its_address_family_field() {
        let loop_v4 = Ipv4Addr::LOCALHOST;
        let mut p = parser_with(&[IpAddr::V4(loop_v4)]);
        let ip = f::ipv4(f::V4 {
            src: loop_v4,
            dst: loop_v4,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        })
        .with_payload(&f::tcp(51000, 8080));
        let frame = f::loopback_raw(&2u32.to_le_bytes(), &ip);
        let outcome = p.parse(LinkType::NULL, &frame);
        assert!(outcome.flow().is_some(), "the four byte prefix was skipped");
        assert_eq!(outcome.direction(), None, "loopback stays undetermined");
    }

    #[test]
    fn an_extension_header_chain_does_not_move_the_ports() {
        let mut p = local_parser();
        let peer: Ipv6Addr = "2001:db8::5".parse().expect("test address must parse");
        let frame = f::ethernet(
            ETHERTYPE_IPV6,
            &f::ipv6(f::V6 {
                src: local_v6(),
                dst: peer,
                next: 0,
            })
            .with_payload(&f::extension(60))
            .with_payload(&f::extension(IPPROTO_TCP))
            .with_payload(&f::tcp(51000, 443)),
        );
        let flow = p
            .parse(LinkType::ETHERNET, &frame)
            .flow()
            .expect("a chained packet parses");
        assert_eq!(flow.local.port(), 51000);
        assert_eq!(flow.remote.port(), 443);
    }

    // ---- User Story 2: a frame fragcap cannot parse says why -------------

    #[test]
    fn every_rejection_cause_is_reachable_and_moves_exactly_its_own_counter() {
        // SC-002. One frame per cause. The corpus is the contract table in
        // contracts/parse-api.md, written out so a reviewer can count it.
        let peer: Ipv6Addr = "2001:db8::5".parse().expect("test address must parse");
        let v4 = |proto, transport: &[u8]| {
            eth_v4(
                f::V4 {
                    src: LOCAL_V4,
                    dst: PEER_V4,
                    proto,
                    ..f::V4::default()
                },
                transport,
            )
        };
        let v6 = |next, tail: &[u8]| {
            f::ethernet(
                ETHERTYPE_IPV6,
                &f::ipv6(f::V6 {
                    src: local_v6(),
                    dst: peer,
                    next,
                })
                .with_payload(tail),
            )
        };

        let mut chain = f::ipv6(f::V6 {
            src: local_v6(),
            dst: peer,
            next: 0,
        });
        for _ in 0..9 {
            chain = chain.with_payload(&f::extension(0));
        }
        let long_chain = f::ethernet(ETHERTYPE_IPV6, &chain.with_payload(&f::tcp(1, 2)));

        let mut bad_ihl = v4(IPPROTO_TCP, &f::tcp(1, 2));
        bad_ihl[14] = 0x44;

        let mut p = local_parser();
        let cases: Vec<(ParseReject, LinkType, Vec<u8>)> = vec![
            (
                ParseReject::UnsupportedLinkType,
                LinkType::from_code(108),
                v4(IPPROTO_TCP, &f::tcp(1, 2)),
            ),
            (
                ParseReject::UnsupportedEtherType,
                LinkType::ETHERNET,
                f::ethernet(0x8100, &[0; 40]),
            ),
            (
                ParseReject::UnsupportedAddressFamily,
                LinkType::NULL,
                f::loopback_raw(&99u32.to_le_bytes(), &[0x45; 40]),
            ),
            (
                ParseReject::UnsupportedIpVersion,
                LinkType::RAW,
                vec![0x50; 40],
            ),
            (
                ParseReject::MalformedNetworkHeader,
                LinkType::ETHERNET,
                bad_ihl,
            ),
            (
                ParseReject::ExtensionChainTooLong,
                LinkType::ETHERNET,
                long_chain,
            ),
            (ParseReject::NoNextHeader, LinkType::ETHERNET, v6(59, &[])),
            (
                ParseReject::UnsupportedTransport,
                LinkType::ETHERNET,
                v4(1, &[0; 20]),
            ),
            (
                ParseReject::MalformedTransportHeader,
                LinkType::ETHERNET,
                v4(IPPROTO_UDP, &f::udp(1, 2, 4)),
            ),
            (
                ParseReject::ShortHeader,
                LinkType::ETHERNET,
                v4(IPPROTO_TCP, &f::tcp(1, 2)[..2]),
            ),
            (
                ParseReject::UnmatchedFragment,
                LinkType::ETHERNET,
                eth_v4(
                    f::V4 {
                        src: LOCAL_V4,
                        dst: PEER_V4,
                        proto: IPPROTO_UDP,
                        ident: 77,
                        frag_offset: 185,
                        more_fragments: true,
                        ..f::V4::default()
                    },
                    &[0; 16],
                ),
            ),
            (
                ParseReject::NoLocalEndpoint,
                LinkType::ETHERNET,
                eth_v4(
                    f::V4 {
                        src: OTHER_V4,
                        dst: PEER_V4,
                        proto: IPPROTO_TCP,
                        ..f::V4::default()
                    },
                    &f::tcp(1, 2),
                ),
            ),
        ];

        assert_eq!(cases.len(), 12, "every enumerated cause needs a frame");
        for (want, link, frame) in cases {
            assert_rejected(&mut p, link, &frame, want);
        }
        assert_eq!(p.stats().rejected(), 12);
    }

    // FR-036. Nothing above was a drop.
    #[test]
    fn no_rejection_is_a_drop() {
        let mut p = local_parser();
        p.parse(LinkType::from_code(108), &[0; 64]);
        p.parse(LinkType::ETHERNET, &f::ethernet(0x8100, &[0; 40]));
        let mut stats = crate::stats::CaptureStats {
            parse: *p.stats(),
            ..crate::stats::CaptureStats::default()
        };
        stats.packets_captured = 2;
        assert_eq!(stats.parse.rejected(), 2);
        assert_eq!(stats.fragcap_dropped(), 0);
        assert!(!stats.lost_anything());
    }

    // The stage ordering in data-model.md: a frame wrong at more than one
    // layer is counted at the first.
    #[test]
    fn a_frame_wrong_at_two_layers_is_counted_at_the_first() {
        let mut p = local_parser();
        // Truncated to nothing after the Ethernet header, and carrying an
        // EtherType fragcap does not parse. The parser never gets far enough
        // to notice the truncation.
        let frame = f::ethernet(0x8100, &[]);
        assert_rejected(
            &mut p,
            LinkType::ETHERNET,
            &frame,
            ParseReject::UnsupportedEtherType,
        );
    }

    #[test]
    fn a_short_header_is_reported_at_each_boundary_in_turn() {
        let mut p = local_parser();
        let full = eth_v4(
            f::V4 {
                src: LOCAL_V4,
                dst: PEER_V4,
                proto: IPPROTO_TCP,
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        // Every truncation that lands inside the IPv4 or TCP headers is short,
        // never malformed and never a silently defaulted port.
        for len in 14..full.len().min(14 + 20 + 4) {
            let (outcome, _) = parse_counting(&mut p, LinkType::ETHERNET, &full[..len]);
            assert_eq!(
                outcome,
                ParseOutcome::Rejected(ParseReject::ShortHeader),
                "{len} bytes should be short"
            );
        }
        // And the first length that carries both ports resolves.
        assert!(p
            .parse(LinkType::ETHERNET, &full[..14 + 20 + 4])
            .flow()
            .is_some());
    }

    // ---- User Story 3: direction is honest about loopback ---------------

    // SC-004.
    #[test]
    fn ambiguous_and_absent_locality_are_different_outcomes() {
        let loop_v4 = Ipv4Addr::LOCALHOST;
        let mut p = parser_with(&[IpAddr::V4(loop_v4)]);
        let ambiguous = p.parse(
            LinkType::ETHERNET,
            &eth_v4(
                f::V4 {
                    src: loop_v4,
                    dst: loop_v4,
                    proto: IPPROTO_TCP,
                    ..f::V4::default()
                },
                &f::tcp(1, 2),
            ),
        );
        let absent = p.parse(
            LinkType::ETHERNET,
            &eth_v4(
                f::V4 {
                    src: OTHER_V4,
                    dst: PEER_V4,
                    proto: IPPROTO_TCP,
                    ..f::V4::default()
                },
                &f::tcp(1, 2),
            ),
        );
        assert!(ambiguous.flow().is_some(), "loopback keeps its key");
        assert_eq!(ambiguous.direction(), None);
        assert_eq!(absent.flow(), None, "no local endpoint, no key");
        assert_eq!(absent.direction(), None);
        assert_eq!(p.stats().direction_ambiguous, 1);
        assert_eq!(p.stats().no_local_endpoint, 1);
    }

    // FR-032.
    #[test]
    fn replacing_the_address_set_changes_the_direction_of_an_identical_frame() {
        let frame = eth_v4(
            f::V4 {
                src: LOCAL_V4,
                dst: PEER_V4,
                proto: IPPROTO_TCP,
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        let mut p = parser_with(&[IpAddr::V4(LOCAL_V4)]);
        assert_eq!(
            p.parse(LinkType::ETHERNET, &frame).direction(),
            Some(Direction::Outbound)
        );
        p.set_interface_addrs(InterfaceAddrs::new([IpAddr::V4(PEER_V4)]));
        assert_eq!(
            p.parse(LinkType::ETHERNET, &frame).direction(),
            Some(Direction::Inbound),
            "no derivation of the previous set may survive"
        );
        p.set_interface_addrs(InterfaceAddrs::default());
        assert_eq!(
            p.parse(LinkType::ETHERNET, &frame),
            ParseOutcome::Rejected(ParseReject::NoLocalEndpoint)
        );
    }

    #[test]
    fn an_empty_address_set_rejects_every_packet_loudly_and_drops_none() {
        let mut p = HeaderParser::new(InterfaceAddrs::default());
        for port in 0..5u16 {
            let frame = eth_v4(
                f::V4 {
                    src: LOCAL_V4,
                    dst: PEER_V4,
                    proto: IPPROTO_TCP,
                    ..f::V4::default()
                },
                &f::tcp(port, 443),
            );
            assert_eq!(
                p.parse(LinkType::ETHERNET, &frame),
                ParseOutcome::Rejected(ParseReject::NoLocalEndpoint)
            );
        }
        assert_eq!(p.stats().no_local_endpoint, 5, "once per packet, loudly");
        assert_eq!(p.stats().rejected(), 5);
    }

    // ---- User Story 4: fragments without reassembly ---------------------

    fn fragment(offset: u16, more: bool, transport: &[u8]) -> Vec<u8> {
        eth_v4(
            f::V4 {
                src: LOCAL_V4,
                dst: PEER_V4,
                proto: IPPROTO_UDP,
                ident: 4242,
                frag_offset: offset,
                more_fragments: more,
                ..f::V4::default()
            },
            transport,
        )
    }

    // SC-006.
    #[test]
    fn a_subsequent_fragment_resolves_to_its_first_fragments_key() {
        let mut p = local_parser();
        let first = p
            .parse(
                LinkType::ETHERNET,
                &fragment(0, true, &f::udp(30000, 5055, 800)),
            )
            .flow()
            .expect("the first fragment carries its transport header");
        let second = p
            .parse(LinkType::ETHERNET, &fragment(185, true, &[0xab; 16]))
            .flow()
            .expect("the second resolves from the remembered identity");
        assert_eq!(first, second);
        assert_eq!(p.stats().rejected(), 0);
    }

    #[test]
    fn the_last_fragment_resolves_and_then_forgets_the_datagram() {
        let mut p = local_parser();
        p.parse(
            LinkType::ETHERNET,
            &fragment(0, true, &f::udp(30000, 5055, 800)),
        );
        assert!(p
            .parse(LinkType::ETHERNET, &fragment(370, false, &[0; 16]))
            .flow()
            .is_some());
        // The entry is gone, so a repeat of the same last fragment is honestly
        // unmatched rather than matched against a stale memory.
        assert_eq!(
            p.parse(LinkType::ETHERNET, &fragment(370, false, &[0; 16])),
            ParseOutcome::Rejected(ParseReject::UnmatchedFragment)
        );
    }

    #[test]
    fn an_orphaned_subsequent_fragment_gets_no_key() {
        let mut p = local_parser();
        assert_eq!(
            p.parse(LinkType::ETHERNET, &fragment(185, true, &[0; 16])),
            ParseOutcome::Rejected(ParseReject::UnmatchedFragment),
            "out of order arrival is not licence to invent a key"
        );
    }

    // FR-021a.
    #[test]
    fn a_first_fragment_that_could_not_be_parsed_records_nothing() {
        let mut p = local_parser();
        // Its own UDP header contradicts itself, so no ports were observed.
        assert_eq!(
            p.parse(LinkType::ETHERNET, &fragment(0, true, &f::udp(1, 2, 4))),
            ParseOutcome::Rejected(ParseReject::MalformedTransportHeader)
        );
        assert_eq!(
            p.parse(LinkType::ETHERNET, &fragment(185, true, &[0; 16])),
            ParseOutcome::Rejected(ParseReject::UnmatchedFragment),
            "nothing was recorded, so nothing may be matched"
        );
    }

    // FR-021 as corrected at the analyze gate.
    #[test]
    fn an_unfragmented_packet_leaves_the_table_empty() {
        let mut p = local_parser();
        for port in 0..300u16 {
            p.parse(
                LinkType::ETHERNET,
                &eth_v4(
                    f::V4 {
                        src: LOCAL_V4,
                        dst: PEER_V4,
                        proto: IPPROTO_UDP,
                        ..f::V4::default()
                    },
                    &f::udp(port, 5055, 8),
                ),
            );
        }
        assert_eq!(
            p.stats().fragment_evicted,
            0,
            "ordinary traffic must not churn the fragment table"
        );
        // And a real fragment still resolves, which it could not if 300
        // non-fragments had filled a 256 entry table.
        p.parse(
            LinkType::ETHERNET,
            &fragment(0, true, &f::udp(30000, 5055, 800)),
        );
        assert!(p
            .parse(LinkType::ETHERNET, &fragment(185, true, &[0; 16]))
            .flow()
            .is_some());
    }

    // FR-022a.
    #[test]
    fn a_fragments_direction_is_recomputed_rather_than_inherited() {
        let mut p = parser_with(&[IpAddr::V4(LOCAL_V4)]);
        assert_eq!(
            p.parse(
                LinkType::ETHERNET,
                &fragment(0, true, &f::udp(30000, 5055, 800))
            )
            .direction(),
            Some(Direction::Outbound)
        );
        p.set_interface_addrs(InterfaceAddrs::new([IpAddr::V4(PEER_V4)]));
        assert_eq!(
            p.parse(LinkType::ETHERNET, &fragment(185, true, &[0; 16]))
                .direction(),
            Some(Direction::Inbound),
            "the address set moved, so the direction must move with it"
        );
    }

    // FR-025. Structural, since the parser takes a shared slice and returns a
    // Copy outcome, but this is the requirement's only assertion.
    #[test]
    fn parsing_a_fragment_neither_alters_nor_joins_anything() {
        let mut p = local_parser();
        let first = fragment(0, true, &f::udp(30000, 5055, 800));
        let second = fragment(185, true, &[0xab; 16]);
        let first_before = first.clone();
        let second_before = second.clone();
        p.parse(LinkType::ETHERNET, &first);
        p.parse(LinkType::ETHERNET, &second);
        assert_eq!(first, first_before, "the frame was modified");
        assert_eq!(second, second_before, "the frame was modified");
        assert_eq!(
            second.len(),
            second_before.len(),
            "nothing was joined onto it"
        );
    }

    #[test]
    fn overflowing_the_fragment_table_evicts_and_drops_nothing() {
        let mut p = local_parser();
        for ident in 0..=fragment::FRAGMENT_TABLE_CAPACITY as u16 {
            let frame = eth_v4(
                f::V4 {
                    src: LOCAL_V4,
                    dst: PEER_V4,
                    proto: IPPROTO_UDP,
                    ident,
                    more_fragments: true,
                    ..f::V4::default()
                },
                &f::udp(30000, 5055, 800),
            );
            assert!(p.parse(LinkType::ETHERNET, &frame).flow().is_some());
        }
        assert_eq!(p.stats().fragment_evicted, 1);
        assert_eq!(p.stats().rejected(), 0, "an eviction is not a rejection");
    }

    // ---- apply, the call site S04 and S08 share -------------------------

    #[test]
    fn apply_writes_both_fields_onto_the_packet() {
        let mut p = local_parser();
        let frame = eth_v4(
            f::V4 {
                src: LOCAL_V4,
                dst: PEER_V4,
                proto: IPPROTO_TCP,
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        let len = frame.len() as u32;
        let mut packet = CapturedPacket::from_raw(RawPacket::new(
            Timestamp::from_nanos(1),
            Payload::from(frame),
            len,
        ));
        let outcome = p.apply(LinkType::ETHERNET, &mut packet);
        assert_eq!(packet.flow, outcome.flow());
        assert_eq!(packet.direction, Some(Direction::Outbound));
        assert_eq!(
            packet.attribution_state(),
            crate::packet::AttributionState::Unresolved,
            "a key with no attribution is retained and marked, per P-4"
        );
    }

    #[test]
    fn apply_leaves_both_fields_absent_on_a_rejection() {
        let mut p = local_parser();
        let frame = f::ethernet(0x8100, &[0; 40]);
        let len = frame.len() as u32;
        let mut packet = CapturedPacket::from_raw(RawPacket::new(
            Timestamp::from_nanos(1),
            Payload::from(frame),
            len,
        ));
        let outcome = p.apply(LinkType::ETHERNET, &mut packet);
        assert_eq!(outcome.reject(), Some(ParseReject::UnsupportedEtherType));
        assert_eq!(packet.flow, None);
        assert_eq!(packet.direction, None);
        assert_eq!(
            packet.attribution_state(),
            crate::packet::AttributionState::NotAttempted,
            "nothing to attempt attribution with, which is not the same as failing"
        );
    }

    #[test]
    fn a_reject_names_its_counter() {
        assert_eq!(ParseReject::ShortHeader.as_str(), "short_header");
        assert_eq!(ParseReject::NoLocalEndpoint.as_str(), "no_local_endpoint");
    }
}
