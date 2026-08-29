// SPDX-License-Identifier: Apache-2.0

//! The emitter that routes progress, events, warnings, and errors to standard
//! error, honoring quiet, silent, and the structured-output option.
//!
//! Specification section 17.6 and FR-018 through FR-020. The stream rule is
//! simple here: every diagnostic and progress line goes to standard error, so
//! that when a sink writes capture data to standard output nothing the tool says
//! contaminates it. Capture data goes to sinks; this emitter never touches the
//! sink stream.
//!
//! The two output shapes are exclusive. In human mode the emitter prints
//! progress and the completion summary; in JSON mode it prints the section 17.5
//! event stream and nothing human. Errors are never suppressed in either shape.

use std::io::Write;
use std::time::SystemTime;

use fragcap::write_json_string;

use crate::events::{rfc3339_utc, Event};
use crate::output::CompletionSummary;

/// The output shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// Human progress and a rendered completion summary.
    Human,
    /// The newline-delimited structured event stream.
    Json,
}

/// How much the emitter says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verbosity {
    /// Everything.
    Normal,
    /// Suppress progress; keep warnings and errors.
    Quiet,
    /// Suppress everything except errors.
    Silent,
}

impl Verbosity {
    /// Derive verbosity from the two flags. `--silent` outranks `--quiet`.
    pub fn from_flags(quiet: bool, silent: bool) -> Verbosity {
        if silent {
            Verbosity::Silent
        } else if quiet {
            Verbosity::Quiet
        } else {
            Verbosity::Normal
        }
    }
}

/// Routes progress, events, and diagnostics to a single writer (standard error
/// in production, a captured buffer in a test).
pub struct Emitter<'w> {
    err: &'w mut dyn Write,
    format: Format,
    verbosity: Verbosity,
    /// Injected so a test can produce a deterministic event stream; the real
    /// entry passes `SystemTime::now`.
    clock: fn() -> SystemTime,
    captured_events: Option<Vec<String>>,
    /// How many progress lines [`Emitter::progress`] has actually written
    /// (not merely been asked to). A caller (the S069 non-terminal
    /// heartbeat) reads this before and after a span of calls that may or
    /// may not have produced real output, to decide whether the heartbeat's
    /// own interval should reset, without `Emitter` knowing anything about
    /// heartbeats itself.
    ///
    /// `etw`+`windows`-gated along with its one reader,
    /// [`Emitter::progress_written`]; see `lib.rs`'s note on `mod
    /// live_status` for why an unread field is otherwise dead code on a
    /// platform where that reader's caller does not exist.
    #[cfg(all(feature = "etw", windows))]
    progress_written: u64,
}

impl<'w> Emitter<'w> {
    /// A production emitter, stamping events with the wall clock.
    pub fn new(err: &'w mut dyn Write, format: Format, verbosity: Verbosity) -> Self {
        Emitter {
            err,
            format,
            verbosity,
            clock: SystemTime::now,
            captured_events: None,
            #[cfg(all(feature = "etw", windows))]
            progress_written: 0,
        }
    }

    /// Emit a lifecycle event. A no-op in human mode, where progress lines carry
    /// the same information.
    pub fn event(&mut self, event: &Event) {
        if self.format == Format::Json || self.captured_events.is_some() {
            let line = event.render((self.clock)());
            if let Some(events) = &mut self.captured_events {
                events.push(line.clone());
            }
            if self.format == Format::Json {
                let _ = writeln!(self.err, "{line}");
            }
        }
    }

    /// Begin copying structured lifecycle events for a session sidecar.
    pub fn begin_event_capture(&mut self) {
        self.captured_events = Some(Vec::new());
    }

    /// Finish event copying and return the captured structured records.
    pub fn take_captured_events(&mut self) -> Vec<String> {
        self.captured_events.take().unwrap_or_default()
    }

    /// A human progress line, suppressed by quiet and silent and by JSON mode.
    pub fn progress(&mut self, line: &str) {
        if self.format == Format::Human && self.verbosity == Verbosity::Normal {
            let _ = writeln!(self.err, "{line}");
            #[cfg(all(feature = "etw", windows))]
            {
                self.progress_written += 1;
            }
        }
    }

