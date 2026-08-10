// SPDX-License-Identifier: Apache-2.0

//! Flow identity: protocols, endpoints, flow keys, and the socket table
//! matching key derived from them.
//!
//! The asymmetry between TCP and UDP in [`AttributionKey`] is the load-bearing
//! part of this module. It is a property of the platform interface, not a
//! fragcap design choice, and specification section 8.4 requires that
//! implementations not paper over it.

use std::net::SocketAddr;

/// Transport protocol of a flow.
///
/// Exactly two variants, and deliberately closed rather than
/// `#[non_exhaustive]`: the socket table join in specification section 11 is
/// defined for TCP and UDP only, so a third variant would have no meaning to
/// the attributor and would silently widen every match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Proto {
    Tcp,
    Udp,
}

/// Which way an individual packet travelled.
///
/// A property of the packet rather than of the flow. [`FlowKey`] already
/// normalized endpoint position, so direction carries no redundant information
/// and cannot disagree with the key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Inbound,
    Outbound,
}

/// An address, port, and protocol on some host.
///
/// Returned in bulk by [`crate::traits::FlowAttributor::active_endpoints`].
/// Endpoint retention after a socket leaves the table is specification section
/// 11.4 and belongs to slice S10; this type is only the shape.
///
/// `Ord` so a set of endpoints keys a `BTreeSet`, which slice S13 uses to
/// compile a filter deterministically and to count filter gaps by set
/// difference. The order is arbitrary but total; nothing reads meaning into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Endpoint {
    pub addr: SocketAddr,
    pub proto: Proto,
}

impl Endpoint {
    pub fn new(addr: SocketAddr, proto: Proto) -> Self {
        Endpoint { addr, proto }
    }
}

/// The identity of one conversation.
///
/// `local` is always the endpoint on the capturing host. That normalization is
/// what makes a single flow one key rather than two, and it is why
/// [`Direction`] is an independent per-packet property. Deriving the local
/// position is the header parser's job in slice S03, by matching against the
/// interface address set; this type assumes it has been done.
///
/// Equality and hashing are part of the contract, not a convenience: this type
/// is the lookup key into the attribution index on the capture thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlowKey {
    pub proto: Proto,
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

impl FlowKey {
    pub fn new(proto: Proto, local: SocketAddr, remote: SocketAddr) -> Self {
        FlowKey {
            proto,
            local,
            remote,
        }
    }

    /// The subset of this key that can be matched against a socket table entry.
    ///
    /// TCP matches on both endpoints, because the TCP socket table carries
    /// both. UDP matches on the local endpoint alone, because a UDP socket
    /// generally has no fixed peer and the table carries no remote for it.
    ///
    /// This is specification section 8.4. The asymmetry is a property of the
    /// platform interface and holds on every backend in section 9.4.
    pub fn attribution_key(&self) -> AttributionKey {
        match self.proto {
            Proto::Tcp => AttributionKey::Pair(self.local, self.remote),
            Proto::Udp => AttributionKey::Local(self.local),
        }
    }
}

/// The part of a [`FlowKey`] a socket table can actually answer.
///
/// There is deliberately no variant carrying a remote endpoint for UDP. That
/// absence is the enforcement of specification section 8.4's requirement that
/// implementations MUST NOT invent a remote endpoint for a UDP entry, because
/// doing so produces confident wrong attributions rather than honest coarse
/// ones. The rule is structural here rather than a comment someone has to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributionKey {
    /// Both endpoints, from the TCP socket table.
    Pair(SocketAddr, SocketAddr),
    /// The local endpoint only, from the UDP socket table.
    Local(SocketAddr),
}

impl AttributionKey {
    /// The local endpoint, which both variants carry.
    pub fn local(&self) -> SocketAddr {
        match self {
            AttributionKey::Pair(local, _) => *local,
            AttributionKey::Local(local) => *local,
        }
    }

