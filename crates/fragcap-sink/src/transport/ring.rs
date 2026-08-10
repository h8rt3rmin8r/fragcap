// SPDX-License-Identifier: Apache-2.0

//! Ring mode: the rolling retained window (specification section 7.2, FR-8).
//!
//! A [`RingSink`] holds the most recently captured packets in memory, bounded by
//! a duration or a byte size, discarding the oldest as new ones arrive. It writes
//! the retained window to a capture file only when the capture ends, which is the
//! [`Sink::finish`] the pipeline already calls once at drain for every one of the
//! six session stop conditions. Ring mode therefore adds no stop condition and no
//! new trigger path; it is a retention policy over what a file sink would hold,
//! materialized through the same [`SinkFactory`] and pcapng writer the file sink
//! uses.
//!
//! This is deliberately distinct from the pipeline's internal bounded buffer of
//! specification section 12.4, which is also a "ring" (bounded, drop-oldest) but
//! is the backpressure buffer between the capture thread and the sink thread, not
//! an output mode. The retained set here is the ring window; the 12.4 buffer is
//! the bounded buffer. The glossary keeps the two terms apart.
//!
//! A ring eviction is not a captured-packet loss. [`RingSink::write`] accepts
//! every packet the pipeline delivers and returns `Ok`, so the pipeline's
//! conservation accounting (received + buffer_dropped + refusals = captured) is
//! preserved exactly as for any other sink; the count of evicted packets is the
//! ring sink's own reported accounting, the way a streaming sink's per-consumer
//! drops are its own (slice S15). The eviction is the operator's declared
//! retention scope, which constitution P-9 permits as long as it is counted (P-4).

use std::collections::VecDeque;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use fragcap_core::packet::CapturedPacket;
use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::SinkError;

use super::SinkFactory;

/// The bound on a ring's retained set: a time window or a byte size.
///
/// A size window is measured by captured length, the same per-packet quantity the
/// `--max-bytes` volume bound sums, so an operator reasons about one notion of
/// capture size across `--ring 64mb` and `--max-bytes 64mb`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingWindow {
    /// Retain packets whose capture instant is within this window measured back
    /// from the newest retained packet.
    Duration(Duration),
    /// Retain the newest packets whose total captured length is within this many
    /// bytes.
    Size(u64),
}

/// A sink that retains a rolling window of recent packets and dumps it on finish.
///
/// Holds no open file during capture; the dump file is created at
/// [`finish`](Sink::finish). The retained deque is ordered oldest-at-front, which
/// for the offline replay source is capture order, and the dump preserves it.
#[derive(Debug)]
pub struct RingSink {
    /// The dump target (the `--out` path).
    path: PathBuf,
    /// The retention bound.
    window: RingWindow,
    /// Builds the pcapng encoder at dump time (header preamble plus one Interface
    /// Description Block per declared interface).
    factory: SinkFactory,
    /// The rolling window, oldest at the front.
    retained: VecDeque<CapturedPacket>,
    /// Running sum of `captured_len()` over `retained`, for the size window.
    retained_bytes: u64,
    /// The greatest capture instant observed so far, in nanoseconds. The duration
    /// window is measured back from this, not from the last-arrived packet, so a
    /// late out-of-order packet carrying an old instant never redefines "newest"
    /// and so never evicts a genuinely recent packet. `i64::MIN` before any
    /// packet arrives.
    newest_nanos: i64,
    /// Count of packets evicted from the window: the sink's own accounting,
    /// distinct from any capture-loss counter.
    evicted: u64,
}

impl RingSink {
    /// A new ring sink over `path`, bounded by `window`, dumping through `factory`.
    pub fn create(path: PathBuf, window: RingWindow, factory: SinkFactory) -> Self {
        RingSink {
            path,
            window,
            factory,
            retained: VecDeque::new(),
            retained_bytes: 0,
            newest_nanos: i64::MIN,
            evicted: 0,
        }
    }

    /// Packets currently retained in the window.
    pub fn retained(&self) -> usize {
        self.retained.len()
    }

    /// Packets evicted from the window so far (the sink's own accounting).
    pub fn evicted(&self) -> u64 {
        self.evicted
    }

    /// Evict from the front until the retained set is within the window, but never
    /// below one packet: the newest packet is always retained even if it alone
    /// exceeds a size window, so a capture that saw traffic never dumps an empty
    /// file (the retained-inclusive rule the write gate already uses for a byte
    /// bound).
    fn evict_to_window(&mut self) {
        match self.window {
            RingWindow::Size(limit) => {
                while self.retained.len() > 1 && self.retained_bytes > limit {
                    if let Some(front) = self.retained.pop_front() {
                        self.retained_bytes = self
                            .retained_bytes
                            .saturating_sub(front.captured_len() as u64);
                        self.evicted = self.evicted.saturating_add(1);
                    }
                }
            }
            RingWindow::Duration(window) => {
                // Measured back from the greatest instant observed
                // (`newest_nanos`), not from the last-arrived packet, so a late
                // out-of-order packet with an old instant cannot shrink the window
                // and evict a genuinely recent packet. In the common monotonic
                // case the front is the oldest instant, so front eviction is exact
                // and O(evicted); a rare out-of-order old packet not at the front
                // is over-retained (safe) until it reaches the front.
                let window_nanos = window.as_nanos() as i64;
                while self.retained.len() > 1 {
                    match self.retained.front().map(|p| p.ts.as_nanos()) {
                        Some(front) if self.newest_nanos.saturating_sub(front) > window_nanos => {
                            if let Some(front) = self.retained.pop_front() {
                                self.retained_bytes = self
                                    .retained_bytes
                                    .saturating_sub(front.captured_len() as u64);
                                self.evicted = self.evicted.saturating_add(1);
                            }
                        }
                        _ => break,
                    }
                }
            }
        }
    }
}

