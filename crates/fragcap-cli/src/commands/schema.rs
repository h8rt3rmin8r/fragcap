// SPDX-License-Identifier: Apache-2.0

//! `schema`: validate a JSON target file against the master schema, or print
//! the embedded schema.
//!
//! Validation reports every structural violation in one pass. An invalid file
//! is a configuration error (exit 2), matching `profile validate`; a file that
//! cannot be read is an expected failure (exit 1). A JSON syntax error is a
//! configuration error too, but its message says the file is not valid JSON so
//! the author fixes the syntax before worrying about structure.

use std::io::Write;

use fragcap::profile::{schema_document, validate_json, Validation};

use crate::cli::{SchemaArgs, SchemaCommand};
use crate::exit::{CliError, Exit};

/// Run a `schema` subcommand, writing output to `out`.
pub fn run(args: &SchemaArgs, out: &mut dyn Write) -> Result<Exit, CliError> {
    match &args.command {
        SchemaCommand::Print => {
            // Emit the embedded schema byte for byte, so a caller (or the drift
            // check) can compare it to the published asset.
            let _ = write!(out, "{}", schema_document());
            Ok(Exit::SUCCESS)
        }
        SchemaCommand::Validate { file } => {
            let text = std::fs::read_to_string(file)
                .map_err(|e| CliError::failure(format!("cannot read {}: {e}", file.display())))?;
            match validate_json(&text) {
                Validation::Malformed(msg) => Err(CliError::usage(format!(
                    "{}: not valid JSON: {msg}",
                    file.display()
                ))),
                Validation::Checked(d) if d.is_empty() => {
                    let _ = writeln!(out, "{} is valid", file.display());
                    Ok(Exit::SUCCESS)
                }
                Validation::Checked(d) => Err(CliError::usage(d.to_string())),
            }
        }
    }
}
