// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const REGISTRY: &str = "fuzz/fuzz-targets.json";
const EXPECTED_SURFACES: &[&str] = &[
    "application-jsonl",
    "bundle-manifest",
    "certificate-identity",
    "destination-authority",
    "grpc-envelope-state",
    "http1-chunk-framing",
    "http1-request-head",
    "http1-response-head",
    "http3-evidence-state",
    "lifecycle-jsonl",
    "manifest-relative-path",
    "process-trace-jsonl",
    "proxy-basic-auth",
    "quic-initial-classifier",
    "resource-journal-jsonl",
    "socks5-auth-state",
    "socks5-request-state",
    "socks5-udp-datagram",
    "sse-state",
    "websocket-state",
];

pub fn run(root: &Path) -> io::Result<usize> {
    let value: Value =
        serde_json::from_slice(&fs::read(root.join(REGISTRY))?).map_err(io::Error::other)?;
    let problems = validate(root, &value)?;
    for problem in &problems {
        eprintln!("fuzz: {problem}");
    }
    if problems.is_empty() {
        let targets = value["targets"].as_array().map_or(0, Vec::len);
        let surfaces = value["targets"].as_array().map_or(0, |values| {
            values
                .iter()
                .filter_map(|target| target["surfaces"].as_array())
                .map(Vec::len)
                .sum()
        });
        let seeds = seed_files(root, &value).map_or(0, |values| values.len());
        println!("fuzz: schema 1, {targets} targets, {surfaces} surfaces, {seeds} synthetic seeds");
    }
    Ok(problems.len())
}

fn validate(root: &Path, value: &Value) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    if value["schema_version"].as_u64() != Some(1) {
        problems.push("schema_version must be 1".into());
    }
    let max = value["max_input_bytes"].as_u64().unwrap_or_default();
    if max != 65_536 {
        problems.push("max_input_bytes must be exactly 65536".into());
    }
    for (field, expected) in [
        ("toolchain", "nightly-2026-08-25"),
        ("cargo_fuzz_version", "0.13.2"),
        ("libfuzzer_sys_version", "0.4.13"),
    ] {
        if value[field].as_str() != Some(expected) {
            problems.push(format!("{field} must be {expected}"));
        }
    }

    let targets = value["targets"]
        .as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "targets must be an array"))?;
    let mut target_ids = BTreeSet::new();
    let mut surfaces = BTreeSet::new();
    for target in targets {
        let id = required(target, "id", &mut problems);
        if !target_ids.insert(id.clone()) {
            problems.push(format!("duplicate target {id}"));
        }
        if !matches!(
            target["owner"].as_str(),
            Some("fragcap") | Some("fragcap-proxy")
        ) {
            problems.push(format!("target {id} has unknown owner"));
        }
        if target["ci_runs"].as_u64() != Some(256) || target["timeout_seconds"].as_u64() != Some(5)
        {
            problems.push(format!(
                "target {id} has unsafe or unreproducible CI bounds"
            ));
        }
        let source = root.join(format!("fuzz/fuzz_targets/{id}.rs"));
        if !source.is_file() {
            problems.push(format!("target {id} has no source"));
        }
        let corpus = root.join(required(target, "corpus", &mut problems));
        if !corpus.is_dir() {
            problems.push(format!("target {id} has no corpus directory"));
        }
        if let Some(dictionary) = target["dictionary"].as_str() {
            if !root.join(dictionary).is_file() {
                problems.push(format!("target {id} dictionary is missing"));
            }
        }
        let rows = target["surfaces"].as_array();
        if rows.is_none_or(Vec::is_empty) {
            problems.push(format!("target {id} has no surfaces"));
        }
        for surface in rows.into_iter().flatten() {
            let surface_id = required(surface, "id", &mut problems);
            if !surfaces.insert(surface_id.clone()) {
                problems.push(format!("duplicate surface {surface_id}"));
            }
            let retained = surface["max_retained_bytes"].as_u64();
            if retained.is_none() || retained.is_some_and(|limit| limit > max) {
                problems.push(format!("surface {surface_id} has invalid retention bound"));
            }
            if surface["states"].as_array().is_none_or(Vec::is_empty) {
                problems.push(format!("surface {surface_id} has no states"));
            }
        }
    }
    let expected = EXPECTED_SURFACES
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    if surfaces != expected {
        problems.push(format!(
            "surface inventory drift: expected {expected:?}, found {surfaces:?}"
        ));
    }

    validate_manifest(root, value, &target_ids, &mut problems)?;
    validate_workflow(root, value, &target_ids, &mut problems)?;
    validate_replay(root, &target_ids, &mut problems)?;
    validate_corpus(root, value, &mut problems)?;
    Ok(problems)
}

