// SPDX-License-Identifier: Apache-2.0

//! Link layer dispatch: which network protocol follows, and at what offset.
//!
//! Three encapsulations, per specification section 12.5 and the shared libpcap
//! and pcapng registry that its "Ethernet and raw IP link types" resolves
//! against. Anything else is a counted rejection rather than a guess.

use crate::link::LinkType;

use super::ParseReject;

/// Which network layer parser the link layer named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetProto {
    V4,
    V6,
}

const ETHERNET_HEADER_LEN: usize = 14;
const ETHERTYPE_IPV4: u16 = 0x0800;
const ETHERTYPE_IPV6: u16 = 0x86dd;

const LOOPBACK_HEADER_LEN: usize = 4;
/// `AF_INET`, which is 2 on every platform that produces this encapsulation.
const AF_INET: u32 = 2;
/// `AF_INET6`, which is not. Linux, Windows, FreeBSD, OpenBSD and macOS, and
/// other Darwin-derived systems each chose differently, and a fixture recorded
/// on any of them should be readable. See research R-2.
const AF_INET6_VALUES: [u32; 5] = [10, 23, 24, 28, 30];

/// Read the link layer header and report what follows it.
///
/// Returns the network protocol and the offset at which its header begins.
/// Never reads past `frame`.
pub(crate) fn dispatch(link: LinkType, frame: &[u8]) -> Result<(NetProto, usize), ParseReject> {
    match link {
        LinkType::ETHERNET => ethernet(frame),
        LinkType::RAW => raw_ip(frame),
        LinkType::NULL => bsd_loopback(frame),
        _ => Err(ParseReject::UnsupportedLinkType),
    }
}

fn ethernet(frame: &[u8]) -> Result<(NetProto, usize), ParseReject> {
    if frame.len() < ETHERNET_HEADER_LEN {
        return Err(ParseReject::ShortHeader);
    }
    let ether_type = u16::from_be_bytes([frame[12], frame[13]]);
    match ether_type {
        ETHERTYPE_IPV4 => Ok((NetProto::V4, ETHERNET_HEADER_LEN)),
        ETHERTYPE_IPV6 => Ok((NetProto::V6, ETHERNET_HEADER_LEN)),
        // VLAN tags land here, per FR-009. Specification section 12.5
        // enumerates what fragcap parses and does not name them, so they are
        // counted rather than handled, which is what makes the gap visible if
        // it ever fires.
        _ => Err(ParseReject::UnsupportedEtherType),
    }
}

fn raw_ip(frame: &[u8]) -> Result<(NetProto, usize), ParseReject> {
    let first = *frame.first().ok_or(ParseReject::ShortHeader)?;
    match first >> 4 {
        4 => Ok((NetProto::V4, 0)),
        6 => Ok((NetProto::V6, 0)),
        _ => Err(ParseReject::UnsupportedIpVersion),
    }
}

