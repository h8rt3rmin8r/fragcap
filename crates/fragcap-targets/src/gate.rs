// SPDX-License-Identifier: Apache-2.0

//! The corpus gate.
//!
//! The Steam app-list universe is large and mostly noise; seeding all of it would
//! bury the useful hints. The gate scopes the corpus to titles that are games and
//! clear a popularity threshold. An entry whose popularity signal is missing is
//! excluded rather than admitted on a guess (P-9): the seeder records what it can
//! confirm, not what it hopes.

use crate::catalog::{CatalogEntry, Classification};

/// The default review-count threshold. A few hundred, matching the working figure
/// in the catalog research (issue #83). Not a load-bearing correctness value; the
/// operator tunes it.
pub const DEFAULT_MIN_REVIEWS: u64 = 500;

/// The rule deciding whether a catalog entry belongs in the corpus.
#[derive(Clone, Copy, Debug)]
pub struct CorpusGate {
    /// A title needs at least this many reviews to be admitted.
    pub min_reviews: u64,
}

impl CorpusGate {
    /// A gate at the given threshold.
    pub fn new(min_reviews: u64) -> Self {
        CorpusGate { min_reviews }
    }

    /// Whether the entry is in the corpus: a game with a known review count at or
    /// above the threshold.
    pub fn admits(&self, entry: &CatalogEntry) -> bool {
        entry.classification == Classification::Game
            && entry.review_count.is_some_and(|n| n >= self.min_reviews)
    }
}

impl Default for CorpusGate {
    fn default() -> Self {
        CorpusGate::new(DEFAULT_MIN_REVIEWS)
    }
}
