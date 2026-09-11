// SPDX-License-Identifier: Apache-2.0

//! Guided-calibration parent acceptance and executable-evidence gate.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Component, Path};
use std::process::Command;

use serde_json::Value;

const REGISTRY: &str = "integration/guided-calibration-acceptance-v1.json";
const CI_WORKFLOW: &str = ".github/workflows/ci.yml";
const REQUIRED_CRITERIA: [&str; 13] = [
    "one-target-guidance",
    "supported-topology-parity",
    "explicit-ambiguity-choice",
    "conservative-defaults-and-overrides",
    "reachability-before-trust",
    "complete-plan-visibility",
    "operator-owned-warm-retry",
    "distinct-operator-actions",
    "exact-append-only-facts",
    "coverage-and-deep-capture-handoff",
    "selective-retest",
    "human-json-resume",
    "controlled-and-windows-matrix",
];
const REQUIRED_CASES: [&str; 10] = [
    "steam",
    "direct",
    "publisher",
    "warm-state",
    "ambiguity",
    "partial-evidence",
    "trust-refusal",
    "interruption",
    "cleanup",
    "successful-handoff",
];

pub fn run(root: &Path) -> io::Result<usize> {
    let registry: Value =
        serde_json::from_str(&fs::read_to_string(root.join(REGISTRY))?).map_err(invalid_data)?;
    let tracked = tracked_rust_sources(root)?;
    let mut problems = validate_registry(&registry);
    problems.extend(validate_references(root, &registry, &tracked)?);
    problems.extend(validate_ci_workflow(&fs::read_to_string(
        root.join(CI_WORKFLOW),
    )?));
    for problem in &problems {
        eprintln!("guided-calibration-acceptance: {problem}");
    }
    if problems.is_empty() {
        let tests = evidence_records(&registry).count();
        println!(
            "guided-calibration-acceptance: schema 1, 13 criteria, {tests} executable references, controlled implementation accepted; live game compatibility was not demonstrated"
        );
    }
    Ok(problems.len())
}

fn invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn validate_registry(registry: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    exact_keys(
        registry,
        &[
            "schema_version",
            "issue",
            "reviewed_on",
            "scope",
            "completion_boundary",
            "criteria",
            "controlled_matrix",
        ],
        "registry",
        &mut problems,
    );
    if registry["schema_version"].as_u64() != Some(1) {
        problems.push("schema_version must be 1".to_string());
    }
    if registry["issue"].as_u64() != Some(380) {
        problems.push("issue must be 380".to_string());
    }
    for field in ["reviewed_on", "scope"] {
        require_string(registry, field, "registry", &mut problems);
    }
    validate_completion_boundary(&registry["completion_boundary"], &mut problems);
    validate_records(
        &registry["criteria"],
        "criterion",
        &REQUIRED_CRITERIA,
        &mut problems,
    );
    validate_records(
        &registry["controlled_matrix"],
        "controlled case",
        &REQUIRED_CASES,
        &mut problems,
    );
    problems
}

fn validate_completion_boundary(boundary: &Value, problems: &mut Vec<String>) {
    exact_keys(
        boundary,
        &[
            "implementation",
            "live_game_execution",
            "live_game_blocks_implementation",
            "live_compatibility_claim",
            "sensitive_automation",
            "raw_live_evidence",
        ],
        "completion boundary",
        problems,
    );
    for (field, expected) in [
        ("implementation", "controlled-automated-evidence"),
        ("live_game_execution", "operator-only-published-release"),
        (
            "live_compatibility_claim",
            "not-demonstrated-by-this-acceptance",
        ),
        ("sensitive_automation", "prohibited"),
        ("raw_live_evidence", "private-outside-repository"),
    ] {
        if boundary[field].as_str() != Some(expected) {
            problems.push(format!("completion boundary {field} must be {expected:?}"));
        }
    }
    if boundary["live_game_blocks_implementation"].as_bool() != Some(false) {
        problems
            .push("completion boundary live_game_blocks_implementation must be false".to_string());
    }
}

fn validate_records(value: &Value, kind: &str, required: &[&str], problems: &mut Vec<String>) {
    let Some(records) = value.as_array() else {
        problems.push(format!("registry {kind} records must be an array"));
        return;
    };
    let mut ids = BTreeSet::new();
    for record in records {
        exact_keys(
            record,
            &["id", "statement", "evidence_class", "tests"],
            kind,
            problems,
        );
        let id = record["id"].as_str().unwrap_or("<missing>");
        if !required.contains(&id) {
            problems.push(format!("{kind} has unknown id {id}"));
        } else if !ids.insert(id.to_string()) {
            problems.push(format!("duplicate {kind} id {id}"));
        }
        require_string(record, "statement", id, problems);
        if record["evidence_class"].as_str() != Some("controlled-automated") {
            problems.push(format!(
                "{kind} {id} must use controlled-automated evidence"
            ));
        }
        validate_tests(record, kind, id, problems);
    }
    let expected = required
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    if ids != expected {
        let missing = expected.difference(&ids).cloned().collect::<Vec<_>>();
        let stale = ids.difference(&expected).cloned().collect::<Vec<_>>();
        problems.push(format!(
            "{kind} inventory is incomplete: missing={missing:?}, stale={stale:?}"
        ));
    }
}

