// SPDX-License-Identifier: Apache-2.0

//! Mechanical intake for a reviewer-owned completed native-product review.
//!
//! This module rejects incomplete or inconsistent records. It cannot establish
//! reviewer authenticity or independence and therefore never emits an approval.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

const CANDIDATE: &str = "docs/security/native-product-review-candidate.v1.json";
const PUBLISHED: &str = "docs/published-release.json";
const WORKFLOW: &str = ".github/workflows/published-review-candidate.yml";
const MAX_RECORD_BYTES: usize = 512 * 1024;
const MAX_STRING_CHARS: usize = 4096;
const MAX_ARRAY_ITEMS: usize = 128;
const AREAS: [&str; 12] = [
    "architecture",
    "dependencies",
    "unsafe-code",
    "parsers",
    "tls",
    "certificate-issuance",
    "trust",
    "listener-isolation",
    "routing",
    "artifact-protection",
    "recovery",
    "packaging",
];
const INSTALLED_CASES: [&str; 7] = [
    "listener-binding",
    "destination-refusal",
    "trust-recovery",
    "artifact-protection-export",
    "parser-abuse",
    "installer-lifecycle",
    "quic-installed-retest",
];

pub fn run(root: &Path, arguments: &[String]) -> io::Result<usize> {
    if arguments.len() > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "use review-record [reviewer-owned-record.json]",
        ));
    }
    let candidate = read_json(&root.join(CANDIDATE), MAX_RECORD_BYTES)?;
    let published = read_json(&root.join(PUBLISHED), MAX_RECORD_BYTES)?;
    let mut problems = validate_candidate(&candidate, &published);
    problems.extend(validate_repository(root));
    if let Some(path) = arguments.first() {
        let bytes = fs::read(path)?;
        if bytes.len() > MAX_RECORD_BYTES {
            problems.push("review record exceeds 512 KiB".into());
        } else {
            let record: Value = serde_json::from_slice(&bytes).map_err(invalid_data)?;
            problems.extend(validate_record(&record, &candidate));
        }
    }
    for problem in &problems {
        eprintln!("review-record: {problem}");
    }
    if problems.is_empty() {
        if arguments.is_empty() {
            println!("review-record: immutable published candidate registry is valid; independent review NOT performed");
        } else {
            println!("review-record: no mechanical blocker found; reviewer authenticity, independence, and #333 acceptance remain external");
        }
    }
    Ok(problems.len())
}

fn validate_repository(root: &Path) -> Vec<String> {
    let workflow = fs::read_to_string(root.join(WORKFLOW)).unwrap_or_default();
    [
        "runs-on: windows-2025",
        "cargo xtask review-record",
        "native-product-review-candidate.v1.json",
        "Test-PackageCertification.ps1",
        "cargo xtask package-certification validate-report",
        "published-review-candidate-summary",
    ]
    .into_iter()
    .filter(|marker| !workflow.contains(marker))
    .map(|marker| format!("published review workflow lacks required marker {marker}"))
    .collect()
}

fn read_json(path: &Path, limit: usize) -> io::Result<Value> {
    let bytes = fs::read(path)?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} exceeds {limit} bytes", path.display()),
        ));
    }
    serde_json::from_slice(&bytes).map_err(invalid_data)
}

