// SPDX-License-Identifier: Apache-2.0

//! The three injectable halves of the socket table attributor: where time comes
//! from, where a socket table comes from, and where an image name comes from.
//!
//! All three exist because specification section 25.1 requires the whole of
//! section 11 be exercisable with no capture driver, no elevation, and no game.
//! A cadence of one second, a rate limit of two hundred milliseconds, and a
//! retention window of thirty seconds are not testable against a real clock in
//! any way anyone would run twice.
//!
//! The declared implementations are public rather than `#[cfg(test)]`, for the
//! same reason [`crate::ScriptedAttributor`] is: S13 and S14 will need to drive
//! an attributor without a platform, and a test-only type cannot be used from
//! another crate.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use fragcap_core::error::AttrError;
use fragcap_core::packet::Timestamp;

use crate::table::SocketTable;

/// Where the attributor's sense of now comes from.
///
/// Scoped to this crate deliberately. It exists because sections 11.2 and 11.4
/// are otherwise untestable at tier 1, not as a workspace-wide abstraction, and
/// it should not become one without a reason of its own.
///
/// Note what this is not for. It is never the source of a packet's instant:
/// that arrives on [`fragcap_core::traits::FlowAttributor::resolve`] and is the
/// instant the packet was observed. This clock answers "now", which the
/// attributor needs for exactly two things, the refresh cadence and the trigger
/// rate limit, both of which are wall-clock costs.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// The system clock, in nanoseconds since the Unix epoch.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => Timestamp::from_parts(d.as_secs() as i64, d.subsec_nanos()),
            // Before the epoch. Saturate at it rather than reporting a
            // plausible wrong instant.
            Err(_) => Timestamp::from_nanos(0),
        }
    }
}

/// A clock a test drives.
///
/// `set` and `advance` take `&self` over an atomic rather than `&mut self`,
/// because the attributor holds its clock as an `Arc<dyn Clock>` and a test
/// that could not move time through a shared handle could not drive the cadence
/// at all.
#[derive(Debug)]
pub struct TestClock(AtomicI64);

impl TestClock {
    pub fn at(t: Timestamp) -> Self {
        TestClock(AtomicI64::new(t.as_nanos()))
    }

    pub fn set(&self, t: Timestamp) {
        self.0.store(t.as_nanos(), Ordering::SeqCst);
    }

    pub fn advance(&self, nanos: i64) {
        self.0.fetch_add(nanos, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_nanos(self.0.load(Ordering::SeqCst))
    }
}

/// Produces a socket table snapshot.
///
/// Fallible, because a real table read can fail transiently and specification
/// section 11 requires that a failure leave the previous snapshot standing
/// rather than replacing it with an empty one.
///
/// `Sync` even though only `&mut self` ever touches it. The attributor that
/// owns this is shared across every capture thread, which specification section
/// 11.6 requires, and a type is only shareable if everything it owns is. The
/// bound costs nothing real: a source is a handle and a buffer, and both
/// declared and platform implementations satisfy it without effort.
pub trait SocketTableSource: Send + Sync {
    fn read(&mut self) -> Result<SocketTable, AttrError>;
}

/// A source that returns declared tables in order, then repeats the last one.
///
/// Scriptable to fail, which is what makes the FR-030 failure path testable.
#[derive(Debug, Default)]
pub struct DeclaredTable {
    tables: Vec<Result<SocketTable, AttrError>>,
    next: usize,
}

impl DeclaredTable {
    /// One table, returned for every read.
    pub fn once(table: SocketTable) -> Self {
        DeclaredTable {
            tables: vec![Ok(table)],
            next: 0,
        }
    }

    /// A sequence. The last entry is repeated once the sequence is exhausted,
    /// so a test that refreshes more often than it declared does not fall off
    /// the end into a different failure than the one it meant to exercise.
    pub fn sequence(tables: Vec<Result<SocketTable, AttrError>>) -> Self {
        DeclaredTable { tables, next: 0 }
    }

