// SPDX-License-Identifier: Apache-2.0

//! Closed native Deep Capture failure-injection inventory.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Component, Path};
use std::process::Command;

use serde_json::Value;

const REGISTRY: &str = "docs/security/deep-capture-failures.v1.json";
const SIDES: &[&str] = &["before", "after"];
const FAMILIES: &[&str] = &[
    "broken-pipe",
    "cancellation",
    "disk-full",
    "network-reset",
    "permission-denial",
    "port-theft",
    "task-panic",
    "timeout",
    "trust-denial",
    "writer-corruption",
];
const OUTCOMES: &[&str] = &[
    "artifact", "cleanup", "event", "fact", "journal", "recovery", "terminal",
];
const DRIVERS: &[&str] = &[
    "artifact-finalization-corruption",
    "artifact-write-failure",
    "authorization-or-start-refusal",
    "authorization-refusal",
    "capture-interrupted",
    "capture-run-failure",
    "capture-start-failure",
    "capture-stop-timeout",
    "cleanup-denied",
    "fact-writer-failure",
    "late-start-result",
    "launcher-late-success",
    "launcher-start-failure",
    "listener-bind-refusal",
    "manifest-corruption",
    "observation-event-failure",
    "operator-stop",
    "proxy-cancelled-after-start",
    "proxy-late-success",
    "proxy-start-refusal",
    "proxy-stop-reset",
    "proxy-task-join-failure",
    "route-apply-denied",
    "route-owned-then-launch-fails",
    "started-event-delivery-failure",
    "terminal-event-failure",
    "trust-acquire-denied",
    "trust-cleanup-denied",
];

pub fn run(root: &Path) -> io::Result<usize> {
    let value: Value =
        serde_json::from_slice(&fs::read(root.join(REGISTRY))?).map_err(io::Error::other)?;
    let problems = validate(root, &value)?;
    for problem in &problems {
        eprintln!("failure-matrix: {problem}");
    }
    if problems.is_empty() {
        let boundaries = value["effects"].as_array().map_or(0, Vec::len)
            + value["transitions"].as_array().map_or(0, Vec::len);
        println!(
            "failure-matrix: schema 1, {boundaries} boundaries, {} scenarios, {} failure families, {} outcome authorities",
            boundaries * SIDES.len(),
            FAMILIES.len(),
            OUTCOMES.len()
        );
    }
    Ok(problems.len())
}

fn validate(root: &Path, value: &Value) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    if value["schema_version"].as_u64() != Some(1) {
        problems.push("schema_version must be 1".into());
    }
    for field in ["reviewed_on", "issue"] {
        required(value, field, "registry", &mut problems);
    }
    exact_strings(value, "failure_families", FAMILIES, &mut problems);
    exact_strings(value, "outcome_dimensions", OUTCOMES, &mut problems);

    let resource_source =
        fs::read_to_string(root.join("crates/fragcap/src/deep_capture/journal.rs"))?;
    let lifecycle_source =
        fs::read_to_string(root.join("crates/fragcap/src/deep_capture/model.rs"))?;
    let coordinator_source =
        fs::read_to_string(root.join("crates/fragcap/src/deep_capture/session.rs"))?;
    compare_inventory(
        value,
        "resource_kinds",
        &enum_variants(&resource_source, "ResourceKind")
            .into_iter()
            .map(to_kebab)
            .collect(),
        &mut problems,
    );
    compare_inventory(
        value,
        "lifecycle_states",
        &enum_variants(&lifecycle_source, "LifecycleState")
            .into_iter()
            .map(to_kebab)
            .collect(),
        &mut problems,
    );

    let effects = array(value, "effects", &mut problems);
    let transitions = array(value, "transitions", &mut problems);
    let mut ids = BTreeSet::new();
    let mut families = BTreeSet::new();
    let mut effect_ids = BTreeSet::new();
    for boundary in effects.iter().copied() {
        let id = validate_boundary(boundary, "effect", &mut ids, &mut families, &mut problems);
        let resource = required(boundary, "resource_kind", &id, &mut problems);
        if !strings(value, "resource_kinds").contains(&resource) {
            problems.push(format!("effect {id} has unknown resource kind {resource}"));
        }
        effect_ids.insert(id);
    }
    for boundary in transitions.iter().copied() {
        let id = validate_boundary(
            boundary,
            "transition",
            &mut ids,
            &mut families,
            &mut problems,
        );
        for field in ["from", "to"] {
            let state = required(boundary, field, &id, &mut problems);
            if !strings(value, "lifecycle_states").contains(&state) {
                problems.push(format!("transition {id} has unknown {field} state {state}"));
            }
        }
    }
    let source_effects = coordinator_effects(&coordinator_source);
    if effect_ids != source_effects {
        problems.push(format!(
            "coordinator effect inventory drift: registry={effect_ids:?}, source={source_effects:?}"
        ));
    }
    let expected_families = FAMILIES.iter().map(|value| (*value).to_string()).collect();
    if families != expected_families {
        problems.push(format!(
            "failure family execution drift: expected={expected_families:?}, found={families:?}"
        ));
    }
    if effects.len() != 7 || transitions.len() != 8 {
        problems.push(format!(
            "matrix must contain 7 effects and 8 transitions, found {} and {}",
            effects.len(),
            transitions.len()
        ));
    }
    validate_test_references(root, value, &mut problems)?;
    Ok(problems)
}

