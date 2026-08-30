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

use fragcap::profile::FidelityTier;
use fragcap::targets::{resolved_client_launch, Store};
use fragcap::{CaptureScope, FlowRegistry};
use std::sync::Arc;

use crate::assemble;
use crate::attach;
use crate::cli::CaptureArgs;
use crate::commands::target_resolve::{self, Promotion, StoredRef, TargetInputs};
use crate::emit::Emitter;
use crate::exit::{CliError, Exit};
use crate::orchestrator;

/// Run `capture`.
pub fn run(args: &CaptureArgs, emitter: &mut Emitter) -> Result<Exit, CliError> {
    run_inner(args, emitter, None)
}

/// A stored target and effective configuration validated before capture resources
/// are opened. Deep Capture prepares this value before starting its proxy or
/// changing trust, so an unsupported managed launch is a side-effect-free refusal.
pub(crate) struct PreparedCapture {
    profile: fragcap::Profile,
    promotion: Option<Promotion>,
    config: assemble::EffectiveConfig,
}

impl PreparedCapture {
    /// Add child-only environment values to a retained direct launch.
    pub(crate) fn with_launch_environment<I, K, V>(&mut self, entries: I) -> Result<(), CliError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<std::ffi::OsString>,
        V: Into<std::ffi::OsString>,
    {
        let launch = self.config.launch.take().ok_or_else(|| {
            CliError::usage("the prepared Capture configuration has no managed launch")
        })?;
        self.config.launch = Some(
            launch
                .with_environment(entries)
                .map_err(|error| CliError::usage(error.to_string()))?,
        );
        Ok(())
    }
}

/// Resolve the target and validate every effective Capture option, including the
/// managed launch request, without opening capture resources or launching anything.
pub(crate) fn prepare(
    args: &CaptureArgs,
    emitter: &mut Emitter,
) -> Result<PreparedCapture, CliError> {
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
    // A stored target may carry a promotion: an unresolved target (a `no`/`unsure`
    // authoring answer) captured in observe mode is promoted after the run once it
    // observes the real socket holder (slice S059). A `--process` synthesis and a
    // resolved target carry none.
    let (profile, promotion, stored_entry) = match (selector, args.id, &args.process) {
        (Some(selector), None, None) => {
            let resolved =
                target_resolve::resolve_stored(StoredRef::Selector(selector), &inputs, emitter)?;
            (resolved.profile, resolved.promotion, Some(resolved.entry))
        }
        (None, Some(id), None) => {
            let resolved = target_resolve::resolve_stored(StoredRef::Id(id), &inputs, emitter)?;
            (resolved.profile, resolved.promotion, Some(resolved.entry))
        }
        (None, None, Some(process)) => {
            let profile = target_resolve::synthesize_named_profile(
                process,
                args.path.as_deref(),
                args.path_regex.as_deref(),
            )?;
            (profile, None, None)
        }
        // The clap group guarantees exactly one target input; this documents it.
        _ => {
            return Err(CliError::usage(
                "exactly one of a target selector, --id, or --process is required",
            ))
        }
    };

    let config = assemble::effective_config_with_target(args, &profile, stored_entry.as_ref())?;
    Ok(PreparedCapture {
        profile,
        promotion,
        config,
    })
}

/// Run a capture that Deep Capture prepared before starting mutable session
/// resources. Preparation is consumed so the validated launch request is the one
/// the orchestrator executes.
pub(crate) fn run_prepared_with_flow_registry(
    args: &CaptureArgs,
    emitter: &mut Emitter,
    prepared: PreparedCapture,
    flow_registry: Arc<FlowRegistry>,
) -> Result<orchestrator::CaptureOutcome, CliError> {
    run_prepared_outcome(args, emitter, prepared, Some(flow_registry))
}

fn run_inner(
    args: &CaptureArgs,
    emitter: &mut Emitter,
    flow_registry: Option<Arc<FlowRegistry>>,
) -> Result<Exit, CliError> {
    let prepared = prepare(args, emitter)?;
    run_prepared(args, emitter, prepared, flow_registry)
}

fn run_prepared(
    args: &CaptureArgs,
    emitter: &mut Emitter,
    prepared: PreparedCapture,
    flow_registry: Option<Arc<FlowRegistry>>,
) -> Result<Exit, CliError> {
    run_prepared_outcome(args, emitter, prepared, flow_registry).map(|outcome| outcome.exit)
}

