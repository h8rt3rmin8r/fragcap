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
//! The descent contract (S052 FR-015, corrected by S077): a walk tests each
//! directory through a classifier and stops descending on a
//! [`ClassifierVerdict::Hit`], emitting one candidate. A
//! [`ClassifierVerdict::Container`] emits nothing and requests bounded descent. The
//! walk never enumerates a directory's executables first and then asks whether each
//! is a game (FR-009).
//!
//! A [`ClassifierVerdict::Hit`] carries the fidelity the match earns (a definitive
//! local engine marker is [`FidelityTier::Verified`], which outranks a remote
//! catalog attribution, P-9) and the neutral evidence detected alongside it (any
//! anti-cheat or DRM). The evidence is a set of facts and nothing more: no field on
//! it characterizes a title as off limits (section 3.6).

use std::collections::HashSet;
use std::path::Path;

use fragcap_profile::signature::{SignatureCategory, SignatureSet};
use fragcap_profile::{DetectionFinding, FidelityTier};

use crate::entry::{DetectionScan, TargetClassification};

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
        /// Whether the directory was scanned and whether that scan was complete
        /// (slice S065). `None` for a classifier that carries no signatures and so
        /// ran no scan.
        detection_scan: Option<DetectionScan>,
    },
    /// This directory aggregates more than one engine product and is therefore a
    /// container to descend through, not one target to emit.
    Container,
    /// Not a classified target: count it considered-not-a-game.
    Miss,
}

/// A classification result: the verdict plus a named diagnostic for everything the
/// signature scan did not cover. Surfacing the latter keeps reduced detection
/// coverage visible rather than presenting a partial scan as complete (P-4); the walk
/// folds them into [`crate::source::Discovery::warnings`].
///
/// The field carries finished warning lines rather than unreadable paths. It was the
/// latter, and that shape could only ever express one cause: when a scan bound began
/// truncating the candidate set, the classifier seam had nowhere to put the count and
/// dropped it, so a known-root child could be classified with reduced coverage and no
/// stated reason. A carrier that names one cause silently excludes the rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifierResult {
    /// Whether the directory is a target.
    pub verdict: ClassifierVerdict,
    /// Named diagnostics for what classification could not cover: an unreadable
    /// subtree, an unreadable root, or candidates a scan bound truncated.
    pub coverage_warnings: Vec<String>,
}

impl ClassifierResult {
    /// A classification with a verdict and complete coverage.
    fn just(verdict: ClassifierVerdict) -> Self {
        ClassifierResult {
            verdict,
            coverage_warnings: Vec::new(),
        }
    }
}