fn validate_tests(record: &Value, kind: &str, id: &str, problems: &mut Vec<String>) {
    let Some(tests) = record["tests"].as_array() else {
        problems.push(format!("{kind} {id} tests must be an array"));
        return;
    };
    if tests.is_empty() {
        problems.push(format!("{kind} {id} has no executable evidence"));
    }
    let mut references = BTreeSet::new();
    for test in tests {
        exact_keys(
            test,
            &["path", "function", "platform", "proves"],
            &format!("{kind} {id} test"),
            problems,
        );
        for field in ["path", "function", "proves"] {
            require_string(test, field, &format!("{kind} {id} test"), problems);
        }
        if !matches!(test["platform"].as_str(), Some("portable" | "windows")) {
            problems.push(format!("{kind} {id} test has an invalid platform"));
        }
        let reference = format!(
            "{}::{}",
            test["path"].as_str().unwrap_or_default(),
            test["function"].as_str().unwrap_or_default()
        );
        if !references.insert(reference.clone()) {
            problems.push(format!("{kind} {id} duplicates test {reference}"));
        }
    }
}

fn exact_keys(value: &Value, expected: &[&str], label: &str, problems: &mut Vec<String>) {
    let Some(object) = value.as_object() else {
        problems.push(format!("{label} must be an object"));
        return;
    };
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        problems.push(format!(
            "{label} keys differ: missing={:?}, unknown={:?}",
            expected.difference(&actual).collect::<Vec<_>>(),
            actual.difference(&expected).collect::<Vec<_>>()
        ));
    }
}

fn require_string(value: &Value, field: &str, label: &str, problems: &mut Vec<String>) {
    if value[field].as_str().is_none_or(str::is_empty) {
        problems.push(format!("{label} requires non-empty {field}"));
    }
}

fn evidence_records(registry: &Value) -> impl Iterator<Item = &Value> {
    ["criteria", "controlled_matrix"]
        .into_iter()
        .flat_map(|field| registry[field].as_array().into_iter().flatten())
        .flat_map(|record| record["tests"].as_array().into_iter().flatten())
}

fn validate_references(
    root: &Path,
    registry: &Value,
    tracked: &BTreeSet<String>,
) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    for test in evidence_records(registry) {
        let path = test["path"].as_str().unwrap_or_default();
        let function = test["function"].as_str().unwrap_or_default();
        let platform = test["platform"].as_str().unwrap_or_default();
        if path.is_empty() || function.is_empty() {
            continue;
        }
        let relative = Path::new(path);
        let confined = relative
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
        if !confined || !path.ends_with(".rs") {
            problems.push(format!("test path is not a confined Rust source: {path}"));
            continue;
        }
        if !tracked.contains(&path.replace('\\', "/")) {
            problems.push(format!("test path is not tracked: {path}"));
            continue;
        }
        let source = fs::read_to_string(root.join(relative))?;
        if let Err(reason) = validate_test_source(&source, function, platform) {
            problems.push(format!("{reason}: {path}::{function}"));
        }
    }
    Ok(problems)
}

fn validate_test_source(source: &str, function: &str, platform: &str) -> Result<(), &'static str> {
    if platform == "portable" {
        return super::threat_model::validate_test_source(source, function);
    }
    if platform != "windows" {
        return Err("reference has invalid platform");
    }
    let source = super::threat_model::strip_rust_comments(source);
    let declarations = [format!("fn {function}("), format!("async fn {function}(")];
    let mut attributes = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            attributes.push(trimmed);
            continue;
        }
        if declarations
            .iter()
            .any(|declaration| trimmed.starts_with(declaration))
        {
            if !attributes
                .iter()
                .any(|attribute| *attribute == "#[test]" || attribute.starts_with("#[tokio::test"))
            {
                return Err("reference is not an attributed test");
            }
            if attributes.iter().any(|attribute| {
                attribute.starts_with("#[ignore") || attribute.starts_with("#[cfg_attr")
            }) {
                return Err("reference is ignored or conditionally disabled");
            }
            let cfgs = attributes
                .iter()
                .filter(|attribute| attribute.starts_with("#[cfg"))
                .copied()
                .collect::<Vec<_>>();
            if cfgs != ["#[cfg(windows)]"] {
                return Err("Windows reference must use exactly #[cfg(windows)]");
            }
            return Ok(());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            attributes.clear();
        }
    }
    Err("reference is missing test")
}

