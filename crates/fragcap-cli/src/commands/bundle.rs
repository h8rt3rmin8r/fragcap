// SPDX-License-Identifier: Apache-2.0

use std::io::{IsTerminal, Write};
use std::path::Path;
use std::time::Duration;

use crate::cli::{BundleArgs, BundleCommand};
use crate::exit::{CliError, Exit};

pub(crate) fn resolve_output_recipient(
    sid: Option<&str>,
    bundle: &Path,
    controlled: bool,
    mut announce: impl FnMut(&serde_json::Value) -> Result<(), CliError>,
) -> Result<fragcap::deep_capture::OutputRecipient, CliError> {
    use fragcap::deep_capture::{OutputRecipient, RecipientHandoff};
    if controlled {
        if sid.is_some() {
            return Err(CliError::usage(
                "controlled targets use their explicit current-account access contract",
            ));
        }
        return OutputRecipient::current_account()
            .map_err(|error| CliError::failure(format!("output recipient unavailable: {error}")));
    }
    let default = OutputRecipient::desktop_session();
    if sid.is_none() {
        return default.map_err(|error| {
            CliError::failure(format!(
                "cannot establish the ordinary output recipient: {error}; select --output-recipient <SID> with an explicit --bundle and authenticate from that user's normal session"
            ))
        });
    }
    let sid = sid.expect("explicit recipient branch");
    if let Ok(recipient) = default {
        if recipient.sid() == sid {
            return Ok(recipient);
        }
    }
    let bundle = std::path::absolute(bundle).map_err(|error| {
        CliError::usage(format!(
            "cannot resolve exact recipient destination: {error}"
        ))
    })?;
    let handoff = RecipientHandoff::new(sid, &bundle)
        .map_err(|error| CliError::usage(format!("invalid output recipient request: {error}")))?;
    let request = handoff.request_id();
    let command = format!(
        "fragcap bundle access-authorize --request {} --bundle {}",
        crate::workflow_help::quote_powershell_argument(request),
        crate::workflow_help::quote_powershell_argument(&bundle.display().to_string())
    );
    announce(&serde_json::json!({
        "schema_version": 1,
        "recipient_sid": sid,
        "bundle": bundle,
        "request": request,
        "command": command,
        "timeout_seconds": 60,
        "status": "waiting-for-ordinary-recipient"
    }))?;
    handoff.accept(Duration::from_secs(60)).map_err(|error| {
        CliError::failure(format!(
            "ordinary output recipient authentication refused: {error}"
        ))
    })
}

fn owned_session_container(bundle: &Path) -> Option<std::path::PathBuf> {
    if std::env::var_os("FRAGCAP_SESSION_DIR").is_some() {
        return None;
    }
    let root = crate::paths::deep_capture_session_dir()?;
    let root = root.canonicalize().ok()?;
    let bundle = bundle.canonicalize().ok()?;
    (bundle.starts_with(&root) && bundle != root).then_some(root)
}

fn require_inactive_bundle(bundle: &Path) -> Result<(), CliError> {
    require_inactive_bundle_at(bundle, crate::paths::deep_capture_session_dir().as_deref())
}

fn require_inactive_bundle_at(bundle: &Path, root: Option<&Path>) -> Result<(), CliError> {
    let Some(root) = root.filter(|root| root.is_dir()) else {
        return Ok(());
    };
    let bundle = bundle.canonicalize().map_err(|error| {
        CliError::failure(format!("cannot inspect exact bundle ownership: {error}"))
    })?;
    let owners = crate::doctor::residue::registered_session_owners(root).map_err(|error| {
        CliError::failure(format!("bundle owner registry is unresolved: {error}"))
    })?;
    for owner in owners.iter().filter(|owner| owner.bundle == bundle) {
        if crate::doctor::residue::owner_is_active(owner).map_err(|error| {
            CliError::failure(format!("bundle owner lease is unresolved: {error}"))
        })? {
            return Err(CliError::failure(
                "bundle access repair refuses an active session; finish the owning session first",
            ));
        }
        if owner.lease_id.is_none() {
            return Err(CliError::failure(
                "bundle access repair cannot prove the legacy owner generation is inactive",
            ));
        }
    }
    Ok(())
}

fn render_access(
    value: &serde_json::Value,
    json: bool,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    if json {
        writeln!(out, "{value}").map_err(|error| CliError::failure(error.to_string()))?;
    } else {
        let fields = value.as_object().map_or_else(Vec::new, |object| {
            object
                .iter()
                .map(|(name, value)| {
                    (
                        format!("{name}:"),
                        value
                            .as_str()
                            .map_or_else(|| value.to_string(), str::to_owned),
                    )
                })
                .collect::<Vec<_>>()
        });
        write!(
            out,
            "{}",
            crate::display::render_fields(
                0,
                &fields,
                crate::display::selected_stdout_width(std::io::stdout().is_terminal())
            )
        )
        .map_err(|error| CliError::failure(error.to_string()))?;
    }
    out.flush()
        .map_err(|error| CliError::failure(error.to_string()))
}

fn cleanup_failure(message: impl Into<String>, json: bool) -> CliError {
    let error = CliError::failure(message.into());
    if json {
        error
    } else {
        crate::workflow_help::with_next_command(
            CliError::failure(format!(
                "{}; retain manifest.json, cleanup.jsonl, resources.jsonl, and the session-owner record",
                error.message()
            )),
            "fragcap doctor --fix",
        )
    }
}

