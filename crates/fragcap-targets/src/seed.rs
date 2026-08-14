// SPDX-License-Identifier: Apache-2.0

//! The Tier 1 catalog seeder.
//!
//! Reads catalog entries from a [`CatalogSource`], applies the [`CorpusGate`],
//! merges admitted titles into the store's Tier 1 columns, and returns a summary
//! whose four counts reconcile to the number of titles fetched. The seeder resumes
//! from the catalog tier's recorded cursor, records progress after each page, and
//! never prunes: a stored title absent from a run is left as it is.

use std::collections::HashSet;

use crate::catalog::CatalogSource;
use crate::engine_feed::EngineFeed;
use crate::gate::CorpusGate;
use crate::model::{Engine, EngineSource, SeedState, SeedTier};
use crate::store::Store;
use crate::TargetsError;

/// The truthful account of one seed run. Every fetched title lands in exactly one
/// of these, so a truncated corpus can never read as complete (P-4, P-9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeedSummary {
    /// Titles the source yielded (parsed entries plus items it could not parse).
    pub fetched: u64,
    /// Distinct titles admitted by the gate and merged into the store. A repeated
    /// appid is counted here once, not once per occurrence.
    pub written: u64,
    /// Titles the gate excluded (not a game, or below the review threshold, or no
    /// known review count).
    pub excluded: u64,
    /// Admitted titles whose appid had already been written earlier in this run: a
    /// repeated entry, merged idempotently but not double-counted as written.
    pub duplicates: u64,
    /// Titles the source could not parse. Counted, never silently dropped; a bad
    /// title does not abort the run.
    pub failed: u64,
}

impl SeedSummary {
    /// The conservation identity every run satisfies: every fetched title is
    /// written, excluded, a within-run duplicate, or failed.
    pub fn is_conserved(&self) -> bool {
        self.fetched == self.written + self.excluded + self.duplicates + self.failed
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
    // Appids written in this run, so a repeated appid is merged idempotently but
    // counted once (the summary must not overstate the corpus).
    let mut written_appids: HashSet<u32> = HashSet::new();

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
            if !gate.admits(entry) {
                summary.excluded += 1;
                continue;
            }
            store.merge_catalog(
                entry.appid,
                entry.name.as_deref(),
                entry.review_count,
                entry.owners,
                entry.peak_ccu,
            )?;
            // The merge is idempotent; count a repeated appid once.
            if written_appids.insert(entry.appid) {
                summary.written += 1;
            } else {
                summary.duplicates += 1;
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

/// Seed the store's Tier 3 (engine attribution) columns from an engine source.
///
/// Reuses [`SeedSummary`] and its conservation identity. `now` is the run timestamp
/// recorded in the engine tier's seed state (the caller supplies it, keeping the
/// seeder free of ambient time and deterministic in tests). A store error aborts the
/// run and is returned; a per-title parse failure is counted in
/// [`SeedSummary::failed`] and the run continues.
///
/// A title the source resolves a single unambiguous engine for is written
/// (`engine_source = "pcgamingwiki"`); a title with no engine or an ambiguous engine
/// is left absent and counted excluded, never guessed (P-9). There is no corpus gate:
/// unlike Tier 1, the engine tier enriches whatever titles the source names an engine
/// for. The seeder never prunes.
pub fn seed_engine(
    store: &mut Store,
    source: &dyn EngineFeed,
    now: Option<String>,
) -> Result<SeedSummary, TargetsError> {
    let mut summary = SeedSummary::default();
    // Appids written in this run, so a repeated appid is merged exactly once (the
    // first-seen resolved engine wins) and counted once (the summary must not
    // overstate the enrichment).
    let mut written_appids: HashSet<u32> = HashSet::new();

    // Resume from the recorded engine-tier cursor, if any.
    let mut cursor: Option<String> = store
        .seed_state(SeedTier::Engine)?
        .and_then(|s| s.resume_cursor);

    loop {
        let batch = source.fetch_batch(cursor.as_deref())?;

        // Items the source could not parse were still fetched.
        summary.fetched += batch.failed;
        summary.failed += batch.failed;

        for entry in &batch.entries {
            summary.fetched += 1;
            let Some(resolved) = &entry.engine else {
                // No engine, or an ambiguous one: left absent, not guessed.
                summary.excluded += 1;
                continue;
            };
            // A repeated appid is merged once: the first resolved engine for an
            // appid in this run is authoritative, and a later duplicate is counted
            // but not re-merged. Merging the duplicate would make the stored
            // attribution depend on source order when two entries disagree.
            if !written_appids.insert(entry.appid) {
                summary.duplicates += 1;
                continue;
            }
            let engine = Engine {
                name: Some(resolved.name.clone()),
                source: EngineSource::Pcgamingwiki,
                confidence: resolved.confidence,
            };
            store.merge_engine(entry.appid, &engine)?;
            summary.written += 1;
        }

        // Record progress after each page so a later run resumes here.
        cursor = batch.next_cursor.clone();
        store.set_seed_state(&SeedState {
            tier: SeedTier::Engine,
            last_run_at: now.clone(),
            resume_cursor: cursor.clone(),
        })?;

        if batch.next_cursor.is_none() {
            break;
        }
    }

    Ok(summary)
}