fn validate_candidate(candidate: &Value, published: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    exact_keys(
        candidate,
        &[
            "schema_version",
            "product_version",
            "tag",
            "source_revision",
            "lockfile_sha256",
            "release_url",
            "release_run_id",
            "target",
            "features",
            "assets",
            "predecessor",
        ],
        "candidate",
        &mut problems,
    );
    if candidate["schema_version"] != 1
        || candidate["product_version"] != published["version"]
        || candidate["source_revision"] != published["source_revision"]
        || candidate["release_url"] != published["release_url"]
        || candidate["tag"] != format!("v{}", published["version"].as_str().unwrap_or_default())
    {
        problems.push("candidate does not match the published release authority".into());
    }
    for (name, length) in [("source_revision", 40), ("lockfile_sha256", 64)] {
        if !is_lower_hex(candidate[name].as_str().unwrap_or_default(), length) {
            problems.push(format!("candidate.{name} is not canonical hexadecimal"));
        }
    }
    if candidate["release_run_id"]
        .as_u64()
        .is_none_or(|id| id == 0)
        || candidate["target"] != "x86_64-pc-windows-msvc"
        || !is_https(candidate["release_url"].as_str().unwrap_or_default())
    {
        problems.push("candidate release run, target, or URL is invalid".into());
    }
    let features = string_set(&candidate["features"]);
    if features
        != BTreeSet::from([
            "etw".into(),
            "live".into(),
            "native-deep-capture".into(),
            "socket-table".into(),
        ])
    {
        problems.push("candidate feature closure is not the official set".into());
    }
    let expected_roles = BTreeSet::from([
        "portable-zip",
        "portable-zip-checksum",
        "standalone-catalog",
        "standalone-catalog-checksum",
        "windows-msi",
        "windows-msi-checksum",
    ]);
    let assets = candidate["assets"].as_array();
    let mut roles = BTreeSet::new();
    let mut filenames = BTreeSet::new();
    for asset in assets.into_iter().flatten() {
        exact_keys(
            asset,
            &["role", "filename", "url", "size_bytes", "sha256"],
            "candidate asset",
            &mut problems,
        );
        let role = asset["role"].as_str().unwrap_or_default();
        let filename = asset["filename"].as_str().unwrap_or_default();
        if !roles.insert(role) || !filenames.insert(filename) {
            problems.push("candidate assets contain a duplicate role or filename".into());
        }
        if Path::new(filename)
            .file_name()
            .and_then(|name| name.to_str())
            != Some(filename)
            || asset["size_bytes"].as_u64().is_none_or(|size| size == 0)
            || !is_lower_hex(asset["sha256"].as_str().unwrap_or_default(), 64)
            || !is_https(asset["url"].as_str().unwrap_or_default())
        {
            problems.push(format!("candidate asset {role} has an invalid identity"));
        }
    }
    if roles != expected_roles || assets.is_none_or(|rows| rows.len() != 6) {
        problems.push(format!("candidate asset role set mismatch: {roles:?}"));
    }
    exact_keys(
        &candidate["predecessor"],
        &["version", "filename", "url", "size_bytes", "sha256"],
        "candidate predecessor",
        &mut problems,
    );
    if candidate["predecessor"]["version"] != "0.8.0"
        || candidate["predecessor"]["size_bytes"] != 3_436_544
        || candidate["predecessor"]["sha256"]
            != "eaf2554b1da3721400c1b00f5ea0a298455f59454b0084e617ed2efcdcf83901"
        || !is_https(candidate["predecessor"]["url"].as_str().unwrap_or_default())
    {
        problems.push("candidate predecessor identity is invalid".into());
    }
    problems
}

fn validate_record(record: &Value, candidate: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    validate_bounds(record, "$", &mut problems);
    exact_keys(
        record,
        &[
            "schema_version",
            "state",
            "candidate",
            "reviewer",
            "independence",
            "environment",
            "checks",
            "findings",
            "summary",
        ],
        "review record",
        &mut problems,
    );
    if record["schema_version"] != 1 || record["state"] != "performed" {
        problems.push("review record must declare schema 1 and state performed".into());
    }
    if record["candidate"] != *candidate {
        problems
            .push("review record candidate differs from the immutable published candidate".into());
    }
    validate_reviewer(&record["reviewer"], &mut problems);
    validate_independence(&record["independence"], &mut problems);
    validate_environment(&record["environment"], &mut problems);
    let finding_ids = validate_findings(
        &record["findings"],
        record["reviewer"]["id"].as_str().unwrap_or_default(),
        candidate["source_revision"].as_str().unwrap_or_default(),
        &mut problems,
    );
    validate_checks(&record["checks"], &finding_ids, &mut problems);
    validate_summary(record, &mut problems);
    problems
}

