// SPDX-License-Identifier: Apache-2.0

//! Shared framing and flag precedence for command-owned results.

use std::io::Write;

use serde_json::Value;

use crate::exit::CliError;

#[derive(Clone, Copy, Default)]
pub(crate) struct ResultMode {
    pub json: bool,
    pub silent: bool,
}

impl ResultMode {
    pub const fn human_result(self) -> bool {
        !self.json && !self.silent
    }
}

pub(crate) fn write_json(out: &mut dyn Write, value: &Value) -> Result<(), CliError> {
    let mut line = serde_json::to_vec(value)
        .map_err(|error| CliError::failure(format!("cannot serialize CLI result: {error}")))?;
    line.push(b'\n');
    out.write_all(&line)
        .map_err(|error| CliError::failure(format!("cannot write CLI result: {error}")))
}
