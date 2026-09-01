// SPDX-License-Identifier: Apache-2.0

//! Closed native HTTP and TLS conformance evidence gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

const EVIDENCE: &str = "conformance/native-http-tls";
const PROTOCOLS: [&str; 6] = ["http1", "https", "http2", "websocket", "sse", "grpc"];
const ARTIFACTS: [&str; 10] = [
    "application-jsonl",
    "har",
    "tls-key-log",
    "pcapng",
    "correlation",
    "proxy-lifecycle",
    "cleanup-lifecycle",
    "cleanup-summary",
    "resource-journal",
    "manifest-v2",
];

pub fn run(root: &Path, analyzer: bool) -> io::Result<usize> {
    let matrix = read_json(&root.join(EVIDENCE).join("matrix-v1.json"))?;
    let report = read_json(&root.join(EVIDENCE).join("report-v1.json"))?;
    let mut problems = validate(&matrix, &report);
    problems.extend(validate_evidence_references(root, &matrix)?);
    problems.extend(validate_version_identities(root, &matrix, &report)?);
    problems.extend(validate_fixture_drift(root)?);
    problems.extend(scan_committed_evidence(&report));
    if problems.is_empty() {
        problems.extend(run_portable_harness(root));
    }
    if analyzer {
        if let Err(error) = run_analyzer(root) {
            problems.push(error);
        }
    }
    for problem in &problems {
        eprintln!("conformance: {problem}");
    }
    if problems.is_empty() {
        println!(
            "conformance: {} required rows, six protocols, independent peers, and ten artifact authorities pass",
            required_rows(&matrix)
        );
        if !analyzer {
            println!("conformance: live TShark proof is owned by `conformance --analyzer`");
        }
    }
    Ok(problems.len())
}

fn run_portable_harness(root: &Path) -> Vec<String> {
    let commands: [&[&str]; 2] = [
        &["test", "-p", "fragcap-proxy", "--locked"],
        &[
            "test",
            "-p",
            "fragcap",
            "--features",
            "deep-capture",
            "--locked",
        ],
    ];
    for arguments in commands {
        let status = Command::new(env!("CARGO"))
            .current_dir(root)
            .args(arguments)
            .status();
        if !matches!(status, Ok(value) if value.success()) {
            return vec![format!(
                "portable harness failed: cargo {}",
                arguments.join(" ")
            )];
        }
    }
    Vec::new()
}

fn validate_version_identities(
    root: &Path,
    matrix: &Value,
    report: &Value,
) -> io::Result<Vec<String>> {
    let manifest = fs::read_to_string(root.join("Cargo.toml"))?;
    let lock = fs::read_to_string(root.join("Cargo.lock"))?;
    let workspace_version = manifest
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("version      = \"")?
                .strip_suffix('"')
        })
        .unwrap_or_default();
    let mut problems = Vec::new();
    if matrix["product_version"] != workspace_version
        || report["product_version"] != workspace_version
    {
        problems.push("matrix or report product version differs from the workspace".to_string());
    }
    let Some(implementations) = matrix["implementations"].as_array() else {
        problems.push("matrix implementations must be an array".to_string());
        return Ok(problems);
    };
    for implementation in implementations {
        let id = implementation["id"].as_str().unwrap_or_default();
        let version = implementation["version"].as_str().unwrap_or_default();
        if id == "fragcap-native" && version != workspace_version {
            problems.push(format!(
                "matrix implementation {id} version {version} differs from the workspace"
            ));
        }
        if implementation["transport"] != "async-library" {
            continue;
        }
        for identity in version.split('/') {
            let Some((name, package_version)) = identity.rsplit_once('-') else {
                problems.push(format!(
                    "matrix implementation {id} has invalid package identity {identity}"
                ));
                continue;
            };
            if name.is_empty()
                || package_version.is_empty()
                || !lock.contains(&format!(
                    "name = \"{name}\"\nversion = \"{package_version}\""
                ))
            {
                problems.push(format!(
                    "matrix implementation {id} package {name} {package_version} is not lock-resolved"
                ));
            }
        }
    }
    Ok(problems)
}