fn validate_boundary(
    boundary: &Value,
    kind: &str,
    ids: &mut BTreeSet<String>,
    families: &mut BTreeSet<String>,
    problems: &mut Vec<String>,
) -> String {
    let id = required(boundary, "id", kind, problems);
    if !valid_id(&id) {
        problems.push(format!("{kind} id {id:?} is not lowercase kebab-case"));
    }
    if !ids.insert(id.clone()) {
        problems.push(format!("duplicate boundary {id}"));
    }
    required(boundary, "source", &id, problems);
    for side in SIDES {
        let driver = &boundary[*side];
        if !driver.is_object() {
            problems.push(format!("boundary {id} is missing {side} driver"));
            continue;
        }
        let family = required(driver, "family", &format!("{id}:{side}"), problems);
        if FAMILIES.contains(&family.as_str()) {
            families.insert(family);
        } else {
            problems.push(format!("boundary {id}:{side} has unknown family {family}"));
        }
        let driver_id = required(driver, "driver", &format!("{id}:{side}"), problems);
        if !DRIVERS.contains(&driver_id.as_str()) {
            problems.push(format!(
                "boundary {id}:{side} has unknown controlled driver {driver_id}"
            ));
        }
        let expected = &driver["expected"];
        if !expected.is_object() {
            problems.push(format!(
                "boundary {id}:{side} has no expected outcome vector"
            ));
            continue;
        }
        for dimension in OUTCOMES {
            let outcome = required(expected, dimension, &format!("{id}:{side}"), problems);
            if !allowed_outcomes(dimension).contains(&outcome.as_str()) {
                problems.push(format!(
                    "boundary {id}:{side} has unknown {dimension} outcome {outcome}"
                ));
            }
        }
    }
    id
}

fn validate_test_references(
    root: &Path,
    value: &Value,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    let tracked = Command::new("git")
        .current_dir(root)
        .args(["ls-files"])
        .output()?;
    if !tracked.status.success() {
        return Err(io::Error::other("git ls-files failed"));
    }
    let tracked = String::from_utf8_lossy(&tracked.stdout)
        .lines()
        .map(|line| line.replace('\\', "/"))
        .collect::<BTreeSet<_>>();
    let tests = array(value, "tests", problems);
    if tests.is_empty() {
        problems.push("registry has no executable tests".into());
    }
    for test in tests {
        let path = required(test, "path", "test", problems).replace('\\', "/");
        let function = required(test, "function", &path, problems);
        let relative = Path::new(&path);
        if !path.ends_with(".rs")
            || !relative
                .components()
                .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
        {
            problems.push(format!(
                "test reference is not a repository Rust source: {path}"
            ));
            continue;
        }
        if !tracked.contains(path.as_str()) {
            problems.push(format!("test reference is not tracked: {path}"));
            continue;
        }
        let source = fs::read_to_string(root.join(&path))?;
        if crate::threat_model::validate_test_source(&source, &function).is_err() {
            problems.push(format!(
                "{path} references missing, ignored, or conditional test {function}"
            ));
        }
    }
    Ok(())
}

fn array<'a>(value: &'a Value, field: &str, problems: &mut Vec<String>) -> Vec<&'a Value> {
    match value[field].as_array() {
        Some(values) if !values.is_empty() => values.iter().collect(),
        _ => {
            problems.push(format!("{field} must be a nonempty array"));
            Vec::new()
        }
    }
}

fn strings(value: &Value, field: &str) -> BTreeSet<String> {
    value[field]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn exact_strings(value: &Value, field: &str, expected: &[&str], problems: &mut Vec<String>) {
    let ordered = value[field]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let found = strings(value, field);
    let expected = expected.iter().map(|value| (*value).to_string()).collect();
    if found != expected {
        problems.push(format!(
            "{field} drift: expected={expected:?}, found={found:?}"
        ));
    }
    if ordered.len() != found.len() {
        problems.push(format!("{field} contains duplicate or non-string entries"));
    }
}

fn allowed_outcomes(dimension: &str) -> &'static [&'static str] {
    match dimension {
        "terminal" => &[
            "failed",
            "interrupted",
            "interrupted-or-failed",
            "interrupted-or-partial",
            "partial",
            "partial-or-failed",
        ],
        "artifact" => &["failed-not-complete", "incomplete", "independent", "none"],
        "fact" => &[
            "failed-counted",
            "independent",
            "no-positive-invention",
            "none",
        ],
        "event" => &["failed-counted", "terminal-attempted"],
        "cleanup" => &[
            "acquired-attempted",
            "all-acquired-attempted",
            "all-attempted",
            "already-attempted",
            "child-and-earlier-attempted",
            "earlier-resources-attempted",
            "later-cleanup-attempted",
            "later-obligations-not-applied",
            "none",
            "owned-only",
            "proxy-attempted",
            "route-and-earlier-attempted",
        ],
        "journal" => &[
            "complete-or-recoverable",
            "failed",
            "failed-or-recoverable",
            "none",
            "none-or-complete",
            "not-applied",
            "retained-or-failed",
            "terminal-or-recoverable",
            "timed-out",
        ],
        "recovery" => &[
            "exact",
            "exact-or-none",
            "exact-or-refused",
            "exact-trust-action",
            "none",
            "pending-obligation-exact",
        ],
        _ => &[],
    }
}

