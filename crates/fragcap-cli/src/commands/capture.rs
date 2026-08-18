// SPDX-License-Identifier: Apache-2.0

//! `capture`: identify a target, build the effective configuration, assemble the
//! pipeline and session, and capture. The single capture verb (section 17.2),
//! superseding the retired `run`, `tap`, and `watch`.
//!
//! The command is the front half; the capture engine in [`crate::orchestrator`]
//! is the shared back half. Identification and overlay decide what to capture and
//! with what options; the orchestrator arms, waits for the target, captures, stops
//! on a bound or interrupt, and reports. Stored-target resolution (a selector to a
//! validated one-stage [`fragcap::profile::Profile`]) lives in
//! [`crate::commands::target_resolve`], shared with the `extcap` capture path
//! (slice S058).
//!
//! `capture` has two mutually-exclusive, required target inputs (a clap group
//! enforces exactly one):
//!
//! - a positional selector or `--target <selector>` resolves a stored target from
//!   the user store (local.db) by an S051 selector; `--id` selects one by its
//!   durable identifier. A target carrying a Steam anchor is resolved through the
//!   install-layout cascade keyed on its app id, so its client executable and (for
//!   `--launch`) its app id are recovered; a target carrying a stored launch
//!   executable synthesizes directly from it. No process handle is opened and no
//!   process memory is read (P-1).
//! - `--process <image>` names a raw process image directly, with optional
//!   `--path`/`--path-regex` anchors to disambiguate two processes sharing the
//!   name (the capability the retired `watch` carried).

use crate::assemble;
use crate::attach;
use crate::cli::CaptureArgs;
use crate::commands::target_resolve::{self, StoredRef, TargetInputs};
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;

/// Run `capture`.
pub fn run(args: &CaptureArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    // Exactly one target input is present (the clap group guarantees it). A stored
    // target resolves against the local store (and, when Steam-anchored, through the
    // install-layout cascade); a raw process image synthesizes an identity directly.
    // The positional selector and `--target` are the same input by two spellings and
    // are mutually exclusive in the group, so at most one is set; prefer the
    // positional when present.
    let selector = args.selector.as_deref().or(args.target.as_deref());
    let inputs = TargetInputs {
        catalog_db: args.catalog_db.as_deref(),
        local_db: args.local_db.as_deref(),
        path_contains: args.path.as_deref(),
        path_regex: args.path_regex.as_deref(),
    };
    let profile = match (selector, args.id, &args.process) {
        (Some(selector), None, None) => {
            target_resolve::resolve_stored(StoredRef::Selector(selector), &inputs, emitter)?
        }
        (None, Some(id), None) => {
            target_resolve::resolve_stored(StoredRef::Id(id), &inputs, emitter)?
        }
        (None, None, Some(process)) => target_resolve::synthesize_named_profile(
            process,
            args.path.as_deref(),
            args.path_regex.as_deref(),
        )?,
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
