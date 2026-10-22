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
                "cannot establish the ordinary output recipient: {error}; same-account elevation is automatic and requires an ordinary Windows desktop session. For a genuinely different account, use --output-recipient <SID> with --bundle and authenticate from that account's ordinary session"
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

fn require_existing_bundle(bundle: &std::path::Path) -> Result<(), CliError> {
    let metadata = bundle.symlink_metadata().map_err(|error| {
        CliError::usage(format!(
            "bundle path {} is unavailable: {error}. Check the bundle path and inspect the completed bundle before retrying",
            bundle.display()
        ))
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(CliError::usage(format!(
            "bundle path {} is not an exact directory. Check the bundle path before retrying",
            bundle.display()
        )));
    }
    Ok(())
}

fn actionable_collection_preview(value: &serde_json::Value) -> bool {
    value["mode"] == "preview"
        && value["complete"] == true
        && value["eligible_sessions"]
            .as_u64()
            .is_some_and(|count| count > 0)
        && value["unresolved_sessions"] == 0
}

fn render_collection(
    value: &serde_json::Value,
    json: bool,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    if json {
        return render_access(value, true, out);
    }
    let width = crate::display::selected_stdout_width(std::io::stdout().is_terminal());
    let mut fields = vec![
        (
            "Mode:".into(),
            value["mode"].as_str().unwrap_or("unavailable").into(),
        ),
        (
            "Session root:".into(),
            value["root"].as_str().unwrap_or("unavailable").into(),
        ),
        (
            "Apply scope:".into(),
            format!(
                "delete eligible bundle contents{}{}",
                if value["include_retained"] == true {
                    ", including explicitly selected retained evidence"
                } else {
                    ""
                },
                if value["purge_empty"] == true {
                    ", and eligible exact empty containers"
                } else {
                    ""
                }
            ),
        ),
    ];
    for (label, key) in [
        ("Eligible sessions:", "eligible_sessions"),
        ("Retained sessions:", "retained_sessions"),
        ("Unresolved sessions:", "unresolved_sessions"),
        ("Empty containers:", "empty_containers"),
        ("Recoverable bundle bytes:", "recoverable_bytes"),
        ("Removed bundle bytes:", "removed_bytes"),
        ("Removed bundle files:", "removed_files"),
        ("Complete:", "complete"),
    ] {
        fields.push((
            label.into(),
            value
                .get(key)
                .map_or_else(|| "unavailable".into(), ToString::to_string),
        ));
    }
    write!(out, "{}", crate::display::render_fields(0, &fields, width))
        .map_err(|error| CliError::failure(error.to_string()))?;
    if let Some(rows) = value["results"].as_array() {
        for row in rows {
            let mut fields = vec![
                (
                    "Bundle:".into(),
                    row["bundle"].as_str().unwrap_or("unavailable").into(),
                ),
                (
                    "Status:".into(),
                    row["status"].as_str().unwrap_or("unresolved").into(),
                ),
                (
                    "Reason:".into(),
                    row["reason"]
                        .as_str()
                        .unwrap_or("see collection results")
                        .into(),
                ),
                (
                    "Recoverable bundle bytes:".into(),
                    row.get("recoverable_logical_bytes")
                        .map_or_else(|| "unavailable".into(), ToString::to_string),
                ),
            ];
            if let Some(detail) = row["detail"].as_str() {
                fields.push(("Detail:".into(), detail.into()));
            }
            write!(
                out,
                "\n{}",
                crate::display::render_fields(0, &fields, width)
            )
            .map_err(|error| CliError::failure(error.to_string()))?;
            if let Some(collection) = row.get("collection") {
                let fields = [
                    (
                        "Removed bundle bytes:".into(),
                        collection["removed_bytes"].to_string(),
                    ),
                    (
                        "Removed bundle files:".into(),
                        collection["removed_files"].to_string(),
                    ),
                    (
                        "Container preserved:".into(),
                        collection["preserved_container"].to_string(),
                    ),
                    ("Container purged:".into(), collection["purged"].to_string()),
                ];
                write!(out, "{}", crate::display::render_fields(0, &fields, width))
                    .map_err(|error| CliError::failure(error.to_string()))?;
                for path in collection["paths"].as_array().into_iter().flatten() {
                    let fields = [
                        (
                            "Path:".into(),
                            path["path"].as_str().unwrap_or("unavailable").into(),
                        ),
                        (
                            "Result:".into(),
                            path["status"].as_str().unwrap_or("unresolved").into(),
                        ),
                        (
                            "Reason:".into(),
                            path["reason"].as_str().unwrap_or("unavailable").into(),
                        ),
                    ];
                    write!(out, "{}", crate::display::render_fields(4, &fields, width))
                        .map_err(|error| CliError::failure(error.to_string()))?;
                }
            }
        }
    }
    if let Some(limits) = value["limitations"].as_array() {
        for limit in limits {
            write!(
                out,
                "{}",
                crate::display::render_fields(
                    0,
                    &[(
                        "Limitation:".into(),
                        limit.as_str().unwrap_or("unresolved inventory").into()
                    )],
                    width
                )
            )
            .map_err(|error| CliError::failure(error.to_string()))?;
        }
    }
    out.flush()
        .map_err(|error| CliError::failure(error.to_string()))
}

pub fn run(args: &BundleArgs, json: bool, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        BundleCommand::Collect {
            bundle,
            include_retained,
            purge_empty,
            authorize,
        } => {
            let root = crate::paths::deep_capture_session_dir()
                .ok_or_else(|| CliError::usage("no session root is available for collection"))?;
            let result = match authorize {
                Some(id) => crate::session_gc::apply(
                    &root,
                    bundle.as_deref(),
                    *include_retained,
                    *purge_empty,
                    id,
                ),
                None => crate::session_gc::preview(
                    &root,
                    bundle.as_deref(),
                    *include_retained,
                    *purge_empty,
                ),
            }
            .map_err(|error| {
                CliError::failure(format!("session collection unresolved: {error}"))
            })?;
            render_collection(&result, json, out)?;
            if !json && authorize.is_none() {
                if actionable_collection_preview(&result) {
                    let id = result["proposal_id"].as_str().ok_or_else(|| {
                        CliError::failure("complete collection preview has no proposal identifier")
                    })?;
                    let mut command = String::from("fragcap bundle collect");
                    if let Some(bundle) = bundle {
                        command.push_str(" --bundle ");
                        command.push_str(&crate::workflow_help::quote_powershell_argument(
                            &bundle.display().to_string(),
                        ));
                    }
                    if *include_retained {
                        command.push_str(" --include-retained");
                    }
                    if *purge_empty {
                        command.push_str(" --purge-empty");
                    }
                    command.push_str(" --authorize ");
                    command.push_str(&crate::workflow_help::quote_powershell_argument(id));
                    write!(
                        out,
                        "{}",
                        crate::display::render_fields(
                            0,
                            &[("Apply reviewed proposal:".into(), command)],
                            crate::display::selected_stdout_width(std::io::stdout().is_terminal()),
                        )
                    )
                    .map_err(|error| CliError::failure(error.to_string()))?;
                    out.flush()
                        .map_err(|error| CliError::failure(error.to_string()))?;
                } else {
                    let next = if result["unresolved_sessions"].as_u64().unwrap_or(0) > 0
                        || result["complete"] == false
                    {
                        "Check the bundle path and unresolved authority, then rerun this read-only preview"
                    } else {
                        "No eligible contents were selected; review the scope before another read-only preview"
                    };
                    write!(
                        out,
                        "{}",
                        crate::display::render_fields(
                            0,
                            &[("Next step:".into(), next.into())],
                            crate::display::selected_stdout_width(std::io::stdout().is_terminal()),
                        )
                    )
                    .map_err(|error| CliError::failure(error.to_string()))?;
                    out.flush()
                        .map_err(|error| CliError::failure(error.to_string()))?;
                }
            }
            if authorize.is_some() && result["complete"] == false {
                return Err(CliError::failure("session collection remains unresolved; review actual results and obtain a fresh preview"));
            }
            Ok(Exit::SUCCESS)
        }
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
        BundleCommand::Cleanup { bundle, yes } => {
            require_existing_bundle(bundle)?;
            fragcap::deep_capture::preflight_sensitive_cleanup(bundle).map_err(|error| {
                CliError::usage(format!(
                    "bundle path {} is not ready for sensitive cleanup: {error}. Inspect the bundle status before retrying",
                    bundle.display()
                ))
            })?;
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
            require_existing_bundle(bundle)?;
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
    use super::{actionable_collection_preview, cleanup_failure};

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

    #[test]
    fn apply_advice_requires_complete_positive_eligible_preview() {
        let complete = serde_json::json!({
            "mode":"preview",
            "complete":true,
            "eligible_sessions":1,
            "unresolved_sessions":0
        });
        assert!(actionable_collection_preview(&complete));
        for changed in [
            serde_json::json!({"mode":"preview","complete":false,"eligible_sessions":1,"unresolved_sessions":1}),
            serde_json::json!({"mode":"preview","complete":true,"eligible_sessions":0,"unresolved_sessions":0}),
            serde_json::json!({"mode":"apply","complete":true,"eligible_sessions":1,"unresolved_sessions":0}),
        ] {
            assert!(!actionable_collection_preview(&changed), "{changed}");
        }
    }
}
