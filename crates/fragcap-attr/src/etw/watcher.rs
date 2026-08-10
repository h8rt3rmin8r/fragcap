// SPDX-License-Identifier: Apache-2.0

//! The [`ProcessWatcher`] itself: session, consumer, snapshot, in that order.

use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

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
}

impl EtwWatcher {
    /// Start watching.
    ///
    /// The order here is load-bearing and is not an implementation detail.
    /// Subscribing before snapshotting can report one process twice, once as an
    /// event and once in the snapshot, and the tree reconciles that into a
    /// single node. Snapshotting first leaves a window in which a process
    /// created after the snapshot and before the subscription is reported by
    /// neither source, and nothing downstream can detect that it is missing. A
    /// duplicate is visible and fixable; a gap in a launcher chain is neither,
    /// and it is the failure this whole slice exists to prevent.
    pub fn start(session_name: &str) -> Result<Self, WatcherError> {
        let session = Session::start(session_name)?;
        let fanout = Arc::new(Fanout::new());

        // Subscribed and consuming from here on.
        let consumer = Consumer::open(session.name(), Arc::clone(&fanout)).ok_or(
            WatcherError::SessionUnavailable {
                code: 0,
                detail: "the session started but could not be opened for reading.".into(),
            },
        )?;

        // Only now: everything created during the two lines above appears in
        // the event stream, in the snapshot, or in both.
        let snapshot = snapshot::take();

        Ok(EtwWatcher {
            fanout,
            session,
            consumer: Some(consumer),
            snapshot,
        })
    }

    /// What the watcher has observed about its own operation.
    ///
    /// The kernel's own loss counts are relayed rather than accumulated. A
    /// query that fails leaves the previous figures rather than reporting zero,
    /// because reporting zero losses because the question could not be asked is
    /// the comfortable untruth P-9 forbids.
    pub fn report(&self) -> WatcherReport {
        let (events, buffers) = self.session.lost();
        let unreadable = events == u64::MAX;
        WatcherReport {
            events_lost: if unreadable {
                self.fanout.unparsed.load(Ordering::Relaxed)
            } else {
                events + self.fanout.unparsed.load(Ordering::Relaxed)
            },
            buffers_lost: if unreadable { 0 } else { buffers },
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