/// BSD loopback encapsulation: a four byte address family value in the
/// capturing host's byte order.
///
/// The value is read in both byte orders and the reading that matches a known
/// family wins. That resolves rather than guesses: every known value is small,
/// so its byte-swapped form is a large value with its low bytes zero, and no
/// such value is in the known set. A four byte field therefore has at most one
/// valid reading.
fn bsd_loopback(frame: &[u8]) -> Result<(NetProto, usize), ParseReject> {
    if frame.len() < LOOPBACK_HEADER_LEN {
        return Err(ParseReject::ShortHeader);
    }
    let bytes = [frame[0], frame[1], frame[2], frame[3]];
    for family in [u32::from_le_bytes(bytes), u32::from_be_bytes(bytes)] {
        if family == AF_INET {
            return Ok((NetProto::V4, LOOPBACK_HEADER_LEN));
        }
        if AF_INET6_VALUES.contains(&family) {
            return Ok((NetProto::V6, LOOPBACK_HEADER_LEN));
        }
    }
    Err(ParseReject::UnsupportedAddressFamily)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::testframe as f;

    // FR-006.
    #[test]
    fn ethernet_dispatches_on_the_ether_type() {
        let v4 = f::ethernet(ETHERTYPE_IPV4, &[0; 20]);
        assert_eq!(
            dispatch(LinkType::ETHERNET, &v4),
            Ok((NetProto::V4, ETHERNET_HEADER_LEN))
        );
        let v6 = f::ethernet(ETHERTYPE_IPV6, &[0; 40]);
        assert_eq!(
            dispatch(LinkType::ETHERNET, &v6),
            Ok((NetProto::V6, ETHERNET_HEADER_LEN))
        );
    }

    // FR-007.
    #[test]
    fn raw_ip_dispatches_on_the_version_nibble_at_offset_zero() {
        assert_eq!(dispatch(LinkType::RAW, &[0x45; 20]), Ok((NetProto::V4, 0)));
        assert_eq!(dispatch(LinkType::RAW, &[0x60; 40]), Ok((NetProto::V6, 0)));
    }

    // FR-008 and research R-2.
    #[test]
    fn bsd_loopback_accepts_the_ipv4_family_in_either_byte_order() {
        let le = f::loopback_raw(&AF_INET.to_le_bytes(), &[0x45; 20]);
        let be = f::loopback_raw(&AF_INET.to_be_bytes(), &[0x45; 20]);
        assert_eq!(
            dispatch(LinkType::NULL, &le),
            Ok((NetProto::V4, LOOPBACK_HEADER_LEN))
        );
        assert_eq!(
            dispatch(LinkType::NULL, &be),
            Ok((NetProto::V4, LOOPBACK_HEADER_LEN))
        );
    }

    #[test]
    fn bsd_loopback_accepts_every_known_ipv6_family_value() {
        for family in AF_INET6_VALUES {
            for bytes in [family.to_le_bytes(), family.to_be_bytes()] {
                let frame = f::loopback_raw(&bytes, &[0x60; 40]);
                assert_eq!(
                    dispatch(LinkType::NULL, &frame),
                    Ok((NetProto::V6, LOOPBACK_HEADER_LEN)),
                    "family {family} must be recognized in both byte orders"
                );
            }
        }
    }

    #[test]
    fn no_known_family_is_also_a_known_family_byte_swapped() {
        // The property that makes accepting both orders unambiguous rather
        // than a guess. If it ever stops holding, this fails before any
        // fixture misparses.
        let known: Vec<u32> = std::iter::once(AF_INET).chain(AF_INET6_VALUES).collect();
        for value in &known {
            let swapped = value.swap_bytes();
            assert!(
                !known.contains(&swapped),
                "{value} byte-swapped is {swapped}, which is also known"
            );
        }
    }

    // FR-009 and FR-010.
    #[test]
    fn a_vlan_tagged_frame_is_an_unsupported_ether_type() {
        let frame = f::ethernet(0x8100, &[0; 20]);
        assert_eq!(
            dispatch(LinkType::ETHERNET, &frame),
            Err(ParseReject::UnsupportedEtherType)
        );
    }

    #[test]
    fn an_unhandled_link_type_is_rejected_without_reading_the_frame() {
        assert_eq!(
            dispatch(LinkType::from_code(108), &[]),
            Err(ParseReject::UnsupportedLinkType),
            "the link type is wrong before the frame's length can be"
        );
    }

    #[test]
    fn an_unknown_address_family_is_its_own_cause() {
        let frame = f::loopback_raw(&99u32.to_le_bytes(), &[0x45; 20]);
        assert_eq!(
            dispatch(LinkType::NULL, &frame),
            Err(ParseReject::UnsupportedAddressFamily)
        );
    }

    #[test]
    fn an_unknown_ip_version_on_a_raw_frame_is_its_own_cause() {
        assert_eq!(
            dispatch(LinkType::RAW, &[0x50; 20]),
            Err(ParseReject::UnsupportedIpVersion)
        );
    }

    #[test]
    fn a_frame_shorter_than_its_link_header_is_short_rather_than_malformed() {
        assert_eq!(
            dispatch(LinkType::ETHERNET, &[0; 13]),
            Err(ParseReject::ShortHeader)
        );
        assert_eq!(
            dispatch(LinkType::NULL, &[0; 3]),
            Err(ParseReject::ShortHeader)
        );
        assert_eq!(dispatch(LinkType::RAW, &[]), Err(ParseReject::ShortHeader));
    }
}
