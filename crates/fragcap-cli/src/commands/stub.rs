// SPDX-License-Identifier: Apache-2.0

//! The registered-but-not-yet-implemented commands.
//!
//! `replay` appears in the top-level help so the tool does not appear to change
//! shape between releases (specification FR-001), but reports that it is not yet
//! implemented and exits 2. Foreshadowing the command without implementing it is
//! the cheap honesty of specification User Story 5. The user-facing message
//! carries no internal roadmap identifier. (`steam` was such a stub until it was
//! delivered, and `extcap` likewise; see [`crate::commands::steam`] and
//! [`crate::commands::extcap`].)

use crate::exit::{CliError, Exit};

/// Which stub was invoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stub {
    /// `replay`.
    Replay,
}

impl Stub {
    fn name(self) -> &'static str {
        match self {
            Stub::Replay => "replay",
        }
    }
}

/// Report the stub as not yet implemented and exit 2 (an unsupported command is
/// a usage error under the section 17.4 contract).
pub fn run(stub: Stub) -> Result<Exit, CliError> {
    Err(CliError::usage(format!(
        "`{}` is not yet implemented",
        stub.name()
    )))
}