fn required(value: &Value, field: &str, problems: &mut Vec<String>) -> String {
    value[field]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            problems.push(format!("missing nonempty {field}"));
            String::new()
        })
}

fn validate_manifest(
    root: &Path,
    value: &Value,
    targets: &BTreeSet<String>,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    let manifest = fs::read_to_string(root.join("fuzz/Cargo.toml"))?;
    let lock = fs::read_to_string(root.join("fuzz/Cargo.lock"))?;
    let product_lock = fs::read_to_string(root.join("Cargo.lock"))?;
    if !manifest.contains("libfuzzer-sys = \"=0.4.13\"") {
        problems.push("fuzz manifest does not exact-pin libfuzzer-sys 0.4.13".into());
    }
    if !manifest.contains("[workspace]\n") {
        problems.push("fuzz manifest is not an isolated workspace".into());
    }
    if !lock.contains("name = \"libfuzzer-sys\"\nversion = \"0.4.13\"") {
        problems.push("fuzz lockfile does not resolve libfuzzer-sys 0.4.13".into());
    }
    if product_lock.contains("name = \"libfuzzer-sys\"") {
        problems.push("product lockfile contains the fuzz-only engine".into());
    }
    for target in targets {
        if !manifest.contains(&format!("name = \"{target}\"")) {
            problems.push(format!("target {target} is absent from fuzz manifest"));
        }
    }
    if value["libfuzzer_sys_version"].as_str() != Some("0.4.13") {
        problems.push("registry and fuzz manifest version disagree".into());
    }
    Ok(())
}

fn validate_workflow(
    root: &Path,
    value: &Value,
    targets: &BTreeSet<String>,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    let workflow = fs::read_to_string(root.join(".github/workflows/fuzz.yml"))?;
    for expected in [
        "nightly-2026-08-25",
        "0.13.2",
        "-runs=256",
        "-timeout=5",
        "-max_len=65536",
        "metadata --manifest-path fuzz/Cargo.toml --locked",
    ] {
        if !workflow.contains(expected) {
            problems.push(format!("fuzz workflow is missing {expected}"));
        }
    }
    for target in targets {
        if !workflow.contains(&format!("target: {target}")) {
            problems.push(format!("target {target} is absent from CI matrix"));
        }
    }
    if value["toolchain"].as_str() != Some("nightly-2026-08-25") {
        problems.push("registry and workflow toolchain disagree".into());
    }
    Ok(())
}

fn validate_replay(
    root: &Path,
    targets: &BTreeSet<String>,
    problems: &mut Vec<String>,
) -> io::Result<()> {
    let replay = fs::read_to_string(root.join("crates/fragcap/tests/fuzz_seeds.rs"))?;
    for target in targets {
        if !replay.contains(&format!("\"{target}\" =>")) {
            problems.push(format!("target {target} is absent from stable replay"));
        }
    }
    Ok(())
}

