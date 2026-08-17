// SPDX-License-Identifier: Apache-2.0

//! The directory-shape classifier seam (slice S052), with its real signature
//! matcher (slice S053), specification section 7.3 and 3.6.
//!
//! Tiers 2 and 3 classify a directory by its shape, the presence of an engine
//! signature such as a Unity player library or an Unreal engine-binaries tree,
//! rather than by a curated per-title list, so a standalone or non-catalog title
//! is recognized as a game. S052 shipped the seam and the descent contract; S053
//! fills it with [`SignatureClassifier`], the generic matcher over the catalog's
//! signature table.
//!
//! The descent contract (S052 FR-015): a walk tests each directory through a
//! classifier and stops descending on a [`ClassifierVerdict::Hit`], emitting one
//! candidate; it never enumerates a directory's executables first and then asks
//! whether each is a game (FR-009).
//!
//! A [`ClassifierVerdict::Hit`] carries the fidelity the match earns (a definitive
//! local engine marker is [`FidelityTier::Verified`], which outranks a remote
//! catalog attribution, P-9) and the neutral evidence detected alongside it (any
//! anti-cheat or DRM). The evidence is a set of facts and nothing more: no field on
//! it characterizes a title as off limits (section 3.6).

use std::path::Path;

use fragcap_profile::signature::SignatureSet;
use fragcap_profile::{DetectionFinding, FidelityTier};

use crate::entry::TargetClassification;

/// The classifier's decision for one directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassifierVerdict {
    /// This directory is a game (or other classified target): emit one candidate
    /// and stop descending into its subtree.
    Hit {
        /// What the directory is.
        classification: TargetClassification,
        /// The fidelity the match earns: [`FidelityTier::Verified`] for a definitive
        /// local engine marker, otherwise a weaker tier.
        fidelity: FidelityTier,
        /// The technologies detected in the directory, carried as neutral evidence
        /// (the detected engine plus any anti-cheat or DRM). Empty for a classifier
        /// that carries no signatures.
        evidence: Vec<DetectionFinding>,
    },
    /// Not a classified target: count it considered-not-a-game.
    Miss,
}

/// A classification result: the verdict plus any subtree paths the signature scan
/// could not read. Surfacing the latter keeps reduced detection coverage visible
/// rather than presenting a partial scan as complete (P-4); the walk folds them into
/// [`crate::source::Discovery::warnings`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifierResult {
    /// Whether the directory is a target.
    pub verdict: ClassifierVerdict,
    /// Subtree paths that could not be read during classification.
    pub unreadable: Vec<String>,
}

impl ClassifierResult {
    /// A classification with a verdict and no unreadable paths.
    fn just(verdict: ClassifierVerdict) -> Self {
        ClassifierResult {
            verdict,
            unreadable: Vec::new(),
        }
    }
}

/// Decides, from a directory's shape, whether it is a target. A seam so S053's
/// signature matcher drops in without touching the walk.
pub trait DirectoryClassifier {
    /// Classify one directory by its path and its shape, reporting any subtree it
    /// could not read alongside the verdict.
    fn classify(&self, dir: &str) -> ClassifierResult;
}

/// The S052 placeholder classifier: every immediate subdirectory of a known root is
/// a game, because a known root is a directory that only ever contains games
/// (FR-007). It carries no signature logic and stamps `heuristic-unverified` with no
/// evidence. Retained for the known-roots-only path where no catalog is available;
/// [`SignatureClassifier`] supersedes it when a signature set is loaded.
pub struct KnownRootChildIsGame;

impl DirectoryClassifier for KnownRootChildIsGame {
    fn classify(&self, _dir: &str) -> ClassifierResult {
        ClassifierResult::just(ClassifierVerdict::Hit {
            classification: TargetClassification::Game,
            fidelity: FidelityTier::HeuristicUnverified,
            evidence: Vec::new(),
        })
    }
}

