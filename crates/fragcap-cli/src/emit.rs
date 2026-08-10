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

use crate::events::Event;
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
}

impl<'w> Emitter<'w> {
    /// A production emitter, stamping events with the wall clock.
    pub fn new(err: &'w mut dyn Write, format: Format, verbosity: Verbosity) -> Self {
        Emitter {
            err,
            format,
            verbosity,
            clock: SystemTime::now,
        }
    }

    /// Emit a lifecycle event. A no-op in human mode, where progress lines carry
    /// the same information.
    pub fn event(&mut self, event: &Event) {
        if self.format == Format::Json {
            let line = event.render((self.clock)());
            let _ = writeln!(self.err, "{line}");
        }
    }

    /// A human progress line, suppressed by quiet and silent and by JSON mode.
    pub fn progress(&mut self, line: &str) {
        if self.format == Format::Human && self.verbosity == Verbosity::Normal {
            let _ = writeln!(self.err, "{line}");
        }
    }

    /// A warning, kept under quiet, suppressed under silent, and printed in
    /// either output shape.
    pub fn warn(&mut self, line: &str) {
        if self.verbosity != Verbosity::Silent {
            let _ = writeln!(self.err, "warning: {line}");
        }
    }

    /// An error. Never suppressed, in any mode or verbosity.
    pub fn error(&mut self, line: &str) {
        let _ = writeln!(self.err, "error: {line}");
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

    #[test]
    fn warnings_survive_quiet_but_not_silent_and_errors_survive_both() {
        assert!(emit(Format::Human, Verbosity::Quiet, |e| e.warn("w")).contains("warning: w"));
        assert!(emit(Format::Human, Verbosity::Silent, |e| e.warn("w")).is_empty());
        assert!(emit(Format::Human, Verbosity::Silent, |e| e.error("boom")).contains("error: boom"));
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
}
