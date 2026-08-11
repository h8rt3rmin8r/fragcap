// SPDX-License-Identifier: Apache-2.0

//! `steam`: enumerate installed titles and scaffold profiles (specification
//! section 16.3, roadmap slice S17).
//!
//! `steam profile <app_id>` discovers the Steam installation, resolves the
//! app_id to an installed title, scaffolds a profile skeleton, and prints it to
//! standard output (FR-019: command results go to stdout). The scaffold is a
//! validated starting point the operator reviews and redirects; discovery
//! warnings (a skipped malformed manifest, a duplicate app_id) go to standard
//! error so they never contaminate the profile on stdout.

use std::io::Write;

use fragcap::steam::{self, SteamError};

use crate::cli::{SteamArgs, SteamCommand};
use crate::exit::{CliError, Exit};

/// Run a `steam` subcommand, writing its output to `out`.
pub fn run(args: &SteamArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        SteamCommand::Profile { app_id } => profile(app_id, out),
    }
}

/// Scaffold a profile for one installed title.
fn profile(app_id: &str, out: &mut dyn Write) -> Result<Exit, CliError> {
    let installation = steam::discover().map_err(map_steam_error)?;

    for warning in &installation.warnings {
        eprintln!("warning: {warning}");
    }

    let Some(title) = installation.find(app_id) else {
        return Err(CliError::usage(
            SteamError::TitleNotFound {
                app_id: app_id.to_string(),
            }
            .to_string(),
        ));
    };

    let text = steam::scaffold(title).map_err(map_steam_error)?;
    let _ = write!(out, "{text}");
    Ok(Exit::SUCCESS)
}

/// Map a Steam error to the CLI exit contract.
///
/// A missing Steam installation or an unsupported platform is a configuration
/// problem (exit 2); a filesystem or launch failure is an expected runtime
/// failure (exit 1).
fn map_steam_error(error: SteamError) -> CliError {
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
