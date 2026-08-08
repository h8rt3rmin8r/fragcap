// SPDX-License-Identifier: Apache-2.0

//! Network headers: IPv4, IPv6, and the IPv6 extension header chain.
//!
//! Two distinctions carry most of the weight here.
//!
//! **Malformed is not short.** A header whose own fields contradict each other
//! indicates a broken sender or a defect in this parser. A header that is
//! internally legal but extends past the captured bytes indicates a snapshot
//! length, which is the operator's own choice. The remedies are opposite, so
//! the counters are separate. See research R-3.
//!
//! **The declared length bounds reads in one direction only.** When a declared
//! length exceeds the captured bytes the capture was truncated, and reads are
//! bounded by what was captured. When it is *shorter* than the captured bytes
//! the excess is not part of the datagram at all: it is Ethernet padding on a
//! frame below the sixty byte minimum, or trailing data. Reading it would let a
//! packet carrying no transport header produce a flow key out of padding, which
//! is exactly the fabrication constitution P-9 forbids. The datagram extent is
//! therefore the smaller of the two, and reads are bounded by it.
//!
//! A declared length of zero is neither, and is not an error. Large send
//! offload leaves the field for the adapter to fill in after the capture point,
//! so outbound traffic captured on the sending host routinely reads zero. That
//! is common on the focal platform, and treating it as malformed would reject
//! real game traffic.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::fragment::FragmentKey;
use super::link::NetProto;
use super::ParseReject;

/// The most extension headers the walk will follow.
///
/// The IPv6 standard places no limit on chain length, so a crafted packet can
/// carry an arbitrarily long one, and an unbounded walk over attacker
/// controlled bytes on the capture thread is a denial of service against the
/// capture rather than merely a parse bug. Real traffic uses zero to two.
const MAX_EXT_HEADERS: usize = 8;

const IPV4_MIN_HEADER_LEN: usize = 20;
const IPV6_FIXED_HEADER_LEN: usize = 40;

const EXT_HOP_BY_HOP: u8 = 0;
const EXT_ROUTING: u8 = 43;
const EXT_FRAGMENT: u8 = 44;
const EXT_AUTHENTICATION: u8 = 51;
const EXT_DESTINATION_OPTIONS: u8 = 60;
/// No next header. A well-formed packet that legitimately carries no
/// transport, which is not the same as one whose transport fragcap declined to
/// parse.
const NO_NEXT_HEADER: u8 = 59;

/// Where a packet sits in its datagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FragmentRole {
    /// Offset zero with more fragments following. Carries the transport
    /// header, so it is parsed normally and its identity recorded.
    Initial,
    /// A non-zero offset with more still to come. No transport header.
    Subsequent,
    /// A non-zero offset and nothing following. No transport header, and
    /// observing it is what forgets the datagram.
    Last,
}

/// What the network layer resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NetInfo {
    pub src: IpAddr,
    pub dst: IpAddr,
    /// The transport protocol number, meaningful only when this is not a
    /// non-initial fragment.
    pub proto: u8,
    /// Where the transport header begins, relative to the start of the network
    /// header.
    pub transport_offset: usize,
    /// How many of the captured bytes belong to this datagram, relative to the
    /// start of the network header.
    ///
    /// Not the same as the captured length. A frame below Ethernet's sixty
    /// byte minimum is padded, and the padding is not the datagram. Reading
    /// past this boundary would attribute bytes the sender never put there.
    /// Always at least `transport_offset`.
    pub extent: usize,
    /// Present only when the packet is actually fragmented. A packet with
    /// offset zero and nothing following is not a fragment, whatever headers
    /// it carries, and recording it would fill the table with entries no
    /// second fragment will ever match.
    pub fragment: Option<(FragmentKey, FragmentRole)>,
}

/// Parse the network header. `bytes` begins at it.
pub(crate) fn parse(proto: NetProto, bytes: &[u8]) -> Result<NetInfo, ParseReject> {
    match proto {
        NetProto::V4 => parse_v4(bytes),
        NetProto::V6 => parse_v6(bytes),
    }
}

