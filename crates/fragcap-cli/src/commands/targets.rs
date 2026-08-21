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
    handle, identifier, is_row_index, resolve_id, resolve_positional, CandidateIdentity,
    ClassificationSource, DirectorySource, Discovery, Selection, SocketHolderAnswer, Store,
    TargetClassification, TargetEntry, TargetSource,
};

use crate::cli::{
    TargetsAddArgs, TargetsArgs, TargetsCommand, TargetsDiscoverArgs, TargetsExportArgs,
    TargetsShowArgs,
};
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
                    empty_listing(out, false);
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
            empty_listing(out, footer);
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
        empty_listing(out, footer);
        return Ok(Exit::SUCCESS);
    }

    render_table(&targets, out);

    // Pin what was shown so `capture <n>` names the row the user saw (FR-004).
    let rows: Vec<(i64, &str)> = targets
        .iter()
        .map(|t| (t.stable_id, t.handle.as_str()))
        .collect();
    store
        .write_listing_snapshot(&rows)
        .map_err(|e| CliError::failure(e.to_string()))?;

    // End by naming the next command: the first ready row, else the first row.
    let next = targets
        .iter()
        .position(|t| {
            fragcap::targets::capture_readiness(t) == fragcap::targets::CaptureReadiness::Ready
        })
        .map(|i| i + 1)
        .unwrap_or(1);
    let _ = writeln!(out, "\n  fragcap capture {next}");

    if footer {
        let _ = writeln!(out, "\nRun `fragcap --help` to see all commands.");
    }
    Ok(Exit::SUCCESS)
}

/// The empty case: no registered targets and discovery found nothing. Print the
/// concrete commands that populate the store, still naming a next command so hero
/// criterion 5 holds in the empty case (FR-006, SC-006).
fn empty_listing(out: &mut dyn Write, footer: bool) {
    let _ = writeln!(out, "  No targets yet.");
    let _ = writeln!(out);
    let _ = writeln!(out, "  Add one:        fragcap targets add");
    let _ = writeln!(out, "  Scan a folder:  fragcap targets scan <dir>");
    if footer {
        let _ = writeln!(out, "\nRun `fragcap --help` to see all commands.");
    }
}

/// Render the numbered CAPTURE/KNOWN table. Columns size to their content so the
/// output stays aligned; the target order is the caller's (handle order).
fn render_table(targets: &[TargetEntry], out: &mut dyn Write) {
    let num_w = targets.len().to_string().len().max(1);
    let target_w = targets
        .iter()
        .map(|t| t.handle.chars().count())
        .chain(std::iter::once("TARGET".len()))
        .max()
        .unwrap_or(6);
    let capture_w = "needs a target".len();
    let _ = writeln!(
        out,
        "  {:>num_w$}  {:<target_w$}  {:<capture_w$}  KNOWN",
        "#", "TARGET", "CAPTURE"
    );
    for (i, t) in targets.iter().enumerate() {
        let capture = fragcap::targets::capture_readiness(t).label();
        let known = fragcap::targets::known_summary(t);
        let _ = writeln!(
            out,
            "  {:>num_w$}  {:<target_w$}  {:<capture_w$}  {}",
            i + 1,
            t.handle,
            capture,
            known
        );
    }
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
        "account: considered={} produced={} parse_failed={} declined={} not_a_game={} volume_skipped={} access_error={}",
        a.considered,
        a.produced,
        a.parse_failed,
        a.declined_by_user,
        a.considered_not_a_game,
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
fn add(args: &TargetsAddArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    // Resolve `--steam` first: it can supply both the name and the anchor. The
    // enumeration warnings go to `out` as surfaced diagnostics rather than being
    // dropped (P-4). A `--steam` app id that is not installed is a usage error.
    let (name, steam_anchor) = if let Some(app_id) = &args.steam {
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
        (name, Some(format!("steam:{app_id}")))
    } else {
        let name = args.name.clone().ok_or_else(|| {
            CliError::usage("a target name is required (or supply one with --steam <app_id>)")
        })?;
        (name, None)
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
    let evidence = args
        .exe
        .as_deref()
        .and_then(|exe| scan_exe_evidence(exe, out));

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
        install_root: None,
        evidence,
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
/// or DRM findings inline, returning them as the `evidence` JSON. Best-effort: a bare
/// exe name (no directory), a missing default catalog, or a scan error yields no
/// evidence rather than failing the add.
fn scan_exe_evidence(exe: &str, out: &mut dyn Write) -> Option<serde_json::Value> {
    let dir = Path::new(exe)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())?;
    let catalog_db = paths::catalog_db_path(None).or_else(paths::default_catalog_db_path)?;
    if !catalog_db.exists() {
        return None;
    }
    let catalog = Store::open(&catalog_db).ok()?;
    let signatures = catalog.load_signatures().ok()?;
    let set = fragcap::profile::signature::SignatureSet::compile(&signatures);
    let outcome = set.detect(dir).ok()?;
    if outcome.findings.is_empty() {
        return None;
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
    Some(serde_json::Value::Array(findings))
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
            print_target(&t, out);
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

/// Print one resolved target's fields.
fn print_target(t: &TargetEntry, out: &mut dyn Write) {
    let _ = writeln!(out, "handle:         {}", t.handle);
    let _ = writeln!(out, "name:           {}", t.name);
    let _ = writeln!(out, "id:             {}", t.stable_id);
    let _ = writeln!(out, "classification: {}", t.classification.as_str());
    let _ = writeln!(out, "fidelity:       {}", t.fidelity.as_str());
    if let Some(anchor) = &t.anchor {
        let _ = writeln!(out, "anchor:         {anchor}");
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
