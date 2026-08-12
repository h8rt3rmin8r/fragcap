// SPDX-License-Identifier: Apache-2.0

//! `tap`: capture a named running process without an authored profile.
//!
//! A single-stage profile is synthesized for the named process and constructed
//! through the same `Profile::parse` validation path an authored profile uses,
//! so there is no unvalidated construction (specification FR-012). It then hands
//! that profile to the shared capture engine, so `tap` gets the same completion
//! summary and exit contract as `run` for free.

use fragcap::Profile;

use crate::assemble;
use crate::cli::TapArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;

/// Run `tap`.
pub fn run(args: &TapArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let profile = synthesize_profile(&args.process)?;
    let config = assemble::effective_config_for_tap(args, &profile);
    let components = assemble::components(&args.offline, &config)?;

    orchestrator::install_interrupt_handler();
    // `tap` scopes to its single synthesized stage, so it imposes no role
    // restriction of its own: the whole (one-stage) profile is in scope.
    orchestrator::capture(
        profile,
        &config,
        components,
        emitter,
        &orchestrator::INTERRUPT,
        args.offline.fire_interrupt,
        None,
        // A sink failure is an unrecoverable end for `tap`, not a clean stop.
        false,
    )
}

/// Build a validated one-stage profile for the named process.
///
/// The process image name is placed into a JSON profile (serde_json handles the
/// escaping) and the whole document is validated through `Profile::parse`, the
/// same path an authored profile takes. A name that cannot form a valid profile
/// (empty, or one whose glob does not compile) surfaces as the profile's own
/// diagnostics, exit 2.
fn synthesize_profile(process: &str) -> Result<Profile, CliError> {
    let profile = serde_json::json!({
        "schema": 1,
        "kind": "profile",
        "fidelity": "authored",
        "game": { "id": "tap", "name": "ad hoc tap" },
        "stage": [
            { "role": "target", "lifecycle": "session", "terminal": true, "match": { "exe": process } }
        ]
    });
    Profile::parse(&profile.to_string()).map_err(CliError::from)
}
