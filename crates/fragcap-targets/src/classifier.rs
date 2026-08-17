// SPDX-License-Identifier: Apache-2.0

//! The directory-shape classifier seam (slice S052), spec 7.3.
//!
//! Tiers 2 and 3 classify a directory by its shape, the presence of an engine
//! signature such as a Unity player library or an Unreal engine-binaries tree,
//! rather than by a curated per-title list, so a standalone or non-catalog title
//! is recognized as a game. This module ships the seam and the descent contract
//! the walk applies; the real signature matcher lands in S053 and implements
//! [`DirectoryClassifier`] with no change to the walk.
//!
//! The descent contract (FR-015): a walk tests each directory through a classifier
//! and stops descending on a [`ClassifierVerdict::Hit`], emitting one candidate;
//! it never enumerates a directory's executables first and then asks whether each
//! is a game (FR-009).

use crate::entry::TargetClassification;

/// The classifier's decision for one directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassifierVerdict {
    /// This directory is a game (or other classified target): emit one candidate
    /// and stop descending into its subtree.
    Hit {
        /// What the directory is.
        classification: TargetClassification,
    },
    /// Not a classified target: count it considered-not-a-game.
    Miss,
}

/// Decides, from a directory's shape, whether it is a target. A seam so S053's
/// signature matcher drops in without touching the walk.
pub trait DirectoryClassifier {
    /// Classify one directory by its path (and, in S053, its shape).
    fn classify(&self, dir: &str) -> ClassifierVerdict;
}

/// The S052 production classifier: every immediate subdirectory of a known root is
/// a game, because a known root is a directory that only ever contains games
/// (FR-007). It carries no signature logic; S053 replaces it with the shape
/// matcher that generalizes tiers 2 and 3 beyond curated roots.
pub struct KnownRootChildIsGame;

impl DirectoryClassifier for KnownRootChildIsGame {
    fn classify(&self, _dir: &str) -> ClassifierVerdict {
        ClassifierVerdict::Hit {
            classification: TargetClassification::Game,
        }
    }
}

/// A classifier that reports [`ClassifierVerdict::Hit`] only for a fixed set of
/// directory paths, [`ClassifierVerdict::Miss`] for the rest. Lets a test drive the
/// stop-on-hit descent deterministically without a real signature matcher.
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
    fn classify(&self, dir: &str) -> ClassifierVerdict {
        if self.hits.iter().any(|h| h == dir) {
            ClassifierVerdict::Hit {
                classification: TargetClassification::Game,
            }
        } else {
            ClassifierVerdict::Miss
        }
    }
}
