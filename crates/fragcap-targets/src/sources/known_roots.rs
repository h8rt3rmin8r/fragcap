// SPDX-License-Identifier: Apache-2.0

//! Tier 2: the known-roots walk (slice S052), spec 7.2.
//!
//! A machine without Steam must still show games. [`KnownRootsSource`] enumerates a
//! fixed, hard-coded list of conventional game-library locations, across
//! every eligible fixed volume (a second or third drive holds games as often as
//! the system drive). Exhaustive enumeration of every executable on the machine is
//! rejected (FR-009): a normal machine carries thousands of updaters, uninstallers,
//! and helpers that would bury the game. Instead the walk tests each directory
//! through a [`DirectoryClassifier`], stops descending on a title hit, and descends
//! through a multi-engine container while the shallow bound permits (FR-015,
//! S077).

use std::collections::HashSet;

use fragcap_profile::FidelityTier;

use crate::classifier::{ClassifierVerdict, DirectoryClassifier};
use crate::source::{CandidateIdentity, CandidateTarget, Discovery, TargetSource};
use crate::sources::{base_name, DirListing, DirectoryLister};
use crate::volume::VolumeInventory;
use crate::TargetsError;

/// The fixed v0.5.0 known-root list (FR-007): conventional game-library and
/// install locations, each relative to a volume root. Location bounds the walk but
/// does not itself authorize automatic registration (S133).
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
    excluded_dirs: HashSet<String>,
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
            excluded_dirs: HashSet::new(),
        }
    }

    /// Exclude exact directory roots and their descendants when a more
    /// authoritative platform source owns that subtree. Component boundaries are
    /// significant, so excluding `Steam` does not exclude `Steamship`.
    pub fn with_excluded_dirs(mut self, dirs: impl IntoIterator<Item = String>) -> Self {
        self.excluded_dirs = dirs
            .into_iter()
            .map(|dir| normalized_dir_key(&dir))
            .collect();
        self
    }

    /// Walk one directory, classifying its immediate children. A hit emits one
    /// candidate and stops (no descent into the hit's subtree, FR-015); a container
    /// is counted separately and descended while depth remains; a miss is counted
    /// and, while depth remains, descended one level (a launcher-nested layout).
    /// `Absent` contributes nothing (FR-010); an access error is counted.
    fn walk(&self, dir: &str, depth: usize, out: &mut Discovery) {
        if self.is_excluded(dir) {
            out.account.considered += 1;
            out.account.considered_not_a_game += 1;
            return;
        }
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
                    if self.is_excluded(&child) {
                        out.account.considered_not_a_game += 1;
                        continue;
                    }
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
                                install_root: Some(child.clone()),
                                folder_name: None,
                                executable_hint: None,
                            });
                            // Stop-on-hit: do not descend into a hit's subtree.
                        }
                        ClassifierVerdict::Container => {
                            if depth + 1 < MAX_DESCENT {
                                out.account.container_descended += 1;
                                self.walk(&child, depth + 1, out);
                            } else {
                                out.account.container_descent_truncated += 1;
                                out.warnings.push(format!(
                                    "known-roots container reached the descent limit; descendants may remain undiscovered: {child}"
                                ));
                            }
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

    fn is_excluded(&self, dir: &str) -> bool {
        let key = normalized_dir_key(dir);
        self.excluded_dirs.iter().any(|root| {
            key == *root
                || key
                    .strip_prefix(root)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        })
    }
}

fn normalized_dir_key(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let normalized = if let Some(rest) = normalized.strip_prefix("//?/UNC/") {
        format!("//{rest}")
    } else if let Some(rest) = normalized.strip_prefix("//?/") {
        rest.to_string()
    } else {
        normalized
    };
    normalized.trim_end_matches('/').to_ascii_lowercase()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriveType, FixtureClassifier, FixtureInventory, FixtureTree, Volume};

    fn inventory() -> FixtureInventory {
        FixtureInventory::new(vec![Volume {
            identity: "volume-a".to_string(),
            mount_point: "A:".to_string(),
            drive_type: DriveType::Fixed,
        }])
    }

    fn eligible() -> HashSet<String> {
        HashSet::from(["volume-a".to_string()])
    }

    #[test]
    fn excluded_platform_root_prunes_the_complete_client_fixture_before_listing() {
        let common = "A:/Program Files (x86)/Steam/steamapps/common";
        let children = [
            "appcache",
            "config",
            "logs",
            "package",
            "resource",
            "steamui",
            "userdata",
            "Supported Game",
            "Unsupported Game",
        ]
        .map(|name| format!("{common}/{name}"));
        let child_refs = children.iter().map(String::as_str).collect::<Vec<_>>();
        let tree = FixtureTree::new().with_dir(common, &child_refs);
        let classifier = FixtureClassifier::new(children.to_vec());
        let inventory = inventory();
        let eligible = eligible();
        let source = KnownRootsSource::new(&inventory, &eligible, &tree, &classifier)
            .with_excluded_dirs(["\\\\?\\a:\\PROGRAM FILES (X86)\\STEAM\\".to_string()]);

        let discovery = source.discover().expect("discover");
        assert!(discovery.candidates.is_empty());
        assert_eq!(discovery.account.considered, 1);
        assert_eq!(discovery.account.considered_not_a_game, 1);
        assert!(discovery.account.is_conserved());
    }

    #[test]
    fn excluded_platform_root_prunes_nested_generic_games_child_but_not_prefix_sibling() {
        let games = "A:/Games";
        let steam = "A:/Games/Steam";
        let prefix_sibling = "A:/Games/Steamship";
        let tree = FixtureTree::new().with_dir(games, &[steam, prefix_sibling]);
        let classifier =
            FixtureClassifier::new(vec![steam.to_string(), prefix_sibling.to_string()]);
        let inventory = inventory();
        let eligible = eligible();
        let source = KnownRootsSource::new(&inventory, &eligible, &tree, &classifier)
            .with_excluded_dirs(["A:/Games/Steam".to_string()]);

        let discovery = source.discover().expect("discover");
        assert_eq!(discovery.candidates.len(), 1);
        assert_eq!(
            discovery.candidates[0].install_root.as_deref(),
            Some(prefix_sibling)
        );
        assert_eq!(discovery.account.considered_not_a_game, 1);
        assert!(discovery.account.is_conserved());
    }
}
