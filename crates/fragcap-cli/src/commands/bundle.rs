// SPDX-License-Identifier: Apache-2.0

use std::io::Write;

use crate::cli::{BundleArgs, BundleCommand};
use crate::exit::{CliError, Exit};

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
            for result in results {
                writeln!(
                    out,
                    "{}\t{}\t{}",
                    result.status,
                    result.path.display(),
                    result.reason
                )
                .map_err(|error| CliError::failure(error.to_string()))?;
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