fn validate_evidence_references(root: &Path, matrix: &Value) -> io::Result<Vec<String>> {
    let mut source = String::new();
    collect_rust_source(&root.join("crates"), &mut source)?;
    let mut problems = Vec::new();
    for row in matrix["rows"].as_array().into_iter().flatten() {
        let id = row["id"].as_str().unwrap_or_default();
        for reference in row["evidence"].as_array().into_iter().flatten() {
            let reference = reference.as_str().unwrap_or_default();
            let function = reference.rsplit("::").next().unwrap_or_default();
            if function.is_empty() || !source.contains(&format!("fn {function}(")) {
                problems.push(format!(
                    "row {id} references missing executable evidence {reference}"
                ));
            } else if evidence_is_ignored(&source, function) {
                problems.push(format!(
                    "required row {id} references ignored evidence {reference}"
                ));
            }
        }
    }
    Ok(problems)
}

fn evidence_is_ignored(source: &str, function: &str) -> bool {
    let marker = format!("fn {function}(");
    let Some(position) = source.find(&marker) else {
        return false;
    };
    let prefix = &source[..position];
    let boundary = prefix
        .rfind("\nfn ")
        .into_iter()
        .chain(prefix.rfind("\n    fn "))
        .max()
        .unwrap_or_else(|| prefix.len().saturating_sub(256));
    prefix[boundary..].contains("#[ignore]")
}

fn collect_rust_source(path: &Path, target: &mut String) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source(&path, target)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            target.push_str(&fs::read_to_string(path)?);
            target.push('\n');
        }
    }
    Ok(())
}

fn regenerate_analyzer_fixture(source: &str) -> Result<(Vec<u8>, String), String> {
    let mut pcapng = Vec::new();
    push_u32(&mut pcapng, 0x0a0d0d0a);
    push_u32(&mut pcapng, 28);
    push_u32(&mut pcapng, 0x1a2b3c4d);
    pcapng.extend_from_slice(&1_u16.to_le_bytes());
    pcapng.extend_from_slice(&0_u16.to_le_bytes());
    pcapng.extend_from_slice(&u64::MAX.to_le_bytes());
    push_u32(&mut pcapng, 28);
    push_u32(&mut pcapng, 1);
    push_u32(&mut pcapng, 20);
    pcapng.extend_from_slice(&1_u16.to_le_bytes());
    pcapng.extend_from_slice(&0_u16.to_le_bytes());
    push_u32(&mut pcapng, 65_535);
    push_u32(&mut pcapng, 20);

    let mut key_log =
        String::from("# Synthetic TLS 1.3 secrets for the committed S110 analyzer fixture\n");
    let mut packets = 0_usize;
    let mut secrets = 0_usize;
    for (index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("packet ") {
            let mut fields = rest.split_whitespace();
            let timestamp = parse_source_number(fields.next(), index, "timestamp")?;
            let original_length = parse_source_number(fields.next(), index, "original length")?;
            let packet = decode_hex(fields.next().ok_or_else(|| {
                format!("analyzer source line {} lacks packet bytes", index + 1)
            })?)?;
            if fields.next().is_some() || packet.len() != original_length as usize {
                return Err(format!(
                    "analyzer source line {} has inconsistent packet length",
                    index + 1
                ));
            }
            let padded_length = (packet.len() + 3) & !3;
            let block_length = (32 + padded_length) as u32;
            push_u32(&mut pcapng, 6);
            push_u32(&mut pcapng, block_length);
            push_u32(&mut pcapng, 0);
            push_u32(&mut pcapng, (timestamp >> 32) as u32);
            push_u32(&mut pcapng, timestamp as u32);
            push_u32(&mut pcapng, packet.len() as u32);
            push_u32(&mut pcapng, original_length as u32);
            pcapng.extend_from_slice(&packet);
            pcapng.resize(pcapng.len() + padded_length - packet.len(), 0);
            push_u32(&mut pcapng, block_length);
            packets += 1;
        } else if let Some(secret) = line.strip_prefix("keylog ") {
            if secret.split_whitespace().count() != 3 {
                return Err(format!(
                    "analyzer source line {} has an invalid key-log entry",
                    index + 1
                ));
            }
            key_log.push_str(secret);
            key_log.push('\n');
            secrets += 1;
        } else {
            return Err(format!("unknown analyzer source line {}", index + 1));
        }
    }
    if packets != 9 || secrets != 5 {
        return Err(format!(
            "analyzer source has {packets} packets and {secrets} secrets; expected 9 and 5"
        ));
    }
    Ok((pcapng, key_log))
}

