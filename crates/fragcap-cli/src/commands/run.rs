// SPDX-License-Identifier: Apache-2.0

//! `run`: resolve a target, build the effective configuration, assemble the
//! pipeline and session, and capture.
//!
//! The command is the front half; the capture engine in [`crate::orchestrator`]
//! is the shared back half `tap` and `watch` also reach. Resolution and overlay
//! decide what to capture and with what options; the orchestrator arms, waits for
//! the target, captures, stops on a bound or interrupt, and reports.
//!
//! `run` has three mutually-exclusive target inputs (a clap group enforces
//! exactly one):
//!
//! - `--profile <ref>` resolves a profile through the cascade and captures with
//!   it, unchanged and byte-identical to before the cascade existed.
//! - `--install-dir <path>` and `--steam <app_id>` resolve a target from an
//!   install location with no authored profile. When the cascade answers with a
//!   non-profile target (an engine rule, the platform walker, or runtime
//!   observation), `run` synthesizes a one-stage capture identity from the
//!   resolved target's `MatchPredicates`, stamps it `heuristic-unverified`
//!   (never `authored`, because it was resolved by a heuristic rather than typed
//!   by an operator, P-9), and captures it through the same launch-agnostic
//!   engine `watch` uses. No process handle is opened and no process memory is
//!   read (P-1).

use std::path::{Path, PathBuf};

use fragcap::profile::{
    EngineRuleProvider, MatchPredicates, ObservationProvider, Profile, ProfileProvider,
    ResolutionError, ResolutionRequest, TargetProvider, TargetResolver,
};
use fragcap::steam::SteamWalkerProvider;

use crate::assemble;
use crate::attach;
use crate::cli::RunArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;
use crate::paths;

/// Run `run`.
pub fn run(args: &RunArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let search = paths::search_path(&[]);
    let bundled = paths::bundled();

    // Assemble the cascade, registering the hint-database provider only when the
    // operator supplied a present database (and this build carries the targets
    // feature). A missing database is not an error; a present-but-unopenable one
    // is (FR-013, FR-014).
    let hint_db = paths::hint_db_path(args.hint_db.as_deref());
    let resolver = build_resolver(hint_db.as_deref())?;

    // Exactly one target input is present (the clap group guarantees it). A
    // profile reference takes the unchanged profile path; an install location
    // takes the non-profile path.
    let (profile, nonprofile) = if let Some(reference) = args.profile.as_deref() {
        let request = ResolutionRequest::for_reference(reference, &search, &bundled);
        let target = resolver.resolve(&request)?;
        let profile = target.into_profile().ok_or_else(|| {
            // A profile reference is answered only by the profile provider, so a
            // non-profile target cannot arise here; this documents the invariant.
            CliError::failure("resolved a target with no profile from a profile reference")
        })?;
        (profile, false)
    } else {
        (
            resolve_nonprofile(&resolver, args, emitter, &search, &bundled)?,
            true,
        )
    };

    let config = assemble::effective_config(args, &profile)?;
    let components = assemble::components(&args.offline, &config)?;

    // The non-profile path is launch-agnostic like `watch`: report an
    // already-running attach, and warn when a resolved path anchor (an engine-rule
    // Unreal client carries one) cannot be checked against the executable-only
    // startup snapshot, so acquisition is never silently impossible (review of PR
    // #88). The `--profile` path keeps its existing behavior unchanged.
    if nonprofile {
        attach::report_attach_to_running(&profile, &components, emitter);
    }

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
        // A sink failure is an unrecoverable end for `run`, not a clean stop.
        false,
    )
}

