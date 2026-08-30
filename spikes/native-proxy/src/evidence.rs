// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Complete,
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
            scenario: scenario.to_string(),
            kind: kind.to_string(),
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

    pub fn result(scenario: &str, kind: &str, status: Status, detail: impl Into<String>) -> Self {
        Self {
            scenario: scenario.to_string(),
            kind: kind.to_string(),
            status,
            protocol: None,
            direction: None,
            byte_length: 0,
            digest: None,
            detail: Some(detail.into()),
        }
    }

    pub fn key(&self) -> (String, String, Option<String>) {
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
            backend: backend.to_string(),
            version: version.to_string(),
            platform: "windows-x86_64".to_string(),
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
    pub candidate: Status,
    pub baseline: Status,
    pub parity: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Comparison {
    pub candidate: BackendRun,
    pub baseline: BackendRun,
    pub rows: Vec<ComparisonRow>,
}

impl Comparison {
    pub fn new(mut candidate: BackendRun, mut baseline: BackendRun) -> Self {
        candidate.sort();
        baseline.sort();
        let candidate_rows: BTreeMap<_, _> = candidate
            .observations
            .iter()
            .map(|row| (row.key(), row.status))
            .collect();
        let baseline_rows: BTreeMap<_, _> = baseline
            .observations
            .iter()
            .map(|row| (row.key(), row.status))
            .collect();
        let mut keys: Vec<_> = candidate_rows
            .keys()
            .chain(baseline_rows.keys())
            .cloned()
            .collect();
        keys.sort();
        keys.dedup();
        let rows = keys
            .into_iter()
            .map(|(scenario, kind, direction)| {
                let key = (scenario.clone(), kind.clone(), direction.clone());
                let candidate_status = candidate_rows
                    .get(&key)
                    .copied()
                    .unwrap_or(Status::NotMeasured);
                let baseline_status = baseline_rows
                    .get(&key)
                    .copied()
                    .unwrap_or(Status::NotMeasured);
                ComparisonRow {
                    scenario,
                    kind,
                    direction,
                    candidate: candidate_status,
                    baseline: baseline_status,
                    parity: candidate_status == Status::Complete
                        && baseline_status == Status::Complete,
                }
            })
            .collect();
        Self {
            candidate,
            baseline,
            rows,
        }
    }
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