/// The S053 production classifier: it scans a directory's bounded subtree against
/// the catalog's detection signatures and reports what it found.
///
/// It has two modes, because tiers 2 and 3 rest on different priors:
///
/// - **Known-root mode** ([`SignatureClassifier::for_known_root`]): a child of a
///   known game-only root is a game by structure (S052 FR-007), so every child is a
///   `Hit`. Detection only enriches it: a detected engine raises the fidelity to
///   what its signature earns (a definitive marker is [`FidelityTier::Verified`],
///   P-9) and attaches the engine plus any anti-cheat or DRM as neutral evidence; a
///   child with no detected engine is still a game, at `heuristic-unverified`.
/// - **Pointed mode** ([`SignatureClassifier::new`]): a directory with no structural
///   prior (a folder a user pointed at, tier 3) is a game only if an engine
///   signature confirms it; otherwise it is a `Miss`.
///
/// It reads the real filesystem at the directory it classifies, so it is exercised
/// against real (temporary) directory trees, the same posture as the matcher's own
/// tests. In pointed mode an unreadable directory is a `Miss`; in known-root mode it
/// stays a game (the structural prior holds) at `heuristic-unverified` with no
/// evidence, and the walk's lister accounts any access error separately.
pub struct SignatureClassifier {
    signatures: SignatureSet,
    assume_game: bool,
}

impl SignatureClassifier {
    /// A pointed-mode classifier: a directory is a game only if an engine signature
    /// matches (tier 3, no structural prior).
    pub fn new(signatures: SignatureSet) -> Self {
        SignatureClassifier {
            signatures,
            assume_game: false,
        }
    }

    /// A known-root-mode classifier: every directory is a game (the structural prior
    /// of a game-only root), enriched by detection (tier 2).
    pub fn for_known_root(signatures: SignatureSet) -> Self {
        SignatureClassifier {
            signatures,
            assume_game: true,
        }
    }
}

impl DirectoryClassifier for SignatureClassifier {
    fn classify(&self, dir: &str) -> ClassifierResult {
        let outcome = match self.signatures.detect(Path::new(dir)) {
            Ok(outcome) => outcome,
            // The root itself is unreadable. In known-root mode the structural prior
            // still holds (it is a game we could not scan); in pointed mode it cannot
            // be confirmed a game. Either way the unreadable root is surfaced.
            Err(e) => {
                let verdict = if self.assume_game {
                    ClassifierVerdict::Hit {
                        classification: TargetClassification::Game,
                        fidelity: FidelityTier::HeuristicUnverified,
                        evidence: Vec::new(),
                    }
                } else {
                    ClassifierVerdict::Miss
                };
                return ClassifierResult {
                    verdict,
                    unreadable: vec![e.path.to_string_lossy().into_owned()],
                };
            }
        };
        // A subtree the scan could not read reduces coverage; carry it so a partial
        // scan is never presented as complete (P-4).
        let unreadable: Vec<String> = outcome
            .unreadable
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        let verdict = match outcome.detected_engine() {
            // A detected engine: a game at the fidelity the signature earns, carrying
            // every finding (engine, anti-cheat, DRM) as neutral evidence.
            Some(engine) => ClassifierVerdict::Hit {
                classification: TargetClassification::Game,
                fidelity: engine.fidelity,
                evidence: outcome.findings,
            },
            // No engine signature. In known-root mode the child is still a game, at
            // heuristic-unverified, carrying any anti-cheat or DRM as evidence. In
            // pointed mode a directory with no engine is not confirmed a game.
            None if self.assume_game => ClassifierVerdict::Hit {
                classification: TargetClassification::Game,
                fidelity: FidelityTier::HeuristicUnverified,
                evidence: outcome.findings,
            },
            None => ClassifierVerdict::Miss,
        };
        ClassifierResult {
            verdict,
            unreadable,
        }
    }
}

/// A classifier that reports [`ClassifierVerdict::Hit`] only for a fixed set of
/// directory paths, [`ClassifierVerdict::Miss`] for the rest. Lets a test drive the
/// stop-on-hit descent deterministically without a real signature matcher. It
/// carries no evidence and stamps `heuristic-unverified`.
pub struct FixtureClassifier {
    hits: Vec<String>,
}

impl FixtureClassifier {
    /// Build a classifier that hits exactly the given directory paths.
    pub fn new(hits: Vec<String>) -> Self {
        FixtureClassifier { hits }
    }
}

