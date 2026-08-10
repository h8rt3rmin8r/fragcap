// SPDX-License-Identifier: Apache-2.0

//! `run`: resolve a profile, build the effective configuration, assemble the
//! pipeline and session, and capture.
//!
//! The command is the front half; the capture engine in [`crate::orchestrator`]
//! is the shared back half `tap` also reaches. Resolution and overlay decide
//! what to capture and with what options; the orchestrator arms, waits for the
//! target, captures, stops on a bound or interrupt, and reports.

use fragcap::profile::resolve;

use crate::assemble;
use crate::cli::RunArgs;
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;
use crate::paths;

/// Run `run`.
pub fn run(args: &RunArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    let search = paths::search_path(&[]);
    let bundled = paths::bundled();
    let resolved = resolve(&args.profile, &search, &bundled)?;

    let config = assemble::effective_config(args, &resolved.profile)?;
    let components = assemble::components(&args.offline, &config)?;

    orchestrator::install_interrupt_handler();
    orchestrator::capture(
        resolved.profile,
        &config,
        components,
        emitter,
        &orchestrator::INTERRUPT,
        args.offline.fire_interrupt,
    )
}
