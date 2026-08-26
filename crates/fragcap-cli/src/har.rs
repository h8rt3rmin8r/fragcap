// SPDX-License-Identifier: Apache-2.0

//! Utility-wide HAR projection for application observations with HTTP semantics.

use serde_json::json;

/// One HTTP-semantic observation eligible for HAR projection.
pub(crate) struct Entry<'a> {
    pub(crate) started_at: &'a str,
    pub(crate) method: &'a str,
    pub(crate) url: &'a str,
    pub(crate) status: u16,
}

/// Render a HAR 1.2 document from observable HTTP-semantic records.
pub(crate) fn render(entries: &[Entry<'_>]) -> Result<String, serde_json::Error> {
    let entries: Vec<_> = entries
        .iter()
        .map(|entry| {
            json!({
                "startedDateTime": entry.started_at,
                "time": 0,
                "request": {
                    "method": entry.method,
                    "url": entry.url,
                    "httpVersion": "HTTP/1.1",
                    "headers": [],
                    "queryString": [],
                    "cookies": [],
                    "headersSize": -1,
                    "bodySize": 0,
                },
                "response": {
                    "status": entry.status,
                    "statusText": "",
                    "httpVersion": "HTTP/1.1",
                    "headers": [],
                    "cookies": [],
                    "content": { "size": 0, "mimeType": "application/octet-stream" },
                    "redirectURL": "",
                    "headersSize": -1,
                    "bodySize": 0,
                },
                "cache": {},
                "timings": { "send": 0, "wait": 0, "receive": 0 },
            })
        })
        .collect();
    serde_json::to_string_pretty(&json!({
        "log": {
            "version": "1.2",
            "creator": { "name": "fragcap", "version": env!("CARGO_PKG_VERSION") },
            "entries": entries,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_preserves_observed_time_and_http_fields() {
        let output = render(&[Entry {
            started_at: "2026-01-01T00:00:00Z",
            method: "GET",
            url: "http://127.0.0.1/controlled",
            status: 200,
        }])
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        let entry = &value["log"]["entries"][0];
        assert_eq!(entry["startedDateTime"], "2026-01-01T00:00:00Z");
        assert_eq!(entry["request"]["method"], "GET");
        assert_eq!(entry["response"]["status"], 200);
    }
}
