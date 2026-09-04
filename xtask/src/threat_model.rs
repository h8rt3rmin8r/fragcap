// SPDX-License-Identifier: Apache-2.0

//! Native Deep Capture threat-model and executable-evidence gate.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Component;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

const REGISTRY: &str = "docs/security/deep-capture-threats.v1.json";
const CLASSIFICATION: &str = "crates/fragcap/src/deep_capture/classification.rs";
const PROXY_MANIFEST: &str = "crates/fragcap-proxy/Cargo.toml";
const REQUIRED_CATEGORIES: [&str; 10] = [
    "unrelated-local-client",
    "malicious-target-or-origin",
    "dns-rebinding",
    "ssrf",
    "request-smuggling-desync",
    "resource-exhaustion",
    "certificate-abuse",
    "artifact-theft",
    "cleanup-interruption",
    "confused-deputy",
];

pub fn run(root: &Path) -> io::Result<usize> {
    let registry: Value = serde_json::from_str(&fs::read_to_string(root.join(REGISTRY))?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let protocols = protocol_inventory(&fs::read_to_string(root.join(CLASSIFICATION))?);
    let dependencies = dependency_inventory(&fs::read_to_string(root.join(PROXY_MANIFEST))?);
    let tracked = tracked_rust_sources(root)?;
    let mut problems = validate_registry(&registry, &protocols, &dependencies);
    problems.extend(validate_test_references(root, &registry, &tracked)?);
    for problem in &problems {
        eprintln!("threat-model: {problem}");
    }
    if problems.is_empty() {
        let threats = registry["threats"].as_array().map_or(0, Vec::len);
        let tests = ["threats", "protocol_reviews"]
            .into_iter()
            .flat_map(|field| registry[field].as_array().into_iter().flatten())
            .map(|record| record["tests"].as_array().map_or(0, Vec::len))
            .sum::<usize>();
        println!(
            "threat-model: schema {}, {threats} threats, {tests} executable tests, {} protocols, and {} direct proxy dependencies pass",
            registry["schema_version"],
            protocols.len(),
            dependencies.len()
        );
    }
    Ok(problems.len())
}

fn validate_registry(
    registry: &Value,
    protocols: &BTreeSet<String>,
    dependencies: &BTreeSet<String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    if registry["schema_version"].as_u64() != Some(1) {
        problems.push("schema_version must be 1".to_string());
    }
    for field in ["reviewed_on", "scope"] {
        require_string(registry, field, "registry", &mut problems);
    }
    compare_inventory(registry, "protocol_families", protocols, &mut problems);
    compare_inventory(registry, "proxy_dependencies", dependencies, &mut problems);

    let boundaries = validate_named_records(registry, "boundaries", &mut problems);
    let assets = validate_named_records(registry, "assets", &mut problems);
    let mut threat_ids = BTreeSet::new();
    let mut categories = BTreeSet::new();
    let Some(threats) = registry["threats"].as_array() else {
        problems.push("registry threats must be an array".to_string());
        return problems;
    };
    for threat in threats {
        let id = threat["id"].as_str().unwrap_or("<missing>");
        if !valid_id(id) {
            problems.push(format!("threat {id} has an invalid id"));
        } else if !threat_ids.insert(id.to_string()) {
            problems.push(format!("duplicate threat id {id}"));
        }
        require_string(threat, "title", id, &mut problems);
        if !matches!(threat["severity"].as_str(), Some("high" | "medium" | "low")) {
            problems.push(format!("threat {id} has an invalid severity"));
        }
        for (field, known) in [("boundaries", &boundaries), ("assets", &assets)] {
            for value in string_array(threat, field, id, &mut problems) {
                if !known.contains(&value) {
                    problems.push(format!(
                        "threat {id} references unknown {field} value {value}"
                    ));
                }
            }
        }
        for category in string_array(threat, "categories", id, &mut problems) {
            if !REQUIRED_CATEGORIES.contains(&category.as_str()) {
                problems.push(format!("threat {id} has unknown category {category}"));
            }
            categories.insert(category);
        }
        for field in ["prevention", "detection", "containment", "evidence"] {
            let _ = string_array(threat, field, id, &mut problems);
        }
        let tests = threat["tests"].as_array();
        if threat["severity"].as_str() == Some("high") && tests.is_none_or(Vec::is_empty) {
            problems.push(format!("high-risk threat {id} has no executable tests"));
        }
        if threat.get("residual_risk").is_some() {
            problems.push(format!(
                "threat {id} carries residual risk without operator acceptance"
            ));
        }
        let mut refs = BTreeSet::new();
        for test in tests.into_iter().flatten() {
            let path = test["path"].as_str().unwrap_or_default();
            let function = test["function"].as_str().unwrap_or_default();
            require_string(test, "proves", id, &mut problems);
            if path.is_empty() || function.is_empty() {
                problems.push(format!("threat {id} has an incomplete test reference"));
            } else if !refs.insert(format!("{path}::{function}")) {
                problems.push(format!("threat {id} duplicates test {path}::{function}"));
            }
        }
    }
    for required in REQUIRED_CATEGORIES {
        if !categories.contains(required) {
            problems.push(format!("required threat category {required} is absent"));
        }
    }
    validate_protocol_reviews(registry, protocols, &threat_ids, &mut problems);
    problems
}

fn validate_protocol_reviews(
    registry: &Value,
    protocols: &BTreeSet<String>,
    threat_ids: &BTreeSet<String>,
    problems: &mut Vec<String>,
) {
    let Some(reviews) = registry["protocol_reviews"].as_array() else {
        problems.push("registry protocol_reviews must be an array".to_string());
        return;
    };
    let mut reviewed = BTreeSet::new();
    for review in reviews {
        let family = review["family"].as_str().unwrap_or("<missing>");
        if !protocols.contains(family) {
            problems.push(format!("protocol review names unknown family {family}"));
        } else if !reviewed.insert(family.to_string()) {
            problems.push(format!("duplicate protocol review for {family}"));
        }
        for threat in string_array(review, "threats", family, problems) {
            if !threat_ids.contains(&threat) {
                problems.push(format!(
                    "protocol review {family} references unknown threat {threat}"
                ));
            }
        }
        if review["tests"].as_array().is_none_or(Vec::is_empty) {
            problems.push(format!(
                "protocol review {family} has no executable abuse-case test"
            ));
        }
    }
    if reviewed != *protocols {
        let missing = protocols.difference(&reviewed).cloned().collect::<Vec<_>>();
        problems.push(format!(
            "protocol review coverage is incomplete: missing={missing:?}"
        ));
    }
}

fn validate_named_records(
    registry: &Value,
    field: &str,
    problems: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(records) = registry[field].as_array() else {
        problems.push(format!("registry {field} must be an array"));
        return ids;
    };
    for record in records {
        let id = record["id"].as_str().unwrap_or("<missing>");
        if !valid_id(id) {
            problems.push(format!("{field} record {id} has an invalid id"));
        } else if !ids.insert(id.to_string()) {
            problems.push(format!("duplicate {field} id {id}"));
        }
        require_string(record, "description", id, problems);
        require_string(record, "owner", id, problems);
    }
    ids
}

fn validate_test_references(
    root: &Path,
    registry: &Value,
    tracked: &BTreeSet<String>,
) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    for (kind, records) in [
        ("threat", registry["threats"].as_array()),
        ("protocol review", registry["protocol_reviews"].as_array()),
    ] {
        for record in records.into_iter().flatten() {
            let id = record["id"]
                .as_str()
                .or_else(|| record["family"].as_str())
                .unwrap_or("<missing>");
            for test in record["tests"].as_array().into_iter().flatten() {
                let path = test["path"].as_str().unwrap_or_default();
                let function = test["function"].as_str().unwrap_or_default();
                if path.is_empty() || function.is_empty() {
                    continue;
                }
                let relative = Path::new(path);
                let confined = relative
                    .components()
                    .all(|component| matches!(component, Component::Normal(_) | Component::CurDir));
                if !confined || !path.ends_with(".rs") {
                    problems.push(format!(
                        "{kind} {id} test path is not a repository Rust source: {path}"
                    ));
                    continue;
                }
                if !tracked.contains(&path.replace('\\', "/")) {
                    problems.push(format!("{kind} {id} test path is not tracked: {path}"));
                    continue;
                }
                let candidate = root.join(relative);
                let source = match fs::read_to_string(&candidate) {
                    Ok(source) => source,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        problems.push(format!("{kind} {id} references missing test path {path}"));
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                if let Err(reason) = validate_test_source(&source, function) {
                    problems.push(format!("{kind} {id} {reason}: {path}::{function}"));
                }
            }
        }
    }
    Ok(problems)
}

fn validate_test_source(source: &str, function: &str) -> Result<(), &'static str> {
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
                attribute.starts_with("#[ignore")
                    || attribute.starts_with("#[cfg")
                    || attribute.starts_with("#[cfg_attr")
            }) {
                return Err("references ignored or conditionally disabled test");
            }
            return Ok(());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            attributes.clear();
        }
    }
    Err("references missing test")
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

