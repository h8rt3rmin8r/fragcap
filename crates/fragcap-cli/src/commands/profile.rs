// SPDX-License-Identifier: Apache-2.0

//! `profile`: validate, list, and show, over the section 15 resolution and
//! validation machinery.
//!
//! Validation reports every diagnostic in one pass. An invalid profile is a
//! configuration error (exit 2); a reference that resolves to nothing is an
//! expected failure (exit 1), and the two agree across `validate` and `show`
//! because both defer to the `From<ResolveError>` mapping. Under `--json` every
//! subcommand emits the section 17.5 structured stream on standard output: one
//! `diagnostic` record per problem and a terminal `summary`, so a consumer never
//! re-parses a human string. Listing reports the bundled and per-directory
//! counts.

use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use fragcap::profile::{resolve, Diagnostic, LoadError, ProfileSource, ResolveError, Resolved};
use fragcap::write_json_string;

use crate::cli::{ProfileArgs, ProfileCommand};
use crate::events::rfc3339_utc;
use crate::exit::{CliError, Exit};
use crate::paths;

/// Run a `profile` subcommand, writing its output to `out`. Under `json` the
/// output is the section 17.5 structured event stream rather than human text.
pub fn run(args: &ProfileArgs, json: bool, out: &mut dyn Write) -> Result<Exit, CliError> {
    let search = paths::search_path(&args.profile_dir);
    let bundled = paths::bundled();

    match &args.command {
        ProfileCommand::Validate { reference } => {
            if json {
                return Ok(validate_json(resolve(reference, &search, &bundled), out));
            }
            let resolved = resolve(reference, &search, &bundled)?;
            // When the reference is itself an explicit path, `ProfileSource`
            // repeats it as "path <ref>", so a trailing source suffix would just
            // duplicate the backticked reference; omit it in that case.
            if matches!(resolved.source, ProfileSource::ExplicitPath(_)) {
                let _ = writeln!(out, "profile `{reference}` is valid");
            } else {
                let _ = writeln!(out, "profile `{reference}` is valid ({})", resolved.source);
            }
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
            let mut dirs: Vec<PathBuf> = args.profile_dir.clone();
            if let Some(user) = &search.user {
                dirs.push(user.clone());
            }
            let counts: Vec<(PathBuf, usize)> = dirs
                .iter()
                .map(|d| (d.clone(), count_profiles(d)))
                .collect();
            let user_total: usize = counts.iter().map(|(_, c)| c).sum();

            if json {
                emit_profiles(out, bundled.len(), user_total, &counts);
                return Ok(Exit::SUCCESS);
            }
            let _ = writeln!(out, "profiles");
            let _ = writeln!(out, "  bundled: {}", bundled.len());
            for (dir, count) in &counts {
                let _ = writeln!(out, "  {}: {count}", dir.display());
            }
            let _ = writeln!(out, "  user total: {user_total}");
            Ok(Exit::SUCCESS)
        }
        ProfileCommand::Show { reference } => {
            if json {
                return Ok(show_json(resolve(reference, &search, &bundled), out));
            }
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

/// Emit `validate` results as the section 17.5 stream: one `diagnostic` record
/// per problem, then a terminal `summary`. An invalid profile is exit 2; a
/// reference that resolves to nothing (or cannot be read) is mapped through the
/// shared exit contract so `--json` and human mode agree on the code.
fn validate_json(result: Result<Resolved, ResolveError>, out: &mut dyn Write) -> Exit {
    match result {
        Ok(_) => {
            emit_summary(out, 0, true);
            Exit::SUCCESS
        }
        Err(ResolveError::Load {
            source: LoadError::Invalid(diags),
            ..
        }) => {
            for d in diags.iter() {
                emit_diagnostic(out, d);
            }
            emit_summary(out, diags.len(), false);
            Exit::USAGE
        }
        Err(e) => {
            // Not-found / invalid-reference / read error: no per-diagnostic
            // structure, so one error record plus a summary, at the mapped code.
            let cli = CliError::from(e);
            emit_error(out, cli.message());
            emit_summary(out, 0, false);
            cli.exit()
        }
    }
}

/// Emit `show` results as a single `profile` record, or an `error` record at the
/// mapped exit code when the reference resolves to nothing.
fn show_json(result: Result<Resolved, ResolveError>, out: &mut dyn Write) -> Exit {
    match result {
        Ok(resolved) => {
            let mut line = json_prefix("profile");
            line.push_str(",\"source\":");
            write_json_string(&resolved.source.to_string(), &mut line);
            line.push_str(",\"game\":");
            write_json_string(resolved.profile.game().name(), &mut line);
            line.push_str(",\"id\":");
            write_json_string(&resolved.profile.game().id().to_string(), &mut line);
            line.push_str(",\"stages\":");
            line.push_str(&resolved.profile.stages().len().to_string());
            line.push('}');
            let _ = writeln!(out, "{line}");
            Exit::SUCCESS
        }
        Err(e) => {
            let cli = CliError::from(e);
            emit_error(out, cli.message());
            cli.exit()
        }
    }
}

/// The `{"ts":...,"event":<event>` prefix shared by every profile JSON record.
fn json_prefix(event: &str) -> String {
    let mut s = String::from("{\"ts\":");
    write_json_string(&rfc3339_utc(SystemTime::now()), &mut s);
    s.push_str(",\"event\":");
    write_json_string(event, &mut s);
    s
}

/// One `diagnostic` record, preserving the diagnostic's structured fields rather
/// than pre-rendering them into a string.
fn emit_diagnostic(out: &mut dyn Write, d: &Diagnostic) {
    let mut line = json_prefix("diagnostic");
    line.push_str(",\"code\":");
    write_json_string(d.code.as_str(), &mut line);
    line.push_str(",\"path\":");
    write_json_string(&d.location, &mut line);
    if let Some(pos) = &d.position {
        line.push_str(",\"line\":");
        line.push_str(&pos.line.to_string());
        line.push_str(",\"col\":");
        line.push_str(&pos.column.to_string());
    }
    line.push_str(",\"message\":");
    write_json_string(&d.message, &mut line);
    line.push('}');
    let _ = writeln!(out, "{line}");
}

/// The terminal `summary` record, distinguishing a clean profile (zero
/// diagnostics, `ok:true`) from no output at all.
fn emit_summary(out: &mut dyn Write, diagnostics: usize, ok: bool) {
    let mut line = json_prefix("summary");
    line.push_str(",\"diagnostics\":");
    line.push_str(&diagnostics.to_string());
    line.push_str(",\"ok\":");
    line.push_str(if ok { "true" } else { "false" });
    line.push('}');
    let _ = writeln!(out, "{line}");
}

/// An `error` record for a failure with no per-diagnostic structure.
fn emit_error(out: &mut dyn Write, message: &str) {
    let mut line = json_prefix("error");
    line.push_str(",\"message\":");
    write_json_string(message, &mut line);
    line.push('}');
    let _ = writeln!(out, "{line}");
}

/// The `profiles` count record for `list --json`.
fn emit_profiles(
    out: &mut dyn Write,
    bundled: usize,
    user_total: usize,
    dirs: &[(PathBuf, usize)],
) {
    let mut line = json_prefix("profiles");
    line.push_str(",\"bundled\":");
    line.push_str(&bundled.to_string());
    line.push_str(",\"user_total\":");
    line.push_str(&user_total.to_string());
    line.push_str(",\"directories\":[");
    for (i, (dir, count)) in dirs.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        line.push_str("{\"path\":");
        write_json_string(&dir.display().to_string(), &mut line);
        line.push_str(",\"count\":");
        line.push_str(&count.to_string());
        line.push('}');
    }
    line.push_str("]}");
    let _ = writeln!(out, "{line}");
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