    /// Whether a socket table entry bound to `bind` could own this key.
    ///
    /// A UDP socket bound to a wildcard address matches a datagram observed on
    /// a specific interface address, because the table reports the bind address
    /// rather than the address a datagram arrived on. Specification section 8.4
    /// requires that both be matchable.
    ///
    /// The wildcard allowance applies to UDP only. A TCP entry carries both
    /// endpoints, so an exact local match is available and a looser rule would
    /// only add false positives.
    ///
    /// Dual-stack sockets, where an IPv6 wildcard accepts IPv4 traffic, are not
    /// handled here. Slice S10 owns that, and Appendix D found no focal title
    /// relying on it.
    pub fn local_matches_bind(&self, bind: SocketAddr) -> bool {
        match self {
            AttributionKey::Pair(local, _) => *local == bind,
            AttributionKey::Local(local) => {
                *local == bind || (bind.ip().is_unspecified() && bind.port() == local.port())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn tcp() -> FlowKey {
        FlowKey::new(
            Proto::Tcp,
            addr("192.0.2.10:51000"),
            addr("198.51.100.5:443"),
        )
    }

    fn udp() -> FlowKey {
        FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        )
    }

    // V-3. The whole point of the type. Asserted for each protocol separately,
    // because a single test over both would pass if the match arms were swapped
    // and the assertion were written loosely.
    #[test]
    fn tcp_resolves_on_both_endpoints() {
        let key = tcp();
        assert_eq!(
            key.attribution_key(),
            AttributionKey::Pair(key.local, key.remote)
        );
    }

    #[test]
    fn udp_resolves_on_the_local_endpoint_alone() {
        let key = udp();
        assert_eq!(key.attribution_key(), AttributionKey::Local(key.local));
    }

    #[test]
    fn a_udp_key_never_carries_a_remote() {
        // If a Pair variant is ever produced for UDP, the asymmetry has been
        // papered over and confident wrong attributions become possible.
        match udp().attribution_key() {
            AttributionKey::Local(_) => {}
            AttributionKey::Pair(_, _) => {
                panic!("UDP produced a Pair key, which specification 8.4 forbids")
            }
        }
    }

    // V-1. Both key types are map keys by contract.
    #[test]
    fn flow_keys_work_as_map_keys() {
        let mut seen: HashMap<FlowKey, u32> = HashMap::new();
        seen.insert(tcp(), 1);
        *seen.entry(tcp()).or_insert(0) += 1;
        assert_eq!(seen.len(), 1, "equal keys must collide in a map");
        assert_eq!(seen[&tcp()], 2);
    }

    #[test]
    fn attribution_keys_work_as_map_keys() {
        let mut seen: HashMap<AttributionKey, u32> = HashMap::new();
        seen.insert(udp().attribution_key(), 1);
        seen.insert(tcp().attribution_key(), 2);
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[&udp().attribution_key()], 1);
    }

    #[test]
    fn keys_differing_only_by_protocol_are_distinct() {
        let a = FlowKey::new(Proto::Tcp, addr("192.0.2.1:1"), addr("192.0.2.2:2"));
        let b = FlowKey::new(Proto::Udp, addr("192.0.2.1:1"), addr("192.0.2.2:2"));
        assert_ne!(a, b, "proto must participate in equality");
    }

    // V-10. Wildcard bind matching, both address families.
    #[test]
    fn udp_wildcard_bind_matches_a_specific_interface_address() {
        let key = AttributionKey::Local(addr("192.0.2.10:30000"));
        assert!(key.local_matches_bind(addr("0.0.0.0:30000")));
        assert!(key.local_matches_bind(addr("192.0.2.10:30000")));
    }

    #[test]
    fn udp_wildcard_bind_still_requires_the_port_to_match() {
        let key = AttributionKey::Local(addr("192.0.2.10:30000"));
        assert!(!key.local_matches_bind(addr("0.0.0.0:30001")));
    }

    #[test]
    fn udp_ipv6_wildcard_bind_matches() {
        let key = AttributionKey::Local(addr("[2001:db8::10]:30000"));
        assert!(key.local_matches_bind(addr("[::]:30000")));
    }

    #[test]
    fn tcp_bind_matching_does_not_take_the_wildcard_allowance() {
        // TCP carries both endpoints, so the looser rule would only add false
        // positives.
        let key = AttributionKey::Pair(addr("192.0.2.10:51000"), addr("198.51.100.5:443"));
        assert!(!key.local_matches_bind(addr("0.0.0.0:51000")));
        assert!(key.local_matches_bind(addr("192.0.2.10:51000")));
    }

    #[test]
    fn local_is_available_from_either_variant() {
        assert_eq!(tcp().attribution_key().local(), tcp().local);
        assert_eq!(udp().attribution_key().local(), udp().local);
    }
}