pub fn run(args: &BundleArgs, json: bool, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        BundleCommand::AccessAuthorize { request, bundle } => {
            fragcap::deep_capture::authorize_output_recipient(
                request,
                bundle,
                Duration::from_secs(60),
            )
            .map_err(|error| {
                CliError::failure(format!("output recipient authentication failed: {error}"))
            })?;
            render_access(
                &serde_json::json!({"schema_version":1,"status":"recipient-authenticated","bundle":bundle}),
                json,
                out,
            )?;
            Ok(Exit::SUCCESS)
        }
        BundleCommand::AccessInspect {
            bundle,
            output_recipient,
        }
        | BundleCommand::AccessRepair {
            bundle,
            output_recipient,
            ..
        } => {
            let recipient =
                resolve_output_recipient(output_recipient.as_deref(), bundle, false, |request| {
                    render_access(request, json, out)
                })?;
            require_inactive_bundle(bundle)?;
            let container = owned_session_container(bundle);
            if let BundleCommand::AccessRepair { authorize, .. } = &args.command {
                require_inactive_bundle(bundle)?;
                let report = fragcap::deep_capture::repair_bundle_access(
                    bundle,
                    &recipient,
                    container.as_deref(),
                    authorize,
                )
                .map_err(|error| {
                    CliError::failure(format!("bundle access repair refused: {error}"))
                })?;
                render_access(&report.json(), json, out)?;
                if !report.verified {
                    return Err(CliError::failure("bundle permission repair remains unresolved; inspect a fresh proposal before retrying"));
                }
            } else {
                let inspection = fragcap::deep_capture::inspect_bundle_access(
                    bundle,
                    &recipient,
                    container.as_deref(),
                )
                .map_err(|error| {
                    CliError::failure(format!("bundle access inspection unresolved: {error}"))
                })?;
                render_access(&inspection.json(), json, out)?;
            }
            Ok(Exit::SUCCESS)
        }
        BundleCommand::Cleanup { bundle, yes } => {
            if !yes {
                let retry = format!(
                    "fragcap bundle cleanup {} --yes",
                    crate::workflow_help::quote_powershell_argument(&bundle.display().to_string())
                );
                let error = CliError::usage(
                    "bundle cleanup deletes sensitive evidence; review the bundle and pass --yes",
                );
                return Err(if json {
                    error
                } else {
                    crate::workflow_help::with_next_command(error, retry)
                });
            }
            let recovered =
                fragcap::deep_capture::recover_sensitive_actions(bundle).map_err(|error| {
                    cleanup_failure(format!("sensitive recovery failed: {error}"), json)
                })?;
            let results = fragcap::deep_capture::cleanup_sensitive(bundle).map_err(|error| {
                cleanup_failure(format!("sensitive cleanup failed: {error}"), json)
            })?;
            let results: Vec<_> = recovered.into_iter().chain(results).collect();
            let failed = results.iter().any(|result| result.status == "failed");
            let rows: Vec<Vec<String>> = results
                .iter()
                .map(|result| {
                    vec![
                        crate::display::human_display_value(&result.status),
                        crate::display::human_display_value(&result.path.display().to_string()),
                        crate::display::human_display_value(&result.reason),
                    ]
                })
                .collect();
            let layout = crate::display::ColumnLayout::new(0, &rows);
            for (result, row) in results.iter().zip(&rows) {
                if json {
                    writeln!(
                        out,
                        "{}\t{}\t{}",
                        result.status,
                        result.path.display(),
                        result.reason
                    )
                    .map_err(|error| CliError::failure(error.to_string()))?;
                } else {
                    writeln!(out, "{}", layout.render_wrapped_row(row, 80))
                        .map_err(|error| CliError::failure(error.to_string()))?;
                }
            }
            if failed {
                return Err(cleanup_failure(
                    "sensitive cleanup completed with one or more failed artifacts",
                    json,
                ));
            }
            Ok(Exit::SUCCESS)
        }
        BundleCommand::Export {
            bundle,
            out: destination,
        } => {
            let manifest = fragcap::deep_capture::export_share_copy(bundle, destination)
                .map_err(|error| CliError::failure(format!("bundle export failed: {error}")))?;
            writeln!(out, "{}", manifest.display())
                .map_err(|error| CliError::failure(error.to_string()))?;
            Ok(Exit::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn access_repair_checks_exact_generation_owner_before_effects() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        std::fs::create_dir(&bundle).unwrap();
        let lease = crate::doctor::residue::register_session_owner(root.path(), &bundle).unwrap();
        let error = super::require_inactive_bundle_at(&bundle, Some(root.path())).unwrap_err();
        assert!(error.to_string().contains("active session"));
        drop(lease);
        super::require_inactive_bundle_at(&bundle, Some(root.path())).unwrap();
    }
    use super::cleanup_failure;

    #[test]
    fn human_cleanup_failure_retains_recovery_authority_and_json_stays_stable() {
        let human = cleanup_failure("sensitive cleanup failed: denied", false);
        assert!(human.message().contains("manifest.json"));
        assert!(human.message().contains("cleanup.jsonl"));
        assert!(human.message().contains("resources.jsonl"));
        assert!(human.message().contains("session-owner record"));
        assert!(human
            .message()
            .contains("Next command:  fragcap doctor --fix"));

        let json = cleanup_failure("sensitive cleanup failed: denied", true);
        assert_eq!(json.message(), "sensitive cleanup failed: denied");
    }
}