/// Assemble the resolution cascade, registering the concrete hint-database
/// provider at precedence 2 only when this build carries the `targets` feature and
/// the operator supplied a present database file.
///
/// A missing database (no path, or a path that does not exist) leaves precedence 2
/// empty, so resolution is identical to a build without the feature (FR-012,
/// FR-013). A present-but-unopenable database (corrupt or a wrong schema version)
/// fails here, at the boundary where the operator named it, rather than as a
/// per-request surprise (FR-014). The built-in providers occupy distinct
/// precedence positions by construction, so `TargetResolver::new` cannot fail for
/// that reason; the message documents the invariant.
fn build_resolver(hint_db: Option<&Path>) -> Result<TargetResolver, CliError> {
    let mut providers: Vec<Box<dyn TargetProvider>> = vec![
        Box::new(ProfileProvider::new()),
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
        Box::new(ObservationProvider::new()),
    ];

    // The shipped tool always carries the targets database, so `fragcap::targets`
    // is always available here; the graceful "no database" degradation is the
    // runtime check below, not a compile-time feature gate. A library consumer of
    // `fragcap` that builds without the `targets` feature never reaches this code.
    if let Some(path) = hint_db {
        // `try_exists` distinguishes a genuinely absent file (Ok(false), which is
        // not an error, FR-013) from a path whose existence cannot be determined
        // (Err, for example a denying ACL on a parent). `Path::exists` collapses
        // the latter to `false` and would silently leave precedence 2 empty for a
        // database the operator explicitly named, so an inaccessible path is
        // surfaced loudly like an unopenable one (FR-014).
        match path.try_exists() {
            Ok(true) => {
                let store = fragcap::targets::Store::open(path).map_err(|e| {
                    CliError::failure(format!("cannot open hint database {}: {e}", path.display()))
                })?;
                providers.push(Box::new(fragcap::targets::HintDatabaseProvider::new(store)));
            }
            Ok(false) => {}
            Err(e) => {
                return Err(CliError::failure(format!(
                    "cannot access hint database {}: {e}",
                    path.display()
                )))
            }
        }
    }

    TargetResolver::new(providers)
        .map_err(|e| CliError::failure(format!("provider precedence conflict: {e}")))
}

/// Resolve a non-profile target from an install location and synthesize the
/// one-stage profile that captures it.
fn resolve_nonprofile(
    resolver: &TargetResolver,
    args: &RunArgs,
    emitter: &mut Emitter,
    search: &fragcap::profile::SearchPath,
    bundled: &fragcap::profile::BundledSet,
) -> Result<Profile, CliError> {
    // Resolve the install root, either given directly or looked up from a Steam
    // app id. A Steam lookup surfaces its enumeration warnings and fails loudly
    // when the title is not installed (P-4).
    let (install_root, app_id): (PathBuf, Option<&str>) = if let Some(dir) = &args.install_dir {
        (dir.clone(), None)
    } else {
        let app_id = args
            .steam
            .as_deref()
            .expect("the clap group guarantees exactly one target input");
        let lookup = fragcap::steam::install_root_for(app_id)
            .map_err(|e| CliError::failure(format!("cannot look up Steam app {app_id}: {e}")))?;
        for warning in &lookup.warnings {
            emitter.warn(warning);
        }
        let root = lookup.install_dir.ok_or_else(|| {
            CliError::failure(format!(
                "Steam app {app_id} is not installed in any library"
            ))
        })?;
        (root, Some(app_id))
    };

    // Offer the hint provider the Steam app id (when the target is a Steam title
    // and the id is numeric) while the install root stays available to the engine
    // rule and platform walker: the higher-precedence hint answer wins when the
    // database names a client, and the lower providers answer when it does not.
    let mut request = ResolutionRequest::for_install(&install_root, search, bundled);
    if let Some(id) = app_id.and_then(|s| s.parse::<u32>().ok()) {
        request = request.with_steam_app_id(id);
    }
    let target = match resolver.resolve(&request) {
        Ok(target) => target,
        Err(error) => return Err(nonprofile_resolution_error(error, &install_root)),
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
    fn build_resolver_without_a_hint_db_succeeds() {
        // FR-013: no database supplied is not an error; the cascade is assembled
        // without the hint provider, exactly as before this slice.
        assert!(build_resolver(None).is_ok());
    }

    #[test]
    fn a_missing_hint_db_file_is_not_an_error() {
        // FR-013: a path that does not exist leaves precedence 2 empty and raises
        // no error.
        let absent = scratch("absent");
        assert!(!absent.exists());
        assert!(build_resolver(Some(&absent)).is_ok());
    }

    #[test]
    fn a_present_valid_hint_db_registers_the_provider() {
        // FR-012: a present, openable database registers the provider (the
        // assembly succeeds with the extra precedence position).
        let path = scratch("valid.db");
        // Creating a store makes a valid database file at the path.
        fragcap::targets::Store::open(&path).expect("create store");
        assert!(build_resolver(Some(&path)).is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_present_unopenable_hint_db_is_an_error() {
        // FR-014: a present-but-unopenable database (here, a file that is not a
        // SQLite database) fails loudly rather than being treated as absent.
        let path = scratch("corrupt.db");
        std::fs::write(&path, b"this is not a sqlite database").expect("write garbage");
        let result = build_resolver(Some(&path));
        assert!(result.is_err(), "a corrupt database must be a loud error");
        let _ = std::fs::remove_file(&path);
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
