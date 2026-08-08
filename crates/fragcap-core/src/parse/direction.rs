// SPDX-License-Identifier: Apache-2.0

//! The locality rule from specification section 12.6, and the address set it
//! tests against.
//!
//! Section 12.6 defines three of the four combinations of endpoint locality
//! and is silent on the fourth. This module makes all four explicit, because
//! the silent one, a packet with no local endpoint at all, is the one a stale
//! address set produces on every packet.

use std::net::{IpAddr, SocketAddr};

use crate::flow::Direction;

use super::ParseReject;

/// The addresses belonging to the capturing host.
///
/// Supplied by the caller and never queried from the platform: enumerating
/// interfaces is platform work that constitution P-2 keeps out of this crate,
/// and it arrives in S09 and S13.
///
/// Replaced wholesale rather than mutated. There is deliberately no insert and
/// no remove, so a stale entry has no incremental path by which to survive a
/// refresh, which is what FR-032 asks for structurally rather than by
/// discipline.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InterfaceAddrs(Vec<IpAddr>);

impl InterfaceAddrs {
    /// Build a set. Allocates once, here, and never again.
    pub fn new(addrs: impl IntoIterator<Item = IpAddr>) -> Self {
        InterfaceAddrs(addrs.into_iter().collect())
    }

    /// Whether this address is on the capturing host.
    ///
    /// A linear scan, deliberately. A host has a handful of addresses, and at
    /// that size a scan beats hashing and needs no hasher. It allocates
    /// nothing, which is the property the parse path requires.
    pub fn contains(&self, addr: &IpAddr) -> bool {
        self.0.iter().any(|a| a == addr)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// The outcome of the locality rule: which endpoint is local, and which way
/// the packet travelled if that can be told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Locality {
    pub local: SocketAddr,
    pub remote: SocketAddr,
    pub direction: Option<Direction>,
    /// Both endpoints were local. The caller counts it; nothing else about the
    /// outcome distinguishes it from a resolved direction being absent.
    pub ambiguous: bool,
}

/// Apply specification section 12.6 to a wire-order endpoint pair.
///
/// Four outcomes, not three. A local source is outbound and a local
/// destination is inbound, as the section says. Both local is loopback, which
/// the section says is resolved later from the attributed process's endpoint,
/// so the direction is left undetermined here rather than guessed. Neither
/// local is the case the section does not cover, and it is a rejection: the
/// flow key's local field is defined as the endpoint on the capturing host,
/// and there is not one.
pub(crate) fn resolve(
    addrs: &InterfaceAddrs,
    src: SocketAddr,
    dst: SocketAddr,
) -> Result<Locality, ParseReject> {
    let src_local = addrs.contains(&src.ip());
    let dst_local = addrs.contains(&dst.ip());

    match (src_local, dst_local) {
        (true, false) => Ok(Locality {
            local: src,
            remote: dst,
            direction: Some(Direction::Outbound),
            ambiguous: false,
        }),
        (false, true) => Ok(Locality {
            local: dst,
            remote: src,
            direction: Some(Direction::Inbound),
            ambiguous: false,
        }),
        (true, true) => {
            // Both endpoints genuinely are local, so the only open question is
            // which one to write down, and an arbitrary total order answers it
            // without asserting anything untrue. What it buys is that both
            // halves of one loopback conversation produce one key. See plan
            // decision D-5.
            let (local, remote) = if order(src) <= order(dst) {
                (src, dst)
            } else {
                (dst, src)
            };
            Ok(Locality {
                local,
                remote,
                direction: None,
                ambiguous: true,
            })
        }
        (false, false) => Err(ParseReject::NoLocalEndpoint),
    }
}

/// The total order used to pick the local position when both endpoints are
/// local. Ordering across address families never fires in practice, because
/// both endpoints of a conversation share a family, but the order has to be
/// total and this one is.
fn order(addr: SocketAddr) -> (IpAddr, u16) {
    (addr.ip(), addr.port())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("test address must parse")
    }

