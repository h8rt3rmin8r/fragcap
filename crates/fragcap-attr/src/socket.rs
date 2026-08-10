// SPDX-License-Identifier: Apache-2.0

//! The production [`FlowAttributor`]. Specification section 11.
//!
//! This is the first attributor in the project that can be wrong, and the shape
//! of the type follows from that.
//!
//! [`SocketTableAttributor::refresh`] does everything expensive and everything
//! fallible: it reads a table, ages the retention map against it, resolves image
//! names for the identifiers the table reported, builds an immutable index, and
//! publishes it. [`SocketTableAttributor::resolve`] does one atomic load and a
//! bounded scan over a value, and calls into no operating system interface at
//! all.
//!
//! That division is specification section 11.6, and the reason it is worth the
//! ceremony is section 11.2's measurement: a snapshot through the table
//! interface costs one to three milliseconds, and through the object-model
//! projection of the same data it costs 1400 to 2000. An implementation that
//! put either on the acquisition path would be visible as a periodic stall in
//! packet capture, and the second would make polling look unworkable when it is
//! not.
//!
//! # What this slice does not know
//!
//! Stage two of the section 11.1 lookup is the process tree, which is slice
//! S11. This attributor resolves the process identifier from the table and
//! takes the image name from [`ProcessNamer`], so S11 replaces a default rather
//! than restructures an attributor. Role and stage stay absent; S12 fills them.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use fragcap_core::attribution::Attribution;
use fragcap_core::error::AttrError;
use fragcap_core::flow::{Endpoint, FlowKey};
use fragcap_core::packet::Timestamp;
use fragcap_core::traits::FlowAttributor;

use crate::index::{AttributionIndex, PublishedIndex, RetainedEntry, RetainedKey, RetentionMap};
use crate::resolver::PublishedResolver;
use crate::schedule::RefreshSchedule;
use crate::seam::{Clock, ProcessNamer, SocketTableSource};

/// The cadence and retention settings of specification sections 11.2 and 11.4.
///
/// Plain values on this struct and not keys in a game profile. `fragcap-profile`
/// accepts a closed set of five capture keys and refuses unknown ones, and it
/// refuses them deliberately: a key with no consumer is a key whose behavior is
/// untested and whose meaning is set by whoever first reads it. S14 owns adding
/// keys when it owns a command line that can set them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttributorConfig {
    /// How often the socket table is re-read. Section 11.2.
    pub interval: Duration,
    /// How long an endpoint stays resolvable after leaving the table.
    /// Section 11.4.
    pub retention: Duration,
    /// The floor between two unseen-endpoint refresh requests. Section 11.2.
    pub trigger_limit: Duration,
}

impl Default for AttributorConfig {
    fn default() -> Self {
        AttributorConfig {
            interval: Duration::from_secs(1),
            retention: Duration::from_secs(30),
            trigger_limit: Duration::from_millis(200),
        }
    }
}

/// Attribution against the operating system socket table.
///
/// Construction takes every seam explicitly and defaults none of them. That is
/// the same argument [`Attribution::new`] makes about fidelity: a default is an
/// inference, and an attributor that silently defaulted its clock to the system
/// clock would let a test that meant to control time pass while measuring
/// nothing.
pub struct SocketTableAttributor {
    source: Box<dyn SocketTableSource>,
    namer: Box<dyn ProcessNamer>,
    clock: Arc<dyn Clock>,
    config: AttributorConfig,
    published: Arc<PublishedIndex>,
    schedule: Arc<RefreshSchedule>,
    /// The retention map carried between refreshes. Owned by the refreshing
    /// side; the published index gets a copy.
    retained: RetentionMap,
}

impl SocketTableAttributor {
    pub fn new(
        source: Box<dyn SocketTableSource>,
        namer: Box<dyn ProcessNamer>,
        clock: Arc<dyn Clock>,
        config: AttributorConfig,
    ) -> Self {
        let retention = nanos(config.retention);
        SocketTableAttributor {
            source,
            namer,
            clock,
            config,
            published: Arc::new(PublishedIndex::new(AttributionIndex::new(
                crate::table::SocketTable::default(),
                HashMap::new(),
                RetentionMap::new(),
                retention,
            ))),
            schedule: Arc::new(RefreshSchedule::new()),
            retained: RetentionMap::new(),
        }
    }

