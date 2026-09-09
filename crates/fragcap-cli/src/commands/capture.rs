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
use serde_json::{json, Value};
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
    /// Canonical, secret-free authority for the exact resolved profile and
    /// managed launch that this preparation would execute.
    pub(crate) fn authorization_authority(&self) -> Value {
        json!({
            "managed_launch": managed_launch_authority(self.config.launch.as_ref()),
            "profile": profile_authority(&self.profile),
        })
    }

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

fn profile_authority(profile: &fragcap::Profile) -> Value {
    let game = profile.game();
    let stages = profile
        .stages()
        .iter()
        .map(|stage| {
            let predicates = stage.predicates();
            json!({
                "lifecycle": match stage.lifecycle() {
                    fragcap::profile::Lifecycle::Transient => "transient",
                    fragcap::profile::Lifecycle::Session => "session",
                    fragcap::profile::Lifecycle::Service => "service",
                },
                "match": {
                    "cmdline_contains": predicates.cmdline_contains(),
                    "descends_from": predicates.descends_from(),
                    "exe": predicates.exe().map(|value| value.as_str()),
                    "path_contains": predicates.path_contains(),
                    "path_regex": predicates.path_regex().map(|value| value.as_str()),
                },
                "role": stage.role(),
                "terminal": stage.is_terminal(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "fidelity": profile.fidelity().as_str(),
        "game": {
            "app_id": game.app_id(),
            "id": game.id().as_str(),
            "name": game.name(),
            "platform": game.platform(),
        },
        "stages": stages,
    })
}

fn direct_launch_authority(launch: &fragcap::managed_launch::DirectExecutableLaunch) -> Value {
    json!({
        "arguments": launch
            .arguments()
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        "executable": launch.executable().display().to_string(),
        "working_directory": launch.working_directory().display().to_string(),
    })
}

fn managed_launch_authority(launch: Option<&fragcap::managed_launch::ManagedLaunch>) -> Value {
    use fragcap::managed_launch::ManagedLaunch;

    match launch {
        None => Value::Null,
        Some(ManagedLaunch::Steam(request)) => json!({
            "application_id": request.app_id,
            "kind": "steam-protocol",
            "url": request.url,
        }),
        Some(ManagedLaunch::Platform(platform)) => json!({
            "application_id": platform.application_id(),
            "kind": "platform",
            "platform": platform.platform(),
            "root": direct_launch_authority(platform.root()),
        }),
        Some(ManagedLaunch::Direct(direct)) => json!({
            "kind": "direct",
            "root": direct_launch_authority(direct),
        }),
        Some(ManagedLaunch::Publisher(publisher)) => json!({
            "kind": "publisher",
            "root": direct_launch_authority(publisher.root()),
            "stages": publisher
                .stages()
                .iter()
                .map(|stage| json!({
                    "executable": stage.executable().display().to_string(),
                    "parent_role": stage.parent_role(),
                    "role": stage.role(),
                    "terminal": stage.is_terminal(),
                }))
                .collect::<Vec<_>>(),
        }),
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

/// Prepare the Deep Capture cold platform path without changing ordinary Capture.
///
/// Ordinary preparation first validates the existing Steam protocol request.
/// This function then replaces that request with one immutable exact platform
/// plan and replaces the resolved profile with its platform-rooted identity.
/// All work remains side-effect-free.
pub(crate) fn prepare_owned_platform(
    args: &CaptureArgs,
    emitter: &mut Emitter,
) -> Result<PreparedCapture, CliError> {
    use fragcap::managed_launch::{ManagedLaunch, PlatformLaunchAdapter, SteamPlatformAdapter};

    let mut prepared = prepare(args, emitter)?;
    if !matches!(prepared.config.launch, Some(ManagedLaunch::Steam(_))) {
        return Err(CliError::usage(
            "owned platform preparation requires a stored Steam managed launch",
        ));
    }
    let platform = SteamPlatformAdapter::discover()
        .and_then(|adapter| adapter.prepare(&prepared.profile))
        .map_err(|error| CliError::usage(error.to_string()))?;
    prepared.profile = target_resolve::synthesize_platform_profile(&prepared.profile, &platform)?;
    prepared.config.launch = Some(ManagedLaunch::Platform(platform));
    prepared.config.exact_stage_ownership = true;
    Ok(prepared)
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

#[cfg(test)]
mod authorization_tests {
    use super::*;

    #[test]
    fn authorization_authority_names_profile_predicates_and_exact_launch() {
        let profile =
            target_resolve::synthesize_named_profile("client.exe", Some("\\Games\\Sample"), None)
                .expect("profile");
        let profile_value = profile_authority(&profile);
        assert_eq!(profile_value["stages"][0]["match"]["exe"], "client.exe");
        assert_eq!(
            profile_value["stages"][0]["match"]["path_contains"],
            "\\Games\\Sample"
        );

        let executable = std::env::current_exe().expect("current executable");
        let working_directory = executable
            .parent()
            .expect("executable parent")
            .to_path_buf();
        let direct = fragcap::managed_launch::DirectExecutableLaunch::new(
            executable.clone(),
            working_directory.clone(),
            vec!["--probe".into()],
        )
        .expect("direct launch");
        let value = managed_launch_authority(Some(
            &fragcap::managed_launch::ManagedLaunch::Direct(direct),
        ));
        assert_eq!(value["kind"], "direct");
        assert_eq!(
            value["root"]["executable"],
            executable.canonicalize().unwrap().display().to_string()
        );
        assert_eq!(
            value["root"]["working_directory"],
            working_directory
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
        );
        assert_eq!(value["root"]["arguments"], json!(["--probe"]));
    }
}