fn parse_v4(bytes: &[u8]) -> Result<NetInfo, ParseReject> {
    if bytes.len() < IPV4_MIN_HEADER_LEN {
        return Err(ParseReject::ShortHeader);
    }
    if bytes[0] >> 4 != 4 {
        // The link layer said IPv4 and the header says otherwise. That is a
        // contradiction rather than a truncation.
        return Err(ParseReject::MalformedNetworkHeader);
    }
    let header_len = (bytes[0] & 0x0f) as usize * 4;
    if header_len < IPV4_MIN_HEADER_LEN {
        // Below the fixed header's own size. Self-contradictory.
        return Err(ParseReject::MalformedNetworkHeader);
    }
    if header_len > bytes.len() {
        // Legal, but the options it declares were not captured.
        return Err(ParseReject::ShortHeader);
    }

    let declared = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
    let extent = extent(declared, header_len, bytes.len())?;

    let proto = bytes[9];
    let src = Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]);
    let dst = Ipv4Addr::new(bytes[16], bytes[17], bytes[18], bytes[19]);

    let ident = u16::from_be_bytes([bytes[4], bytes[5]]);
    let flags_and_offset = u16::from_be_bytes([bytes[6], bytes[7]]);
    let more_fragments = flags_and_offset & 0x2000 != 0;
    let offset = flags_and_offset & 0x1fff;

    let fragment = role(offset, more_fragments).map(|r| {
        (
            FragmentKey::V4 {
                src,
                dst,
                proto,
                ident,
            },
            r,
        )
    });

    Ok(NetInfo {
        src: IpAddr::V4(src),
        dst: IpAddr::V4(dst),
        proto,
        transport_offset: header_len,
        extent,
        fragment,
    })
}

/// How many of the captured bytes belong to the datagram.
///
/// Three cases, and conflating any two of them produces a different defect.
///
/// A declared length longer than the capture means the capture was truncated,
/// usually by a snapshot length the operator chose. The captured length wins.
///
/// A declared length shorter than the capture means the frame carries bytes
/// that are not the datagram: Ethernet pads anything below sixty bytes, and
/// some senders append trailing data. The declared length wins, because
/// reading the excess would let a datagram with no transport header yield
/// ports out of padding.
///
/// A declared length of zero means the field was never filled in. Large send
/// offload defers it to the adapter, past the point the capture is taken, so
/// this is ordinary on outbound traffic rather than an error. The captured
/// length is the only information available, so it wins.
fn extent(declared: usize, header_len: usize, captured: usize) -> Result<usize, ParseReject> {
    if declared == 0 {
        return Ok(captured);
    }
    if declared < header_len {
        // A datagram shorter than its own header. Self-contradictory, and
        // distinct from a truncated capture, which leaves the field intact.
        return Err(ParseReject::MalformedNetworkHeader);
    }
    Ok(declared.min(captured))
}