    /// The publication cell. Shareable, and the seam slice S13's control thread
    /// attaches to.
    pub fn published(&self) -> Arc<PublishedIndex> {
        Arc::clone(&self.published)
    }

    /// The refresh cadence and its triggers. Shareable, because the
    /// unseen-endpoint trigger is recorded on a capture thread.
    pub fn schedule(&self) -> Arc<RefreshSchedule> {
        Arc::clone(&self.schedule)
    }

    pub fn config(&self) -> &AttributorConfig {
        &self.config
    }

    /// A read-only resolver over this attributor's shared publication.
    ///
    /// The read side of the section 11.6 split. The returned
    /// [`PublishedResolver`] shares this attributor's publication cell, its
    /// refresh schedule, and its clock, so a refresh on this attributor is
    /// visible to the resolver and an unseen-endpoint lookup on the resolver
    /// records a request this attributor's owner acts on. It is what the capture
    /// pipeline resolves against while a control thread keeps the mutable
    /// attributor and refreshes it.
    pub fn resolver(&self) -> PublishedResolver {
        PublishedResolver::new(
            Arc::clone(&self.published),
            Arc::clone(&self.schedule),
            Arc::clone(&self.clock),
            self.config.trigger_limit,
        )
    }

    /// Whether a refresh is due by the interval, or has been requested by
    /// either trigger. Specification section 11.2.
    pub fn wants_refresh(&self) -> bool {
        let now = self.clock.now();
        self.schedule.is_due(now, self.config.interval) || self.schedule.is_requested()
    }

    /// Ask for a refresh because a process matching a profile stage started.
    ///
    /// Not rate limited, per section 11.2. Slice S11 calls this when it has
    /// process events to call it with; until then it exists so that the
    /// trigger is implemented and tested rather than described.
    pub fn note_matched_process_start(&self) {
        self.schedule.request_immediate();
    }
}

impl FlowAttributor for SocketTableAttributor {
    /// Who owned this flow at the instant the packet was observed.
    ///
    /// One atomic load and a bounded scan over an immutable value. This path
    /// reads no socket table, enumerates no process, and opens no handle, which
    /// is requirement FR-017 and what makes section 11.6's promise real.
    ///
    /// It does read the injected clock, and only when the lookup failed on an
    /// endpoint the index does not carry, to rate limit the refresh request
    /// that failure triggers. The limit bounds how often fragcap reads the
    /// platform's table, which is a wall-clock cost, so it cannot be measured
    /// in capture time.
    fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution> {
        let index = self.published.load();
        if let Some(a) = index.resolve(key, at) {
            return Some(a);
        }
        // Unresolved. Section 11.2: an unattributed packet on a previously
        // unseen endpoint triggers a snapshot, rate limited. An endpoint the
        // index already carries is not unseen, and a flow fragcap will never
        // attribute must not be able to drive the table read rate.
        if !index.carries(key) {
            self.schedule
                .request_triggered(self.clock.now(), self.config.trigger_limit);
        }
        None
    }