fn seed_files(root: &Path, value: &Value) -> io::Result<Vec<(String, PathBuf)>> {
    let mut result = Vec::new();
    for target in value["targets"].as_array().into_iter().flatten() {
        let id = target["id"].as_str().unwrap_or_default();
        let corpus = root.join(target["corpus"].as_str().unwrap_or_default());
        if !corpus.is_dir() {
            continue;
        }
        for entry in fs::read_dir(corpus)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                result.push((id.to_string(), entry.path()));
            }
        }
    }
    result.sort();
    Ok(result)
}

fn validate_corpus(root: &Path, value: &Value, problems: &mut Vec<String>) -> io::Result<()> {
    let files = seed_files(root, value)?;
    let mut per_target = BTreeMap::<String, usize>::new();
    let mut seen = BTreeMap::<String, BTreeSet<Vec<u8>>>::new();
    let tracked = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "--", "fuzz/corpus"])
        .output()?;
    let tracked = String::from_utf8_lossy(&tracked.stdout)
        .lines()
        .map(|line| line.replace('\\', "/").to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for (target, path) in files {
        *per_target.entry(target.clone()).or_default() += 1;
        let data = fs::read(&path)?;
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        if data.is_empty() {
            problems.push(format!("empty corpus seed {relative}"));
        }
        if data.len() > 65_536 {
            problems.push(format!("oversized corpus seed {relative}"));
        }
        if !tracked.contains(&relative) {
            problems.push(format!("untracked corpus seed {relative}"));
        }
        if !seen.entry(target.clone()).or_default().insert(data.clone()) {
            problems.push(format!(
                "duplicate corpus bytes in target {target}: {relative}"
            ));
        }
        let lower = String::from_utf8_lossy(&data).to_ascii_lowercase();
        for forbidden in [
            "private key",
            "authorization: bearer",
            "cookie:",
            "set-cookie:",
            "password=",
        ] {
            if lower.contains(forbidden) {
                problems.push(format!(
                    "forbidden corpus content in {relative}: {forbidden}"
                ));
            }
        }
    }
    for target in value["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|target| target["id"].as_str())
    {
        if per_target.get(target).copied().unwrap_or_default() == 0 {
            problems.push(format!("target {target} has an empty corpus"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_surface_inventory_is_unique_and_sorted() {
        assert!(EXPECTED_SURFACES.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn missing_required_string_is_reported() {
        let mut problems = Vec::new();
        assert_eq!(required(&serde_json::json!({}), "id", &mut problems), "");
        assert_eq!(problems, ["missing nonempty id"]);
    }

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask is in the repository root")
            .to_path_buf()
    }

    fn registry() -> Value {
        serde_json::from_slice(
            &fs::read(repository_root().join(REGISTRY)).expect("registry is readable"),
        )
        .expect("registry is valid JSON")
    }

    #[test]
    fn repository_registry_satisfies_the_gate() {
        assert_eq!(
            validate(&repository_root(), &registry()).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn surface_inventory_drift_is_rejected() {
        let mut value = registry();
        value["targets"][0]["surfaces"]
            .as_array_mut()
            .unwrap()
            .pop();
        let problems = validate(&repository_root(), &value).unwrap();
        assert!(problems
            .iter()
            .any(|problem| problem.contains("surface inventory drift")));
    }

    #[test]
    fn duplicate_target_and_unsafe_bounds_are_rejected() {
        let mut value = registry();
        value["targets"][1]["id"] = value["targets"][0]["id"].clone();
        value["targets"][0]["ci_runs"] = Value::from(257);
        let problems = validate(&repository_root(), &value).unwrap();
        assert!(problems
            .iter()
            .any(|problem| problem.contains("duplicate target")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("unsafe or unreproducible")));
    }

    #[test]
    fn missing_corpus_and_version_drift_are_rejected() {
        let mut value = registry();
        value["targets"][0]["corpus"] = Value::from("fuzz/corpus/absent");
        value["cargo_fuzz_version"] = Value::from("unreviewed");
        let problems = validate(&repository_root(), &value).unwrap();
        assert!(problems
            .iter()
            .any(|problem| problem.contains("no corpus directory")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("cargo_fuzz_version")));
    }
}
