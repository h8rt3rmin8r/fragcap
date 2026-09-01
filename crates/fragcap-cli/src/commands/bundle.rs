// SPDX-License-Identifier: Apache-2.0

use std::io::Write;

use crate::cli::{BundleArgs, BundleCommand};
use crate::exit::{CliError, Exit};

pub fn run(args: &BundleArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        BundleCommand::Cleanup { bundle, yes } => {
            if !yes {
                return Err(CliError::usage(
                    "bundle cleanup deletes sensitive evidence; review the bundle and pass --yes",
                ));
            }
            let recovered =
                fragcap::deep_capture::recover_sensitive_actions(bundle).map_err(|error| {
                    CliError::failure(format!("sensitive recovery failed: {error}"))
                })?;
            let results = fragcap::deep_capture::cleanup_sensitive(bundle)
                .map_err(|error| CliError::failure(format!("sensitive cleanup failed: {error}")))?;
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
                return Err(CliError::failure(
                    "sensitive cleanup completed with one or more failed artifacts",
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
