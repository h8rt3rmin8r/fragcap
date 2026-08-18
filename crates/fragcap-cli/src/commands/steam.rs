// SPDX-License-Identifier: Apache-2.0

//! `steam`: Steam-specific inspection (specification section 16.3).
//!
//! `steam list` discovers the Steam installation and prints the installed titles
//! it can enumerate, one per line (FR-019: command results go to stdout).
//! Enumeration diagnostics (a skipped malformed manifest, a duplicate app id) go to
//! standard error so they never contaminate the listing on stdout.
//!
//! Registering an installed title as a capture target is `targets add --steam
//! <app_id>` (it lands in the user store); the retired `steam profile <app_id>`
//! scaffolding is gone.

use std::io::Write;

use fragcap::steam::{self, SteamError};

use crate::cli::{SteamArgs, SteamCommand};
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};

/// Run a `steam` subcommand, writing its result to `out` and any diagnostics
/// through `emitter`.
pub fn run(args: &SteamArgs, out: &mut dyn Write, emitter: &mut Emitter) -> Result<Exit, CliError> {
    match &args.command {
        SteamCommand::List => list(out, emitter),
    }
}

/// List the installed Steam titles this machine can enumerate.
fn list(out: &mut dyn Write, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let installation = steam::discover().map_err(map_steam_error)?;

    // Enumeration diagnostics go through the emitter so they honor the configured
    // writer, verbosity, and format, and never contaminate the listing on stdout.
    for warning in &installation.warnings {
        emitter.warn(warning);
    }

    if installation.titles.is_empty() {
        let _ = writeln!(out, "no installed titles enumerated");
        return Ok(Exit::SUCCESS);
    }
    for title in &installation.titles {
        let _ = writeln!(out, "{}\t{}", title.app_id, title.name);
    }
    Ok(Exit::SUCCESS)
}

/// Map a Steam error to the CLI exit contract.
///
/// A missing Steam installation or an unsupported platform is a configuration
/// problem (exit 2); a filesystem failure is an expected runtime failure (exit 1).
/// Shared with `targets add --steam` so the two Steam entry points classify the
/// same error the same way.
pub(crate) fn map_steam_error(error: SteamError) -> CliError {
    match error {
        SteamError::NotInstalled
        | SteamError::UnsupportedPlatform
        | SteamError::TitleNotFound { .. } => CliError::usage(error.to_string()),
        SteamError::Io { .. }
        | SteamError::NoExecutables { .. }
        | SteamError::Scaffold(_)
        | SteamError::LaunchFailed { .. } => CliError::failure(error.to_string()),
    }
}
