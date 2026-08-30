// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type Key = (String, String, Option<String>);

const REQUIRED: &[(&str, &str, Option<&str>)] = &[
    ("http1", "client-response", None),
    ("http1", "proxy-request", None),
    ("http1", "proxy-response", None),
    ("https-http1", "client-response", None),
    ("https-http1", "proxy-request", None),
    ("https-http1", "proxy-response", None),
    ("https-http2", "client-response", None),
    ("https-http2", "proxy-request", None),
    ("https-http2", "proxy-response", None),
    ("websocket", "client-message", None),
    ("websocket", "proxy-handshake-request", None),
    ("websocket", "proxy-handshake-response", None),
    ("websocket", "proxy-message", Some("client-to-server")),
    ("websocket", "proxy-message", Some("server-to-client")),
    ("matrix", "har-source", None),
    ("matrix", "har-output", None),
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Complete,
    Partial,
    Empty,
    Bounded,
    Truncated,
    Unsupported,
    Failed,
    NotMeasured,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Observation {
    pub scenario: String,
    pub kind: String,
    pub status: Status,
    pub protocol: Option<String>,
    pub direction: Option<String>,
    pub byte_length: usize,
    pub digest: Option<String>,
    pub detail: Option<String>,
}

impl Observation {
    pub fn complete(scenario: &str, kind: &str, protocol: Option<&str>, bytes: &[u8]) -> Self {
        Self {
            scenario: scenario.into(),
            kind: kind.into(),
            status: if bytes.is_empty() {
                Status::Empty
            } else {
                Status::Complete
            },
            protocol: protocol.map(str::to_string),
            direction: None,
            byte_length: bytes.len(),
            digest: Some(digest(bytes)),
            detail: None,
        }
    }

    pub fn complete_empty(scenario: &str, kind: &str, protocol: Option<&str>) -> Self {
        Self {
            scenario: scenario.into(),
            kind: kind.into(),
            status: Status::Complete,
            protocol: protocol.map(str::to_string),
            direction: None,
            byte_length: 0,
            digest: Some(digest(&[])),
            detail: None,
        }
    }

    pub fn result(scenario: &str, kind: &str, status: Status, detail: impl Into<String>) -> Self {
        Self {
            scenario: scenario.into(),
            kind: kind.into(),
            status,
            protocol: None,
            direction: None,
            byte_length: 0,
            digest: None,
            detail: Some(detail.into()),
        }
    }

    pub fn key(&self) -> Key {
        (
            self.scenario.clone(),
            self.kind.clone(),
            self.direction.clone(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BackendRun {
    pub backend: String,
    pub version: String,
    pub platform: String,
    pub loopback_only: bool,
    pub trust_store_mutated: bool,
    pub cache_capacity: Option<u64>,
    pub key_log_lines: usize,
    pub shutdown_trials: Vec<Status>,
    pub observations: Vec<Observation>,
    pub limitations: Vec<String>,
}

impl BackendRun {
    pub fn failed(backend: &str, version: &str, detail: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
            version: version.into(),
            platform: "windows-x86_64".into(),
            loopback_only: true,
            trust_store_mutated: false,
            cache_capacity: None,
            key_log_lines: 0,
            shutdown_trials: Vec::new(),
            observations: vec![Observation::result(
                "matrix",
                "backend-run",
                Status::Failed,
                detail,
            )],
            limitations: Vec::new(),
        }
    }

    pub fn sort(&mut self) {
        self.observations.sort_by_key(Observation::key);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComparisonRow {
    pub scenario: String,
    pub kind: String,
    pub direction: Option<String>,
    pub left: Status,
    pub right: Status,
    pub parity: bool,
}

pub fn compare(left: &BackendRun, right: &BackendRun) -> Vec<ComparisonRow> {
    let left_rows: BTreeMap<_, _> = left
        .observations
        .iter()
        .map(|row| (row.key(), row))
        .collect();
    let right_rows: BTreeMap<_, _> = right
        .observations
        .iter()
        .map(|row| (row.key(), row))
        .collect();
    let mut keys: Vec<_> = REQUIRED
        .iter()
        .map(|(s, k, d)| ((*s).into(), (*k).into(), d.map(str::to_string)))
        .chain(left_rows.keys().cloned())
        .chain(right_rows.keys().cloned())
        .collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .map(|(scenario, kind, direction)| {
            let key = (scenario.clone(), kind.clone(), direction.clone());
            let a = left_rows.get(&key).copied();
            let b = right_rows.get(&key).copied();
            ComparisonRow {
                scenario,
                kind,
                direction,
                left: a.map_or(Status::NotMeasured, |row| row.status),
                right: b.map_or(Status::NotMeasured, |row| row.status),
                parity: observations_agree(a, b),
            }
        })
        .collect()
}

fn observations_agree(left: Option<&Observation>, right: Option<&Observation>) -> bool {
    matches!((left, right), (Some(a), Some(b)) if a.status == Status::Complete
        && b.status == Status::Complete && a.protocol == b.protocol
        && a.byte_length == b.byte_length && a.digest == b.digest)
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
