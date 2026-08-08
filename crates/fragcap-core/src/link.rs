// SPDX-License-Identifier: Apache-2.0

//! Link layer encapsulation reported by a packet source.

/// The link layer encapsulation a source produces.
///
/// Modelled as the standard link-layer type code rather than a closed
/// enumeration. The codes are the de facto registry shared by libpcap and
/// pcapng, so a backend reporting one fragcap has never seen is representable
/// rather than a parse failure, which matters because slice S09 discovers what
/// npcap actually reports on real interfaces.
///
/// Named constants cover what the focal titles need. Anything else arrives as
/// its code and is written through unchanged, which is what constitution P-9
/// requires of an observation fragcap does not understand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkType(u16);

impl LinkType {
    /// BSD loopback encapsulation: a four byte address family value in the
    /// capturing host's byte order, then a network layer header.
    ///
    /// S02 documented this constant as having no link layer header, which is
    /// code 101's property rather than this one's. The error was harmless
    /// while nothing parsed and was corrected in S03, which is the slice that
    /// would have read a network header out of the address family field. See
    /// that slice's plan decision D-7.
    pub const NULL: LinkType = LinkType(0);
    /// Ethernet, which every focal title's traffic is carried over.
    pub const ETHERNET: LinkType = LinkType(1);
    /// Raw IP: no link layer header at all, so the frame begins with a network
    /// layer header.
    pub const RAW: LinkType = LinkType(101);

    pub const fn from_code(code: u16) -> Self {
        LinkType(code)
    }

    pub const fn code(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_constants_carry_the_standard_codes() {
        assert_eq!(LinkType::NULL.code(), 0);
        assert_eq!(LinkType::ETHERNET.code(), 1);
        assert_eq!(LinkType::RAW.code(), 101);
    }

    #[test]
    fn an_unknown_code_is_representable_rather_than_a_failure() {
        let odd = LinkType::from_code(276);
        assert_eq!(odd.code(), 276);
        assert_ne!(odd, LinkType::ETHERNET);
    }

    #[test]
    fn codes_round_trip() {
        for c in [0_u16, 1, 101, 65535] {
            assert_eq!(LinkType::from_code(c).code(), c);
        }
    }
}
