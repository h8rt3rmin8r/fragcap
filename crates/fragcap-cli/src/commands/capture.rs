// SPDX-License-Identifier: Apache-2.0

//! `capture`: identify a target, build the effective configuration, assemble the
//! pipeline and session, and capture. The single capture verb (section 17.2),
//! superseding the retired `run`, `tap`, and `watch`.
//!
//! The command is the front half; the capture engine in [`crate::orchestrator`]
//! is the shared back half. Identification and overlay decide what to capture and
//! with what options; the orchestrator arms, waits for the target, captures, stops
//! on a bound or interrupt, and reports.
//!
//! `capture` has two mutually-exclusive, required target inputs (a clap group
//! enforces exactly one):
//!
//! - `--target <selector>` resolves a stored target from the user store (local.db)
//!   by an S051 selector. A target carrying a Steam anchor is resolved through the
//!   same install-layout cascade the retired `run --steam` used, keyed on the
//!   anchor's app id, so its client executable and (for `--launch`) its app id are
//!   recovered; a target carrying a stored launch executable synthesizes directly
//!   from it. No process handle is opened and no process memory is read (P-1).
//! - `--process <image>` names a raw process image directly, with optional
//!   `--path`/`--path-regex` anchors to disambiguate two processes sharing the
//!   name (the capability the retired `watch` carried).
//!
//! In every case a one-stage capture identity is synthesized and validated through
//! `Profile::parse`, the same path an authored profile took, so an unusable
//! identity surfaces as a profile diagnostic (exit 2).

use std::path::{Path, PathBuf};

use fragcap::profile::{
    EngineRuleProvider, MatchPredicates, ObservationProvider, Profile, ProfileProvider,
    ResolutionError, ResolutionRequest, TargetProvider, TargetResolver,
};
use fragcap::steam::SteamWalkerProvider;
use fragcap::targets::{resolve_id, resolve_positional, Selection, Store, TargetEntry};

use crate::assemble;
use crate::attach;
use crate::cli::CaptureArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;
use crate::paths;

/// How a stored target was named: a positional selector or a durable identifier.
enum StoredRef<'a> {
    /// A positional selector: a handle, an exact name, or a 1-based row index.
    Selector(&'a str),
    /// A durable stable identifier, the machine-facing form automation addresses.
    Id(i64),
}

/// Run `capture`.
pub fn run(args: &CaptureArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    // Exactly one target input is present (the clap group guarantees it). A stored
    // target resolves against the local store (and, when Steam-anchored, through the
    // install-layout cascade); a raw process image synthesizes an identity directly.
    // The positional selector and `--target` are the same input by two spellings and
    // are mutually exclusive in the group, so at most one is set; prefer the
    // positional when present.
    let selector = args.selector.as_deref().or(args.target.as_deref());
    let profile = match (selector, args.id, &args.process) {
        (Some(selector), None, None) => {
            resolve_stored(StoredRef::Selector(selector), args, emitter)?
        }
        (None, Some(id), None) => resolve_stored(StoredRef::Id(id), args, emitter)?,
        (None, None, Some(process)) => {
            synthesize_named_profile(process, args.path.as_deref(), args.path_regex.as_deref())?
        }
        // The clap group guarantees exactly one target input; this documents it.
        _ => {
            return Err(CliError::usage(
                "exactly one of a target selector, --id, or --process is required",
            ))
        }
    };

    let config = assemble::effective_config(args, &profile)?;
    let components = assemble::components(&args.offline, &config)?;

    // Capture is launch-agnostic: report an already-running attach, and warn when a
    // resolved path anchor cannot be checked against the executable-only startup
    // snapshot, so acquisition is never silently impossible (review of PR #88).
    attach::report_attach_to_running(&profile, &components, emitter);

    orchestrator::install_interrupt_handler();
    let allowed_roles = config.roles.clone();
    orchestrator::capture(
        profile,
        &config,
        components,
        emitter,
        &orchestrator::INTERRUPT,
        args.offline.fire_interrupt,
        allowed_roles,
        // A sink failure is an unrecoverable end for `capture`, not a clean stop.
        false,
    )
}

