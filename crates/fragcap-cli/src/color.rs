// SPDX-License-Identifier: Apache-2.0

//! The shared terminal color palette (slice S066).
//!
//! `doctor` was the only surface that colorized output, so its `use_color()`
//! predicate and its Warn/Reset ANSI codes lived as private items in
//! `commands/doctor.rs` and `doctor/mod.rs`. The missing-install-root note in
//! `targets` (issue #167) needs the same warning color `doctor` already uses, and
//! duplicating the escape codes there would let the two surfaces drift on what a
//! warning looks like. This module is the one place both read from instead.
//!
//! Slice S069 added the `Stream` parameter: `doctor` and `targets` render to
//! stdout, but the live capture status display renders to stderr and must
//! never be influenced by, or reported through, what stdout happens to be
//! (stdout may be piped capture bytes under `--mode stream --out -`). A single
//! predicate hard-coded to `std::io::stdout()` would silently gate the wrong
//! stream for that caller, so every caller now states which stream it means.

use std::io::IsTerminal;

/// Which stream a caller wants the terminal/color test run against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stream {
    /// `doctor` and `targets` render here.
    Stdout,
    /// The live capture status display (slice S069) renders here, and must
    /// never be gated on stdout's terminal-ness.
    Stderr,
}

/// Whether to colorize output on `stream`: only when that stream is an
/// interactive terminal and `NO_COLOR` is unset.
pub(crate) fn use_color(stream: Stream) -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    match stream {
        Stream::Stdout => std::io::stdout().is_terminal(),
        Stream::Stderr => std::io::stderr().is_terminal(),
    }
}

/// The warning color: doctor's `Status::Warn`.
pub(crate) const WARN: &str = "\x1b[33m";
/// Reset all ANSI styling.
pub(crate) const RESET: &str = "\x1b[0m";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shared_palette_matches_doctors_values_exactly() {
        // Doctor's own escape codes, unchanged since before the extraction
        // (crates/fragcap-cli/src/doctor/mod.rs's Status::Warn arm and ANSI_RESET).
        // If either drifts, `targets` and `doctor` would print two different
        // warning colors, which this asserts against.
        assert_eq!(WARN, "\x1b[33m");
        assert_eq!(RESET, "\x1b[0m");
    }

    // S069 T010. `NO_COLOR` short-circuits both streams before either
    // terminal check runs; the terminal check itself is not independently
    // testable in an automated run (this process's real stdout/stderr are
    // not swappable here), matching this module's existing test posture.
    #[test]
    fn no_color_disables_both_streams_regardless_of_terminal_state() {
        // SAFETY: this test does not run concurrently with any other test in
        // this crate that reads or writes `NO_COLOR` (grep confirms this is
        // the only file that names it), and it restores the prior value
        // before returning.
        let previous = std::env::var_os("NO_COLOR");
        unsafe { std::env::set_var("NO_COLOR", "1") };

        assert!(!use_color(Stream::Stdout));
        assert!(!use_color(Stream::Stderr));

        match previous {
            Some(v) => unsafe { std::env::set_var("NO_COLOR", v) },
            None => unsafe { std::env::remove_var("NO_COLOR") },
        }
    }
}
