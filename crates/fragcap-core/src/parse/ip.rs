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
//! **The declared length is not a bound.** An IPv4 total length describes the
//! frame on the wire, and a truncated capture legitimately holds fewer bytes.
//! Reads are bounded by what was captured, always.

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
        fragment,
    })
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
                fragment_header = Some((ident, flags_and_offset >> 3, flags_and_offset & 1 != 0));
                next = bytes[cursor];
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
    use crate::parse::testframe::WithPayload;
    use crate::parse::transport::{IPPROTO_TCP, IPPROTO_UDP};

    fn v4(src: &str, dst: &str) -> (Ipv4Addr, Ipv4Addr) {
        (
            src.parse().expect("test address must parse"),
            dst.parse().expect("test address must parse"),
        )
    }

    // FR-012.
    #[test]
    fn a_minimum_length_ipv4_header_parses() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        })
        .with_payload(&f::tcp(51000, 443));
        let info = parse(NetProto::V4, &packet).expect("a legal header parses");
        assert_eq!(info.src, IpAddr::V4(s));
        assert_eq!(info.dst, IpAddr::V4(d));
        assert_eq!(info.proto, IPPROTO_TCP);
        assert_eq!(info.transport_offset, IPV4_MIN_HEADER_LEN);
        assert_eq!(info.fragment, None);
    }

    #[test]
    fn ipv4_options_are_skipped_by_the_declared_header_length() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_UDP,
            option_words: 3,
            ..f::V4::default()
        })
        .with_payload(&f::udp(30000, 5055, 8));
        let info = parse(NetProto::V4, &packet).expect("options are legal");
        assert_eq!(
            info.transport_offset,
            IPV4_MIN_HEADER_LEN + 12,
            "the transport starts after the options, not after twenty bytes"
        );
    }

    // FR-013.
    #[test]
    fn a_bare_ipv6_header_parses() {
        let packet = f::ipv6(f::V6 {
            next: IPPROTO_TCP,
            ..f::V6::default()
        })
        .with_payload(&f::tcp(51000, 443));
        let info = parse(NetProto::V6, &packet).expect("a legal header parses");
        assert_eq!(info.proto, IPPROTO_TCP);
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN);
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
            let packet = f::ipv6(f::V6 {
                next: ext,
                ..f::V6::default()
            })
            .with_payload(&f::extension(IPPROTO_UDP))
            .with_payload(&f::udp(30000, 5055, 8));
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
        let packet = f::ipv6(f::V6 {
            next: EXT_HOP_BY_HOP,
            ..f::V6::default()
        })
        .with_payload(&ext)
        .with_payload(&f::udp(1, 2, 8));
        let info = parse(NetProto::V6, &packet).expect("a two unit header is legal");
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN + 16);
    }

    #[test]
    fn a_chain_of_several_headers_is_walked_to_the_end() {
        let packet = f::ipv6(f::V6 {
            next: EXT_HOP_BY_HOP,
            ..f::V6::default()
        })
        .with_payload(&f::extension(EXT_ROUTING))
        .with_payload(&f::extension(EXT_DESTINATION_OPTIONS))
        .with_payload(&f::extension(IPPROTO_TCP))
        .with_payload(&f::tcp(1, 2));
        let info = parse(NetProto::V6, &packet).expect("three headers is legal");
        assert_eq!(info.proto, IPPROTO_TCP);
        assert_eq!(info.transport_offset, IPV6_FIXED_HEADER_LEN + 24);
    }

    // FR-015 and SC-007. Without the bound this test hangs rather than fails.
    #[test]
    fn a_chain_past_the_bound_terminates_the_walk() {
        let mut packet = f::ipv6(f::V6 {
            next: EXT_HOP_BY_HOP,
            ..f::V6::default()
        });
        for _ in 0..MAX_EXT_HEADERS + 1 {
            packet = packet.with_payload(&f::extension(EXT_HOP_BY_HOP));
        }
        packet = packet.with_payload(&f::tcp(1, 2));
        assert_eq!(
            parse(NetProto::V6, &packet),
            Err(ParseReject::ExtensionChainTooLong)
        );
    }

    #[test]
    fn a_chain_exactly_at_the_bound_still_parses() {
        let mut packet = f::ipv6(f::V6 {
            next: EXT_HOP_BY_HOP,
            ..f::V6::default()
        });
        for i in 0..MAX_EXT_HEADERS {
            let next = if i == MAX_EXT_HEADERS - 1 {
                IPPROTO_TCP
            } else {
                EXT_HOP_BY_HOP
            };
            packet = packet.with_payload(&f::extension(next));
        }
        packet = packet.with_payload(&f::tcp(1, 2));
        let info = parse(NetProto::V6, &packet).expect("eight headers is the bound, not past it");
        assert_eq!(info.proto, IPPROTO_TCP);
    }

    // FR-016.
    #[test]
    fn a_chain_ending_in_no_next_header_is_its_own_cause() {
        let packet = f::ipv6(f::V6 {
            next: NO_NEXT_HEADER,
            ..f::V6::default()
        });
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
        let mut packet = f::ipv4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        })
        .with_payload(&f::tcp(1, 2));
        packet[0] = 0x44; // four words, below the fixed header's five
        assert_eq!(
            parse(NetProto::V4, &packet),
            Err(ParseReject::MalformedNetworkHeader)
        );
    }

    #[test]
    fn a_legal_header_length_past_the_captured_bytes_is_short() {
        let (s, d) = v4("192.0.2.10", "198.51.100.5");
        let packet = f::ipv4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            option_words: 5,
            ..f::V4::default()
        });
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
        let mut packet = f::ipv4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_TCP,
            ..f::V4::default()
        })
        .with_payload(&f::tcp(1, 2));
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
        let packet = f::ipv4(f::V4 {
            src: s,
            dst: d,
            proto: IPPROTO_UDP,
            ..f::V4::default()
        })
        .with_payload(&f::udp(1, 2, 8));
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
            let packet = f::ipv4(f::V4 {
                src: s,
                dst: d,
                proto: IPPROTO_UDP,
                ident: 4242,
                frag_offset: offset,
                more_fragments: more,
                ..f::V4::default()
            })
            .with_payload(&f::udp(1, 2, 8));
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
        let packet = f::ipv6(f::V6 {
            next: EXT_FRAGMENT,
            ..f::V6::default()
        })
        .with_payload(&f::fragment_ext(IPPROTO_UDP, 0, false, 99))
        .with_payload(&f::udp(1, 2, 8));
        assert_eq!(parse(NetProto::V6, &packet).expect("legal").fragment, None);
    }

    #[test]
    fn an_ipv6_fragment_key_carries_the_thirty_two_bit_identification() {
        let packet = f::ipv6(f::V6 {
            next: EXT_FRAGMENT,
            ..f::V6::default()
        })
        .with_payload(&f::fragment_ext(IPPROTO_UDP, 185, true, 0xdeadbeef))
        .with_payload(&f::udp(1, 2, 8));
        let info = parse(NetProto::V6, &packet).expect("legal");
        let (key, role) = info.fragment.expect("this packet is a fragment");
        assert_eq!(role, FragmentRole::Subsequent);
        match key {
            FragmentKey::V6 { ident, .. } => assert_eq!(ident, 0xdeadbeef),
            FragmentKey::V4 { .. } => panic!("an IPv6 packet produced an IPv4 key"),
        }
    }
}
