// SPDX-License-Identifier: Apache-2.0

//! Flow identity: protocols, endpoints, flow keys, and the socket table
//! matching key derived from them.
//!
//! The asymmetry between TCP and UDP in [`AttributionKey`] is the load-bearing
//! part of this module. It is a property of the platform interface, not a
//! fragcap design choice, and specification section 8.4 requires that
//! implementations not paper over it.

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::attribution::Attribution;
use crate::packet::Timestamp;

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

/// An active [`Endpoint`] paired with the process identifier that owns it, when
/// the source can supply one.
///
/// Returned by [`crate::traits::FlowAttributor::active_endpoints_owned`]. It
/// exists because [`crate::traits::FlowAttributor::active_endpoints`] reports
/// only the endpoint, having dropped the owner the socket table carried, and the
/// phase-two narrowing of specification section 12.2 admits only endpoints
/// belonging to profiled processes, which is a decision the owning identifier is
/// needed to make. Kept to endpoint plus owner deliberately: the narrowing needs
/// no name and no role, only which process a socket belongs to.
///
/// `owner` is `None` for a source that does not track ownership (the scripted
/// attributor, the stubs), which is why the trait's default implementation maps
/// every endpoint to an unowned one: a consumer that does not filter by owner
/// then sees exactly the endpoints it saw before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedEndpoint {
    pub endpoint: Endpoint,
    pub owner: Option<u32>,
}

impl OwnedEndpoint {
    pub fn new(endpoint: Endpoint, owner: Option<u32>) -> Self {
        OwnedEndpoint { endpoint, owner }
    }

    /// An endpoint whose owner is not known, for sources that do not track it.
    pub fn unowned(endpoint: Endpoint) -> Self {
        OwnedEndpoint {
            endpoint,
            owner: None,
        }
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

/// A stable, session-local identity assigned to one canonical flow key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlowId(u64);

impl FlowId {
    /// Construct a nonzero flow ordinal.
    pub fn new(ordinal: u64) -> Option<Self> {
        (ordinal != 0).then_some(Self(ordinal))
    }

    /// The numeric session-local ordinal.
    pub fn ordinal(self) -> u64 {
        self.0
    }
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "flow-{:08}", self.0)
    }
}

#[derive(Debug)]
struct FlowRegistryState {
    next: u64,
    flows: HashMap<FlowKey, RegisteredFlow>,
    retained_observations: usize,
    observation_limit: usize,
}

#[derive(Clone, Debug)]
struct RegisteredFlow {
    id: FlowId,
    attribution: Option<Attribution>,
    observations: Vec<FlowObservation>,
    unretained_observations: u64,
}

/// Packet-side evidence retained for deterministic Deep Capture correlation.
#[derive(Clone, Debug)]
pub struct FlowObservation {
    pub timestamp: Timestamp,
    pub attribution: Option<Attribution>,
}

/// Immutable history for one canonical flow.
#[derive(Clone, Debug)]
pub struct FlowSummary {
    pub id: FlowId,
    pub observations: Vec<FlowObservation>,
    pub unretained_observations: u64,
    pub global_unretained_observations: u64,
}

/// The capture-wide mapping from canonical flow keys to session-local ids.
///
/// Ordinary assignment happens on the pipeline's single output thread, after
/// the write gate admits a packet. Buffer evictions and gate rejections record
/// conservative unretained markers, so withheld evidence cannot later support
/// a confident Deep Capture correlation.
#[derive(Debug)]
pub struct FlowRegistry {
    state: Mutex<FlowRegistryState>,
    globally_unretained: AtomicU64,
}

impl Default for FlowRegistry {
    fn default() -> Self {
        Self::with_history_limit(262_144)
    }
}

impl FlowRegistry {
    /// Construct a registry with a finite capture-wide correlation history.
    pub fn with_history_limit(observation_limit: usize) -> Self {
        Self {
            state: Mutex::new(FlowRegistryState {
                next: 1,
                flows: HashMap::new(),
                retained_observations: 0,
                observation_limit: observation_limit.max(1),
            }),
            globally_unretained: AtomicU64::new(0),
        }
    }
    /// Return the existing id or assign the next session-local id.
    pub fn assign(&self, key: FlowKey) -> FlowId {
        self.observe(key, None)
    }

    /// Record a flow and its latest resolved attribution, returning its id.
    pub fn observe(&self, key: FlowKey, attribution: Option<&Attribution>) -> FlowId {
        self.observe_at(key, Timestamp::from_nanos(0), attribution)
    }

    /// Record timestamped packet evidence for later interval reconciliation.
    pub fn observe_at(
        &self,
        key: FlowKey,
        timestamp: Timestamp,
        attribution: Option<&Attribution>,
    ) -> FlowId {
        let mut state = self.state.lock().expect("flow registry mutex poisoned");
        let retain = state.retained_observations < state.observation_limit;
        if let Some(flow) = state.flows.get_mut(&key) {
            if let Some(attribution) = attribution {
                flow.attribution = Some(attribution.clone());
            }
            let id = flow.id;
            if retain {
                flow.observations.push(FlowObservation {
                    timestamp,
                    attribution: attribution.cloned(),
                });
            } else {
                flow.unretained_observations = flow.unretained_observations.saturating_add(1);
            }
            if retain {
                state.retained_observations += 1;
            }
            return id;
        }
        let id = FlowId::new(state.next).expect("the flow id counter starts at one");
        state.next = state
            .next
            .checked_add(1)
            .expect("the session-local flow id space is exhausted");
        state.flows.insert(
            key,
            RegisteredFlow {
                id,
                attribution: attribution.cloned(),
                observations: retain
                    .then(|| FlowObservation {
                        timestamp,
                        attribution: attribution.cloned(),
                    })
                    .into_iter()
                    .collect(),
                unretained_observations: u64::from(!retain),
            },
        );
        if retain {
            state.retained_observations += 1;
        }
        id
    }

