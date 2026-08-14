// SPDX-License-Identifier: Apache-2.0

//! The process exit contract and the error type the commands map to it.
//!
//! Specification section 17 fixes a 0/1/2 contract: 0 for success (including an
//! operator interrupt during capture), 1 for an expected failure, and 2 for a
//! usage or configuration error. Every command returns either an exit code it
//! chose (a completed run is 0, a blocked `doctor` is 1) or a [`CliError`], and
//! the library maps the error to its class at one site. The `From` impls here
//! are that mapping, so a command body reads `resolve(...)?` and the exit class
//! of a resolution failure is decided in one place rather than at every call.

use std::fmt;

use fragcap::core::{ConfigError, PipelineError};
use fragcap::profile::{LoadError, ProviderError, ResolutionError, ResolveError};
use fragcap::{Diagnostics, DurationError, SizeError, SourceError};

/// A process exit code, constrained to the three the contract defines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit(u8);

impl Exit {
    /// Success: a completed capture, passed diagnostics, or an operator
    /// interrupt during capture.
    pub const SUCCESS: Exit = Exit(0);
    /// An expected failure: the target never appeared, the capture driver or a
    /// usable interface was absent, a sink failed, or `doctor` found a blocking
    /// problem.
    pub const FAILURE: Exit = Exit(1);
    /// A usage or configuration error: bad arguments, an invalid profile, or an
    /// unsupported mode, sink, or command.
    pub const USAGE: Exit = Exit(2);

    /// Build an exit from a raw code. Only 0, 1, and 2 are meaningful.
    pub const fn new(code: u8) -> Exit {
        Exit(code)
    }

    /// The raw code.
    pub const fn code(self) -> u8 {
        self.0
    }
}

/// Why a command could not complete, carrying the exit class it maps to.
///
/// Two classes only, because the contract has two failure classes: a usage or
/// configuration error is 2, and an expected failure is 1. A command that
/// completed and merely wants to report a non-zero code (a blocked `doctor`, an
/// acquisition timeout whose summary was already printed) returns the code
/// directly rather than an error, so an error here always means "print this and
/// stop".
#[derive(Debug)]
pub enum CliError {
    /// Maps to exit 2.
    Usage(String),
    /// Maps to exit 1.
    Failure(String),
}

impl CliError {
    /// A usage or configuration error (exit 2).
    pub fn usage(message: impl Into<String>) -> CliError {
        CliError::Usage(message.into())
    }

    /// An expected failure (exit 1).
    pub fn failure(message: impl Into<String>) -> CliError {
        CliError::Failure(message.into())
    }

    /// The exit this error maps to.
    pub fn exit(&self) -> Exit {
        match self {
            CliError::Usage(_) => Exit::USAGE,
            CliError::Failure(_) => Exit::FAILURE,
        }
    }

    /// The message to show the operator.
    pub fn message(&self) -> &str {
        match self {
            CliError::Usage(m) | CliError::Failure(m) => m,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for CliError {}

impl From<Diagnostics> for CliError {
    /// An invalid profile is a configuration error (exit 2). The message is the
    /// full diagnostic set, every problem in one pass, which is what section
    /// 15.4 requires.
    fn from(d: Diagnostics) -> CliError {
        CliError::Usage(d.to_string())
    }
}

impl From<DurationError> for CliError {
    fn from(e: DurationError) -> CliError {
        CliError::Usage(e.to_string())
    }
}

impl From<SizeError> for CliError {
    fn from(e: SizeError) -> CliError {
        CliError::Usage(e.to_string())
    }
}

impl From<LoadError> for CliError {
    /// An invalid profile is a usage error (exit 2); a file that cannot be read
    /// is an expected failure (exit 1).
    fn from(e: LoadError) -> CliError {
        match e {
            LoadError::Invalid(d) => CliError::Usage(d.to_string()),
            LoadError::Read(_) => CliError::Failure(e.to_string()),
        }
    }
}

impl From<ResolveError> for CliError {
    /// A reference that resolves to no profile is an expected failure (exit 1),
    /// whether it is an absent id-slug (`NotFound`) or an unresolvable
    /// path-shaped string that is neither a file nor a valid slug
    /// (`InvalidReference`). The two shapes describe the same "names nothing
    /// resolvable" outcome, so `profile show` and `profile validate` agree on
    /// exit 1 for either. A candidate that won its step and then failed
    /// validation is an invalid *profile*, a configuration error (exit 2); a
    /// candidate that could not be read is an expected failure (exit 1).
    fn from(e: ResolveError) -> CliError {
        match e {
            ResolveError::InvalidReference { .. } => CliError::Failure(e.to_string()),
            ResolveError::NotFound { .. } => CliError::Failure(e.to_string()),
            ResolveError::Load {
                source: LoadError::Invalid(_),
                ..
            } => CliError::Usage(e.to_string()),
            ResolveError::Load { .. } => CliError::Failure(e.to_string()),
        }
    }
}

impl From<ResolutionError> for CliError {
    /// The cascade's failure maps to the same classes as the profile lookup it
    /// wraps, so `run` exits exactly as it did before the resolver was
    /// introduced. A hard provider error and a not-found outcome both reduce to
    /// the underlying [`ResolveError`] and reuse its mapping; a not-resolved
    /// outcome with no profile detail (no provider answered and none recorded a
    /// reason) is an expected failure (exit 1).
    fn from(e: ResolutionError) -> CliError {
        match e {
            ResolutionError::Provider(ProviderError::Profile(inner)) => CliError::from(inner),
            // A hint-database read that failed after the store opened is an
            // operational failure surfaced verbatim (exit 1), not a not-found.
            ResolutionError::Provider(ProviderError::Hint(message)) => {
                CliError::Failure(format!("hint database read failed: {message}"))
            }
            ResolutionError::Unresolved(u) => match u.into_profile_not_found() {
                Some(re) => CliError::from(re),
                None => CliError::Failure("no target could be resolved".to_string()),
            },
        }
    }
}

impl From<ConfigError> for CliError {
    /// A pipeline with no source is a usable-interface-absent failure (exit 1).
    fn from(e: ConfigError) -> CliError {
        CliError::Failure(e.to_string())
    }
}

impl From<PipelineError> for CliError {
    /// An unrecoverable sink failure that ended the run is an expected failure
    /// (exit 1), not a usage error: the output may be partial but the request
    /// was well formed. Specification FR-005a.
    fn from(e: PipelineError) -> CliError {
        CliError::Failure(e.to_string())
    }
}

impl From<SourceError> for CliError {
    fn from(e: SourceError) -> CliError {
        CliError::Failure(e.to_string())
    }
}
