// SPDX-License-Identifier: Apache-2.0

//! Bounded HAR 1.2 projection from authoritative application JSON Lines.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use base64::Engine;
use serde_json::{json, Value};

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TRANSACTIONS: usize = 4_096;
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

#[derive(Default)]
struct Transaction {
    started_ns: Option<u64>,
    protocol: Option<String>,
    request: Option<Value>,
    response: Option<Value>,
    timing: Option<Value>,
    terminal: Option<String>,
    request_bodies: Bodies,
    response_bodies: Bodies,
    correlation: Option<Value>,
    body_evidence_gap: bool,
}

#[derive(Default)]
struct Bodies {
    representations: BTreeMap<String, Vec<u8>>,
    limited: BTreeMap<String, bool>,
    observed: BTreeMap<String, u64>,
}

impl Bodies {
    fn selected(&self) -> (&[u8], bool, u64, &'static str) {
        for (name, label) in [
            ("content-decoded", "content-decoded"),
            ("transfer-decoded", "transfer-decoded"),
            ("raw", "raw"),
        ] {
            if let Some(bytes) = self.representations.get(name) {
                return (
                    bytes,
                    self.limited.get(name).copied().unwrap_or(false),
                    self.observed
                        .get(name)
                        .copied()
                        .unwrap_or(bytes.len() as u64),
                    label,
                );
            }
            if self.limited.get(name).copied().unwrap_or(false) {
                return (
                    &[],
                    true,
                    self.observed.get(name).copied().unwrap_or(0),
                    label,
                );
            }
        }
        (&[], false, 0, "none")
    }
}

pub struct HarProjection {
    pub json: String,
    pub standard_entries: usize,
    pub partial_entries: usize,
}

impl HarProjection {
    pub fn publish(self, destination: &Path) -> io::Result<Self> {
        let temporary = destination.with_extension("har.tmp");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| {
            file.write_all(self.json.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::hard_link(&temporary, destination)?;
            let _ = fs::remove_file(&temporary);
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map(|()| self)
    }
}

pub fn project_application_har(path: &Path) -> io::Result<HarProjection> {
    if path.metadata()?.len() > MAX_SOURCE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "application stream exceeds HAR source bound",
        ));
    }
    if super::read_application_prefix(path)?.status != super::ApplicationStreamStatus::Complete {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "application stream trailer does not reconcile",
        ));
    }
    let mut transactions = BTreeMap::<(u64, u64), Transaction>::new();
    let mut correlations = BTreeMap::<u64, Value>::new();
    let mut trailer = false;
    for line in BufReader::new(File::open(path)?).lines() {
        let record: Value = serde_json::from_str(&line?).map_err(io::Error::other)?;
        let kind = record
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if kind == "application.trailer" {
            trailer = record.get("writer_status").and_then(Value::as_str) == Some("complete");
            continue;
        }
        if kind == "application.gap" {
            apply_body_losses(&record, &mut transactions)?;
            continue;
        }
        let Some(connection) = record.get("proxy_connection_id").and_then(Value::as_u64) else {
            continue;
        };
        if kind == "application.correlation" {
            correlations.insert(connection, record);
            continue;
        }
        let Some(stream) = record.get("http_stream_id").and_then(Value::as_u64) else {
            continue;
        };
        if transactions.len() >= MAX_TRANSACTIONS
            && !transactions.contains_key(&(connection, stream))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HAR transaction bound exceeded",
            ));
        }
        let tx = transactions.entry((connection, stream)).or_default();
        tx.protocol = record
            .get("protocol")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| tx.protocol.take());
        match kind {
            "http.metadata" => match record.get("kind").and_then(Value::as_str) {
                Some("request") => {
                    tx.started_ns = record.get("event_time_ns").and_then(Value::as_u64);
                    tx.request = Some(record);
                }
                Some("response") => tx.response = Some(record),
                _ => {}
            },
            "http.timing" => tx.timing = Some(record),
            "http.stream.terminal" => {
                tx.terminal = record
                    .get("outcome")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            }
            "http.body_segment" => append_body(tx, &record)?,
            _ => {}
        }
    }
    if !trailer {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "application stream has no complete trailer",
        ));
    }
    let mut entries = Vec::new();
    let mut partial = Vec::new();
    for ((connection, stream), mut tx) in transactions {
        tx.correlation = correlations.get(&connection).cloned();
        match standard_entry(connection, stream, &tx) {
            Ok(entry) => entries.push(entry),
            Err(reasons) => partial.push(json!({
                "proxyConnectionId": connection, "httpStreamId": stream,
                "missingOrPartial": reasons, "correlation": tx.correlation,
                "source": {
                    "request": tx.request,
                    "response": tx.response,
                    "timing": tx.timing,
                    "terminal": tx.terminal,
                    "requestBody": body_summary(&tx.request_bodies),
                    "responseBody": body_summary(&tx.response_bodies)
                }
            })),
        }
    }
    let standard_entries = entries.len();
    let partial_entries = partial.len();
    let json = serde_json::to_string_pretty(&json!({"log": {
        "version": "1.2", "creator": {"name":"fragcap","version":env!("CARGO_PKG_VERSION")},
        "entries": entries, "_fragcapPartialEntries": partial
    }}))
    .map_err(io::Error::other)?;
    Ok(HarProjection {
        json,
        standard_entries,
        partial_entries,
    })
}

