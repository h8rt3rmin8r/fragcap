// SPDX-License-Identifier: Apache-2.0

//! Static review readiness, deliberately not an independent audit verdict.

use serde_json::Value;
use std::{collections::BTreeSet, fs, io, path::Path};

const REGISTRY: &str = "docs/security/native-product-review-scope.v1.json";
const TEMPLATE: &str = "docs/security/native-product-review-record.v1.json";
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

pub fn run(root: &Path) -> io::Result<usize> {
    let scope = read(root, REGISTRY)?;
    let template = read(root, TEMPLATE)?;
    let problems = validate(root, &scope, &template);
    for problem in &problems {
        eprintln!("review-handoff: {problem}");
    }
    if problems.is_empty() {
        println!("review-handoff: twelve-area readiness passes; independent review NOT performed");
    }
    Ok(problems.len())
}

fn read(root: &Path, path: &str) -> io::Result<Value> {
    serde_json::from_str(&fs::read_to_string(root.join(path))?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn validate(root: &Path, scope: &Value, template: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    exact_keys(
        scope,
        &["schema_version", "review_state", "areas"],
        "scope",
        &mut problems,
    );
    if scope["schema_version"].as_u64() != Some(1)
        || scope["review_state"].as_str() != Some("not-performed")
    {
        problems.push("scope must declare schema 1 and independent review not-performed".into());
    }
    let mut seen = BTreeSet::new();
    for area in scope["areas"].as_array().into_iter().flatten() {
        exact_keys(
            area,
            &["id", "method", "sources", "tests"],
            "area",
            &mut problems,
        );
        let id = area["id"].as_str().unwrap_or("");
        if !AREAS.contains(&id) || !seen.insert(id) {
            problems.push(format!("unknown or duplicate area {id}"));
        }
        if area["method"]
            .as_str()
            .is_none_or(|text| text.trim().is_empty())
        {
            problems.push(format!("area {id} has no review method"));
        }
        let mut sources = BTreeSet::new();
        if area["sources"].as_array().is_none_or(Vec::is_empty) {
            problems.push(format!("area {id} has no source evidence"));
        }
        for path in area["sources"].as_array().into_iter().flatten() {
            let path = path.as_str().unwrap_or("");
            if !sources.insert(path) {
                problems.push(format!("area {id} duplicates source {path}"));
            }
            if let Err(reason) = confined_source(root, path) {
                problems.push(format!("area {id} source {path}: {reason}"));
            }
        }
        let mut tests = BTreeSet::new();
        if area["tests"].as_array().is_none_or(Vec::is_empty) {
            problems.push(format!("area {id} has no executable test references"));
        }
        for test in area["tests"].as_array().into_iter().flatten() {
            exact_keys(test, &["path", "function"], id, &mut problems);
            let path = test["path"].as_str().unwrap_or("");
            let function = test["function"].as_str().unwrap_or("");
            if !tests.insert((path, function)) {
                problems.push(format!("area {id} duplicates test {path}::{function}"));
            }
            if !path.ends_with(".rs")
                || function.is_empty()
                || !function
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                problems.push(format!(
                    "area {id} has invalid test reference {path}::{function}"
                ));
                continue;
            }
            match confined_source(root, path).and_then(|source| {
                crate::threat_model::validate_test_source(&source, function).map_err(str::to_string)
            }) {
                Ok(()) => {}
                Err(reason) => {
                    problems.push(format!("area {id} test {path}::{function}: {reason}"))
                }
            }
        }
    }
    if seen != AREAS.into_iter().collect() {
        problems.push("review scope must contain exactly the twelve required areas".into());
    }
    let expected = serde_json::json!({"schema_version":1,"state":"not-started","candidate":{"product_version":"0.10.0","source_revision":null,"lockfile_sha256":null,"packages":[]},"reviewer":null,"independence":null,"environment":null,"checks":[],"findings":[],"summary":null});
    if *template != expected {
        problems.push("committed record must remain the exact not-started template with no invented provenance, findings, or approval".into());
    }
    problems
}

fn exact_keys(value: &Value, expected: &[&str], label: &str, problems: &mut Vec<String>) {
    let keys: BTreeSet<_> = value
        .as_object()
        .into_iter()
        .flat_map(|object| object.keys().map(String::as_str))
        .collect();
    if keys != expected.iter().copied().collect() {
        problems.push(format!("{label} keys are not the closed schema"));
    }
}

fn confined_source(root: &Path, path: &str) -> Result<String, String> {
    if path.is_empty()
        || path.contains(['\\', ':'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("not a confined repository-relative path".into());
    }
    let boundary = root.canonicalize().map_err(|error| error.to_string())?;
    let candidate = root
        .join(path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !candidate.starts_with(boundary) {
        return Err("resolved outside repository".into());
    }
    let metadata = candidate.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > 1_048_576 {
        return Err("not a bounded regular evidence file".into());
    }
    fs::read_to_string(candidate).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
    }
    fn fixture() -> (Value, Value) {
        (
            json!({"schema_version":1,"review_state":"not-performed","areas":AREAS.map(|id|json!({"id":id,"method":"Inspect exact source and run controlled evidence independently.","sources":["README.md"],"tests":[{"path":"xtask/src/lint.rs","function":"clean_source_produces_no_findings"}]}))}),
            json!({"schema_version":1,"state":"not-started","candidate":{"product_version":"0.10.0","source_revision":null,"lockfile_sha256":null,"packages":[]},"reviewer":null,"independence":null,"environment":null,"checks":[],"findings":[],"summary":null}),
        )
    }
    #[test]
    fn committed_handoff_is_complete_and_unperformed() {
        assert_eq!(run(root()).unwrap(), 0);
    }
    #[test]
    fn complete_scope_and_unperformed_template_pass() {
        let (scope, template) = fixture();
        assert!(validate(root(), &scope, &template).is_empty());
    }
    #[test]
    fn missing_duplicate_unknown_area_and_unknown_key_fail() {
        let (scope, template) = fixture();
        let mut missing = scope.clone();
        missing["areas"].as_array_mut().unwrap().pop();
        let mut duplicate = scope.clone();
        duplicate["areas"][1]["id"] = json!("architecture");
        let mut unknown = scope.clone();
        unknown["areas"][1]["id"] = json!("guessed-area");
        let mut key = scope;
        key["approved"] = json!(true);
        for changed in [missing, duplicate, unknown, key] {
            assert!(!validate(root(), &changed, &template).is_empty());
        }
    }
    #[test]
    fn absent_or_unsafe_source_and_missing_test_fail() {
        let (scope, template) = fixture();
        for path in [
            "../../outside.md",
            "C:/outside.md",
            "docs/absent-review-evidence.md",
        ] {
            let mut changed = scope.clone();
            changed["areas"][0]["sources"] = json!([path]);
            assert!(!validate(root(), &changed, &template).is_empty());
        }
        let mut changed = scope;
        changed["areas"][0]["tests"][0]["function"] = json!("invented_test");
        assert!(!validate(root(), &changed, &template).is_empty());
    }
    #[test]
    fn ignored_conditional_and_commented_test_evidence_fail() {
        for source in [
            "#[test]\n#[ignore]\nfn check() {}",
            "#[test]\n#[cfg(any())]\nfn check() {}",
            "/* #[test]\nfn check() {} */",
        ] {
            assert!(crate::threat_model::validate_test_source(source, "check").is_err());
        }
    }
    #[test]
    fn invented_completion_or_provenance_fails() {
        let (scope, template) = fixture();
        for (field, value) in [
            ("state", json!("approved")),
            ("reviewer", json!("agent-self-review")),
            ("checks", json!([{"result":"passed"}])),
            ("summary", json!("No findings")),
        ] {
            let mut changed = template.clone();
            changed[field] = value;
            assert!(!validate(root(), &scope, &changed).is_empty());
        }
        let mut changed = template;
        changed["candidate"]["source_revision"] = json!("invented");
        assert!(!validate(root(), &scope, &changed).is_empty());
    }
}
