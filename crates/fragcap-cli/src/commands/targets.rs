// SPDX-License-Identifier: Apache-2.0

//! `targets`: manage capture targets in the user-owned local store (`local.db`).
//!
//! Register a target (`add`, deriving a unique handle and a stable identity, or
//! `--steam` to register an installed title), list and show registered targets,
//! and discover installed games (`discover`, `scan`). Operations that write the
//! shipped catalog store live under `catalog`.
//!
//! With no subcommand, the command lists registered targets from the default local
//! store, the same listing a bare `fragcap` invocation prints.

use std::io::Write;
use std::path::{Path, PathBuf};

use fragcap::profile::FidelityTier;
use std::io::IsTerminal;

use fragcap::targets::{
    handle, identifier, install_presence, is_row_index, name_divergence, resolve_id,
    resolve_positional, CandidateIdentity, ClassificationSource, CompatibilityMatrix,
    DetectionScan, DirectorySource, Discovery, InstallPresence, NameDivergence, Selection,
    SocketHolderAnswer, Store, TargetClassification, TargetEntry, TargetSource,
    INSTALL_MISSING_NOTE,
};

use crate::cli::{
    TargetsAddArgs, TargetsArgs, TargetsCommand, TargetsDiscoverArgs, TargetsExportArgs,
    TargetsShowArgs,
};
use crate::color::{use_color, Stream, RESET, WARN};
use crate::commands::target_resolve;
use crate::exit::{CliError, Exit};
use crate::paths;

/// Run the `targets` command, writing results to `out`. With no subcommand, list
/// the registered targets from the default local store (no footer).
pub fn run(args: &TargetsArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let Some(command) = &args.command else {
        return list_default(out, false);
    };
    match command {
        TargetsCommand::Add(args) => add(args, out),
        // `list` mirrors the bare `fragcap targets`: an omitted `--db` resolves the
        // default store, and an unresolvable location degrades to the empty listing
        // rather than erroring, since there is nothing to list either way.
        TargetsCommand::List { db } => {
            let path = db.clone().or_else(default_local_store);
            match path {
                Some(path) => hero_listing(&path, false, out),
                None => {
                    empty_listing(out);
                    Ok(Exit::SUCCESS)
                }
            }
        }
        TargetsCommand::Show(args) => show(args, out),
        TargetsCommand::Discover(args) => discover(args, out),
        TargetsCommand::Scan {
            dir,
            catalog_db,
            db,
        } => scan(dir, catalog_db.as_deref(), db.as_deref(), out),
        TargetsCommand::Remove(args) => remove(args, out),
        TargetsCommand::Export(args) => export(args, out),
        TargetsCommand::Import { file, db } => import(file, db.as_deref(), out),
    }
}

/// List registered targets from the default local store, for a bare `fragcap` or a
/// `targets` invocation with no subcommand. With `footer`, append a line pointing at
/// `--help`, which is what distinguishes a bare `fragcap` (footer) from an explicit
/// `fragcap targets` (no footer). A store that does not exist yet is an empty
/// listing, not an error.
pub fn list_default(out: &mut dyn Write, footer: bool) -> Result<Exit, CliError> {
    // Resolve like the rest of the surface: the FRAGCAP_LOCAL_DB override, else the
    // per-user default (whose parent is created for first use).
    let exit = match default_local_store() {
        Some(path) => hero_listing(&path, footer, out)?,
        None => {
            empty_listing(out);
            print_footer(out, footer);
            Exit::SUCCESS
        }
    };
    Ok(exit)
}

/// Resolve the default local store when no `--db` is given: the `FRAGCAP_LOCAL_DB`
/// override, else the per-user default, the same order the bare `fragcap targets`
/// command uses. `None` only when no location can be determined at all.
///
/// The per-user default is created on first use, but SQLite does not create a missing
/// parent directory, so this ensures the default's parent exists before the store is
/// opened (FR-004, the first-use flow). Only the per-user default has its parent
/// created; an explicit `--db` and the `FRAGCAP_LOCAL_DB` override are operator-named
/// and used as given, matching `capture`'s rule of bootstrapping only defaulted
/// locations. A create failure is left for the subsequent open to surface.
///
/// `pub(crate)` since slice S063 so `targets discover` resolves its local store
/// through the same precedence rather than a second implementation of it.
pub(crate) fn default_local_store() -> Option<PathBuf> {
    if let Some(env) = paths::local_db_path(None) {
        return Some(env);
    }
    let path = paths::default_local_db_path()?;
    ensure_parent_dir(&path);
    Some(path)
}

/// Best-effort creation of a store path's parent directory, so a first-use open can
/// create the database: the store layer opens SQLite, which does not create a missing
/// parent directory, so on a clean machine the per-user application-data directory
/// must be made first. A create failure is left for the subsequent open to surface.
fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
}

/// Resolve the local store a subcommand operates on: an explicit `--db` wins, else the
/// default resolution above. A subcommand that must open a store treats an
/// unresolvable location as a named failure rather than a panic or a silent no-op
/// (FR-003).
fn resolve_store(db: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(db) = db {
        return Ok(db.to_path_buf());
    }
    default_local_store().ok_or_else(|| {
        CliError::failure(
            "the local store path could not be determined; pass --db or set FRAGCAP_LOCAL_DB",
        )
    })
}

/// The hero listing: run discovery and register newly found titles (so each row is
/// a registered, capturable target), then present the registered targets ordered by
/// handle with their CAPTURE and KNOWN columns, write the row-index snapshot, and
/// name the next command. An empty result prints the commands that populate the
/// store. Registration is additive and idempotent; no existing entry is modified
/// (FR-001, FR-007).
fn hero_listing(db: &Path, footer: bool, out: &mut dyn Write) -> Result<Exit, CliError> {
    hero_listing_with_machine_probe(db, footer, out, real_machine_probe().as_ref())
}

/// The real machine-wide anti-cheat probe `hero_listing` uses in production: the
/// real Windows adapter, or, on any other platform, a probe that never finds
/// anything (there is no adapter to run there). Returned as a trait object behind
/// one function so the whole path can be exercised in a test with
/// [`fragcap::targets::FixtureMachineAntiCheatProbe`] instead (FR-009), via
/// [`hero_listing_with_machine_probe`].
fn real_machine_probe() -> Box<dyn fragcap::targets::MachineAntiCheatProbe> {
    #[cfg(windows)]
    {
        Box::new(fragcap::WindowsMachineAntiCheatProbe::new())
    }
    #[cfg(not(windows))]
    {
        Box::new(fragcap::targets::FixtureMachineAntiCheatProbe::new(
            Vec::new(),
        ))
    }
}

