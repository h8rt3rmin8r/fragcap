// SPDX-License-Identifier: Apache-2.0

//! The engine source the Tier 3 seeder reads from.
//!
//! [`EngineFeed`] is the seam that keeps the engine seeder testable offline: every
//! test drives [`FixtureEngineFeed`] over committed data, and the live
//! [`crate::http_engine::HttpEngineFeed`] (behind the `net` feature) is the thin
//! wire adapter that CI compiles but never runs, the same posture as the S035
//! catalog source and live packet capture. The trait fixes the shape the seeder
//! consumes, so the seeder cannot come to depend on a live-only detail a fixture
//! cannot express.
//!
//! The trait is named [`EngineFeed`], not `EngineSource`, because
//! [`crate::model::EngineSource`] is already the schema `engine.source` token enum;
//! the trait names the paged source the seeder reads from, the enum names
//! provenance, and the distinct names keep both reachable without ambiguity.

use serde_json::Value;

use crate::model::EngineConfidence;
use crate::TargetsError;

/// The confidence a live PCGamingWiki lookup assigns a cleanly resolved single
/// engine, and the default a fixture entry takes when it omits `confidence`. A
/// within-field grade of a well-attested but binary-unverified community field
/// (P-9); the row stays heuristic-unverified regardless. Documented and tunable,
/// not a load-bearing correctness value.
pub const DEFAULT_ENGINE_CONFIDENCE: EngineConfidence = EngineConfidence::High;

/// The resolved engine a source attributes to a title: a single engine name and a
/// within-field confidence grade. Present only when the source settled on one
/// unambiguous engine; a missing or ambiguous engine is [`None`] on the entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEngine {
    /// The single resolved engine name. Non-empty by construction.
    pub name: String,
    /// The within-field confidence grade (never a fidelity tier, P-9).
    pub confidence: EngineConfidence,
}

/// One title as an engine source presents it, before the seeder's keep-or-exclude
/// decision. `engine` is `None` when the source names no engine, or an ambiguous
/// one; such a title is excluded (left absent, never guessed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineEntry {
    pub appid: u32,
    pub engine: Option<ResolvedEngine>,
}

/// A page of engine entries, a count of items the source could not parse in this
/// page, and the cursor to resume after them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineBatch {
    pub entries: Vec<EngineEntry>,
    /// Items in this page the source fetched but could not parse. Counted as failed
    /// by the seeder, never silently dropped (P-4); a single bad item does not abort
    /// the run.
    pub failed: u64,
    /// The cursor a resumed run passes back to [`EngineFeed::fetch_batch`]. `None`
    /// means the source is exhausted.
    pub next_cursor: Option<String>,
}

/// A source of engine attributions, paged by an opaque cursor.
pub trait EngineFeed {
    /// Yield the next page starting after `cursor` (`None` = from the beginning).
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<EngineBatch, TargetsError>;
}

/// The default page size a [`FixtureEngineFeed`] returns.
const DEFAULT_BATCH: usize = 100;

/// An offline engine source backed by a committed JSON document.
///
/// The document is an array of entry objects, each with a required numeric `appid`,
/// an optional `engine` (a string, or an array of strings), and an optional string
/// `confidence`. The engine field resolves to a single name (a string or a
/// one-element array), to no engine (absent, null, empty, or an empty array), or to
/// ambiguous (more than one distinct engine); an out-of-set confidence or a
/// wrong-typed field makes the entry malformed. Entries are held sorted by appid
/// (stably, so duplicate appids keep document order) and paged by a consumed-count
/// offset cursor, so the cursor contract (and resumability) is exercised offline
/// without an appid-keyed cursor collapsing duplicate appids.
pub struct FixtureEngineFeed {
    entries: Vec<EngineEntry>,
    /// Items the document could not parse. Tolerated at load and surfaced to the
    /// seeder as failed on the first page, so a bad entry is counted rather than
    /// aborting the load.
    malformed: u64,
    batch_size: usize,
}

impl FixtureEngineFeed {
    /// Parse an engine document with the default page size.
    pub fn from_json(text: &str) -> Result<Self, TargetsError> {
        Self::from_json_with_batch(text, DEFAULT_BATCH)
    }

    /// Parse an engine document with an explicit page size (tests use a small one to
    /// exercise multi-page resumption).
    pub fn from_json_with_batch(text: &str, batch_size: usize) -> Result<Self, TargetsError> {
        let batch_size = batch_size.max(1);
        let value: Value = serde_json::from_str(text)
            .map_err(|e| TargetsError::Seed(format!("engine document is not valid JSON: {e}")))?;
        let array = value.as_array().ok_or_else(|| {
            TargetsError::Seed("engine document must be a JSON array".to_string())
        })?;

        // A structural error (not an array) is a hard failure; a single unparsable
        // entry within a valid array is tolerated and counted, so the seeder can
        // report it as failed rather than the whole load aborting.
        let mut entries = Vec::with_capacity(array.len());
        let mut malformed = 0u64;
        for (i, item) in array.iter().enumerate() {
            match parse_entry(item, i) {
                Ok(entry) => entries.push(entry),
                Err(_) => malformed += 1,
            }
        }
        entries.sort_by_key(|e| e.appid);
        Ok(FixtureEngineFeed {
            entries,
            malformed,
            batch_size,
        })
    }
}

