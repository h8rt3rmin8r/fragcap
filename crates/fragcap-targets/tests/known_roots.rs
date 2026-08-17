// SPDX-License-Identifier: Apache-2.0

//! Tier 2, the known-roots walk (S052 spec US2, FR-007/008/009/010/014/015).
//!
//! Driven entirely by a fixture inventory and a fixture directory tree, no
//! filesystem. Every assertion also checks `is_conserved()` (P-4).

use std::collections::HashSet;

use fragcap_targets::{
    CandidateIdentity, DriveType, FixtureClassifier, FixtureTree, KnownRootChildIsGame,
    KnownRootsSource, TargetSource, Volume,
};

fn vol(id: &str, mount: &str) -> Volume {
    Volume {
        identity: id.to_string(),
        mount_point: mount.to_string(),
        drive_type: DriveType::Fixed,
    }
}

/// A single-volume inventory value, boxed so the source can borrow it.
struct Inv(Vec<Volume>);
impl fragcap_targets::VolumeInventory for Inv {
    fn fixed_volumes(&self) -> Vec<Volume> {
        self.0.clone()
    }
}

fn candidate_paths(d: &fragcap_targets::Discovery) -> Vec<String> {
    d.candidates
        .iter()
        .map(|c| match &c.identity {
            CandidateIdentity::Path(p) => p.clone(),
            other => panic!("expected a path identity, got {other:?}"),
        })
        .collect()
}

#[test]
fn no_steam_but_a_known_root_lists_games_across_volumes() {
    // Two fixed volumes; an Epic Games root on each, holding games. No Steam.
    let inv = Inv(vec![vol("vol-c", "C:"), vol("vol-d", "D:")]);
    let eligible: HashSet<String> = ["vol-c".to_string(), "vol-d".to_string()]
        .into_iter()
        .collect();
    let tree = FixtureTree::new()
        .with_dir(
            "C:/Program Files/Epic Games",
            &[
                "C:/Program Files/Epic Games/Fortnite",
                "C:/Program Files/Epic Games/RocketLeague",
            ],
        )
        .with_dir(
            "D:/Program Files/Epic Games",
            &["D:/Program Files/Epic Games/Alan Wake"],
        );
    let classifier = KnownRootChildIsGame;

    let source = KnownRootsSource::new(&inv, &eligible, &tree, &classifier);
    let d = source.discover().unwrap();

    let paths = candidate_paths(&d);
    assert_eq!(paths.len(), 3, "two games on C:, one on D:");
    assert!(
        paths.iter().any(|p| p.ends_with("Alan Wake")),
        "the second volume is enumerated (FR-008)"
    );
    assert!(d.account.is_conserved());
    assert_eq!(d.account.produced, 3);
}

#[test]
fn a_missing_root_contributes_nothing_and_no_error() {
    let inv = Inv(vec![vol("vol-c", "C:")]);
    let eligible: HashSet<String> = ["vol-c".to_string()].into_iter().collect();
    // Only one of the eleven roots exists; the other ten are absent.
    let tree = FixtureTree::new().with_dir("C:/EA Games", &["C:/EA Games/Battlefield"]);
    let classifier = KnownRootChildIsGame;

    let source = KnownRootsSource::new(&inv, &eligible, &tree, &classifier);
    let d = source.discover().unwrap();

    assert_eq!(candidate_paths(&d).len(), 1);
    assert_eq!(
        d.account.access_error, 0,
        "a missing root is not an error (FR-010)"
    );
    assert!(d.account.is_conserved());
}

#[test]
fn descent_stops_on_a_hit_and_descends_on_a_miss() {
    let inv = Inv(vec![vol("vol-c", "C:")]);
    let eligible: HashSet<String> = ["vol-c".to_string()].into_iter().collect();
    // Epic Games holds "Fortnite" (a game, hit) which itself holds a "decoy"
    // subdirectory that WOULD be a hit if the walk ever classified it. Epic Games
    // also holds "Launcher" (a miss) which holds "RealGame" (a hit one level down).
    let tree = FixtureTree::new()
        .with_dir(
            "C:/Program Files/Epic Games",
            &[
                "C:/Program Files/Epic Games/Fortnite",
                "C:/Program Files/Epic Games/Launcher",
            ],
        )
        .with_dir(
            "C:/Program Files/Epic Games/Fortnite",
            &["C:/Program Files/Epic Games/Fortnite/decoy"],
        )
        .with_dir(
            "C:/Program Files/Epic Games/Launcher",
            &["C:/Program Files/Epic Games/Launcher/RealGame"],
        );
    // The classifier hits Fortnite, the decoy, and RealGame; it misses Launcher.
    let classifier = FixtureClassifier::new(vec![
        "C:/Program Files/Epic Games/Fortnite".to_string(),
        "C:/Program Files/Epic Games/Fortnite/decoy".to_string(),
        "C:/Program Files/Epic Games/Launcher/RealGame".to_string(),
    ]);

    let source = KnownRootsSource::new(&inv, &eligible, &tree, &classifier);
    let d = source.discover().unwrap();
    let paths = candidate_paths(&d);

    // Fortnite is a hit, so the walk never descends into it: the decoy beneath it
    // is never classified and never becomes a candidate (stop-on-hit, FR-015).
    assert!(paths.iter().any(|p| p.ends_with("Fortnite")));
    assert!(
        !paths.iter().any(|p| p.ends_with("decoy")),
        "a hit's subtree is not descended"
    );
    // Launcher is a miss, so the walk descends one level and finds RealGame.
    assert!(
        paths.iter().any(|p| p.ends_with("RealGame")),
        "a miss is descended one level"
    );
    assert!(
        !paths.iter().any(|p| p.ends_with("Launcher")),
        "the miss itself is not a candidate"
    );
    assert_eq!(
        d.account.considered_not_a_game, 1,
        "Launcher is the one miss"
    );
    assert!(d.account.is_conserved());
}

#[test]
fn an_ineligible_volume_is_never_enumerated() {
    let inv = Inv(vec![vol("vol-c", "C:"), vol("vol-d", "D:")]);
    // Only C: is eligible; D: is excluded/unseen.
    let eligible: HashSet<String> = ["vol-c".to_string()].into_iter().collect();
    let tree = FixtureTree::new()
        .with_dir("C:/Games", &["C:/Games/Indie"])
        .with_dir("D:/Games", &["D:/Games/SecretOnExcludedVolume"]);
    let classifier = KnownRootChildIsGame;

    let source = KnownRootsSource::new(&inv, &eligible, &tree, &classifier);
    let d = source.discover().unwrap();
    let paths = candidate_paths(&d);

    assert!(paths.iter().any(|p| p.ends_with("Indie")));
    assert!(
        !paths.iter().any(|p| p.contains("SecretOnExcludedVolume")),
        "an ineligible volume is enumerated zero times (SC-003)"
    );
    assert_eq!(
        d.account.volume_skipped, 1,
        "the excluded volume is counted, not silent"
    );
    assert!(d.account.is_conserved());
}

#[test]
fn an_unreadable_root_is_counted_access_error() {
    let inv = Inv(vec![vol("vol-c", "C:")]);
    let eligible: HashSet<String> = ["vol-c".to_string()].into_iter().collect();
    let tree = FixtureTree::new().with_access_error("C:/Program Files/Epic Games");
    let classifier = KnownRootChildIsGame;

    let source = KnownRootsSource::new(&inv, &eligible, &tree, &classifier);
    let d = source.discover().unwrap();

    assert_eq!(d.account.access_error, 1);
    assert_eq!(d.account.produced, 0);
    assert!(d.account.is_conserved());
}