fn compare_inventory(
    registry: &Value,
    field: &str,
    actual: &BTreeSet<String>,
    problems: &mut Vec<String>,
) {
    let expected = registry[field]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if expected != *actual {
        let missing = actual.difference(&expected).cloned().collect::<Vec<_>>();
        let stale = expected.difference(actual).cloned().collect::<Vec<_>>();
        problems.push(format!(
            "{field} review drift: unreviewed={missing:?}, stale={stale:?}"
        ));
    }
}

fn protocol_inventory(source: &str) -> BTreeSet<String> {
    let body = source
        .split("pub fn as_str(self)")
        .nth(1)
        .and_then(|tail| tail.split("pub fn from_proxy_label").next())
        .unwrap_or_default();
    body.lines()
        .filter_map(|line| line.split("=>").nth(1))
        .filter_map(|tail| tail.split('"').nth(1))
        .map(str::to_string)
        .collect()
}

fn dependency_inventory(source: &str) -> BTreeSet<String> {
    let mut active = false;
    let mut dependencies = BTreeSet::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            let table = trimmed.trim_matches(['[', ']']);
            active = table == "dependencies"
                || (table.starts_with("target.") && table.ends_with(".dependencies"));
            if let Some(name) = table.strip_prefix("dependencies.").or_else(|| {
                table
                    .strip_prefix("target.")
                    .and_then(|tail| tail.split_once(".dependencies.").map(|(_, name)| name))
            }) {
                dependencies.insert(name.trim_matches('"').to_string());
            }
            continue;
        }
        if active && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some((name, _)) = trimmed.split_once('=') {
                dependencies.insert(
                    name.trim()
                        .split('.')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                );
            }
        }
    }
    dependencies
}

