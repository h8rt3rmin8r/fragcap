// SPDX-License-Identifier: Apache-2.0

//! Shared stored-target resolution: turn a target selector into the validated
//! one-stage capture [`Profile`] that captures it.
//!
//! Extracted from `capture` (slice S058) so both the `capture` command and the
//! Wireshark `extcap` capture path resolve a stored target through one
//! implementation, rather than extcap resolving a retired profile file. A stored
//! target carrying a Steam anchor is resolved through the install-layout cascade
//! keyed on its app id; a target carrying a stored launch executable synthesizes
//! directly from it. No process handle is opened and no process memory is read
//! (P-1). In every case a one-stage identity is validated through `Profile::parse`,
//! so an unusable identity surfaces as a profile diagnostic (exit 2).

use std::path::{Path, PathBuf};

use fragcap::profile::{
    EngineRuleProvider, MatchPredicates, ObservationProvider, Profile, ProfileProvider,
    ResolutionError, ResolutionRequest, TargetProvider, TargetResolver,
};
use fragcap::steam::SteamWalkerProvider;
use fragcap::targets::{
    entry_windows_launch_entries, is_row_index, launch_is_unresolved, observed_executable,
    resolve_id, resolve_positional, Selection, Store, TargetEntry,
};

use crate::emit::Emitter;
use crate::exit::CliError;
use crate::paths;

/// The store paths and optional path anchors a stored-target resolution needs,
/// gathered from whichever command is resolving (the `capture` command or the
/// `extcap` capture path). Decoupled from any one command's argument struct so the
/// single resolution implementation serves both callers (slice S058).
pub(crate) struct TargetInputs<'a> {
    /// The catalog store override, or `None` to use the environment or default.
    pub catalog_db: Option<&'a Path>,
    /// The local store override, or `None` to use the environment or default.
    pub local_db: Option<&'a Path>,
    /// A substring the target's full image path must contain (disambiguates a
    /// shared image name). Never set by `extcap`.
    pub path_contains: Option<&'a str>,
    /// A regular expression the target's full image path must match. Never set by
    /// `extcap`.
    pub path_regex: Option<&'a str>,
}

/// How a stored target was named: a positional selector or a durable identifier.
pub(crate) enum StoredRef<'a> {
    /// A positional selector: a handle, an exact name, or a 1-based row index.
    Selector(&'a str),
    /// A durable stable identifier, the machine-facing form automation addresses.
    Id(i64),
}

/// The result of resolving a stored target: the validated capture profile, and,
/// for a target that was resolved in observe mode, how to promote it after the
/// run (slice S059).
pub(crate) struct ResolvedTarget {
    /// The validated one- or two-stage capture profile.
    pub profile: Profile,
    /// The exact stored row that produced the profile and any managed launch.
    pub entry: TargetEntry,
    /// Present only when an unresolved target (a `no`/`unsure` authoring answer)
    /// was resolved in observe mode. `None` for a resolved client, a
    /// Steam-anchored target, or a `--process` synthesis.
    pub promotion: Option<Promotion>,
}

/// What `capture` needs to promote an unresolved target after a run that
/// observed its real socket-holding process (slice S059).
///
/// Carries the resolved entry's durable row id and the local store it was
/// resolved from, so `run` can reopen that store and rewrite the launch chain
/// without re-deriving the store-path resolution.
pub(crate) struct Promotion {
    /// The resolved entry's autoincrement row key.
    pub target_id: i64,
    /// The local store the target was resolved from.
    pub local_db: PathBuf,
}

