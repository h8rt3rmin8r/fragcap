// SPDX-License-Identifier: Apache-2.0

//! The machine-wide anti-cheat presence seam (slice S068, issue #170).
//!
//! Modern Easy Anti-Cheat installs once per machine as a service and driver
//! outside any game's install tree, structurally invisible to a directory scan no
//! matter how many signature rows are added. This module carries the model and the
//! injectable seam; the one real (Windows) implementation, reading the service
//! registry, lives in the `fragcap` facade, the crate that already carries the
//! other platform-specific adapters ([`crate::VolumeInventory`]'s Windows
//! implementation among them). `fragcap-targets` stays free of any platform
//! dependency (`cargo xtask deps` and the neutral-build gate both check this).
//!
//! A machine-wide finding is a distinct kind of fact from a per-title
//! [`crate::readiness`] evidence entry and is never merged into one: doing so
//! would attribute a fact about the whole machine to one title with no proof that
//! title caused it, exactly the false-positive class issue #170 warns against for
//! `EOSSDK-Win64-Shipping.dll`. See `docs/glossary/anti-cheat-and-security.md`'s
//! "Machine scope" entry.

/// A product found present at machine scope, with the evidence that established
/// it. No fidelity is carried: a machine-wide finding never competes with, or
/// merges into, a title-scope [`crate::DetectionFinding`], so it never needs to be
/// ranked against one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineAntiCheatFinding {
    /// The product name, matching the vocabulary a title-scope finding uses (for
    /// example "Easy Anti-Cheat").
    pub product: String,
    /// What was actually observed (for example the registry key that was found
    /// present), so the claim always traces to an inspectable fact.
    pub evidence: String,
}

/// The machine-wide anti-cheat presence check. Injectable so the machine-scope
/// path is testable on a runner with no anti-cheat software installed (FR-009).
///
/// `detect` never fails: a probe that could not run at all (a non-Windows host, a
/// permission failure) returns an empty `Vec`, identical in shape to "ran and
/// found nothing." A caller must never render that empty result as a confirmed
/// negative ("no anti-cheat products found"), it must render nothing at all
/// (FR-008), since neither case can be told apart from the result alone and only
/// the "found nothing" case is actually a completed check.
pub trait MachineAntiCheatProbe {
    /// The products found present on this machine.
    fn detect(&self) -> Vec<MachineAntiCheatFinding>;
}

/// A fixture probe returning a canned finding list, for tests that need to drive
/// the machine-scope rendering path without a real registry read.
pub struct FixtureMachineAntiCheatProbe {
    findings: Vec<MachineAntiCheatFinding>,
}

impl FixtureMachineAntiCheatProbe {
    /// Build a fixture probe from a canned finding list.
    pub fn new(findings: Vec<MachineAntiCheatFinding>) -> Self {
        FixtureMachineAntiCheatProbe { findings }
    }
}

impl MachineAntiCheatProbe for FixtureMachineAntiCheatProbe {
    fn detect(&self) -> Vec<MachineAntiCheatFinding> {
        self.findings.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixture_probe_returns_exactly_what_it_was_built_with() {
        let findings = vec![MachineAntiCheatFinding {
            product: "Easy Anti-Cheat".to_string(),
            evidence: "service EasyAntiCheat_EOS registered".to_string(),
        }];
        let probe = FixtureMachineAntiCheatProbe::new(findings.clone());
        assert_eq!(probe.detect(), findings);
    }

    #[test]
    fn an_empty_fixture_probe_returns_no_findings() {
        let probe = FixtureMachineAntiCheatProbe::new(Vec::new());
        assert!(probe.detect().is_empty());
    }
}