fn parse_v6(bytes: &[u8]) -> Result<NetInfo, ParseReject> {
    if bytes.len() < IPV6_FIXED_HEADER_LEN {
        return Err(ParseReject::ShortHeader);
    }
    if bytes[0] >> 4 != 6 {
        return Err(ParseReject::MalformedNetworkHeader);
    }

    let src = ipv6_at(bytes, 8);
    let dst = ipv6_at(bytes, 24);

    // The declared payload excludes the fixed header. Zero means unset, which
    // is either large send offload or a jumbogram, so the capture is all there
    // is to go on. The malformed case cannot arise here: the extent is never
    // below the fixed header.
    let declared = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    let extent = extent(
        if declared == 0 {
            0
        } else {
            IPV6_FIXED_HEADER_LEN + declared
        },
        IPV6_FIXED_HEADER_LEN,
        bytes.len(),
    )?;
    // Everything below reads through this slice, so the padding boundary is
    // enforced once rather than at each of the walk's four length checks.
    let bytes = &bytes[..extent];

    let mut next = bytes[6];
    let mut cursor = IPV6_FIXED_HEADER_LEN;
    let mut seen = 0usize;
    let mut fragment_header: Option<(u32, u16, bool)> = None;

    while is_extension(next) {
        if seen >= MAX_EXT_HEADERS {
            return Err(ParseReject::ExtensionChainTooLong);
        }
        seen += 1;

        let advance = match next {
            EXT_FRAGMENT => {
                if cursor + 8 > bytes.len() {
                    return Err(ParseReject::ShortHeader);
                }
                let flags_and_offset = u16::from_be_bytes([bytes[cursor + 2], bytes[cursor + 3]]);
                let ident = u32::from_be_bytes([
                    bytes[cursor + 4],
                    bytes[cursor + 5],
                    bytes[cursor + 6],
                    bytes[cursor + 7],
                ]);
                let offset = flags_and_offset >> 3;
                let more = flags_and_offset & 1 != 0;
                let inner_next = bytes[cursor];

                if offset != 0 {
                    // A non-initial fragment's data is a chunk from the middle
                    // of the original packet's fragmentable part. The fragment
                    // header's next header field names the first header of
                    // that part in the *original* packet, which this fragment
                    // does not begin with and may not contain at all.
                    //
                    // Walking on would parse payload bytes as whatever that
                    // field names, and a Destination Options or authentication
                    // header there is both legal and common. The walk would
                    // then advance by a length read out of payload and reject
                    // a valid fragment before its recorded identity was ever
                    // consulted. Stop here: the transport came from the first
                    // fragment, and the table holds it.
                    return Ok(NetInfo {
                        src: IpAddr::V6(src),
                        dst: IpAddr::V6(dst),
                        // Not meaningful for a non-initial fragment, and not
                        // read for one. The ports come from the table.
                        proto: inner_next,
                        transport_offset: cursor + 8,
                        extent,
                        fragment: role(offset, more)
                            .map(|r| (FragmentKey::V6 { src, dst, ident }, r)),
                    });
                }

                fragment_header = Some((ident, offset, more));
                next = inner_next;
                8
            }
            EXT_AUTHENTICATION => {
                if cursor + 2 > bytes.len() {
                    return Err(ParseReject::ShortHeader);
                }
                // Four-octet units, excluding two of them. The odd formula out,
                // and the likeliest place for an off-by-one.
                let advance = (bytes[cursor + 1] as usize + 2) * 4;
                next = bytes[cursor];
                advance
            }
            _ => {
                if cursor + 2 > bytes.len() {
                    return Err(ParseReject::ShortHeader);
                }
                // Eight-octet units, excluding the first eight.
                let advance = (bytes[cursor + 1] as usize + 1) * 8;
                next = bytes[cursor];
                advance
            }
        };

        if advance == 0 {
            // Unreachable given the encodings above, all of which yield at
            // least eight. Kept so that termination is a property of the walk
            // rather than of the arithmetic, and sharing the chain counter so
            // that no counter exists which no frame can advance. See plan
            // decision D-8.
            return Err(ParseReject::ExtensionChainTooLong);
        }
        cursor += advance;
        if cursor > bytes.len() {
            return Err(ParseReject::ShortHeader);
        }
    }

    if next == NO_NEXT_HEADER {
        return Err(ParseReject::NoNextHeader);
    }

    let fragment = fragment_header.and_then(|(ident, offset, more)| {
        role(offset, more).map(|r| (FragmentKey::V6 { src, dst, ident }, r))
    });

    Ok(NetInfo {
        src: IpAddr::V6(src),
        dst: IpAddr::V6(dst),
        proto: next,
        transport_offset: cursor,
        extent,
        fragment,
    })
}

/// Which fragment role an offset and flag pair describes, or `None` when the
/// packet is not fragmented at all.
///
/// Offset zero with no more fragments following is not a fragment. For IPv4
/// that is ordinary traffic; for IPv6 it is an atomic fragment, a fragment
/// header on a whole datagram. Both must be parsed normally rather than
/// recorded.
fn role(offset: u16, more_fragments: bool) -> Option<FragmentRole> {
    match (offset, more_fragments) {
        (0, false) => None,
        (0, true) => Some(FragmentRole::Initial),
        (_, true) => Some(FragmentRole::Subsequent),
        (_, false) => Some(FragmentRole::Last),
    }
}