/// Resolve a stored target against the local store into the profile that captures
/// it.
///
/// The target is named by a positional selector (handle, exact name, or 1-based row
/// index) or a durable `--id`. A Steam-anchored target is resolved through the
/// install-layout cascade keyed on its app id, recovering its client executable and
/// carrying the app id so `--launch` can reach it. A target carrying stored launch
/// entries synthesizes from its single Windows client, refusing an empty or
/// ambiguous set rather than guessing (P-9). A target that names neither is a usage
/// error rather than a silent empty capture (P-4).
fn resolve_stored(
    target: StoredRef,
    args: &CaptureArgs,
    emitter: &mut Emitter,
) -> Result<Profile, CliError> {
    let (catalog_db, local_db) = setup_stores(args, emitter)?;

    // Resolve against the local store (S051 section 5.4). A store the operator never
    // created has no targets, so a target matches nothing there.
    let local_path = local_db.clone().ok_or_else(|| {
        CliError::usage("no local store is available to resolve the target; register one first")
    })?;
    let store = Store::open(&local_path)
        .map_err(|e| CliError::failure(format!("cannot open local store: {e}")))?;
    let selection = match target {
        StoredRef::Selector(s) => resolve_positional(&store, s),
        StoredRef::Id(id) => resolve_id(&store, id),
    }
    .map_err(|e| CliError::failure(e.to_string()))?;
    let entry = match selection {
        Selection::Resolved(t) => t,
        Selection::NoMatch => {
            return Err(CliError::usage(
                "no target matches; list targets with `fragcap targets`".to_string(),
            ))
        }
        Selection::Ambiguous(matches) => {
            // List the matches rather than only the count, so the operator can pick
            // one by its durable id (P-9, the selector contract).
            let mut msg = format!(
                "the selector is ambiguous ({} targets match); select by handle or `--id`:",
                matches.len()
            );
            for t in &matches {
                msg.push_str(&format!("\n  {}\t{}\t{}", t.handle, t.stable_id, t.name));
            }
            return Err(CliError::usage(msg));
        }
    };

    // A Steam anchor recovers the client through the cascade; stored launch entries
    // synthesize directly; neither is a named usage error.
    if let Some(app_id) = steam_app_id(&entry) {
        // The install-layout cascade determines the client identity for a
        // Steam-anchored target, so an operator path anchor has nothing to attach to
        // here; refuse it rather than silently discard it (P-9).
        if args.path.is_some() || args.path_regex.is_some() {
            return Err(CliError::usage(
                "--path/--path-regex do not apply to a Steam-anchored target; the install-layout \
                 cascade determines the client identity",
            ));
        }
        let search = paths::search_path(&[]);
        let bundled = paths::bundled();
        let resolver = build_resolver(catalog_db.as_deref(), local_db.as_deref())?;
        let lookup = fragcap::steam::install_root_for(&app_id)
            .map_err(|e| CliError::failure(format!("cannot look up Steam app {app_id}: {e}")))?;
        for warning in &lookup.warnings {
            emitter.warn(warning);
        }
        let install_root = lookup.install_dir.ok_or_else(|| {
            CliError::failure(format!(
                "Steam app {app_id} is not installed in any library"
            ))
        })?;
        resolve_from_install(&resolver, &install_root, Some(&app_id), &search, &bundled)
    } else {
        // Reduce the stored launch entries to their distinct Windows clients by the
        // same filter, file-name reduction, and case-insensitive dedup the hint
        // provider applies (P-10). Exactly one is capturable; zero or more than one
        // is refused rather than an arbitrary pick (P-9). The operator path anchors
        // ride onto the synthesized identity, so `--target ... --path ...` is honored
        // rather than discarded.
        let clients = fragcap::targets::entry_windows_clients(&entry);
        match clients.as_slice() {
            [exe] => {
                synthesize_named_profile(exe, args.path.as_deref(), args.path_regex.as_deref())
            }
            [] => Err(CliError::usage(format!(
                "target {} names no Windows client executable; re-register it with \
                 `targets add --exe` or `--steam`",
                entry.handle
            ))),
            many => Err(CliError::usage(format!(
                "target {} names {} distinct Windows client executables ({}); it is ambiguous, so \
                 register a single-client target or capture the process directly with `--process`",
                entry.handle,
                many.len(),
                many.join(", ")
            ))),
        }
    }
}

/// The Steam app id a target's anchor names, if it is a `steam:<app_id>` anchor.
fn steam_app_id(entry: &TargetEntry) -> Option<String> {
    entry
        .anchor
        .as_deref()
        .and_then(|a| a.strip_prefix("steam:"))
        .map(|id| id.to_string())
}