impl EngineFeed for FixtureEngineFeed {
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<EngineBatch, TargetsError> {
        // The cursor is the number of entries already consumed (an offset), not an
        // appid. An appid-keyed cursor (`appid > last`) would silently skip a
        // duplicate appid that straddles a page boundary, dropping it from the
        // fetched total while conservation still appeared to hold against the reduced
        // total (P-4). An offset preserves every occurrence and matches the live
        // source's offset pagination.
        let offset: usize = match cursor {
            None => 0,
            Some(c) => c
                .parse()
                .map_err(|_| TargetsError::Seed(format!("bad engine cursor: {c:?}")))?,
        };

        let entries: Vec<EngineEntry> = self
            .entries
            .iter()
            .skip(offset)
            .take(self.batch_size)
            .cloned()
            .collect();

        // A full page may have more behind it; a short page is the end.
        let next_cursor = if entries.len() == self.batch_size {
            Some((offset + entries.len()).to_string())
        } else {
            None
        };

        // Surface the load-time malformed count once, on the initial page, so a fresh
        // seed counts it and a resumed seed (which skips the processed prefix) does
        // not re-count it.
        let failed = if offset == 0 { self.malformed } else { 0 };

        Ok(EngineBatch {
            entries,
            failed,
            next_cursor,
        })
    }
}

/// Parse one engine entry object.
pub fn parse_entry(item: &Value, index: usize) -> Result<EngineEntry, TargetsError> {
    let appid = match item.get("appid") {
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| {
                TargetsError::Seed(format!("engine entry {index} appid is not a u32"))
            })?,
        None => {
            return Err(TargetsError::Seed(format!(
                "engine entry {index} has no appid"
            )))
        }
    };

    // Parse confidence unconditionally: a present but wrong-typed or out-of-set
    // confidence is a malformed entry even when the engine is absent, so the summary
    // does not silently accept a bad grade (P-9, FR-013). Absent means "use the
    // default when an engine resolves".
    let confidence = match item.get("confidence") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(EngineConfidence::parse(s).map_err(|_| {
            TargetsError::Seed(format!(
                "engine entry {index} confidence is out of set: {s:?}"
            ))
        })?),
        Some(_) => {
            return Err(TargetsError::Seed(format!(
                "engine entry {index} confidence is not a string"
            )))
        }
    };

    // Resolve the engine field to a single name, no engine, or ambiguous. A wrong
    // JSON type (a number, an object, or an array carrying a non-string) is a
    // malformed entry.
    let resolved_name = resolve_engine(item.get("engine"), index)?;

    let engine = resolved_name.map(|name| ResolvedEngine {
        name,
        confidence: confidence.unwrap_or(DEFAULT_ENGINE_CONFIDENCE),
    });

    Ok(EngineEntry { appid, engine })
}

/// Resolve the `engine` field to a single non-empty engine name.
///
/// - absent / null / `""` / `[]` (or an array of only empty strings) -> `Ok(None)`
///   (no engine -> excluded).
/// - a single non-empty string, or a one-element array with one non-empty string
///   -> `Ok(Some(name))` (written).
/// - more than one distinct non-empty engine name -> `Ok(None)` (ambiguous ->
///   excluded).
/// - any other JSON type, or an array carrying a non-string element -> `Err`
///   (malformed -> failed).
fn resolve_engine(field: Option<&Value>, index: usize) -> Result<Option<String>, TargetsError> {
    let candidates: Vec<String> = match field {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::String(s)) => {
            if s.is_empty() {
                Vec::new()
            } else {
                vec![s.clone()]
            }
        }
        Some(Value::Array(items)) => {
            let mut names = Vec::with_capacity(items.len());
            for element in items {
                match element {
                    Value::String(s) => {
                        if !s.is_empty() {
                            names.push(s.clone());
                        }
                    }
                    _ => {
                        return Err(TargetsError::Seed(format!(
                            "engine entry {index} engine array carries a non-string element"
                        )))
                    }
                }
            }
            names
        }
        Some(_) => {
            return Err(TargetsError::Seed(format!(
                "engine entry {index} engine field is not a string or array of strings"
            )))
        }
    };

    // Distinct names decide single-vs-ambiguous: ["Unity", "Unity"] is one engine
    // named twice, not an ambiguity.
    let mut distinct: Vec<&String> = Vec::new();
    for name in &candidates {
        if !distinct.contains(&name) {
            distinct.push(name);
        }
    }

    match distinct.len() {
        0 => Ok(None),                      // no engine
        1 => Ok(Some(distinct[0].clone())), // resolved
        _ => Ok(None),                      // ambiguous
    }
}
