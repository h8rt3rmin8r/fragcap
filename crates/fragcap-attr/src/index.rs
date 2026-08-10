// SPDX-License-Identifier: Apache-2.0

//! The join, and the value it reads.
//!
//! Specification section 11.1 makes attribution a two-stage lookup: endpoint to
//! process identifier through the socket table, and process identifier to image
//! name and role through the process tree. This module is stage one plus the
//! naming half of stage two; roles arrive with slice S12.
//!
//! Everything here is a pure function of a value. An [`AttributionIndex`] holds
//! a socket table, the names resolved for the identifiers in it, and the
//! retention map of specification section 11.4, and answers questions about a
//! flow at an instant. It performs no I/O, takes no lock, and reads no clock.
//!
//! # Why the order is spelled out
//!
//! More than one table entry can match one flow: a wildcard bind and a specific
//! bind on the same port, or two sockets that held the same port at different
//! times. "Prefer the more exact match" reads as settled and is not. An
//! implementation that iterates the platform's rows and takes the first hit
//! produces an answer that depends on row order, and therefore changes between
//! runs over identical traffic. [`MatchRank`] and the tiebreaks below make the
//! order total, and a test permutes a table to prove it.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use arc_swap::ArcSwap;
use fragcap_core::attribution::{Attribution, Fidelity};
use fragcap_core::flow::{AttributionKey, Endpoint, FlowKey, Proto};
use fragcap_core::packet::Timestamp;

use crate::table::{SocketTable, SocketTableEntry};

/// How exactly a table entry matched a flow, most exact first.
///
/// The variants are ordered, and the derived `Ord` is load-bearing: a smaller
/// rank is a better match. They are mutually exclusive by construction, because
/// [`rank_of`] tests them in order and the first that holds fixes the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchRank {
    /// TCP, local and remote both equal. The table carried both endpoints and
    /// both agreed, which is as certain as this join gets.
    BothEndpoints,
    /// The local address and port matched exactly.
    ExactLocal,
    /// The entry is bound to a wildcard address of the same family, on the
    /// port the flow used. Specification section 8.4: the table reports the
    /// bind address rather than the address a datagram arrived on.
    WildcardBind,
    /// An IPv6 wildcard bind against an IPv4 local endpoint. A dual-stack
    /// socket, which `AttributionKey::local_matches_bind` names slice S10 as
    /// the owner of.
    DualStack,
}

/// The rank at which `entry` matches `key`, or `None` if it does not.
///
/// The wildcard and dual-stack allowances are UDP only. A TCP entry carries
/// both endpoints, so an exact local match is available and a looser rule would
/// only add false positives; `AttributionKey` already encodes that, and this
/// function does not widen it.
fn rank_of(entry: &SocketTableEntry, key: &FlowKey) -> Option<MatchRank> {
    if entry.proto != key.proto {
        return None;
    }
    let akey = key.attribution_key();
    match akey {
        AttributionKey::Pair(local, remote) => {
            // TCP. Both endpoints or nothing.
            if entry.local == local && entry.remote == Some(remote) {
                Some(MatchRank::BothEndpoints)
            } else {
                None
            }
        }
        AttributionKey::Local(local) => {
            // UDP. The local endpoint alone, and never against a remote.
            if entry.local == local {
                return Some(MatchRank::ExactLocal);
            }
            if entry.local.port() != local.port() {
                return None;
            }
            match (entry.local.ip(), local.ip()) {
                (IpAddr::V4(bind), IpAddr::V4(_)) if bind.is_unspecified() => {
                    Some(MatchRank::WildcardBind)
                }
                (IpAddr::V6(bind), IpAddr::V6(_)) if bind.is_unspecified() => {
                    Some(MatchRank::WildcardBind)
                }
                // The dual-stack case. An IPv6 wildcard bind accepts IPv4
                // traffic, and the table reports the bind rather than the
                // address the datagram arrived on.
                //
                // Appendix D found no focal title relying on this, which makes
                // the rule unexercised by them rather than wrong. Refusing it
                // would make a whole class of sockets silently unattributable,
                // and a silent unattributable class is worse than a rare
                // imprecise match that ranks below every exact one.
                (IpAddr::V6(bind), IpAddr::V4(_)) if bind.is_unspecified() => {
                    Some(MatchRank::DualStack)
                }
                _ => None,
            }
        }
    }
}