/// `hero_listing`'s body, taking the machine-wide probe as a parameter so a test
/// can substitute a fixture result for it (FR-009) while the public entry point
/// above always uses the real one.
fn hero_listing_with_machine_probe(
    db: &Path,
    footer: bool,
    out: &mut dyn Write,
    machine_probe: &dyn fragcap::targets::MachineAntiCheatProbe,
) -> Result<Exit, CliError> {
    let mut store = Store::open(db).map_err(|e| CliError::failure(e.to_string()))?;

    // Discovery is a best-effort bootstrap: a missing catalog, an absent Steam, or a
    // platform without the known-roots walk registers nothing and lists whatever is
    // already registered. A discovery failure never sinks the listing.
    register_from_discovery(&mut store, out);

    let mut targets = store
        .targets()
        .map_err(|e| CliError::failure(e.to_string()))?;
    targets.sort_by(|a, b| a.handle.cmp(&b.handle));

    if targets.is_empty() {
        // The most recent listing displayed no rows: replace the snapshot with an
        // empty set so a stale row index from an earlier listing cannot resolve
        // through it after the row it named is gone (a re-registered stable id must
        // not silently inherit an old position).
        store
            .write_listing_snapshot(&[])
            .map_err(|e| CliError::failure(e.to_string()))?;
        empty_listing(out);
        // The machine-wide check is unrelated to whether any target is registered
        // (slice S068, issue #170, review of PR #195): an EAC service installed by
        // a title fragcap has not registered, or has not discovered at all, is
        // still worth reporting. Run it here too, not only on the non-empty path
        // below.
        render_machine_section(&machine_probe.detect(), out);
        // The footer is always the very last thing printed, in both this path and
        // the non-empty one below: a bare `fragcap` and an explicit `fragcap
        // targets` must differ by exactly the footer line and nothing else,
        // whatever the machine-scope section adds (Codex review of PR #195 caught
        // this breaking when the section was printed before a conditional footer).
        print_footer(out, footer);
        return Ok(Exit::SUCCESS);
    }

    render_table(&targets, out);

    // A machine-wide anti-cheat fact (slice S068, issue #170) is never a title's
    // evidence: it is rendered separately, once, and only when the probe actually
    // found something, so it can never be mistaken for a claim about any row above
    // it, and a probe that found nothing never prints a false "confirmed clean"
    // section (FR-007, FR-008).
    render_machine_section(&machine_probe.detect(), out);

    // Pin what was shown so `capture <n>` names the row the user saw (FR-004).
    let rows: Vec<(i64, &str)> = targets
        .iter()
        .map(|t| (t.stable_id, t.handle.as_str()))
        .collect();
    store
        .write_listing_snapshot(&rows)
        .map_err(|e| CliError::failure(e.to_string()))?;

    // End by naming the next command: the first ready row whose install root is not
    // missing; else the first non-missing row of any readiness; else, only when
    // every registered row's install root is missing, the first row (there is no
    // better answer). `.unwrap_or(1)` alone was wrong here: it named row 1 even
    // when row 1's own install root was missing and a healthy row existed further
    // down (review of PR #193). A row whose files are gone is never offered ahead
    // of one that is not, since suggesting one is a bad first command (issue #167).
    let next = targets
        .iter()
        .position(|t| {
            fragcap::targets::capture_readiness(t) == fragcap::targets::CaptureReadiness::Ready
                && install_presence(t) != InstallPresence::Missing
        })
        .or_else(|| {
            targets
                .iter()
                .position(|t| install_presence(t) != InstallPresence::Missing)
        })
        .map(|i| i + 1)
        .unwrap_or(1);
    let _ = writeln!(out, "\nNext command:  fragcap capture {next}");

    print_footer(out, footer);
    Ok(Exit::SUCCESS)
}

/// Append the `--help` pointer line a bare `fragcap` invocation carries and an
/// explicit `fragcap targets` omits (section 17.4). Always called last, after
/// everything else the listing prints (the machine-scope section among it, slice
/// S068), so a bare and an explicit listing differ by exactly this line and
/// nothing else, regardless of what else the listing rendered.
fn print_footer(out: &mut dyn Write, footer: bool) {
    if footer {
        let _ = writeln!(out, "\nRun `fragcap --help` to see all commands.");
    }
}

/// The empty case: no registered targets and discovery found nothing. Print the
/// concrete commands that populate the store, still naming a next command so hero
/// criterion 5 holds in the empty case (FR-006, SC-006). The footer, if any, is
/// the caller's job via [`print_footer`], printed after anything else the caller
/// adds (the machine-scope section).
fn empty_listing(out: &mut dyn Write) {
    let _ = writeln!(out, "  No targets yet.");
    let _ = writeln!(out);
    let _ = writeln!(out, "  Add one:        fragcap targets add");
    let _ = writeln!(out, "  Scan a folder:  fragcap targets scan <dir>");
}

/// Render the numbered CAPTURE / ENGINE / SENSITIVITIES table. The target order is
/// the caller's (handle order).
///
/// # The width rule
///
/// Every column but the last sizes to its own content; the last, SENSITIVITIES, is
/// free-running and is neither padded nor truncated. Nothing here truncates a value
/// and nothing wraps a row.
///
/// That is a decision, not an accident. Truncating a product name is the silent loss
/// P-4 forbids: an operator reading `Easy Anti-Che...` cannot tell whether the value
/// was clipped or whether a second product was dropped. Wrapping a row would break
/// the column alignment the split exists to provide. A value wider than the terminal
/// therefore overflows visibly, which is a legible failure rather than a lie.
///
/// The budget is therefore stated over the columns the tool controls. With every
/// bounded column at its widest, everything but the handle costs 53 of an 80 column
/// terminal, leaving 27 for a handle. The readiness column keeping its two short
/// labels is what buys that much, which is why slice S065 retired the two long
/// readiness sentences rather than moving them here (see the slice decisions
/// fragment).
///
/// The operator's own machine does not fit, and that is the declared behavior rather
/// than a defect: its longest handle is 47 characters
/// (`warhammer_40_000_dawn_of_war_definitive_edition`), so its rows run to 100
/// columns with every value intact. Shortening the handles a target carries is
/// issues #166 and #173. `cli_targets.rs` measures the non-handle budget, the fit at
/// the longest fitting handle, and the no-clipping overflow at that real 47
/// character handle, all from rendered output, so none of this can drift unnoticed.
fn render_table(targets: &[TargetEntry], out: &mut dyn Write) {
    let num_w = targets.len().to_string().len().max(1);
    let target_w = width_of(targets.iter().map(|t| t.handle.clone()), "TARGET");
    let capture_w = "needs a target".len();
    let engine_w = width_of(
        targets.iter().map(fragcap::targets::engine_summary),
        "ENGINE",
    );
    let _ = writeln!(
        out,
        "  {:>num_w$}  {:<target_w$}  {:<capture_w$}  {:<engine_w$}  SENSITIVITIES",
        "#", "TARGET", "CAPTURE", "ENGINE"
    );
    let color = use_color(Stream::Stdout);
    for (i, t) in targets.iter().enumerate() {
        let capture = fragcap::targets::capture_readiness(t).label();
        let engine = fragcap::targets::engine_summary(t);
        let sensitivities = sensitivities_cell(t, color);
        let _ = writeln!(
            out,
            "  {:>num_w$}  {:<target_w$}  {:<capture_w$}  {:<engine_w$}  {}",
            i + 1,
            t.handle,
            capture,
            engine,
            sensitivities
        );
    }
}

/// Render the machine-scope anti-cheat section (slice S068, issue #170): a
/// heading and one indented `<product> (<evidence>)` line per finding, printed
/// after the per-target table and never touching any target row. Nothing is
/// printed for an empty result, which covers both "the probe ran and found
/// nothing" and "the probe could not run at all" (FR-008): rendering a "no
/// anti-cheat products found" line would assert a completed check the second
/// case never made.
fn render_machine_section(
    findings: &[fragcap::targets::MachineAntiCheatFinding],
    out: &mut dyn Write,
) {
    if findings.is_empty() {
        return;
    }
    let _ = writeln!(out, "\nMachine:");
    for f in findings {
        let _ = writeln!(out, "  {} ({})", f.product, f.evidence);
    }
}

/// The SENSITIVITIES cell for one row: the ordinary
/// [`fragcap::targets::sensitivities_summary`] value, prefixed with
/// [`INSTALL_MISSING_NOTE`] when the row's `install_root` is recorded and absent
/// (issue #167). This is the whole of the missing-install-root rendering: the cell
/// is free-running and exempt from padding (`render_table`'s own width budget), so
/// a row not in this state returns exactly what it always has, unchanged in every
/// color mode (FR-009). The `-` clean marker is replaced outright rather than
/// joined, since `install folder not found; -` reads worse than the note alone.
fn sensitivities_cell(t: &TargetEntry, color: bool) -> String {
    let base = fragcap::targets::sensitivities_summary(t);
    if install_presence(t) != InstallPresence::Missing {
        return base;
    }
    let note = if base == fragcap::targets::SCANNED_CLEAN_MARKER || base.is_empty() {
        INSTALL_MISSING_NOTE.to_string()
    } else {
        format!("{INSTALL_MISSING_NOTE}; {base}")
    };
    if color {
        format!("{WARN}{note}{RESET}")
    } else {
        note
    }
}

/// The display width of a column: the widest value, never narrower than its heading.
/// Counts characters rather than bytes so a non-ASCII product name (`Ren'Py` is
/// ASCII, but a future one need not be) does not over-pad.
fn width_of(values: impl Iterator<Item = String>, heading: &str) -> usize {
    values
        .map(|v| v.chars().count())
        .chain(std::iter::once(heading.chars().count()))
        .max()
        .unwrap_or_else(|| heading.chars().count())
}