/// The message for a selector that matched no target.
///
/// One constructor for all four emit sites (`resolve_stored` here, and `show`,
/// `remove`, and `export` in `targets.rs`), so a fifth cannot drift from the
/// other four.
///
/// The numeric case is the one that matters. `is_row_index` gates resolution
/// first and never falls through, so a bare integer is resolved as a listing row
/// number and as nothing else. Issue #181 records an operator reading a table of
/// Steam app ids, passing one to `--target`, and being told only "no target
/// matches": the resolver knew the token was numeric, knew it had therefore
/// taken the row-index path, and knew how many rows the snapshot held, and
/// reported none of it. Withholding an observation the instrument made is a P-9
/// defect, so the message names the interpretation it used, the size of the
/// space it searched, and the route to the other namespace.
///
/// A non-numeric selector keeps the short message: there is no second namespace
/// to disambiguate, and a handle or name miss means what it says.
pub(crate) fn no_match_message(store: &Store, selector: Option<&str>) -> String {
    let Some(token) = selector.filter(|t| is_row_index(t)) else {
        return "no target matches; list targets with `fragcap targets`".to_string();
    };
    // A snapshot read that fails reports an unknown size rather than replacing
    // the whole message with a store error: the caller is already on a failure
    // path, and the interpretation is the half that unblocks the operator.
    let rows = match store.listing_snapshot_len() {
        Ok(n) => format!("the listing has {n} row(s)"),
        Err(_) => "the listing size could not be read".to_string(),
    };
    format!(
        "no target matches\n  \
         `{token}` was read as a listing row number; {rows}.\n  \
         A bare number is always a row number here, never a handle, a name, or a\n  \
         platform app id. If {token} is a Steam app id, register the title first:\n      \
         fragcap targets add --steam {token}\n  \
         Then capture it by handle or row number:\n      \
         fragcap targets"
    )
}