/// A table entry that matched, with its rank.
#[derive(Clone, Copy, Debug)]
struct Candidate<'a> {
    entry: &'a SocketTableEntry,
    rank: MatchRank,
}

/// The total order over candidates. Smaller is better.
///
/// Three terms, applied in order:
///
/// 1. The rank. A more exact match always wins.
/// 2. The creation instant, latest first, with `None` last. Under port reuse
///    the socket created most recently at or before the packet is the one that
///    most plausibly owned it, and an entry whose creation the platform did not
///    report cannot make that claim.
/// 3. The process identifier, ascending. This decides nothing meaningful and
///    exists only so the order is total rather than dependent on the order the
///    platform reported its rows in. Do not give it a meaning it does not have.
fn better(a: &Candidate<'_>, b: &Candidate<'_>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match a.rank.cmp(&b.rank) {
        Ordering::Equal => {}
        other => return other,
    }
    // Latest creation first; `None` sorts after every `Some`.
    let by_created = match (a.entry.created, b.entry.created) {
        (Some(x), Some(y)) => y.as_nanos().cmp(&x.as_nanos()),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    match by_created {
        Ordering::Equal => a.entry.pid.cmp(&b.entry.pid),
        other => other,
    }
}

/// An endpoint that has left the table but is still resolvable.
///
/// Specification section 11.4. Retention exists because capture and socket
/// table observation are not synchronized: a connection closing produces final
/// packets processed after the socket has gone, and discarding attribution at
/// that instant would leave the tail of every connection unattributed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetainedEntry {
    pub pid: u32,
    pub created: Option<Timestamp>,
    pub remote: Option<SocketAddr>,
    /// When this endpoint was last observed *present* in a table.
    ///
    /// Not the instant of the refresh that noticed it gone. Those differ by up
    /// to one interval, and measuring from the later one would make a thirty
    /// second window silently thirty-one seconds. A retained answer carries a
    /// marked risk of being wrong, and quietly widening the window that
    /// produces them widens that risk without saying so.
    pub last_seen: Timestamp,
}

/// Endpoints that have left the table, with their last known owner.
pub type RetentionMap = HashMap<Endpoint, RetainedEntry>;

/// The published value a lookup reads.
///
/// Everything an answer can contain is in here before the lookup begins: the
/// table, the names, and the retention map. That is requirement SC-015, and it
/// is what makes specification section 11.6's promise, that attribution lookup
/// never blocks packet acquisition, true rather than aspirational.
#[derive(Clone, Debug, Default)]
pub struct AttributionIndex {
    table: SocketTable,
    names: HashMap<u32, Arc<str>>,
    retained: RetentionMap,
    retention: i64,
}

impl AttributionIndex {
    pub fn new(
        table: SocketTable,
        names: HashMap<u32, Arc<str>>,
        retained: RetentionMap,
        retention_nanos: i64,
    ) -> Self {
        AttributionIndex {
            table,
            names,
            retained,
            retention: retention_nanos,
        }
    }

    pub fn table(&self) -> &SocketTable {
        &self.table
    }

    pub fn retained(&self) -> &RetentionMap {
        &self.retained
    }

    /// Who owned this flow at `at`, if it can be determined.
    ///
    /// `at` is the packet's own instant and never the present moment.
    /// Specification section 11.4 is explicit that the question is always who
    /// owned this flow *then*.
    pub fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution> {
        // A live entry beats a retained one, always. The table is evidence and
        // retention is inference, and the case where they disagree is exactly
        // the port-reuse case where the inference is wrong.
        if let Some(entry) = self.best_live(key, at) {
            return Some(self.attribution(entry.pid, Fidelity::Live));
        }
        self.best_retained(key, at)
            .map(|r| self.attribution(r.pid, Fidelity::Retained))
    }

