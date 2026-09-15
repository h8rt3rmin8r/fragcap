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
use crate::display::selected_stdout_width;
use crate::doctor::action::Capabilities;
use crate::doctor::probe::ProbeObserver;
use crate::doctor::progress::{begin_line, complete_line, waiting_line, ProbeName};
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
        return Ok(run_read_only(
            args,
            json,
            out,
            emitter,
            stdout_terminal,
            probe::gather_with,
        ));
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
    let human_width = selected_stdout_width(true);
    Ok(fix::run_fix(
        caps,
        args.yes,
        use_color(Stream::Stdout),
        human_width,
        out,
        emitter,
    ))
}

/// Inject gathering for command-contract tests without touching the host.
fn run_read_only(
    args: &DoctorArgs,
    json: bool,
    out: &mut dyn Write,
    emitter: &mut Emitter,
    stdout_terminal: bool,
    gather: impl FnOnce(&mut dyn ProbeObserver) -> crate::doctor::Inputs + Send,
) -> Exit {
    let enabled = !json && stdout_terminal && emitter.allows_progress();
    let width = selected_stdout_width(stdout_terminal);
    let color = !json && use_color(Stream::Stdout);
    let work = move |observer: &mut dyn ProbeObserver| {
        let report = checks::run(&gather(observer));
        let text = probe::observe(observer, ProbeName::ReportRendering, || {
            if json {
                report.render_json()
            } else {
                report.render_human_with_width(color, width)
            }
        });
        (text, report.exit())
    };
    let (text, exit) = if enabled {
        probe::run_observed(&mut DoctorProgress::new(Some(emitter), args.timings), work)
    } else {
        work(&mut probe::NoopObserver)
    };
    let _ = write!(out, "{text}");
    exit
}

struct DoctorProgress<'e, 'w> {
    emitter: Option<&'e mut Emitter<'w>>,
    timings: bool,
}

impl<'e, 'w> DoctorProgress<'e, 'w> {
    fn new(emitter: Option<&'e mut Emitter<'w>>, timings: bool) -> Self {
        DoctorProgress { emitter, timings }
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

    fn waiting(&mut self, probe: ProbeName, elapsed: std::time::Duration) {
        if let Some(emitter) = self.emitter.as_deref_mut() {
            emitter.progress(&waiting_line(probe, elapsed));
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

        let _ = run_read_only(&args, false, &mut stdout, &mut emitter, false, fake_gather);
        drop(emitter);
        assert!(stderr.is_empty(), "redirected stdout suppresses progress");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut emitter = Emitter::new(&mut stderr, Format::Json, Verbosity::Normal);

        let _ = run_read_only(&args, true, &mut stdout, &mut emitter, true, fake_gather);
        drop(emitter);
        assert!(stderr.is_empty(), "json suppresses progress");
    }

    fn fake_inputs() -> crate::doctor::Inputs {
        use crate::doctor::*;
        Inputs {
            fragcap_version: "controlled-version".into(),
            binary_path: None,
            catalog_db_path: None,
            catalog_db_present: false,
            local_db_path: None,
            local_db_present: false,
            os: "controlled-platform".into(),
            subsystem: Subsystem::Native,
            privilege: Privilege::NotElevated,
            npcap: None,
            etw_available: None,
            live_available: None,
            socket_table_available: None,
            interfaces: vec![],
            interface_error: None,
            extcap_installed: false,
            extcap_dir: None,
            extcap_system_installed: false,
            extcap_system_dir: None,
            target_entry_count: None,
            deep_capture: DeepCaptureInputs {
                session_dir: None,
                session_dir_present: false,
                proxy_backend: None,
                proxy_backend_error: None,
                ipv4_loopback: LoopbackReadiness::Undetermined,
                ipv6_loopback: LoopbackReadiness::Undetermined,
                analyzer_keylog_configured: false,
                ca: DeepCaptureCa::Unknown("controlled-indeterminate".into()),
                native_residue: residue::NativeResidueInventory::default(),
            },
        }
    }

    fn fake_gather(observer: &mut dyn ProbeObserver) -> crate::doctor::Inputs {
        observer.begin(ProbeName::DeepCaptureReadiness);
        observer.waiting(
            ProbeName::DeepCaptureReadiness,
            std::time::Duration::from_secs(2),
        );
        observer.complete(
            ProbeName::DeepCaptureReadiness,
            std::time::Duration::from_secs(2),
        );
        fake_inputs()
    }

    #[test]
    fn supplied_facts_keep_final_reports_and_exits_identical_in_all_output_modes() {
        let report = checks::run(&fake_inputs());
        for json in [false, true] {
            for terminal in [false, true] {
                for verbosity in [Verbosity::Normal, Verbosity::Quiet, Verbosity::Silent] {
                    let mut stdout = Vec::new();
                    let mut stderr = Vec::new();
                    let format = if json { Format::Json } else { Format::Human };
                    let mut emitter = Emitter::new(&mut stderr, format, verbosity);
                    let exit = run_read_only(
                        &doctor_args(true, false),
                        json,
                        &mut stdout,
                        &mut emitter,
                        terminal,
                        fake_gather,
                    );
                    drop(emitter);
                    let expected = if json {
                        report.render_json()
                    } else {
                        report.render_human_with_width(
                            use_color(Stream::Stdout),
                            selected_stdout_width(terminal),
                        )
                    };
                    assert_eq!(stdout, expected.as_bytes());
                    assert_eq!(exit, report.exit());
                    let enabled = !json && terminal && verbosity == Verbosity::Normal;
                    assert_eq!(!stderr.is_empty(), enabled);
                    if enabled {
                        let text = String::from_utf8(stderr).unwrap();
                        assert!(text.contains("Deep Capture readiness in 2000 ms"));
                        assert!(text.contains("report rendering"));
                    }
                }
            }
        }
    }

    #[test]
    fn failed_diagnostic_writes_preserve_original_report_and_exit() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut stdout = Vec::new();
        let mut broken = Broken;
        let mut emitter = Emitter::new(&mut broken, Format::Human, Verbosity::Normal);
        let exit = run_read_only(
            &doctor_args(true, false),
            false,
            &mut stdout,
            &mut emitter,
            true,
            fake_gather,
        );
        let report = checks::run(&fake_inputs());
        assert_eq!(exit, report.exit());
        assert_eq!(
            stdout,
            report
                .render_human_with_width(use_color(Stream::Stdout), selected_stdout_width(true))
                .as_bytes()
        );
    }
}