/// Build a validated one-stage `authored` profile from an executable name and the
/// optional path anchors, the identity the retired `tap` and `watch` synthesized.
///
/// The identity is placed into a JSON profile (serde_json handles escaping) and
/// validated through `Profile::parse`, so an empty match or a glob or path regex
/// that does not compile surfaces as the profile's own diagnostics (exit 2).
fn synthesize_named_profile(
    exe: &str,
    path_contains: Option<&str>,
    path_regex: Option<&str>,
) -> Result<Profile, CliError> {
    let mut predicates = serde_json::Map::new();
    predicates.insert(
        "exe".to_string(),
        serde_json::Value::String(exe.to_string()),
    );
    if let Some(path) = path_contains {
        predicates.insert(
            "path_contains".to_string(),
            serde_json::Value::String(path.to_string()),
        );
    }
    if let Some(re) = path_regex {
        predicates.insert(
            "path_regex".to_string(),
            serde_json::Value::String(re.to_string()),
        );
    }
    let profile = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "authored",
        "game": { "id": "capture", "name": "ad hoc capture" },
        "stage": [
            { "role": "target", "lifecycle": "session", "terminal": true, "match": predicates }
        ]
    });
    Profile::parse(&profile.to_string()).map_err(CliError::from)
}

/// Set up the catalog and local stores (slice S050): resolve their paths from the
/// flags, environment, or per-user defaults, refuse a shared file, bootstrap the
/// defaulted locations, and learn this machine's Steam launch executables into the
/// local store. Returns the resolved (catalog, local) paths, either possibly `None`
/// when a store could not be made available (a warning was emitted).
fn setup_stores(
    args: &CaptureArgs,
    emitter: &mut Emitter,
) -> Result<(Option<PathBuf>, Option<PathBuf>), CliError> {
    let (mut catalog_db, catalog_default) = match paths::catalog_db_path(args.catalog_db.as_deref())
    {
        Some(path) => (Some(path), false),
        None => (paths::default_catalog_db_path(), true),
    };
    let (mut local_db, local_default) = match paths::local_db_path(args.local_db.as_deref()) {
        Some(path) => (Some(path), false),
        None => (paths::default_local_db_path(), true),
    };

    // The two stores must be different files, or accumulation would write learned
    // user data into the catalog and a future catalog replacement would discard it
    // (FR-007). Refuse before bootstrap or accumulation touches it.
    if let (Some(catalog), Some(local)) = (catalog_db.as_deref(), local_db.as_deref()) {
        if same_store_file(catalog, local) {
            return Err(CliError::failure(format!(
                "the catalog store and the local store must be different files, \
                 but both resolve to {}",
                catalog.display()
            )));
        }
    }

    // Bootstrap only defaulted locations, never a path the operator named. A
    // bootstrap failure is a warning that drops that store, never fatal (FR-005).
    if catalog_default {
        if let Some(default) = catalog_db.clone() {
            let template = bundled_catalog_template();
            if let Err(e) = ensure_store(&default, template.as_deref()) {
                emitter.warn(&format!(
                    "catalog store could not be initialized at {}: {e}",
                    default.display()
                ));
                catalog_db = None;
            }
        }
    }
    if local_default {
        if let Some(default) = local_db.clone() {
            if let Err(e) = ensure_store(&default, None) {
                emitter.warn(&format!(
                    "local store could not be initialized at {}: {e}",
                    default.display()
                ));
                local_db = None;
            }
        }
    }

    // Learn this machine's own Steam launch executables into the local store, so the
    // hint provider can name a socket-holding client the engine rule and the walker
    // would miss (issue #78). It reads the local appinfo cache, writes only launch
    // data, ships nothing, and never opens a process handle (P-1).
    if let Some(path) = local_db.as_deref() {
        accumulate_launch(path, emitter);
    }

    Ok((catalog_db, local_db))
}

/// Whether two store paths identify the same file.
///
/// Compares the canonical identity of each path. A store file may not exist yet
/// (the defaults are created on first run), so when a path cannot be canonicalized
/// whole, its existing parent directory is canonicalized and the file name
/// rejoined, which resolves `.`, `..`, a relative prefix, and a symlinked parent
/// even before the leaf exists. Only when neither the path nor its parent can be
/// resolved does it fall back to a lexical comparison.
fn same_store_file(a: &Path, b: &Path) -> bool {
    resolve_store_identity(a) == resolve_store_identity(b)
}

/// The canonical identity of a store path, resolving as much as exists.
fn resolve_store_identity(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    if let Some(name) = path.file_name() {
        // An empty parent (a bare file name) resolves against the current
        // directory, so `x.db` and `./x.db` fold together.
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        if let Ok(canonical_parent) = parent.canonicalize() {
            return canonical_parent.join(name);
        }
    }
    path.to_path_buf()
}

