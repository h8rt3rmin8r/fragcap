// SPDX-License-Identifier: Apache-2.0

//! Tier 2: the known-roots walk (slice S052), spec 7.2.
//!
//! A machine without Steam must still show games. [`KnownRootsSource`] enumerates a
//! fixed, hard-coded list of directories that only ever contain games, across
//! every eligible fixed volume (a second or third drive holds games as often as
//! the system drive). Exhaustive enumeration of every executable on the machine is
//! rejected (FR-009): a normal machine carries thousands of updaters, uninstallers,
//! and helpers that would bury the game. Instead the walk tests each directory
//! through a [`DirectoryClassifier`] and stops descending on a hit (FR-015).

use std::collections::HashSet;

use fragcap_profile::FidelityTier;

use crate::classifier::{ClassifierVerdict, DirectoryClassifier};
use crate::source::{CandidateIdentity, CandidateTarget, Discovery, TargetSource};
use crate::sources::{base_name, DirListing, DirectoryLister};
use crate::volume::VolumeInventory;
use crate::TargetsError;

/// The fixed v0.5.0 known-root list (FR-007): directories that only ever contain
/// games, each a path relative to a volume root. The walk applies all of them to
/// every eligible volume. A separator-normalized relative form is used so the same
/// constant drives both the real filesystem walk and the fixture tree.
pub const KNOWN_ROOTS: &[&str] = &[
    "SteamLibrary/steamapps/common",
    "Program Files (x86)/Steam/steamapps/common",
    "Program Files/Epic Games",
    "GOG Galaxy/Games",
    "Riot Games",
    "Battle.net",
    "Ubisoft/Ubisoft Game Launcher/games",
    "EA Games",
    "Origin Games",
    "XboxGames",
    "Games",
];

/// The greatest depth the walk descends beneath a known root before giving up. A
/// known root holds games directly (depth 1); one further level covers a
/// launcher-nested layout (publisher folder then game). Bounded so the walk stays
/// shallow: deep filesystem scanning is deferred to v0.6.0.
const MAX_DESCENT: usize = 2;

/// Tier 2 discovery: the known-roots walk over the eligible fixed volumes.
///
/// Pure: the live volume list, the eligible-volume identities, the directory
/// lister, and the classifier are all injected, so the whole walk is a decision
/// over values in tests (FR-019). The caller (the facade) seeds and queries the
/// eligibility table and passes the eligible identities in; this source holds no
/// store.
pub struct KnownRootsSource<'a> {
    inventory: &'a dyn VolumeInventory,
    eligible_ids: &'a HashSet<String>,
    lister: &'a dyn DirectoryLister,
    classifier: &'a dyn DirectoryClassifier,
}

impl<'a> KnownRootsSource<'a> {
    /// Build a known-roots source from the live inventory, the set of eligible
    /// volume identities, the directory lister, and the classifier.
    pub fn new(
        inventory: &'a dyn VolumeInventory,
        eligible_ids: &'a HashSet<String>,
        lister: &'a dyn DirectoryLister,
        classifier: &'a dyn DirectoryClassifier,
    ) -> Self {
        KnownRootsSource {
            inventory,
            eligible_ids,
            lister,
            classifier,
        }
    }

    /// Walk one directory, classifying its immediate children. A hit emits one
    /// candidate and stops (no descent into the hit's subtree, FR-015); a miss is
    /// counted and, while depth remains, descended one level (a launcher-nested
    /// layout). `Absent` contributes nothing (FR-010); an access error is counted.
    fn walk(&self, dir: &str, depth: usize, out: &mut Discovery) {
        match self.lister.subdirectories(dir) {
            DirListing::Absent => {}
            DirListing::AccessError => {
                out.account.considered += 1;
                out.account.access_error += 1;
                // Name the root that failed so "some access error occurred" is
                // recoverable to which of the eleven roots on which volume failed,
                // while the scalar count stays conserved (P-4).
                out.warnings
                    .push(format!("could not read known root: {dir}"));
            }
            DirListing::Present(children) => {
                for child in children {
                    out.account.considered += 1;
                    let classification = self.classifier.classify(&child);
                    // Whatever the classifier could not cover reduces detection
                    // coverage; name it so a partial scan is visible, not silent
                    // (P-4). The lines arrive finished, so a cause added later is
                    // forwarded here without this walk knowing about it.
                    out.warnings.extend(classification.coverage_warnings);
                    match classification.verdict {
                        ClassifierVerdict::Hit {
                            classification,
                            fidelity,
                            evidence,
                            detection_scan,
                        } => {
                            out.account.produced += 1;
                            out.candidates.push(CandidateTarget {
                                identity: CandidateIdentity::Path(child.clone()),
                                display_name: base_name(&child),
                                // The classifier earns the fidelity: a definitive
                                // local engine marker is Verified (P-9).
                                fidelity,
                                classification,
                                evidence,
                                detection_scan,
                                source_name: self.name().to_string(),
                            });
                            // Stop-on-hit: do not descend into a hit's subtree.
                        }
                        ClassifierVerdict::Miss => {
                            out.account.considered_not_a_game += 1;
                            if depth + 1 < MAX_DESCENT {
                                self.walk(&child, depth + 1, out);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl TargetSource for KnownRootsSource<'_> {
    fn name(&self) -> &str {
        "known-roots"
    }

    fn discover(&self) -> Result<Discovery, TargetsError> {
        let mut out = Discovery::default();
        for volume in self.inventory.fixed_volumes() {
            // An ineligible or unseen volume is never enumerated; the skip is
            // counted so it is visible, not silent (FR-017, SC-003).
            if !self.eligible_ids.contains(&volume.identity) {
                out.account.considered += 1;
                out.account.volume_skipped += 1;
                continue;
            }
            let mount = volume.mount_point.trim_end_matches(['/', '\\']);
            for root in KNOWN_ROOTS {
                let root_path = format!("{mount}/{root}");
                self.walk(&root_path, 0, &mut out);
            }
        }
        Ok(out)
    }

    fn default_fidelity(&self) -> FidelityTier {
        FidelityTier::HeuristicUnverified
    }
}
