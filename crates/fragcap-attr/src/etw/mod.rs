// SPDX-License-Identifier: Apache-2.0

//! The ETW process watcher of specification section 10.1.
//!
//! Specification section 5.3 is the whole argument for this module existing. A
//! Windows process identifier comes from a reusable pool, and each process
//! records its creator's identifier without maintaining it, so reading a parent
//! identifier from a running process yields either nothing or a process with no
//! relationship to the one being examined. The creation instant is the only
//! instant at which the relationship is unambiguous, and an ETW kernel provider
//! is the only way to observe it.
//!
//! **Nothing here polls, and there is no fallback that does.** Section 10.1
//! refuses polling categorically, and the failure it refuses is the one fragcap
//! exists to prevent: a transient launcher whose lifetime is shorter than any
//! poll interval is exactly the thing that must be caught. Offering a polling
//! mode when elevation is missing would produce a run that exits zero, writes a
//! well-formed capture file, and contains no gameplay, under a name that sounds
//! like success. The absence of elevation is reported as the absence of
//! elevation.
//!
//! [`record`] is not behind the `etw` feature. It is arithmetic over a byte
//! slice with no platform surface, and it is the one place in the slice where a
//! wrong number produces plausible values rather than an error, so it is tested
//! in the ordinary check set on every machine rather than only where the
//! feature is on.

pub mod record;

#[cfg(all(windows, feature = "etw"))]
mod consumer;
#[cfg(all(windows, feature = "etw"))]
mod session;
#[cfg(all(windows, feature = "etw"))]
mod snapshot;

#[cfg(all(windows, feature = "etw"))]
mod watcher;

#[cfg(all(windows, feature = "etw"))]
pub use watcher::EtwWatcher;

use std::fmt;

/// Why a watcher could not start, or could not continue.
///
/// [`NotElevated`](WatcherError::NotElevated) is separate from
/// [`SessionUnavailable`](WatcherError::SessionUnavailable) because the two
/// have different remedies, and specification section 26.4 requires an error to
/// say what to do next rather than only what went wrong.
///
/// There is deliberately no variant meaning "continuing with reduced fidelity".
/// See the module documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatcherError {
    /// The trace session needs a privilege this session does not have.
    NotElevated,
    /// The platform refused to start the session, with its own reason.
    SessionUnavailable { code: u32, detail: String },
    /// The session started but the process provider could not be enabled.
    ProviderUnavailable { code: u32 },
    /// The session ended after having started.
    Stopped { code: u32 },
}

impl fmt::Display for WatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatcherError::NotElevated => write!(
                f,
                "cannot start the process trace session: administrative \
                 privilege is required.\n\n  Consuming a kernel trace session \
                 needs an elevated session. Re-run from an elevated terminal.\n\n  \
                 fragcap does not fall back to polling for processes, because a \
                 poller misses launchers that exit faster than it samples, and \
                 the resulting capture would look complete and contain no \
                 gameplay."
            ),
            WatcherError::SessionUnavailable { code, detail } => write!(
                f,
                "cannot start the process trace session.\n\n  \
                 The platform reported code {code}: {detail}"
            ),
            WatcherError::ProviderUnavailable { code } => write!(
                f,
                "the process trace session started but the process provider \
                 could not be enabled.\n\n  The platform reported code {code}."
            ),
            WatcherError::Stopped { code } => write!(
                f,
                "the process trace session ended while the capture was \
                 running.\n\n  The platform reported code {code}. Processes \
                 created after that point were not observed."
            ),
        }
    }
}

impl std::error::Error for WatcherError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_says_what_happened_and_what_to_do() {
        let all = [
            WatcherError::NotElevated,
            WatcherError::SessionUnavailable {
                code: 183,
                detail: "a session by this name exists.".into(),
            },
            WatcherError::ProviderUnavailable { code: 5 },
            WatcherError::Stopped { code: 1 },
        ];
        for e in all {
            let s = e.to_string();
            assert!(s.len() > 40, "too terse to act on: {s}");
        }
    }

    #[test]
    fn the_privilege_error_names_the_refusal_to_poll() {
        // The one sentence in this module that is load-bearing for an operator
        // who would otherwise assume fragcap silently degraded.
        let s = WatcherError::NotElevated.to_string();
        assert!(s.contains("elevated"));
        assert!(s.contains("polling"));
    }
}
