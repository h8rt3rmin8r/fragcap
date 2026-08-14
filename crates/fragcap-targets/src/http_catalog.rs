// SPDX-License-Identifier: Apache-2.0

//! The live catalog source, behind the `net` feature.
//!
//! `HttpCatalog` performs read-only HTTPS GETs against a public catalog API and
//! maps the responses into [`CatalogEntry`] values. It is passive: it opens no
//! process handle and touches no capture path (P-1). It is compiled under the
//! `net` feature so the all-features gate covers it, but it is never run by a test
//! (the offline [`crate::catalog::FixtureCatalog`] drives every test), the same
//! posture as live packet capture.
//!
//! The default endpoint is SteamSpy's paginated `all` listing, which yields, per
//! page, a batch of titles with their names and review tallies (the gate's
//! popularity signal). The page number is the cursor. The precise endpoint is an
//! operator-facing detail, not a tested contract: any source that produces
//! `CatalogEntry` values feeds the same gate and merge.

use http_req::request;
use serde_json::Value;

use crate::catalog::{CatalogBatch, CatalogEntry, CatalogSource, Classification};
use crate::TargetsError;

/// The default catalog endpoint: SteamSpy's `all` listing, paged by `page`.
const DEFAULT_BASE_URL: &str = "https://steamspy.com/api.php?request=all";

/// A live catalog source over HTTPS.
pub struct HttpCatalog {
    base_url: String,
}

impl HttpCatalog {
    /// A source against the default public endpoint.
    pub fn new() -> Self {
        HttpCatalog {
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// A source against an explicit base URL (the operator may point elsewhere).
    /// The page number is appended as `&page=N`.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        HttpCatalog {
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

impl Default for HttpCatalog {
    fn default() -> Self {
        HttpCatalog::new()
    }
}

impl CatalogSource for HttpCatalog {
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<CatalogBatch, TargetsError> {
        let page: u32 = match cursor {
            None => 0,
            Some(c) => c
                .parse()
                .map_err(|_| TargetsError::Fetch(format!("bad page cursor: {c:?}")))?,
        };

        let url = format!("{}&page={page}", self.base_url);
        let text = self.get(&url)?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| TargetsError::Fetch(format!("{url}: response was not JSON: {e}")))?;
        let object = value.as_object().ok_or_else(|| {
            TargetsError::Fetch(format!("{url}: expected a JSON object keyed by appid"))
        })?;

        let mut entries = Vec::with_capacity(object.len());
        let mut failed = 0u64;
        for item in object.values() {
            match parse_steamspy_entry(item) {
                Some(entry) => entries.push(entry),
                None => failed += 1,
            }
        }

        // An empty page is the end of the listing.
        let next_cursor = if entries.is_empty() {
            None
        } else {
            Some((page + 1).to_string())
        };

        Ok(CatalogBatch {
            entries,
            failed,
            next_cursor,
        })
    }
}

/// Map one SteamSpy record into a catalog entry. Returns `None` for a record the
/// seeder should count as failed (no usable appid).
fn parse_steamspy_entry(item: &Value) -> Option<CatalogEntry> {
    let appid = item
        .get("appid")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())?;
    let name = item
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let positive = item.get("positive").and_then(Value::as_u64).unwrap_or(0);
    let negative = item.get("negative").and_then(Value::as_u64).unwrap_or(0);
    let peak_ccu = item.get("ccu").and_then(Value::as_u64);

    Some(CatalogEntry {
        appid,
        name,
        // The SteamSpy `all` listing is games; the classification is implicit.
        classification: Classification::Game,
        review_count: Some(positive + negative),
        // Owners is a formatted range string; left unparsed (secondary metric).
        owners: None,
        peak_ccu,
    })
}
