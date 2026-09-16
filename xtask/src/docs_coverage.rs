// SPDX-License-Identifier: Apache-2.0

//! Closed native documentation traceability, not independent review acceptance.

use serde_json::Value;
use std::{collections::BTreeSet, fs, io, path::Path};

const REGISTRY: &str = "docs/audits/native-documentation-coverage.v1.json";
const TOPICS: [&str; 11] = [
    "architecture",
    "setup",
    "cli",
    "protocols-routing",
    "refusals",
    "artifacts-correlation",
    "diagnostics-recovery",
    "security-privacy",
    "bounds",
    "packaging-migration",
    "public-api",
];

pub fn run(root: &Path) -> io::Result<Vec<String>> {
    let inventory: Value = serde_json::from_str(&fs::read_to_string(root.join(REGISTRY))?)
        .map_err(io::Error::other)?;
    let mut problems = validate(root, &inventory);
    if problems.is_empty() {
        let scope = serde_json::json!({"areas": inventory["topics"]});
        problems.extend(crate::review_handoff::discover_references(root, &scope)?);
    }
    Ok(problems)
}

fn exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}

fn confined_text(root: &Path, path: &str) -> Result<String, String> {
    if path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
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
        return Err("not a bounded regular file".into());
    }
    fs::read_to_string(candidate).map_err(|error| error.to_string())
}

fn validate(root: &Path, inventory: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    if !exact_keys(inventory, &["schema_version", "topics"])
        || inventory["schema_version"].as_u64() != Some(1)
    {
        problems.push("inventory must declare exactly schema_version 1 and topics".into());
    }
    let mut seen = BTreeSet::new();
    for topic in inventory["topics"].as_array().into_iter().flatten() {
        let id = topic["id"].as_str().unwrap_or("");
        if !TOPICS.contains(&id) || !seen.insert(id) {
            problems.push(format!("unknown or duplicate topic {id}"));
        }
        if !exact_keys(topic, &["id", "page", "tests"]) {
            problems.push(format!("topic {id}: expected exactly id, page and tests"));
        }
        let page = topic["page"].as_str().unwrap_or("");
        if !page.starts_with("site/content/docs/")
            || !page.ends_with(".mdx")
            || page
                .split('/')
                .any(|part| matches!(part, "changelog" | "glossary"))
        {
            problems.push(format!("topic {id}: not a current authored page: {page}"));
        } else if let Err(reason) = confined_text(root, page) {
            problems.push(format!("topic {id} page {page}: {reason}"));
        }
        if topic["tests"].as_array().is_none_or(Vec::is_empty) {
            problems.push(format!("topic {id}: no executable test authorities"));
        }
        let mut tests = BTreeSet::new();
        for test in topic["tests"].as_array().into_iter().flatten() {
            let path = test["path"].as_str().unwrap_or("");
            let function = test["function"].as_str().unwrap_or("");
            if !exact_keys(test, &["path", "function", "features"])
                || !tests.insert((path, function))
            {
                problems.push(format!("topic {id}: malformed or duplicate test authority"));
            }
            if !(path.starts_with("crates/") || path.starts_with("xtask/src/"))
                || !path.ends_with(".rs")
                || function.is_empty()
                || !function
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                problems.push(format!(
                    "topic {id}: invalid test reference {path}::{function}"
                ));
                continue;
            }
            match confined_text(root, path) {
                Ok(source) => {
                    if let Err(reason) =
                        crate::threat_model::validate_test_source(&source, function)
                    {
                        problems.push(format!("topic {id} test {path}::{function}: {reason}"));
                    }
                }
                Err(reason) => problems.push(format!("topic {id} test {path}: {reason}")),
            }
            if test["features"].as_array().is_none_or(|features| {
                features
                    .iter()
                    .any(|feature| feature.as_str().is_none_or(str::is_empty))
                    || features
                        .iter()
                        .map(Value::to_string)
                        .collect::<BTreeSet<_>>()
                        .len()
                        != features.len()
            }) {
                problems.push(format!(
                    "topic {id}: invalid or duplicate feature declarations"
                ));
            }
        }
    }
    for id in TOPICS {
        if !seen.contains(id) {
            problems.push(format!("missing topic {id}"));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
    }
    fn fixture() -> Value {
        json!({"schema_version":1,"topics":TOPICS.map(|id|json!({
            "id":id,"page":"site/content/docs/architecture.mdx",
            "tests":[{"path":"xtask/src/lint.rs","function":"clean_source_produces_no_findings","features":[]}]
        }))})
    }
    #[test]
    fn complete_topic_inventory_passes() {
        assert!(validate(root(), &fixture()).is_empty());
    }
    #[test]
    fn committed_inventory_has_complete_current_source_authorities() {
        let inventory =
            serde_json::from_str(&fs::read_to_string(root().join(REGISTRY)).unwrap()).unwrap();
        assert_eq!(validate(root(), &inventory), Vec::<String>::new());
    }
    #[test]
    fn missing_duplicate_and_unknown_topics_fail() {
        let mut missing = fixture();
        missing["topics"].as_array_mut().unwrap().pop();
        assert!(validate(root(), &missing)
            .iter()
            .any(|p| p.contains("missing topic public-api")));
        for id in ["architecture", "invented"] {
            let mut changed = fixture();
            changed["topics"][1]["id"] = json!(id);
            assert!(validate(root(), &changed)
                .iter()
                .any(|p| p.contains("unknown or duplicate")));
        }
    }
    #[test]
    fn unsafe_absent_and_historical_pages_fail() {
        for page in [
            "site/content/docs/../../outside.mdx",
            "site/content/docs/absent.mdx",
            "site/content/docs/changelog/v0.10.0.mdx",
            "C:/outside.mdx",
        ] {
            let mut changed = fixture();
            changed["topics"][0]["page"] = json!(page);
            assert!(!validate(root(), &changed).is_empty(), "{page}");
        }
    }
    #[test]
    fn absent_non_test_and_malformed_feature_authorities_fail() {
        for function in ["invented_test", "run"] {
            let mut changed = fixture();
            changed["topics"][0]["tests"][0]["function"] = json!(function);
            assert!(!validate(root(), &changed).is_empty());
        }
        for features in [
            json!(null),
            json!([1]),
            json!(["deep-capture", "deep-capture"]),
        ] {
            let mut changed = fixture();
            changed["topics"][0]["tests"][0]["features"] = features;
            assert!(!validate(root(), &changed).is_empty());
        }
        for source in [
            "#[test]\n#[ignore]\nfn check() {}",
            "/* #[test]\nfn check() {} */",
            "#[test]\n#[cfg(any())]\nfn check() {}",
        ] {
            assert!(crate::threat_model::validate_test_source(source, "check").is_err());
        }
    }
}
