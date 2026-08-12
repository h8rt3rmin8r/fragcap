// SPDX-License-Identifier: Apache-2.0

//! `watch`: capture a target by identity, launch-agnostic (specification section
//! 15.7).
//!
//! Watch mode is the default launch-agnostic capture path. It synthesizes a
//! one-stage identity profile from an executable name and an optional path
//! anchor, validated through `Profile::parse` exactly as `tap` and an authored
//! profile are, and hands it to the shared capture engine. The operator launches
//! the game however their setup demands (a mod manager, a desktop shortcut,
//! another storefront); fragcap catches the process that matches.
//!
//! It attaches to a target already running when the command starts, not only one
//! that starts later: the process watcher takes a query-only startup snapshot at
//! arm, the shared engine folds it into the session (attach-to-running), and this
//! command additionally resolves the identity against that snapshot through the
//! S027 target resolution cascade to report the honest observed answer that names
//! the already-running process. The session is the single acquisition authority;
//! the resolver names the answer.

use fragcap::profile::{
    EngineRuleProvider, HintProvider, ObservationProvider, PlatformWalkerProvider, ProfileProvider,
    ResolutionError, ResolutionRequest, SearchPath, TargetOrigin, TargetResolver,
};
use fragcap::{BundledSet, ProcessTree, Profile};

use crate::assemble::{self, ARMED_AT};
use crate::cli::WatchArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;

/// Run `watch`.
pub fn run(args: &WatchArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let profile = synthesize_profile(args)?;
    let config = assemble::effective_config_for_watch(args, &profile);
    let components = assemble::components(&args.offline, &config)?;

    // Attach-to-running: resolve the identity against the startup snapshot through
    // the cascade. A hit means the target is already running; report the observed
    // answer that names it. The session's own snapshot fold (in the shared engine)
    // performs the acquisition, so this reports rather than acquires.
    report_attach_to_running(&profile, &components, emitter);

    orchestrator::install_interrupt_handler();
    // `watch` scopes to its single synthesized stage, so it imposes no role
    // restriction of its own.
    orchestrator::capture(
        profile,
        &config,
        components,
        emitter,
        &orchestrator::INTERRUPT,
        args.offline.fire_interrupt,
        None,
        // A sink failure is an unrecoverable end for `watch`, not a clean stop.
        false,
    )
}

/// Resolve the identity against the startup snapshot and report an
/// already-running attach through the S027 cascade.
///
/// The `ObservationProvider` answers at the `observed` tier over a tree built
/// from the snapshot; a hit is an already-running target. The report is the
/// honest observed answer (P-9). No answer means the target is not yet running
/// and will be acquired by wait-for-start.
fn report_attach_to_running(
    profile: &Profile,
    components: &assemble::CaptureComponents,
    emitter: &mut Emitter,
) {
    if components.startup_snapshot.is_empty() {
        return;
    }
    let mut tree = ProcessTree::new();
    tree.apply_snapshot_at(
        &components.startup_snapshot,
        components.snapshot_at.unwrap_or(ARMED_AT),
    );
    let identity = profile.stages()[0].predicates();

    let resolver = TargetResolver::new(vec![
        Box::new(ProfileProvider::new()),
        Box::new(HintProvider::new()),
        Box::new(EngineRuleProvider::new()),
        Box::new(PlatformWalkerProvider::new()),
        Box::new(ObservationProvider::new()),
    ])
    .expect("the built-in providers have distinct precedence positions");

    let search = SearchPath::new();
    let bundled = BundledSet::empty();
    let request = ResolutionRequest::for_observation(identity, &tree, &search, &bundled);
    match resolver.resolve(&request) {
        Ok(target) => {
            if let TargetOrigin::Observed(o) = target.origin() {
                emitter.progress(&format!(
                    "attached to already-running pid {} {} ({})",
                    o.pid(),
                    o.image_name(),
                    target.fidelity().as_str()
                ));
            }
        }
        Err(ResolutionError::Unresolved(_)) => {
            emitter.progress("target not yet running; waiting for it to start");
        }
        Err(ResolutionError::Provider(_)) => {}
    }
}

/// Build a validated one-stage identity profile from the executable and the
/// optional path anchor.
///
/// The identity is placed into a JSON profile (serde_json handles escaping) and
/// validated through `Profile::parse`, the same path an authored profile takes,
/// so an empty match or a glob or path regex that does not compile surfaces as
/// the profile's own diagnostics (exit 2). The fidelity is `authored`: the
/// operator typed the identity, exactly as `tap`'s is; `observed` is refused on a
/// profile because it is a runtime result, not an author's claim (S027).
fn synthesize_profile(args: &WatchArgs) -> Result<Profile, CliError> {
    let mut predicates = serde_json::Map::new();
    predicates.insert(
        "exe".to_string(),
        serde_json::Value::String(args.exe.clone()),
    );
    if let Some(path) = &args.path {
        predicates.insert(
            "path_contains".to_string(),
            serde_json::Value::String(path.clone()),
        );
    }
    if let Some(re) = &args.path_regex {
        predicates.insert(
            "path_regex".to_string(),
            serde_json::Value::String(re.clone()),
        );
    }
    let profile = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "authored",
        "game": { "id": "watch", "name": "ad hoc watch" },
        "stage": [
            { "role": "target", "lifecycle": "session", "terminal": true, "match": predicates }
        ]
    });
    Profile::parse(&profile.to_string()).map_err(CliError::from)
}
