// SPDX-License-Identifier: Apache-2.0

//! Tier 3, the interactive source (slice S052), spec 7.2.
//!
//! [`InteractiveSource`] wraps a [`DirectorySource`] with a human confirmation
//! step: an accepted candidate is stamped at authored fidelity, because a human
//! vouched for it (FR-012); a rejected one is counted `declined_by_user`, never
//! lost (P-4). The confirmation is injected through the [`Confirm`] seam so the
//! source is testable with a scripted answer and no console (FR-019).
//!
//! Whether a console is present at all (the non-interactive case) is decided by the
//! caller before this source runs: the CLI refuses interactive authoring with a
//! stated reason when there is no console, rather than this source guessing. So
//! [`Confirm::confirm`] always returns a real yes/no.

use fragcap_profile::FidelityTier;

use crate::source::{CandidateTarget, Discovery, DiscoveryAccount, TargetSource};
use crate::sources::directory::DirectorySource;
use crate::TargetsError;

/// A human confirmation step. Injected so the source is driven by a scripted answer
/// in tests. The real implementation reads a console prompt in the CLI.
pub trait Confirm {
    /// Ask whether to accept `candidate`. `true` accepts (stamp authored), `false`
    /// rejects (count declined).
    fn confirm(&self, candidate: &CandidateTarget) -> bool;
}

/// A scripted confirmation that always returns the same answer. For tests.
pub struct ScriptedConfirm {
    answer: bool,
}

impl ScriptedConfirm {
    /// A confirmation that always accepts (`true`) or always rejects (`false`).
    pub fn new(answer: bool) -> Self {
        ScriptedConfirm { answer }
    }
}

impl Confirm for ScriptedConfirm {
    fn confirm(&self, _candidate: &CandidateTarget) -> bool {
        self.answer
    }
}

/// Tier 3 discovery with a human confirmation step wrapping a [`DirectorySource`].
pub struct InteractiveSource<'a> {
    inner: DirectorySource,
    confirm: &'a dyn Confirm,
}

impl<'a> InteractiveSource<'a> {
    /// Wrap a directory source with a confirmation step.
    pub fn new(inner: DirectorySource, confirm: &'a dyn Confirm) -> Self {
        InteractiveSource { inner, confirm }
    }
}

impl TargetSource for InteractiveSource<'_> {
    fn name(&self) -> &str {
        "interactive"
    }

    fn discover(&self) -> Result<Discovery, TargetsError> {
        let base = self.inner.discover()?;
        let mut account = DiscoveryAccount::default();
        let mut candidates = Vec::new();
        for mut candidate in base.candidates {
            account.considered += 1;
            if self.confirm.confirm(&candidate) {
                // A human vouched for it: stamp authored and attribute to this source.
                candidate.fidelity = FidelityTier::Authored;
                candidate.source_name = self.name().to_string();
                account.produced += 1;
                candidates.push(candidate);
            } else {
                // Rejected, not lost.
                account.declined_by_user += 1;
            }
        }
        Ok(Discovery {
            candidates,
            account,
            ..Discovery::default()
        })
    }

    fn default_fidelity(&self) -> FidelityTier {
        FidelityTier::Authored
    }
}