fn tracked_rust_sources(root: &Path) -> io::Result<BTreeSet<String>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "--", "crates", "xtask"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("git ls-files failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|path| path.ends_with(".rs"))
        .map(str::to_string)
        .collect())
}

fn validate_ci_workflow(source: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for marker in [
        "windows-latest",
        "cargo test --workspace --locked",
        "cargo run --package xtask -- guided-calibration-acceptance",
    ] {
        if !source.contains(marker) {
            problems.push(format!("CI workflow is missing {marker:?}"));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_ref(function: &str) -> Value {
        json!({
            "path": "crates/demo/tests/demo.rs",
            "function": function,
            "platform": "portable",
            "proves": "a controlled proposition"
        })
    }

    fn valid() -> Value {
        let records = |ids: &[&str]| {
            ids.iter()
                .map(|id| {
                    json!({
                        "id": id,
                        "statement": format!("statement for {id}"),
                        "evidence_class": "controlled-automated",
                        "tests": [test_ref(&format!("test_{}", id.replace('-', "_")))]
                    })
                })
                .collect::<Vec<_>>()
        };
        json!({
            "schema_version": 1,
            "issue": 380,
            "reviewed_on": "2026-09-11",
            "scope": "guided calibration",
            "completion_boundary": {
                "implementation": "controlled-automated-evidence",
                "live_game_execution": "operator-only-published-release",
                "live_game_blocks_implementation": false,
                "live_compatibility_claim": "not-demonstrated-by-this-acceptance",
                "sensitive_automation": "prohibited",
                "raw_live_evidence": "private-outside-repository"
            },
            "criteria": records(&REQUIRED_CRITERIA),
            "controlled_matrix": records(&REQUIRED_CASES)
        })
    }

    #[test]
    fn registry_shape_is_closed_and_complete() {
        assert!(validate_registry(&valid()).is_empty());
        let mut missing = valid();
        missing["criteria"].as_array_mut().unwrap().pop();
        assert!(validate_registry(&missing)
            .iter()
            .any(|problem| problem.contains("inventory is incomplete")));
        let mut duplicate = valid();
        duplicate["controlled_matrix"][1]["id"] = json!(REQUIRED_CASES[0]);
        assert!(validate_registry(&duplicate)
            .iter()
            .any(|problem| problem.contains("duplicate controlled case")));
    }

    #[test]
    fn completion_boundary_cannot_claim_live_compatibility() {
        let mut value = valid();
        value["completion_boundary"]["live_compatibility_claim"] = json!("verified");
        value["completion_boundary"]["sensitive_automation"] = json!("allowed");
        let problems = validate_registry(&value);
        assert!(problems
            .iter()
            .any(|problem| problem.contains("live_compatibility_claim")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("sensitive_automation")));
    }

    #[test]
    fn evidence_rows_reject_empty_duplicate_and_unknown_fields() {
        let mut value = valid();
        value["criteria"][0]["tests"] = json!([]);
        value["criteria"][1]["tests"] = json!([test_ref("same"), test_ref("same")]);
        value["criteria"][2]["unexpected"] = json!(true);
        let problems = validate_registry(&value);
        assert!(problems
            .iter()
            .any(|problem| problem.contains("no executable")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("duplicates test")));
        assert!(problems.iter().any(|problem| problem.contains("unknown")));
    }

    #[test]
    fn portable_and_windows_test_sources_have_exact_execution_attributes() {
        assert!(validate_test_source("#[test]\nfn portable() {}", "portable", "portable").is_ok());
        assert!(validate_test_source(
            "#[test]\n#[cfg(windows)]\nfn hosted() {}",
            "hosted",
            "windows"
        )
        .is_ok());
        assert!(
            validate_test_source("#[test]\n#[ignore]\nfn skipped() {}", "skipped", "portable")
                .is_err()
        );
        assert!(validate_test_source(
            "#[test]\n#[cfg(windows)]\nfn conditional() {}",
            "conditional",
            "portable"
        )
        .is_err());
        assert!(validate_test_source("#[test]\nfn broad() {}", "broad", "windows").is_err());
        assert!(validate_test_source(
            "/*\n#[test]\n#[cfg(windows)]\nfn commented_out() {}\n*/",
            "commented_out",
            "windows"
        )
        .is_err());
    }

    #[test]
    fn ci_workflow_requires_windows_tests_and_the_acceptance_gate() {
        let complete = "windows-latest\ncargo test --workspace --locked\ncargo run --package xtask -- guided-calibration-acceptance";
        assert!(validate_ci_workflow(complete).is_empty());
        assert_eq!(validate_ci_workflow("windows-latest").len(), 2);
    }

    #[test]
    fn repository_registry_satisfies_the_gate() {
        assert_eq!(run(&super::super::repo_root()).unwrap(), 0);
    }
}
