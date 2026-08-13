// SPDX-License-Identifier: Apache-2.0

//! Attach-to-running reporting, shared by the launch-agnostic capture commands.
//!
//! A launch-agnostic capture (`watch`, and `run` over an install location) can
//! attach to a target that is already running when the command starts: the
//! process watcher takes a query-only startup snapshot at arm and the shared
//! engine folds it into the session. This module resolves the capture identity
//! against that snapshot through the S027 cascade to report the honest observed
//! answer, and, crucially, to say something explicit when the snapshot cannot
//! confirm an already-running match rather than let acquisition look impossible.
//!
//! The Windows toolhelp startup snapshot carries only the executable file name,
//! never the full path (reading a running process's path is the handle the
//! no-handle P-1 choice precludes). So an identity that carries a path anchor (as
//! an engine-rule resolution of an Unreal client does) cannot be fully matched
//! against the snapshot: the anchor is kept for the process-start event that will
//! carry the full path, but an already-running match on the executable is
//! surfaced as a warning so the operator is not left watching a silent wait.

use fragcap::profile::{
    EngineRuleProvider, HintProvider, ObservationProvider, ProfileProvider, ResolutionError,
    ResolutionRequest, SearchPath, TargetOrigin, TargetResolver,
};
use fragcap::steam::SteamWalkerProvider;
use fragcap::{BundledSet, ProcessTree, Profile};

use crate::assemble::{self, ARMED_AT};
use crate::emit::Emitter;

/// Resolve the identity against the startup snapshot and report an
/// already-running attach through the S027 cascade.
///
/// The `ObservationProvider` answers at the `observed` tier over a tree built
/// from the snapshot; a hit is an already-running target. The report is the
/// honest observed answer (P-9). No answer means either the target is not yet
/// running (and will be acquired by wait-for-start), or it is running but a path
/// anchor cannot be checked against the executable-only snapshot, which is
/// surfaced as a warning rather than a silent wait.
pub fn report_attach_to_running(
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
        Box::new(SteamWalkerProvider::new()),
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
            // No fully-matching already-running process. The toolhelp startup
            // snapshot carries only the executable file name, never the full path
            // (the no-handle P-1 choice), so a path anchor cannot be checked
            // against it. If a process whose executable matches is already running
            // and only the path anchor excluded it, say so rather than let a
            // silent wait-until-timeout look like nothing is running (review of PR
            // #84 for watch; PR #88 for the non-profile run path).
            if let Some(exe) = identity.exe() {
                let has_path_anchor =
                    identity.path_contains().is_some() || identity.path_regex().is_some();
                let exe_already_running = tree
                    .nodes()
                    .any(|n| n.is_live() && exe.matches(n.image_name()));
                if has_path_anchor && exe_already_running {
                    emitter.warn(
                        "a process matching the executable is already running, but a path \
                         anchor cannot be checked against the startup snapshot, which carries \
                         only the executable name; it will be captured when it next starts",
                    );
                    return;
                }
            }
            emitter.progress("target not yet running; waiting for it to start");
        }
        Err(ResolutionError::Provider(_)) => {}
    }
}
