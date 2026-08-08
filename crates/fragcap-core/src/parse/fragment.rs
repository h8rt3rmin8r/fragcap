// SPDX-License-Identifier: Apache-2.0

//! The fragment identity table.
//!
//! Specification section 12.5 requires that subsequent fragments be attributed
//! by their fragment identifier and address pair, and separately refuses to
//! reassemble. Those two together imply a memory: the transport header lives
//! only in the first fragment, so attributing the rest means remembering what
//! the first one said.
//!
//! The section does not describe that memory, so this slice defines it and
//! records the divergence for promotion to specification section 29. It is
//! bounded by entry count, it evicts oldest first, and it holds no clock. See
//! plan decision D-4 for why bounding by age was rejected.

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::flow::Proto;

/// How many datagrams the table remembers at once.
///
/// Generous against the reconnaissance finding that the focal titles' traffic
/// is predominantly unfragmented, and small against the 65,536 packet ring
/// specification section 12.4 already budgets.
pub(crate) const FRAGMENT_TABLE_CAPACITY: usize = 256;

/// What associates the fragments of one datagram with each other.
///
/// The two address families are separate variants because the two standards
/// define different reassembly keys. IPv4 keys on source, destination,
/// protocol, and a sixteen bit identification, because that identification is
/// only unique per protocol. IPv6 keys on source, destination, and a thirty
/// two bit identification, and its fragment header carries no protocol number
/// at all.
///
/// A single shared definition would have meant inventing a protocol number for
/// the IPv6 case, which is the same class of fabrication specification section
/// 8.4 prohibits for UDP remote endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FragmentKey {
    V4 {
        src: Ipv4Addr,
        dst: Ipv4Addr,
        proto: u8,
        ident: u16,
    },
    V6 {
        src: Ipv6Addr,
        dst: Ipv6Addr,
        ident: u32,
    },
}

/// What the first fragment's transport header said, in wire order.
///
/// Wire order rather than an assembled [`crate::flow::FlowKey`], deliberately.
/// Every fragment carries the same wire-order address pair, but the interface
/// address set may change between the first fragment and the last, so the
/// local position and the direction have to be recomputed per fragment.
/// Storing ports rather than a key makes that fall out of the design instead
/// of being a rule someone has to remember. See FR-022a.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FragmentPorts {
    pub proto: Proto,
    pub src_port: u16,
    pub dst_port: u16,
}

/// A fixed-size ring of remembered fragment identities.
///
/// No allocation on any path, including construction. Lookup is a linear scan,
/// which at this size beats hashing and costs nothing on the overwhelmingly
/// common path where no fragment is present at all.
pub(crate) struct FragmentTable {
    slots: [Option<(FragmentKey, FragmentPorts)>; FRAGMENT_TABLE_CAPACITY],
    cursor: usize,
}

impl Default for FragmentTable {
    fn default() -> Self {
        FragmentTable {
            slots: [None; FRAGMENT_TABLE_CAPACITY],
            cursor: 0,
        }
    }
}

impl FragmentTable {
    /// Remember what a first fragment resolved to. Returns whether doing so
    /// evicted a live entry, so the caller advances the eviction counter at the
    /// one site that knows.
    ///
    /// A repeated key overwrites in place rather than adding a second entry,
    /// because a retransmitted first fragment would otherwise leave a stale
    /// duplicate behind after the real one is taken.
    ///
    /// When the table is not full the write lands on whatever slot the cursor
    /// points at, which may be a hole left by a previous take. The cursor
    /// advances monotonically, so on a full table it is the oldest entry that
    /// goes. On a table with holes it is possible for a live entry to be
    /// evicted while a hole exists elsewhere; that is a bounded imprecision in
    /// exchange for never scanning twice, and it is stated rather than hidden.
    pub(crate) fn record(&mut self, key: FragmentKey, ports: FragmentPorts) -> bool {
        if let Some(slot) = self.find_mut(&key) {
            *slot = Some((key, ports));
            return false;
        }
        let evicted = self.slots[self.cursor].is_some();
        self.slots[self.cursor] = Some((key, ports));
        self.cursor = (self.cursor + 1) % FRAGMENT_TABLE_CAPACITY;
        evicted
    }

    /// What the first fragment of this datagram said, if it was seen.
    pub(crate) fn lookup(&self, key: &FragmentKey) -> Option<FragmentPorts> {
        self.slots
            .iter()
            .flatten()
            .find(|(k, _)| k == key)
            .map(|(_, p)| *p)
    }