fn validate_reviewer(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &[
            "id",
            "name",
            "organization",
            "contact",
            "started_at",
            "completed_at",
        ],
        "reviewer",
        problems,
    );
    for field in [
        "id",
        "name",
        "organization",
        "contact",
        "started_at",
        "completed_at",
    ] {
        required_text(value, field, "reviewer", problems);
    }
    let started = value["started_at"].as_str().unwrap_or_default();
    let completed = value["completed_at"].as_str().unwrap_or_default();
    if !is_timestamp(started) || !is_timestamp(completed) || started >= completed {
        problems.push("reviewer timestamps are invalid or unordered".into());
    }
}

fn validate_independence(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &[
            "implementation_contributions",
            "conflicts",
            "relationship",
            "statement",
            "attestation_url",
        ],
        "independence",
        problems,
    );
    if value["implementation_contributions"] != false
        || value["conflicts"]
            .as_array()
            .is_none_or(|rows| !rows.is_empty())
    {
        problems.push("reviewer declares implementation contributions or conflicts".into());
    }
    for field in ["relationship", "statement"] {
        required_text(value, field, "independence", problems);
    }
    if !is_https(value["attestation_url"].as_str().unwrap_or_default()) {
        problems.push("independence attestation_url must be HTTPS".into());
    }
}

fn validate_environment(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &[
            "windows_version",
            "architecture",
            "disposable",
            "installed_sha256",
            "privilege",
            "network",
            "synthetic_data",
            "tools",
            "limitations",
            "installed_cases",
        ],
        "environment",
        problems,
    );
    if value["architecture"] != "x86_64"
        || value["disposable"] != true
        || value["synthetic_data"] != true
        || !is_lower_hex(value["installed_sha256"].as_str().unwrap_or_default(), 64)
    {
        problems
            .push("review environment is not a disposable synthetic x86_64 installed run".into());
    }
    for field in ["windows_version", "privilege", "network"] {
        required_text(value, field, "environment", problems);
    }
    if value["tools"].as_array().is_none_or(Vec::is_empty) {
        problems.push("review environment must identify its tools".into());
    }
    validate_results(
        &value["installed_cases"],
        &INSTALLED_CASES,
        "installed case",
        problems,
    );
}

fn validate_checks(value: &Value, finding_ids: &BTreeSet<String>, problems: &mut Vec<String>) {
    validate_results(value, &AREAS, "area result", problems);
    for row in value.as_array().into_iter().flatten() {
        let area = row["id"].as_str().unwrap_or_default();
        for finding in row["findings"].as_array().into_iter().flatten() {
            let id = finding.as_str().unwrap_or_default();
            if !finding_ids.contains(id) {
                problems.push(format!("area {area} references unknown finding {id}"));
            }
        }
    }
}

fn validate_results(value: &Value, expected: &[&str], context: &str, problems: &mut Vec<String>) {
    let rows = value.as_array();
    let mut seen = BTreeSet::new();
    for row in rows.into_iter().flatten() {
        exact_keys(
            row,
            &[
                "id",
                "outcome",
                "method",
                "evidence",
                "findings",
                "limitations",
            ],
            context,
            problems,
        );
        let id = row["id"].as_str().unwrap_or_default();
        if !expected.contains(&id) || !seen.insert(id) {
            problems.push(format!("unknown or duplicate {context} {id}"));
        }
        if row["outcome"] != "passed" {
            problems.push(format!("{context} {id} is not passed"));
        }
        required_text(row, "method", context, problems);
        validate_evidence(&row["evidence"], context, problems);
        if !row["findings"].is_array() || !row["limitations"].is_array() {
            problems.push(format!(
                "{context} {id} findings and limitations must be arrays"
            ));
        }
    }
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if seen != expected || rows.is_none_or(|rows| rows.len() != expected.len()) {
        problems.push(format!("{context} set mismatch: {seen:?}"));
    }
}

