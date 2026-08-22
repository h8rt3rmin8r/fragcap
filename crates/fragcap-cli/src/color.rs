// SPDX-License-Identifier: Apache-2.0

//! The shared terminal color palette (slice S066).
//!
//! `doctor` was the only surface that colorized output, so its `use_color()`
//! predicate and its Warn/Reset ANSI codes lived as private items in
//! `commands/doctor.rs` and `doctor/mod.rs`. The missing-install-root note in
//! `targets` (issue #167) needs the same warning color `doctor` already uses, and
//! duplicating the escape codes there would let the two surfaces drift on what a
//! warning looks like. This module is the one place both read from instead.

use std::io::IsTerminal;

/// Whether to colorize output: only when the process's real stdout is an
/// interactive terminal and `NO_COLOR` is unset. Callers pass a type-erased sink for
/// where the text goes, but the terminal test is always against
/// `std::io::stdout()`, matching how a terminal program decides.
pub(crate) fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
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
}
