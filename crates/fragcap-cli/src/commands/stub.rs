// SPDX-License-Identifier: Apache-2.0

//! The registered-but-not-yet-implemented commands.
//!
//! `replay` appears in the top-level help so the tool does not appear to change
//! shape between releases (specification FR-001), but reports that it is not yet
//! implemented, names the slice that will deliver it, and exits 2. Naming the
//! slice is the cheap honesty of specification User Story 5: the roadmap is
//! visible without being implemented. (`steam` was such a stub until slice S17
//! delivered it, and `extcap` until slice S18; see [`crate::commands::steam`]
//! and [`crate::commands::extcap`].)

use crate::exit::{CliError, Exit};

/// Which stub was invoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stub {
    /// `replay`, delivered by slice S15.
    Replay,
}

impl Stub {
    fn name(self) -> &'static str {
        match self {
            Stub::Replay => "replay",
        }
    }

    fn slice(self) -> &'static str {
        match self {
            Stub::Replay => "S15",
        }
    }
}

/// Report the stub as not yet implemented and exit 2.
pub fn run(stub: Stub) -> Result<Exit, CliError> {
    Err(CliError::usage(format!(
        "`{}` is not yet implemented (slice {})",
        stub.name(),
        stub.slice()
    )))
}