fn validate_findings(
    value: &Value,
    reviewer_id: &str,
    reviewed_revision: &str,
    problems: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(rows) = value.as_array() else {
        problems.push("findings must be an array".into());
        return ids;
    };
    for finding in rows {
        exact_keys(
            finding,
            &[
                "id",
                "area",
                "severity",
                "candidate_revision",
                "reproduction",
                "impact",
                "evidence",
                "owner",
                "linked_issue",
                "disposition",
                "remediation",
                "retest",
            ],
            "finding",
            problems,
        );
        let id = finding["id"].as_str().unwrap_or_default();
        if id.is_empty() || !ids.insert(id.to_string()) {
            problems.push(format!("finding has an empty or duplicate id {id}"));
        }
        if !AREAS.contains(&finding["area"].as_str().unwrap_or_default())
            || !["critical", "high", "medium", "low", "informational"]
                .contains(&finding["severity"].as_str().unwrap_or_default())
            || finding["candidate_revision"] != reviewed_revision
        {
            problems.push(format!(
                "finding {id} has invalid area, severity, or candidate"
            ));
        }
        for field in ["reproduction", "impact", "linked_issue", "disposition"] {
            required_text(finding, field, "finding", problems);
        }
        validate_evidence(&finding["evidence"], "finding", problems);
        let severity = finding["severity"].as_str().unwrap_or_default();
        let disposition = finding["disposition"].as_str().unwrap_or_default();
        if disposition == "open" {
            problems.push(format!("finding {id} remains open"));
        }
        if severity == "medium"
            && finding["owner"]
                .as_str()
                .is_none_or(|owner| owner.trim().is_empty())
        {
            problems.push(format!("medium finding {id} lacks an owner"));
        }
        if matches!(severity, "critical" | "high") {
            if disposition != "remediated" {
                problems.push(format!("{severity} finding {id} is not remediated"));
            }
            validate_remediation(finding, reviewer_id, reviewed_revision, problems);
        }
    }
    ids
}

fn validate_remediation(
    finding: &Value,
    reviewer_id: &str,
    reviewed_revision: &str,
    problems: &mut Vec<String>,
) {
    let id = finding["id"].as_str().unwrap_or_default();
    exact_keys(
        &finding["remediation"],
        &["fixed_revision", "pull_request"],
        "remediation",
        problems,
    );
    let fixed = finding["remediation"]["fixed_revision"]
        .as_str()
        .unwrap_or_default();
    if !is_lower_hex(fixed, 40)
        || fixed == reviewed_revision
        || !is_https(
            finding["remediation"]["pull_request"]
                .as_str()
                .unwrap_or_default(),
        )
    {
        problems.push(format!("finding {id} has invalid remediation identity"));
    }
    exact_keys(
        &finding["retest"],
        &[
            "reviewer_id",
            "candidate_revision",
            "outcome",
            "method",
            "evidence",
        ],
        "retest",
        problems,
    );
    if finding["retest"]["reviewer_id"] == reviewer_id
        || finding["retest"]["candidate_revision"] != fixed
        || finding["retest"]["outcome"] != "passed"
    {
        problems.push(format!(
            "finding {id} lacks a separate passed independent retest"
        ));
    }
    required_text(&finding["retest"], "method", "retest", problems);
    validate_evidence(&finding["retest"]["evidence"], "retest", problems);
}

