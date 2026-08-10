// SPDX-License-Identifier: Apache-2.0

//! The registered-but-not-yet-implemented commands.
//!
//! `replay`, `steam`, and `extcap` appear in the top-level help so the tool does
//! not appear to change shape between releases (specification FR-001), but each
//! reports that it is not yet implemented, names the slice that will deliver it,
//! and exits 2. Naming the slice is the cheap honesty of specification User
//! Story 5: the roadmap is visible without being implemented.

use crate::exit::{CliError, Exit};

/// Which stub was invoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stub {
    /// `replay`, delivered by slice S15.
    Replay,
    /// `steam`, delivered by slice S17.
    Steam,
    /// `extcap`, delivered by slice S18.
    Extcap,
}

impl Stub {
    fn name(self) -> &'static str {
        match self {
            Stub::Replay => "replay",
            Stub::Steam => "steam",
            Stub::Extcap => "extcap",
        }
    }

    fn slice(self) -> &'static str {
        match self {
            Stub::Replay => "S15",
            Stub::Steam => "S17",
            Stub::Extcap => "S18",
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