    /// Write required human-facing text even when ordinary progress is quiet.
    /// JSON mode remains exclusive and receives the corresponding event instead.
    pub fn required_human(&mut self, text: &str) {
        if self.format == Format::Human {
            let _ = write!(self.err, "{text}");
        }
    }

    /// Flush the diagnostic stream before reading an interactive answer.
    pub fn flush(&mut self) {
        let _ = self.err.flush();
    }

    /// Whether this emitter is producing the structured event stream.
    pub fn is_json(&self) -> bool {
        self.format == Format::Json
    }

    /// How many progress lines have actually been written so far. See the
    /// field's own documentation for why this exists.
    ///
    /// `etw`+`windows`-gated with its one caller; see `lib.rs`'s note on
    /// `mod live_status`.
    #[cfg(all(feature = "etw", windows))]
    pub fn progress_written(&self) -> u64 {
        self.progress_written
    }

    /// The current output format, for a caller (the live capture status
    /// display, slice S069) that must pick among three mutually exclusive
    /// behaviors per tick rather than calling a single gated method.
    ///
    /// `etw`+`windows`-gated with its one caller; see `lib.rs`'s note on
    /// `mod live_status`.
    #[cfg(all(feature = "etw", windows))]
    pub fn format(&self) -> Format {
        self.format
    }

    /// The current verbosity, for the same reason as [`Emitter::format`].
    #[cfg(all(feature = "etw", windows))]
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Raw bytes, with no appended newline and no `format`/`verbosity` gate
    /// of its own: the caller (the live status redraw, slice S069) has
    /// already decided, via [`Emitter::format`] and [`Emitter::verbosity`],
    /// that this is the right tick to write. Suppressed only by `--silent`,
    /// matching every other non-error output; a caller that also wants the
    /// `--quiet` gate checks `verbosity() == Verbosity::Normal` itself before
    /// calling, the same check `progress` already makes.
    ///
    /// `etw`+`windows`-gated with its one caller; see `lib.rs`'s note on
    /// `mod live_status`.
    #[cfg(all(feature = "etw", windows))]
    pub fn live_write(&mut self, text: &str) {
        if self.verbosity != Verbosity::Silent {
            let _ = write!(self.err, "{text}");
        }
    }

    /// A warning, kept under quiet, suppressed under silent. In human mode a
    /// `warning:` line; in JSON mode a `warning` NDJSON record, so a `--json`
    /// consumer reading standard error line by line never meets a line that is
    /// not JSON.
    pub fn warn(&mut self, line: &str) {
        if self.verbosity != Verbosity::Silent {
            self.diagnostic("warning", line);
        }
    }

    /// An error. Never suppressed, in any mode or verbosity. In human mode an
    /// `error:` line; in JSON mode an `error` NDJSON record.
    pub fn error(&mut self, line: &str) {
        self.diagnostic("error", line);
    }

    /// Write one diagnostic, shaped by the output format. `kind` is the human
    /// label prefix and the JSON `event` discriminator; the two agree by
    /// construction so a reader keys on the same word either way.
    fn diagnostic(&mut self, kind: &str, message: &str) {
        match self.format {
            Format::Human => {
                let _ = writeln!(self.err, "{kind}: {message}");
            }
            Format::Json => {
                let mut record = String::from("{\"ts\":");
                write_json_string(&rfc3339_utc((self.clock)()), &mut record);
                record.push_str(",\"event\":");
                write_json_string(kind, &mut record);
                record.push_str(",\"message\":");
                write_json_string(message, &mut record);
                record.push('}');
                let _ = writeln!(self.err, "{record}");
            }
        }
    }