/// The read-only catalog store template shipped beside the executable, if present.
///
/// Both distribution forms place `catalog.db` next to `fragcap.exe`: the installer
/// under its install directory, the portable archive beside the unzipped binary.
/// The first-run bootstrap copies it to the writable per-user default. A bare-exe
/// copy has no sibling template, and the bootstrap creates an empty store instead.
/// This opens no process and reads no memory; it only inspects the executable's
/// own directory (P-1).
fn bundled_catalog_template() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let sibling = exe.parent()?.join("catalog.db");
    sibling.is_file().then_some(sibling)
}

/// Ensure a default store exists, so resolution and local accumulation have a
/// store the first time fragcap runs with no configuration.
///
/// Pure over its arguments and so tested with scratch directories; it touches only
/// `default`:
///
/// - `default` already exists: it is left exactly as it was (idempotent).
/// - `default` absent with a `template`: the template is copied to `default`
///   (the catalog seed).
/// - `default` absent with no template: an empty current-schema store is created
///   (the local store always, the catalog on a bare-exe build).
///
/// The parent directory is created as needed. This is only ever called for a
/// per-user default, never for a path the operator named explicitly, so it can
/// never create or overwrite an operator-supplied file (FR-004).
fn ensure_store(default: &Path, template: Option<&Path>) -> std::io::Result<()> {
    if default.exists() {
        return Ok(());
    }
    if let Some(parent) = default.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let result = match template {
        Some(template) if template.is_file() => copy_writable_store(template, default),
        // No template: materialize an empty current-schema store. Opening a store
        // at a fresh path creates a valid, empty database.
        _ => fragcap::targets::Store::open(default)
            .map(|_| ())
            .map_err(|e| std::io::Error::other(e.to_string())),
    };
    if result.is_err() {
        // A copy or create that fails partway can leave a truncated file behind.
        // Remove it, so a later `build_resolver` cannot see a present-but-invalid
        // store and turn a non-fatal bootstrap failure into a fatal one (the caller
        // also drops the path; this is the on-disk half). FR-005.
        let _ = std::fs::remove_file(default);
    }
    result
}

/// Copy the shipped template to the writable per-user default.
///
/// The installer ships the template read-only, and a plain file copy preserves
/// that attribute on Windows. The per-user copy must be writable, because local
/// accumulation persists into it on every run, so the read-only attribute is
/// cleared on the destination after the copy.
fn copy_writable_store(template: &Path, default: &Path) -> std::io::Result<()> {
    std::fs::copy(template, default)?;
    let perms = std::fs::metadata(default)?.permissions();
    if perms.readonly() {
        make_writable(default, perms)?;
    }
    Ok(())
}

/// Grant write access to a freshly copied file. On Windows this clears the
/// read-only file attribute the MSI template carries; on Unix it adds owner write
/// without widening group or other, so the Unix "world writable" caveat behind the
/// `set_readonly(false)` lint does not apply.
#[cfg(not(unix))]
fn make_writable(path: &Path, mut perms: std::fs::Permissions) -> std::io::Result<()> {
    #[allow(clippy::permissions_set_readonly_false)]
    {
        perms.set_readonly(false);
    }
    std::fs::set_permissions(path, perms)
}

#[cfg(unix)]
fn make_writable(path: &Path, perms: std::fs::Permissions) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = perms.mode() | 0o200;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

/// Learn this machine's Steam launch executables into the user-owned local store.
///
/// Runs only when the store is present and openable; a missing or unopenable store
/// is left for [`build_resolver`] to handle (a present-but-unopenable one is a loud
/// error there, a missing one is not an error at all), so this never duplicates or
/// pre-empts that decision. Progress and the final account go to the emitter, which
/// respects the operator's quiet or silent choice. A Steam that is absent or has no
/// appinfo yields a zero-considered result and stays quiet; a genuine fault is a
/// warning, never fatal to the capture.
fn accumulate_launch(local_db: &Path, emitter: &mut Emitter) {
    // Only act on a present, openable store. `Ok(false)`/`Err` and an open
    // failure both defer to build_resolver rather than reporting twice.
    if !matches!(local_db.try_exists(), Ok(true)) {
        return;
    }
    let mut store = match fragcap::targets::Store::open(local_db) {
        Ok(store) => store,
        Err(_) => return,
    };

    let mut ticked = 0usize;
    let outcome = fragcap::accumulate_from_local_steam(&mut store, &mut |p| {
        // Bounded progress: the first app, the last, and every 25 in between.
        if p.total > 0 && (p.done == 1 || p.done == p.total || p.done - ticked >= 25) {
            ticked = p.done;
            emitter.progress(&format!(
                "learning launch data from Steam: {}/{} apps",
                p.done, p.total
            ));
        }
    });

    match outcome {
        // The appinfo cache was present but malformed at the top level (bad magic,
        // a broken string table, or a truncated tail): nothing was learned, and
        // this is a fault the operator must see rather than a quiet zero (P-4/P-9).
        Ok(summary) if summary.file_faults > 0 => emitter.warn(
            "the Steam appinfo cache could not be read (unrecognized or truncated); \
             no launch data was learned this run",
        ),
        Ok(summary) if summary.considered > 0 => emitter.progress(&format!(
            "launch data: {} learned, {} up to date, {} without a launch entry, \
             {} unreadable (of {} installed)",
            summary.written, summary.skipped, summary.empty, summary.failed, summary.considered
        )),
        // No Steam, or no appinfo cache: nothing to learn, and nothing to say.
        Ok(_) => {}
        Err(e) => emitter.warn(&format!("launch-data accumulation skipped: {e}")),
    }
}

