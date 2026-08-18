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
use std::path::Path;

use fragcap::profile::FidelityTier;
use fragcap::targets::{
    handle, identifier, resolve_id, resolve_positional, CandidateIdentity, ClassificationSource,
    DirectorySource, Discovery, Selection, Store, TargetClassification, TargetEntry, TargetSource,
};

use crate::cli::{
    TargetsAddArgs, TargetsArgs, TargetsCommand, TargetsDiscoverArgs, TargetsShowArgs,
};
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
        TargetsCommand::List { db } => list(db, out),
        TargetsCommand::Show(args) => show(args, out),
        TargetsCommand::Discover(args) => discover(args, out),
        TargetsCommand::Scan { dir, catalog_db } => scan(dir, catalog_db.as_deref(), out),
    }
}

/// List registered targets from the default local store, for a bare `fragcap` or a
/// `targets` invocation with no subcommand. With `footer`, append a line pointing at
/// `--help`, which is what distinguishes a bare `fragcap` (footer) from an explicit
/// `fragcap targets` (no footer). A store that does not exist yet is an empty
/// listing, not an error.
pub fn list_default(out: &mut dyn Write, footer: bool) -> Result<Exit, CliError> {
    // Resolve like the rest of the surface: the FRAGCAP_LOCAL_DB override, else the
    // per-user default. A store that does not exist yet is an empty listing.
    let path = paths::local_db_path(None).or_else(paths::default_local_db_path);
    let exit = match path {
        Some(path) if path.exists() => list(&path, out)?,
        _ => {
            let _ = writeln!(out, "no targets registered");
            Exit::SUCCESS
        }
    };
    if footer {
        let _ = writeln!(out, "\nRun `fragcap --help` to see all commands.");
    }
    Ok(exit)
}

/// Run `targets scan <dir>`: point discovery at one directory and list it as a
/// single candidate (the tier-3 [`DirectorySource`], slice S052). With a catalog,
/// the directory is scanned for technologies and they ride as evidence (slice S053).
fn scan(dir: &Path, catalog_db: Option<&Path>, out: &mut dyn Write) -> Result<Exit, CliError> {
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
    Ok(Exit::SUCCESS)
}

/// Run `targets discover`: walk Steam (tier 1) and the known game-install roots
/// (tier 2) and list the candidates found (slice S052). Reads only; nothing is
/// persisted except the first-run volume eligibility seeding the cross-volume walk
/// needs to be safe.
fn discover(args: &TargetsDiscoverArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let catalog = Store::open(&args.catalog_db).map_err(|e| CliError::failure(e.to_string()))?;

    // Locate the Steam root: the explicit flag, else Steam's own installation.
    let steam_root = match &args.steam_root {
        Some(root) => Some(root.clone()),
        None => fragcap::steam::discover().ok().map(|i| i.root),
    };
    let steam = steam_root
        .as_ref()
        .map(|root| fragcap::SteamSource::new(root, &catalog));

    // The known-roots walk enumerates fixed volumes, which is a platform operation;
    // it runs on Windows, where the tool captures. The eligibility allowlist lives
    // in local.db and is seeded permissively on first run (FR-016a).
    #[cfg(windows)]
    let mut local = Store::open(&args.local_db).map_err(|e| CliError::failure(e.to_string()))?;
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

    let discovery =
        fragcap::targets::discover_all(&sources).map_err(|e| CliError::failure(e.to_string()))?;
    print_discovery(&discovery, out);
    Ok(Exit::SUCCESS)
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
        let installation =
            fragcap::steam::discover().map_err(|e| CliError::usage(e.to_string()))?;
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

    let mut store = Store::open(&args.db).map_err(|e| CliError::failure(e.to_string()))?;

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
    let launch_entries = args
        .exe
        .as_ref()
        .map(|exe| serde_json::json!([{ "executable": exe }]));

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
        evidence: None,
    };
    store
        .insert_target(&entry)
        .map_err(|e| CliError::failure(e.to_string()))?;

    let _ = writeln!(out, "registered {handle_value} (id {stable_id})");
    Ok(Exit::SUCCESS)
}

/// List registered targets: the 1-based row index, handle, identifier, and name.
fn list(db: &Path, out: &mut dyn Write) -> Result<Exit, CliError> {
    let store = Store::open(db).map_err(|e| CliError::failure(e.to_string()))?;
    let targets = store
        .targets()
        .map_err(|e| CliError::failure(e.to_string()))?;
    if targets.is_empty() {
        let _ = writeln!(out, "no targets registered");
        return Ok(Exit::SUCCESS);
    }
    for (i, t) in targets.iter().enumerate() {
        let _ = writeln!(out, "{}\t{}\t{}\t{}", i + 1, t.handle, t.stable_id, t.name);
    }
    Ok(Exit::SUCCESS)
}

/// Show one target resolved by a selector.
///
/// The no-match exit code follows the selector kind (the section 5.4 contract): a
/// handle or name that matches nothing is a clean miss (exit 0), while an unknown
/// `--id` or an out-of-range row index is a bad machine reference (exit 2). An
/// ambiguous name lists its matches and exits 2 rather than guessing (P-9).
fn show(args: &TargetsShowArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    let store = Store::open(&args.db).map_err(|e| CliError::failure(e.to_string()))?;
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
            let _ = writeln!(out, "no target matches");
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

/// Whether a positional selector is a bare integer, i.e. a row index rather than a
/// handle or name. Handles are never purely numeric, so this never shadows one.
fn is_row_index(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit())
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
