// SPDX-License-Identifier: Apache-2.0

//! The catalog source the seeder reads from.
//!
//! [`CatalogSource`] is the seam that keeps the seeder testable offline: every
//! test drives [`FixtureCatalog`] over committed data, and the live
//! [`crate::http_catalog::HttpCatalog`] (behind the `net` feature) is the thin
//! wire adapter that CI compiles but never runs, the same posture as live packet
//! capture. The trait fixes the shape the seeder consumes, so the seeder cannot
//! come to depend on a live-only detail a fixture cannot express.

use serde_json::Value;

use crate::TargetsError;

/// How a catalog classifies a title. The gate keeps only games.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classification {
    Game,
    Other,
}

/// One title as a catalog source presents it, before the corpus gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogEntry {
    pub appid: u32,
    /// Absent or empty maps to a stored NULL (the S034 guard); never `""`.
    pub name: Option<String>,
    pub classification: Classification,
    /// The gate's popularity signal. Absent means the gate cannot admit the entry.
    pub review_count: Option<u64>,
    pub owners: Option<u64>,
    pub peak_ccu: Option<u64>,
}

/// A page of catalog entries, a count of items the source could not parse in this
/// page, and the cursor to resume after them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogBatch {
    pub entries: Vec<CatalogEntry>,
    /// Items in this page the source fetched but could not parse. Counted as
    /// failed by the seeder, never silently dropped (P-4); a single bad item does
    /// not abort the run (FR-006).
    pub failed: u64,
    /// The cursor a resumed run passes back to [`CatalogSource::fetch_batch`].
    /// `None` means the source is exhausted.
    pub next_cursor: Option<String>,
}

/// A source of catalog entries, paged by an opaque cursor.
pub trait CatalogSource {
    /// Yield the next page starting after `cursor` (`None` = from the beginning).
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<CatalogBatch, TargetsError>;
}

/// The default page size a [`FixtureCatalog`] returns.
const DEFAULT_BATCH: usize = 100;

/// An offline catalog source backed by a committed JSON document.
///
/// The document is an array of entry objects, each with a required numeric
/// `appid`, an optional string `name`, a `classification` (`"game"` or anything
/// else, treated as other), and optional numeric `review_count`, `owners`, and
/// `peak_ccu`. Entries are held sorted by appid and paged by the last appid
/// returned, so the cursor contract (and resumability) is exercised offline.
pub struct FixtureCatalog {
    entries: Vec<CatalogEntry>,
    /// Items the document could not parse (missing appid, wrong types). Tolerated
    /// at load and surfaced to the seeder as failed on the first page, so a bad
    /// entry is counted rather than aborting the load (FR-006).
    malformed: u64,
    batch_size: usize,
}

impl FixtureCatalog {
    /// Parse a catalog document with the default page size.
    pub fn from_json(text: &str) -> Result<Self, TargetsError> {
        Self::from_json_with_batch(text, DEFAULT_BATCH)
    }

    /// Parse a catalog document with an explicit page size (tests use a small one
    /// to exercise multi-page resumption).
    pub fn from_json_with_batch(text: &str, batch_size: usize) -> Result<Self, TargetsError> {
        let batch_size = batch_size.max(1);
        let value: Value = serde_json::from_str(text)
            .map_err(|e| TargetsError::Seed(format!("catalog is not valid JSON: {e}")))?;
        let array = value
            .as_array()
            .ok_or_else(|| TargetsError::Seed("catalog must be a JSON array".to_string()))?;

        // A structural error (not an array) is a hard failure; a single
        // unparsable entry within a valid array is tolerated and counted, so the
        // seeder can report it as failed rather than the whole load aborting.
        let mut entries = Vec::with_capacity(array.len());
        let mut malformed = 0u64;
        for (i, item) in array.iter().enumerate() {
            match parse_entry(item, i) {
                Ok(entry) => entries.push(entry),
                Err(_) => malformed += 1,
            }
        }
        entries.sort_by_key(|e| e.appid);
        Ok(FixtureCatalog {
            entries,
            malformed,
            batch_size,
        })
    }
}

impl CatalogSource for FixtureCatalog {
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<CatalogBatch, TargetsError> {
        let after: Option<u32> = match cursor {
            None => None,
            Some(c) => Some(
                c.parse()
                    .map_err(|_| TargetsError::Seed(format!("bad catalog cursor: {c:?}")))?,
            ),
        };

        let entries: Vec<CatalogEntry> = self
            .entries
            .iter()
            .filter(|e| after.is_none_or(|a| e.appid > a))
            .take(self.batch_size)
            .cloned()
            .collect();

        // A full page may have more behind it; a short page is the end.
        let next_cursor = if entries.len() == self.batch_size {
            entries.last().map(|e| e.appid.to_string())
        } else {
            None
        };

        // Surface the load-time malformed count once, on the initial page, so a
        // fresh seed counts it and a resumed seed (which skips the processed
        // prefix) does not re-count it.
        let failed = if after.is_none() { self.malformed } else { 0 };

        Ok(CatalogBatch {
            entries,
            failed,
            next_cursor,
        })
    }
}

/// Parse one catalog entry object.
pub fn parse_entry(item: &Value, index: usize) -> Result<CatalogEntry, TargetsError> {
    let appid = item
        .get("appid")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| TargetsError::Seed(format!("catalog entry {index} has no u32 appid")))?;

    let name = item
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let classification = match item.get("classification").and_then(Value::as_str) {
        Some(s) if s.eq_ignore_ascii_case("game") => Classification::Game,
        _ => Classification::Other,
    };

    Ok(CatalogEntry {
        appid,
        name,
        classification,
        review_count: item.get("review_count").and_then(Value::as_u64),
        owners: item.get("owners").and_then(Value::as_u64),
        peak_ccu: item.get("peak_ccu").and_then(Value::as_u64),
    })
}
