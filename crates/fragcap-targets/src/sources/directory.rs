// SPDX-License-Identifier: Apache-2.0

//! Tier 3, the user-pointed directory source (slice S052, S053), spec 7.2.
//!
//! A user who knows exactly where their game is points discovery straight at it.
//! [`DirectorySource`] takes one path and yields one candidate for it. It asserts
//! no classification (the user vouches for the location, not for what the tool
//! should call it), so the candidate is `Unknown` until authored (P-9). It backs
//! `targets scan <dir>` and, wrapped by [`super::interactive::InteractiveSource`],
//! `targets add <exe>`.
//!
//! When built with a signature set (slice S053), it scans the pointed-at directory
//! for technologies: a detected engine rides as evidence and raises the candidate's
//! fidelity to what its signature earns (a definitive marker is verified, P-9), and
//! any anti-cheat or DRM rides as neutral evidence. Detection is signature-driven
//! and runs in every source's scan phase (FR-006). Built without one (the bare
//! [`DirectorySource::new`]), it emits a heuristic candidate with no evidence, the
//! S052 behavior.

use std::path::Path;

use fragcap_profile::signature::SignatureSet;
use fragcap_profile::{DetectionFinding, FidelityTier};

use crate::source::{
    CandidateIdentity, CandidateTarget, Discovery, DiscoveryAccount, TargetSource,
};
use crate::sources::base_name;
use crate::TargetsError;

/// Tier 3 discovery from a single user-supplied path.
pub struct DirectorySource {
    path: String,
    signatures: Option<SignatureSet>,
}

impl DirectorySource {
    /// Build a directory source for one path, running no signature detection.
    pub fn new(path: impl Into<String>) -> Self {
        DirectorySource {
            path: path.into(),
            signatures: None,
        }
    }

    /// Build a directory source that scans the path for technologies with the given
    /// signature set (slice S053).
    pub fn with_signatures(path: impl Into<String>, signatures: SignatureSet) -> Self {
        DirectorySource {
            path: path.into(),
            signatures: Some(signatures),
        }
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
        let mut warnings = Vec::new();
        // At most one candidate: an empty path yields none.
        if !self.path.trim().is_empty() {
            account.produce();
            let (fidelity, evidence, detection_scan) = self.detect(&mut warnings);
            candidates.push(CandidateTarget {
                identity: CandidateIdentity::Path(self.path.clone()),
                display_name: base_name(&self.path),
                fidelity,
                // The user vouches for the location, not for what the tool should call
                // it; classification stays Unknown until authored (P-9). Detected
                // technologies ride as evidence regardless.
                classification: crate::entry::TargetClassification::Unknown,
                evidence,
                detection_scan,
                source_name: self.name().to_string(),
                // A directory-scan candidate has no installdir concept distinct
                // from `display_name`, and no observed launch executable.
                folder_name: None,
                executable_hint: None,
            });
        }
        Ok(Discovery {
            candidates,
            account,
            warnings,
        })
    }

    fn default_fidelity(&self) -> FidelityTier {
        // A pointed-at directory with no human confirmation is a guess, not an
        // authored definition; the interactive wrapper upgrades an accepted one.
        FidelityTier::HeuristicUnverified
    }
}

impl DirectorySource {
    /// Detect the technologies in the pointed-at directory, or return the bare
    /// heuristic default when no signature set was supplied. A detected engine raises
    /// the fidelity; an unreadable path or subtree is surfaced into `warnings` rather
    /// than dropped (P-4). The third value is the coverage state (slice S065):
    /// `None` when no signature set was supplied and so no scan ran at all.
    fn detect(
        &self,
        warnings: &mut Vec<String>,
    ) -> (
        FidelityTier,
        Vec<DetectionFinding>,
        Option<crate::entry::DetectionScan>,
    ) {
        let Some(signatures) = &self.signatures else {
            return (self.default_fidelity(), Vec::new(), None);
        };
        match signatures.detect(Path::new(&self.path)) {
            Ok(outcome) => {
                // Everything the scan did not cover, named rather than only counted
                // (P-4). One shared implementation, so this source and the platform
                // walk cannot drift apart on what they report.
                warnings.extend(outcome.coverage_warnings());
                let scan = crate::entry::DetectionScan::from_outcome(&outcome);
                let fidelity = outcome
                    .detected_engine()
                    .map(|e| e.fidelity)
                    .unwrap_or_else(|| self.default_fidelity());
                (fidelity, outcome.findings, Some(scan))
            }
            Err(e) => {
                warnings.push(format!(
                    "could not read directory during detection: {}",
                    e.path.display()
                ));
                // A scan was attempted and covered nothing. `Incomplete`, not
                // `None`: an attempt that failed is a different fact from no
                // attempt (P-4).
                (
                    self.default_fidelity(),
                    Vec::new(),
                    Some(crate::entry::DetectionScan::Incomplete),
                )
            }
        }
    }
}