impl Sink for RingSink {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(packet.captured_len() as u64);
        self.newest_nanos = self.newest_nanos.max(packet.ts.as_nanos());
        self.retained.push_back(packet.clone());
        self.evict_to_window();
        // Always accepted: a ring never fails a packet and is never retired for
        // its own eviction, so the pipeline conservation invariant holds.
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        // Nothing is written to disk until finish, so there is nothing to flush.
        Ok(())
    }

    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError> {
        let file = File::create(&self.path).map_err(|e| SinkError::Write {
            detail: format!("cannot create {}: {e}", self.path.display()),
        })?;
        let mut encoder = self
            .factory
            .build(Box::new(file))
            .map_err(SinkError::from)?;
        for packet in &self.retained {
            encoder.write(packet)?;
        }
        // The dump carries the run's real statistics trailer, exactly as the file
        // sink does, so a whole-input dump is byte-comparable to a plain capture.
        encoder.finish(stats)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_core::interface::InterfaceId;
    use fragcap_core::packet::{Payload, RawPacket, Timestamp};

    use crate::transport::{Format, InterfaceSpec};
    use fragcap_core::LinkType;

    /// A captured packet of `len` bytes at instant `ts` nanoseconds.
    fn packet(len: usize, ts: i64) -> CapturedPacket {
        CapturedPacket::from_raw(
            RawPacket::new(
                Timestamp::from_nanos(ts),
                Payload::copy_from_slice(&vec![0u8; len]),
                len as u32,
            ),
            InterfaceId::default(),
        )
    }

    /// A ring sink with the given window over a throwaway pcapng factory. The
    /// retention tests never dump, so the factory's interfaces are irrelevant.
    fn ring(window: RingWindow) -> Box<RingSink> {
        let factory = SinkFactory::new(
            Format::Pcapng,
            vec![InterfaceSpec::new("capture", LinkType::ETHERNET, 65_535)],
        );
        Box::new(RingSink::create(
            PathBuf::from("unused.fcapng"),
            window,
            factory,
        ))
    }

    // FR-001, FR-002 (size), R5. A size window retains exactly the newest packets
    // fitting the window, evicting from the front, and keeps at least the newest
    // packet even when the window is smaller than one packet.
    #[test]
    fn a_size_window_retains_the_newest_bytes() {
        // Window of 100 bytes, packets of 40 bytes each.
        let mut ring = ring(RingWindow::Size(100));
        for i in 0..6 {
            ring.write(&packet(40, i)).unwrap();
        }
        // 40 + 40 = 80 fits; a third (120) would exceed 100, so the oldest is
        // evicted down to the newest two (80 bytes).
        assert_eq!(ring.retained(), 2);
        assert_eq!(ring.evicted(), 4);
        assert_eq!(ring.evicted() + ring.retained() as u64, 6, "conservation");
    }

    // R5. A window smaller than one packet keeps that one packet, so a capture
    // that saw traffic never dumps an empty file.
    #[test]
    fn a_window_smaller_than_one_packet_keeps_one() {
        let mut ring = ring(RingWindow::Size(10));
        for i in 0..4 {
            ring.write(&packet(40, i)).unwrap();
        }
        assert_eq!(ring.retained(), 1, "the newest packet is always retained");
        assert_eq!(ring.evicted(), 3);
    }

    // FR-002 (duration), R4. A duration window retains exactly the packets whose
    // instant is within the window measured back from the newest retained instant.
    #[test]
    fn a_duration_window_retains_the_recent_tail_by_instant() {
        // Window of 100 ns; packets at 0, 50, 100, 150, 200 ns.
        let mut ring = ring(RingWindow::Duration(Duration::from_nanos(100)));
        for i in 0..5 {
            ring.write(&packet(10, i * 50)).unwrap();
        }
        // Newest is at 200; the window keeps instants > 100 (100, 150, 200), so
        // the packets at 0 and 50 are evicted.
        assert_eq!(ring.retained(), 3);
        assert_eq!(ring.evicted(), 2);
    }

    // R4, the dangerous direction. A late-arriving packet with an old instant must
    // not redefine "newest" and evict a genuinely recent packet. The window is
    // measured back from the greatest instant observed, so the recent packet is
    // kept; the stale late packet is over-retained (safe), never allowed to shrink
    // the window.
    #[test]
    fn a_late_old_packet_does_not_evict_a_recent_one() {
        let mut ring = ring(RingWindow::Duration(Duration::from_nanos(100)));
        ring.write(&packet(10, 300)).unwrap(); // recent
        ring.write(&packet(10, 50)).unwrap(); // far older, arrives late
                                              // The recent packet at 300 is retained: the late old packet did not
                                              // shrink the window. Nothing recent was evicted.
        assert!(
            ring.retained() >= 1,
            "the recent packet must survive a late old arrival"
        );
        assert_eq!(ring.evicted(), 0, "a late old packet evicts nothing recent");
    }

    // FR-009. An unbounded-enough window retains everything; nothing is evicted
    // and conservation holds trivially.
    #[test]
    fn a_large_window_retains_everything() {
        let mut ring = ring(RingWindow::Size(1_000_000));
        for i in 0..10 {
            ring.write(&packet(50, i)).unwrap();
        }
        assert_eq!(ring.retained(), 10);
        assert_eq!(ring.evicted(), 0);
    }
}