fn parse_source_number(value: Option<&str>, index: usize, field: &str) -> Result<u64, String> {
    value
        .ok_or_else(|| format!("analyzer source line {} lacks {field}", index + 1))?
        .parse()
        .map_err(|_| format!("analyzer source line {} has invalid {field}", index + 1))
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("analyzer source contains odd-length packet hex".to_string());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).expect("hex source is ASCII");
            u8::from_str_radix(digits, 16)
                .map_err(|_| "analyzer source contains non-hex packet bytes".to_string())
        })
        .collect()
}

fn push_u32(target: &mut Vec<u8>, value: u32) {
    target.extend_from_slice(&value.to_le_bytes());
}

fn validate_fixture_drift(root: &Path) -> io::Result<Vec<String>> {
    let committed = fs::read(root.join(EVIDENCE).join("analyzer.pcapng"))?;
    let key_log = fs::read_to_string(root.join(EVIDENCE).join("tls-keylog.log"))?;
    let source = fs::read_to_string(root.join(EVIDENCE).join("analyzer-source-v1.txt"))?;
    let (regenerated, regenerated_key_log) = regenerate_analyzer_fixture(&source)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut problems = Vec::new();
    if committed != regenerated {
        problems
            .push("analyzer.pcapng differs from its deterministic transcript source".to_string());
    }
    if key_log != regenerated_key_log {
        problems
            .push("tls-keylog.log differs from its deterministic transcript source".to_string());
    }
    Ok(problems)
}

fn read_json(path: &Path) -> io::Result<Value> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} has a UTF-8 BOM", path.display()),
        ));
    }
    serde_json::from_slice(&bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} is not JSON: {error}", path.display()),
        )
    })
}

fn required_rows(matrix: &Value) -> usize {
    matrix["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter(|row| row["required"].as_bool() == Some(true))
                .count()
        })
        .unwrap_or_default()
}