/// Resolve a stored target against the local store into the profile that captures
/// it.
///
/// The target is named by a positional selector (handle, exact name, or 1-based row
/// index) or a durable `--id`. A Steam-anchored target is resolved through the
/// install-layout cascade keyed on its app id, recovering its client executable and
/// carrying the app id so `--launch` can reach it. A target carrying an unresolved
/// launch chain (a `no`/`unsure` authoring answer) that still names an observed
/// executable is resolved in observe mode, so a run can promote it once it observes
/// the real socket holder (slice S059). A target carrying resolved launch entries
/// synthesizes from its single Windows client, refusing an empty or ambiguous set
/// rather than guessing (P-9). A target that names neither is a usage error rather
/// than a silent empty capture (P-4).
pub(crate) fn resolve_stored(
    target: StoredRef,
    inputs: &TargetInputs,
    emitter: &mut Emitter,
) -> Result<ResolvedTarget, CliError> {
    let (catalog_db, local_db) = setup_stores(inputs.catalog_db, inputs.local_db, emitter)?;

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
            let selector = match target {
                StoredRef::Selector(t) => Some(t),
                StoredRef::Id(_) => None,
            };
            return Err(CliError::usage(no_match_message(&store, selector)));
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

    // A Steam anchor recovers the client through the cascade; an unresolved chain
    // resolves in observe mode; resolved launch entries synthesize directly; none is
    // a named usage error.
    if let Some(app_id) = steam_app_id(&entry) {
        // The install-layout cascade determines the client identity for a
        // Steam-anchored target, so an operator path anchor has nothing to attach to
        // here; refuse it rather than silently discard it (P-9).
        if inputs.path_contains.is_some() || inputs.path_regex.is_some() {
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
        // A Steam-anchored target is a resolved client, never promoted here.
        let profile =
            resolve_from_install(&resolver, &install_root, Some(&app_id), &search, &bundled)?;
        return Ok(ResolvedTarget {
            profile,
            entry: *entry,
            promotion: None,
        });
    }

    // An unresolved launch chain (a `no`/`unsure` authoring answer) that still names
    // an observed executable is captured in observe mode: synthesize a profile from
    // that executable and carry a promotion so the run can rewrite the launch chain
    // once it observes the real socket holder (slice S059). An unresolved chain that
    // names nothing to observe from falls through to the refusal below.
    if launch_is_unresolved(&entry) {
        if let Some(exe) = observed_executable(&entry) {
            // The observe-mode profile determines the client by process ancestry, so
            // an operator path anchor has nothing to attach to; refuse it rather than
            // silently discard it (P-9), as the Steam-anchored branch does.
            if inputs.path_contains.is_some() || inputs.path_regex.is_some() {
                return Err(CliError::usage(
                    "--path/--path-regex do not apply to an unresolved target captured in observe \
                     mode; the observed socket holder determines the client identity",
                ));
            }
            let profile = synthesize_observe_profile(exe)?;
            // Promote after the run only when the resolved row carries an id (it
            // always does once read back from the store); without one there is
            // nothing to write back to.
            let promotion = entry.id.map(|target_id| Promotion {
                target_id,
                local_db: local_path.clone(),
            });
            return Ok(ResolvedTarget {
                profile,
                entry: *entry,
                promotion,
            });
        }
    }

    let launches = entry_windows_launch_entries(&entry);
    if launches.len() > 1 && launches.iter().any(|launch| launch.role().is_some()) {
        let profile = synthesize_publisher_profile(&entry)?;
        return Ok(ResolvedTarget {
            profile,
            entry: *entry,
            promotion: None,
        });
    }

    // Reduce the resolved launch entries to their distinct Windows clients by the
    // same filter, file-name reduction, and case-insensitive dedup the hint provider
    // applies (P-10). Exactly one is capturable; zero or more than one is refused
    // rather than an arbitrary pick (P-9). The operator path anchors ride onto the
    // synthesized identity, so `--target ... --path ...` is honored rather than
    // discarded. A resolved client is never promoted.
    let clients = fragcap::targets::entry_windows_clients(&entry);
    match clients.as_slice() {
        [exe] => {
            synthesize_named_profile(exe, inputs.path_contains, inputs.path_regex).map(|profile| {
                ResolvedTarget {
                    profile,
                    entry: *entry,
                    promotion: None,
                }
            })
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

/// Build the shared Capture identity for one exact stored publisher chain.
fn synthesize_publisher_profile(entry: &TargetEntry) -> Result<Profile, CliError> {
    let launch = fragcap::managed_launch::prepare_managed_launch(entry)
        .map_err(|error| CliError::usage(error.to_string()))?;
    let fragcap::managed_launch::ManagedLaunch::Publisher(publisher) = launch else {
        return Err(CliError::usage(
            "stored target does not declare a complete publisher chain",
        ));
    };
    let stages: Vec<serde_json::Value> = publisher
        .stages()
        .iter()
        .map(|stage| {
            let mut predicates = serde_json::Map::new();
            predicates.insert(
                "exe".to_string(),
                serde_json::Value::String(stage.image_name().to_string_lossy().into_owned()),
            );
            predicates.insert(
                "path_regex".to_string(),
                serde_json::Value::String(exact_path_regex(stage.executable())),
            );
            if let Some(parent) = stage.parent_role() {
                predicates.insert(
                    "descends_from".to_string(),
                    serde_json::Value::String(parent.to_string()),
                );
            }
            serde_json::json!({
                "role": stage.role(),
                "lifecycle": "session",
                "terminal": stage.is_terminal(),
                "match": predicates,
            })
        })
        .collect();
    let profile = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": entry.fidelity.as_str(),
        "game": { "id": entry.handle, "name": entry.name },
        "stage": stages,
    });
    Profile::parse(&profile.to_string()).map_err(CliError::from)
}

/// Encode one canonical executable path as an exact case-insensitive regex.
///
/// Process paths on Windows are case-insensitive, while `path_regex` otherwise
/// uses the regex engine's default case-sensitive comparison. Escaping every
/// metacharacter and anchoring both ends preserves the retained executable
/// identity instead of reducing it to a basename or substring.
fn exact_path_regex(path: &Path) -> String {
    let path = process_path_spelling(path);
    let mut escaped = String::with_capacity(path.len().saturating_mul(2).saturating_add(7));
    for ch in path.chars() {
        if matches!(
            ch,
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    format!("(?i)^{escaped}$")
}

fn process_path_spelling(path: &Path) -> String {
    let canonical = path.to_string_lossy();
    // `std::fs::canonicalize` uses the Win32 verbatim prefix, while process
    // creation events report the ordinary DOS or UNC spelling. They name the
    // same path, so remove only that representation prefix before anchoring.
    if let Some(rest) = canonical.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else {
        canonical
            .strip_prefix(r"\\?\")
            .unwrap_or(&canonical)
            .to_string()
    }
}

/// Build a validated two-stage observe-mode profile from an observed executable
/// (slice S059).
///
/// An unresolved target names the process the operator pointed at but not the
/// socket holder: a `no` answer says a different, child process holds the sockets,
/// and an `unsure` answer leaves it open. The profile has a terminal `client` stage
/// matching descendants of a `launcher` stage that matches the observed executable.
/// Both shapes bind the socket holder: the child case through the terminal
/// `descends_from` stage, the case where the observed executable itself holds the
/// sockets through the launcher stage.
///
/// The terminal `client` stage is declared before the `launcher` stage on purpose.
/// Stage matching binds a process to the first declared stage whose predicates hold
/// ([`fragcap::profile`] `stage_for`), and a game that relaunches a child under the
/// same image name as the observed executable would otherwise match the `exe`
/// launcher stage first and bind as a launcher, so the terminal client stage would
/// never bind and its exit could not stop the capture. Declaring `descends_from`
/// first makes such a descendant bind as the terminal client; the root observed
/// executable still binds as the launcher, because when it is evaluated (in creation
/// order, before any descendant) nothing is yet bound to `launcher`, so its
/// `descends_from` predicate does not hold.
///
/// It is not a wildcard: `descends_from` is a non-empty predicate and carries no
/// `exe`, so the profile passes validation (an empty-predicate stage is a hard
/// error, and `ambiguous_image_match` only fires when two stages both carry an
/// `exe`). Stage order is not otherwise validated: at most one terminal stage and a
/// `descends_from` naming a declared role are both order-independent. The identity is
/// placed into JSON (serde_json handles escaping) and validated through
/// `Profile::parse`, so an unusable executable glob surfaces as a diagnostic
/// (exit 2). The fidelity is `heuristic-unverified`: the identity was synthesized,
/// not typed by an operator, so `authored` would be a lie (P-9). Promotion of the
/// stored target's own fidelity to `verified` is a separate axis the run writes back.
fn synthesize_observe_profile(exe: &str) -> Result<Profile, CliError> {
    let profile = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "heuristic-unverified",
        "game": { "id": "observe", "name": "launch-and-observe" },
        "stage": [
            {
                "role": "client",
                "lifecycle": "session",
                "terminal": true,
                "match": { "descends_from": "launcher" }
            },
            { "role": "launcher", "lifecycle": "session", "match": { "exe": exe } }
        ]
    });
    Profile::parse(&profile.to_string()).map_err(CliError::from)
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
pub(crate) fn synthesize_named_profile(
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
    catalog_db_flag: Option<&Path>,
    local_db_flag: Option<&Path>,
    emitter: &mut Emitter,
) -> Result<(Option<PathBuf>, Option<PathBuf>), CliError> {
    // Resolve both store paths without side effects first, so the same-file refusal
    // fires before either default is created. The catalog resolves the same way the
    // shared bootstrap helper does below (flag, env, else per-user default).
    let catalog_resolved =
        paths::catalog_db_path(catalog_db_flag).or_else(paths::default_catalog_db_path);
    let (mut local_db, local_default) = match paths::local_db_path(local_db_flag) {
        Some(path) => (Some(path), false),
        None => (paths::default_local_db_path(), true),
    };

    // The two stores must be different files, or accumulation would write learned
    // user data into the catalog and a future catalog replacement would discard it
    // (FR-007). Refuse before bootstrap or accumulation touches it.
    if let (Some(catalog), Some(local)) = (catalog_resolved.as_deref(), local_db.as_deref()) {
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
    // The catalog resolves and seeds through the one shared helper the `targets`
    // hero listing also calls, so the shipped catalog is referenced identically by
    // both discovery entry points and cannot drift.
    let catalog_db = match ensure_catalog_store(catalog_db_flag) {
        Ok(path) => path,
        Err(message) => {
            emitter.warn(&message);
            None
        }
    };
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

/// Resolve the catalog store path and, when it is the per-user default, seed it from
/// the template shipped beside the executable (the first-run bootstrap).
///
/// This is the single place the two discovery entry points -- the `capture`
/// resolution ([`setup_stores`]) and the `targets` hero listing -- resolve and seed
/// the catalog, so the shipped catalog is referenced identically by both and cannot
/// drift. Before this existed the seeding lived only in the capture path, so a fresh
/// install running `fragcap targets` (the documented first command) found no catalog
/// in the per-user location, skipped discovery, and listed nothing until a capture
/// happened to seed it or the operator copied the file by hand.
///
/// An operator-named path (the `--catalog-db` flag or `FRAGCAP_CATALOG_DB`) is
/// returned as given and never created; only the per-user default is bootstrapped,
/// from the template when one ships beside the exe, or empty on a bare-exe build.
/// Returns the resolved path, `Ok(None)` when no location can be determined at all,
/// or `Err(message)` when a defaulted store could not be initialized (the caller
/// turns that into a warning and drops the store, never a fatal error, FR-005).
pub(crate) fn ensure_catalog_store(
    catalog_db_flag: Option<&Path>,
) -> Result<Option<PathBuf>, String> {
    // Operator-named (flag or environment): used as given, never created.
    if let Some(path) = paths::catalog_db_path(catalog_db_flag) {
        return Ok(Some(path));
    }
    // Per-user default: seed it from the shipped template on first run.
    let Some(default) = paths::default_catalog_db_path() else {
        return Ok(None);
    };
    let template = bundled_catalog_template();
    ensure_store(&default, template.as_deref()).map_err(|e| {
        format!(
            "catalog store could not be initialized at {}: {e}",
            default.display()
        )
    })?;
    Ok(Some(default))
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

    #[test]
    fn publisher_profile_preserves_exact_order_ancestry_and_terminal_client() {
        use fragcap::core::{ProcessEvent, ProcessTree, Timestamp};
        use fragcap::profile::{bind_stages, stage_for};
        use fragcap::targets::{ClassificationSource, TargetClassification};

        let root = scratch("publisher-profile");
        std::fs::create_dir_all(&root).unwrap();
        for image in ["Publisher.exe", "Bootstrap.exe", "Game.exe"] {
            std::fs::write(root.join(image), b"fixture").unwrap();
        }
        let entry = TargetEntry {
            id: Some(1),
            stable_id: 1,
            handle: "publisher-game".into(),
            name: "Publisher Game".into(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: None,
            launch_entries: Some(serde_json::json!([
                { "executable": "Publisher.exe", "role": "launcher" },
                { "executable": "Bootstrap.exe", "role": "bootstrap" },
                { "executable": "Game.exe", "role": "client" }
            ])),
            install_root: Some(root.to_string_lossy().into_owned()),
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        };

        let profile = synthesize_publisher_profile(&entry).expect("valid publisher profile");
        let stages = profile.stages();
        assert_eq!(
            stages.iter().map(|stage| stage.role()).collect::<Vec<_>>(),
            ["launcher", "bootstrap", "client"]
        );
        assert_eq!(stages[0].predicates().descends_from(), None);
        assert_eq!(stages[1].predicates().descends_from(), Some("launcher"));
        assert_eq!(stages[2].predicates().descends_from(), Some("bootstrap"));
        assert!(!stages[0].is_terminal());
        assert!(!stages[1].is_terminal());
        assert!(stages[2].is_terminal());
        for (stage, image) in stages
            .iter()
            .zip(["Publisher.exe", "Bootstrap.exe", "Game.exe"])
        {
            let exact = stage
                .predicates()
                .path_regex()
                .expect("publisher stages retain an exact path predicate");
            let canonical = root.join(image).canonicalize().unwrap();
            let expected = process_path_spelling(&canonical);
            assert!(
                exact.regex().is_match(&expected),
                "{} does not match {}",
                exact.as_str(),
                expected
            );
            assert!(!exact
                .regex()
                .is_match(&root.join("other").join(image).to_string_lossy()));
        }

        let publisher_path =
            process_path_spelling(&root.join("Publisher.exe").canonicalize().unwrap());
        let bootstrap_path =
            process_path_spelling(&root.join("Bootstrap.exe").canonicalize().unwrap());
        let game_path = process_path_spelling(&root.join("Game.exe").canonicalize().unwrap());
        let mut tree = ProcessTree::new();
        tree.apply(ProcessEvent::started(
            100,
            0,
            &publisher_path,
            "publisher",
            Timestamp::from_nanos(1),
        ));
        tree.apply(ProcessEvent::started(
            200,
            100,
            &bootstrap_path,
            "bootstrap",
            Timestamp::from_nanos(2),
        ));
        tree.apply(ProcessEvent::started(
            300,
            200,
            &game_path,
            "game",
            Timestamp::from_nanos(3),
        ));
        tree.apply(ProcessEvent::started(
            400,
            100,
            &game_path,
            "escaped",
            Timestamp::from_nanos(4),
        ));
        bind_stages(&profile, &mut tree);
        let role_for = |pid| {
            let node = tree
                .nodes()
                .find(|node| node.pid().get() == pid)
                .expect("timeline node");
            stage_for(&profile, &tree, node.id()).map(|stage| stage.role())
        };
        assert_eq!(role_for(100), Some("launcher"));
        assert_eq!(role_for(200), Some("bootstrap"));
        assert_eq!(role_for(300), Some("client"));
        assert_eq!(
            role_for(400),
            None,
            "escaped client is not guessed into the chain"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    // Slice S059. The observe-mode profile validates and has the two-stage shape
    // that binds a child socket holder without tripping validation.
    #[test]
    fn an_observe_profile_is_a_valid_two_stage_launcher_and_client() {
        let profile =
            synthesize_observe_profile("launcher.exe").expect("the observe profile validates");
        let stages = profile.stages();
        assert_eq!(stages.len(), 2, "launcher stage plus terminal client stage");

        let launcher = stages
            .iter()
            .find(|s| s.role() == "launcher")
            .expect("a launcher stage");
        assert!(
            !launcher.is_terminal(),
            "the launcher stage is not terminal"
        );
        assert_eq!(
            launcher.predicates().exe().map(|e| e.as_str()),
            Some("launcher.exe"),
            "the launcher stage matches the observed executable"
        );

        let client = stages
            .iter()
            .find(|s| s.role() == "client")
            .expect("a client stage");
        assert!(client.is_terminal(), "the client stage is terminal");
        assert_eq!(
            client.predicates().descends_from(),
            Some("launcher"),
            "the client stage matches descendants of the launcher"
        );
        assert!(
            client.predicates().exe().is_none(),
            "the client stage carries no exe, so ambiguous_image_match cannot fire"
        );
    }

    // Slice S059, PR #159 review (Codex P1). The terminal client stage is declared
    // before the launcher stage, so a descendant whose image name equals the observed
    // executable binds as the terminal client (via descends_from) rather than as the
    // launcher. Were the launcher (exe) stage declared first, stage matching would
    // bind such a child to it and the terminal client would never bind, leaving the
    // capture without a terminal exit to stop it. The root observed executable still
    // binds as the launcher, because it descends from nothing bound to `launcher`.
    #[test]
    fn a_same_named_child_binds_the_terminal_client_not_the_launcher() {
        use fragcap::core::{ProcessEvent, ProcessTree, Timestamp};
        use fragcap::profile::{bind_stages, stage_for};

        let profile = synthesize_observe_profile("game.exe").expect("valid observe profile");

        // A tree where game.exe (the observed executable) relaunches a child also
        // named game.exe. Applied in creation order, as the session folds events.
        let mut tree = ProcessTree::new();
        tree.apply(ProcessEvent::started(
            100,
            0,
            "C:\\G\\game.exe",
            "g",
            Timestamp::from_nanos(1),
        ));
        tree.apply(ProcessEvent::started(
            200,
            100,
            "C:\\G\\game.exe",
            "g",
            Timestamp::from_nanos(2),
        ));
        bind_stages(&profile, &mut tree);

        let node_id = |pid: u32| {
            tree.nodes()
                .find(|n| n.pid().get() == pid)
                .unwrap_or_else(|| panic!("a node for pid {pid}"))
                .id()
        };

        assert_eq!(
            stage_for(&profile, &tree, node_id(100)).map(|s| s.role()),
            Some("launcher"),
            "the root observed executable binds as the launcher"
        );
        let child_stage =
            stage_for(&profile, &tree, node_id(200)).expect("the same-named child binds a stage");
        assert_eq!(
            child_stage.role(),
            "client",
            "a same-named descendant binds the terminal client, not the launcher"
        );
        assert!(
            child_stage.is_terminal(),
            "the descendant binds the terminal stage, so its exit can stop the capture"
        );
    }

    // Slice S059. The synthesized identity was not typed by an operator, so it is
    // never stamped authored (P-9). Promotion of the stored target's own fidelity
    // to verified is a separate axis written back after the run.
    #[test]
    fn an_observe_profile_is_not_authored_fidelity() {
        let profile = synthesize_observe_profile("game.exe").expect("valid");
        assert_ne!(profile.fidelity(), FidelityTier::Authored);
        assert_eq!(profile.fidelity(), FidelityTier::HeuristicUnverified);
    }

    // Slice S059. An unusable executable glob surfaces as a profile diagnostic
    // (exit 2) rather than a malformed capture.
    #[test]
    fn an_observe_profile_rejects_an_unusable_executable_glob() {
        // An empty executable is not a usable image pattern.
        assert!(synthesize_observe_profile("").is_err());
    }
}
