// SPDX-License-Identifier: Apache-2.0

//! The signature classifier flowing through the known-roots walk (slice S053 US1).
//!
//! Unlike the S052 known-roots tests, which drive a fixture directory tree, the
//! `SignatureClassifier` reads the real filesystem, so this exercises it against a
//! real temporary tree with `FsDirectoryLister`. It proves the detected engine's
//! `verified` fidelity and its neutral evidence reach the emitted candidate, that a
//! hit stops descent, and that the account stays conserved (P-4).

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use fragcap_profile::{
    FidelityTier, Signature, SignatureCategory, SignatureConfidence, SignatureKind, SignatureSet,
};
use fragcap_targets::{
    CandidateIdentity, DriveType, FsDirectoryLister, KnownRootsSource, SignatureClassifier,
    TargetSource, Volume, VolumeInventory,
};

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(tag: &str) -> TempTree {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "fragcap-detection-walk-{}-{}-{}",
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

fn unity_set() -> SignatureSet {
    SignatureSet::compile(&[
        Signature {
            category: SignatureCategory::Engine,
            kind: SignatureKind::Filename,
            pattern: "UnityPlayer.dll".to_string(),
            product: "Unity".to_string(),
            confidence: SignatureConfidence::Definitive,
        },
        Signature {
            category: SignatureCategory::AntiCheat,
            kind: SignatureKind::Filename,
            pattern: "EasyAntiCheat*.dll".to_string(),
            product: "Easy Anti-Cheat".to_string(),
            confidence: SignatureConfidence::Definitive,
        },
    ])
}

fn multi_engine_set() -> SignatureSet {
    SignatureSet::compile(&[
        Signature {
            category: SignatureCategory::Engine,
            kind: SignatureKind::Filename,
            pattern: "EngineAlpha.dll".to_string(),
            product: "Engine Alpha".to_string(),
            confidence: SignatureConfidence::Definitive,
        },
        Signature {
            category: SignatureCategory::Engine,
            kind: SignatureKind::Filename,
            pattern: "EngineBeta.dll".to_string(),
            product: "Engine Beta".to_string(),
            confidence: SignatureConfidence::Definitive,
        },
    ])
}

#[test]
fn a_detected_engine_reaches_the_candidate_and_stops_descent() {
    // The "Games" known root under a temp volume holds two game directories: one a
    // Unity game carrying EasyAntiCheat, one with no detectable engine.
    let tree = TempTree::new("games");
    tree.touch("Games/UnityGame/UnityPlayer.dll");
    tree.touch("Games/UnityGame/EasyAntiCheat/EasyAntiCheat_x64.dll");
    tree.touch("Games/UnityGame/Nested/DeepDir/marker.txt");
    tree.touch("Games/MysteryGame/readme.txt");

    let inv = Inv(vec![Volume {
        identity: "vol-t".to_string(),
        mount_point: tree.mount(),
        drive_type: DriveType::Fixed,
    }]);
    let eligible: HashSet<String> = ["vol-t".to_string()].into_iter().collect();
    let lister = FsDirectoryLister;
    let classifier = SignatureClassifier::for_known_root(unity_set());

    let source = KnownRootsSource::new(&inv, &eligible, &lister, &classifier);
    let d = source.discover().unwrap();

    // Two games: the Unity one (verified) and the mystery one (heuristic prior).
    assert_eq!(d.account.produced, 2, "two known-root children are games");
    assert!(d.account.is_conserved(), "account conserved (P-4)");

    let unity = d
        .candidates
        .iter()
        .find(|c| c.display_name == "UnityGame")
        .expect("the Unity game is a candidate");
    assert_eq!(
        unity.fidelity,
        FidelityTier::Verified,
        "a definitive local engine marker is verified (P-9)"
    );
    assert!(
        unity.evidence.iter().any(|f| f.product == "Unity"),
        "the detected engine rides as evidence"
    );
    assert!(
        unity
            .evidence
            .iter()
            .any(|f| f.product == "Easy Anti-Cheat"),
        "the anti-cheat rides as neutral evidence"
    );

    // Stop-on-hit: the Unity game's nested subtree produced no extra candidate.
    let nested = d.candidates.iter().any(|c| match &c.identity {
        CandidateIdentity::Path(p) => p.contains("Nested") || p.contains("DeepDir"),
        _ => false,
    });
    assert!(!nested, "descent stopped at the hit (no nested candidate)");

    let mystery = d
        .candidates
        .iter()
        .find(|c| c.display_name == "MysteryGame")
        .expect("the mystery game is still a candidate (structural prior)");
    assert_eq!(
        mystery.fidelity,
        FidelityTier::HeuristicUnverified,
        "a known-root child with no engine is a heuristic game"
    );
    assert!(mystery.evidence.is_empty(), "no evidence without a match");
}

#[test]
fn a_multi_engine_container_yields_its_child_titles_with_native_paths() {
    let tree = TempTree::new("container");
    tree.touch("Games/Collection/TitleA/EngineAlpha.dll");
    tree.touch("Games/Collection/TitleB/EngineBeta.dll");

    let inv = Inv(vec![Volume {
        identity: "vol-t".to_string(),
        mount_point: tree.mount(),
        drive_type: DriveType::Fixed,
    }]);
    let eligible: HashSet<String> = ["vol-t".to_string()].into_iter().collect();
    let lister = FsDirectoryLister;
    let classifier = SignatureClassifier::for_known_root(multi_engine_set());

    let source = KnownRootsSource::new(&inv, &eligible, &lister, &classifier);
    let d = source.discover().unwrap();

    assert_eq!(d.account.container_descended, 1);
    assert_eq!(d.account.container_descent_truncated, 0);
    assert_eq!(d.account.produced, 2);
    assert!(d.account.is_conserved());
    assert!(!d
        .candidates
        .iter()
        .any(|candidate| candidate.display_name == "Collection"));
    assert!(d
        .candidates
        .iter()
        .any(|candidate| candidate.display_name == "TitleA"));
    assert!(d
        .candidates
        .iter()
        .any(|candidate| candidate.display_name == "TitleB"));

    for candidate in &d.candidates {
        let CandidateIdentity::Path(identity) = &candidate.identity else {
            panic!("known-roots candidate must use a path identity");
        };
        assert_eq!(candidate.install_root.as_deref(), Some(identity.as_str()));
        assert!(
            !(identity.contains('/') && identity.contains('\\')),
            "candidate path mixes separators: {identity}"
        );
        #[cfg(windows)]
        assert!(
            !identity.contains('/'),
            "Windows candidate path must be native: {identity}"
        );
    }
}