fn validate(matrix: &Value, report: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    if matrix["schema_version"] != 1 || report["schema_version"] != 1 {
        problems.push("matrix and report schema_version must be 1".to_string());
    }
    if matrix["matrix_id"] != report["matrix_id"] {
        problems.push("report does not identify the committed matrix".to_string());
    }

    let Some(implementations) = matrix["implementations"].as_array() else {
        problems.push("matrix implementations must be an array".to_string());
        return problems;
    };
    let mut identities = BTreeMap::<String, (&str, &str)>::new();
    for implementation in implementations {
        let Some(id) = implementation["id"].as_str() else {
            problems.push("implementation without id".to_string());
            continue;
        };
        let role = implementation["role"].as_str().unwrap_or_default();
        let lineage = implementation["driver_lineage"]
            .as_str()
            .unwrap_or_default();
        let version = implementation["version"].as_str().unwrap_or_default();
        if role.is_empty() || lineage.is_empty() || version.is_empty() {
            problems.push(format!(
                "implementation {id} lacks role, lineage, or exact version"
            ));
        }
        if identities.insert(id.to_string(), (role, lineage)).is_some() {
            problems.push(format!("duplicate implementation id {id}"));
        }
    }

    let matrix_rows = matrix["rows"].as_array().cloned().unwrap_or_default();
    let report_rows = report["rows"].as_array().cloned().unwrap_or_default();
    let mut declared = BTreeMap::<String, &Value>::new();
    for row in &matrix_rows {
        let id = row["id"].as_str().unwrap_or_default();
        if id.is_empty() {
            problems.push("matrix row without id".to_string());
        } else if declared.insert(id.to_string(), row).is_some() {
            problems.push(format!("duplicate matrix row {id}"));
        }
    }
    let mut observed = BTreeMap::<String, &Value>::new();
    for row in &report_rows {
        let id = row["id"].as_str().unwrap_or_default();
        if id.is_empty() || observed.insert(id.to_string(), row).is_some() {
            problems.push(format!("duplicate or empty report row {id}"));
        }
    }

    let mut clients = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut origins = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut failures = BTreeSet::new();
    for (id, row) in &declared {
        let required = row["required"].as_bool() == Some(true);
        let protocol = row["protocol"].as_str().unwrap_or_default();
        let case = row["case"].as_str().unwrap_or_default();
        let standards_ok = row["standards"]
            .as_array()
            .is_some_and(|items| !items.is_empty());
        if required && (!PROTOCOLS.contains(&protocol) || !standards_ok) {
            problems.push(format!(
                "required row {id} lacks a known protocol or standard"
            ));
        }
        let Some(result) = observed.get(id) else {
            if required {
                problems.push(format!("required row {id} is missing from report"));
            }
            continue;
        };
        if required && result["status"] != "pass" {
            problems.push(format!(
                "required row {id} has non-pass status {}",
                result["status"]
            ));
        }
        if result["observed"] != row["expected"] {
            problems.push(format!(
                "row {id} observed result differs from expected result"
            ));
        }
        if required && case == "positive" && result["status"] == "pass" {
            for (field, role, set) in [
                ("client_id", "client", &mut clients),
                ("origin_id", "origin", &mut origins),
            ] {
                let reference = row[field].as_str().unwrap_or_default();
                match identities.get(reference) {
                    Some((actual_role, lineage)) if *actual_role == role => {
                        set.entry(protocol).or_default().insert(*lineage);
                    }
                    _ => problems.push(format!("row {id} has invalid {field} {reference}")),
                }
            }
        }
        if required && case != "positive" && result["status"] == "pass" {
            failures.insert(case);
        }
    }
    for id in observed.keys() {
        if !declared.contains_key(id) {
            problems.push(format!("report contains undeclared row {id}"));
        }
    }
    for protocol in PROTOCOLS {
        if clients.get(protocol).map_or(0, BTreeSet::len) < 2 {
            problems.push(format!("{protocol} has fewer than two independent clients"));
        }
        if origins.get(protocol).map_or(0, BTreeSet::len) < 2 {
            problems.push(format!("{protocol} has fewer than two independent origins"));
        }
    }
    for case in [
        "authentication-refusal",
        "malformed-framing",
        "wrong-name",
        "untrusted-chain",
        "disconnect",
        "timeout",
        "cancellation",
        "cleanup-failure",
    ] {
        if !failures.contains(case) {
            problems.push(format!("required failure case {case} is absent"));
        }
    }

    let artifact_results = report["artifacts"].as_array().cloned().unwrap_or_default();
    let passing_artifacts: BTreeSet<_> = artifact_results
        .iter()
        .filter(|item| item["status"] == "pass")
        .filter_map(|item| item["role"].as_str())
        .collect();
    for role in ARTIFACTS {
        if !passing_artifacts.contains(role) {
            problems.push(format!("artifact authority {role} did not pass"));
        }
    }
    let required = required_rows(matrix) as u64;
    if report["summary"]["required"] != required
        || report["summary"]["passed"] != required
        || report["summary"]["failed"] != 0
        || report["summary"]["skipped"] != 0
        || report["summary"]["not_run"] != 0
        || report["summary"]["missing"] != 0
        || report["summary"]["duplicate"] != 0
    {
        problems.push("report summary does not reconcile with required rows".to_string());
    }
    problems
}

fn scan_committed_evidence(report: &Value) -> Vec<String> {
    let text = report.to_string();
    [
        "Proxy-Authorization",
        "Basic ",
        "BEGIN PRIVATE KEY",
        "C:\\\\Users\\\\",
        "A:\\\\",
        "authorization_secret",
    ]
    .into_iter()
    .filter(|needle| text.contains(needle))
    .map(|needle| format!("committed report contains prohibited material {needle:?}"))
    .collect()
}