    fn set(items: &[&str]) -> InterfaceAddrs {
        InterfaceAddrs::new(items.iter().map(|s| ip(s)))
    }

    // FR-028. The two cases section 12.6 states outright.
    #[test]
    fn a_local_source_is_outbound() {
        let r = resolve(
            &set(&["192.0.2.10"]),
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
        .expect("one endpoint is local");
        assert_eq!(r.direction, Some(Direction::Outbound));
        assert_eq!(r.local, addr("192.0.2.10:51000"));
        assert_eq!(r.remote, addr("198.51.100.5:443"));
        assert!(!r.ambiguous);
    }

    #[test]
    fn a_local_destination_is_inbound_and_swaps_the_positions() {
        let r = resolve(
            &set(&["192.0.2.10"]),
            addr("198.51.100.5:443"),
            addr("192.0.2.10:51000"),
        )
        .expect("one endpoint is local");
        assert_eq!(r.direction, Some(Direction::Inbound));
        assert_eq!(
            r.local,
            addr("192.0.2.10:51000"),
            "local is the capturing host's endpoint, not the wire source"
        );
        assert_eq!(r.remote, addr("198.51.100.5:443"));
    }

    // FR-029. Loopback, where the rule returns both answers.
    #[test]
    fn both_local_is_ambiguous_and_yields_no_direction() {
        let r = resolve(
            &set(&["127.0.0.1"]),
            addr("127.0.0.1:51000"),
            addr("127.0.0.1:8080"),
        )
        .expect("a key is still produced");
        assert_eq!(r.direction, None, "a guess here is right half the time");
        assert!(r.ambiguous);
    }

    #[test]
    fn both_halves_of_a_loopback_conversation_agree_on_the_positions() {
        let s = set(&["127.0.0.1"]);
        let a = resolve(&s, addr("127.0.0.1:51000"), addr("127.0.0.1:8080"))
            .expect("a key is produced");
        let b = resolve(&s, addr("127.0.0.1:8080"), addr("127.0.0.1:51000"))
            .expect("a key is produced");
        assert_eq!(a.local, b.local);
        assert_eq!(a.remote, b.remote);
    }

    #[test]
    fn the_ordering_rule_is_total_across_families() {
        let s = set(&["192.0.2.10", "2001:db8::10"]);
        let a =
            resolve(&s, addr("192.0.2.10:1"), addr("[2001:db8::10]:2")).expect("both are local");
        let b =
            resolve(&s, addr("[2001:db8::10]:2"), addr("192.0.2.10:1")).expect("both are local");
        assert_eq!(a.local, b.local);
    }

    // FR-030. The case section 12.6 does not cover.
    #[test]
    fn neither_local_is_a_rejection_rather_than_a_fabricated_key() {
        let r = resolve(
            &set(&["192.0.2.10"]),
            addr("198.51.100.5:443"),
            addr("203.0.113.7:51000"),
        );
        assert_eq!(r, Err(ParseReject::NoLocalEndpoint));
    }

    #[test]
    fn an_empty_set_rejects_everything_and_says_which_cause() {
        let r = resolve(
            &InterfaceAddrs::default(),
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        );
        assert_eq!(
            r,
            Err(ParseReject::NoLocalEndpoint),
            "a stale or empty address set must announce itself on every packet"
        );
    }

    #[test]
    fn identical_endpoints_are_the_loopback_case() {
        let r = resolve(
            &set(&["127.0.0.1"]),
            addr("127.0.0.1:9000"),
            addr("127.0.0.1:9000"),
        )
        .expect("both are local");
        assert!(r.ambiguous);
        assert_eq!(r.direction, None);
    }

    #[test]
    fn the_set_reports_its_own_shape() {
        assert!(InterfaceAddrs::default().is_empty());
        assert_eq!(set(&["192.0.2.10", "2001:db8::10"]).len(), 2);
        assert!(set(&["192.0.2.10"]).contains(&ip("192.0.2.10")));
        assert!(!set(&["192.0.2.10"]).contains(&ip("192.0.2.11")));
    }
}