/// Discover and register into `store`, returning the number of newly registered
/// targets or the first hard error. A missing or absent catalog is not an error:
/// there is simply nothing to classify against, so it returns `Ok(0)`. A discovery
/// composition failure or a registration failure is a real error and is returned,
/// so a caller that must report an honest outcome (the `doctor --fix` action) can.
fn discover_and_register(store: &mut Store, out: &mut dyn Write) -> Result<usize, CliError> {
    // Resolve and, on first run, seed the per-user catalog from the template shipped
    // beside the executable, through the same helper the capture path uses. Without
    // this the shipped catalog was never copied into the per-user location for the
    // hero listing, so a fresh install discovered and classified nothing until a
    // capture happened to seed it (the drift this shares one helper to prevent). A
    // bootstrap failure is a warning, never fatal: the listing continues with an
    // empty catalog, exactly as an absent one.
    let catalog_db = match target_resolve::ensure_catalog_store(None) {
        Ok(path) => path,
        Err(message) => {
            let _ = writeln!(out, "warning: {message}");
            None
        }
    };
    let Some(catalog_db) = catalog_db else {
        return Ok(0);
    };
    if !catalog_db.exists() {
        return Ok(0);
    }
    let discovery = compose_and_discover(&catalog_db, store, None)?;
    for warning in &discovery.warnings {
        let _ = writeln!(out, "warning: {warning}");
    }
    let outcome = fragcap::targets::register_candidates(store, &discovery.candidates)
        .map_err(|e| CliError::failure(e.to_string()))?;
    Ok(outcome.registered)
}

/// The hero listing's best-effort bootstrap: discover and register, but never let a
/// discovery failure sink the listing. A failure is reported as a warning and the
/// listing continues with whatever is already registered.
fn register_from_discovery(store: &mut Store, out: &mut dyn Write) {
    match discover_and_register(store, out) {
        Ok(registered) if registered > 0 => {
            let _ = writeln!(
                out,
                "  registered {registered} newly discovered target(s).\n"
            );
        }
        Ok(_) => {}
        Err(e) => {
            let _ = writeln!(out, "warning: discovery skipped: {}", e.message());
        }
    }
}

/// Run discovery into the default local store, for the `doctor --fix` RunDiscovery
/// action (slice S056). Reuses the same discovery composition the hero listing runs
/// (P-10), but propagates a real discovery or registration failure so `doctor --fix`
/// reports a failed action rather than a false success (P-9). A missing catalog or
/// an absent platform source registers nothing and is reported as such, not a
/// failure.
pub(crate) fn run_discovery_default(out: &mut dyn Write) -> Result<Exit, CliError> {
    let db = default_local_store()
        .ok_or_else(|| CliError::failure("the local store path could not be determined"))?;
    let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
    let registered = discover_and_register(&mut store, out)?;
    let _ = writeln!(out, "  discovery registered {registered} target(s).");
    Ok(Exit::SUCCESS)
}

/// Run `targets scan <dir>`: point discovery at one directory (the tier-3
/// [`DirectorySource`], slice S052). With a catalog, the directory is scanned for
/// technologies and they ride as evidence (slice S053). The discovered titles are
/// registered into the local store the same idempotent way the hero listing
/// registers (FR-016): `--db` names it explicitly, otherwise it resolves to the
/// default local store so the `targets scan <dir>` the empty listing recommends
/// actually populates it.
fn scan(
    dir: &Path,
    catalog_db: Option<&Path>,
    db: Option<&Path>,
    out: &mut dyn Write,
) -> Result<Exit, CliError> {
    let path = dir.to_string_lossy().into_owned();
    let source = match catalog_db {
        Some(catalog_db) => {
            let catalog = Store::open(catalog_db).map_err(|e| CliError::failure(e.to_string()))?;
            let signatures = catalog
                .load_signatures()
                .map_err(|e| CliError::failure(e.to_string()))?;
            let set = fragcap::profile::signature::SignatureSet::compile(&signatures);
            DirectorySource::with_signatures(path, set)
        }
        None => DirectorySource::new(path),
    };
    let discovery = source
        .discover()
        .map_err(|e| CliError::failure(e.to_string()))?;
    print_discovery(&discovery, out);

    // Register the discovered titles (FR-016). The `--db` flag wins, else the local
    // store resolves like the rest of the surface (the `FRAGCAP_LOCAL_DB` override,
    // else the per-user default). Registration is idempotent, so a rescan does not
    // duplicate (P-10), and the conserved account is surfaced (P-4).
    let db = db.map(Path::to_path_buf).or_else(default_local_store);
    if let Some(db) = db {
        let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
        let outcome = fragcap::targets::register_candidates(&mut store, &discovery.candidates)
            .map_err(|e| CliError::failure(e.to_string()))?;
        let _ = writeln!(
            out,
            "registered {} target(s), {} already present",
            outcome.registered, outcome.already_present
        );
    }
    Ok(Exit::SUCCESS)
}

/// Run `targets remove <selector>`: remove exactly the resolved target. An ambiguous
/// name lists its matches and refuses (exit 2, P-9); a clean handle/name miss reports
/// it and exits 0; an out-of-range row index or unknown `--id` exits 2 (FR-017).
fn remove(args: &TargetsShowArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let db = resolve_store(args.db.as_deref())?;
    let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
    let (selection, miss_exit) = match (args.id, &args.selector) {
        (Some(id), _) => (resolve_id(&store, id), Exit::USAGE),
        (None, Some(token)) if is_row_index(token) => {
            (resolve_positional(&store, token), Exit::USAGE)
        }
        (None, Some(token)) => (resolve_positional(&store, token), Exit::SUCCESS),
        (None, None) => return Err(CliError::usage("a selector or --id is required")),
    };
    match selection.map_err(|e| CliError::failure(e.to_string()))? {
        Selection::Resolved(t) => {
            let id =
                t.id.ok_or_else(|| CliError::failure("resolved target has no row id"))?;
            store
                .delete_target(id)
                .map_err(|e| CliError::failure(e.to_string()))?;
            let _ = writeln!(out, "removed {} (id {})", t.handle, t.stable_id);
            Ok(Exit::SUCCESS)
        }
        Selection::NoMatch => {
            let _ = writeln!(
                out,
                "{}",
                target_resolve::no_match_message(&store, args.selector.as_deref())
            );
            Ok(miss_exit)
        }
        Selection::Ambiguous(matches) => {
            let _ = writeln!(
                out,
                "ambiguous: {} targets match; select by handle or --id:",
                matches.len()
            );
            for t in &matches {
                let _ = writeln!(out, "  {}\t{}\t{}", t.handle, t.stable_id, t.name);
            }
            Ok(Exit::USAGE)
        }
    }
}

/// Run `targets export [selector]`: emit the target-entry JSON array to stdout. No
/// selector exports every target; a selector exports the one it resolves. The
/// no-match behavior follows the selector kind (the section 5.4 contract, as `show`
/// and `remove` do): an unmatched handle or name is a clean miss that emits `[]` and
/// exits 0, while an out-of-range row index or an unknown `--id` is an invalid
/// machine reference (exit 2). An ambiguous name refuses (exit 2) (FR-018).
fn export(args: &TargetsExportArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let db = resolve_store(args.db.as_deref())?;
    let store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
    let entries = match (args.id, &args.selector) {
        (None, None) => store
            .targets()
            .map_err(|e| CliError::failure(e.to_string()))?,
        (id, selector) => {
            let (selection, miss_exit) = match (id, selector) {
                (Some(id), _) => (resolve_id(&store, id), Exit::USAGE),
                (None, Some(token)) if is_row_index(token) => {
                    (resolve_positional(&store, token), Exit::USAGE)
                }
                (None, Some(token)) => (resolve_positional(&store, token), Exit::SUCCESS),
                (None, None) => unreachable!("covered by the first arm"),
            };
            match selection.map_err(|e| CliError::failure(e.to_string()))? {
                Selection::Resolved(t) => vec![*t],
                Selection::NoMatch if miss_exit == Exit::SUCCESS => Vec::new(),
                Selection::NoMatch => {
                    let _ = writeln!(
                        out,
                        "{}",
                        target_resolve::no_match_message(&store, selector.as_deref())
                    );
                    return Ok(miss_exit);
                }
                Selection::Ambiguous(matches) => {
                    let _ = writeln!(
                        out,
                        "ambiguous: {} targets match; select by handle or --id",
                        matches.len()
                    );
                    return Ok(Exit::USAGE);
                }
            }
        }
    };
    let _ = write!(out, "{}", fragcap::targets::export_targets(&entries));
    Ok(Exit::SUCCESS)
}