fn run_analyzer(root: &Path) -> Result<(), String> {
    let directory = root.join(EVIDENCE);
    let version = Command::new("tshark")
        .arg("--version")
        .output()
        .map_err(|error| format!("TShark is required for analyzer mode: {error}"))?;
    if !version.status.success() {
        return Err("TShark version probe failed".to_string());
    }
    let output = Command::new("tshark")
        .current_dir(root)
        .args([
            "-r",
            &directory.join("analyzer.pcapng").to_string_lossy(),
            "-o",
            &format!(
                "tls.keylog_file:{}",
                directory.join("tls-keylog.log").display()
            ),
            "-Y",
            "http.request",
            "-T",
            "fields",
            "-e",
            "frame.number",
            "-e",
            "tls.record.version",
            "-e",
            "http.request.method",
            "-e",
            "http.host",
        ])
        .output()
        .map_err(|error| format!("could not execute TShark: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    validate_analyzer_output(
        output.status.success(),
        &stdout,
        &String::from_utf8_lossy(&output.stderr),
    )?;
    println!(
        "conformance: analyzer passed with {}",
        String::from_utf8_lossy(&version.stdout)
            .lines()
            .next()
            .unwrap_or("TShark version unavailable")
    );
    Ok(())
}

fn validate_analyzer_output(success: bool, stdout: &str, stderr: &str) -> Result<(), String> {
    if !success {
        return Err(format!(
            "TShark rejected analyzer fixtures: {}",
            stderr.trim()
        ));
    }
    let decrypted = stdout.lines().any(|line| {
        let fields = line.split('\t').collect::<Vec<_>>();
        fields.len() >= 4
            && !fields[0].is_empty()
            && !fields[1].is_empty()
            && fields[2] == "GET"
            && fields[3] == "s110.invalid"
    });
    if !decrypted {
        return Err(
            "TShark did not decrypt the synthetic GET for s110.invalid with the key log"
                .to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> (Value, Value) {
        let root = super::super::repo_root();
        (
            read_json(&root.join(EVIDENCE).join("matrix-v1.json")).unwrap(),
            read_json(&root.join(EVIDENCE).join("report-v1.json")).unwrap(),
        )
    }

    #[test]
    fn committed_evidence_is_complete() {
        let (matrix, report) = fixtures();
        assert_eq!(validate(&matrix, &report), Vec::<String>::new());
        assert_eq!(scan_committed_evidence(&report), Vec::<String>::new());
    }

    #[test]
    fn skipped_required_row_is_never_a_pass() {
        let (matrix, mut report) = fixtures();
        report["rows"][0]["status"] = Value::String("skip".to_string());
        assert!(validate(&matrix, &report)
            .iter()
            .any(|problem| problem.contains("non-pass status")));
    }

    #[test]
    fn an_alias_does_not_satisfy_independence() {
        let (mut matrix, report) = fixtures();
        let lineage = matrix["implementations"][0]["driver_lineage"].clone();
        matrix["implementations"][1]["driver_lineage"] = lineage;
        assert!(validate(&matrix, &report)
            .iter()
            .any(|problem| problem.contains("fewer than two independent clients")));
    }

    #[test]
    fn missing_required_row_is_named() {
        let (matrix, mut report) = fixtures();
        report["rows"].as_array_mut().unwrap().remove(0);
        assert!(validate(&matrix, &report)
            .iter()
            .any(|problem| problem.contains("missing from report")));
    }

    #[test]
    fn every_committed_evidence_reference_resolves_to_a_test() {
        let (matrix, _) = fixtures();
        let root = super::super::repo_root();
        assert_eq!(
            validate_evidence_references(&root, &matrix).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn analyzer_fixture_is_a_dedicated_tls_transcript() {
        let root = super::super::repo_root();
        assert_eq!(validate_fixture_drift(&root).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn analyzer_exit_zero_without_packets_or_protocol_is_not_a_pass() {
        assert!(validate_analyzer_output(true, "", "")
            .unwrap_err()
            .contains("did not decrypt"));
        assert!(validate_analyzer_output(true, "1\t0x0303\t\t\n", "")
            .unwrap_err()
            .contains("did not decrypt"));
        assert!(validate_analyzer_output(false, "", "broken capture")
            .unwrap_err()
            .contains("broken capture"));
        validate_analyzer_output(true, "8\t0x0303\tGET\ts110.invalid\n", "").unwrap();
    }

    #[test]
    fn matrix_versions_are_lock_resolved() {
        let (matrix, report) = fixtures();
        let root = super::super::repo_root();
        assert_eq!(
            validate_version_identities(&root, &matrix, &report).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_stale_matrix_package_version_is_rejected() {
        let (mut matrix, report) = fixtures();
        matrix["implementations"][1]["version"] =
            Value::String("tokio-0.0.0/hyper-1.11.1/h2-0.4.19".to_string());
        let root = super::super::repo_root();
        assert!(validate_version_identities(&root, &matrix, &report)
            .unwrap()
            .iter()
            .any(|problem| problem.contains("tokio 0.0.0 is not lock-resolved")));
    }

    #[test]
    fn an_ignored_required_evidence_function_is_detected() {
        assert!(evidence_is_ignored(
            "#[test]\n#[ignore]\nfn required_row() {}\n",
            "required_row"
        ));
        assert!(!evidence_is_ignored(
            "#[test]\nfn required_row() {}\n",
            "required_row"
        ));
    }
}
