// SPDX-License-Identifier: Apache-2.0

//! Transport headers: the two port fields, and nothing else.
//!
//! Specification section 12.5 names TCP and UDP. Anything else is a counted
//! rejection, including an encrypted payload, whose ports exist but cannot be
//! read. A port is never inferred, defaulted, or zeroed: a flow key built on a
//! port that was not observed is a confident wrong answer, which is the
//! outcome constitution P-9 exists to prevent.

use crate::flow::Proto;

use super::ParseReject;

pub(crate) const IPPROTO_TCP: u8 = 6;
pub(crate) const IPPROTO_UDP: u8 = 17;

/// The smallest prefix of a TCP header carrying both ports.
///
/// Only four bytes are required, because nothing beyond the ports is read.
/// Demanding the full twenty byte header would reject a snapshotted frame that
/// does carry everything this parser needs.
const TCP_PORTS_LEN: usize = 4;

/// The full UDP header.
///
/// Eight rather than four, unlike TCP, because the length field is read and
/// validated. The rule in both cases is the same: require exactly what is
/// read, and no more.
const UDP_HEADER_LEN: usize = 8;

/// Read the source and destination ports.
///
/// `bytes` begins at the transport header. Never reads past it.
pub(crate) fn ports(proto: u8, bytes: &[u8]) -> Result<(Proto, u16, u16), ParseReject> {
    match proto {
        IPPROTO_TCP => {
            if bytes.len() < TCP_PORTS_LEN {
                return Err(ParseReject::ShortHeader);
            }
            Ok((
                Proto::Tcp,
                u16::from_be_bytes([bytes[0], bytes[1]]),
                u16::from_be_bytes([bytes[2], bytes[3]]),
            ))
        }
        IPPROTO_UDP => {
            if bytes.len() < UDP_HEADER_LEN {
                return Err(ParseReject::ShortHeader);
            }
            let declared = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
            if declared < UDP_HEADER_LEN {
                // The ports are physically present, but a header that
                // contradicts itself is not a trustworthy source for them.
                return Err(ParseReject::MalformedTransportHeader);
            }
            Ok((
                Proto::Udp,
                u16::from_be_bytes([bytes[0], bytes[1]]),
                u16::from_be_bytes([bytes[2], bytes[3]]),
            ))
        }
        _ => Err(ParseReject::UnsupportedTransport),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::testframe as f;

    // FR-018.
    #[test]
    fn tcp_ports_are_read_in_network_order() {
        let seg = f::tcp(51000, 443);
        assert_eq!(
            ports(IPPROTO_TCP, &seg),
            Ok((Proto::Tcp, 51000, 443)),
            "big-endian, not host-endian"
        );
    }

    #[test]
    fn udp_ports_are_read_in_network_order() {
        let dg = f::udp(30000, 5055, 8);
        assert_eq!(ports(IPPROTO_UDP, &dg), Ok((Proto::Udp, 30000, 5055)));
    }

    #[test]
    fn tcp_needs_only_the_two_port_fields() {
        let seg = f::tcp(1, 2);
        assert_eq!(
            ports(IPPROTO_TCP, &seg[..TCP_PORTS_LEN]),
            Ok((Proto::Tcp, 1, 2)),
            "a snapshot that kept the ports kept enough"
        );
    }

    // FR-019.
    #[test]
    fn a_transport_that_is_neither_tcp_nor_udp_is_counted_as_unsupported() {
        // 1 is ICMP, 50 is the encapsulating security payload. Both arrive
        // here, and both are the same answer: fragcap has no ports to read.
        for proto in [1u8, 50, 132] {
            assert_eq!(
                ports(proto, &[0; 64]),
                Err(ParseReject::UnsupportedTransport)
            );
        }
    }

    // FR-020. Truncation never becomes an inferred port.
    #[test]
    fn a_tcp_header_truncated_before_both_ports_is_short() {
        for len in 0..TCP_PORTS_LEN {
            assert_eq!(
                ports(IPPROTO_TCP, &[0; 4][..len]),
                Err(ParseReject::ShortHeader),
                "{len} bytes is not two ports"
            );
        }
    }

    #[test]
    fn a_udp_header_truncated_before_its_length_field_is_short() {
        let dg = f::udp(1, 2, 8);
        for len in 0..UDP_HEADER_LEN {
            assert_eq!(
                ports(IPPROTO_UDP, &dg[..len]),
                Err(ParseReject::ShortHeader)
            );
        }
    }

    #[test]
    fn a_udp_length_shorter_than_its_own_header_is_malformed_not_short() {
        let dg = f::udp(1, 2, 4);
        assert_eq!(
            ports(IPPROTO_UDP, &dg),
            Err(ParseReject::MalformedTransportHeader),
            "the bytes are all present; the header disagrees with itself"
        );
    }

    #[test]
    fn a_udp_length_of_exactly_the_header_is_legal() {
        // A zero-payload datagram. Legal, and the boundary the malformed check
        // must not swallow.
        let dg = f::udp(1, 2, UDP_HEADER_LEN as u16);
        assert_eq!(ports(IPPROTO_UDP, &dg), Ok((Proto::Udp, 1, 2)));
    }
}
