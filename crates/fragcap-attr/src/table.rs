// SPDX-License-Identifier: Apache-2.0

//! What the operating system's socket table says, as an immutable value.
//!
//! Specification section 11.1: each snapshot yields a map from endpoint, meaning
//! the tuple of protocol, local address, and local port, to owning process
//! identifier. This module is that, normalized away from any one platform's row
//! layout, plus the socket creation instant that narrows the section 11.3 race
//! window.
//!
//! Constructible from declared entries, which is the whole reason it is a
//! separate type from the thing that reads it. Every matching rule in
//! [`crate::index`] is then a pure function of a value a test can write down.

use std::net::SocketAddr;

use fragcap_core::flow::{Endpoint, Proto};
use fragcap_core::packet::Timestamp;

/// One row of a socket table.
///
/// `remote` is an `Option` rather than being derived from `proto`, for two
/// reasons that point the same way. Specification section 8.4 forbids inventing
/// a remote endpoint for a UDP entry, because doing so produces confident wrong
/// attributions rather than honest coarse ones. And a listening TCP socket has
/// no peer either, so a field derived from the protocol would have to invent one
/// there too.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SocketTableEntry {
    pub proto: Proto,
    /// The address and port the socket is bound to, as the table reports it.
    /// May be a wildcard address; see [`crate::index`] for what that matches.
    pub local: SocketAddr,
    /// The peer, for a connected TCP socket. Never present for UDP.
    pub remote: Option<SocketAddr>,
    /// The owning process identifier.
    pub pid: u32,
    /// When the socket was created, if the platform reports it.
    ///
    /// `None` is not the same as "at the epoch": an entry with no reported
    /// instant still matches, it simply cannot be excluded by the section 11.3
    /// filter. Distinguishing the two is requirement FR-003.
    pub created: Option<Timestamp>,
}

impl SocketTableEntry {
    /// A connected TCP entry.
    pub fn tcp(local: SocketAddr, remote: SocketAddr, pid: u32) -> Self {
        SocketTableEntry {
            proto: Proto::Tcp,
            local,
            remote: Some(remote),
            pid,
            created: None,
        }
    }

    /// A listening TCP entry, which has no peer.
    pub fn tcp_listening(local: SocketAddr, pid: u32) -> Self {
        SocketTableEntry {
            proto: Proto::Tcp,
            local,
            remote: None,
            pid,
            created: None,
        }
    }

    /// A UDP entry.
    ///
    /// There is deliberately no constructor taking a remote for UDP. The rule
    /// is structural here rather than a comment someone has to read, which is
    /// the same choice [`fragcap_core::flow::AttributionKey`] makes.
    pub fn udp(local: SocketAddr, pid: u32) -> Self {
        SocketTableEntry {
            proto: Proto::Udp,
            local,
            remote: None,
            pid,
            created: None,
        }
    }

    /// Attach a creation instant.
    pub fn created_at(mut self, t: Timestamp) -> Self {
        self.created = Some(t);
        self
    }

    /// The endpoint this entry occupies: protocol, address, and port.
    pub fn endpoint(&self) -> Endpoint {
        Endpoint::new(self.local, self.proto)
    }
}

/// A whole socket table at one instant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocketTable {
    taken_at: Timestamp,
    entries: Vec<SocketTableEntry>,
}

/// An empty table taken at the epoch.
///
/// Written out rather than derived because [`Timestamp`] has no `Default`, and
/// deliberately so: an instant that nobody supplied is not zero, it is a
/// question about which instant was meant. Here the answer is explicit, and it
/// is only ever the state of an attributor that has not refreshed yet.
impl Default for SocketTable {
    fn default() -> Self {
        SocketTable {
            taken_at: Timestamp::from_nanos(0),
            entries: Vec::new(),
        }
    }
}

impl SocketTable {
    pub fn new(taken_at: Timestamp, entries: Vec<SocketTableEntry>) -> Self {
        SocketTable { taken_at, entries }
    }

    /// An empty table. Distinct from a failed read: this one was taken and
    /// reported nothing, which is a genuine observation.
    pub fn empty(taken_at: Timestamp) -> Self {
        SocketTable {
            taken_at,
            entries: Vec::new(),
        }
    }

    pub fn entries(&self) -> &[SocketTableEntry] {
        &self.entries
    }

    pub fn taken_at(&self) -> Timestamp {
        self.taken_at
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    // FR-001, FR-004. A declared table is the substrate every other test in
    // this crate stands on, so it round-trips exactly.
    #[test]
    fn a_declared_table_round_trips() {
        let t = SocketTable::new(
            Timestamp::from_nanos(1_000),
            vec![
                SocketTableEntry::tcp(addr("192.0.2.10:51000"), addr("198.51.100.5:443"), 42),
                SocketTableEntry::udp(addr("0.0.0.0:30000"), 43),
            ],
        );
        assert_eq!(t.taken_at(), Timestamp::from_nanos(1_000));
        assert_eq!(t.len(), 2);
        assert_eq!(t.entries()[0].pid, 42);
        assert_eq!(t.entries()[1].proto, Proto::Udp);
    }

    // FR-002. The asymmetry specification section 8.4 requires not be papered
    // over, asserted at the type that carries it.
    #[test]
    fn a_udp_entry_carries_no_remote() {
        let e = SocketTableEntry::udp(addr("192.0.2.10:30000"), 1);
        assert!(
            e.remote.is_none(),
            "a UDP table entry has no remote to report"
        );
    }

    #[test]
    fn a_connected_tcp_entry_carries_both_endpoints() {
        let e = SocketTableEntry::tcp(addr("192.0.2.10:51000"), addr("198.51.100.5:443"), 1);
        assert_eq!(e.remote, Some(addr("198.51.100.5:443")));
    }

    #[test]
    fn a_listening_tcp_entry_carries_no_remote_either() {
        let e = SocketTableEntry::tcp_listening(addr("0.0.0.0:443"), 1);
        assert_eq!(e.proto, Proto::Tcp);
        assert!(e.remote.is_none());
    }

    // FR-003. Absent is not the epoch.
    #[test]
    fn a_creation_instant_is_optional_and_distinct_from_zero() {
        let bare = SocketTableEntry::udp(addr("192.0.2.10:30000"), 1);
        let at_zero = bare.created_at(Timestamp::from_nanos(0));
        assert_eq!(bare.created, None);
        assert_eq!(at_zero.created, Some(Timestamp::from_nanos(0)));
        assert_ne!(bare.created, at_zero.created);
    }

    #[test]
    fn an_entry_reports_its_endpoint() {
        let e = SocketTableEntry::udp(addr("192.0.2.10:30000"), 1);
        assert_eq!(
            e.endpoint(),
            Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp)
        );
    }

    #[test]
    fn an_empty_table_is_a_real_observation() {
        let t = SocketTable::empty(Timestamp::from_nanos(5));
        assert!(t.is_empty());
        assert_eq!(t.taken_at(), Timestamp::from_nanos(5));
    }
}