/// Run `targets import <file>`: parse the target-entry array and merge each element
/// on its stable identifier (update in place, or insert). A nonconforming file is
/// rejected whole, applying nothing (FR-019).
fn import(file: &Path, db: Option<&Path>, out: &mut dyn Write) -> Result<Exit, CliError> {
    let json = std::fs::read_to_string(file)
        .map_err(|e| CliError::failure(format!("cannot read {}: {e}", file.display())))?;
    let entries =
        fragcap::targets::import_targets(&json).map_err(|e| CliError::usage(e.to_string()))?;
    let db = resolve_store(db)?;
    let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
    // The whole batch merges in one transaction: all-or-nothing, so a constraint
    // violation partway through leaves the store untouched (FR-019).
    let (inserted, updated) = store
        .merge_targets(&entries)
        .map_err(|e| CliError::usage(e.to_string()))?;
    let _ = writeln!(out, "imported {inserted} new, {updated} updated");
    Ok(Exit::SUCCESS)
}

/// Run `targets discover`: walk Steam (tier 1) and the known game-install roots
/// (tier 2) and list the candidates found (slice S052). An inspection command: it
/// reads and prints, and (unlike the hero listing) registers nothing beyond the
/// first-run volume eligibility seeding the cross-volume walk needs to be safe.
fn discover(args: &TargetsDiscoverArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    // Both stores are overrides, never requirements (issue #179). The local one
    // resolves through the same precedence the hero listing uses; the catalog
    // one through the shared bootstrap, so a first run creates it rather than
    // asking the operator for a path to a component fragcap installs.
    let local_db = match args.local_db.clone().or_else(default_local_store) {
        Some(p) => p,
        None => {
            return Err(CliError::failure(concat!(
                "no local store could be resolved: pass --local-db, set ",
                "FRAGCAP_LOCAL_DB, or run on a machine with a per-user ",
                "application data directory",
            )))
        }
    };
    let catalog_db = target_resolve::ensure_catalog_store(args.catalog_db.as_deref())
        .map_err(CliError::failure)?
        .ok_or_else(|| {
            CliError::failure(concat!(
                "no catalog store could be resolved: pass --catalog-db, set ",
                "FRAGCAP_CATALOG_DB, or run on a machine with a per-user ",
                "application data directory",
            ))
        })?;
    // Name both stores. Since slice S063 either can come from a flag, an
    // environment override, or the per-user default, and discovery reads one
    // while writing volume eligibility to the other, so an operator who cannot
    // see which is which cannot tell what was consulted or what was touched
    // (FR-005, raised in review of PR #190).
    let _ = writeln!(
        out,
        "discovering with catalog {} into local store {}",
        catalog_db.display(),
        local_db.display()
    );
    let mut local = Store::open(&local_db).map_err(|e| CliError::failure(e.to_string()))?;
    let discovery = compose_and_discover(&catalog_db, &mut local, args.steam_root.as_deref())?;
    print_discovery(&discovery, out);
    Ok(Exit::SUCCESS)
}

/// Compose the discovery sources (Steam tier 1, and on Windows the known-roots tier
/// 2) against a catalog and run them through the shared driver, returning the
/// conserved [`Discovery`]. The `local` store carries the volume eligibility
/// allowlist the cross-volume walk reads; it is seeded permissively on first run
/// (FR-016a) and otherwise only read. This one composition backs both the `discover`
/// inspection command and the hero listing's registration bootstrap (P-10, SC-006).
fn compose_and_discover(
    catalog_db: &Path,
    local: &mut Store,
    steam_root_flag: Option<&Path>,
) -> Result<Discovery, CliError> {
    let catalog = Store::open(catalog_db).map_err(|e| CliError::failure(e.to_string()))?;

    // Locate the Steam root: the explicit flag, else Steam's own installation.
    let steam_root = match steam_root_flag {
        Some(root) => Some(root.to_path_buf()),
        None => fragcap::steam::discover().ok().map(|i| i.root),
    };
    let steam = steam_root
        .as_ref()
        .map(|root| fragcap::SteamSource::new(root, &catalog));

    // The known-roots walk enumerates fixed volumes, a platform operation that runs
    // on Windows, where the tool captures. The eligibility allowlist lives in
    // local.db and is seeded permissively on first run, then only read.
    #[cfg(windows)]
    let inventory = fragcap::WindowsVolumeInventory::new();
    #[cfg(windows)]
    {
        use fragcap::targets::VolumeInventory;
        let volumes = inventory.fixed_volumes();
        local
            .seed_volume_eligibility(&volumes)
            .map_err(|e| CliError::failure(e.to_string()))?;
    }
    #[cfg(windows)]
    let eligible: std::collections::HashSet<String> = local
        .eligible_volumes()
        .map_err(|e| CliError::failure(e.to_string()))?
        .into_iter()
        .map(|v| v.volume_id)
        .collect();
    #[cfg(not(windows))]
    let _ = local;
    #[cfg(windows)]
    let lister = fragcap::targets::FsDirectoryLister;
    // Detection is signature-driven (slice S053): the classifier scans each known-root
    // child against the catalog's signature table, stamping a detected engine
    // `verified` and carrying any anti-cheat or DRM as neutral evidence. A child with
    // no detected engine is still a game (the known-root structural prior), at
    // heuristic-unverified. An unseeded catalog yields no detections, only the prior.
    #[cfg(windows)]
    let classifier = {
        let signatures = catalog
            .load_signatures()
            .map_err(|e| CliError::failure(e.to_string()))?;
        let set = fragcap::profile::signature::SignatureSet::compile(&signatures);
        fragcap::targets::SignatureClassifier::for_known_root(set)
    };
    #[cfg(windows)]
    let known_roots =
        fragcap::targets::KnownRootsSource::new(&inventory, &eligible, &lister, &classifier);

    // Compose the sources into one listing through the shared driver (SC-006).
    let mut sources: Vec<&dyn TargetSource> = Vec::new();
    if let Some(steam) = &steam {
        sources.push(steam);
    }
    #[cfg(windows)]
    sources.push(&known_roots);

    fragcap::targets::discover_all(&sources).map_err(|e| CliError::failure(e.to_string()))
}

/// Print a discovery listing: one line per candidate (source, identity, name,
/// classification) then the conserved account, so an excluded volume or an
/// unparsable title is visible rather than silent (P-4).
fn print_discovery(discovery: &Discovery, out: &mut dyn Write) {
    if discovery.candidates.is_empty() {
        let _ = writeln!(out, "no candidates discovered");
    }
    for c in &discovery.candidates {
        let identity = match &c.identity {
            CandidateIdentity::SteamAppId(appid) => format!("steam:{appid}"),
            CandidateIdentity::Path(path) => path.clone(),
        };
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}",
            c.source_name,
            identity,
            c.classification.as_str(),
            c.fidelity.as_str(),
            c.display_name
        );
        // Detected technologies ride as neutral evidence (slice S053): a fact per
        // line, never a status that frames the title as off limits (spec 3.6).
        for f in &c.evidence {
            let _ = writeln!(
                out,
                "    {}: {} ({})",
                f.category.as_str(),
                f.product,
                f.fidelity.as_str()
            );
        }
    }
    let a = &discovery.account;
    let _ = writeln!(
        out,
        "account: considered={} produced={} parse_failed={} declined={} not_a_game={} container_descended={} container_descent_truncated={} volume_skipped={} access_error={}",
        a.considered,
        a.produced,
        a.parse_failed,
        a.declined_by_user,
        a.considered_not_a_game,
        a.container_descended,
        a.container_descent_truncated,
        a.volume_skipped,
        a.access_error,
    );
    // Surface the named diagnostics so a loss the account counts (an unreadable
    // root, a malformed manifest) is recoverable to which one failed (P-4).
    for warning in &discovery.warnings {
        let _ = writeln!(out, "warning: {warning}");
    }
}