fn run_prepared_outcome(
    args: &CaptureArgs,
    emitter: &mut Emitter,
    prepared: PreparedCapture,
    flow_registry: Option<Arc<FlowRegistry>>,
) -> Result<orchestrator::CaptureOutcome, CliError> {
    let PreparedCapture {
        profile,
        promotion,
        mut config,
    } = prepared;
    // An observe-mode run cannot scope its output to a target it has not yet
    // identified. That is the whole point of the run: slice S059 promotes an
    // unresolved target to the socket holder this capture observes, and the
    // observation is `holder_tally`, which counts only packets the write gate
    // admitted. Scoping to the target would therefore starve the mechanism that
    // decides what the target is, and the run would write nothing and promote
    // nothing (issue #184's gate, meeting S059's promotion).
    //
    // So the scope widens, and the run says so. Overriding silently would be the
    // P-9 defect this slice exists to remove; an operator who asked for a scoped
    // file and got an unscoped one has to be told, and told why.
    if promotion.is_some() && config.scope != CaptureScope::All {
        emitter.warn(concat!(
            "this target's socket holder is not known yet, so this run captures ",
            "everything while it observes one; the scope you asked for applies ",
            "once the target is promoted",
        ));
        config.scope = CaptureScope::All;
    }
    let mut components = assemble::components(&args.offline, &config)?;
    components.flow_registry = flow_registry;

    // Capture is launch-agnostic: report an already-running attach, and warn when a
    // resolved path anchor cannot be checked against the executable-only startup
    // snapshot, so acquisition is never silently impossible (review of PR #88).
    attach::report_attach_to_running(&profile, &components, emitter);

    orchestrator::install_interrupt_handler();
    let allowed_roles = config.roles.clone();
    let outcome = orchestrator::capture(
        profile,
        &config,
        components,
        emitter,
        &orchestrator::INTERRUPT,
        args.offline.fire_interrupt,
        allowed_roles,
        // A sink failure is an unrecoverable end for `capture`, not a clean stop.
        false,
    )?;

    // Capture-time promotion (slice S059): if this was an unresolved target and the
    // run observed a dominant socket holder, rewrite the stored launch chain to that
    // client and raise the target's fidelity. Observing nothing leaves it unchanged,
    // because promoting on no observation would fabricate a holder (P-9). Promotion
    // is a post-capture side effect on a run that already succeeded, so a write
    // failure is surfaced as a warning but never changes the capture's exit.
    if let Some(promotion) = promotion {
        promote_if_observed(&promotion, outcome.observed_holder.as_deref(), emitter);
    }

    Ok(outcome)
}

/// Promote an unresolved target after a run that observed its socket holder.
///
/// Reopens the local store the target was resolved from and rewrites its launch
/// chain to the observed client at `verified` fidelity. A run that observed nothing
/// (`observed_holder` is `None`) writes nothing (P-9). A promotion write failure is
/// surfaced as a warning rather than silently swallowed, but it does not change the
/// run's exit: the capture itself already succeeded and its file is written, so a
/// failure to update the stored target is not a capture failure.
fn promote_if_observed(
    promotion: &Promotion,
    observed_holder: Option<&str>,
    emitter: &mut Emitter,
) {
    let Some(image) = observed_holder else {
        emitter.progress(
            "observed no socket-holding process; the target is left unresolved for a later run",
        );
        return;
    };
    let mut store = match Store::open(&promotion.local_db) {
        Ok(store) => store,
        Err(e) => {
            emitter.warn(&format!(
                "captured, but could not open the local store to promote the target: {e}"
            ));
            return;
        }
    };
    match store.promote_target_launch(
        promotion.target_id,
        &resolved_client_launch(image),
        FidelityTier::Verified,
    ) {
        Ok(true) => emitter.progress(&format!(
            "promoted the target to its observed socket holder {image} (verified)"
        )),
        // The row was resolved for this run, so a missing row here is unexpected;
        // say so rather than silently pass.
        Ok(false) => emitter.warn(
            "captured, but the target row was not found to promote (it may have been removed)",
        ),
        Err(e) => emitter.warn(&format!("captured, but could not promote the target: {e}")),
    }
}