    /// Whether the index carries any entry, live or retained, on this key's
    /// local endpoint.
    ///
    /// Requirement FR-014: an unresolved lookup on an endpoint the index does
    /// not carry is what triggers a refresh, and a lookup on an endpoint the
    /// index does carry is a flow that is simply not attributable, which
    /// triggers nothing.
    pub fn carries(&self, key: &FlowKey) -> bool {
        let endpoint = Endpoint::new(key.attribution_key().local(), key.proto);
        if self.retained.contains_key(&endpoint) {
            return true;
        }
        self.table
            .entries()
            .iter()
            .any(|e| rank_of(e, key).is_some())
    }

    /// Every endpoint believed active at `at`: those in the table, plus those
    /// still inside the retention window. Requirement FR-023.
    pub fn endpoints(&self, at: Timestamp) -> Vec<Endpoint> {
        let mut out: Vec<Endpoint> = self.table.entries().iter().map(|e| e.endpoint()).collect();
        for (endpoint, r) in self.retained.iter() {
            if self.within_retention(r, at) && !out.contains(endpoint) {
                out.push(*endpoint);
            }
        }
        out
    }

    fn attribution(&self, pid: u32, fidelity: Fidelity) -> Attribution {
        match self.names.get(&pid) {
            Some(name) => Attribution {
                pid,
                process: Arc::clone(name),
                role: None,
                stage: None,
                fidelity,
            },
            // No name was resolved. The attribution is produced anyway,
            // carrying the identifier, because the identifier is what was
            // observed. Constitution P-9 and requirement FR-032: reporting
            // nothing here would discard an observation because a convenience
            // could not be supplied.
            None => Attribution::new(pid, "", fidelity),
        }
    }

    fn best_live(&self, key: &FlowKey, at: Timestamp) -> Option<&SocketTableEntry> {
        self.table
            .entries()
            .iter()
            .filter(|e| Self::existed_by(e.created, at))
            .filter_map(|entry| rank_of(entry, key).map(|rank| Candidate { entry, rank }))
            .min_by(better)
            .map(|c| c.entry)
    }

    fn best_retained(&self, key: &FlowKey, at: Timestamp) -> Option<&RetainedEntry> {
        let endpoint = Endpoint::new(key.attribution_key().local(), key.proto);
        let r = self.retained.get(&endpoint)?;
        if !self.within_retention(r, at) {
            return None;
        }
        if !Self::existed_by(r.created, at) {
            return None;
        }
        // A retained TCP entry still has to agree on the remote, when it
        // recorded one. The flow's identity is both endpoints for TCP, and a
        // retained entry that answered about any peer would be a broader claim
        // than the live path makes.
        if key.proto == Proto::Tcp {
            if let (Some(recorded), AttributionKey::Pair(_, remote)) =
                (r.remote, key.attribution_key())
            {
                if recorded != remote {
                    return None;
                }
            }
        }
        Some(r)
    }

    fn within_retention(&self, r: &RetainedEntry, at: Timestamp) -> bool {
        let elapsed = at.nanos_since(r.last_seen);
        // A packet from before the endpoint was last seen is inside the window
        // trivially: the endpoint was present then. Only elapsed time past the
        // grace period expires it.
        elapsed < self.retention
    }

    /// Whether a socket that reports `created` could have existed at `at`.
    ///
    /// Requirement FR-009. A socket created after the packet cannot have owned
    /// it, and rejecting it is the only mechanism available that tells the
    /// previous owner of a reused port from the current one. An entry with no
    /// reported creation instant is not excluded: absence of evidence is not
    /// evidence, and excluding it would unattribute every socket on a platform
    /// that reports no creation time at all.
    fn existed_by(created: Option<Timestamp>, at: Timestamp) -> bool {
        match created {
            Some(c) => c.as_nanos() <= at.as_nanos(),
            None => true,
        }
    }
}