/// Register a target from a name (slice S051): derive a unique handle, assign a
/// stable identity (anchored when an `--anchor` or `--steam` is given, otherwise
/// random), and store it. A user-registered target is `authored` with a `user`
/// classification source and, until enriched, an `unknown` classification.
///
/// `--steam <app_id>` resolves the installed title through the local Steam
/// installation, supplying its name (when the positional name is omitted) and a
/// `steam:<app_id>` anchor, replacing the retired `steam profile <app_id>`.
/// The `install_root`, `folder_name`, and `executable_hint` a `--steam`-resolved
/// `InstalledTitle` supplies to a `targets add --steam` registration, carrying
/// every observation the lookup already made rather than discarding it (review of
/// PR #193). Without this, an explicitly `--steam`-registered title had no
/// `install_root` (so the missing-install-root detection, issue #167, could never
/// fire for it) and no `folder_name`/`executable_hint` (so it was neither
/// findable by them nor eligible for the divergence note, issue #173), unlike the
/// same title reached through automatic discovery. Split out so the mapping is
/// unit-testable without a real Steam installation (the registry lookup
/// `targets add --steam` itself performs has no fixture seam).
fn steam_add_metadata(
    title: &fragcap::steam::InstalledTitle,
) -> (Option<String>, Option<String>, Option<String>) {
    (
        Some(title.install_dir.display().to_string()),
        Some(title.installdir.clone()),
        title.launch_executable.clone(),
    )
}

fn add(args: &TargetsAddArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    // Resolve `--steam` first: it can supply both the name and the anchor. The
    // enumeration warnings go to `out` as surfaced diagnostics rather than being
    // dropped (P-4). A `--steam` app id that is not installed is a usage error.
    let (name, steam_anchor, steam_install_root, steam_folder_name, steam_executable_hint) =
        if let Some(app_id) = &args.steam {
            // A missing Steam or an unsupported platform is a usage error (exit 2); a
            // filesystem read failure is an expected runtime failure (exit 1). Reuse the
            // `steam` command's own mapping so the two entry points agree.
            let installation =
                fragcap::steam::discover().map_err(crate::commands::steam::map_steam_error)?;
            for warning in &installation.warnings {
                let _ = writeln!(out, "warning: {warning}");
            }
            let title = installation.find(app_id).ok_or_else(|| {
                CliError::usage(format!(
                    "Steam app {app_id} is not installed in any library"
                ))
            })?;
            let name = args.name.clone().unwrap_or_else(|| title.name.clone());
            let (install_root, folder_name, executable_hint) = steam_add_metadata(title);
            (
                name,
                Some(format!("steam:{app_id}")),
                install_root,
                folder_name,
                executable_hint,
            )
        } else {
            let name = args.name.clone().ok_or_else(|| {
                CliError::usage("a target name is required (or supply one with --steam <app_id>)")
            })?;
            (name, None, None, None, None)
        };

    let db = resolve_store(args.db.as_deref())?;
    let mut store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;

    // Canonicalize the anchor once (lowercase the platform prefix, trim) so a
    // CLI-supplied `STEAM:620` matches and is stored identically to `steam:620`. The
    // `--steam` anchor and an explicit `--anchor` are mutually exclusive (a clap
    // group enforces it), so at most one is present.
    let anchor = args
        .anchor
        .clone()
        .or(steam_anchor)
        .map(|a| identifier::canonicalize_anchor(&a));

    // An anchor already present means this title is registered: report it rather
    // than creating a duplicate (identity is deterministic from the anchor, P-10).
    if let Some(a) = &anchor {
        if let Some(existing) = store
            .target_by_anchor(a)
            .map_err(|e| CliError::failure(e.to_string()))?
        {
            let _ = writeln!(
                out,
                "already registered as {} (id {})",
                existing.handle, existing.stable_id
            );
            return Ok(Exit::SUCCESS);
        }
    }

    // Derive or validate the handle, then disambiguate against existing handles so
    // a collision suffixes the new item (_2, _3) and leaves the existing untouched.
    // A store error during the existence check propagates rather than being
    // swallowed into a false "free", which could attempt a duplicate insert.
    let exe_stem = args.exe.as_deref().map(exe_stem);
    let base = match &args.handle_override {
        Some(h) => handle::validate_override(h).map_err(CliError::usage)?,
        None => handle::derive_handle(
            &name,
            exe_stem.as_deref(),
            store
                .targets()
                .map_err(|e| CliError::failure(e.to_string()))?
                .len() as u64
                + 1,
        ),
    };
    let handle_value = handle::disambiguate(&base, |h| store.handle_exists(h))
        .map_err(|e| CliError::failure(e.to_string()))?;

    let stable_id = match &anchor {
        Some(a) => identifier::anchored_id(a),
        None => identifier::unanchored_id(),
    };

    // Run detection on the executable's directory and show the evidence inline
    // before the socket-holder decision depends on it (FR-009). Best-effort: a bare
    // exe name or a missing catalog yields no evidence, not an error.
    // The coverage state rides with the evidence: a scan that ran records whether it
    // was complete, and no scan at all records nothing rather than claiming a clean
    // one. This is the fourth producing source and is plumbed like the other three
    // (FR-015).
    let (evidence, detection_scan) = match args.exe.as_deref() {
        Some(exe) => scan_exe_evidence(exe, out),
        None => (None, None),
    };

    // The socket-holder answer decides the stored launch chain. Interactive when
    // stdin is a terminal and no `--socket-holder` was given; otherwise the flag.
    // When an `--exe` is given with no answer (non-interactive, no flag), the chain
    // is recorded unresolved rather than assuming the executable is the client: the
    // tool never claims a socket holder the user did not (P-9). To register the exe
    // as the client, answer `--socket-holder yes`.
    let answer = resolve_socket_holder(args, out)?;
    let launch_entries = match (&args.exe, answer) {
        (Some(exe), Some(a)) => Some(fragcap::targets::launch_entries_for(a, exe)),
        (Some(exe), None) => Some(fragcap::targets::launch_entries_for(
            SocketHolderAnswer::Unsure,
            exe,
        )),
        (None, _) => None,
    };

    let entry = TargetEntry {
        id: None,
        stable_id,
        handle: handle_value.clone(),
        name,
        classification: TargetClassification::Unknown,
        classification_source: ClassificationSource::User,
        fidelity: FidelityTier::Authored,
        provenance: Some(serde_json::json!({ "source": "user", "command": "targets add" })),
        anchor,
        launch_entries,
        // `--steam` already observed these; a bare, non-Steam authoring has
        // nothing of the kind distinct from what the user supplied via
        // --name/--exe.
        install_root: steam_install_root,
        evidence,
        detection_scan,
        folder_name: steam_folder_name,
        executable_hint: steam_executable_hint,
    };
    store
        .insert_target(&entry)
        .map_err(|e| CliError::failure(e.to_string()))?;

    let _ = writeln!(out, "registered {handle_value} (id {stable_id})");
    Ok(Exit::SUCCESS)
}

/// Resolve the socket-holder answer for `add`. A `--socket-holder` flag is parsed
/// (and requires `--exe`); otherwise, when an `--exe` is given and standard input is
/// a terminal, the question is asked interactively; otherwise there is no answer and
/// the caller keeps the pre-S055 default. A malformed flag, or a missing `--exe`
/// under the flag, is a usage error, never a blocking prompt (FR-015).
fn resolve_socket_holder(
    args: &TargetsAddArgs,
    out: &mut dyn Write,
) -> Result<Option<SocketHolderAnswer>, CliError> {
    if let Some(token) = &args.socket_holder {
        if args.exe.is_none() {
            return Err(CliError::usage("--socket-holder requires --exe"));
        }
        return SocketHolderAnswer::parse(token)
            .map(Some)
            .ok_or_else(|| CliError::usage("--socket-holder must be yes, no, or unsure"));
    }
    if args.exe.is_some() && std::io::stdin().is_terminal() {
        return Ok(Some(prompt_socket_holder(out)?));
    }
    Ok(None)
}

