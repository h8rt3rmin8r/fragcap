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
use serde_json::json;

use crate::cli::{SchemaArgs, SchemaCommand};
use crate::commands::result::{write_json, ResultMode};
use crate::exit::{CliError, Exit};

/// Run a `schema` subcommand, writing output to `out`.
pub(crate) fn run_mode(
    args: &SchemaArgs,
    out: &mut dyn Write,
    mode: ResultMode,
) -> Result<Exit, CliError> {
    match &args.command {
        SchemaCommand::Print => {
            // Emit the embedded schema byte for byte, so a caller (or the drift
            // check) can compare it to the published asset.
            out.write_all(schema_document().as_bytes())
                .map_err(|e| CliError::failure(format!("cannot write schema: {e}")))?;
            Ok(Exit::SUCCESS)
        }
        SchemaCommand::Validate { file } => {
            let text = std::fs::read_to_string(file)
                .map_err(|e| CliError::failure(format!("cannot read {}: {e}", file.display())))?;
            match validate_json(&text) {
                Validation::Malformed(msg) if mode.json => {
                    let position = serde_json::from_str::<serde_json::Value>(&text).err();
                    write_json(
                        out,
                        &json!({
                            "event": "schema.diagnostic",
                            "code": "invalid-json",
                            "path": "",
                            "line": position.as_ref().map(serde_json::Error::line),
                            "column": position.as_ref().map(serde_json::Error::column),
                            "message": msg,
                        }),
                    )?;
                    write_json(
                        out,
                        &json!({"event":"schema.summary","valid":false,"diagnostics":1}),
                    )?;
                    Ok(Exit::USAGE)
                }
                Validation::Malformed(msg) => Err(CliError::usage(format!(
                    "{}: not valid JSON: {msg}",
                    file.display()
                ))),
                Validation::Checked(d) if d.is_empty() => {
                    if mode.json {
                        write_json(
                            out,
                            &json!({"event":"schema.summary","valid":true,"diagnostics":0}),
                        )?;
                    } else if mode.human_result() {
                        writeln!(out, "{} is valid", file.display()).map_err(|e| {
                            CliError::failure(format!("cannot write schema result: {e}"))
                        })?;
                    }
                    Ok(Exit::SUCCESS)
                }
                Validation::Checked(d) if mode.json => {
                    for defect in d.iter() {
                        write_json(
                            out,
                            &json!({
                                "event": "schema.diagnostic",
                                "code": defect.code.as_str(),
                                "path": defect.pointer,
                                "line": null,
                                "column": null,
                                "message": defect.message,
                            }),
                        )?;
                    }
                    write_json(
                        out,
                        &json!({"event":"schema.summary","valid":false,"diagnostics":d.len()}),
                    )?;
                    Ok(Exit::USAGE)
                }
                Validation::Checked(d) => Err(CliError::usage(d.to_string())),
            }
        }
    }
}
