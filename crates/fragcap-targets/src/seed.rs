// SPDX-License-Identifier: Apache-2.0

//! The Tier 1 catalog seeder.
//!
//! Reads catalog entries from a [`CatalogSource`], applies the [`CorpusGate`],
//! merges admitted titles into the store's Tier 1 columns, and returns a summary
//! whose four counts reconcile to the number of titles fetched. The seeder resumes
//! from the catalog tier's recorded cursor, records progress after each page, and
//! never prunes: a stored title absent from a run is left as it is.

use crate::catalog::CatalogSource;
use crate::gate::CorpusGate;
use crate::model::{SeedState, SeedTier};
use crate::store::Store;
use crate::TargetsError;

/// The truthful account of one seed run. Every fetched title lands in exactly one
/// of these, so a truncated corpus can never read as complete (P-4, P-9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeedSummary {
    /// Titles the source yielded (parsed entries plus items it could not parse).
    pub fetched: u64,
    /// Titles admitted by the gate and merged into the store.
    pub written: u64,
    /// Titles the gate excluded (not a game, or below the review threshold, or no
    /// known review count).
    pub excluded: u64,
    /// Titles the source could not parse. Counted, never silently dropped; a bad
    /// title does not abort the run.
    pub failed: u64,
}

impl SeedSummary {
    /// The conservation identity every run satisfies.
    pub fn is_conserved(&self) -> bool {
        self.fetched == self.written + self.excluded + self.failed
    }
}

/// Seed the store's Tier 1 columns from a catalog source.
///
/// `now` is the run timestamp recorded in the seed state (the caller supplies it,
/// keeping the seeder free of ambient time and deterministic in tests). A store
/// error aborts the run and is returned; a per-title parse failure is counted in
/// [`SeedSummary::failed`] and the run continues.
pub fn seed_catalog(
    store: &mut Store,
    source: &dyn CatalogSource,
    gate: &CorpusGate,
    now: Option<String>,
) -> Result<SeedSummary, TargetsError> {
    let mut summary = SeedSummary::default();

    // Resume from the recorded cursor, if any.
    let mut cursor: Option<String> = store
        .seed_state(SeedTier::Catalog)?
        .and_then(|s| s.resume_cursor);

    loop {
        let batch = source.fetch_batch(cursor.as_deref())?;

        // Items the source could not parse were still fetched.
        summary.fetched += batch.failed;
        summary.failed += batch.failed;

        for entry in &batch.entries {
            summary.fetched += 1;
            if gate.admits(entry) {
                store.merge_catalog(
                    entry.appid,
                    entry.name.as_deref(),
                    entry.review_count,
                    entry.owners,
                    entry.peak_ccu,
                )?;
                summary.written += 1;
            } else {
                summary.excluded += 1;
            }
        }

        // Record progress after each page so a later run resumes here.
        cursor = batch.next_cursor.clone();
        store.set_seed_state(&SeedState {
            tier: SeedTier::Catalog,
            last_run_at: now.clone(),
            resume_cursor: cursor.clone(),
        })?;

        if batch.next_cursor.is_none() {
            break;
        }
    }

    Ok(summary)
}