fn is_extension(next: u8) -> bool {
    matches!(
        next,
        EXT_HOP_BY_HOP | EXT_ROUTING | EXT_FRAGMENT | EXT_AUTHENTICATION | EXT_DESTINATION_OPTIONS
    )
}

fn ipv6_at(bytes: &[u8], at: usize) -> Ipv6Addr {
    let mut octets = [0u8; 16];
    octets.copy_from_slice(&bytes[at..at + 16]);
    Ipv6Addr::from(octets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::testframe as f;
    use crate::parse::transport::{IPPROTO_TCP, IPPROTO_UDP};

    fn v4(src: &str, dst: &str) -> (Ipv4Addr, Ipv4Addr) {
        (
            src.parse().expect("test address must parse"),
            dst.parse().expect("test address must parse"),
        )
    }

    fn tcp_v4(h: f::V4) -> Vec<u8> {
        f::ipv4(h, &f::tcp(51000, 443))
    }

    // FR-012.
    #[test]
    fn a_minimum_length_ipv4_header_parses() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = tcp_v4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        });
        let info = parse(NetProto::V4, &packet).expect("a legal header parses");
        assert_eq!(info.src, IpAddr::V4(s));
        assert_eq!(info.dst, IpAddr::V4(d));
        assert_eq!(info.proto, IPPROTO_TCP);
        assert_eq!(info.transport_offset, IPV4_MIN_HEADER_LEN);
        assert_eq!(info.extent, packet.len());
        assert_eq!(info.fragment, None);
    }

    #[test]
    fn ipv4_options_are_skipped_by_the_declared_header_length() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_UDP,
                option_words: 3,
                ..f::V4::default()
            },
            &f::udp(30000, 5055, 8),
        );
        let info = parse(NetProto::V4, &packet).expect("options are legal");
        assert_eq!(
            info.transport_offset,
            IPV4_MIN_HEADER_LEN + 12,
            "the transport starts after the options, not after twenty bytes"
        );
    }

    // The datagram extent. Three cases, and conflating any two is a different
    // defect. Raised in review of pull request 6.
    #[test]
    fn a_total_length_shorter_than_the_capture_bounds_the_datagram() {
        // Ethernet pads anything below sixty bytes. The padding is not the
        // datagram, and reading it would produce ports nobody sent.
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let mut packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_TCP,
                ..f::V4::default()
            },
            &[],
        );
        packet.extend_from_slice(&[0u8; 26]);
        let info = parse(NetProto::V4, &packet).expect("the header itself is legal");
        assert_eq!(
            info.extent, IPV4_MIN_HEADER_LEN,
            "the padding is not part of the datagram"
        );
        assert_eq!(
            info.extent, info.transport_offset,
            "so there are no transport bytes at all"
        );
    }

    #[test]
    fn a_total_length_longer_than_the_capture_is_bounded_by_the_capture() {
        // The snapshot length case, which is truncation rather than padding.
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_TCP,
                total_len: Some(1500),
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        let info = parse(NetProto::V4, &packet).expect("a truncated capture still parses");
        assert_eq!(info.extent, packet.len());
    }

    #[test]
    fn a_zero_total_length_falls_back_to_the_capture() {
        // Large send offload leaves the field for the adapter to fill in after
        // the capture point. Common on outbound traffic on the focal platform,
        // and rejecting it would lose real game traffic.
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_TCP,
                total_len: Some(0),
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        let info = parse(NetProto::V4, &packet).expect("an unset length is not an error");
        assert_eq!(info.extent, packet.len());
    }

    #[test]
    fn a_total_length_below_the_header_length_is_malformed() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_TCP,
                total_len: Some(12),
                ..f::V4::default()
            },
            &f::tcp(51000, 443),
        );
        assert_eq!(
            parse(NetProto::V4, &packet),
            Err(ParseReject::MalformedNetworkHeader),
            "a datagram shorter than its own header contradicts itself"
        );
    }

    // FR-013.
    #[test]
    fn a_bare_ipv6_header_parses() {
        let packet = f::ipv6(
            f::V6 {
                next: IPPROTO_TCP,
                ..f::V6::default()
            },
            &f::tcp(51000, 443),
        );
        let info = parse(NetProto::V6, &packet).expect("a legal header parses");
        assert_eq!(info.proto, IPPROTO_TCP);
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN);
        assert_eq!(info.extent, packet.len());
    }

    #[test]
    fn an_ipv6_payload_length_shorter_than_the_capture_bounds_the_datagram() {
        // The same padding case as IPv4. Raised in review of pull request 6.
        let mut packet = f::ipv6(
            f::V6 {
                next: IPPROTO_TCP,
                payload_len: Some(2),
                ..f::V6::default()
            },
            &f::tcp(51000, 443),
        );
        packet.extend_from_slice(&[0u8; 8]);
        let info = parse(NetProto::V6, &packet).expect("the fixed header is legal");
        assert_eq!(
            info.extent,
            IPV6_FIXED_HEADER_LEN + 2,
            "two declared bytes are two bytes, whatever else was captured"
        );
    }

    #[test]
    fn a_zero_ipv6_payload_length_falls_back_to_the_capture() {
        // Offload again, or a jumbogram. Either way the capture is all there
        // is to go on.
        let packet = f::ipv6(
            f::V6 {
                next: IPPROTO_TCP,
                payload_len: Some(0),
                ..f::V6::default()
            },
            &f::tcp(51000, 443),
        );
        let info = parse(NetProto::V6, &packet).expect("an unset length is not an error");
        assert_eq!(info.extent, packet.len());
    }

    // FR-014 and research R-4. One test per handled extension header, because
    // a single test over all five would pass with one advance formula wrong.
    #[test]
    fn each_extension_header_advances_by_its_own_encoding() {
        for (name, ext, len) in [
            ("hop-by-hop", EXT_HOP_BY_HOP, 8usize),
            ("routing", EXT_ROUTING, 8),
            ("destination options", EXT_DESTINATION_OPTIONS, 8),
            ("fragment", EXT_FRAGMENT, 8),
            ("authentication", EXT_AUTHENTICATION, 8),
        ] {
            let packet = f::ipv6(
                f::V6 {
                    next: ext,
                    ..f::V6::default()
                },
                &f::cat(&[&f::extension(IPPROTO_UDP), &f::udp(30000, 5055, 8)]),
            );
            let info = parse(NetProto::V6, &packet)
                .unwrap_or_else(|e| panic!("{name} chain must parse, got {e:?}"));
            assert_eq!(info.proto, IPPROTO_UDP, "{name} must reach the transport");
            assert_eq!(
                info.transport_offset,
                IPV6_FIXED_HEADER_LEN + len,
                "{name} advanced by the wrong amount"
            );
        }
    }

    #[test]
    fn a_longer_option_header_advances_further() {
        // Two eight-octet units: the length byte counts units after the first.
        let mut ext = f::extension(IPPROTO_UDP);
        ext[1] = 1;
        ext.extend_from_slice(&[0; 8]);
        let packet = f::ipv6(
            f::V6 {
                next: EXT_HOP_BY_HOP,
                ..f::V6::default()
            },
            &f::cat(&[&ext, &f::udp(1, 2, 8)]),
        );
        let info = parse(NetProto::V6, &packet).expect("a two unit header is legal");
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN + 16);
    }

    #[test]
    fn a_chain_of_several_headers_is_walked_to_the_end() {
        let packet = f::ipv6(
            f::V6 {
                next: EXT_HOP_BY_HOP,
                ..f::V6::default()
            },
            &f::cat(&[
                &f::extension(EXT_ROUTING),
                &f::extension(EXT_DESTINATION_OPTIONS),
                &f::extension(IPPROTO_TCP),
                &f::tcp(1, 2),
            ]),
        );
        let info = parse(NetProto::V6, &packet).expect("three headers is legal");
        assert_eq!(info.proto, IPPROTO_TCP);
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN + 24);
    }

    // FR-015 and SC-007. Without the bound this test hangs rather than fails.
    #[test]
    fn a_chain_past_the_bound_terminates_the_walk() {
        let mut tail = Vec::new();
        for _ in 0..MAX_EXT_HEADERS + 1 {
            tail.extend_from_slice(&f::extension(EXT_HOP_BY_HOP));
        }
        tail.extend_from_slice(&f::tcp(1, 2));
        let packet = f::ipv6(
            f::V6 {
                next: EXT_HOP_BY_HOP,
                ..f::V6::default()
            },
            &tail,
        );
        assert_eq!(
            parse(NetProto::V6, &packet),
            Err(ParseReject::ExtensionChainTooLong)
        );
    }

    #[test]
    fn a_chain_exactly_at_the_bound_still_parses() {
        let mut tail = Vec::new();
        for i in 0..MAX_EXT_HEADERS {
            let next = if i == MAX_EXT_HEADERS - 1 {
                IPPROTO_TCP
            } else {
                EXT_HOP_BY_HOP
            };
            tail.extend_from_slice(&f::extension(next));
        }
        tail.extend_from_slice(&f::tcp(1, 2));
        let packet = f::ipv6(
            f::V6 {
                next: EXT_HOP_BY_HOP,
                ..f::V6::default()
            },
            &tail,
        );
        let info = parse(NetProto::V6, &packet).expect("eight headers is the bound, not past it");
        assert_eq!(info.proto, IPPROTO_TCP);
    }

    // FR-016.
    #[test]
    fn a_chain_ending_in_no_next_header_is_its_own_cause() {
        let packet = f::ipv6(
            f::V6 {
                next: NO_NEXT_HEADER,
                ..f::V6::default()
            },
            &[],
        );
        assert_eq!(
            parse(NetProto::V6, &packet),
            Err(ParseReject::NoNextHeader),
            "a legitimate absence of transport is not an unsupported one"
        );
    }

    // FR-017. The distinction the analyze gate corrected.
    #[test]
    fn an_illegal_header_length_is_malformed() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let mut packet = tcp_v4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        });
        packet[0] = 0x44; // four words, below the fixed header's five
        assert_eq!(
            parse(NetProto::V4, &packet),
            Err(ParseReject::MalformedNetworkHeader)
        );
    }

    #[test]
    fn a_legal_header_length_past_the_captured_bytes_is_short() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_TCP,
                option_words: 5,
                ..f::V4::default()
            },
            &[],
        );
        let truncated = &packet[..IPV4_MIN_HEADER_LEN + 4];
        assert_eq!(
            parse(NetProto::V4, truncated),
            Err(ParseReject::ShortHeader),
            "a legal length on a snapshotted frame is truncation, not malformation"
        );
    }

    #[test]
    fn a_version_disagreeing_with_the_link_layer_is_malformed() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let mut packet = tcp_v4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        });
        packet[0] = 0x65;
        assert_eq!(
            parse(NetProto::V4, &packet),
            Err(ParseReject::MalformedNetworkHeader)
        );
    }

    #[test]
    fn a_network_header_shorter_than_its_fixed_part_is_short() {
        assert_eq!(
            parse(NetProto::V4, &[0x45; 19]),
            Err(ParseReject::ShortHeader)
        );
        assert_eq!(
            parse(NetProto::V6, &[0x60; 39]),
            Err(ParseReject::ShortHeader)
        );
    }

    // FR-026 and research R-7. The classification, including the case the
    // analyze gate corrected.
    #[test]
    fn an_unfragmented_packet_has_no_fragment_role() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(
            f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_UDP,
                ..f::V4::default()
            },
            &f::udp(1, 2, 8),
        );
        assert_eq!(
            parse(NetProto::V4, &packet).expect("legal").fragment,
            None,
            "offset zero with nothing following is ordinary traffic"
        );
    }

    #[test]
    fn the_three_fragment_roles_are_distinguished() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        for (offset, more, expected) in [
            (0u16, true, FragmentRole::Initial),
            (185, true, FragmentRole::Subsequent),
            (370, false, FragmentRole::Last),
        ] {
            let packet = f::ipv4(
                f::V4 {
                    src: s,
                    dst: d,
                    proto: IPPROTO_UDP,
                    ident: 4242,
                    frag_offset: offset,
                    more_fragments: more,
                    ..f::V4::default()
                },
                &f::udp(1, 2, 8),
            );
            let info = parse(NetProto::V4, &packet).expect("legal");
            let (key, role) = info.fragment.expect("this packet is a fragment");
            assert_eq!(role, expected);
            assert_eq!(
                key,
                FragmentKey::V4 {
                    src: s,
                    dst: d,
                    proto: IPPROTO_UDP,
                    ident: 4242
                }
            );
        }
    }

    #[test]
    fn an_ipv6_atomic_fragment_is_not_treated_as_a_fragment() {
        // A fragment header with offset zero and nothing following describes a
        // whole datagram. Recording it would leave an entry nothing matches.
        let packet = f::ipv6(
            f::V6 {
                next: EXT_FRAGMENT,
                ..f::V6::default()
            },
            &f::cat(&[
                &f::fragment_ext(IPPROTO_UDP, 0, false, 99),
                &f::udp(1, 2, 8),
            ]),
        );
        assert_eq!(parse(NetProto::V6, &packet).expect("legal").fragment, None);
    }

    #[test]
    fn an_ipv6_fragment_key_carries_the_thirty_two_bit_identification() {
        let packet = f::ipv6(
            f::V6 {
                next: EXT_FRAGMENT,
                ..f::V6::default()
            },
            &f::cat(&[
                &f::fragment_ext(IPPROTO_UDP, 185, true, 0xdeadbeef),
                &f::udp(1, 2, 8),
            ]),
        );
        let info = parse(NetProto::V6, &packet).expect("legal");
        let (key, role) = info.fragment.expect("this packet is a fragment");
        assert_eq!(role, FragmentRole::Subsequent);
        match key {
            FragmentKey::V6 { ident, .. } => assert_eq!(ident, 0xdeadbeef),
            FragmentKey::V4 { .. } => panic!("an IPv6 packet produced an IPv4 key"),
        }
    }

    // Raised in review of pull request 6. A non-initial fragment's data is a
    // chunk from the middle of the original fragmentable part, so the fragment
    // header's next header field names something this fragment does not begin
    // with and may not contain at all.
    #[test]
    fn a_non_initial_fragment_stops_the_walk_at_its_fragment_header() {
        for named in [EXT_DESTINATION_OPTIONS, EXT_AUTHENTICATION, EXT_ROUTING] {
            let packet = f::ipv6(
                f::V6 {
                    next: EXT_FRAGMENT,
                    ..f::V6::default()
                },
                // The fragment header names an extension header, and what
                // follows is payload rather than that header.
                &f::cat(&[&f::fragment_ext(named, 185, true, 7), &[0xff; 24]]),
            );
            let info = parse(NetProto::V6, &packet)
                .unwrap_or_else(|e| panic!("next header {named} must not be walked, got {e:?}"));
            let (_, role) = info.fragment.expect("this packet is a fragment");
            assert_eq!(role, FragmentRole::Subsequent);
        }
    }

    #[test]
    fn an_initial_fragment_still_walks_on_past_its_fragment_header() {
        // The first fragment does begin the fragmentable part, so the field
        // does name what follows and the walk must continue.
        let packet = f::ipv6(
            f::V6 {
                next: EXT_FRAGMENT,
                ..f::V6::default()
            },
            &f::cat(&[
                &f::fragment_ext(EXT_DESTINATION_OPTIONS, 0, true, 7),
                &f::extension(IPPROTO_UDP),
                &f::udp(30000, 5055, 8),
            ]),
        );
        let info = parse(NetProto::V6, &packet).expect("an initial fragment walks on");
        assert_eq!(info.proto, IPPROTO_UDP);
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN + 16);
        let (_, role) = info.fragment.expect("this packet is a fragment");
        assert_eq!(role, FragmentRole::Initial);
    }
}