fn apply_body_losses(
    record: &Value,
    transactions: &mut BTreeMap<(u64, u64), Transaction>,
) -> io::Result<()> {
    let Some(losses) = record.get("body_losses").and_then(Value::as_array) else {
        return Ok(());
    };
    for loss in losses {
        let Some(connection) = loss.get("proxy_connection_id").and_then(Value::as_u64) else {
            continue;
        };
        let Some(stream) = loss.get("http_stream_id").and_then(Value::as_u64) else {
            continue;
        };
        if transactions.len() >= MAX_TRANSACTIONS
            && !transactions.contains_key(&(connection, stream))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HAR transaction bound exceeded",
            ));
        }
        let tx = transactions.entry((connection, stream)).or_default();
        tx.body_evidence_gap = true;
        let bodies = if loss.get("direction").and_then(Value::as_str) == Some("request") {
            &mut tx.request_bodies
        } else {
            &mut tx.response_bodies
        };
        let representation = loss
            .get("representation")
            .and_then(Value::as_str)
            .unwrap_or("raw");
        let observed = loss
            .get("observed_bytes")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let slot = bodies
            .observed
            .entry(representation.to_string())
            .or_default();
        *slot = slot.saturating_add(observed);
        bodies.limited.insert(representation.to_string(), true);
    }
    Ok(())
}

fn body_summary(bodies: &Bodies) -> Value {
    let (retained, limited, observed, representation) = bodies.selected();
    json!({
        "representation": representation,
        "observedBytes": observed,
        "retainedBytes": retained.len(),
        "limited": limited,
    })
}

/// Project and atomically publish a HAR without ever exposing a partial file.
pub fn publish_application_har(
    application: &Path,
    destination: &Path,
) -> io::Result<HarProjection> {
    project_application_har(application)?.publish(destination)
}

