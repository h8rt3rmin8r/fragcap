// SPDX-License-Identifier: Apache-2.0

//! `run`: resolve a profile, build the effective configuration, assemble the
//! pipeline and session, and capture.
//!
//! The command is the front half; the capture engine in [`crate::orchestrator`]
//! is the shared back half `tap` also reaches. Resolution and overlay decide
//! what to capture and with what options; the orchestrator arms, waits for the
//! target, captures, stops on a bound or interrupt, and reports.

use fragcap::profile::{
    EngineRuleProvider, HintProvider, ObservationProvider, ProfileProvider, ResolutionRequest,
    TargetResolver,
};
use fragcap::steam::SteamWalkerProvider;

use crate::assemble;
use crate::cli::RunArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;
use crate::paths;

/// Run `run`.
///
/// Resolution now flows through the target resolution cascade (section 15.7): the
/// resolver consults its providers in precedence order and returns a fidelity
/// stamped target. For a profile reference the profile provider answers, and the
/// backing profile is handed to the capture path exactly as before, so output is
/// unchanged. The launch-agnostic observation path (a target with no profile) is
/// wired but not yet driven from the command line; that is a later slice.
pub fn run(args: &RunArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let search = paths::search_path(&[]);
    let bundled = paths::bundled();

    // The built-in providers occupy distinct precedence positions by
    // construction, so this cannot fail; the expect documents that invariant.
    let resolver = TargetResolver::new(vec![
        Box::new(ProfileProvider::new()),
        Box::new(HintProvider::new()),
        Box::new(EngineRuleProvider::new()),
        Box::new(SteamWalkerProvider::new()),
        Box::new(ObservationProvider::new()),
    ])
    .expect("the built-in providers have distinct precedence positions");
    let request = ResolutionRequest::for_reference(&args.profile, &search, &bundled);
    let target = resolver.resolve(&request)?;
    let profile = target.into_profile().ok_or_else(|| {
        // The command-line request carries only a profile reference, so only the
        // profile provider can answer; a non-profile target cannot arise here.
        CliError::failure("resolved a target with no profile, which run cannot capture yet")
    })?;

    let config = assemble::effective_config(args, &profile)?;
    let components = assemble::components(&args.offline, &config)?;

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