/// Ask the socket-holder question interactively, re-prompting until an answer
/// parses. An empty line re-prompts rather than assuming a default, since the honest
/// default is unknown (P-9). Used only when standard input is a terminal.
fn prompt_socket_holder(out: &mut dyn Write) -> Result<SocketHolderAnswer, CliError> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    loop {
        let _ = write!(
            out,
            "Is the executable above the process that holds the sockets? [Y/n/unsure] "
        );
        let mut line = String::new();
        let read = stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| CliError::failure(e.to_string()))?;
        if read == 0 {
            // End of input with no answer: do not guess a holder (P-9).
            return Ok(SocketHolderAnswer::Unsure);
        }
        if let Some(answer) = SocketHolderAnswer::parse(&line) {
            return Ok(answer);
        }
    }
}

/// Run detection on the directory containing `exe` and print any engine, anti-cheat,
/// or DRM findings inline, returning them as the `evidence` JSON and the coverage
/// state of the scan.
///
/// Best-effort: nothing here fails the add. A bare exe name (no directory) or a
/// missing or unreadable catalog means no scan was possible, so the coverage state is
/// `None`; recording `Complete` there would claim a clean scan that never happened
/// (P-9). A scan that ran and matched nothing returns no evidence but does record its
/// coverage state, which is what makes "scanned clean" distinguishable from "never
/// scanned" on this path. A scan attempted against a directory that could not be read
/// records `Incomplete`, matching the other three producing sources: an attempt that
/// failed is not the absence of an attempt (FR-015).
///
/// Anything the scan did not cover is written to `out` as a named warning, so a row
/// that lists as `incomplete` has its cause stated at the moment it was registered
/// rather than counted and left unexplained (P-4).
fn scan_exe_evidence(
    exe: &str,
    out: &mut dyn Write,
) -> (Option<serde_json::Value>, Option<DetectionScan>) {
    evidence_from_scan(run_exe_scan(exe), out)
}

/// Map a scan attempt to the evidence JSON and the coverage state it earns, printing
/// the findings and anything the scan did not cover.
///
/// Split from [`scan_exe_evidence`] so the mapping is reachable by a test: the CLI
/// test harness points every test at a catalog path that does not exist, so a test
/// driving the command surface can only ever reach [`ExeScan::NotRun`], and the
/// `Failed` arm below is exactly the one that was wrong.
fn evidence_from_scan(
    scan: ExeScan,
    out: &mut dyn Write,
) -> (Option<serde_json::Value>, Option<DetectionScan>) {
    let outcome = match scan {
        ExeScan::NotRun => return (None, None),
        ExeScan::Failed { path } => {
            // A scan was attempted against a directory that could not be read. That
            // is `Incomplete`, not `None`: an attempt that failed is a different
            // fact from no attempt, and recording it as no attempt would claim
            // nobody looked (P-9, FR-015). The other three producing sources record
            // it the same way.
            let _ = writeln!(
                out,
                "  warning: could not read {} during detection",
                path.display()
            );
            return (None, Some(DetectionScan::Incomplete));
        }
        ExeScan::Ran(outcome) => outcome,
    };
    let scan = Some(DetectionScan::from_outcome(&outcome));
    // Everything the scan did not cover, named here rather than only recorded on
    // the entry (P-4). Without this the row lists as `incomplete` with the cause
    // stated nowhere, which counts the loss without surfacing it. Emitted before
    // the early return, so a truncated scan that also matched nothing still says
    // why it is incomplete.
    for warning in outcome.coverage_warnings() {
        let _ = writeln!(out, "  warning: {warning}");
    }
    if outcome.findings.is_empty() {
        return (None, scan);
    }
    let mut findings = Vec::new();
    for f in &outcome.findings {
        let _ = writeln!(
            out,
            "  {}: {} ({})",
            f.category.as_str(),
            f.product,
            f.fidelity.as_str()
        );
        findings.push(serde_json::json!({
            "category": f.category.as_str(),
            "product": f.product,
            "evidence": f.evidence,
            "fidelity": f.fidelity.as_str(),
        }));
    }
    (Some(serde_json::Value::Array(findings)), scan)
}

/// What came of trying to scan the directory containing an executable.
///
/// Three outcomes, not two. Collapsing the last two loses the distinction the whole
/// coverage state exists to carry: a scan that was never possible and a scan that was
/// attempted and failed are different facts, and only the first of them means nobody
/// looked (P-9).
enum ExeScan {
    /// No scan was possible: a bare exe name with no directory, no resolvable
    /// catalog, a catalog that is absent, or one that could not be opened or read.
    /// Nothing was attempted, so the target records no coverage claim.
    NotRun,
    /// A scan was attempted and the directory could not be read.
    Failed {
        /// The directory that could not be read, so the failure is nameable.
        path: PathBuf,
    },
    /// A scan ran and produced an outcome, complete or otherwise.
    Ran(fragcap::profile::signature::ScanOutcome),
}

/// Scan the directory containing `exe`, distinguishing a scan that could not be run
/// from one that ran and one that was attempted and failed.
fn run_exe_scan(exe: &str) -> ExeScan {
    let Some(dir) = Path::new(exe)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    else {
        return ExeScan::NotRun;
    };
    let Some(catalog_db) = paths::catalog_db_path(None).or_else(paths::default_catalog_db_path)
    else {
        return ExeScan::NotRun;
    };
    if !catalog_db.exists() {
        return ExeScan::NotRun;
    }
    let Ok(catalog) = Store::open(&catalog_db) else {
        return ExeScan::NotRun;
    };
    let Ok(signatures) = catalog.load_signatures() else {
        return ExeScan::NotRun;
    };
    let set = fragcap::profile::signature::SignatureSet::compile(&signatures);
    match set.detect(dir) {
        Ok(outcome) => ExeScan::Ran(outcome),
        Err(e) => ExeScan::Failed { path: e.path },
    }
}

/// Show one target resolved by a selector.
///
/// The no-match exit code follows the selector kind (the section 5.4 contract): a
/// handle or name that matches nothing is a clean miss (exit 0), while an unknown
/// `--id` or an out-of-range row index is a bad machine reference (exit 2). An
/// ambiguous name lists its matches and exits 2 rather than guessing (P-9).
fn show(args: &TargetsShowArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let db = resolve_store(args.db.as_deref())?;
    let store = Store::open(&db).map_err(|e| CliError::failure(e.to_string()))?;
    let (selection, miss_exit) = match (args.id, &args.selector) {
        (Some(id), _) => (resolve_id(&store, id), Exit::USAGE),
        (None, Some(token)) if is_row_index(token) => {
            (resolve_positional(&store, token), Exit::USAGE)
        }
        (None, Some(token)) => (resolve_positional(&store, token), Exit::SUCCESS),
        // The clap group guarantees exactly one is present.
        (None, None) => return Err(CliError::usage("a selector or --id is required")),
    };
    let selection = selection.map_err(|e| CliError::failure(e.to_string()))?;

    match selection {
        Selection::Resolved(t) => {
            let target_id = t.id.ok_or_else(|| {
                CliError::failure(
                    "resolved target has no local row id; cannot read compatibility facts",
                )
            })?;
            let facts = store
                .compatibility_facts_for_target(target_id)
                .map_err(|e| CliError::failure(e.to_string()))?;
            let compatibility = CompatibilityMatrix::from_facts(&facts);
            print_target(&t, out);
            print_compatibility(&compatibility, out);
            Ok(Exit::SUCCESS)
        }
        Selection::NoMatch => {
            let _ = writeln!(
                out,
                "{}",
                target_resolve::no_match_message(&store, args.selector.as_deref())
            );
            Ok(miss_exit)
        }
        Selection::Ambiguous(matches) => {
            let _ = writeln!(
                out,
                "ambiguous: {} targets match; select by handle or --id:",
                matches.len()
            );
            for t in &matches {
                let _ = writeln!(out, "  {}\t{}\t{}", t.handle, t.stable_id, t.name);
            }
            Ok(Exit::USAGE)
        }
    }
}