fn require_string(value: &Value, field: &str, owner: &str, problems: &mut Vec<String>) {
    if value[field].as_str().is_none_or(str::is_empty) {
        problems.push(format!("{owner} field {field} must be a nonempty string"));
    }
}

fn string_array(
    value: &Value,
    field: &str,
    owner: &str,
    problems: &mut Vec<String>,
) -> Vec<String> {
    let Some(items) = value[field].as_array() else {
        problems.push(format!(
            "{owner} field {field} must be a nonempty string array"
        ));
        return Vec::new();
    };
    let strings = items
        .iter()
        .filter_map(Value::as_str)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if strings.len() != items.len() || strings.is_empty() {
        problems.push(format!(
            "{owner} field {field} must be a nonempty string array"
        ));
    }
    strings
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid() -> Value {
        json!({
            "schema_version": 1,
            "reviewed_on": "2026-09-04",
            "scope": "test",
            "protocol_families": ["http1"],
            "proxy_dependencies": ["tokio"],
            "boundaries": [{"id":"client","description":"d","owner":"o"}],
            "assets": [{"id":"capability","description":"d","owner":"o"}],
            "threats": [{
                "id":"threat", "title":"t", "severity":"high",
                "categories": REQUIRED_CATEGORIES,
                "boundaries":["client"], "assets":["capability"],
                "prevention":["p"], "detection":["d"],
                "containment":["c"], "evidence":["e"],
                "tests":[{"path":"test.rs","function":"rejects","proves":"p"}]
            }],
            "protocol_reviews": [{
                "family":"http1", "threats":["threat"],
                "tests":[{"path":"test.rs","function":"rejects","proves":"p"}]
            }]
        })
    }

    #[test]
    fn incomplete_high_risk_rows_and_unknown_references_fail() {
        let mut registry = valid();
        registry["threats"][0]["prevention"] = json!([]);
        registry["threats"][0]["assets"] = json!(["unknown"]);
        registry["threats"][0]["tests"] = json!([]);
        let problems = validate_registry(
            &registry,
            &BTreeSet::from(["http1".to_string()]),
            &BTreeSet::from(["tokio".to_string()]),
        );
        assert!(problems.iter().any(|p| p.contains("prevention")));
        assert!(problems.iter().any(|p| p.contains("unknown assets")));
        assert!(problems.iter().any(|p| p.contains("no executable tests")));
    }

    #[test]
    fn protocol_and_dependency_drift_fail() {
        let problems = validate_registry(
            &valid(),
            &BTreeSet::from(["http3".to_string()]),
            &BTreeSet::from(["rustls".to_string()]),
        );
        assert!(problems
            .iter()
            .any(|p| p.contains("protocol_families review drift")));
        assert!(problems
            .iter()
            .any(|p| p.contains("proxy_dependencies review drift")));
    }

    #[test]
    fn source_inventories_are_closed_and_sorted() {
        let protocols = protocol_inventory(
            "pub fn as_str(self) { Self::Http => \"http\", } pub fn from_proxy_label() {}",
        );
        assert_eq!(protocols, BTreeSet::from(["http".to_string()]));
        let dependencies = dependency_inventory(
            "[dependencies]\ntokio.workspace = true\n[dev-dependencies]\ntempfile.workspace = true\n[target.'cfg(windows)'.dependencies]\nwindows-sys.workspace = true\n",
        );
        assert_eq!(
            dependencies,
            BTreeSet::from(["tokio".to_string(), "windows-sys".to_string()])
        );
    }

    #[test]
    fn missing_unattributed_and_ignored_test_sources_fail() {
        assert_eq!(
            validate_test_source("#[test]\nfn present() {}", "missing"),
            Err("references missing test")
        );
        assert_eq!(
            validate_test_source("fn helper() {}", "helper"),
            Err("reference is not an attributed test")
        );
        assert_eq!(
            validate_test_source("#[test]\n#[ignore]\nfn skipped() {}", "skipped"),
            Err("references ignored or conditionally disabled test")
        );
        assert_eq!(
            validate_test_source(
                "#[test]\n#[cfg(windows)]\nfn conditional() {}",
                "conditional"
            ),
            Err("references ignored or conditionally disabled test")
        );
        assert_eq!(
            validate_test_source("#[test]\nfn neighbor() {}\n// fn ghost() {}", "ghost"),
            Err("references missing test")
        );
    }

    #[test]
    fn every_protocol_requires_its_own_threat_and_test_mapping() {
        let mut registry = valid();
        registry["protocol_reviews"] = json!([]);
        let problems = validate_registry(
            &registry,
            &BTreeSet::from(["http1".to_string()]),
            &BTreeSet::from(["tokio".to_string()]),
        );
        assert!(problems
            .iter()
            .any(|problem| problem.contains("missing=[\"http1\"]")));
    }

    #[test]
    fn dependency_subtables_are_part_of_the_review_inventory() {
        let dependencies = dependency_inventory(
            "[dependencies.tokio]\nworkspace = true\n[target.'cfg(windows)'.dependencies.windows-sys]\nworkspace = true\n[dev-dependencies.tempfile]\nworkspace = true\n",
        );
        assert_eq!(
            dependencies,
            BTreeSet::from(["tokio".to_string(), "windows-sys".to_string()])
        );
    }
}