    /// Look up and forget, for the last fragment of a datagram.
    ///
    /// Forgetting here is what stops an entry outliving the datagram it
    /// describes, and it is the main thing narrowing the identifier reuse
    /// window described in the spec's Known limitation section.
    pub(crate) fn take(&mut self, key: &FragmentKey) -> Option<FragmentPorts> {
        let slot = self.find_mut(key)?;
        slot.take().map(|(_, p)| p)
    }

    /// How many entries are live. Test support only: the operator-facing
    /// signal is `fragment_evicted`, not the occupancy.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    fn find_mut(&mut self, key: &FragmentKey) -> Option<&mut Option<(FragmentKey, FragmentPorts)>> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s, Some((k, _)) if k == key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(ident: u16) -> FragmentKey {
        FragmentKey::V4 {
            src: Ipv4Addr::new(192, 0, 2, 10),
            dst: Ipv4Addr::new(198, 51, 100, 5),
            proto: 17,
            ident,
        }
    }

    fn v6(ident: u32) -> FragmentKey {
        FragmentKey::V6 {
            src: "2001:db8::10".parse().expect("test address must parse"),
            dst: "2001:db8::5".parse().expect("test address must parse"),
            ident,
        }
    }

    fn ports() -> FragmentPorts {
        FragmentPorts {
            proto: Proto::Udp,
            src_port: 30000,
            dst_port: 5055,
        }
    }

    #[test]
    fn a_recorded_identity_is_found_again() {
        let mut t = FragmentTable::default();
        assert!(!t.record(v4(7), ports()));
        assert_eq!(t.lookup(&v4(7)), Some(ports()));
    }

    #[test]
    fn an_identity_never_recorded_is_not_found() {
        let t = FragmentTable::default();
        assert_eq!(t.lookup(&v4(7)), None);
    }

    #[test]
    fn the_two_address_families_do_not_collide() {
        let mut t = FragmentTable::default();
        t.record(v4(7), ports());
        assert_eq!(t.lookup(&v6(7)), None, "a u16 7 is not a u32 7 here");
    }

    // FR-022. The IPv4 key carries the protocol number; changing it must miss.
    #[test]
    fn the_ipv4_key_discriminates_on_the_protocol_number() {
        let mut t = FragmentTable::default();
        t.record(v4(7), ports());
        let tcp_same_ident = FragmentKey::V4 {
            src: Ipv4Addr::new(192, 0, 2, 10),
            dst: Ipv4Addr::new(198, 51, 100, 5),
            proto: 6,
            ident: 7,
        };
        assert_eq!(
            t.lookup(&tcp_same_ident),
            None,
            "identification is only unique per protocol"
        );
    }

    // FR-024a. Taking is what stops an entry outliving its datagram.
    #[test]
    fn taking_removes_the_entry() {
        let mut t = FragmentTable::default();
        t.record(v4(7), ports());
        assert_eq!(t.take(&v4(7)), Some(ports()));
        assert_eq!(t.lookup(&v4(7)), None);
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn taking_an_absent_entry_reports_nothing_and_changes_nothing() {
        let mut t = FragmentTable::default();
        t.record(v4(7), ports());
        assert_eq!(t.take(&v4(8)), None);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn a_repeated_identity_overwrites_rather_than_duplicating() {
        let mut t = FragmentTable::default();
        t.record(v4(7), ports());
        let other = FragmentPorts {
            proto: Proto::Tcp,
            src_port: 1,
            dst_port: 2,
        };
        assert!(!t.record(v4(7), other));
        assert_eq!(t.len(), 1);
        assert_eq!(t.lookup(&v4(7)), Some(other));
    }

    // FR-024. Overflow evicts and says so.
    #[test]
    fn filling_the_table_evicts_the_oldest_and_reports_it() {
        let mut t = FragmentTable::default();
        for i in 0..FRAGMENT_TABLE_CAPACITY {
            assert!(
                !t.record(v4(i as u16), ports()),
                "no eviction while slots remain"
            );
        }
        assert_eq!(t.len(), FRAGMENT_TABLE_CAPACITY);
        assert!(
            t.record(v4(9999), ports()),
            "the entry past capacity must evict"
        );
        assert_eq!(t.lookup(&v4(0)), None, "the oldest went");
        assert_eq!(t.lookup(&v4(9999)), Some(ports()));
        assert_eq!(
            t.len(),
            FRAGMENT_TABLE_CAPACITY,
            "the bound is a ceiling, not a target"
        );
    }
}