/// Print every stored compatibility fact without selecting an aggregate verdict.
fn print_compatibility(matrix: &CompatibilityMatrix, out: &mut dyn Write) {
    if matrix.rows().is_empty() {
        let _ = writeln!(out, "compatibility:  unknown (no stored evidence)");
        return;
    }

    let _ = writeln!(out, "compatibility:");
    for row in matrix.rows() {
        let _ = write!(out, "  {} = {}", row.key.as_str(), row.value);
        if let Some(launch_case) = row.launch_case {
            let _ = write!(out, " | launch={}", launch_case.as_str());
        }
        let _ = writeln!(
            out,
            " | source={} | freshness={}",
            row.evidence_source.as_str(),
            row.freshness.as_str()
        );
    }
}

/// Print one resolved target's fields.
///
/// The engine and sensitivities lines are the same derivations the listing renders,
/// so the detail view and the table cannot disagree about what a technology is or
/// about whether a scan happened.
fn print_target(t: &TargetEntry, out: &mut dyn Write) {
    let _ = writeln!(out, "handle:         {}", t.handle);
    let _ = writeln!(out, "name:           {}", t.name);
    let _ = writeln!(out, "id:             {}", t.stable_id);
    let _ = writeln!(out, "classification: {}", t.classification.as_str());
    let _ = writeln!(out, "fidelity:       {}", t.fidelity.as_str());
    if let Some(anchor) = &t.anchor {
        let _ = writeln!(out, "anchor:         {anchor}");
    }
    // The observed launch executable (issue #173): shown whenever recorded, not
    // only when it happens to also be the reason a selector found this row, so a
    // user can see every name the target is findable by, not just the one that
    // diverges.
    if let Some(executable_hint) = &t.executable_hint {
        let _ = writeln!(out, "executable:     {executable_hint}");
    }
    let _ = writeln!(
        out,
        "engine:         {}",
        fragcap::targets::engine_summary(t)
    );
    let _ = writeln!(
        out,
        "sensitivities:  {}",
        fragcap::targets::sensitivities_summary(t)
    );
    // A genuinely divergent folder name is worth surfacing (issue #173); a cosmetic
    // or truncation-only difference stays quiet so the signal stays worth reading.
    if name_divergence(t) == NameDivergence::Semantic {
        if let Some(folder_name) = &t.folder_name {
            let _ = writeln!(out, "note:           installed as {folder_name:?}");
        }
    }
}

