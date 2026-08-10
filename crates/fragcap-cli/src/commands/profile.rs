// SPDX-License-Identifier: Apache-2.0

//! `profile`: validate, list, and show, over the section 15 resolution and
//! validation machinery.
//!
//! Validation reports every diagnostic in one pass and exits 2 when the profile
//! is invalid, because an author working against a game update needs every
//! mistake before a capture wastes a session (the `From<ResolveError>` mapping
//! turns an invalid resolved candidate into an exit-2 usage error carrying the
//! whole diagnostic set). Listing reports the bundled and per-directory counts.
//! Show reports the resolved profile and its source, and a well-formed reference
//! that resolves to nothing exits 1.

use std::io::Write;
use std::path::PathBuf;

use fragcap::profile::{resolve, ResolveError};

use crate::cli::{ProfileArgs, ProfileCommand};
use crate::exit::{CliError, Exit};
use crate::paths;

/// Run a `profile` subcommand, writing its output to `out`.
pub fn run(args: &ProfileArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let search = paths::search_path(&args.profile_dir);
    let bundled = paths::bundled();

    match &args.command {
        ProfileCommand::Validate { reference } => {
            let resolved = resolve(reference, &search, &bundled)?;
            let _ = writeln!(out, "profile `{reference}` is valid ({})", resolved.source);
            let _ = writeln!(
                out,
                "  game: {} ({})",
                resolved.profile.game().name(),
                resolved.profile.game().id()
            );
            let _ = writeln!(out, "  stages: {}", resolved.profile.stages().len());
            Ok(Exit::SUCCESS)
        }
        ProfileCommand::List => {
            let _ = writeln!(out, "profiles");
            let _ = writeln!(out, "  bundled: {}", bundled.len());
            let mut user_total = 0usize;
            let mut dirs: Vec<PathBuf> = args.profile_dir.clone();
            if let Some(user) = &search.user {
                dirs.push(user.clone());
            }
            for dir in &dirs {
                let count = count_profiles(dir);
                user_total += count;
                let _ = writeln!(out, "  {}: {count}", dir.display());
            }
            let _ = writeln!(out, "  user total: {user_total}");
            Ok(Exit::SUCCESS)
        }
        ProfileCommand::Show { reference } => {
            match resolve(reference, &search, &bundled) {
                Ok(resolved) => {
                    let _ = writeln!(
                        out,
                        "profile `{reference}` resolved from {}",
                        resolved.source
                    );
                    let _ = writeln!(
                        out,
                        "  game: {} ({})",
                        resolved.profile.game().name(),
                        resolved.profile.game().id()
                    );
                    for stage in resolved.profile.stages() {
                        let _ =
                            writeln!(out, "  stage: {} ({:?})", stage.role(), stage.lifecycle());
                    }
                    Ok(Exit::SUCCESS)
                }
                // A well-formed reference that resolves to nothing is an expected
                // failure for `show` (exit 1), reporting every location searched.
                // A malformed reference is still a usage error (exit 2).
                Err(e @ ResolveError::NotFound { .. }) => {
                    let _ = writeln!(out, "{e}");
                    Err(CliError::failure(format!(
                        "no profile `{reference}` resolved"
                    )))
                }
                Err(e) => Err(e.into()),
            }
        }
    }
}

/// Count the `.toml` profiles directly in a directory, or zero when it cannot
/// be read.
fn count_profiles(dir: &std::path::Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
        .count()
}