    /// The completion summary. In human mode it is rendered unless silent; in
    /// JSON mode the `session.complete` event carries the counters instead.
    pub fn summary(&mut self, summary: &CompletionSummary) {
        match self.format {
            Format::Json => self.event(&summary.complete_event()),
            Format::Human => {
                if self.verbosity != Verbosity::Silent {
                    let mut text = String::new();
                    summary.render(&mut text);
                    let _ = write!(self.err, "{text}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emit(format: Format, verbosity: Verbosity, f: impl FnOnce(&mut Emitter)) -> String {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut e = Emitter {
                err: &mut buf,
                format,
                verbosity,
                clock: || std::time::UNIX_EPOCH,
                captured_events: None,
                #[cfg(all(feature = "etw", windows))]
                progress_written: 0,
            };
            f(&mut e);
        }
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn progress_is_suppressed_by_quiet_and_silent() {
        assert!(emit(Format::Human, Verbosity::Normal, |e| e.progress("armed")).contains("armed"));
        assert!(emit(Format::Human, Verbosity::Quiet, |e| e.progress("armed")).is_empty());
        assert!(emit(Format::Human, Verbosity::Silent, |e| e.progress("armed")).is_empty());
    }

    // `progress_written` and its reader are `etw`+`windows`-gated (see the
    // field's own doc comment), so this test is too.
    #[cfg(all(feature = "etw", windows))]
    #[test]
    fn progress_written_only_counts_lines_that_actually_wrote() {
        let mut buf: Vec<u8> = Vec::new();
        let mut e = Emitter {
            err: &mut buf,
            format: Format::Human,
            verbosity: Verbosity::Normal,
            clock: || std::time::UNIX_EPOCH,
            captured_events: None,
            progress_written: 0,
        };
        assert_eq!(e.progress_written(), 0);
        e.progress("a");
        e.progress("b");
        assert_eq!(e.progress_written(), 2);

        let mut buf: Vec<u8> = Vec::new();
        let mut suppressed = Emitter {
            err: &mut buf,
            format: Format::Human,
            verbosity: Verbosity::Quiet,
            clock: || std::time::UNIX_EPOCH,
            captured_events: None,
            progress_written: 0,
        };
        suppressed.progress("never written");
        assert_eq!(suppressed.progress_written(), 0);
    }

    #[test]
    fn warnings_survive_quiet_but_not_silent_and_errors_survive_both() {
        assert!(emit(Format::Human, Verbosity::Quiet, |e| e.warn("w")).contains("warning: w"));
        assert!(emit(Format::Human, Verbosity::Silent, |e| e.warn("w")).is_empty());
        assert!(emit(Format::Human, Verbosity::Silent, |e| e.error("boom")).contains("error: boom"));
    }

    #[test]
    fn json_diagnostics_are_ndjson_records_honoring_verbosity() {
        // A warning in JSON mode is a record, not a `warning:` line, and it
        // carries the timestamp, the event discriminator, and the escaped
        // message.
        let warn = emit(Format::Json, Verbosity::Normal, |e| {
            e.warn("a \"quoted\" note")
        });
        assert!(!warn.contains("warning: "), "no human prefix in JSON mode");
        assert!(warn.contains("\"event\":\"warning\""));
        assert!(warn.contains("\"ts\":\"1970-01-01T00:00:00Z\""));
        assert!(warn.contains("\"message\":\"a \\\"quoted\\\" note\""));

        // Verbosity still governs: silent drops the warning but never the error.
        assert!(emit(Format::Json, Verbosity::Silent, |e| e.warn("w")).is_empty());
        let err = emit(Format::Json, Verbosity::Silent, |e| e.error("boom"));
        assert!(err.contains("\"event\":\"error\""));
        assert!(err.contains("\"message\":\"boom\""));
    }

    #[test]
    fn events_are_json_only_and_progress_is_human_only() {
        let json = emit(Format::Json, Verbosity::Normal, |e| {
            e.progress("armed");
            e.event(&Event::FilterNarrowed { endpoints: 2 });
        });
        assert!(!json.contains("armed"), "no human progress in JSON mode");
        assert!(json.contains("\"event\":\"filter.narrowed\""));
    }

    #[test]
    fn event_capture_copies_structured_events_without_changing_human_output() {
        let mut output = Vec::new();
        let mut emitter = Emitter::new(&mut output, Format::Human, Verbosity::Normal);
        emitter.begin_event_capture();
        emitter.event(&Event::StageExited {
            role: "client".to_string(),
            pid: 7,
        });
        let events = emitter.take_captured_events();
        assert!(output.is_empty());
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("\"event\":\"stage.exited\""));
    }
}
