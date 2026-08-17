// SPDX-License-Identifier: Apache-2.0

//! End-to-end detection through the facade (slice S053 US1, FR-008/009, SC-005).
//!
//! Composes the discovery known-roots walk with the `SignatureClassifier` exactly as
//! the CLI's `targets discover` does, over a real temporary install tree, and proves
//! a locally detected engine is presented `verified` and outranks the
//! `heuristic-unverified` tier a remote catalog attribution carries (P-9).

#![cfg(feature = "targets")]

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap::profile::FidelityTier;
use fragcap::targets::{
    DriveType, FsDirectoryLister, KnownRootsSource, SignatureClassifier, TargetSource, Volume,
    VolumeInventory,
};

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(tag: &str) -> TempTree {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "fragcap-facade-detection-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        fs::create_dir_all(&root).expect("create temp root");
        TempTree { root }
    }

    fn touch(&self, rel: &str) {
        let full = self.root.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("parents");
        }
        fs::write(&full, b"").expect("write");
    }

    fn mount(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Inv(Vec<Volume>);
impl VolumeInventory for Inv {
    fn fixed_volumes(&self) -> Vec<Volume> {
        self.0.clone()
    }
}

#[test]
fn a_locally_detected_engine_is_verified_and_outranks_the_remote_tier() {
    // A Unity game under the "Games" known root of a temp volume.
    let tree = TempTree::new("unity");
    tree.touch("Games/MyGame/UnityPlayer.dll");

    // The classifier is built from the bundled Appendix B signatures through the
    // facade, the same source the CLI seeds into the catalog.
    let signatures = fragcap::targets::parse_seed_document(fragcap::targets::BUNDLED_SIGNATURES)
        .expect("bundled signatures parse");
    let set = fragcap::profile::signature::SignatureSet::compile(&signatures);
    let classifier = SignatureClassifier::for_known_root(set);

    let inv = Inv(vec![Volume {
        identity: "vol-t".to_string(),
        mount_point: tree.mount(),
        drive_type: DriveType::Fixed,
    }]);
    let eligible: HashSet<String> = ["vol-t".to_string()].into_iter().collect();
    let lister = FsDirectoryLister;
    let source = KnownRootsSource::new(&inv, &eligible, &lister, &classifier);

    let d = source.discover().expect("discovery runs");
    let game = d
        .candidates
        .iter()
        .find(|c| c.display_name == "MyGame")
        .expect("the Unity game is discovered");

    // The local detection is verified, and verified strictly outranks the
    // heuristic-unverified tier a remote catalog engine attribution carries (P-9).
    assert_eq!(game.fidelity, FidelityTier::Verified);
    assert!(
        game.fidelity > FidelityTier::HeuristicUnverified,
        "local verified outranks the remote heuristic-unverified tier"
    );
    assert!(
        game.evidence.iter().any(|f| f.product == "Unity"),
        "the detected engine is Unity"
    );
}