fn validate_summary(record: &Value, problems: &mut Vec<String>) {
    let summary = &record["summary"];
    exact_keys(
        summary,
        &[
            "conclusion",
            "finding_counts",
            "p1_boundary",
            "no_open_proxy",
            "limitations",
            "evidence_sha256",
        ],
        "summary",
        problems,
    );
    if summary["conclusion"] != "no-mechanical-blockers"
        || summary["p1_boundary"] != "passed"
        || summary["no_open_proxy"] != "passed"
        || !summary["limitations"].is_array()
        || !is_lower_hex(summary["evidence_sha256"].as_str().unwrap_or_default(), 64)
    {
        problems.push("review summary is incomplete or overclaims its conclusion".into());
    }
    exact_keys(
        &summary["finding_counts"],
        &["critical", "high", "medium", "low", "informational"],
        "summary finding_counts",
        problems,
    );
    let mut actual = BTreeMap::from([
        ("critical", 0_u64),
        ("high", 0),
        ("medium", 0),
        ("low", 0),
        ("informational", 0),
    ]);
    for finding in record["findings"].as_array().into_iter().flatten() {
        if let Some(count) = finding["severity"]
            .as_str()
            .and_then(|severity| actual.get_mut(severity))
        {
            *count += 1;
        }
    }
    for (severity, count) in actual {
        if summary["finding_counts"][severity].as_u64() != Some(count) {
            problems.push(format!("summary count for {severity} is inconsistent"));
        }
    }
}

fn validate_evidence(value: &Value, context: &str, problems: &mut Vec<String>) {
    let Some(rows) = value.as_array() else {
        problems.push(format!("{context} evidence must be an array"));
        return;
    };
    if rows.is_empty() || rows.len() > 16 {
        problems.push(format!("{context} evidence count must be between 1 and 16"));
    }
    let mut ids = BTreeSet::new();
    for evidence in rows {
        exact_keys(
            evidence,
            &["id", "sha256", "visibility", "url"],
            "evidence",
            problems,
        );
        let id = evidence["id"].as_str().unwrap_or_default();
        if id.is_empty() || !ids.insert(id) {
            problems.push(format!("{context} evidence has an empty or duplicate id"));
        }
        if !is_lower_hex(evidence["sha256"].as_str().unwrap_or_default(), 64)
            || !matches!(evidence["visibility"].as_str(), Some("public" | "private"))
            || !(evidence["url"].is_null() || evidence["url"].as_str().is_some_and(is_https))
        {
            problems.push(format!("{context} evidence {id} has invalid identity"));
        }
    }
}

fn validate_bounds(value: &Value, path: &str, problems: &mut Vec<String>) {
    match value {
        Value::String(text) => {
            if text.chars().count() > MAX_STRING_CHARS {
                problems.push(format!(
                    "{path} string exceeds {MAX_STRING_CHARS} characters"
                ));
            }
            let lower = text.to_ascii_lowercase();
            if text.contains("C:\\Users\\")
                || text.contains("C:/Users/")
                || lower.contains("begin private key")
                || lower.contains("begin rsa private key")
            {
                problems.push(format!(
                    "{path} contains host-sensitive or private material"
                ));
            }
        }
        Value::Array(rows) => {
            if rows.len() > MAX_ARRAY_ITEMS {
                problems.push(format!("{path} array exceeds {MAX_ARRAY_ITEMS} items"));
            }
            for (index, row) in rows.iter().enumerate() {
                validate_bounds(row, &format!("{path}[{index}]"), problems);
            }
        }
        Value::Object(fields) => {
            for (name, child) in fields {
                validate_bounds(child, &format!("{path}.{name}"), problems);
            }
        }
        _ => {}
    }
}

fn exact_keys(value: &Value, expected: &[&str], context: &str, problems: &mut Vec<String>) {
    let actual = value
        .as_object()
        .map(|fields| fields.keys().map(String::as_str).collect::<BTreeSet<_>>());
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual.as_ref() != Some(&expected) {
        problems.push(format!("{context} keys must be exactly {expected:?}"));
    }
}

fn required_text(value: &Value, field: &str, context: &str, problems: &mut Vec<String>) {
    if value[field]
        .as_str()
        .is_none_or(|text| text.trim().is_empty())
    {
        problems.push(format!("{context}.{field} must be non-empty text"));
    }
}