/// Decides, from a directory's shape, whether it is a target or a container that
/// should be descended. A seam so the signature matcher and walk remain separate.
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
        // This classifier carries no signatures, so it ran no scan and records no
        // coverage claim: `None`, never `Complete`.
        ClassifierResult::just(ClassifierVerdict::Hit {
            classification: TargetClassification::Game,
            fidelity: FidelityTier::HeuristicUnverified,
            evidence: Vec::new(),
            detection_scan: None,
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
                        // A scan was attempted and covered nothing. That is
                        // `Incomplete`, not `None`: an attempt that failed is a
                        // different fact from no attempt, and reporting it as no
                        // attempt would lose the failure (P-4).
                        detection_scan: Some(DetectionScan::Incomplete),
                    }
                } else {
                    ClassifierVerdict::Miss
                };
                return ClassifierResult {
                    verdict,
                    coverage_warnings: vec![format!(
                        "could not read install directory during detection: {}",
                        e.path.display()
                    )],
                };
            }
        };
        // Everything the scan did not cover, carried so a partial scan is never
        // presented as complete (P-4). Taken from the outcome's own helper, so a new
        // cause is forwarded without touching this seam.
        let coverage_warnings = outcome.coverage_warnings();
        // The detector already deduplicates per category and canonical product, but
        // count explicitly here so this control decision remains correct if a second
        // finding source is composed without that implementation detail later.
        let engine_products: HashSet<&str> = outcome
            .findings
            .iter()
            .filter(|finding| finding.category == SignatureCategory::Engine)
            .map(|finding| finding.product.as_str())
            .collect();
        let verdict = match outcome.detected_engine() {
            Some(_) if engine_products.len() > 1 => ClassifierVerdict::Container,
            // A detected engine: a game at the fidelity the signature earns, carrying
            // every finding (engine, anti-cheat, DRM) as neutral evidence.
            Some(engine) => ClassifierVerdict::Hit {
                classification: TargetClassification::Game,
                fidelity: engine.fidelity,
                detection_scan: Some(DetectionScan::from_outcome(&outcome)),
                evidence: outcome.findings,
            },
            // No engine signature. In known-root mode the child is still a game, at
            // heuristic-unverified, carrying any anti-cheat or DRM as evidence. In
            // pointed mode a directory with no engine is not confirmed a game.
            None if self.assume_game => ClassifierVerdict::Hit {
                classification: TargetClassification::Game,
                fidelity: FidelityTier::HeuristicUnverified,
                detection_scan: Some(DetectionScan::from_outcome(&outcome)),
                evidence: outcome.findings,
            },
            None => ClassifierVerdict::Miss,
        };
        ClassifierResult {
            verdict,
            coverage_warnings,
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
                // A fixture classifier runs no scan, so it makes no coverage claim.
                detection_scan: None,
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

    fn multi_engine_set() -> SignatureSet {
        SignatureSet::compile(&[
            Signature {
                category: SignatureCategory::Engine,
                kind: SignatureKind::Filename,
                pattern: "UnityPlayer.dll".to_string(),
                product: "Engine Alpha".to_string(),
                confidence: SignatureConfidence::Definitive,
            },
            Signature {
                category: SignatureCategory::Engine,
                kind: SignatureKind::Filename,
                pattern: "GameAssembly.dll".to_string(),
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
    fn distinct_engine_products_identify_a_container() {
        let tree = TempTree::new("multi-engine-container");
        tree.touch("child-a/UnityPlayer.dll");
        tree.touch("child-b/EngineBeta.dll");
        let classifier = SignatureClassifier::for_known_root(multi_engine_set());

        assert_eq!(
            classifier.classify(&tree.path_str()).verdict,
            ClassifierVerdict::Container
        );
    }

    #[test]
    fn repeated_markers_for_one_engine_remain_a_title_hit() {
        let tree = TempTree::new("same-engine-markers");
        tree.touch("UnityPlayer.dll");
        tree.touch("GameAssembly.dll");
        let classifier = SignatureClassifier::for_known_root(multi_engine_set());

        assert!(matches!(
            classifier.classify(&tree.path_str()).verdict,
            ClassifierVerdict::Hit { .. }
        ));
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
                ..
            } => {
                assert_eq!(classification, TargetClassification::Game);
                assert_eq!(fidelity, FidelityTier::Verified);
                // The engine and the anti-cheat both ride as neutral evidence.
                assert!(evidence.iter().any(|f| f.product == "Unity"));
                assert!(evidence.iter().any(|f| f.product == "Easy Anti-Cheat"));
            }
            other => panic!("expected a hit, got {other:?}"),
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
                ..
            } => {
                assert_eq!(classification, TargetClassification::Game);
                assert_eq!(fidelity, FidelityTier::HeuristicUnverified);
                assert!(evidence.iter().any(|f| f.product == "Easy Anti-Cheat"));
            }
            other => panic!("a known-root child is still a game, got {other:?}"),
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
            other => panic!("expected a verified hit, got {other:?}"),
        }
    }

    #[test]
    fn a_truncated_candidate_set_reaches_the_walk_as_a_named_warning() {
        // The seam used to carry unreadable paths only, so when a scan bound began
        // truncating the candidate set the classifier had nowhere to put the count
        // and dropped it: a known-root child could be classified with reduced
        // coverage and no stated reason (P-4).
        use fragcap_profile::signature::MARKER_SCAN_MAX_CANDIDATES;

        let tree = TempTree::new("truncated");
        let pe = fragcap_profile::pe::fixtures::minimal_pe_with_sections(&[".text"]);
        let extra = 2;
        for i in 0..(MARKER_SCAN_MAX_CANDIDATES + extra) {
            fs::write(tree.root.join(format!("game-{i:04}.exe")), &pe).expect("write exe");
        }

        let set = SignatureSet::compile(&[Signature {
            category: SignatureCategory::Drm,
            kind: SignatureKind::BinaryMarker,
            pattern: "section:.bind".to_string(),
            product: "Steam DRM".to_string(),
            confidence: SignatureConfidence::Definitive,
        }]);
        let c = SignatureClassifier::for_known_root(set).classify(&tree.path_str());

        let named = c
            .coverage_warnings
            .iter()
            .find(|w| w.contains("binary marker"))
            .unwrap_or_else(|| panic!("the truncation reaches the walk: {c:?}"));
        assert!(
            named.contains(&format!("{extra} more were not examined")),
            "and says how many were dropped: {named}"
        );
        // The verdict still stands: reduced coverage is reported, not fatal.
        assert!(matches!(c.verdict, ClassifierVerdict::Hit { .. }));
    }

    #[test]
    fn an_unreadable_directory_is_a_miss_and_is_surfaced() {
        let missing =
            std::env::temp_dir().join(format!("fragcap-classifier-absent-{}", std::process::id()));
        let classifier = SignatureClassifier::new(engine_set());
        let c = classifier.classify(&missing.to_string_lossy());
        assert_eq!(c.verdict, ClassifierVerdict::Miss);
        // The unreadable root is surfaced rather than silently swallowed (P-4).
        assert!(
            !c.coverage_warnings.is_empty(),
            "an unreadable root is reported"
        );
    }
}