    /// Re-read the table and publish a new index.
    ///
    /// On failure the previously published index stays exactly as it was.
    /// Replacing a good snapshot with an empty one on a transient failure would
    /// silently unattribute every packet after it, which is the configuration
    /// side of the loss constitution principle P-4 forbids: every packet lost,
    /// none counted.
    fn refresh(&mut self) -> Result<(), AttrError> {
        let table = self.source.read()?;
        let taken_at = table.taken_at();
        let retention = nanos(self.config.retention);

        // Names for the identifiers this table reports, and only those.
        //
        // Retained records are deliberately not re-queried. Their process may
        // have exited, in which case an enumeration returns nothing and a name
        // once known would be lost; or the platform may have reused the
        // identifier, in which case an enumeration returns a different
        // process's name and attaching it here would report a connection as
        // belonging to something that never opened it. Each record keeps the
        // name it was captured with instead. Found by review of pull request
        // 13.
        let mut pids: Vec<u32> = table.entries().iter().map(|e| e.pid).collect();
        pids.sort_unstable();
        pids.dedup();
        let names = self.namer.names(&pids);

        // Age the retention map against the new table, before adding to it, so
        // that a socket present in both is refreshed rather than expired.
        self.retained
            .retain(|_, r| taken_at.nanos_since(r.last_seen) < retention);

        // Every row the table reports is present now, so it either enters the
        // retention map or has its last-seen instant renewed. Renewing rather
        // than removing is what makes FR-018a's origin correct: the grace
        // period runs from the last instant the socket was observed present,
        // not from the refresh that first noticed it gone.
        for entry in table.entries() {
            let key = RetainedKey::of(entry);
            // A name resolved now wins; failing that, whatever this socket was
            // already known by is kept rather than dropped.
            let name = names
                .get(&entry.pid)
                .cloned()
                .or_else(|| self.retained.get(&key).and_then(|r| r.name.clone()));
            self.retained.insert(
                key,
                RetainedEntry {
                    entry: *entry,
                    last_seen: taken_at,
                    name,
                },
            );
        }

        // Marked before the index is published, not after, and the order is
        // load-bearing. Publishing first opens a window in which a capture
        // thread reads the new index, finds an endpoint it still does not
        // carry, and records a request that this call would then erase; because
        // recording it also consumed the rate-limit window, nothing could
        // re-arm the request for the next two hundred milliseconds and a
        // short-lived flow would stay unattributed until the next periodic
        // refresh. Marking first can instead leave a request that this
        // publication happens to satisfy, which costs one extra table read.
        // An extra read is cheap; a missed one loses attribution. Found by
        // review of pull request 13.
        self.schedule.mark_refreshed(self.clock.now());
        self.published.publish(AttributionIndex::new(
            table,
            names,
            self.retained.clone(),
            retention,
        ));
        Ok(())
    }

    /// Every endpoint believed active, including the retention window.
    ///
    /// Answered against the clock's instant, because the trait method carries
    /// none and because this is the one question here that is genuinely about
    /// now. `resolve` is always a question about then, and conflating the two
    /// is what specification section 11.4 warns against.
    fn active_endpoints(&self) -> Vec<Endpoint> {
        self.published.load().endpoints(self.clock.now())
    }
}

impl std::fmt::Debug for SocketTableAttributor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The seams are trait objects and none of them is `Debug`. Report the
        // configuration and the size of what is published, which is what a
        // reader of a debug line actually wants.
        let index = self.published.load();
        f.debug_struct("SocketTableAttributor")
            .field("config", &self.config)
            .field("table_entries", &index.table().len())
            .field("retained", &index.retained().len())
            .finish_non_exhaustive()
    }
}