    /// Record that a packet for this flow was observed but evicted before its
    /// full correlation evidence could be retained.
    pub fn mark_unretained(&self, key: FlowKey) -> FlowId {
        let mut state = self.state.lock().expect("flow registry mutex poisoned");
        if let Some(flow) = state.flows.get_mut(&key) {
            flow.unretained_observations = flow.unretained_observations.saturating_add(1);
            return flow.id;
        }
        let id = FlowId::new(state.next).expect("the flow id counter starts at one");
        state.next = state
            .next
            .checked_add(1)
            .expect("the session-local flow id space is exhausted");
        state.flows.insert(
            key,
            RegisteredFlow {
                id,
                attribution: None,
                observations: Vec::new(),
                unretained_observations: 1,
            },
        );
        id
    }

    /// Conservatively record an eviction without taking the registry mutex.
    /// This is used only on acquisition threads, where blocking behind output
    /// correlation bookkeeping would amplify capture loss.
    pub fn mark_globally_unretained(&self) {
        self.globally_unretained.fetch_add(1, Ordering::Relaxed);
    }

    pub fn globally_unretained(&self) -> u64 {
        self.globally_unretained.load(Ordering::Acquire)
    }

    /// Look up an id without creating one.
    pub fn lookup(&self, key: &FlowKey) -> Option<FlowId> {
        self.state
            .lock()
            .expect("flow registry mutex poisoned")
            .flows
            .get(key)
            .map(|flow| flow.id)
    }

    /// Return the latest resolved attribution observed for a registered flow.
    pub fn attribution(&self, key: &FlowKey) -> Option<Attribution> {
        self.state
            .lock()
            .expect("flow registry mutex poisoned")
            .flows
            .get(key)
            .and_then(|flow| flow.attribution.clone())
    }

    /// Snapshot all packet observations for one canonical flow.
    pub fn summary(&self, key: &FlowKey) -> Option<FlowSummary> {
        self.state
            .lock()
            .expect("flow registry mutex poisoned")
            .flows
            .get(key)
            .map(|flow| FlowSummary {
                id: flow.id,
                observations: flow.observations.clone(),
                unretained_observations: flow.unretained_observations,
                global_unretained_observations: self.globally_unretained(),
            })
    }
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
    fn flow_registry_reuses_ids_and_assigns_distinct_ids_in_order() {
        let registry = FlowRegistry::default();
        let first = registry.assign(tcp());
        let repeated = registry.assign(tcp());
        let second = registry.assign(udp());

        assert_eq!(first, repeated);
        assert_eq!(first.to_string(), "flow-00000001");
        assert_eq!(second.to_string(), "flow-00000002");
        assert_eq!(registry.lookup(&tcp()), Some(first));
    }

    #[test]
    fn flow_registry_retains_the_latest_resolved_attribution() {
        use crate::attribution::Fidelity;

        let registry = FlowRegistry::default();
        registry.assign(tcp());
        assert!(registry.attribution(&tcp()).is_none());

        let attribution = Attribution::new(7, "client.exe", Fidelity::Live).with_role("client");
        registry.observe(tcp(), Some(&attribution));
        assert_eq!(registry.attribution(&tcp()), Some(attribution));
    }

    #[test]
    fn flow_registry_preserves_timestamped_owner_history() {
        use crate::attribution::Fidelity;

        let registry = FlowRegistry::default();
        let first = Attribution::new(7, "first.exe", Fidelity::Live);
        let second = Attribution::new(8, "second.exe", Fidelity::Retained);
        registry.observe_at(tcp(), Timestamp::from_nanos(10), Some(&first));
        registry.observe_at(tcp(), Timestamp::from_nanos(20), Some(&second));

        let summary = registry.summary(&tcp()).unwrap();
        assert_eq!(summary.observations.len(), 2);
        assert_eq!(summary.observations[0].timestamp.as_nanos(), 10);
        assert_eq!(summary.observations[0].attribution, Some(first));
        assert_eq!(summary.observations[1].timestamp.as_nanos(), 20);
        assert_eq!(summary.observations[1].attribution, Some(second));
    }

    #[test]
    fn flow_registry_bounds_history_and_counts_every_unretained_observation() {
        let registry = FlowRegistry::with_history_limit(1);
        registry.observe_at(tcp(), Timestamp::from_nanos(10), None);
        registry.observe_at(tcp(), Timestamp::from_nanos(20), None);
        registry.observe_at(tcp(), Timestamp::from_nanos(30), None);

        let summary = registry.summary(&tcp()).unwrap();
        assert_eq!(summary.observations.len(), 1);
        assert_eq!(summary.unretained_observations, 2);
    }

    #[test]
    fn global_loss_is_not_multiplied_into_each_flow_count() {
        let registry = FlowRegistry::default();
        registry.observe_at(tcp(), Timestamp::from_nanos(10), None);
        registry.mark_globally_unretained();

        let summary = registry.summary(&tcp()).unwrap();
        assert_eq!(summary.unretained_observations, 0);
        assert_eq!(summary.global_unretained_observations, 1);
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

    #[test]
    fn an_owned_endpoint_carries_its_owner_and_an_unowned_one_does_not() {
        let e = Endpoint::new(addr("192.0.2.10:30000"), Proto::Udp);
        let owned = OwnedEndpoint::new(e, Some(4242));
        assert_eq!(owned.endpoint, e);
        assert_eq!(owned.owner, Some(4242));

        let unowned = OwnedEndpoint::unowned(e);
        assert_eq!(unowned.endpoint, e);
        assert_eq!(unowned.owner, None, "an unowned endpoint names no process");
    }
}
