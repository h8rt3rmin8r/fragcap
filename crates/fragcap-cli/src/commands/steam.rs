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
use std::path::Path;

use fragcap::profile::signature::SignatureSet;
use fragcap::steam::{self, SteamError};
use fragcap::targets::Store;

use crate::cli::{SteamArgs, SteamCommand};
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};

/// Run a `steam` subcommand, writing its result to `out` and any diagnostics
/// through `emitter`.
pub fn run(args: &SteamArgs, out: &mut dyn Write, emitter: &mut Emitter) -> Result<Exit, CliError> {
    match &args.command {
        SteamCommand::Profile { app_id, catalog_db } => {
            profile(app_id, catalog_db.as_deref(), out, emitter)
        }
    }
}

/// Detect the technologies in an install directory from the catalog's signature
/// table, or an empty set when no catalog is given (slice S053). A catalog that
/// cannot be opened or scanned is a surfaced failure, not a silent empty result.
fn detect_technologies(
    catalog_db: Option<&Path>,
    install_dir: &Path,
) -> Result<Vec<fragcap::profile::DetectionFinding>, CliError> {
    let Some(catalog_db) = catalog_db else {
        return Ok(Vec::new());
    };
    let store = Store::open(catalog_db).map_err(|e| CliError::failure(e.to_string()))?;
    let signatures = store
        .load_signatures()
        .map_err(|e| CliError::failure(e.to_string()))?;
    let set = SignatureSet::compile(&signatures);
    let outcome = set
        .detect(install_dir)
        .map_err(|e| CliError::failure(e.to_string()))?;
    Ok(outcome.findings)
}

/// Scaffold a profile for one installed title.
fn profile(
    app_id: &str,
    catalog_db: Option<&Path>,
    out: &mut dyn Write,
    emitter: &mut Emitter,
) -> Result<Exit, CliError> {
    let installation = steam::discover().map_err(map_steam_error)?;

    // Discovery diagnostics (a skipped malformed manifest, a duplicate app_id, an
    // unreadable library) go through the emitter so they honor the configured
    // writer, verbosity, and output format, and never contaminate the profile on
    // stdout (Codex review of PR #31).
    for warning in &installation.warnings {
        emitter.warn(warning);
    }

    let Some(title) = installation.find(app_id) else {
        return Err(CliError::usage(
            SteamError::TitleNotFound {
                app_id: app_id.to_string(),
            }
            .to_string(),
        ));
    };

    let technologies = detect_technologies(catalog_db, &title.install_dir)?;
    let text = steam::scaffold(title, &technologies).map_err(map_steam_error)?;
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