fn append_body(tx: &mut Transaction, record: &Value) -> io::Result<()> {
    let representation = record
        .get("representation")
        .and_then(Value::as_str)
        .unwrap_or("raw");
    let Some(payload) = record.get("payload").and_then(Value::as_str) else {
        let bodies = if record.get("direction").and_then(Value::as_str) == Some("request") {
            &mut tx.request_bodies
        } else {
            &mut tx.response_bodies
        };
        *bodies
            .observed
            .entry(representation.to_string())
            .or_default() = bodies
            .observed
            .get(representation)
            .copied()
            .unwrap_or(0)
            .saturating_add(
                record
                    .get("observed_len")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
        bodies.limited.insert(representation.to_string(), true);
        return Ok(());
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let bodies = if record.get("direction").and_then(Value::as_str) == Some("request") {
        &mut tx.request_bodies
    } else {
        &mut tx.response_bodies
    };
    let target = bodies
        .representations
        .entry(representation.to_string())
        .or_default();
    *bodies
        .observed
        .entry(representation.to_string())
        .or_default() = bodies
        .observed
        .get(representation)
        .copied()
        .unwrap_or(0)
        .saturating_add(
            record
                .get("observed_len")
                .and_then(Value::as_u64)
                .unwrap_or(bytes.len() as u64),
        );
    let offset_matches = record.get("offset").and_then(Value::as_u64) == Some(target.len() as u64);
    if target.len().saturating_add(bytes.len()) > MAX_BODY_BYTES {
        bodies.limited.insert(representation.to_string(), true);
    } else {
        target.extend_from_slice(&bytes);
    }
    if !offset_matches || record.get("outcome").and_then(Value::as_str) != Some("complete") {
        bodies.limited.insert(representation.to_string(), true);
    }
    Ok(())
}

fn standard_entry(
    connection: u64,
    stream: u64,
    tx: &Transaction,
) -> Result<Value, Vec<&'static str>> {
    let mut missing = Vec::new();
    let request = tx.request.as_ref().unwrap_or_else(|| {
        missing.push("request");
        &Value::Null
    });
    let response = tx.response.as_ref().unwrap_or_else(|| {
        missing.push("response");
        &Value::Null
    });
    let timing = tx.timing.as_ref().unwrap_or_else(|| {
        missing.push("timing");
        &Value::Null
    });
    if tx.terminal.as_deref() != Some("complete") {
        missing.push("complete-terminal");
    }
    if tx.body_evidence_gap {
        missing.push("body-evidence-gap");
    }
    let method = binary_text(request.get("method"));
    let url = binary_text(request.get("url"));
    let status = response.get("status").and_then(Value::as_u64);
    if method.is_none() {
        missing.push("method");
    }
    if url.is_none() {
        missing.push("absolute-url");
    }
    if status.is_none() {
        missing.push("status");
    }
    let send = nanos_ms(timing.get("send_ns"));
    let wait = nanos_ms(timing.get("wait_ns"));
    let receive = nanos_ms(timing.get("receive_ns"));
    if send.is_none() || wait.is_none() || receive.is_none() {
        missing.push("phase-timings");
    }
    let started = tx.started_ns.map(rfc3339_nanos);
    if started.is_none() {
        missing.push("started-time");
    }
    if !missing.is_empty() {
        return Err(missing);
    }
    let request_headers = headers(request)?;
    let response_headers = headers(response)?;
    let (request_body, request_limited, request_observed, request_representation) =
        tx.request_bodies.selected();
    let (response_body, response_limited, response_observed, response_representation) =
        tx.response_bodies.selected();
    let mime = header_value(&response_headers, "content-type").unwrap_or_default();
    let content = if response_limited {
        json!({"size":-1,"mimeType":mime,"_fragcap":{"limited":true,"representation":response_representation,"retainedBytes":response_body.len(),"observedBytes":response_observed}})
    } else if let Ok(text) = std::str::from_utf8(response_body) {
        json!({"size":response_body.len(),"mimeType":mime,"text":text,"_fragcap":{"representation":response_representation}})
    } else {
        json!({"size":response_body.len(),"mimeType":mime,"text":base64::engine::general_purpose::STANDARD.encode(response_body),"encoding":"base64","_fragcap":{"representation":response_representation}})
    };
    let request_mime = header_value(&request_headers, "content-type").unwrap_or_default();
    let post_data = if request_body.is_empty() {
        None
    } else if let Ok(text) = std::str::from_utf8(request_body) {
        Some(
            json!({"mimeType": request_mime, "text": text, "_fragcap":{"limited":request_limited,"representation":request_representation}}),
        )
    } else {
        None
    };
    let mut request_value = json!({
        "method":method.unwrap(),"url":url.unwrap(),"httpVersion":har_version(tx.protocol.as_deref()),
        "headers":request_headers,"queryString":derived(request,"query"),"cookies":derived(request,"cookies"),
        "headersSize":head_size(request),"bodySize":if request_limited {-1_i64} else {i64::try_from(request_body.len()).unwrap_or(i64::MAX)}
    });
    if let Some(post_data) = post_data {
        request_value["postData"] = post_data;
    } else if !request_body.is_empty() {
        request_value["_fragcapBinaryPostData"] = json!({
            "encoding":"base64",
            "text":base64::engine::general_purpose::STANDARD.encode(request_body),
            "limited":request_limited,
            "representation":request_representation
        });
    }
    let redirect = header_value(&response_headers, "location").unwrap_or_default();
    let total = send.unwrap() + wait.unwrap() + receive.unwrap();
    Ok(json!({"startedDateTime":started.unwrap(),"time":total,
        "request":request_value,
        "response":{"status":status.unwrap(),"statusText":binary_text(response.get("reason")).unwrap_or_default(),"httpVersion":har_version(tx.protocol.as_deref()),"headers":response_headers,"cookies":response_cookies(response),"content":content,"redirectURL":redirect,"headersSize":head_size(response),"bodySize":if response_limited {-1_i64} else {i64::try_from(response_body.len()).unwrap_or(i64::MAX)}},
        "cache":{},"timings":{"send":send.unwrap(),"wait":wait.unwrap(),"receive":receive.unwrap()},
        "_fragcap":{"proxyConnectionId":connection,"httpStreamId":stream,"correlation":tx.correlation,
            "requestBody":{"representation":request_representation,"observedBytes":request_observed,"retainedBytes":request_body.len(),"limited":request_limited},
            "responseBody":{"representation":response_representation,"observedBytes":response_observed,"retainedBytes":response_body.len(),"limited":response_limited}}
    }))
}

fn binary_text(value: Option<&Value>) -> Option<String> {
    let value = value?;
    let raw = value.get("value")?.as_str()?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(raw).ok()?;
    String::from_utf8(bytes).ok()
}
fn headers(record: &Value) -> Result<Vec<Value>, Vec<&'static str>> {
    let Some(fields) = record.get("fields").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    fields
        .iter()
        .map(|field| {
            Ok(json!({
                "name": binary_field(field,"name").ok_or(vec!["binary-header"] )?,
                "value": binary_field(field,"value").ok_or(vec!["binary-header"] )?,
            }))
        })
        .collect()
}
fn binary_field(value: &Value, key: &str) -> Option<String> {
    let raw = value.get(key)?.as_str()?;
    String::from_utf8(base64::engine::general_purpose::STANDARD.decode(raw).ok()?).ok()
}
fn derived(record: &Value, key: &str) -> Vec<Value> {
    record
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| {
            Some(json!({"name":binary_text(v.get("name"))?,"value":binary_text(v.get("value"))?}))
        })
        .collect()
}
fn header_value(headers: &[Value], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|h| {
            h.get("name")
                .and_then(Value::as_str)
                .is_some_and(|v| v.eq_ignore_ascii_case(name))
        })
        .and_then(|h| h.get("value"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}
fn head_size(record: &Value) -> i64 {
    record
        .get("head_bytes")
        .and_then(Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .unwrap_or(-1)
}
fn response_cookies(record: &Value) -> Vec<Value> {
    let Ok(headers) = headers(record) else {
        return Vec::new();
    };
    headers
        .iter()
        .filter(|header| {
            header
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.eq_ignore_ascii_case("set-cookie"))
        })
        .filter_map(|header| {
            let value = header.get("value")?.as_str()?;
            let pair = value.split(';').next()?;
            let (name, value) = pair.split_once('=')?;
            Some(json!({"name":name.trim(),"value":value.trim()}))
        })
        .collect()
}
fn nanos_ms(value: Option<&Value>) -> Option<f64> {
    Some(value?.as_u64()? as f64 / 1_000_000.0)
}
fn har_version(value: Option<&str>) -> &'static str {
    if value == Some("h2") {
        "HTTP/2"
    } else {
        "HTTP/1.1"
    }
}
fn rfc3339_nanos(nanos: u64) -> String {
    let secs = (nanos / 1_000_000_000) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3_600, (rem % 3_600) / 60, rem % 60);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = year + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binary(value: &str) -> Value {
        json!({"encoding":"base64","value":base64::engine::general_purpose::STANDARD.encode(value)})
    }

    fn complete_transaction() -> Transaction {
        let mut response_bodies = Bodies::default();
        response_bodies
            .representations
            .insert("raw".to_string(), b"wire".to_vec());
        response_bodies
            .representations
            .insert("content-decoded".to_string(), b"decoded".to_vec());
        Transaction {
            started_ns: Some(1_700_000_000_000_000_000),
            protocol: Some("http/1.1".to_string()),
            request: Some(json!({
                "method":binary("GET"), "url":binary("https://example.test/a"),
                "fields":[], "query":[], "cookies":[], "head_bytes":40
            })),
            response: Some(json!({
                "status":200, "reason":binary("OK"), "fields":[], "head_bytes":20
            })),
            timing: Some(json!({"send_ns":1_000_000,"wait_ns":2_000_000,"receive_ns":3_000_000})),
            terminal: Some("complete".to_string()),
            response_bodies,
            ..Transaction::default()
        }
    }

    #[test]
    fn complete_transactions_use_the_most_derived_available_body() {
        let entry = standard_entry(7, 9, &complete_transaction()).unwrap();
        assert_eq!(entry["time"], 6.0);
        assert_eq!(entry["response"]["content"]["text"], "decoded");
        assert_eq!(
            entry["response"]["content"]["_fragcap"]["representation"],
            "content-decoded"
        );
    }

    #[test]
    fn incomplete_transactions_never_enter_standard_har_entries() {
        let mut tx = complete_transaction();
        tx.timing = None;
        let reasons = standard_entry(7, 9, &tx).unwrap_err();
        assert!(reasons.contains(&"timing"));
        assert!(reasons.contains(&"phase-timings"));
    }

    #[test]
    fn queue_dropped_body_evidence_forces_a_partial_entry() {
        let mut transactions = BTreeMap::from([((7, 9), complete_transaction())]);
        apply_body_losses(
            &json!({"body_losses":[{
                "proxy_connection_id":7,
                "http_stream_id":9,
                "direction":"response",
                "representation":"content-decoded",
                "observed_bytes":11
            }]}),
            &mut transactions,
        )
        .unwrap();
        let tx = transactions.get(&(7, 9)).unwrap();
        let reasons = standard_entry(7, 9, tx).unwrap_err();
        assert!(reasons.contains(&"body-evidence-gap"));
        assert_eq!(tx.response_bodies.observed["content-decoded"], 11);
        assert!(tx.response_bodies.limited["content-decoded"]);
    }

    #[test]
    fn binary_and_omitted_bodies_have_explicit_non_placeholder_representation() {
        let mut binary = complete_transaction();
        binary.response_bodies = Bodies::default();
        binary
            .response_bodies
            .representations
            .insert("raw".to_string(), vec![0xff, 0x00]);
        let entry = standard_entry(1, 1, &binary).unwrap();
        assert_eq!(entry["response"]["content"]["encoding"], "base64");
        assert_eq!(entry["response"]["content"]["text"], "/wA=");

        let mut omitted = complete_transaction();
        omitted.response_bodies = Bodies::default();
        omitted
            .response_bodies
            .limited
            .insert("raw".to_string(), true);
        omitted
            .response_bodies
            .observed
            .insert("raw".to_string(), 4096);
        let entry = standard_entry(1, 1, &omitted).unwrap();
        assert_eq!(entry["response"]["bodySize"], -1);
        assert_eq!(entry["response"]["content"]["size"], -1);
        assert_eq!(
            entry["response"]["content"]["_fragcap"]["observedBytes"],
            4096
        );
    }

    #[test]
    fn atomic_publication_never_overwrites_an_existing_archive() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("http.har");
        fs::write(&destination, b"existing").unwrap();
        let projection = HarProjection {
            json: "{}".to_string(),
            standard_entries: 0,
            partial_entries: 0,
        };
        assert!(projection.publish(&destination).is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"existing");
        assert!(!directory.path().join("http.har.tmp").exists());
    }
}
