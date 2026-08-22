// SPDX-License-Identifier: Apache-2.0

//! A live-readable mirror of the counters [`super::output_loop`] otherwise
//! keeps as local variables until the run ends (slice S069).
//!
//! `sink_dropped` and the per-image holder tally are, before this module,
//! plain locals inside the output loop's single-threaded function, folded
//! into [`crate::stats::CaptureStats`] only once the loop returns. A caller
//! that wants to show them while the run is still active (the live capture
//! status display) has nowhere to read them from. [`LiveStats`] is the
//! `Arc`-shared handle that closes that gap, mirroring the split
//! [`crate::traits::WriteGate`]'s facade implementation already uses between
//! a gate and its driver-side handle: one side is written from inside the
//! pipeline, the other is cloned out and read from elsewhere, and neither
//! side blocks the other beyond an ordinary lock or atomic operation.
//!
//! `buffer_dropped` is mirrored the same way, but from a value the output
//! loop already had free access to (`Consumer::next_and_evicted`), not a new
//! counter.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// A live-readable clone of three pipeline counters. Cheap to clone: every
/// field is an `Arc`.
///
/// Obtained from [`super::Pipeline::live_stats`] any time after construction,
/// including before [`super::Pipeline::run`] consumes the pipeline by value.
/// Reading a field never blocks the output loop for longer than an atomic
/// store or a `Mutex` held for a `BTreeMap` insert; see the S069
/// `research.md` decision R-2 for why a plain `Mutex` is the right tool for
/// the holder tally here (a coarse, infrequent read) rather than the
/// lock-free structure the per-packet attribution snapshot needs.
#[derive(Clone, Debug, Default)]
pub struct LiveStats {
    sink_dropped: Arc<AtomicU64>,
    holder_tally: Arc<Mutex<BTreeMap<Arc<str>, u64>>>,
    buffer_dropped: Arc<AtomicU64>,
}

impl LiveStats {
    /// A fresh handle, every counter at zero.
    pub(crate) fn new() -> Self {
        LiveStats::default()
    }

    /// Record one sink refusal. Called from the output loop at the same
    /// three sites that already advance a local `sink_dropped: u64`.
    pub(crate) fn record_sink_dropped(&self) {
        self.sink_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Record one admitted packet's socket-holding image. Called from the
    /// output loop at the same site that already updates a local
    /// `holder_tally: BTreeMap<Arc<str>, u64>`.
    pub(crate) fn record_holder(&self, image: &Arc<str>) {
        let mut tally = self
            .holder_tally
            .lock()
            .expect("the holder tally mutex is never poisoned");
        let slot = tally.entry(Arc::clone(image)).or_insert(0);
        *slot = slot.saturating_add(1);
    }

    /// Mirror the buffer's current eviction count. Called from the output
    /// loop on every `Consumer::next_and_evicted` return, per research R-2.
    pub(crate) fn set_buffer_dropped(&self, evicted: u64) {
        self.buffer_dropped.store(evicted, Ordering::Relaxed);
    }

    /// The current sink-dropped count.
    pub fn sink_dropped(&self) -> u64 {
        self.sink_dropped.load(Ordering::Relaxed)
    }

    /// The current buffer-dropped (eviction) count.
    pub fn buffer_dropped(&self) -> u64 {
        self.buffer_dropped.load(Ordering::Relaxed)
    }

    /// A snapshot of the holder tally, sorted by count descending then image
    /// name ascending: a total order, so a caller renders the same top
    /// contributors on every read of an unchanged tally, the same tiebreak
    /// discipline [`crate::stats::CaptureStats::dominant_holder`] already
    /// uses.
    pub fn holder_tally_snapshot(&self) -> Vec<(Arc<str>, u64)> {
        let tally = self
            .holder_tally
            .lock()
            .expect("the holder tally mutex is never poisoned");
        let mut entries: Vec<(Arc<str>, u64)> =
            tally.iter().map(|(k, v)| (Arc::clone(k), *v)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_handle_reports_zero_and_an_empty_tally() {
        let live = LiveStats::new();
        assert_eq!(live.sink_dropped(), 0);
        assert_eq!(live.buffer_dropped(), 0);
        assert!(live.holder_tally_snapshot().is_empty());
    }

    #[test]
    fn a_clone_observes_writes_made_through_another_clone() {
        let live = LiveStats::new();
        let writer = live.clone();

        writer.record_sink_dropped();
        writer.record_sink_dropped();
        writer.set_buffer_dropped(5);
        writer.record_holder(&Arc::from("game.exe"));
        writer.record_holder(&Arc::from("game.exe"));
        writer.record_holder(&Arc::from("launcher.exe"));

        assert_eq!(live.sink_dropped(), 2);
        assert_eq!(live.buffer_dropped(), 5);
        assert_eq!(
            live.holder_tally_snapshot(),
            vec![(Arc::from("game.exe"), 2), (Arc::from("launcher.exe"), 1),]
        );
    }

    #[test]
    fn the_holder_tally_snapshot_breaks_ties_by_name_ascending() {
        let live = LiveStats::new();
        live.record_holder(&Arc::from("zzz.exe"));
        live.record_holder(&Arc::from("aaa.exe"));
        assert_eq!(
            live.holder_tally_snapshot(),
            vec![(Arc::from("aaa.exe"), 1), (Arc::from("zzz.exe"), 1)]
        );
    }
}