fn compare_inventory(
    value: &Value,
    field: &str,
    source: &BTreeSet<String>,
    problems: &mut Vec<String>,
) {
    let registry = strings(value, field);
    if &registry != source {
        problems.push(format!(
            "{field} source drift: registry={registry:?}, source={source:?}"
        ));
    }
}

fn enum_variants(source: &str, name: &str) -> BTreeSet<String> {
    let marker = format!("pub enum {name} {{");
    let Some(body) = source.split_once(&marker).map(|(_, rest)| rest) else {
        return BTreeSet::new();
    };
    let Some(body) = body.split_once('}').map(|(body, _)| body) else {
        return BTreeSet::new();
    };
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("///") && !line.starts_with("#["))
        .filter_map(|line| line.trim_end_matches(',').split_whitespace().next())
        .filter(|name| name.chars().next().is_some_and(char::is_uppercase))
        .map(str::to_owned)
        .collect()
}

fn coordinator_effects(source: &str) -> BTreeSet<String> {
    let mut effects = BTreeSet::new();
    let mut remainder = source;
    while let Some((_, tail)) = remainder.split_once("record_resource(") {
        let argument = tail.trim_start();
        if let Some(quoted) = argument.strip_prefix('"') {
            if let Some((effect, _)) = quoted.split_once('"') {
                effects.insert(effect.to_string());
            }
        }
        remainder = tail;
    }
    effects
}

fn to_kebab(value: String) -> String {
    let mut result = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            result.push('-');
        }
        result.extend(character.to_lowercase());
    }
    result
}

fn required(value: &Value, field: &str, owner: &str, problems: &mut Vec<String>) -> String {
    value[field]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            problems.push(format!("{owner} has no nonempty {field}"));
            String::new()
        })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn registry() -> Value {
        serde_json::from_slice(&fs::read(root().join(REGISTRY)).unwrap()).unwrap()
    }

    #[test]
    fn repository_failure_matrix_is_closed() {
        assert_eq!(
            validate(&root(), &registry()).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn missing_side_and_outcome_are_rejected() {
        let mut value = registry();
        value["effects"][0]["before"] = Value::Null;
        value["effects"][1]["after"]["expected"]["cleanup"] = Value::Null;
        let problems = validate(&root(), &value).unwrap();
        assert!(problems
            .iter()
            .any(|problem| problem.contains("missing before")));
        assert!(problems.iter().any(|problem| problem.contains("cleanup")));
    }

    #[test]
    fn unknown_family_and_inventory_drift_are_rejected() {
        let mut value = registry();
        value["transitions"][0]["before"]["family"] = Value::from("mystery");
        value["resource_kinds"].as_array_mut().unwrap().pop();
        let problems = validate(&root(), &value).unwrap();
        assert!(problems
            .iter()
            .any(|problem| problem.contains("unknown family")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("source drift")));
    }

    #[test]
    fn duplicate_vocabulary_and_unknown_outcome_are_rejected() {
        let mut value = registry();
        let duplicate = value["failure_families"][0].clone();
        value["failure_families"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        value["effects"][0]["after"]["expected"]["terminal"] = Value::from("invented");
        let problems = validate(&root(), &value).unwrap();
        assert!(problems.iter().any(|problem| problem.contains("duplicate")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("unknown terminal outcome")));
    }

    #[test]
    fn unknown_controlled_driver_is_rejected() {
        let mut value = registry();
        value["effects"][0]["before"]["driver"] = Value::from("unimplemented-driver");
        assert!(validate(&root(), &value)
            .unwrap()
            .iter()
            .any(|problem| problem.contains("unknown controlled driver")));
    }

    #[test]
    fn source_inventory_parsers_are_closed() {
        assert_eq!(
            enum_variants("pub enum State {\n One,\n Two,\n }", "State"),
            BTreeSet::from(["One".into(), "Two".into()])
        );
        assert_eq!(
            coordinator_effects(
                "x.record_resource(\n \"one\",\n); y.record_resource(\"two\", value);"
            ),
            BTreeSet::from(["one".into(), "two".into()])
        );
    }

    #[test]
    fn missing_ignored_and_conditional_tests_are_rejected() {
        let validate = crate::threat_model::validate_test_source;
        assert!(validate("#[test]\nfn good() {}", "good").is_ok());
        assert!(validate("#[test]\n#[ignore]\nfn bad() {}", "bad").is_err());
        assert!(validate("#[test]\n#[cfg(windows)]\nfn bad() {}", "bad").is_err());
        assert!(validate("fn helper() {}", "helper").is_err());
    }
}