/// Assemble the resolution cascade, registering one layered hint provider at
/// precedence 2 over the present stores (slice S050). The stores are consulted in
/// resolution order, the user-owned local store before the shipped catalog.
///
/// A missing store (no path, or a path that does not exist) contributes nothing,
/// so a build with neither store resolves identically to one with no hint provider
/// (FR-013). A present-but-unopenable store (corrupt or a wrong schema version)
/// fails here, at the boundary where the operator named it, rather than as a
/// per-request surprise (FR-014). Both stores feed one provider, so precedence 2
/// stays a single position; the built-in providers occupy distinct positions by
/// construction, so `TargetResolver::new` cannot fail for that reason.
fn build_resolver(
    catalog_db: Option<&Path>,
    local_db: Option<&Path>,
) -> Result<TargetResolver, CliError> {
    let mut providers: Vec<Box<dyn TargetProvider>> = vec![
        Box::new(ProfileProvider::new()),
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
        Box::new(ObservationProvider::new()),
    ];

    // Open the present stores in resolution order: local first, then catalog. Each
    // present one is opened here (the shipped tool always carries `fragcap::targets`;
    // a library consumer without the feature never reaches this code). `try_exists`
    // distinguishes a genuinely absent file (Ok(false), not an error, FR-013) from a
    // path whose existence cannot be determined (Err), which is surfaced loudly like
    // an unopenable store rather than silently skipped for a store the operator named
    // (FR-014).
    let mut stores = Vec::new();
    for (label, path) in [("local", local_db), ("catalog", catalog_db)] {
        let Some(path) = path else { continue };
        match path.try_exists() {
            Ok(true) => {
                let store = fragcap::targets::Store::open(path).map_err(|e| {
                    CliError::failure(format!("cannot open {label} store {}: {e}", path.display()))
                })?;
                stores.push(store);
            }
            Ok(false) => {}
            Err(e) => {
                return Err(CliError::failure(format!(
                    "cannot access {label} store {}: {e}",
                    path.display()
                )))
            }
        }
    }
    if !stores.is_empty() {
        providers.push(Box::new(
            fragcap::targets::HintDatabaseProvider::new_layered(stores),
        ));
    }

    TargetResolver::new(providers)
        .map_err(|e| CliError::failure(format!("provider precedence conflict: {e}")))
}

/// Resolve a target from an install location and synthesize the one-stage profile
/// that captures it.
///
/// Used for a `--target` carrying a Steam anchor: the anchor's app id looked up the
/// install root, and the cascade recovers the client executable from it. The app id
/// is carried as a fact on the synthesized profile so `--launch` can reach it.
fn resolve_from_install(
    resolver: &TargetResolver,
    install_root: &Path,
    app_id: Option<&str>,
    search: &fragcap::profile::SearchPath,
    bundled: &fragcap::profile::BundledSet,
) -> Result<Profile, CliError> {
    // Offer the hint provider the Steam app id (when numeric) while the install root
    // stays available to the engine rule and platform walker: the higher-precedence
    // hint answer wins when the database names a client, and the lower providers
    // answer when it does not.
    let mut request = ResolutionRequest::for_install(install_root, search, bundled);
    if let Some(id) = app_id.and_then(|s| s.parse::<u32>().ok()) {
        request = request.with_steam_app_id(id);
    }
    let target = match resolver.resolve(&request) {
        Ok(target) => target,
        Err(error) => return Err(nonprofile_resolution_error(error, install_root)),
    };

    match target.identity() {
        Some(identity) => synthesize_profile(identity, app_id),
        // An install request carries no profile reference, so the cascade cannot
        // return a profile-backed target here; this documents the invariant.
        None => target.into_profile().ok_or_else(|| {
            CliError::failure("resolved a target with neither a profile nor an identity")
        }),
    }
}

