// SPDX-License-Identifier: Apache-2.0

//! The non-terminal heartbeat line (slice S069, FR-004).
//!
//! When stderr is not a terminal, no redraw applies (`contracts/status-block.md`
//! is terminal-only); a run left silent for a long stretch is still not
//! silent forever. A fixed 30-second interval, reset by every ordinary
//! progress line, per `contracts/heartbeat-line.md` and the S069
//! Clarifications session (2026-08-22).

use std::time::{Duration, Instant};

/// The fixed interval a non-terminal run may go with no progress line before
/// a heartbeat line is due. Not configurable: the issue asks for "not
/// silent," not for a tunable rate.
pub const INTERVAL: Duration = Duration::from_secs(30);

/// Tracks when the interval last reset, so `due` can be checked on every
/// tick without a background timer thread.
#[derive(Clone, Copy, Debug)]
pub struct Heartbeat {
    last_progress_at: Instant,
}

impl Heartbeat {
    /// A fresh timer, its interval starting now.
    pub fn new(now: Instant) -> Self {
        Heartbeat {
            last_progress_at: now,
        }
    }

    /// Whether a heartbeat line is due at `now`.
    pub fn due(&self, now: Instant) -> bool {
        now.duration_since(self.last_progress_at) >= INTERVAL
    }

    /// Reset the interval: called whenever an ordinary progress line was
    /// just written (a real update reset the clock a heartbeat exists only
    /// to substitute for), and after a heartbeat line itself is emitted.
    pub fn note_progress(&mut self, now: Instant) {
        self.last_progress_at = now;
    }
}

/// The heartbeat line's text: elapsed time and packets written, the same
/// plain `label: value` shape as every other human progress line, carrying
/// no ANSI byte.
pub fn render_heartbeat(elapsed: Duration, packets: u64) -> String {
    let secs = elapsed.as_secs();
    format!(
        "still capturing: elapsed {:02}:{:02}:{:02}, {packets} packets written",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_heartbeat_is_not_due_before_the_interval_elapses() {
        let start = Instant::now();
        let hb = Heartbeat::new(start);
        assert!(!hb.due(start));
        assert!(!hb.due(start + Duration::from_secs(29)));
    }

    #[test]
    fn a_heartbeat_is_due_at_or_after_the_interval() {
        let start = Instant::now();
        let hb = Heartbeat::new(start);
        assert!(hb.due(start + Duration::from_secs(30)));
        assert!(hb.due(start + Duration::from_secs(45)));
    }

    #[test]
    fn noting_progress_resets_the_interval() {
        let start = Instant::now();
        let mut hb = Heartbeat::new(start);
        let reset_at = start + Duration::from_secs(20);
        hb.note_progress(reset_at);
        assert!(!hb.due(reset_at + Duration::from_secs(29)));
        assert!(hb.due(reset_at + Duration::from_secs(30)));
    }

    #[test]
    fn the_heartbeat_line_carries_elapsed_time_and_a_packet_count_and_no_escape_byte() {
        let text = render_heartbeat(Duration::from_secs(135), 4102);
        assert!(text.contains("00:02:15"));
        assert!(text.contains("4102"));
        assert!(!text.contains('\x1b'));
    }
}
