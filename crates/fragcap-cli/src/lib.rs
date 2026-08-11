// SPDX-License-Identifier: Apache-2.0

//! The fragcap command surface as a library.
//!
//! The library is the product here as much as anywhere else in the workspace:
//! [`run`] is the testable entry, so the whole seven-command surface is driven
//! without spawning a process. The binary in `main.rs` is a shim that exits with
//! [`run`]'s code.
//!
//! Specification section 17. The surface is `run`, `tap`, `doctor`, and
//! `profile`, with `replay`, `steam`, and `extcap` registered as stubs so the
//! help foreshadows the whole tool. Every command returns an exit code it chose
//! or a [`CliError`] the library maps to the 0/1/2 contract at one site.
//!
//! Output routing follows specification FR-019: command results (`doctor`,
//! `profile`) go to standard output, while a capture's progress, completion
//! summary, and structured events go to standard error, so a sink that writes
//! capture data to standard output is never contaminated by what the tool says.

pub mod doctor;

mod args;
mod assemble;
mod cli;
mod commands;
mod emit;
mod events;
mod exit;
mod orchestrator;
mod output;
mod paths;

use std::ffi::OsString;
use std::io::{self, Write};

use clap::Parser;

use cli::{Cli, Command};
use commands::stub::Stub;
use emit::{Emitter, Format, Verbosity};

pub use exit::{CliError, Exit};

/// Run the command surface against `args`, returning the process exit.
///
/// The production entry: command results go to standard output and a capture's
/// diagnostics to standard error. Tests reach [`run_with`] to capture both.
pub fn run<I>(args: I) -> Exit
where
    I: IntoIterator<Item = OsString>,
{
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut out = stdout.lock();
    let mut err = stderr.lock();
    run_with(args, &mut out, &mut err)
}

/// Run the command surface with explicit output and error streams.
///
/// `out` carries command results (the `doctor` report, `profile` output); `err`
/// carries a capture's progress, completion summary, structured events, and all
/// warnings and errors. Separating them is what makes the whole surface, the
/// event stream and the summary included, assertable from a tier-1 test.
pub fn run_with<I>(args: I, out: &mut dyn Write, err: &mut dyn Write) -> Exit
where
    I: IntoIterator<Item = OsString>,
{
    let args = route_extcap(args.into_iter().collect());
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // clap prints help and version as a success on standard output, and
            // a genuine parse error as a usage failure on standard error.
            let rendered = e.to_string();
            if e.use_stderr() {
                let _ = write!(err, "{rendered}");
                return Exit::USAGE;
            }
            let _ = write!(out, "{rendered}");
            return Exit::SUCCESS;
        }
    };

    let format = if cli.json {
        Format::Json
    } else {
        Format::Human
    };
    let verbosity = Verbosity::from_flags(cli.quiet, cli.silent);
    let json = cli.json;

    let mut emitter = Emitter::new(err, format, verbosity);
    let result = dispatch(cli.command, json, out, &mut emitter);

    match result {
        Ok(exit) => exit,
        Err(error) => {
            emitter.error(error.message());
            error.exit()
        }
    }
}

/// Route a direct extcap protocol invocation to the `extcap` subcommand.
///
/// When an analyzer discovers the copied binary it invokes it directly, with no
/// subcommand: `fragcap --extcap-interfaces`, `fragcap --capture --fifo <path>
/// ...`, and so on (the Wireshark extcap protocol). The command surface is
/// otherwise subcommand-first, so those invocations would be rejected by the
/// parser before the `extcap` command ran, and no interface would ever be
/// discovered. When the invocation leads with an extcap protocol flag rather than
/// a subcommand, insert the `extcap` subcommand so a real analyzer reaches the
/// implementation. An operator's explicit `fragcap extcap ...` is unaffected.
fn route_extcap(mut args: Vec<OsString>) -> Vec<OsString> {
    // args[0] is the program name; the first real token decides.
    let first = args.get(1).map(|s| s.to_string_lossy());
    let is_subcommand = matches!(
        first.as_deref(),
        Some(
            "run"
                | "tap"
                | "replay"
                | "profile"
                | "steam"
                | "doctor"
                | "extcap"
                | "help"
                | "-h"
                | "--help"
                | "-V"
                | "--version"
        )
    );
    if is_subcommand {
        return args;
    }
    let leads_with_extcap_flag = args.iter().skip(1).any(|a| {
        let s = a.to_string_lossy();
        s.starts_with("--extcap") || s == "--capture" || s == "--fifo"
    });
    if leads_with_extcap_flag {
        args.insert(1, OsString::from("extcap"));
    }
    args
}

/// Dispatch a parsed command to its implementation.
fn dispatch(
    command: Command,
    json: bool,
    out: &mut dyn Write,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    match command {
        Command::Run(args) => commands::run::run(&args, emitter),
        Command::Tap(args) => commands::tap::run(&args, emitter),
        Command::Doctor(_) => commands::doctor::run(json, out),
        Command::Profile(args) => commands::profile::run(&args, out),
        Command::Replay(_) => commands::stub::run(Stub::Replay),
        Command::Steam(args) => commands::steam::run(&args, out, emitter),
        Command::Extcap(args) => commands::extcap::run(&args, out, emitter),
    }
}
