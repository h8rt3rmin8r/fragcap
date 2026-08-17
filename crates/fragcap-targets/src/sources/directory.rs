// SPDX-License-Identifier: Apache-2.0

//! Tier 3, the user-pointed directory source (slice S052), spec 7.2.
//!
//! A user who knows exactly where their game is points discovery straight at it.
//! [`DirectorySource`] takes one path and yields one candidate for it. It asserts
//! no classification (the user vouches for the location, not for what the tool
//! should call it), so the candidate is `Unknown` until authored (P-9). It backs
//! `targets scan <dir>` and, wrapped by [`super::interactive::InteractiveSource`],
//! `targets add <exe>`.

use fragcap_profile::FidelityTier;

use crate::source::{
    CandidateIdentity, CandidateTarget, Discovery, DiscoveryAccount, TargetSource,
};
use crate::sources::base_name;
use crate::TargetsError;

/// Tier 3 discovery from a single user-supplied path.
pub struct DirectorySource {
    path: String,
}

impl DirectorySource {
    /// Build a directory source for one path.
    pub fn new(path: impl Into<String>) -> Self {
        DirectorySource { path: path.into() }
    }

    /// The path this source points at (used by the interactive wrapper).
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl TargetSource for DirectorySource {
    fn name(&self) -> &str {
        "directory"
    }

    fn discover(&self) -> Result<Discovery, TargetsError> {
        let mut account = DiscoveryAccount::default();
        let mut candidates = Vec::new();
        // At most one candidate: an empty path yields none.
        if !self.path.trim().is_empty() {
            account.produce();
            candidates.push(CandidateTarget {
                identity: CandidateIdentity::Path(self.path.clone()),
                display_name: base_name(&self.path),
                fidelity: self.default_fidelity(),
                classification: crate::entry::TargetClassification::Unknown,
                source_name: self.name().to_string(),
            });
        }
        Ok(Discovery {
            candidates,
            account,
            ..Discovery::default()
        })
    }

    fn default_fidelity(&self) -> FidelityTier {
        // A pointed-at directory with no human confirmation is a guess, not an
        // authored definition; the interactive wrapper upgrades an accepted one.
        FidelityTier::HeuristicUnverified
    }
}
