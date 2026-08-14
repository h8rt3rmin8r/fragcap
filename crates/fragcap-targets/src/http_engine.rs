// SPDX-License-Identifier: Apache-2.0

//! The live engine source, behind the `net` feature.
//!
//! `HttpEngineFeed` performs read-only HTTPS GETs against PCGamingWiki's MediaWiki
//! Cargo query API and maps the responses into [`EngineEntry`] values. It is
//! passive: it opens no process handle and touches no capture path (P-1). It is
//! compiled under the `net` feature so the all-features gate covers it, but it is
//! never run by a test (the offline [`crate::engine_feed::FixtureEngineFeed`] drives
//! every test), the same posture as the S035 `HttpCatalog` and live packet capture.
//!
//! The default endpoint queries the `Infobox_game` Cargo table for the Steam
//! application id, the page name, and the engine field, paged by an offset cursor.
//! The precise query is an operator-facing detail, not a tested contract: any source
//! that produces [`EngineEntry`] values feeds the same seeder.

use http_req::request;
use serde_json::Value;

use crate::engine_feed::DEFAULT_ENGINE_CONFIDENCE;
use crate::engine_feed::{EngineBatch, EngineEntry, EngineFeed, ResolvedEngine};
use crate::TargetsError;

/// The default query base: PCGamingWiki's Cargo API over `Infobox_game`, selecting
/// the Steam application id, the page name, and the engine field. The offset and
/// limit are appended per page.
const DEFAULT_BASE_URL: &str = "https://www.pcgamingwiki.com/w/api.php?action=cargoquery&format=json&tables=Infobox_game&fields=Infobox_game.Steam_AppID=SteamID,Infobox_game._pageName=Page,Infobox_game.Engine=Engine";

/// The page size the live source requests.
const PAGE_LIMIT: u32 = 500;

/// A live engine source over HTTPS against PCGamingWiki.
pub struct HttpEngineFeed {
    base_url: String,
}

impl HttpEngineFeed {
    /// A source against the default PCGamingWiki endpoint.
    pub fn new() -> Self {
        HttpEngineFeed {
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// A source against an explicit base URL (the operator may point elsewhere).
    /// `&limit=N&offset=M` are appended per page.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        HttpEngineFeed {
            base_url: base_url.into(),
        }
    }

    fn get(&self, url: &str) -> Result<String, TargetsError> {
        let mut body: Vec<u8> = Vec::new();
        let response =
            request::get(url, &mut body).map_err(|e| TargetsError::Fetch(format!("{url}: {e}")))?;
        if !response.status_code().is_success() {
            return Err(TargetsError::Fetch(format!(
                "{url}: HTTP {}",
                response.status_code()
            )));
        }
        String::from_utf8(body)
            .map_err(|e| TargetsError::Fetch(format!("{url}: response was not UTF-8: {e}")))
    }
}

impl Default for HttpEngineFeed {
    fn default() -> Self {
        HttpEngineFeed::new()
    }
}

impl EngineFeed for HttpEngineFeed {
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<EngineBatch, TargetsError> {
        let offset: u32 = match cursor {
            None => 0,
            Some(c) => c
                .parse()
                .map_err(|_| TargetsError::Fetch(format!("bad offset cursor: {c:?}")))?,
        };

        let url = format!("{}&limit={PAGE_LIMIT}&offset={offset}", self.base_url);
        let text = self.get(&url)?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| TargetsError::Fetch(format!("{url}: response was not JSON: {e}")))?;
        let rows = value
            .get("cargoquery")
            .and_then(Value::as_array)
            .ok_or_else(|| TargetsError::Fetch(format!("{url}: expected a `cargoquery` array")))?;

        let mut entries = Vec::with_capacity(rows.len());
        let mut failed = 0u64;
        for row in rows {
            match parse_cargo_row(row) {
                Some(entry) => entries.push(entry),
                None => failed += 1,
            }
        }

        // A full page may have more behind it; a short page is the end.
        let next_cursor = if rows.len() as u32 == PAGE_LIMIT {
            Some((offset + PAGE_LIMIT).to_string())
        } else {
            None
        };

        Ok(EngineBatch {
            entries,
            failed,
            next_cursor,
        })
    }
}

/// Map one Cargo query row into an engine entry. Returns `None` for a row the seeder
/// should count as failed: no usable Steam application id, or a present but
/// wrong-typed `Engine` field (FR-013). An absent or null `Engine` is an honest "no
/// engine", not a failure.
fn parse_cargo_row(row: &Value) -> Option<EngineEntry> {
    let title = row.get("title")?;
    let appid = first_appid(title.get("SteamID").and_then(Value::as_str)?)?;
    // A present but non-string `Engine` field is malformed, not "no engine": count it
    // failed rather than coercing it to an absent engine and reporting the row
    // excluded, which would misstate why the engine was omitted (FR-013).
    let engine = match title.get("Engine") {
        None | Some(Value::Null) => None,
        Some(Value::String(field)) => resolve_engine_field(field),
        Some(_) => return None,
    };
    Some(EngineEntry {
        appid,
        engine: engine.map(|name| ResolvedEngine {
            name,
            confidence: DEFAULT_ENGINE_CONFIDENCE,
        }),
    })
}

/// Parse the first Steam application id from a field that may list several
/// comma-separated ids (a PCGamingWiki page can carry more than one).
fn first_appid(field: &str) -> Option<u32> {
    field
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())
}

/// Resolve PCGamingWiki's engine field to a single engine name. The field may be
/// blank (no engine), name one engine, or list several (ambiguous). Splits on commas
/// and semicolons, trims, and collapses duplicates; a single distinct name resolves,
/// zero or more than one yields `None`.
fn resolve_engine_field(field: &str) -> Option<String> {
    let mut distinct: Vec<String> = Vec::new();
    for part in field.split([',', ';']) {
        let name = part.trim();
        if !name.is_empty() && !distinct.iter().any(|n| n == name) {
            distinct.push(name.to_string());
        }
    }
    match distinct.len() {
        1 => Some(distinct.remove(0)),
        _ => None,
    }
}