/// Build a validated one-stage profile from a resolved non-profile identity.
///
/// The identity's predicates are serialized back into a JSON profile and parsed
/// through [`Profile::parse`], the same validating path an authored profile and
/// `watch`/`tap` take, so an unusable identity surfaces as a profile diagnostic
/// (exit 2). The fidelity is `heuristic-unverified`: the identity was resolved by
/// an install-layout heuristic or runtime observation, not typed by an operator,
/// so stamping it `authored` (as `watch` does for its typed identity) would be a
/// lie (P-9). The game identity is a generic placeholder; a `--steam` app id is
/// carried as a fact on `game.app_id`.
fn synthesize_profile(
    identity: &MatchPredicates,
    app_id: Option<&str>,
) -> Result<Profile, CliError> {
    let mut predicates = serde_json::Map::new();
    if let Some(exe) = identity.exe() {
        predicates.insert(
            "exe".to_string(),
            serde_json::Value::String(exe.as_str().to_string()),
        );
    }
    if let Some(path) = identity.path_contains() {
        predicates.insert(
            "path_contains".to_string(),
            serde_json::Value::String(path.to_string()),
        );
    }
    if let Some(re) = identity.path_regex() {
        predicates.insert(
            "path_regex".to_string(),
            serde_json::Value::String(re.as_str().to_string()),
        );
    }
    if let Some(cmdline) = identity.cmdline_contains() {
        predicates.insert(
            "cmdline_contains".to_string(),
            serde_json::Value::String(cmdline.to_string()),
        );
    }
    if let Some(role) = identity.descends_from() {
        predicates.insert(
            "descends_from".to_string(),
            serde_json::Value::String(role.to_string()),
        );
    }

    let mut game = serde_json::Map::new();
    game.insert(
        "id".to_string(),
        serde_json::Value::String("target".to_string()),
    );
    game.insert(
        "name".to_string(),
        serde_json::Value::String("ad hoc target".to_string()),
    );
    if let Some(app_id) = app_id {
        game.insert(
            "platform".to_string(),
            serde_json::Value::String("steam".to_string()),
        );
        game.insert(
            "app_id".to_string(),
            serde_json::Value::String(app_id.to_string()),
        );
    }

    let profile = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "heuristic-unverified",
        "game": game,
        "stage": [
            { "role": "target", "lifecycle": "session", "terminal": true, "match": predicates }
        ]
    });
    Profile::parse(&profile.to_string()).map_err(CliError::from)
}