    /// A source that always fails, for the first-refresh failure case.
    pub fn always_failing(detail: &str) -> Self {
        DeclaredTable {
            tables: vec![Err(AttrError::RefreshFailed {
                detail: detail.to_string(),
            })],
            next: 0,
        }
    }
}

impl SocketTableSource for DeclaredTable {
    fn read(&mut self) -> Result<SocketTable, AttrError> {
        if self.tables.is_empty() {
            return Err(AttrError::Unavailable {
                detail: "no table was declared".to_string(),
            });
        }
        let i = self.next.min(self.tables.len() - 1);
        self.next = self.next.saturating_add(1);
        self.tables[i].clone()
    }
}

/// Turns owning process identifiers into image names.
///
/// Takes the whole set the table reported rather than one identifier at a time,
/// so an implementation enumerates once per refresh rather than once per
/// process. Specification section 11.6 and requirement FR-033a: the names are
/// part of the published snapshot, because resolving one on the acquisition
/// path would put an operating system call there at the worst possible moment,
/// the start of a session when the most sockets are opening at once.
///
/// Returns a map rather than a `Result`. A name that cannot be resolved is a
/// missing name and not a failure: constitution P-9 and requirement FR-032
/// require the attribution be produced carrying the observed identifier
/// regardless, because the identifier is what was observed.
///
/// `Sync` for the same reason [`SocketTableSource`] is.
pub trait ProcessNamer: Send + Sync {
    fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>>;
}

/// Names from a declared map. Identifiers not in the map resolve to nothing,
/// which is the case FR-032 exists for.
#[derive(Clone, Debug, Default)]
pub struct DeclaredNames(HashMap<u32, Arc<str>>);

impl DeclaredNames {
    pub fn new() -> Self {
        DeclaredNames::default()
    }

    pub fn with(mut self, pid: u32, name: &str) -> Self {
        self.0.insert(pid, Arc::from(name));
        self
    }
}

impl<const N: usize> From<[(u32, &str); N]> for DeclaredNames {
    fn from(pairs: [(u32, &str); N]) -> Self {
        let mut m = DeclaredNames::default();
        for (pid, name) in pairs {
            m.0.insert(pid, Arc::from(name));
        }
        m
    }
}

impl ProcessNamer for DeclaredNames {
    fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>> {
        pids.iter()
            .filter_map(|pid| self.0.get(pid).map(|n| (*pid, Arc::clone(n))))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::SocketTableEntry;

    fn addr(s: &str) -> std::net::SocketAddr {
        s.parse().expect("test address must parse")
    }

    fn table(at: i64) -> SocketTable {
        SocketTable::new(
            Timestamp::from_nanos(at),
            vec![SocketTableEntry::udp(addr("192.0.2.10:30000"), 1)],
        )
    }

    #[test]
    fn a_test_clock_moves_through_a_shared_handle() {
        // The property the attributor depends on. If this needed `&mut`, no
        // test could advance the clock an attributor is holding.
        let clock: Arc<dyn Clock> = Arc::new(TestClock::at(Timestamp::from_nanos(100)));
        let handle = Arc::clone(&clock);
        assert_eq!(clock.now(), Timestamp::from_nanos(100));

        // Downcasting is not available through `dyn Clock`, so a test drives
        // the concrete type and shares it as the trait object.
        let concrete = Arc::new(TestClock::at(Timestamp::from_nanos(100)));
        let as_trait: Arc<dyn Clock> = Arc::clone(&concrete) as Arc<dyn Clock>;
        concrete.advance(50);
        assert_eq!(as_trait.now(), Timestamp::from_nanos(150));
        concrete.set(Timestamp::from_nanos(7));
        assert_eq!(as_trait.now(), Timestamp::from_nanos(7));

        assert_eq!(handle.now(), Timestamp::from_nanos(100));
    }

    #[test]
    fn the_system_clock_reports_something_after_the_epoch() {
        // Not a precise assertion, deliberately. The only thing worth checking
        // here is that the conversion is not off by an epoch.
        assert!(SystemClock.now().as_nanos() > 1_600_000_000_000_000_000);
    }

    #[test]
    fn a_declared_table_repeats_its_last_entry() {
        let mut s = DeclaredTable::sequence(vec![Ok(table(1)), Ok(table(2))]);
        assert_eq!(s.read().unwrap().taken_at(), Timestamp::from_nanos(1));
        assert_eq!(s.read().unwrap().taken_at(), Timestamp::from_nanos(2));
        assert_eq!(
            s.read().unwrap().taken_at(),
            Timestamp::from_nanos(2),
            "the last entry repeats rather than the source running out"
        );
    }

    #[test]
    fn a_declared_table_can_be_scripted_to_fail() {
        let mut s = DeclaredTable::sequence(vec![
            Ok(table(1)),
            Err(AttrError::RefreshFailed {
                detail: "declared".to_string(),
            }),
        ]);
        assert!(s.read().is_ok());
        let e = s.read().expect_err("the second read fails");
        assert!(e.is_transient());
    }

    #[test]
    fn declared_names_answer_only_what_was_declared() {
        let mut n = DeclaredNames::from([(1, "a.exe"), (2, "b.exe")]);
        let got = n.names(&[1, 3]);
        assert_eq!(got.len(), 1);
        assert_eq!(&*got[&1], "a.exe");
        assert!(
            !got.contains_key(&3),
            "an unknown identifier resolves to no name rather than a placeholder"
        );
    }
}
