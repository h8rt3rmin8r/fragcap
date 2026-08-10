// SPDX-License-Identifier: Apache-2.0

//! The file transport with rotation (specification section 14.2).
//!
//! Writes a chosen format to a path. With no rotation policy it is a single
//! segment, byte identical to the file sink S06 and S07 produced. With a size
//! or duration policy it closes the current segment at a clean section boundary
//! and opens the next numbered one, so every segment opens on its own in an
//! unmodified analyzer (constitution P-5).
//!
//! A rotated (intermediate) segment is closed by dropping its encoder after a
//! flush, not by writing a trailing statistics block: a per-segment statistics
//! block would either carry the whole run's counters (wrong for the segment) or
//! zeroes (a false statement about a segment that did carry packets, which P-9
//! forbids). The absence of the optional block is not a false statement; a
//! zeroed one would be. The final segment is finished with the run's real
//! statistics, so a single-segment capture is byte identical to today's.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fragcap_core::packet::CapturedPacket;
use fragcap_core::stats::CaptureStats;
use fragcap_core::traits::Sink;
use fragcap_core::SinkError;

use super::SinkFactory;
use crate::error::WriteError;

/// When a file segment is rotated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationPolicy {
    /// No rotation: a single segment at the base path.
    None,
    /// Rotate once the current segment reaches this many bytes.
    Size(u64),
    /// Rotate once the current segment has been open this long.
    Duration(Duration),
}

/// A `Write` that counts the bytes passing through it, so the size policy can
/// read the current segment's size without the encoder exposing it.
struct CountingWriter {
    inner: File,
    count: Arc<AtomicU64>,
}

impl Write for CountingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.count.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// A file sink that rotates into numbered segments.
pub struct RotatingFileSink {
    base: PathBuf,
    policy: RotationPolicy,
    factory: SinkFactory,
    current: Option<Box<dyn Sink>>,
    current_count: Arc<AtomicU64>,
    segment_index: u64,
    packets_in_segment: u64,
    segment_opened_at: Instant,
}

impl std::fmt::Debug for RotatingFileSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RotatingFileSink")
            .field("base", &self.base)
            .field("policy", &self.policy)
            .field("segment_index", &self.segment_index)
            .finish()
    }
}

impl RotatingFileSink {
    /// Open the first segment immediately, so an empty capture still produces a
    /// file (as the pcapng writer does, writing its Section Header Block eagerly).
    pub fn create(
        base: impl AsRef<Path>,
        policy: RotationPolicy,
        factory: SinkFactory,
    ) -> Result<Self, WriteError> {
        let mut sink = RotatingFileSink {
            base: base.as_ref().to_path_buf(),
            policy,
            factory,
            current: None,
            current_count: Arc::new(AtomicU64::new(0)),
            segment_index: 0,
            packets_in_segment: 0,
            segment_opened_at: Instant::now(),
        };
        sink.open_segment(0)?;
        Ok(sink)
    }

    /// The path for a given segment. With no rotation the base path is used
    /// unchanged; otherwise a zero-padded ordinal is inserted before the
    /// extension so segments sort lexically in capture order.
    fn segment_path(&self, index: u64) -> PathBuf {
        if self.policy == RotationPolicy::None {
            return self.base.clone();
        }
        let stem = self.base.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let numbered = match self.base.extension().and_then(|e| e.to_str()) {
            Some(ext) => format!("{stem}-{index:05}.{ext}"),
            None => format!("{stem}-{index:05}"),
        };
        match self.base.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.join(numbered),
            _ => PathBuf::from(numbered),
        }
    }

    fn open_segment(&mut self, index: u64) -> Result<(), WriteError> {
        let path = self.segment_path(index);
        let file = File::create(&path).map_err(|e| WriteError::Io {
            detail: format!("cannot create {}: {e}", path.display()),
        })?;
        let count = Arc::new(AtomicU64::new(0));
        let writer = CountingWriter {
            inner: file,
            count: Arc::clone(&count),
        };
        let encoder = self.factory.build(Box::new(writer))?;
        self.current = Some(encoder);
        self.current_count = count;
        self.segment_index = index;
        self.packets_in_segment = 0;
        self.segment_opened_at = Instant::now();
        Ok(())
    }

    /// Whether the current segment should be closed before the next packet.
    ///
    /// A segment is never rotated before it holds at least one packet, which is
    /// what keeps a threshold smaller than the header from spinning out empty
    /// segments: each segment carries at least one packet.
    fn should_rotate(&self) -> bool {
        if self.packets_in_segment == 0 {
            return false;
        }
        match self.policy {
            RotationPolicy::None => false,
            RotationPolicy::Size(limit) => self.current_count.load(Ordering::Relaxed) >= limit,
            RotationPolicy::Duration(window) => self.segment_opened_at.elapsed() >= window,
        }
    }

    /// Close the current segment without a statistics trailer (see the module
    /// note), then open the next.
    fn rotate(&mut self) -> Result<(), SinkError> {
        if let Some(mut encoder) = self.current.take() {
            encoder.flush()?;
            // Dropping the encoder closes its file at a clean section boundary.
            drop(encoder);
        }
        self.open_segment(self.segment_index + 1)
            .map_err(SinkError::from)
    }
}

impl Sink for RotatingFileSink {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError> {
        if self.should_rotate() {
            self.rotate()?;
        }
        let encoder = self
            .current
            .as_mut()
            .expect("a segment is always open between create and finish");
        encoder.write(packet)?;
        self.packets_in_segment += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        if let Some(encoder) = self.current.as_mut() {
            encoder.flush()?;
        }
        Ok(())
    }

    fn finish(mut self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError> {
        // The final segment carries the run's real statistics trailer, so a
        // single-segment (no-policy) capture is byte identical to today's file.
        if let Some(encoder) = self.current.take() {
            encoder.finish(stats)?;
        }
        Ok(())
    }
}