/// Turn a non-profile resolution failure into a surfaced command error that names
/// the reason, so a declined install location is distinguishable from a game that
/// sent no traffic (P-4, FR-007).
///
/// The generic `From<ResolutionError>` reduces an unresolved outcome to a
/// profile-not-found class, and the error's `Display` names the unreadable cases
/// but not the ambiguity ones, so the ambiguity notes are rendered explicitly
/// here.
fn nonprofile_resolution_error(error: ResolutionError, install_root: &Path) -> CliError {
    let unresolved = match error {
        ResolutionError::Unresolved(u) => u,
        // A hard provider error aborts the cascade; reuse its existing mapping.
        other => return CliError::from(other),
    };

    let detail = if let Some(ambiguity) = unresolved.hint_ambiguous() {
        // The hint provider is the highest-precedence heuristic source, so its note
        // is the most specific one to report (FR-008): name the app id and how many
        // candidate clients its row could not reduce to one.
        format!(
            "the hint database named {} candidate clients for app {}",
            ambiguity.candidates(),
            ambiguity.app_id()
        )
    } else if let Some(ambiguity) = unresolved.engine_rule_ambiguous() {
        format!(
            "an engine layout was recognized but matched {} candidate clients",
            ambiguity.candidates()
        )
    } else if let Some(ambiguity) = unresolved.walker_ambiguous() {
        format!(
            "the platform walker found {} plausible clients",
            ambiguity.candidates()
        )
    } else if let Some(path) = unresolved.engine_rule_unreadable() {
        format!("the engine rule could not fully read {}", path.display())
    } else if let Some(path) = unresolved.walker_unreadable() {
        format!("the platform walker could not read {}", path.display())
    } else {
        "no engine layout or single client executable was recognized".to_string()
    };

    CliError::failure(format!(
        "could not resolve a capture target from {}: {detail}",
        install_root.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap::profile::FidelityTier;

    fn identity() -> MatchPredicates {
        MatchPredicates::with_exe("game.exe").expect("a valid exe glob")
    }

    /// A unique scratch path under the system temp dir, never created.
    fn scratch(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "fragcap-build-resolver-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ))
    }

    #[test]
    fn build_resolver_without_stores_succeeds() {
        // FR-013: no database supplied is not an error; the cascade is assembled
        // without the hint provider, exactly as before this slice.
        assert!(build_resolver(None, None).is_ok());
    }

    #[test]
    fn a_missing_store_file_is_not_an_error() {
        // FR-013: a path that does not exist leaves precedence 2 empty and raises
        // no error.
        let absent = scratch("absent");
        assert!(!absent.exists());
        assert!(build_resolver(Some(&absent), None).is_ok());
    }

    #[test]
    fn a_present_valid_store_registers_the_provider() {
        // FR-012: a present, openable database registers the provider (the
        // assembly succeeds with the extra precedence position).
        let path = scratch("valid.db");
        // Creating a store makes a valid database file at the path.
        fragcap::targets::Store::open(&path).expect("create store");
        assert!(build_resolver(Some(&path), None).is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_present_unopenable_store_is_an_error() {
        // FR-014: a present-but-unopenable database (here, a file that is not a
        // SQLite database) fails loudly rather than being treated as absent.
        let path = scratch("corrupt.db");
        std::fs::write(&path, b"this is not a sqlite database").expect("write garbage");
        let result = build_resolver(Some(&path), None);
        assert!(result.is_err(), "a corrupt database must be a loud error");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn build_resolver_with_both_stores_registers_one_provider() {
        // FR-008: both present stores are opened and fed to the single layered
        // provider; the assembly succeeds with precedence 2 occupied once.
        let catalog = scratch("both-catalog.db");
        let local = scratch("both-local.db");
        fragcap::targets::Store::open(&catalog).expect("catalog store");
        fragcap::targets::Store::open(&local).expect("local store");
        assert!(build_resolver(Some(&catalog), Some(&local)).is_ok());
        let _ = std::fs::remove_file(&catalog);
        let _ = std::fs::remove_file(&local);
    }

    #[test]
    fn the_bootstrap_creates_an_empty_store_when_absent_with_no_template() {
        // FR-003: with no template, the default is materialized as an empty,
        // valid, current-schema store.
        let dir = scratch("bootstrap-empty");
        let default = dir.join("hint.db");
        assert!(!default.exists());
        ensure_store(&default, None).expect("bootstrap creates a store");
        assert!(
            default.is_file(),
            "the default database must exist after bootstrap"
        );
        fragcap::targets::Store::open(&default).expect("the created file is a valid store");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_local_bootstrap_leaves_the_catalog_byte_identical() {
        // FR-007: the two stores are separate files, so bootstrapping the local
        // store never reads or writes the catalog. Seed a catalog, hash it, create
        // the local store beside it, and confirm the catalog is unchanged.
        let dir = scratch("byte-identical");
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let catalog = dir.join("catalog.db");
        fragcap::targets::Store::open(&catalog).expect("catalog store");
        let before = std::fs::read(&catalog).expect("read catalog");

        let local = dir.join("local.db");
        ensure_store(&local, None).expect("bootstrap local");
        assert!(local.is_file(), "the local store must be created");

        let after = std::fs::read(&catalog).expect("read catalog");
        assert_eq!(before, after, "the catalog store must be byte-identical");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn identical_store_paths_are_detected() {
        // The guard that refuses a catalog and a local path that name one file.
        let dir = scratch("same-file");
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let catalog = dir.join("catalog.db");
        let local = dir.join("local.db");
        fragcap::targets::Store::open(&catalog).expect("catalog");
        fragcap::targets::Store::open(&local).expect("local");
        assert!(same_store_file(&catalog, &catalog), "one path is one file");
        assert!(
            !same_store_file(&catalog, &local),
            "two distinct files are not the same store"
        );

        // Lexically different paths to one file are caught even before the file
        // exists, by resolving the parent: `dir/x.db` and `dir/./x.db` fold.
        let absent = dir.join("absent.db");
        let absent_dotted = dir.join(".").join("absent.db");
        assert!(!absent.exists());
        assert!(
            same_store_file(&absent, &absent_dotted),
            "a dotted path to the same not-yet-created file is the same store"
        );
        assert!(
            !same_store_file(&absent, &dir.join("other.db")),
            "distinct not-yet-created files are not the same store"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_bootstrap_copies_the_template_when_present() {
        // FR-003: when a template ships beside the executable, it seeds the
        // per-user default byte-for-byte.
        let dir = scratch("bootstrap-copy");
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let template = dir.join("template.db");
        fragcap::targets::Store::open(&template).expect("template store");
        let template_bytes = std::fs::read(&template).expect("read template");
        let default = dir.join("sub").join("hint.db");
        ensure_store(&default, Some(&template)).expect("bootstrap copies");
        assert_eq!(
            std::fs::read(&default).expect("read default"),
            template_bytes,
            "the default must be a copy of the template"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_bootstrap_makes_a_read_only_template_copy_writable() {
        // PR #97 review (P1): the MSI ships the template read-only, and a plain
        // file copy preserves that attribute, which would make the per-user copy
        // read-only and break the read-write open that local accumulation needs.
        let dir = scratch("bootstrap-readonly");
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let template = dir.join("template.db");
        fragcap::targets::Store::open(&template).expect("template store");
        let mut ro = std::fs::metadata(&template).expect("meta").permissions();
        ro.set_readonly(true);
        std::fs::set_permissions(&template, ro).expect("mark template read-only");

        let default = dir.join("hint.db");
        ensure_store(&default, Some(&template)).expect("bootstrap copies");
        assert!(
            !std::fs::metadata(&default)
                .expect("meta")
                .permissions()
                .readonly(),
            "the bootstrapped default must be writable"
        );
        // It opens as a valid store (a read-write open would fail if read-only).
        fragcap::targets::Store::open(&default).expect("valid writable store");

        // Clear the template's read-only bit so the scratch tree can be removed.
        let tmeta = std::fs::metadata(&template).expect("meta").permissions();
        let _ = make_writable(&template, tmeta);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_bootstrap_leaves_an_existing_default_untouched() {
        // FR-004: an already-present default is never overwritten, even when a
        // template is available (an explicit path never reaches this helper).
        let dir = scratch("bootstrap-present");
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let default = dir.join("hint.db");
        std::fs::write(&default, b"sentinel").expect("seed sentinel");
        let template = dir.join("template.db");
        fragcap::targets::Store::open(&template).expect("template store");
        ensure_store(&default, Some(&template)).expect("no-op on a present default");
        assert_eq!(
            std::fs::read(&default).expect("read default"),
            b"sentinel",
            "an existing default must be left exactly as it was"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_hint_ambiguity_is_named_in_the_nonprofile_error() {
        // FR-008: when the hint row names several candidate clients and nothing
        // lower resolves, the run error names the app id and the candidate count
        // rather than the generic "no client recognized" message.
        use fragcap::profile::{ResolutionRequest, SearchPath, TargetResolver};
        use fragcap::targets::{Game, HintDatabaseProvider, LaunchEntry, Store};

        let mut store = Store::open_in_memory().expect("in-memory store");
        let mut game = Game::new(480);
        let mut a = LaunchEntry::new("client.exe").expect("non-empty");
        a.os = Some("windows".to_string());
        let mut b = LaunchEntry::new("editor.exe").expect("non-empty");
        b.os = Some("windows".to_string());
        game.launch = vec![a, b];
        store.upsert_game(&game).expect("upsert");

        let resolver = TargetResolver::new(vec![Box::new(HintDatabaseProvider::new(store))])
            .expect("one provider");
        let search = SearchPath::new();
        let bundled = fragcap::profile::BundledSet::empty();
        let request =
            ResolutionRequest::for_reference("unused", &search, &bundled).with_steam_app_id(480);
        let error = resolver
            .resolve(&request)
            .expect_err("an ambiguous hint with no lower provider is unresolved");

        let install = scratch("install");
        let cli_error = nonprofile_resolution_error(error, &install);
        let message = cli_error.message();
        assert!(
            message.contains("480") && message.contains('2'),
            "the run error names the app id and candidate count: {message}"
        );
    }

    #[test]
    fn a_synthesized_profile_is_heuristic_unverified_never_authored() {
        let profile = synthesize_profile(&identity(), None).expect("a valid synthesized profile");
        assert_eq!(profile.fidelity(), FidelityTier::HeuristicUnverified);
        assert_ne!(profile.fidelity(), FidelityTier::Authored);
    }

    #[test]
    fn a_steam_synthesized_profile_carries_the_app_id_as_a_fact() {
        let profile = synthesize_profile(&identity(), Some("306130")).expect("a valid profile");
        assert_eq!(profile.game().app_id(), Some("306130"));
        // The display name stays generic; the app id is the only asserted fact.
        assert_eq!(profile.fidelity(), FidelityTier::HeuristicUnverified);
    }

    #[test]
    fn a_synthesized_profile_is_a_single_target_stage() {
        let profile = synthesize_profile(&identity(), None).unwrap();
        assert_eq!(profile.stages().len(), 1);
    }
}