/// The section 11.6 publication cell.
///
/// The control thread builds a new index on each refresh and publishes it
/// atomically; any number of capture threads read the current one without
/// locking. That is the whole of specification section 11.6, and it is why this
/// is a separate type rather than a private field of the attributor: a test
/// that demonstrates concurrent resolution across a publication has to publish
/// from one thread while others read, which a `&mut self` method on a shared
/// object cannot express. Slice S13's control thread needs the same seam.
#[derive(Debug, Default)]
pub struct PublishedIndex(ArcSwap<AttributionIndex>);

impl PublishedIndex {
    pub fn new(index: AttributionIndex) -> Self {
        PublishedIndex(ArcSwap::from_pointee(index))
    }

    /// The current index. Wait-free: an atomic load, no lock taken and none
    /// waited on.
    ///
    /// The returned handle stays valid across a subsequent [`Self::publish`],
    /// so a lookup in progress is unaffected by one.
    pub fn load(&self) -> Arc<AttributionIndex> {
        self.0.load_full()
    }

    /// Replace the index as a whole. A reader sees either the whole old index
    /// or the whole new one, never a mixture.
    pub fn publish(&self, index: AttributionIndex) {
        self.0.store(Arc::new(index));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::SocketTableEntry;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn tcp_key() -> FlowKey {
        FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
    }

    fn udp_key() -> FlowKey {
        FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        )
    }

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    fn names(pairs: &[(u32, &str)]) -> HashMap<u32, Arc<str>> {
        pairs.iter().map(|(p, n)| (*p, Arc::from(*n))).collect()
    }

    /// An index over declared entries, with a thirty second retention and no
    /// retained endpoints.
    fn index(entries: Vec<SocketTableEntry>, named: &[(u32, &str)]) -> AttributionIndex {
        AttributionIndex::new(
            SocketTable::new(at(0), entries),
            names(named),
            RetentionMap::new(),
            30_000_000_000,
        )
    }

    // --- FR-005, FR-006: the per-protocol match rules -------------------

    #[test]
    fn tcp_matches_on_both_endpoints() {
        let i = index(
            vec![SocketTableEntry::tcp(
                addr("192.0.2.10:51000"),
                addr("198.51.100.5:443"),
                42,
            )],
            &[(42, "eso64.exe")],
        );
        let a = i.resolve(&tcp_key(), at(100)).expect("the entry matches");
        assert_eq!(a.pid, 42);
        assert_eq!(&*a.process, "eso64.exe");
        assert_eq!(a.fidelity, Fidelity::Live);
    }

    #[test]
    fn tcp_does_not_match_on_the_local_endpoint_alone() {
        // A different peer on the same local port is a different flow.
        let i = index(
            vec![SocketTableEntry::tcp(
                addr("192.0.2.10:51000"),
                addr("203.0.113.9:443"),
                42,
            )],
            &[],
        );
        assert_eq!(i.resolve(&tcp_key(), at(100)), None);
    }

