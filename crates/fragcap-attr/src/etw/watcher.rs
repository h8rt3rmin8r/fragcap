// SPDX-License-Identifier: Apache-2.0

//! The [`ProcessWatcher`] itself: session, consumer, snapshot, in that order.

use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use fragcap_core::packet::Timestamp;
use fragcap_core::process::{ProcessEvent, ProcessRecord, WatcherReport};
use fragcap_core::traits::ProcessWatcher;

use super::consumer::{Consumer, Fanout};
use super::session::Session;
use super::snapshot;
use super::WatcherError;

/// Observes process creation and exit through an ETW session of its own.
pub struct EtwWatcher {
    fanout: Arc<Fanout>,
    session: Session,
    /// Dropped before the session, which is what stops the consumer thread
    /// before the session it is reading disappears underneath it.
    consumer: Option<Consumer>,
    snapshot: Vec<ProcessRecord>,
    /// The instant the snapshot reflects, for the tree to bound reconciliation
    /// with. See [`snapshot_taken_at`](Self::snapshot_taken_at).
    snapshot_at: Option<Timestamp>,
}

impl EtwWatcher {
    /// Start watching.
    ///
    /// The order here is load-bearing and is not an implementation detail.
    /// Consuming begins before snapshotting, so a process created during
    /// startup appears in the event stream, in the snapshot, or in both, and
    /// the tree reconciles a duplicate into a single node. Snapshotting first
    /// would leave a window in which a process created after the snapshot and
    /// before consumption is reported by neither source, and nothing downstream
    /// could detect that it is missing. A duplicate is visible and fixable; a
    /// gap in a launcher chain is neither, and it is the failure this whole
    /// slice exists to prevent.
    ///
    /// The events consumed before any caller subscribes are held in the
    /// consumer's backlog and delivered to the first subscriber, so the window
    /// between consumption starting and the caller subscribing does not lose
    /// them either. See [`consumer`](super::consumer).
    pub fn start(session_name: &str) -> Result<Self, WatcherError> {
        let session = Session::start(session_name)?;
        let fanout = Arc::new(Fanout::new());

        // Consuming from here on. Events are held in the backlog until the
        // first subscriber attaches.
        let consumer = Consumer::open(session.name(), Arc::clone(&fanout)).ok_or(
            WatcherError::SessionUnavailable {
                code: 0,
                detail: "the session started but could not be opened for reading.".into(),
            },
        )?;

        let snapshot = snapshot::take();
        // Stamped after the snapshot returns, so every process in it was alive
        // at or before this instant. The tree uses it to tell a process's own
        // late start event from a different process that later reused its
        // identifier. The wall clock is the same one the event timestamps come
        // from, both being system time.
        let snapshot_at = now();

        Ok(EtwWatcher {
            fanout,
            session,
            consumer: Some(consumer),
            snapshot,
            snapshot_at,
        })
    }

    /// The instant the startup snapshot reflects, when it could be read.
    ///
    /// A consumer folding [`snapshot`](Self::snapshot) into a
    /// [`ProcessTree`](fragcap_core::ProcessTree) should pass this to
    /// `apply_snapshot_at` rather than `apply_snapshot`, so that a later start
    /// event reusing a snapshot process's identifier is recorded as a distinct
    /// process rather than merged into the snapshot node.
    pub fn snapshot_taken_at(&self) -> Option<Timestamp> {
        self.snapshot_at
    }

    /// What the watcher has observed about its own operation.
    ///
    /// The kernel's own loss counts are relayed rather than accumulated, and a
    /// query that fails returns the last figures a query did read rather than
    /// zero, so a transient failure cannot make an incomplete trace look
    /// lossless. Parser failures are added to the kernel's own event losses,
    /// because both are events fragcap did not get to observe.
    pub fn report(&self) -> WatcherReport {
        let (events, buffers) = self.session.lost();
        WatcherReport {
            events_lost: events + self.fanout.unparsed.load(Ordering::Relaxed),
            buffers_lost: buffers,
            running: self.consumer.is_some(),
        }
    }

    /// Rundown events seen and deliberately not published.
    ///
    /// The kernel emits these at session start to describe processes that were
    /// already running. They are not published because the startup snapshot
    /// already covers those processes, and because a rundown record carries the
    /// same stale parent identifier a running process does, so treating one as
    /// a creation event would claim creation-time ancestry it does not have.
    /// Counted rather than silently dropped, per P-4.
    pub fn rundown_ignored(&self) -> u64 {
        self.fanout.rundown_ignored.load(Ordering::Relaxed)
    }

    /// Stop, and report what was observed.
    pub fn stop(mut self) -> WatcherReport {
        let mut report = self.report();
        self.consumer = None;
        report.running = false;
        report
    }
}

impl ProcessWatcher for EtwWatcher {
    fn subscribe(&self) -> Receiver<ProcessEvent> {
        self.fanout.subscribe()
    }

    fn snapshot(&self) -> Vec<ProcessRecord> {
        self.snapshot.clone()
    }
}

/// The wall clock as nanoseconds since the Unix epoch.
///
/// `SystemTime` reads the operating system's system time, the same clock the
/// ETW event timestamps are derived from, so the snapshot instant and the event
/// timestamps are comparable. `None` only if the clock is set before the Unix
/// epoch, which is not a real configuration and costs nothing to tolerate.
fn now() -> Option<Timestamp> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some(Timestamp::from_nanos(nanos as i64))
}