/// A `Duration` as nanoseconds, saturating. A retention window longer than 292
/// years is the same as forever, and both are a configuration error rather than
/// something to panic over.
fn nanos(d: Duration) -> i64 {
    i64::try_from(d.as_nanos()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seam::{DeclaredNames, DeclaredTable, TestClock};
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

    fn one_entry(at_nanos: i64, pid: u32) -> SocketTable {
        SocketTable::new(
            at(at_nanos),
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), pid)],
        )
    }

    struct Fixture {
        attributor: SocketTableAttributor,
        clock: Arc<TestClock>,
    }

    /// Knows a name once, then forgets it. What an enumeration does when the
    /// process it named has exited.
    struct ForgetfulNamer {
        pid: u32,
        name: &'static str,
        asked: bool,
    }

    impl ForgetfulNamer {
        fn knowing(pid: u32, name: &'static str) -> Self {
            ForgetfulNamer {
                pid,
                name,
                asked: false,
            }
        }
    }

    impl crate::seam::ProcessNamer for ForgetfulNamer {
        fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>> {
            let mut out = HashMap::new();
            if !self.asked && pids.contains(&self.pid) {
                out.insert(self.pid, Arc::from(self.name));
            }
            self.asked = true;
            out
        }
    }

    /// Reports one name, then a different one for the same identifier. What an
    /// enumeration does after the platform reuses a process identifier.
    struct RenamingNamer {
        pid: u32,
        first: &'static str,
        second: &'static str,
        asked: bool,
    }

    impl RenamingNamer {
        fn new(pid: u32, first: &'static str, second: &'static str) -> Self {
            RenamingNamer {
                pid,
                first,
                second,
                asked: false,
            }
        }
    }

    impl crate::seam::ProcessNamer for RenamingNamer {
        fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>> {
            let name = if self.asked { self.second } else { self.first };
            self.asked = true;
            let mut out = HashMap::new();
            if pids.contains(&self.pid) {
                out.insert(self.pid, Arc::from(name));
            }
            out
        }
    }

    fn fixture(tables: Vec<Result<SocketTable, AttrError>>, named: &[(u32, &str)]) -> Fixture {
        let clock = Arc::new(TestClock::at(at(0)));
        let mut names = DeclaredNames::new();
        for (pid, name) in named {
            names = names.with(*pid, name);
        }
        Fixture {
            attributor: SocketTableAttributor::new(
                Box::new(DeclaredTable::sequence(tables)),
                Box::new(names),
                Arc::clone(&clock) as Arc<dyn Clock>,
                AttributorConfig::default(),
            ),
            clock,
        }
    }

    #[test]
    fn the_defaults_are_the_specification_values() {
        let c = AttributorConfig::default();
        assert_eq!(c.interval, Duration::from_secs(1));
        assert_eq!(c.retention, Duration::from_secs(30));
        assert_eq!(c.trigger_limit, Duration::from_millis(200));
    }

    #[test]
    fn an_unrefreshed_attributor_resolves_nothing_rather_than_panicking() {
        let f = fixture(vec![Ok(one_entry(0, 1))], &[]);
        assert_eq!(f.attributor.resolve(&udp_key(), at(100)), None);
        assert!(f.attributor.active_endpoints().is_empty());
    }

    #[test]
    fn a_refresh_publishes_an_index_that_resolves() {
        let mut f = fixture(vec![Ok(one_entry(0, 4242))], &[(4242, "eso64.exe")]);
        f.attributor.refresh().expect("the declared table reads");
        let a = f.attributor.resolve(&udp_key(), at(100)).expect("resolves");
        assert_eq!(a.pid, 4242);
        assert_eq!(&*a.process, "eso64.exe");
        assert_eq!(a.fidelity, Fidelity::Live);
        assert!(a.role.is_none());
        assert!(a.stage.is_none());
    }

    // SC-002 and the FR-018a origin, in one test over one endpoint's whole
    // life. Three separate tests could each pass while the transitions between
    // them were wrong.
    #[test]
    fn one_endpoint_goes_live_then_retained_then_gone() {
        let mut f = fixture(
            vec![
                Ok(one_entry(1_000, 5)),
                Ok(SocketTable::empty(at(2_000_000_000))),
            ],
            &[(5, "g.exe")],
        );

        f.attributor.refresh().expect("the first table reads");
        let live = f.attributor.resolve(&udp_key(), at(1_000)).expect("live");
        assert_eq!(live.fidelity, Fidelity::Live);
        assert_eq!(live.pid, 5);

        f.clock.set(at(2_000_000_000));
        f.attributor.refresh().expect("the second table reads");

        // The endpoint was last seen at 1000 ns. The window is thirty seconds
        // from there, not from the refresh at two seconds that noticed it gone.
        let retained = f
            .attributor
            .resolve(&udp_key(), at(5_000_000_000))
            .expect("inside the window");
        assert_eq!(retained.fidelity, Fidelity::Retained);
        assert_eq!(retained.pid, 5);
        assert_eq!(&*retained.process, "g.exe", "the name survives retention");

        assert_eq!(
            f.attributor.resolve(&udp_key(), at(31_000_000_000)),
            None,
            "thirty seconds after it was last seen, not after it was noticed gone"
        );
    }

    // SC-003. Port reuse inside the retention window.
    #[test]
    fn a_reused_port_resolves_to_the_new_owner() {
        let mut f = fixture(
            vec![Ok(one_entry(1_000, 5)), Ok(one_entry(2_000, 6))],
            &[(5, "old.exe"), (6, "new.exe")],
        );
        f.attributor.refresh().expect("first");
        f.clock.set(at(2_000));
        f.attributor.refresh().expect("second");

        let a = f
            .attributor
            .resolve(&udp_key(), at(2_500))
            .expect("resolves");
        assert_eq!(a.pid, 6, "the table is evidence, retention is inference");
        assert_eq!(a.fidelity, Fidelity::Live);
    }

    // FR-022. Retention does not grow without bound.
    #[test]
    fn entries_past_the_grace_period_are_discarded_on_refresh() {
        let mut f = fixture(
            vec![
                Ok(one_entry(0, 1)),
                Ok(SocketTable::empty(at(1_000_000_000))),
                Ok(SocketTable::empty(at(60_000_000_000))),
            ],
            &[],
        );
        f.attributor.refresh().expect("first");
        f.attributor.refresh().expect("second");
        assert_eq!(
            f.attributor.published().load().retained().len(),
            1,
            "still inside the window"
        );
        f.attributor.refresh().expect("third");
        assert_eq!(
            f.attributor.published().load().retained().len(),
            0,
            "past the window, and dropped rather than accumulated"
        );
    }

    // FR-030 and SC-008. A failed read leaves the published index alone.
    #[test]
    fn a_failed_refresh_leaves_the_previous_index_resolving_exactly_as_before() {
        let mut f = fixture(
            vec![
                Ok(one_entry(0, 7)),
                Err(AttrError::RefreshFailed {
                    detail: "declared".to_string(),
                }),
            ],
            &[(7, "g.exe")],
        );
        f.attributor.refresh().expect("the first read succeeds");
        let before = f.attributor.resolve(&udp_key(), at(100));
        assert!(before.is_some());

        let e = f
            .attributor
            .refresh()
            .expect_err("the second read fails as declared");
        assert!(e.is_transient());

        assert_eq!(
            f.attributor.resolve(&udp_key(), at(100)),
            before,
            "a transient failure must not silently unattribute everything after it"
        );
    }

    // Checklist CHK021. The first-refresh failure, where there is no previous
    // index to keep.
    #[test]
    fn a_failure_on_the_first_refresh_leaves_an_empty_index_rather_than_a_panic() {
        let clock = Arc::new(TestClock::at(at(0)));
        let mut a = SocketTableAttributor::new(
            Box::new(DeclaredTable::always_failing("no platform")),
            Box::new(DeclaredNames::new()),
            clock as Arc<dyn Clock>,
            AttributorConfig::default(),
        );
        assert!(a.refresh().is_err());
        assert_eq!(a.resolve(&udp_key(), at(100)), None);
        assert!(a.active_endpoints().is_empty());
    }

    // FR-014, FR-015, FR-017. The trigger arrives through the lookup path.
    #[test]
    fn an_unresolved_lookup_on_an_unseen_endpoint_requests_a_refresh() {
        let mut f = fixture(vec![Ok(SocketTable::empty(at(0)))], &[]);
        f.attributor.refresh().expect("reads");
        let schedule = f.attributor.schedule();
        assert!(!schedule.take_request(), "nothing requested yet");

        assert_eq!(f.attributor.resolve(&udp_key(), at(100)), None);
        assert!(schedule.take_request(), "the unseen endpoint triggered one");
    }

    #[test]
    fn a_burst_of_unresolved_lookups_requests_one_refresh() {
        let mut f = fixture(vec![Ok(SocketTable::empty(at(0)))], &[]);
        f.attributor.refresh().expect("reads");
        let schedule = f.attributor.schedule();
        schedule.take_request();

        // The clock does not move, so every request after the first is inside
        // the two hundred millisecond limit.
        for _ in 0..1_000 {
            f.attributor.resolve(&udp_key(), at(100));
        }
        assert!(schedule.take_request());
        assert!(!schedule.take_request(), "exactly one request was recorded");
    }

    #[test]
    fn a_lookup_on_a_carried_endpoint_triggers_nothing() {
        // The endpoint is in the table but the socket postdates the packet, so
        // the lookup fails. That is not an unseen endpoint and must not drive
        // the table read rate.
        let table = SocketTable::new(
            at(0),
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1).created_at(at(500))],
        );
        let mut f = fixture(vec![Ok(table)], &[]);
        f.attributor.refresh().expect("reads");
        let schedule = f.attributor.schedule();
        schedule.take_request();

        assert_eq!(f.attributor.resolve(&udp_key(), at(100)), None);
        assert!(!schedule.take_request());
    }

    // FR-013, FR-016.
    #[test]
    fn a_matched_process_start_requests_a_refresh_regardless_of_the_limit() {
        let mut f = fixture(vec![Ok(SocketTable::empty(at(0)))], &[]);
        f.attributor.refresh().expect("reads");
        let schedule = f.attributor.schedule();

        f.attributor.resolve(&udp_key(), at(100));
        assert!(schedule.take_request());

        // Inside the limit now.
        f.attributor.resolve(&udp_key(), at(100));
        assert!(!schedule.take_request());

        f.attributor.note_matched_process_start();
        assert!(schedule.take_request());
    }

    // FR-011, FR-012 through the attributor rather than the schedule.
    #[test]
    fn the_interval_governs_wants_refresh() {
        let mut f = fixture(vec![Ok(one_entry(0, 1))], &[]);
        assert!(f.attributor.wants_refresh(), "nothing read yet");
        f.attributor.refresh().expect("reads");
        assert!(!f.attributor.wants_refresh());

        f.clock.advance(999_999_999);
        assert!(!f.attributor.wants_refresh());
        f.clock.advance(1);
        assert!(f.attributor.wants_refresh(), "one second has elapsed");
    }

    // FR-023.
    #[test]
    fn active_endpoints_reports_current_plus_retained() {
        let mut f = fixture(
            vec![Ok(one_entry(0, 1)), Ok(SocketTable::empty(at(1_000)))],
            &[],
        );
        f.attributor.refresh().expect("first");
        assert_eq!(f.attributor.active_endpoints().len(), 1);

        f.clock.set(at(1_000));
        f.attributor.refresh().expect("second");
        assert_eq!(
            f.attributor.active_endpoints().len(),
            1,
            "gone from the table, still inside the retention window"
        );

        f.clock.set(at(60_000_000_000));
        assert!(
            f.attributor.active_endpoints().is_empty(),
            "past the window"
        );
    }

    // FR-024, FR-025. An unresolved lookup is not an error and drops nothing.
    #[test]
    fn an_unresolved_lookup_is_not_an_error() {
        let mut f = fixture(vec![Ok(SocketTable::empty(at(0)))], &[]);
        f.attributor.refresh().expect("reads");
        assert_eq!(
            f.attributor.resolve(&udp_key(), at(100)),
            None,
            "attempted and unresolved, which P-4 says is retained and marked"
        );
    }

    // The whole thing through the trait object, which is how the pipeline holds
    // it. If any of this needed an inherent method, the pipeline could not
    // reach it.
    #[test]
    fn the_whole_sequence_is_drivable_through_the_seam_alone() {
        let mut f = fixture(
            vec![Ok(one_entry(1_000, 5)), Ok(SocketTable::empty(at(2_000)))],
            &[(5, "g.exe")],
        );
        let seam: &mut dyn FlowAttributor = &mut f.attributor;

        seam.refresh().expect("first");
        assert_eq!(
            seam.resolve(&udp_key(), at(1_000)).expect("live").fidelity,
            Fidelity::Live
        );
        seam.refresh().expect("second");
        assert_eq!(
            seam.resolve(&udp_key(), at(2_000))
                .expect("retained")
                .fidelity,
            Fidelity::Retained
        );
        assert_eq!(seam.active_endpoints().len(), 1);
    }

    // --- Review of pull request 13 ---------------------------------------

    // A process that exits keeps the name it was known by while it held the
    // socket. The first version re-queried every retained identifier on each
    // refresh, so a name was lost the moment the process was gone from the
    // enumeration, and the tail of every connection went out unnamed.
    #[test]
    fn a_name_survives_the_process_that_owned_the_socket() {
        let clock = Arc::new(TestClock::at(at(0)));
        let mut attributor = SocketTableAttributor::new(
            Box::new(DeclaredTable::sequence(vec![
                Ok(one_entry(1_000, 5)),
                Ok(SocketTable::empty(at(2_000))),
            ])),
            // The namer knows the process on the first refresh and forgets it
            // afterwards, which is what an enumeration does once it exits.
            Box::new(ForgetfulNamer::knowing(5, "the-game.exe")),
            Arc::clone(&clock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );

        attributor.refresh().expect("first");
        assert_eq!(
            &*attributor
                .resolve(&udp_key(), at(1_000))
                .expect("live")
                .process,
            "the-game.exe"
        );

        clock.set(at(2_000));
        attributor.refresh().expect("second");

        let a = attributor.resolve(&udp_key(), at(2_000)).expect("retained");
        assert_eq!(a.fidelity, Fidelity::Retained);
        assert_eq!(
            &*a.process, "the-game.exe",
            "the name was known while the socket was live and must not be lost with the process"
        );
    }

    // The platform reuses process identifiers. A retained connection must not
    // acquire the name of whatever now holds its old identifier.
    #[test]
    fn a_recycled_identifier_does_not_rename_a_closed_connection() {
        let clock = Arc::new(TestClock::at(at(0)));
        let mut attributor = SocketTableAttributor::new(
            Box::new(DeclaredTable::sequence(vec![
                Ok(one_entry(1_000, 5)),
                Ok(SocketTable::empty(at(2_000))),
            ])),
            Box::new(RenamingNamer::new(5, "the-game.exe", "something-else.exe")),
            Arc::clone(&clock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );

        attributor.refresh().expect("first");
        clock.set(at(2_000));
        attributor.refresh().expect("second");

        assert_eq!(
            &*attributor
                .resolve(&udp_key(), at(2_000))
                .expect("retained")
                .process,
            "the-game.exe",
            "the identifier now belongs to another process; the connection does not"
        );
    }

    // A request recorded against the newly published index must survive the
    // refresh that published it. The first version published and then marked
    // refreshed, which erased any request made in between, and because
    // recording one also consumes the rate-limit window nothing could re-arm it
    // for two hundred milliseconds.
    #[test]
    fn a_request_made_after_publication_survives_the_refresh() {
        let mut f = fixture(vec![Ok(SocketTable::empty(at(0)))], &[]);
        let schedule = f.attributor.schedule();

        f.attributor.refresh().expect("reads");
        schedule.take_request();

        // A capture thread reads the index this refresh published, finds an
        // endpoint it does not carry, and asks for another.
        f.clock.set(at(1_000_000_000));
        f.attributor.resolve(&udp_key(), at(100));
        assert!(schedule.is_requested(), "the lookup recorded a request");

        // Whoever drives the cadence has not acted on it yet. It must still be
        // there.
        assert!(
            schedule.take_request(),
            "a request made against the published index must not be erased by the \
             refresh that published it"
        );
    }

    #[test]
    fn the_attributor_is_send_and_sync_behind_a_pointer() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<SocketTableAttributor>();

        let f = fixture(vec![Ok(one_entry(0, 1))], &[]);
        let boxed: Box<dyn FlowAttributor> = Box::new(f.attributor);
        let shared: Arc<dyn FlowAttributor> = Arc::from(boxed);
        let _second = Arc::clone(&shared);
    }

    // SC-006 through the attributor: many threads resolving one shared
    // attributor while its index is republished underneath them.
    #[test]
    fn several_threads_resolve_one_attributor_while_it_is_republished() {
        use std::thread;

        let clock = Arc::new(TestClock::at(at(0)));
        let mut attributor = SocketTableAttributor::new(
            Box::new(DeclaredTable::once(one_entry(0, 1))),
            Box::new(DeclaredNames::from([(1, "one.exe")])),
            Arc::clone(&clock) as Arc<dyn Clock>,
            AttributorConfig::default(),
        );
        attributor.refresh().expect("reads");
        let published = attributor.published();

        let shared: Arc<dyn FlowAttributor> =
            Arc::from(Box::new(attributor) as Box<dyn FlowAttributor>);

        let readers: Vec<_> = (0..3)
            .map(|_| {
                let shared = Arc::clone(&shared);
                thread::spawn(move || {
                    for _ in 0..5_000 {
                        if let Some(a) = shared.resolve(&udp_key(), at(100)) {
                            assert!(
                                (a.pid == 1 && &*a.process == "one.exe")
                                    || (a.pid == 2 && &*a.process == "two.exe"),
                                "observed a mixture: {a:?}"
                            );
                        }
                    }
                })
            })
            .collect();

        for i in 0..2_000u32 {
            let pid = if i % 2 == 0 { 1 } else { 2 };
            let name = if pid == 1 { "one.exe" } else { "two.exe" };
            let mut names = HashMap::new();
            names.insert(pid, Arc::from(name));
            published.publish(AttributionIndex::new(
                one_entry(0, pid),
                names,
                RetentionMap::new(),
                30_000_000_000,
            ));
        }

        for r in readers {
            r.join().expect("a reader finishes");
        }
    }
}
