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
use crate::color::{use_color, Stream};
use crate::doctor::action::Capabilities;
use crate::doctor::probe::ProbeObserver;
use crate::doctor::progress::{begin_line, complete_line, ProbeName};
use crate::doctor::{checks, fix, probe};
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};

/// Run `doctor`, writing the report to `out`.
pub fn run(
    args: &DoctorArgs,
    json: bool,
    out: &mut dyn Write,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    run_with_terminal(args, json, out, emitter, std::io::stdout().is_terminal())
}

fn run_with_terminal(
    args: &DoctorArgs,
    json: bool,
    out: &mut dyn Write,
    emitter: &mut Emitter,
    stdout_terminal: bool,
) -> Result<Exit, CliError> {
    // `--yes` only shapes the `--fix` action phase; alone it is a usage error
    // rather than a silent no-op.
    if args.yes && !args.fix {
        return Err(CliError::usage("--yes has no effect without --fix"));
    }

    if !args.fix {
        let mut progress = DoctorProgress::new(
            if !json && stdout_terminal {
                Some(emitter)
            } else {
                None
            },
            args.timings,
        );
        let report = checks::run(&probe::gather_with(&mut progress));
        let text = if json {
            // The machine-readable form is never colorized.
            progress.render_report(|| report.render_json())
        } else {
            progress.render_report(|| report.render_human_with(use_color(Stream::Stdout)))
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
    Ok(fix::run_fix(
        caps,
        args.yes,
        use_color(Stream::Stdout),
        out,
        emitter,
    ))
}

struct DoctorProgress<'e, 'w> {
    emitter: Option<&'e mut Emitter<'w>>,
    timings: bool,
}

impl<'e, 'w> DoctorProgress<'e, 'w> {
    fn new(emitter: Option<&'e mut Emitter<'w>>, timings: bool) -> Self {
        DoctorProgress { emitter, timings }
    }

    fn render_report(&mut self, render: impl FnOnce() -> String) -> String {
        self.begin(ProbeName::ReportRendering);
        let started = std::time::Instant::now();
        let text = render();
        self.complete(ProbeName::ReportRendering, started.elapsed());
        text
    }
}

impl probe::ProbeObserver for DoctorProgress<'_, '_> {
    fn begin(&mut self, probe: ProbeName) {
        if let Some(emitter) = self.emitter.as_deref_mut() {
            emitter.progress(&begin_line(probe));
        }
    }

    fn complete(&mut self, probe: ProbeName, elapsed: std::time::Duration) {
        if let Some(emitter) = self.emitter.as_deref_mut() {
            emitter.progress(&complete_line(probe, elapsed, self.timings));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::{Format, Verbosity};

    fn doctor_args(timings: bool, fix: bool) -> DoctorArgs {
        DoctorArgs {
            fix,
            yes: false,
            timings,
        }
    }

    #[test]
    fn interactive_progress_writes_named_probe_lines() {
        let mut stderr = Vec::new();
        let mut emitter = Emitter::new(&mut stderr, Format::Human, Verbosity::Normal);
        let mut progress = DoctorProgress::new(Some(&mut emitter), false);

        progress.begin(ProbeName::CaptureDriverInterfaces);
        progress.complete(
            ProbeName::CaptureDriverInterfaces,
            std::time::Duration::from_millis(12),
        );

        let text = String::from_utf8(stderr).expect("stderr is UTF-8");
        assert!(text.contains("doctor: checking capture driver and interfaces..."));
        assert!(text.contains("doctor: checked capture driver and interfaces"));
        assert!(
            !text.contains("12 ms"),
            "timings are hidden by default: {text}"
        );
    }

    #[test]
    fn timings_include_elapsed_milliseconds_when_enabled() {
        let mut stderr = Vec::new();
        let mut emitter = Emitter::new(&mut stderr, Format::Human, Verbosity::Normal);
        let mut progress = DoctorProgress::new(Some(&mut emitter), true);

        progress.complete(
            ProbeName::ProcessEventTracing,
            std::time::Duration::from_millis(12),
        );

        let text = String::from_utf8(stderr).expect("stderr is UTF-8");
        assert!(text.contains("doctor: checked process event tracing in 12 ms"));
    }

    #[test]
    fn progress_is_suppressed_when_not_enabled_or_not_verbose() {
        let mut stderr = Vec::new();
        let emitter = Emitter::new(&mut stderr, Format::Human, Verbosity::Normal);
        let mut progress = DoctorProgress::new(None, true);
        progress.begin(ProbeName::Identity);
        progress.complete(ProbeName::Identity, std::time::Duration::from_millis(1));
        drop(emitter);
        assert!(stderr.is_empty());

        let mut stderr = Vec::new();
        let mut emitter = Emitter::new(&mut stderr, Format::Human, Verbosity::Quiet);
        let mut progress = DoctorProgress::new(Some(&mut emitter), true);
        progress.begin(ProbeName::Identity);
        progress.complete(ProbeName::Identity, std::time::Duration::from_millis(1));
        drop(emitter);
        assert!(stderr.is_empty());
    }

    #[test]
    fn command_progress_enablement_respects_terminal_and_json() {
        let args = doctor_args(false, false);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut emitter = Emitter::new(&mut stderr, Format::Human, Verbosity::Normal);

        let _ = run_with_terminal(&args, false, &mut stdout, &mut emitter, false);
        drop(emitter);
        assert!(stderr.is_empty(), "redirected stdout suppresses progress");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut emitter = Emitter::new(&mut stderr, Format::Json, Verbosity::Normal);

        let _ = run_with_terminal(&args, true, &mut stdout, &mut emitter, true);
        drop(emitter);
        assert!(stderr.is_empty(), "json suppresses progress");
    }
}