impl DirectoryClassifier for FixtureClassifier {
    fn classify(&self, dir: &str) -> ClassifierResult {
        ClassifierResult::just(if self.hits.iter().any(|h| h == dir) {
            ClassifierVerdict::Hit {
                classification: TargetClassification::Game,
                fidelity: FidelityTier::HeuristicUnverified,
                evidence: Vec::new(),
            }
        } else {
            ClassifierVerdict::Miss
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragcap_profile::{Signature, SignatureCategory, SignatureConfidence, SignatureKind};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new(tag: &str) -> TempTree {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "fragcap-classifier-{}-{}-{}",
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

        fn path_str(&self) -> String {
            self.root.to_string_lossy().into_owned()
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn engine_set() -> SignatureSet {
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

    #[test]
    fn an_engine_marker_is_a_verified_game_hit() {
        let tree = TempTree::new("unity");
        tree.touch("UnityPlayer.dll");
        tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll");
        let classifier = SignatureClassifier::new(engine_set());
        match classifier.classify(&tree.path_str()).verdict {
            ClassifierVerdict::Hit {
                classification,
                fidelity,
                evidence,
            } => {
                assert_eq!(classification, TargetClassification::Game);
                assert_eq!(fidelity, FidelityTier::Verified);
                // The engine and the anti-cheat both ride as neutral evidence.
                assert!(evidence.iter().any(|f| f.product == "Unity"));
                assert!(evidence.iter().any(|f| f.product == "Easy Anti-Cheat"));
            }
            ClassifierVerdict::Miss => panic!("expected a hit"),
        }
    }

    #[test]
    fn a_directory_with_no_engine_is_a_miss_in_pointed_mode() {
        let tree = TempTree::new("no-engine");
        tree.touch("readme.txt");
        tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll"); // anti-cheat alone: not a game
        let classifier = SignatureClassifier::new(engine_set());
        assert_eq!(
            classifier.classify(&tree.path_str()).verdict,
            ClassifierVerdict::Miss
        );
    }

    #[test]
    fn a_known_root_child_with_no_engine_is_still_a_heuristic_game() {
        // Known-root mode keeps the structural prior: a child of a game-only root is
        // a game even with no detectable engine, at heuristic-unverified, carrying any
        // anti-cheat evidence found.
        let tree = TempTree::new("known-root-no-engine");
        tree.touch("readme.txt");
        tree.touch("EasyAntiCheat/EasyAntiCheat_x64.dll");
        let classifier = SignatureClassifier::for_known_root(engine_set());
        match classifier.classify(&tree.path_str()).verdict {
            ClassifierVerdict::Hit {
                classification,
                fidelity,
                evidence,
            } => {
                assert_eq!(classification, TargetClassification::Game);
                assert_eq!(fidelity, FidelityTier::HeuristicUnverified);
                assert!(evidence.iter().any(|f| f.product == "Easy Anti-Cheat"));
            }
            ClassifierVerdict::Miss => panic!("a known-root child is still a game"),
        }
    }

    #[test]
    fn a_known_root_child_with_an_engine_is_a_verified_game() {
        let tree = TempTree::new("known-root-unity");
        tree.touch("UnityPlayer.dll");
        let classifier = SignatureClassifier::for_known_root(engine_set());
        match classifier.classify(&tree.path_str()).verdict {
            ClassifierVerdict::Hit { fidelity, .. } => {
                assert_eq!(fidelity, FidelityTier::Verified);
            }
            ClassifierVerdict::Miss => panic!("expected a verified hit"),
        }
    }

    #[test]
    fn an_unreadable_directory_is_a_miss_and_is_surfaced() {
        let missing =
            std::env::temp_dir().join(format!("fragcap-classifier-absent-{}", std::process::id()));
        let classifier = SignatureClassifier::new(engine_set());
        let c = classifier.classify(&missing.to_string_lossy());
        assert_eq!(c.verdict, ClassifierVerdict::Miss);
        // The unreadable root is surfaced rather than silently swallowed (P-4).
        assert!(!c.unreadable.is_empty(), "an unreadable root is reported");
    }
}