/// The file stem of an executable name (drop a trailing extension), for the
/// handle fallback chain.
fn exe_stem(exe: &str) -> String {
    Path::new(exe)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(exe)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        evidence_from_scan, hero_listing_with_machine_probe, print_discovery,
        render_machine_section, render_table, steam_add_metadata, ClassificationSource,
        DetectionScan, ExeScan, FidelityTier, TargetClassification, TargetEntry,
    };

    #[test]
    fn discovery_account_renders_both_container_outcomes() {
        let discovery = fragcap::targets::Discovery {
            account: fragcap::targets::DiscoveryAccount {
                considered: 2,
                container_descended: 1,
                container_descent_truncated: 1,
                ..fragcap::targets::DiscoveryAccount::default()
            },
            ..fragcap::targets::Discovery::default()
        };
        let mut out = Vec::new();

        print_discovery(&discovery, &mut out);

        let text = String::from_utf8(out).expect("utf-8");
        assert!(text.contains("container_descended=1"));
        assert!(text.contains("container_descent_truncated=1"));
    }

    /// Point discovery at a catalog that cannot exist, so `hero_listing_with_machine_probe`
    /// registers nothing and the empty-listing path is deterministic regardless of
    /// this machine's own Steam or catalog state (mirrors the integration test
    /// harness's `isolate_from_machine_state`, which these unit tests do not share).
    fn isolate_discovery() {
        std::env::set_var(
            "FRAGCAP_CATALOG_DB",
            std::env::temp_dir().join("fragcap-cli-unit-test-nonexistent-catalog.db"),
        );
    }

    #[test]
    fn hero_listing_runs_the_machine_probe_even_with_zero_targets() {
        // Codex review of PR #195: the machine-wide check is unrelated to whether
        // any target is registered, and the empty-listing early return previously
        // skipped it entirely.
        isolate_discovery();
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("local.db");
        let probe = fragcap::targets::FixtureMachineAntiCheatProbe::new(vec![
            fragcap::targets::MachineAntiCheatFinding {
                product: "Easy Anti-Cheat".to_string(),
                evidence: "service EasyAntiCheat_EOS registered".to_string(),
            },
        ]);
        let mut out: Vec<u8> = Vec::new();
        hero_listing_with_machine_probe(&db, false, &mut out, &probe).expect("hero listing");
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("No targets yet."),
            "the empty listing still renders: {text}"
        );
        assert!(
            text.contains("Machine:") && text.contains("Easy Anti-Cheat"),
            "the machine section renders even with zero targets: {text}"
        );
        assert!(
            !text.contains("Next command:"),
            "an empty listing has population suggestions, not a capture suggestion: {text}"
        );
        assert!(text.contains("Add one:") && text.contains("Scan a folder:"));
    }

    #[test]
    fn populated_listing_labels_the_next_command_after_machine_findings() {
        isolate_discovery();
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("local.db");
        let mut store = fragcap::targets::Store::open(&db).expect("target store");
        store
            .insert_target(&TargetEntry {
                id: None,
                stable_id: 1,
                handle: "sample_title".to_string(),
                name: "Sample Title".to_string(),
                classification: TargetClassification::Unknown,
                classification_source: ClassificationSource::User,
                fidelity: FidelityTier::Authored,
                provenance: None,
                anchor: Some("steam:1".to_string()),
                launch_entries: None,
                install_root: Some(dir.path().to_string_lossy().into_owned()),
                evidence: None,
                detection_scan: None,
                folder_name: None,
                executable_hint: None,
            })
            .expect("insert target");
        drop(store);
        let probe = fragcap::targets::FixtureMachineAntiCheatProbe::new(vec![
            fragcap::targets::MachineAntiCheatFinding {
                product: "Sample Protection".to_string(),
                evidence: "sample machine finding".to_string(),
            },
        ]);
        let mut out = Vec::new();

        hero_listing_with_machine_probe(&db, false, &mut out, &probe).expect("hero listing");

        let text = String::from_utf8(out).expect("utf-8");
        assert!(
            text.contains("Machine:\n  Sample Protection (sample machine finding)\n\nNext command:  fragcap capture 1\n"),
            "the suggestion must be a labelled section after machine findings: {text:?}"
        );
        assert_eq!(text.matches("Next command:").count(), 1);
    }

    #[test]
    fn hero_listing_substitutes_a_fixture_probe_through_the_real_entry_point() {
        // FR-009 (Codex review of PR #195): the machine-scope path is testable end
        // to end, through hero_listing_with_machine_probe, not only by calling
        // render_machine_section directly in isolation.
        isolate_discovery();
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("local.db");
        let empty_probe = fragcap::targets::FixtureMachineAntiCheatProbe::new(Vec::new());
        let mut out: Vec<u8> = Vec::new();
        hero_listing_with_machine_probe(&db, false, &mut out, &empty_probe).expect("hero listing");
        let text = String::from_utf8(out).unwrap();
        assert!(
            !text.contains("Machine:"),
            "an empty fixture probe renders no Machine: section: {text}"
        );
    }

    #[test]
    fn a_footer_and_a_machine_section_together_differ_by_exactly_the_footer_line() {
        // This exact combination (empty targets, a non-empty machine finding, and
        // footer:true vs footer:false) previously broke on this machine, whose real
        // EasyAntiCheat_EOS service made cli_args.rs's
        // bare_invocation_lists_targets_with_a_footer fail in CI (review of PR
        // #195): the machine section's own leading blank line, printed before a
        // conditional footer, made the two invocations differ by more than the
        // footer. Forcing a non-empty finding here (rather than depending on this
        // machine's real registry state) makes the regression reproducible on any
        // runner, including a Linux one where the real probe never runs at all.
        isolate_discovery();
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("local.db");
        let finding = || {
            fragcap::targets::FixtureMachineAntiCheatProbe::new(vec![
                fragcap::targets::MachineAntiCheatFinding {
                    product: "Easy Anti-Cheat".to_string(),
                    evidence: "service EasyAntiCheat_EOS registered".to_string(),
                },
            ])
        };

        let mut with_footer: Vec<u8> = Vec::new();
        hero_listing_with_machine_probe(&db, true, &mut with_footer, &finding())
            .expect("hero listing");
        let with_footer = String::from_utf8(with_footer).unwrap();

        let mut without_footer: Vec<u8> = Vec::new();
        hero_listing_with_machine_probe(&db, false, &mut without_footer, &finding())
            .expect("hero listing");
        let without_footer = String::from_utf8(without_footer).unwrap();

        const FOOTER: &str = "Run `fragcap --help` to see all commands.";
        assert!(with_footer.contains(FOOTER));
        assert!(!without_footer.contains(FOOTER));
        assert_eq!(
            with_footer.replace(FOOTER, "").trim_end(),
            without_footer.trim_end(),
            "a footer and a machine section together must still differ by exactly \
             the footer line:\nwith footer: {with_footer:?}\nwithout footer: {without_footer:?}"
        );
    }

    #[test]
    fn machine_section_renders_a_heading_and_one_line_per_finding() {
        let findings = vec![
            fragcap::targets::MachineAntiCheatFinding {
                product: "Easy Anti-Cheat".to_string(),
                evidence: "service EasyAntiCheat_EOS registered".to_string(),
            },
            fragcap::targets::MachineAntiCheatFinding {
                product: "BattlEye".to_string(),
                evidence: "service BEService registered".to_string(),
            },
        ];
        let mut out: Vec<u8> = Vec::new();
        render_machine_section(&findings, &mut out);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("Machine:"));
        assert!(text.contains("  Easy Anti-Cheat (service EasyAntiCheat_EOS registered)"));
        assert!(text.contains("  BattlEye (service BEService registered)"));
    }

    #[test]
    fn machine_section_renders_nothing_for_an_empty_result() {
        let mut out: Vec<u8> = Vec::new();
        render_machine_section(&[], &mut out);
        assert!(
            out.is_empty(),
            "an empty probe result must never render a Machine: section (FR-008)"
        );
    }

    #[test]
    fn rendering_the_machine_section_never_changes_the_target_tables_bytes() {
        let target = TargetEntry {
            id: None,
            stable_id: 1,
            handle: "some_game".to_string(),
            name: "Some Game".to_string(),
            classification: TargetClassification::Unknown,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::HeuristicUnverified,
            provenance: None,
            anchor: None,
            launch_entries: None,
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };
        let targets = [target];

        let mut table_only: Vec<u8> = Vec::new();
        render_table(&targets, &mut table_only);

        let mut with_machine_section: Vec<u8> = Vec::new();
        render_table(&targets, &mut with_machine_section);
        render_machine_section(
            &[fragcap::targets::MachineAntiCheatFinding {
                product: "Easy Anti-Cheat".to_string(),
                evidence: "service EasyAntiCheat_EOS registered".to_string(),
            }],
            &mut with_machine_section,
        );

        assert!(
            with_machine_section.starts_with(&table_only),
            "the target table's bytes must be unchanged by a following machine \
             section (FR-007): table alone = {:?}, with section = {:?}",
            String::from_utf8_lossy(&table_only),
            String::from_utf8_lossy(&with_machine_section)
        );
    }

    #[test]
    fn steam_add_metadata_carries_every_observed_field() {
        let title = fragcap::steam::InstalledTitle {
            app_id: "2413210".to_string(),
            name: "Trapped with Ivy & Piper".to_string(),
            install_dir: std::path::PathBuf::from(
                "C:/Games/Steam/steamapps/common/Escape from Ivy & Piper",
            ),
            installdir: "Escape from Ivy & Piper".to_string(),
            app_type: Some("Game".to_string()),
            launch_executable: Some("TrappedWithIvyAndPiper-EA.exe".to_string()),
            anti_cheat: Vec::new(),
        };
        let (install_root, folder_name, executable_hint) = steam_add_metadata(&title);
        assert_eq!(
            install_root.as_deref(),
            Some("C:/Games/Steam/steamapps/common/Escape from Ivy & Piper")
        );
        assert_eq!(folder_name.as_deref(), Some("Escape from Ivy & Piper"));
        assert_eq!(
            executable_hint.as_deref(),
            Some("TrappedWithIvyAndPiper-EA.exe")
        );
    }

    #[test]
    fn steam_add_metadata_leaves_the_executable_hint_absent_when_unobserved() {
        let title = fragcap::steam::InstalledTitle {
            app_id: "42".to_string(),
            name: "No Launch Entry".to_string(),
            install_dir: std::path::PathBuf::from("C:/Games/Steam/steamapps/common/No Entry"),
            installdir: "No Entry".to_string(),
            app_type: None,
            launch_executable: None,
            anti_cheat: Vec::new(),
        };
        let (_, _, executable_hint) = steam_add_metadata(&title);
        assert_eq!(executable_hint, None);
    }

    #[test]
    fn a_scan_that_was_never_possible_records_no_coverage_claim() {
        // Nothing was attempted: a bare exe name, or no catalog to scan against.
        // Recording any state here would claim a scan that did not happen (P-9).
        let mut out: Vec<u8> = Vec::new();
        let (evidence, scan) = evidence_from_scan(ExeScan::NotRun, &mut out);
        assert!(evidence.is_none());
        assert_eq!(scan, None, "no attempt means no claim");
        assert!(out.is_empty(), "and nothing to report");
    }

    #[test]
    fn a_scan_attempted_against_an_unreadable_directory_is_incomplete_not_unscanned() {
        // The distinction the whole coverage state exists to carry. This arm used
        // to collapse into `NotRun`, so a target whose directory could not be read
        // listed as `not scanned`, claiming nobody looked when the tool had looked
        // and failed. The other three producing sources record `Incomplete` here,
        // and FR-015 requires this one to agree.
        let mut out: Vec<u8> = Vec::new();
        let (evidence, scan) = evidence_from_scan(
            ExeScan::Failed {
                path: std::path::PathBuf::from("D:/Games/Unreadable"),
            },
            &mut out,
        );
        assert!(evidence.is_none(), "a failed scan found nothing");
        assert_eq!(
            scan,
            Some(DetectionScan::Incomplete),
            "an attempt that failed is not the absence of an attempt"
        );
        let text = String::from_utf8(out).expect("utf-8");
        assert!(
            text.contains("Unreadable") && text.contains("could not read"),
            "and the failure is named, not only recorded: {text}"
        );
    }

    use super::ensure_parent_dir;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A unique scratch base under the system temp dir, never created.
    fn scratch(tag: &str) -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "fragcap-targets-store-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ))
    }

    #[test]
    fn ensure_parent_dir_creates_a_missing_parent() {
        // The first-use flow on a clean machine (FR-004): a defaulted store path whose
        // parent directory does not exist yet must have that parent created before the
        // store is opened, since the store layer does not create missing parents.
        let base = scratch("first-use");
        let _ = std::fs::remove_dir_all(&base);
        let store = base.join("fragcap").join("local.db");
        let parent = store.parent().unwrap().to_path_buf();
        assert!(!parent.exists(), "the parent does not exist yet");

        ensure_parent_dir(&store);

        assert!(
            parent.exists(),
            "the parent directory is created for first use"
        );
        assert!(
            !store.exists(),
            "only the parent is created, not the store file"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn ensure_parent_dir_is_idempotent_when_the_parent_exists() {
        let base = scratch("exists");
        let parent = base.join("fragcap");
        std::fs::create_dir_all(&parent).unwrap();
        // A second call over an existing parent is a no-op, never an error.
        ensure_parent_dir(&parent.join("local.db"));
        assert!(parent.exists());
        let _ = std::fs::remove_dir_all(&base);
    }
}
