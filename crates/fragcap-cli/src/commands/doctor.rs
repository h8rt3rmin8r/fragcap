// SPDX-License-Identifier: Apache-2.0

//! `doctor`: gather the environment, classify it, render, and exit.
//!
//! The command is a thin shell over the pure classifier. It gathers real inputs
//! from the machine (read-only, never installing), runs the section 26.3
//! classifiers, renders as aligned columns or as one JSON record per check, and
//! returns the report's exit code, which is 1 if any check failed and 0
//! otherwise.

use std::io::{IsTerminal, Write};

use crate::doctor::{checks, probe};
use crate::exit::{CliError, Exit};

/// Run `doctor`, writing the report to `out`.
pub fn run(json: bool, out: &mut dyn Write) -> Result<Exit, CliError> {
    let inputs = probe::gather();
    let report = checks::run(&inputs);
    let text = if json {
        // The machine-readable form is never colorized.
        report.render_json()
    } else {
        report.render_human_with(use_color())
    };
    let _ = write!(out, "{text}");
    Ok(report.exit())
}

/// Whether to colorize the human report: only when the process's real stdout is
/// an interactive terminal and `NO_COLOR` is unset. `out` is a type-erased sink
/// the caller supplies, so the terminal test is against `std::io::stdout()`
/// rather than the sink, matching how a terminal program decides.
fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}