    #[test]
    fn udp_matches_on_the_local_endpoint_alone() {
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 43)],
            &[(43, "game.exe")],
        );
        assert_eq!(i.resolve(&udp_key(), at(100)).expect("matches").pid, 43);
    }

    #[test]
    fn udp_never_matches_against_a_remote() {
        // The same local endpoint, any peer at all. If a remote ever entered
        // the UDP comparison this would start failing for some peers.
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 43)],
            &[],
        );
        for peer in ["198.51.100.5:5055", "203.0.113.9:1", "192.0.2.99:65535"] {
            let key = FlowKey::new(Proto::Udp, addr("192.0.2.10:30000"), addr(peer));
            assert!(i.resolve(&key, at(100)).is_some(), "peer {peer}");
        }
    }

    #[test]
    fn a_flow_never_matches_an_entry_of_the_other_protocol() {
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:51000"), 43)],
            &[],
        );
        assert_eq!(
            i.resolve(&tcp_key(), at(100)),
            None,
            "the protocol participates in the match"
        );
    }

    // --- FR-007: wildcard and dual-stack --------------------------------

    #[test]
    fn a_udp_wildcard_bind_matches_a_specific_local_address() {
        let i = index(vec![SocketTableEntry::udp(addr("0.0.0.0:30000"), 9)], &[]);
        assert_eq!(i.resolve(&udp_key(), at(100)).expect("matches").pid, 9);
    }

    #[test]
    fn a_wildcard_bind_still_requires_the_port_to_match() {
        let i = index(vec![SocketTableEntry::udp(addr("0.0.0.0:30001"), 9)], &[]);
        assert_eq!(i.resolve(&udp_key(), at(100)), None);
    }

    #[test]
    fn an_ipv6_wildcard_bind_matches_an_ipv6_local_endpoint() {
        let i = index(vec![SocketTableEntry::udp(addr("[::]:30000"), 9)], &[]);
        let key = FlowKey::new(
            Proto::Udp,
            addr("[2001:db8::10]:30000"),
            addr("[2001:db8::5]:5055"),
        );
        assert_eq!(i.resolve(&key, at(100)).expect("matches").pid, 9);
    }

    // The case `AttributionKey::local_matches_bind` names slice S10 as owner of.
    #[test]
    fn a_dual_stack_bind_matches_ipv4_traffic() {
        let i = index(vec![SocketTableEntry::udp(addr("[::]:30000"), 9)], &[]);
        assert_eq!(
            i.resolve(&udp_key(), at(100))
                .expect("dual stack matches")
                .pid,
            9
        );
    }

    #[test]
    fn a_specific_bind_beats_a_dual_stack_bind() {
        let i = index(
            vec![
                SocketTableEntry::udp(addr("[::]:30000"), 9),
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 7),
            ],
            &[],
        );
        assert_eq!(
            i.resolve(&udp_key(), at(100)).expect("matches").pid,
            7,
            "the exact local match outranks the dual-stack allowance"
        );
    }

    #[test]
    fn a_tcp_entry_does_not_take_the_wildcard_allowance() {
        let i = index(
            vec![SocketTableEntry::tcp(
                addr("0.0.0.0:51000"),
                addr("198.51.100.5:443"),
                9,
            )],
            &[],
        );
        assert_eq!(
            i.resolve(&tcp_key(), at(100)),
            None,
            "TCP carries both endpoints, so a looser rule would only add false positives"
        );
    }

    // --- FR-009: the creation instant filter -----------------------------

    #[test]
    fn a_socket_created_after_the_packet_does_not_match_it() {
        for proto in [Proto::Tcp, Proto::Udp] {
            let entry = match proto {
                Proto::Tcp => {
                    SocketTableEntry::tcp(addr("192.0.2.10:51000"), addr("198.51.100.5:443"), 1)
                }
                Proto::Udp => SocketTableEntry::udp(addr("192.0.2.10:30000"), 1),
            }
            .created_at(at(500));
            let key = match proto {
                Proto::Tcp => tcp_key(),
                Proto::Udp => udp_key(),
            };
            let i = index(vec![entry], &[]);
            assert!(i.resolve(&key, at(600)).is_some(), "{proto:?} after");
            assert_eq!(i.resolve(&key, at(400)), None, "{proto:?} before");
            assert!(
                i.resolve(&key, at(500)).is_some(),
                "{proto:?} at the instant of creation"
            );
        }
    }

    #[test]
    fn an_entry_with_no_creation_instant_still_matches() {
        // Absence of evidence is not evidence. Excluding these would
        // unattribute every socket on a platform that reports no creation time.
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1)],
            &[],
        );
        assert!(i.resolve(&udp_key(), at(0)).is_some());
        assert!(i.resolve(&udp_key(), at(i64::MAX)).is_some());
    }

    // --- FR-008, FR-008a, FR-008b: the total order -----------------------

    #[test]
    fn the_latest_socket_at_or_before_the_packet_wins() {
        // Port reuse. Three sockets have held this endpoint; the packet was
        // observed while the second one was current.
        let i = index(
            vec![
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 1).created_at(at(100)),
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 2).created_at(at(200)),
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 3).created_at(at(300)),
            ],
            &[],
        );
        assert_eq!(i.resolve(&udp_key(), at(250)).expect("matches").pid, 2);
        assert_eq!(i.resolve(&udp_key(), at(350)).expect("matches").pid, 3);
        assert_eq!(i.resolve(&udp_key(), at(150)).expect("matches").pid, 1);
    }

    #[test]
    fn an_entry_with_a_creation_instant_outranks_one_without() {
        let i = index(
            vec![
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 9),
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 5).created_at(at(10)),
            ],
            &[],
        );
        assert_eq!(i.resolve(&udp_key(), at(100)).expect("matches").pid, 5);
    }

    // SC-014. The test a first-hit matcher fails.
    #[test]
    fn permuting_the_table_changes_no_answer() {
        let entries = vec![
            SocketTableEntry::udp(addr("[::]:30000"), 40),
            SocketTableEntry::udp(addr("0.0.0.0:30000"), 30),
            SocketTableEntry::udp(addr("192.0.2.10:30000"), 20).created_at(at(50)),
            SocketTableEntry::udp(addr("192.0.2.10:30000"), 10).created_at(at(50)),
            SocketTableEntry::udp(addr("192.0.2.10:30000"), 25).created_at(at(60)),
        ];
        // Every rotation, and the reverse of each, which covers enough
        // orderings to catch any rule that depends on position.
        let expected = index(entries.clone(), &[])
            .resolve(&udp_key(), at(100))
            .expect("the table matches");
        assert_eq!(expected.pid, 25, "latest creation at or before the packet");

        for rotation in 0..entries.len() {
            let mut permuted = entries.clone();
            permuted.rotate_left(rotation);
            assert_eq!(
                index(permuted.clone(), &[]).resolve(&udp_key(), at(100)),
                Some(expected.clone()),
                "rotation {rotation}"
            );
            permuted.reverse();
            assert_eq!(
                index(permuted, &[]).resolve(&udp_key(), at(100)),
                Some(expected.clone()),
                "reversed rotation {rotation}"
            );
        }
    }

    #[test]
    fn a_residual_tie_is_broken_by_the_lower_identifier() {
        // Two entries indistinguishable under every meaningful rule. The
        // tiebreak decides nothing except that the answer is stable.
        let i = index(
            vec![
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 77).created_at(at(10)),
                SocketTableEntry::udp(addr("192.0.2.10:30000"), 12).created_at(at(10)),
            ],
            &[],
        );
        assert_eq!(i.resolve(&udp_key(), at(100)).expect("matches").pid, 12);
    }

    // --- FR-032: an attribution with no name -----------------------------

    #[test]
    fn an_unnamed_process_still_produces_an_attribution() {
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 4242)],
            &[],
        );
        let a = i.resolve(&udp_key(), at(100)).expect("matches");
        assert_eq!(a.pid, 4242, "the identifier is what was observed");
        assert_eq!(&*a.process, "");
        assert_eq!(a.fidelity, Fidelity::Live);
    }

    #[test]
    fn an_attribution_carries_no_role_or_stage_in_this_slice() {
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1)],
            &[(1, "g.exe")],
        );
        let a = i.resolve(&udp_key(), at(100)).expect("matches");
        assert!(a.role.is_none(), "roles arrive with S12");
        assert!(a.stage.is_none(), "stages arrive with S12");
    }

    // --- FR-018 through FR-022: retention and fidelity -------------------

    fn retained_index(r: RetainedEntry, endpoint: Endpoint) -> AttributionIndex {
        let mut map = RetentionMap::new();
        map.insert(endpoint, r);
        AttributionIndex::new(SocketTable::empty(at(0)), HashMap::new(), map, 30)
    }

    #[test]
    fn a_retained_endpoint_resolves_and_is_marked_retained() {
        let i = retained_index(
            RetainedEntry {
                pid: 5,
                created: None,
                remote: None,
                last_seen: at(100),
            },
            Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp),
        );
        let a = i.resolve(&udp_key(), at(110)).expect("inside the window");
        assert_eq!(a.pid, 5);
        assert_eq!(
            a.fidelity,
            Fidelity::Retained,
            "a retained answer must be visibly retained"
        );
    }

    #[test]
    fn a_retained_endpoint_expires() {
        let i = retained_index(
            RetainedEntry {
                pid: 5,
                created: None,
                remote: None,
                last_seen: at(100),
            },
            Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp),
        );
        assert!(i.resolve(&udp_key(), at(129)).is_some(), "just inside");
        assert_eq!(i.resolve(&udp_key(), at(130)), None, "at the boundary");
        assert_eq!(i.resolve(&udp_key(), at(500)), None, "well past");
    }

    #[test]
    fn a_live_entry_beats_a_retained_one() {
        let mut map = RetentionMap::new();
        map.insert(
            Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp),
            RetainedEntry {
                pid: 5,
                created: None,
                remote: None,
                last_seen: at(100),
            },
        );
        let i = AttributionIndex::new(
            SocketTable::new(
                at(110),
                vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 6)],
            ),
            names(&[(5, "old.exe"), (6, "new.exe")]),
            map,
            30_000_000_000,
        );
        let a = i.resolve(&udp_key(), at(110)).expect("matches");
        assert_eq!(a.pid, 6, "the table is evidence, retention is inference");
        assert_eq!(a.fidelity, Fidelity::Live);
    }

    #[test]
    fn a_retained_tcp_entry_still_has_to_agree_on_the_remote() {
        let mut map = RetentionMap::new();
        map.insert(
            Endpoint::new(addr("192.0.2.10:51000"), Proto::Tcp),
            RetainedEntry {
                pid: 5,
                created: None,
                remote: Some(addr("203.0.113.9:443")),
                last_seen: at(100),
            },
        );
        let i = AttributionIndex::new(SocketTable::empty(at(0)), HashMap::new(), map, 30);
        assert_eq!(
            i.resolve(&tcp_key(), at(110)),
            None,
            "a retained entry must not answer more broadly than the live path would"
        );
    }

    #[test]
    fn a_retained_socket_created_after_the_packet_still_does_not_match() {
        let i = retained_index(
            RetainedEntry {
                pid: 5,
                created: Some(at(200)),
                remote: None,
                last_seen: at(210),
            },
            Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp),
        );
        assert_eq!(i.resolve(&udp_key(), at(150)), None);
        assert!(i.resolve(&udp_key(), at(215)).is_some());
    }

    // --- FR-023, FR-014 --------------------------------------------------

    #[test]
    fn endpoints_reports_current_plus_retained() {
        let mut map = RetentionMap::new();
        map.insert(
            Endpoint::new(addr("192.0.2.99:1234"), Proto::Tcp),
            RetainedEntry {
                pid: 5,
                created: None,
                remote: None,
                last_seen: at(100),
            },
        );
        let i = AttributionIndex::new(
            SocketTable::new(
                at(100),
                vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 6)],
            ),
            HashMap::new(),
            map,
            30,
        );
        let mut got = i.endpoints(at(110));
        got.sort_by_key(|e| e.addr.port());
        assert_eq!(got.len(), 2);
        assert!(got.contains(&Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp)));
        assert!(got.contains(&Endpoint::new(addr("192.0.2.99:1234"), Proto::Tcp)));

        assert_eq!(
            i.endpoints(at(500)).len(),
            1,
            "an expired retained endpoint is no longer active"
        );
    }

    #[test]
    fn carries_distinguishes_an_unseen_endpoint_from_an_unattributable_flow() {
        let i = index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1).created_at(at(500))],
            &[],
        );
        // The endpoint is in the table, so a lookup that fails on the creation
        // filter is not an unseen endpoint and must trigger nothing.
        assert!(i.carries(&udp_key()));
        assert_eq!(i.resolve(&udp_key(), at(100)), None);

        let other = FlowKey::new(Proto::Udp, addr("192.0.2.10:40000"), addr("198.51.100.5:1"));
        assert!(!i.carries(&other), "nothing in the index covers this one");
    }

    #[test]
    fn an_empty_index_resolves_nothing_and_errors_at_nothing() {
        let i = AttributionIndex::default();
        assert_eq!(i.resolve(&udp_key(), at(100)), None);
        assert_eq!(i.resolve(&tcp_key(), at(100)), None);
        assert!(i.endpoints(at(100)).is_empty());
        assert!(!i.carries(&udp_key()));
    }

    #[test]
    fn a_retention_of_zero_makes_an_absent_endpoint_immediately_unresolvable() {
        let mut map = RetentionMap::new();
        map.insert(
            Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp),
            RetainedEntry {
                pid: 5,
                created: None,
                remote: None,
                last_seen: at(100),
            },
        );
        let i = AttributionIndex::new(SocketTable::empty(at(0)), HashMap::new(), map, 0);
        assert_eq!(i.resolve(&udp_key(), at(100)), None);
        assert_eq!(i.resolve(&udp_key(), at(101)), None);
    }

    // --- FR-027, FR-028: publication -------------------------------------

    #[test]
    fn a_loaded_index_survives_a_later_publication() {
        let published = PublishedIndex::new(index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1)],
            &[],
        ));
        let held = published.load();
        published.publish(index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 2)],
            &[],
        ));

        assert_eq!(
            held.resolve(&udp_key(), at(100)).expect("matches").pid,
            1,
            "a lookup in progress is unaffected by a publication"
        );
        assert_eq!(
            published
                .load()
                .resolve(&udp_key(), at(100))
                .expect("matches")
                .pid,
            2
        );
    }

    // SC-006. Readers resolving while a publisher alternates between two
    // indices whose answers are distinct.
    //
    // The assertion is a property that holds for every interleaving rather than
    // a specific one, and every thread runs a fixed bounded count rather than
    // until a flag flips. Both matter. The plan's risk note named this test as
    // the one that could be flaky, and the first version was: readers looped
    // "while not stopped", the publisher finished all two thousand publishes
    // before a reader thread was scheduled, and the reader observed nothing and
    // failed an assertion about having observed something. The fix is not to
    // relax the assertion but to remove the race from the harness. If the
    // threads happen not to overlap on some run, the test proves less that run
    // and never claims more.
    #[test]
    fn concurrent_resolution_across_a_publication_yields_only_whole_indices() {
        use std::thread;

        let published = Arc::new(PublishedIndex::new(index(
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1)],
            &[(1, "one.exe")],
        )));

        let readers: Vec<_> = (0..3)
            .map(|_| {
                let published = Arc::clone(&published);
                thread::spawn(move || {
                    for _ in 0..20_000 {
                        let a = published
                            .load()
                            .resolve(&udp_key(), at(100))
                            .expect("one of the two always matches");
                        // The property: identifier and name always agree,
                        // which they can only do if the whole index was
                        // observed. A torn read would pair one with the other.
                        let consistent = (a.pid == 1 && &*a.process == "one.exe")
                            || (a.pid == 2 && &*a.process == "two.exe");
                        assert!(consistent, "observed a mixture: {a:?}");
                    }
                })
            })
            .collect();

        for i in 0..20_000u32 {
            let pid = if i % 2 == 0 { 1 } else { 2 };
            let name = if pid == 1 { "one.exe" } else { "two.exe" };
            published.publish(index(
                vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), pid)],
                &[(pid, name)],
            ));
        }

        for r in readers {
            r.join().expect("a reader finishes");
        }
    }
}
