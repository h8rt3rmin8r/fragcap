// SPDX-License-Identifier: Apache-2.0

//! The completion summary of specification section 17.5 and FR-021.
//!
//! The end-of-run accounting an operator reads. It surfaces the captured and
//! attributed counts and every existing discard counter, the ones the pipeline
//! and the session already maintain, and invents none: a bare success that hid a
//! buffer drop or a watch-time discard is exactly what FR-021 forbids.

use fragcap::StopReason;

use crate::events::Event;

/// The end-of-run accounting, assembled from the pipeline report and the
/// session statistics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompletionSummary {
    /// Whether a target was ever acquired. `false` is the target-never-appeared
    /// outcome that exits 1.
    pub acquired: bool,
    /// Why capture stopped, once it has.
    pub stop_reason: Option<StopReason>,
    /// Packets fragcap accepted from the source.
    pub packets_captured: u64,
    /// Packets written to the capture: the packet records on disk. With a volume
    /// bound this is the bound, and it equals captured less the gate's discards
    /// (slice 017). Without a gate (target never acquired) it is zero.
    pub retained: u64,
    /// Packets that resolved to a process.
    pub packets_attributed: u64,
    /// Packets retained and marked because attribution did not resolve.
    pub packets_unattributed: u64,
    /// Packets discarded while watching, before a target was acquired.
    pub watching_discarded: u64,
    /// Packets discarded out of the capture window (offered after a stop or
    /// before arm).
    pub discarded_out_of_window: u64,
    /// Packets fragcap's bounded buffer dropped.
    pub buffer_dropped: u64,
    /// Packets a sink could not accept.
    pub sink_dropped: u64,
    /// Packets excluded because they belong to a process this capture does not
    /// cover (slice S064).
    pub scope_discarded: u64,
    /// Packets excluded on scope grounds that carried no attribution at all, so
    /// it is not known whether they were the capture's.
    ///
    /// Reported apart from `scope_discarded` on purpose. These might have been
    /// the target's, dropped because the socket table had not yet published the
    /// socket that would have named them, so a non-zero value here is worth
    /// investigating while a non-zero `scope_discarded` is the feature working.
    pub scope_unresolved_discarded: u64,
    /// What was written, per process image.
    ///
    /// Counted from gate-admitted packets only, so since slice S064 this is a
    /// breakdown of the file rather than of the wire. Under the default scope it
    /// is the target and nothing else; a second image here means something
    /// bound to the profile also had traffic.
    pub written_by_image: Vec<(String, u64)>,
}

impl CompletionSummary {
    /// The stop reason as a stable word, for both the human line and any future
    /// machine field.
    pub fn stop_word(&self) -> &'static str {
        match self.stop_reason {
            Some(StopReason::DurationReached) => "duration-reached",
            Some(StopReason::VolumeReached) => "volume-reached",
            Some(StopReason::TerminalStageExited) => "terminal-stage-exited",
            Some(StopReason::AllProcessesExited) => "all-processes-exited",
            Some(StopReason::Interrupt) => "interrupt",
            Some(StopReason::SinkError) => "sink-error",
            Some(StopReason::AcquisitionTimeout) => "acquisition-timeout",
            Some(StopReason::AmbiguousStageMatch) => "ambiguous-stage-match",
            Some(StopReason::PlatformExitedBeforeClient) => "platform-exited-before-client",
            Some(StopReason::EscapedPlatformClient) => "escaped-platform-client",
            Some(StopReason::PlatformDispatchFailed) => "platform-dispatch-failed",
            Some(StopReason::PlatformStartFailed) => "platform-start-failed",
            Some(StopReason::ProcessWatcherLost) => "process-watcher-lost",
            None => "source-exhausted",
        }
    }

    /// The `session.complete` event carrying the headline counters.
    ///
    /// `dropped` is fragcap's own drops (buffer plus sink), the discards inside
    /// the pipeline's conservation identity. The watch-time and out-of-window
    /// discards are the session's; they ride the event too so a `--json`
    /// consumer, which never sees the human summary, still reads every discard
    /// counter FR-021 requires be surfaced.
    pub fn complete_event(&self) -> Event {
        Event::SessionComplete {
            packets: self.packets_captured,
            attributed: self.packets_attributed,
            dropped: self.buffer_dropped.saturating_add(self.sink_dropped),
            watching_discarded: self.watching_discarded,
            discarded_out_of_window: self.discarded_out_of_window,
            scope_discarded: self.scope_discarded,
            scope_unresolved_discarded: self.scope_unresolved_discarded,
        }
    }

    /// Render the human-readable summary, one field per line, to `out`.
    pub fn render(&self, out: &mut String) {
        out.push_str("capture complete\n");
        if !self.acquired {
            out.push_str("  target:               never acquired\n");
        }
        push_field(out, "stop reason", self.stop_word());
        push_count(out, "packets captured", self.packets_captured);
        push_count(out, "retained", self.retained);
        push_count(out, "attributed", self.packets_attributed);
        push_count(out, "unattributed", self.packets_unattributed);
        push_count(out, "watching discarded", self.watching_discarded);
        push_count(out, "out of window", self.discarded_out_of_window);
        push_count(out, "out of scope", self.scope_discarded);
        push_count(out, "scope unresolved", self.scope_unresolved_discarded);
        push_count(out, "buffer dropped", self.buffer_dropped);
        push_count(out, "sink dropped", self.sink_dropped);
        // What actually reached the file, per image. Issue #184 was invisible in
        // this summary for want of exactly this: `attributed` counted packets
        // resolved to any process on the machine, which read as "attributed to
        // the game" while 91 percent of the file belonged to something else.
        if !self.written_by_image.is_empty() {
            out.push_str(
                "  written by process
",
            );
            for (image, count) in &self.written_by_image {
                out.push_str(&format!(
                    "    {image:<19} {count}
"
                ));
            }
        }
    }
}

fn push_field(out: &mut String, label: &str, value: &str) {
    out.push_str(&format!("  {label:<21} {value}\n"));
}

fn push_count(out: &mut String, label: &str, value: u64) {
    out.push_str(&format!("  {label:<21} {value}\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stop_word_is_stable_per_reason() {
        let mut s = CompletionSummary {
            acquired: true,
            stop_reason: Some(StopReason::DurationReached),
            ..CompletionSummary::default()
        };
        assert_eq!(s.stop_word(), "duration-reached");
        s.stop_reason = Some(StopReason::Interrupt);
        assert_eq!(s.stop_word(), "interrupt");
        s.stop_reason = None;
        assert_eq!(s.stop_word(), "source-exhausted");
    }

    #[test]
    fn the_complete_event_reports_fragcap_drops_only() {
        let s = CompletionSummary {
            packets_captured: 100,
            packets_attributed: 90,
            buffer_dropped: 2,
            sink_dropped: 1,
            watching_discarded: 7,
            discarded_out_of_window: 4,
            ..CompletionSummary::default()
        };
        assert_eq!(
            s.complete_event(),
            Event::SessionComplete {
                packets: 100,
                attributed: 90,
                dropped: 3,
                watching_discarded: 7,
                discarded_out_of_window: 4,
                scope_discarded: 0,
                scope_unresolved_discarded: 0,
            }
        );
    }

    #[test]
    fn an_unacquired_summary_says_so() {
        let s = CompletionSummary {
            acquired: false,
            stop_reason: Some(StopReason::AcquisitionTimeout),
            ..CompletionSummary::default()
        };
        let mut out = String::new();
        s.render(&mut out);
        assert!(out.contains("never acquired"));
        assert!(out.contains("acquisition-timeout"));
    }
}
