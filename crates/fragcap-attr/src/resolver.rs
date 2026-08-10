// SPDX-License-Identifier: Apache-2.0

//! The read-only half of the socket table attributor's read/write split.
//!
//! Specification section 11.6 divides the attributor into a write side that
//! reads a table and publishes an index, and a read side that does one atomic
//! load and a bounded scan over the value it published. [`PublishedResolver`] is
//! that read side, extracted so it can stand alone: it holds the shared
//! publication cell and the shared refresh schedule, resolves against them, and
//! owns no mutable state of its own.
//!
//! It was introduced because [`crate::socket::SocketTableAttributor::refresh`]
//! once took `&mut self`, so an attributor shared across the capture threads
//! could not be refreshed through the pointer they held, and a live capture kept
//! the mutable attributor on a separate control thread and handed the pipeline a
//! [`PublishedResolver`] cloned from it. Slice 015 changed `refresh` to `&self`,
//! so the pipeline can now share and refresh one attributor directly and this
//! split is no longer required; it is retained as a valid read-only view over a
//! publication (both sides read the same [`PublishedIndex`] and the same
//! [`RefreshSchedule`], so a refresh is visible to every resolving thread and an
//! unseen-endpoint lookup records a request the refreshing side acts on). Fully
//! removing the split is a separable cleanup.
//!
//! This type is platform neutral. It names no operating system interface, which
//! is why it lives here rather than behind the `socket-table` feature: the read
//! side of section 11.6 is arithmetic over a published value, and only the write
//! side touches a platform.

use std::sync::Arc;
use std::time::Duration;

use fragcap_core::attribution::Attribution;
use fragcap_core::error::AttrError;
use fragcap_core::flow::{Endpoint, FlowKey};
use fragcap_core::packet::Timestamp;
use fragcap_core::traits::FlowAttributor;

use crate::index::PublishedIndex;
use crate::schedule::RefreshSchedule;
use crate::seam::Clock;

/// A read-only [`FlowAttributor`] over a shared publication.
///
/// A faithful extract of [`crate::socket::SocketTableAttributor`]'s read side:
/// the same atomic load, the same lookup, and the same rate-limited request on
/// an unseen endpoint. It performs no table read and holds no mutable state, so
/// several of these can resolve concurrently against one publication that a
/// control thread republishes underneath them.
pub struct PublishedResolver {
    published: Arc<PublishedIndex>,
    schedule: Arc<RefreshSchedule>,
    clock: Arc<dyn Clock>,
    trigger_limit: Duration,
}

impl PublishedResolver {
    /// Build a resolver over a shared publication and schedule.
    ///
    /// The parts come from the owning attributor through
    /// [`crate::socket::SocketTableAttributor::resolver`]; this constructor is
    /// public so the split can be assembled from another crate.
    pub fn new(
        published: Arc<PublishedIndex>,
        schedule: Arc<RefreshSchedule>,
        clock: Arc<dyn Clock>,
        trigger_limit: Duration,
    ) -> Self {
        PublishedResolver {
            published,
            schedule,
            clock,
            trigger_limit,
        }
    }
}

impl FlowAttributor for PublishedResolver {
    /// Who owned this flow at the instant the packet was observed.
    ///
    /// One atomic load and a bounded scan over the published value, reading no
    /// socket table and taking no lock. On an unresolved lookup against an
    /// endpoint the index does not carry, it records a rate-limited refresh
    /// request on the shared schedule, which is exactly what the owning
    /// attributor's `resolve` does; the control thread that owns the mutable
    /// attributor acts on it.
    fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution> {
        let index = self.published.load();
        if let Some(a) = index.resolve(key, at) {
            return Some(a);
        }
        if !index.carries(key) {
            self.schedule
                .request_triggered(self.clock.now(), self.trigger_limit);
        }
        None
    }

    /// A no-op. The attributor this resolver was cloned from owns the socket
    /// table and the refresh that reads it; a resolver holds no source to
    /// refresh. The method exists to satisfy the trait.
    fn refresh(&self) -> Result<(), AttrError> {
        Ok(())
    }

    /// Every endpoint believed active, including the retention window, answered
    /// against the clock's instant exactly as the owning attributor answers it.
    fn active_endpoints(&self) -> Vec<Endpoint> {
        self.published.load().endpoints(self.clock.now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seam::{DeclaredNames, DeclaredTable, TestClock};
    use crate::socket::{AttributorConfig, SocketTableAttributor};
    use crate::table::{SocketTable, SocketTableEntry};
    use fragcap_core::attribution::Fidelity;
    use fragcap_core::flow::Proto;
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn at(n: i64) -> Timestamp {
        Timestamp::from_nanos(n)
    }

    fn udp_key() -> FlowKey {
        FlowKey::new(
            Proto::Udp,
            addr("192.0.2.10:30000"),
            addr("198.51.100.5:5055"),
        )
    }

    fn one_entry(pid: u32) -> SocketTable {
        SocketTable::new(
            at(0),
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), pid)],
        )
    }

    // The read/write split: a resolver cloned from an attributor answers from
    // the index the attributor publishes, and its own refresh is a harmless
    // no-op. This is the property the CLI's control thread relies on, that a
    // refresh on the owning attributor is visible to a resolver the pipeline
    // holds because both read one shared publication.
    #[test]
    fn a_resolver_answers_from_the_index_the_attributor_publishes() {
        let clock = Arc::new(TestClock::at(at(0)));
        let attributor = SocketTableAttributor::new(
            Box::new(DeclaredTable::once(one_entry(4242))),
            Box::new(DeclaredNames::from([(4242, "eso64.exe")])),
            Arc::clone(&clock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );
        let resolver = attributor.resolver();

        // Before any refresh the shared index is empty, so both resolve nothing.
        assert_eq!(resolver.resolve(&udp_key(), at(100)), None);

        // Refreshing the owning attributor publishes into the shared index the
        // resolver reads.
        attributor.refresh().expect("the declared table reads");

        let via_attributor = attributor
            .resolve(&udp_key(), at(100))
            .expect("the attributor resolves after its refresh");
        let via_resolver = resolver
            .resolve(&udp_key(), at(100))
            .expect("the resolver sees the same publication");
        assert_eq!(
            via_resolver, via_attributor,
            "the resolver answers exactly what the attributor does"
        );
        assert_eq!(via_resolver.pid, 4242);
        assert_eq!(&*via_resolver.process, "eso64.exe");
        assert_eq!(via_resolver.fidelity, Fidelity::Live);

        // The resolver's own refresh touches nothing: the answer is unchanged.
        resolver
            .refresh()
            .expect("the resolver refresh is a harmless no-op");
        assert_eq!(
            resolver
                .resolve(&udp_key(), at(100))
                .expect("still resolves after the no-op refresh")
                .pid,
            4242
        );

        // active_endpoints reads the same publication.
        assert_eq!(resolver.active_endpoints().len(), 1);
    }
}
