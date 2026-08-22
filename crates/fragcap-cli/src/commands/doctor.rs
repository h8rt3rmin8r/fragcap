// SPDX-License-Identifier: Apache-2.0

//! `doctor`: gather the environment, classify it, render, and optionally fix.
//!
//! Without `--fix` the command is a thin shell over the pure classifier: it
//! gathers real inputs (read-only, never installing), runs the section 26.3
//! classifiers, renders as aligned columns or as one JSON record per check, and
//! returns the report's exit code (1 if any check failed, else 0).
//!
//! With `--fix` (slice S056) an action layer runs above the same classifier: it
//! prints the same report, then offers to perform the remediations the report
//! named, under confirmation. The action layer is refused with `--json` and when
//! the session is not an interactive terminal, so it never acts into a pipe or a
//! machine-readable context.

use std::io::{IsTerminal, Write};

use crate::cli::DoctorArgs;
use crate::color::use_color;
use crate::doctor::action::Capabilities;
use crate::doctor::{checks, fix, probe};
use crate::exit::{CliError, Exit};

/// Run `doctor`, writing the report to `out`.
pub fn run(args: &DoctorArgs, json: bool, out: &mut dyn Write) -> Result<Exit, CliError> {
    // `--yes` only shapes the `--fix` action phase; alone it is a usage error
    // rather than a silent no-op.
    if args.yes && !args.fix {
        return Err(CliError::usage("--yes has no effect without --fix"));
    }

    if !args.fix {
        let report = checks::run(&probe::gather());
        let text = if json {
            // The machine-readable form is never colorized.
            report.render_json()
        } else {
            report.render_human_with(use_color())
        };
        let _ = write!(out, "{text}");
        return Ok(report.exit());
    }

    // The action layer is interactive and confirmation-driven; refuse it in any
    // machine-readable or non-interactive context before it can act (FR-007,
    // FR-008).
    if json {
        return Err(CliError::usage(
            "--fix is interactive and cannot be combined with --json",
        ));
    }
    if !std::io::stdout().is_terminal() {
        return Err(CliError::usage(
            "--fix needs an interactive terminal; stdout is not a terminal",
        ));
    }
    if !args.yes && !std::io::stdin().is_terminal() {
        return Err(CliError::usage(
            "--fix needs an interactive terminal to read confirmations; stdin is not a terminal. \
             Pass --yes to pre-confirm every action for unattended use",
        ));
    }

    let caps = Capabilities {
        net: cfg!(feature = "net"),
        // Relaunching elevated is a Windows operation; on other platforms the action
        // is not offered rather than offered only to fail.
        elevation: cfg!(windows),
    };
    Ok(fix::run_fix(caps, args.yes, use_color(), out))
}