fn string_set(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_https(value: &str) -> bool {
    value.starts_with("https://") && value.len() > "https://".len()
}

fn is_timestamp(value: &str) -> bool {
    value.len() >= 20 && value.ends_with('Z') && value.contains('T')
}

fn invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> Value {
        serde_json::json!({
            "schema_version": 1,
            "product_version": "0.10.1",
            "tag": "v0.10.1",
            "source_revision": "a".repeat(40),
            "lockfile_sha256": "b".repeat(64),
            "release_url": "https://example.invalid/v0.10.1",
            "release_run_id": 1,
            "target": "x86_64-pc-windows-msvc",
            "features": ["etw", "live", "native-deep-capture", "socket-table"],
            "assets": [
                asset("standalone-catalog", "catalog.db"),
                asset("standalone-catalog-checksum", "catalog.db.sha256"),
                asset("portable-zip", "fragcap.zip"),
                asset("portable-zip-checksum", "fragcap.zip.sha256"),
                asset("windows-msi", "fragcap.msi"),
                asset("windows-msi-checksum", "fragcap.msi.sha256")
            ],
            "predecessor": {"version":"0.8.0","filename":"fragcap-0.8.0-x86_64.msi","url":"https://example.invalid/predecessor.msi","size_bytes":3436544,"sha256":"eaf2554b1da3721400c1b00f5ea0a298455f59454b0084e617ed2efcdcf83901"}
        })
    }

    fn asset(role: &str, filename: &str) -> Value {
        serde_json::json!({"role":role,"filename":filename,"url":format!("https://example.invalid/{filename}"),"size_bytes":1,"sha256":"c".repeat(64)})
    }

    fn evidence(id: &str) -> Value {
        serde_json::json!({"id":id,"sha256":"d".repeat(64),"visibility":"public","url":format!("https://example.invalid/{id}")})
    }

    fn result(id: &str) -> Value {
        serde_json::json!({"id":id,"outcome":"passed","method":"Independent controlled inspection","evidence":[evidence(id)],"findings":[],"limitations":[]})
    }

    fn record(candidate: &Value) -> Value {
        let checks = AREAS.iter().map(|id| result(id)).collect::<Vec<_>>();
        let installed = INSTALLED_CASES
            .iter()
            .map(|id| result(id))
            .collect::<Vec<_>>();
        serde_json::json!({
            "schema_version":1,
            "state":"performed",
            "candidate":candidate,
            "reviewer":{"id":"reviewer-1","name":"Review Person","organization":"Independent Lab","contact":"security@example.invalid","started_at":"2026-09-01T00:00:00Z","completed_at":"2026-09-02T00:00:00Z"},
            "independence":{"implementation_contributions":false,"conflicts":[],"relationship":"No implementation relationship","statement":"Independent review performed","attestation_url":"https://example.invalid/attestation"},
            "environment":{"windows_version":"Windows 11 24H2","architecture":"x86_64","disposable":true,"installed_sha256":"e".repeat(64),"privilege":"administrator","network":"loopback-contained","synthetic_data":true,"tools":["review-tool"],"limitations":[],"installed_cases":installed},
            "checks":checks,
            "findings":[],
            "summary":{"conclusion":"no-mechanical-blockers","finding_counts":{"critical":0,"high":0,"medium":0,"low":0,"informational":0},"p1_boundary":"passed","no_open_proxy":"passed","limitations":[],"evidence_sha256":"f".repeat(64)}
        })
    }

    fn assert_invalid(mutator: impl FnOnce(&mut Value)) {
        let candidate = candidate();
        let mut record = record(&candidate);
        mutator(&mut record);
        assert!(!validate_record(&record, &candidate).is_empty());
    }

    #[test]
    fn complete_synthetic_record_has_no_mechanical_blocker() {
        let candidate = candidate();
        assert!(validate_record(&record(&candidate), &candidate).is_empty());
    }

    #[test]
    fn mutations_are_rejected_independently() {
        type Mutation = Box<dyn Fn(&mut Value)>;
        let mutations: Vec<Mutation> = vec![
            Box::new(|v| v["state"] = Value::String("not-started".into())),
            Box::new(|v| v["candidate"]["tag"] = Value::String("v0.0.0".into())),
            Box::new(|v| v["reviewer"]["id"] = Value::String(String::new())),
            Box::new(|v| v["reviewer"]["completed_at"] = v["reviewer"]["started_at"].clone()),
            Box::new(|v| v["independence"]["implementation_contributions"] = Value::Bool(true)),
            Box::new(|v| v["independence"]["conflicts"] = serde_json::json!(["financial"])),
            Box::new(|v| v["environment"]["disposable"] = Value::Bool(false)),
            Box::new(|v| {
                v["environment"]["installed_cases"]
                    .as_array_mut()
                    .unwrap()
                    .pop()
                    .map(|_| ())
                    .unwrap()
            }),
            Box::new(|v| {
                v["environment"]["installed_cases"][0]["outcome"] =
                    Value::String("indeterminate".into())
            }),
            Box::new(|v| {
                v["checks"]
                    .as_array_mut()
                    .unwrap()
                    .pop()
                    .map(|_| ())
                    .unwrap()
            }),
            Box::new(|v| v["checks"][0]["id"] = Value::String("unknown".into())),
            Box::new(|v| v["checks"][0]["method"] = Value::String(String::new())),
            Box::new(|v| v["checks"][0]["evidence"] = serde_json::json!([])),
            Box::new(|v| v["summary"]["p1_boundary"] = Value::String("indeterminate".into())),
            Box::new(|v| v["summary"]["finding_counts"]["high"] = Value::from(1)),
            Box::new(|v| {
                v["summary"]["limitations"] = serde_json::json!(["C:\\Users\\private\\evidence"])
            }),
        ];
        for mutation in mutations {
            assert_invalid(|value| mutation(value));
        }
    }

    #[test]
    fn unresolved_high_and_ownerless_medium_findings_are_rejected() {
        let candidate = candidate();
        let mut high = record(&candidate);
        high["findings"] = serde_json::json!([{
            "id":"F-1","area":"tls","severity":"high","candidate_revision":"a".repeat(40),"reproduction":"method","impact":"impact","evidence":[evidence("finding")],"owner":"owner","linked_issue":"https://example.invalid/issues/1","disposition":"open","remediation":null,"retest":null
        }]);
        high["checks"][4]["findings"] = serde_json::json!(["F-1"]);
        high["summary"]["finding_counts"]["high"] = Value::from(1);
        assert!(!validate_record(&high, &candidate).is_empty());

        let mut medium = record(&candidate);
        medium["findings"] = serde_json::json!([{
            "id":"F-2","area":"packaging","severity":"medium","candidate_revision":"a".repeat(40),"reproduction":"method","impact":"impact","evidence":[evidence("finding")],"owner":"","linked_issue":"https://example.invalid/issues/2","disposition":"accepted-by-owner","remediation":null,"retest":null
        }]);
        medium["checks"][11]["findings"] = serde_json::json!(["F-2"]);
        medium["summary"]["finding_counts"]["medium"] = Value::from(1);
        assert!(!validate_record(&medium, &candidate).is_empty());
    }

    #[test]
    fn high_finding_requires_a_distinct_independent_retest() {
        let candidate = candidate();
        let mut record = record(&candidate);
        record["findings"] = serde_json::json!([{
            "id":"F-3","area":"tls","severity":"high","candidate_revision":"a".repeat(40),"reproduction":"method","impact":"impact","evidence":[evidence("finding")],"owner":"owner","linked_issue":"https://example.invalid/issues/3","disposition":"remediated","remediation":{"fixed_revision":"1".repeat(40),"pull_request":"https://example.invalid/pull/3"},"retest":{"reviewer_id":"reviewer-2","candidate_revision":"1".repeat(40),"outcome":"passed","method":"separate retest","evidence":[evidence("retest")]}
        }]);
        record["checks"][4]["findings"] = serde_json::json!(["F-3"]);
        record["summary"]["finding_counts"]["high"] = Value::from(1);
        assert!(validate_record(&record, &candidate).is_empty());
        record["findings"][0]["retest"]["reviewer_id"] = Value::String("reviewer-1".into());
        assert!(!validate_record(&record, &candidate).is_empty());
    }
}
