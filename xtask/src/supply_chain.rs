// SPDX-License-Identifier: Apache-2.0

//! Closed dependency-policy and release-evidence validation for S130.

use ring::digest::{digest, SHA256};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const POLICY_REL: &str = "supply-chain/policy-v1.json";
const WINDOWS_TARGET: &str = "x86_64-pc-windows-msvc";
const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";
const RELEASE_FEATURES: &str = "fragcap-cli/etw,fragcap-cli/live,fragcap-cli/socket-table";
const MAX_FINDINGS: usize = 256;

#[derive(Clone, Debug)]
struct Package {
    id: String,
    key: String,
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    license: Option<String>,
    rust_version: Option<String>,
    features: Vec<String>,
}

#[derive(Clone, Debug)]
struct Graph {
    name: String,
    packages: BTreeMap<String, Package>,
    edges: BTreeSet<String>,
    digest: String,
}

pub fn run(root: &Path, args: &[String]) -> Result<usize, String> {
    match args.first().map(String::as_str) {
        None => validate_repository(root),
        Some("snapshot") => {
            for graph in collect_graphs(root)? {
                println!(
                    "{} digest={} packages={} edges={}",
                    graph.name,
                    graph.digest,
                    graph.packages.len(),
                    graph.edges.len()
                );
            }
            Ok(0)
        }
        Some("stamp-evidence") if args.len() == 3 => {
            stamp_evidence(root, Path::new(&args[1]), Path::new(&args[2]))?;
            validate_evidence(root, Path::new(&args[1]), Path::new(&args[2]))
        }
        Some("validate-evidence") if args.len() == 3 => {
            validate_evidence(root, Path::new(&args[1]), Path::new(&args[2]))
        }
        _ => Err("use supply-chain [snapshot | stamp-evidence <sbom> <notices> | validate-evidence <sbom> <notices>]".to_string()),
    }
}

fn validate_repository(root: &Path) -> Result<usize, String> {
    let policy_bytes = fs::read(root.join(POLICY_REL)).map_err(|e| e.to_string())?;
    let policy: Value = serde_json::from_slice(&policy_bytes).map_err(|e| e.to_string())?;
    let graphs = collect_graphs(root)?;
    let mut findings = validate_policy(&policy, &graphs, today_ymd());
    findings.extend(validate_repository_wiring(root, &policy)?);
    normalize_findings(&mut findings);
    for finding in &findings {
        eprintln!("supply-chain: {finding}");
    }
    Ok(findings.len())
}

fn collect_graphs(root: &Path) -> Result<Vec<Graph>, String> {
    Ok(vec![
        metadata_graph(root, "linux-all", LINUX_TARGET, true, false)?,
        metadata_graph(root, "windows-all", WINDOWS_TARGET, true, false)?,
        metadata_graph(root, "windows-release", WINDOWS_TARGET, false, true)?,
    ])
}

fn metadata_graph(
    root: &Path,
    name: &str,
    target: &str,
    all_features: bool,
    release_only: bool,
) -> Result<Graph, String> {
    let mut command = Command::new(env!("CARGO"));
    command.current_dir(root).args([
        "metadata",
        "--locked",
        "--offline",
        "--format-version",
        "1",
        "--filter-platform",
        target,
    ]);
    if all_features {
        command.arg("--all-features");
    } else {
        command.args(["--no-default-features", "--features", RELEASE_FEATURES]);
    }
    let output = command
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata for {name} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let lock =
        lock_checksums(&fs::read_to_string(root.join("Cargo.lock")).map_err(|e| e.to_string())?)?;
    let graph = normalize_metadata(name, &metadata, release_only, &lock)?;
    if release_only {
        restrict_release_graph(root, graph)
    } else {
        Ok(graph)
    }
}

fn restrict_release_graph(root: &Path, mut graph: Graph) -> Result<Graph, String> {
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "tree",
            "--locked",
            "--offline",
            "-p",
            "fragcap-cli",
            "--target",
            WINDOWS_TARGET,
            "--no-default-features",
            "--features",
            "live,socket-table,etw",
            "--edges",
            "normal",
            "--prefix",
            "none",
            "--format",
            "{p}|{f}",
        ])
        .output()
        .map_err(|e| format!("cargo tree: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo tree for windows-release failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut exact = BTreeMap::<(String, String), Vec<String>>::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.strip_suffix(" (*)").unwrap_or(line);
        let Some((package, feature_text)) = line.split_once('|') else {
            return Err(format!("cargo tree row has no feature separator: {line}"));
        };
        let Some((name, version_and_path)) = package.split_once(" v") else {
            return Err(format!("cargo tree row has no version: {line}"));
        };
        let version = version_and_path.split_whitespace().next().unwrap_or("");
        let mut features: Vec<String> = feature_text
            .split(',')
            .filter(|feature| !feature.is_empty())
            .map(str::to_string)
            .collect();
        features.sort();
        exact.insert((name.to_string(), version.to_string()), features);
    }
    graph.packages.retain(|_, package| {
        exact
            .get(&(package.name.clone(), package.version.clone()))
            .map(|features| {
                package.features.clone_from(features);
                true
            })
            .unwrap_or(false)
    });
    graph.edges.retain(|edge| {
        graph.packages.keys().any(|from| {
            graph
                .packages
                .keys()
                .any(|to| edge.starts_with(&format!("{from}->{to}|")))
        })
    });
    graph.digest = graph_digest(&graph.packages, &graph.edges);
    Ok(graph)
}

fn normalize_metadata(
    name: &str,
    metadata: &Value,
    release_only: bool,
    lock: &BTreeMap<(String, String, String), String>,
) -> Result<Graph, String> {
    let package_values = array(metadata, "packages")?;
    let node_values = metadata
        .get("resolve")
        .and_then(|v| v.get("nodes"))
        .and_then(Value::as_array)
        .ok_or("metadata.resolve.nodes is missing")?;
    let workspace: BTreeSet<String> = array(metadata, "workspace_members")?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    let mut raw = BTreeMap::new();
    for value in package_values {
        let object = value
            .as_object()
            .ok_or("metadata package is not an object")?;
        let id = string(object, "id")?;
        let package_name = string(object, "name")?;
        let version = string(object, "version")?;
        let source = optional_string(object, "source");
        let source_key = source.clone().unwrap_or_else(|| "workspace".to_string());
        let key = format!("{package_name}@{version}|{source_key}");
        let checksum = optional_string(object, "checksum").or_else(|| {
            source.as_ref().and_then(|package_source| {
                lock.get(&(
                    package_name.clone(),
                    version.clone(),
                    package_source.clone(),
                ))
                .cloned()
            })
        });
        raw.insert(
            id.clone(),
            Package {
                id,
                key,
                name: package_name,
                version,
                source,
                checksum,
                license: optional_string(object, "license"),
                rust_version: optional_string(object, "rust_version"),
                features: Vec::new(),
            },
        );
    }
    let mut node_map = BTreeMap::new();
    for node in node_values {
        let object = node.as_object().ok_or("metadata node is not an object")?;
        node_map.insert(string(object, "id")?, object.clone());
    }
    let included: BTreeSet<String> = if release_only {
        let root_id = raw
            .values()
            .find(|p| p.name == "fragcap-cli" && workspace.contains(&p.id))
            .map(|p| p.id.clone())
            .ok_or("release root fragcap-cli is missing")?;
        runtime_closure(&root_id, &node_map)?
    } else {
        node_map.keys().cloned().collect()
    };
    let mut packages = BTreeMap::new();
    for id in &included {
        let mut package = raw
            .get(id)
            .cloned()
            .ok_or_else(|| format!("node package {id} is missing"))?;
        let node = node_map
            .get(id)
            .ok_or_else(|| format!("node {id} is missing"))?;
        package.features = node
            .get("features")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();
        package.features.sort();
        packages.insert(package.key.clone(), package);
    }
    let mut edges = BTreeSet::new();
    for id in &included {
        let from = raw.get(id).ok_or("edge source is missing")?.key.clone();
        let node = node_map.get(id).ok_or("edge node is missing")?;
        for dep in node
            .get("deps")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let dep_obj = dep.as_object().ok_or("dependency is not an object")?;
            let target_id = string(dep_obj, "pkg")?;
            if !included.contains(&target_id) {
                continue;
            }
            let to = raw
                .get(&target_id)
                .ok_or("edge target is missing")?
                .key
                .clone();
            let declared = string(dep_obj, "name")?;
            for kind in dep_obj
                .get("dep_kinds")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let kind_obj = kind.as_object().ok_or("dependency kind is not an object")?;
                let dependency_kind =
                    optional_string(kind_obj, "kind").unwrap_or_else(|| "normal".to_string());
                if release_only && dependency_kind != "normal" {
                    continue;
                }
                let target =
                    optional_string(kind_obj, "target").unwrap_or_else(|| "all".to_string());
                edges.insert(format!(
                    "{from}->{to}|{declared}|{dependency_kind}|{target}"
                ));
            }
        }
    }
    let digest = graph_digest(&packages, &edges);
    Ok(Graph {
        name: name.to_string(),
        packages,
        edges,
        digest,
    })
}

fn graph_digest(packages: &BTreeMap<String, Package>, edges: &BTreeSet<String>) -> String {
    let mut canonical = String::new();
    for package in packages.values() {
        canonical.push_str(&format!(
            "P|{}|{}|{}|{}|{}\n",
            package.key,
            package.checksum.as_deref().unwrap_or("none"),
            package.license.as_deref().unwrap_or("none"),
            package.rust_version.as_deref().unwrap_or("none"),
            package.features.join(",")
        ));
    }
    for edge in edges {
        canonical.push_str("E|");
        canonical.push_str(edge);
        canonical.push('\n');
    }
    sha256_hex(canonical.as_bytes())
}

fn lock_checksums(text: &str) -> Result<BTreeMap<(String, String, String), String>, String> {
    let mut result = BTreeMap::new();
    let mut fields = BTreeMap::<String, String>::new();
    let mut in_package = false;
    let flush = |fields: &mut BTreeMap<String, String>,
                 result: &mut BTreeMap<(String, String, String), String>| {
        if let (Some(name), Some(version), Some(source), Some(checksum)) = (
            fields.get("name"),
            fields.get("version"),
            fields.get("source"),
            fields.get("checksum"),
        ) {
            result.insert(
                (name.clone(), version.clone(), source.clone()),
                checksum.clone(),
            );
        }
        fields.clear();
    };
    for line in text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            flush(&mut fields, &mut result);
            in_package = true;
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            if matches!(key, "name" | "version" | "source" | "checksum") {
                let value = value.trim();
                if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
                    return Err(format!("Cargo.lock {key} is not a quoted string"));
                }
                fields.insert(key.to_string(), value[1..value.len() - 1].to_string());
            }
        }
    }
    flush(&mut fields, &mut result);
    Ok(result)
}

fn runtime_closure(
    root_id: &str,
    nodes: &BTreeMap<String, Map<String, Value>>,
) -> Result<BTreeSet<String>, String> {
    let mut included = BTreeSet::new();
    let mut pending = VecDeque::from([root_id.to_string()]);
    while let Some(id) = pending.pop_front() {
        if !included.insert(id.clone()) {
            continue;
        }
        let node = nodes
            .get(&id)
            .ok_or_else(|| format!("node {id} is missing"))?;
        for dep in node
            .get("deps")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let dep_obj = dep.as_object().ok_or("dependency is not an object")?;
            let normal = dep_obj
                .get("dep_kinds")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|kind| kind.get("kind").is_some_and(Value::is_null));
            if normal {
                pending.push_back(string(dep_obj, "pkg")?);
            }
        }
    }
    Ok(included)
}

fn validate_policy(policy: &Value, graphs: &[Graph], today: String) -> Vec<String> {
    let mut findings = Vec::new();
    let Some(object) = policy.as_object() else {
        return vec!["exception-schema: policy root is not an object".to_string()];
    };
    check_keys(
        object,
        &[
            "allowed_sources",
            "critical_dependencies",
            "duplicate_compatibility_exceptions",
            "exceptions",
            "graph_views",
            "release_features",
            "release_root",
            "release_target",
            "schema_version",
            "tools",
            "unsafe_review",
            "workspace_msrv",
        ],
        "policy",
        &mut findings,
    );
    if object.get("schema_version").and_then(Value::as_u64) != Some(1) {
        findings.push("exception-schema: schema_version must be 1".to_string());
    }
    if object.get("release_root").and_then(Value::as_str) != Some("fragcap-cli") {
        findings.push("graph-drift: release_root must be fragcap-cli".to_string());
    }
    if object.get("release_target").and_then(Value::as_str) != Some(WINDOWS_TARGET) {
        findings.push(format!(
            "graph-drift: release_target must be {WINDOWS_TARGET}"
        ));
    }
    let expected_features = ["etw", "live", "socket-table"];
    let actual_features = strings(object.get("release_features"));
    if actual_features != expected_features {
        findings
            .push("critical-feature: release features must be etw, live, socket-table".to_string());
    }
    let msrv = object
        .get("workspace_msrv")
        .and_then(Value::as_str)
        .unwrap_or("");
    if msrv != "1.88" {
        findings.push("declared-msrv: workspace policy must equal 1.88".to_string());
    }
    validate_graph_views(object, graphs, &mut findings);
    validate_packages(object, graphs, msrv, &mut findings);
    validate_duplicates(object, graphs, &today, &mut findings);
    validate_critical(object, graphs, &today, &mut findings);
    validate_unsafe_review(object, graphs, &today, &mut findings);
    validate_exceptions(object, &today, &mut findings);
    findings
}

fn validate_graph_views(object: &Map<String, Value>, graphs: &[Graph], findings: &mut Vec<String>) {
    let mut declared = BTreeMap::new();
    for view in object
        .get("graph_views")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(item) = view.as_object() else {
            findings.push("exception-schema: graph view is not an object".to_string());
            continue;
        };
        check_keys(
            item,
            &["digest", "edges", "name", "packages"],
            "graph view",
            findings,
        );
        if let Some(name) = item.get("name").and_then(Value::as_str) {
            if declared.insert(name, item).is_some() {
                findings.push(format!("exception-schema: duplicate graph view {name}"));
            }
        }
    }
    for graph in graphs {
        let Some(expected) = declared.remove(graph.name.as_str()) else {
            findings.push(format!("graph-drift: missing graph view {}", graph.name));
            continue;
        };
        if expected.get("digest").and_then(Value::as_str) != Some(graph.digest.as_str()) {
            findings.push(format!(
                "graph-drift: {} expected digest {}, observed {}",
                graph.name,
                expected
                    .get("digest")
                    .and_then(Value::as_str)
                    .unwrap_or("missing"),
                graph.digest
            ));
        }
        if expected.get("packages").and_then(Value::as_u64) != Some(graph.packages.len() as u64) {
            findings.push(format!(
                "package-count: {} observed {}",
                graph.name,
                graph.packages.len()
            ));
        }
        if expected.get("edges").and_then(Value::as_u64) != Some(graph.edges.len() as u64) {
            findings.push(format!(
                "edge-count: {} observed {}",
                graph.name,
                graph.edges.len()
            ));
        }
    }
    for extra in declared.keys() {
        findings.push(format!("graph-drift: unexpected graph view {extra}"));
    }
}

fn validate_packages(
    object: &Map<String, Value>,
    graphs: &[Graph],
    msrv: &str,
    findings: &mut Vec<String>,
) {
    let allowed: BTreeSet<String> = strings(object.get("allowed_sources")).into_iter().collect();
    for graph in graphs {
        for package in graph.packages.values() {
            if let Some(source) = &package.source {
                if !allowed.contains(source) {
                    findings.push(format!(
                        "source: {} has unapproved source {source}",
                        package.key
                    ));
                }
                if source.starts_with("registry+") && package.checksum.is_none() {
                    findings.push(format!(
                        "checksum: {} has no registry checksum",
                        package.key
                    ));
                }
                if package.license.is_none() {
                    findings.push(format!(
                        "license-metadata: {} has no declared license",
                        package.key
                    ));
                }
            }
            if let Some(required) = &package.rust_version {
                if version_gt(required, msrv) {
                    findings.push(format!(
                        "declared-msrv: {} declares Rust {required} above {msrv}",
                        package.key
                    ));
                }
            }
        }
    }
}

fn validate_duplicates(
    object: &Map<String, Value>,
    graphs: &[Graph],
    today: &str,
    findings: &mut Vec<String>,
) {
    let mut approved = BTreeMap::<String, BTreeSet<String>>::new();
    for value in object
        .get("duplicate_compatibility_exceptions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(item) = value.as_object() else {
            findings.push(
                "exception-schema: duplicate compatibility record is not an object".to_string(),
            );
            continue;
        };
        check_keys(
            item,
            &[
                "created",
                "expires",
                "lines",
                "name",
                "owner",
                "rationale",
                "removal_condition",
            ],
            "duplicate compatibility record",
            findings,
        );
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        validate_governance(
            item,
            today,
            &format!("duplicate-compatibility {name}"),
            findings,
        );
        let lines: BTreeSet<String> = strings(item.get("lines")).into_iter().collect();
        if name.is_empty() || approved.insert(name.to_string(), lines).is_some() {
            findings.push(format!(
                "exception-schema: duplicate compatibility identity {name}"
            ));
        }
    }
    let mut used = BTreeSet::new();
    for graph in graphs {
        let mut by_name: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
        for package in graph.packages.values() {
            by_name
                .entry(&package.name)
                .or_default()
                .insert(compatibility_line(&package.version));
        }
        for (name, lines) in by_name.into_iter().filter(|(_, lines)| lines.len() > 1) {
            if approved.get(name) != Some(&lines) {
                findings.push(format!(
                    "duplicate-compatibility: {} has lines {} in {}",
                    name,
                    lines.into_iter().collect::<Vec<_>>().join(","),
                    graph.name
                ));
            } else {
                used.insert(name.to_string());
            }
        }
    }
    for name in approved.keys() {
        if !used.contains(name) {
            findings.push(format!(
                "exception-unused: duplicate compatibility record {name}"
            ));
        }
    }
}

fn validate_critical(
    object: &Map<String, Value>,
    graphs: &[Graph],
    today: &str,
    findings: &mut Vec<String>,
) {
    let Some(release) = graphs.iter().find(|g| g.name == "windows-release") else {
        findings.push("graph-drift: windows-release graph missing".to_string());
        return;
    };
    let mut names = BTreeSet::new();
    for value in object
        .get("critical_dependencies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(item) = value.as_object() else {
            findings.push("exception-schema: critical dependency is not an object".to_string());
            continue;
        };
        check_keys(
            item,
            &[
                "cadence_days",
                "compatibility_boundary",
                "default_features",
                "emergency_expectation",
                "features",
                "last_reviewed",
                "name",
                "owner",
                "version",
            ],
            "critical dependency",
            findings,
        );
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let version = item.get("version").and_then(Value::as_str).unwrap_or("");
        validate_maintenance(
            item,
            today,
            &format!("critical dependency {name}"),
            findings,
        );
        if item.get("default_features").and_then(Value::as_bool) != Some(false) {
            findings.push(format!(
                "critical-default-features: {name} must disable defaults"
            ));
        }
        for field in ["compatibility_boundary", "emergency_expectation"] {
            if item
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                findings.push(format!(
                    "exception-schema: critical dependency {name} lacks {field}"
                ));
            }
        }
        if !names.insert(name.to_string()) {
            findings.push(format!(
                "exception-schema: duplicate critical dependency {name}"
            ));
        }
        let matches: Vec<&Package> = release
            .packages
            .values()
            .filter(|p| p.name == name)
            .collect();
        if matches.len() != 1 || matches[0].version != version {
            findings.push(format!("critical-pin: {name} expected {version}"));
            continue;
        }
        if strings(item.get("features")) != matches[0].features {
            findings.push(format!("critical-feature: {name} resolved feature drift"));
        }
    }
    for value in object
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(item) = value.as_object() else {
            findings.push("exception-schema: tool record is not an object".to_string());
            continue;
        };
        check_keys(
            item,
            &["cadence_days", "last_reviewed", "name", "owner", "version"],
            "tool record",
            findings,
        );
        let label = format!(
            "tool {}",
            item.get("name")
                .and_then(Value::as_str)
                .unwrap_or("missing")
        );
        validate_maintenance(item, today, &label, findings);
    }
}

fn validate_unsafe_review(
    object: &Map<String, Value>,
    graphs: &[Graph],
    today: &str,
    findings: &mut Vec<String>,
) {
    let Some(review) = object.get("unsafe_review").and_then(Value::as_object) else {
        findings.push("unsafe-review-drift: unsafe review is missing".to_string());
        return;
    };
    check_keys(
        review,
        &[
            "expires",
            "graph_digest",
            "graph_view",
            "owner",
            "rationale",
            "reviewed",
        ],
        "unsafe review",
        findings,
    );
    validate_governance_dates(review, "reviewed", today, "unsafe review", findings);
    let view = review
        .get("graph_view")
        .and_then(Value::as_str)
        .unwrap_or("");
    let expected = review
        .get("graph_digest")
        .and_then(Value::as_str)
        .unwrap_or("");
    if graphs
        .iter()
        .find(|g| g.name == view)
        .map(|g| g.digest.as_str())
        != Some(expected)
    {
        findings.push("unsafe-review-drift: reviewed graph digest no longer matches".to_string());
    }
}

fn validate_exceptions(object: &Map<String, Value>, today: &str, findings: &mut Vec<String>) {
    let mut ids = BTreeSet::new();
    for value in object
        .get("exceptions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(item) = value.as_object() else {
            findings.push("exception-schema: exception is not an object".to_string());
            continue;
        };
        check_keys(
            item,
            &[
                "created",
                "expires",
                "id",
                "owner",
                "package",
                "rationale",
                "removal_condition",
                "rule",
            ],
            "exception",
            findings,
        );
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        if id.is_empty() || !ids.insert(id.to_string()) {
            findings.push(format!(
                "exception-schema: duplicate or empty exception id {id}"
            ));
        }
        let package = item.get("package").and_then(Value::as_str).unwrap_or("");
        if package.is_empty() || package.contains('*') {
            findings.push(format!(
                "exception-schema: {id} package scope must be exact"
            ));
        }
        validate_governance(item, today, &format!("exception {id}"), findings);
        let rule = item.get("rule").and_then(Value::as_str).unwrap_or("");
        let used = matches!((rule, package), ("tool-yanked", "xml-rs@0.8.19"));
        if !used {
            findings.push(format!("exception-unused: {id}"));
        }
    }
}

fn validate_repository_wiring(root: &Path, policy: &Value) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    findings.extend(validate_critical_declarations(root, policy)?);
    let audit =
        fs::read_to_string(root.join(".github/workflows/audit.yml")).map_err(|e| e.to_string())?;
    let release = fs::read_to_string(root.join(".github/workflows/release.yml"))
        .map_err(|e| e.to_string())?;
    let wix = fs::read_to_string(root.join("crates/fragcap-cli/wix/main.wxs"))
        .map_err(|e| e.to_string())?;
    let procedure =
        fs::read_to_string(root.join("docs/maintainers/supply-chain.md")).unwrap_or_default();
    for required in [
        "pull_request:",
        "branches: [main]",
        "schedule:",
        "workflow_dispatch:",
    ] {
        if !audit.contains(required) {
            findings.push(format!("workflow-trigger: audit.yml lacks {required}"));
        }
    }
    for required in [
        "EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25",
        "version: 0.20.2",
        "rust-version: 1.88.0",
        "--all-features",
    ] {
        if !audit.contains(required) {
            findings.push(format!("workflow-tool-pin: audit.yml lacks {required}"));
        }
    }
    let tools = policy
        .get("tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for tool in tools {
        let name = tool.get("name").and_then(Value::as_str).unwrap_or("");
        let version = tool.get("version").and_then(Value::as_str).unwrap_or("");
        if name != "cargo-deny" && (!release.contains(name) || !release.contains(version)) {
            findings.push(format!(
                "workflow-tool-pin: release.yml lacks {name} {version}"
            ));
        }
    }
    let generation = release.find("Generate dependency evidence");
    let validation = release.find("stamp-evidence");
    let msi = release.find("Build the MSI installer");
    let archive = release.find("Assemble the distribution archive");
    if !matches!((generation, validation, msi, archive), (Some(g), Some(v), Some(m), Some(a)) if g < v && v < m && v < a)
    {
        findings.push("workflow-order: evidence must be generated and validated before MSI and archive assembly".to_string());
    }
    for file in ["fragcap.cdx.json", "THIRD-PARTY-NOTICES.txt"] {
        if !release.contains(file) || !wix.contains(file) {
            findings.push(format!(
                "artifact-wiring: {file} is not in release and WiX inputs"
            ));
        }
    }
    for required in [
        "Routine update",
        "Emergency advisory",
        "Rollback",
        "cargo xtask supply-chain",
    ] {
        if !procedure.contains(required) {
            findings.push(format!(
                "workflow-order: maintenance procedure lacks {required}"
            ));
        }
    }
    Ok(findings)
}

fn validate_critical_declarations(root: &Path, policy: &Value) -> Result<Vec<String>, String> {
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--no-deps",
        ])
        .output()
        .map_err(|e| format!("cargo metadata for declarations: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata for declarations failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let dependencies: Vec<&Value> = metadata
        .get("packages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|package| {
            package
                .get("dependencies")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|dependency| dependency.get("kind").is_none_or(Value::is_null))
        .collect();
    let mut findings = Vec::new();
    for critical in policy
        .get("critical_dependencies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = critical.get("name").and_then(Value::as_str).unwrap_or("");
        let version = critical
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("");
        let declarations: Vec<&&Value> = dependencies
            .iter()
            .filter(|dependency| dependency.get("name").and_then(Value::as_str) == Some(name))
            .collect();
        if declarations.is_empty() {
            findings.push(format!(
                "critical-declaration: {name} is not a direct runtime dependency"
            ));
        }
        let expected = format!("={version}");
        for declaration in declarations {
            if declaration.get("req").and_then(Value::as_str) != Some(expected.as_str()) {
                findings.push(format!(
                    "critical-declaration: {name} is not exact-pinned at {version}"
                ));
            }
            if declaration
                .get("uses_default_features")
                .and_then(Value::as_bool)
                != Some(false)
            {
                findings.push(format!(
                    "critical-default-features: {name} enables defaults"
                ));
            }
        }
    }
    Ok(findings)
}

fn stamp_evidence(root: &Path, sbom_path: &Path, notices_path: &Path) -> Result<(), String> {
    let policy = fs::read(root.join(POLICY_REL)).map_err(|e| e.to_string())?;
    let policy_document: Value = serde_json::from_slice(&policy).map_err(|e| e.to_string())?;
    let lock = fs::read(root.join("Cargo.lock")).map_err(|e| e.to_string())?;
    let version = workspace_version(root)?;
    let revision = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local-validation".to_string());
    let epoch = std::env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| "0".to_string());
    let sbom_tool = format!(
        "cargo-cyclonedx {}",
        policy_tool_version(&policy_document, "cargo-cyclonedx")?
    );
    let notices_tool = format!(
        "cargo-about {}",
        policy_tool_version(&policy_document, "cargo-about")?
    );
    let fields = [
        ("fragcap:evidence-schema", "1"),
        ("fragcap:completeness", "complete"),
        ("fragcap:version", version.as_str()),
        ("fragcap:source-revision", revision.as_str()),
        ("fragcap:source-date-epoch", epoch.as_str()),
        ("fragcap:sbom-tool", sbom_tool.as_str()),
        ("fragcap:notices-tool", notices_tool.as_str()),
        ("fragcap:target", WINDOWS_TARGET),
        ("fragcap:features", "etw,live,socket-table"),
        ("fragcap:lock-sha256", &sha256_hex(&lock)),
        ("fragcap:policy-sha256", &sha256_hex(&policy)),
    ];
    let release = metadata_graph(root, "windows-release", WINDOWS_TARGET, false, true)?;
    let allowed: BTreeSet<(String, String)> = release
        .packages
        .values()
        .map(|package| (package.name.clone(), package.version.clone()))
        .collect();
    let mut sbom: Value = serde_json::from_slice(&fs::read(sbom_path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    filter_sbom(&mut sbom, &allowed)?;
    canonicalize_sbom_refs(&mut sbom)?;
    filter_sbom_edges(&mut sbom, &release)?;
    let metadata = sbom
        .as_object_mut()
        .ok_or("SBOM root is not an object")?
        .entry("metadata")
        .or_insert_with(|| json!({}));
    let metadata_obj = metadata
        .as_object_mut()
        .ok_or("SBOM metadata is not an object")?;
    metadata_obj.insert(
        "properties".to_string(),
        Value::Array(
            fields
                .iter()
                .map(|(name, value)| json!({"name": name, "value": value}))
                .collect(),
        ),
    );
    fs::write(
        sbom_path,
        serde_json::to_vec_pretty(&sbom).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let body = fs::read_to_string(notices_path).map_err(|e| e.to_string())?;
    let mut header = String::from("fragcap dependency evidence\n");
    for (name, value) in fields {
        header.push_str(&format!("{name}: {value}\n"));
    }
    header.push_str("\nShipped third-party components:\n");
    for package in release
        .packages
        .values()
        .filter(|package| package.source.is_some())
    {
        header.push_str(&format!(
            "COMPONENT: {} {}\n",
            package.name, package.version
        ));
    }
    header.push('\n');
    header.push_str(body.trim_start());
    fs::write(notices_path, header).map_err(|e| e.to_string())?;
    Ok(())
}

fn filter_sbom(sbom: &mut Value, allowed: &BTreeSet<(String, String)>) -> Result<(), String> {
    let root = sbom.as_object_mut().ok_or("SBOM root is not an object")?;
    let components = root
        .get_mut("components")
        .and_then(Value::as_array_mut)
        .ok_or("SBOM components are missing")?;
    components.retain(|component| component_allowed(component, allowed));
    let mut allowed_refs: BTreeSet<String> = components
        .iter()
        .filter_map(|component| component.get("bom-ref").and_then(Value::as_str))
        .map(str::to_string)
        .collect();
    if let Some(root_ref) = root
        .get("metadata")
        .and_then(|metadata| metadata.get("component"))
        .and_then(|component| component.get("bom-ref"))
        .and_then(Value::as_str)
    {
        allowed_refs.insert(root_ref.to_string());
    }
    if let Some(dependencies) = root.get_mut("dependencies").and_then(Value::as_array_mut) {
        dependencies.retain_mut(|dependency| {
            let Some(reference) = dependency.get("ref").and_then(Value::as_str) else {
                return false;
            };
            if !allowed_refs.contains(reference) {
                return false;
            }
            if let Some(depends_on) = dependency
                .get_mut("dependsOn")
                .and_then(Value::as_array_mut)
            {
                depends_on.retain(|value| {
                    value
                        .as_str()
                        .is_some_and(|item| allowed_refs.contains(item))
                });
            }
            true
        });
    }
    Ok(())
}

fn component_allowed(component: &Value, allowed: &BTreeSet<(String, String)>) -> bool {
    match (
        component.get("name").and_then(Value::as_str),
        component.get("version").and_then(Value::as_str),
    ) {
        (Some(name), Some(version)) => allowed.contains(&(name.to_string(), version.to_string())),
        _ => false,
    }
}

fn canonicalize_sbom_refs(sbom: &mut Value) -> Result<(), String> {
    let mut replacements = BTreeMap::new();
    if let Some(components) = sbom.get("components").and_then(Value::as_array) {
        collect_ref_replacements(components, &mut replacements)?;
    }
    if let Some(component) = sbom
        .get("metadata")
        .and_then(|value| value.get("component"))
    {
        collect_ref_replacement(component, &mut replacements)?;
    }
    if let Some(components) = sbom.get_mut("components").and_then(Value::as_array_mut) {
        rewrite_component_refs(components, &replacements)?;
    }
    if let Some(component) = sbom
        .get_mut("metadata")
        .and_then(|value| value.get_mut("component"))
    {
        rewrite_component_ref(component, &replacements)?;
    }
    for dependency in sbom
        .get_mut("dependencies")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        let object = dependency
            .as_object_mut()
            .ok_or("SBOM dependency is not an object")?;
        if let Some(reference) = object.get_mut("ref") {
            rewrite_ref_value(reference, &replacements)?;
        }
        for reference in object
            .get_mut("dependsOn")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
        {
            rewrite_ref_value(reference, &replacements)?;
        }
    }
    Ok(())
}

fn collect_ref_replacements(
    components: &[Value],
    replacements: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for component in components {
        collect_ref_replacement(component, replacements)?;
        if let Some(children) = component.get("components").and_then(Value::as_array) {
            collect_ref_replacements(children, replacements)?;
        }
    }
    Ok(())
}

fn collect_ref_replacement(
    component: &Value,
    replacements: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let old = component
        .get("bom-ref")
        .and_then(Value::as_str)
        .ok_or("SBOM component lacks bom-ref")?;
    let purl = component
        .get("purl")
        .and_then(Value::as_str)
        .ok_or("SBOM component lacks purl")?;
    let canonical = purl.split('?').next().unwrap_or("");
    if !canonical.starts_with("pkg:cargo/") || !canonical.contains('@') {
        return Err(format!("SBOM component has unusable purl {purl}"));
    }
    replacements.insert(old.to_string(), canonical.to_string());
    Ok(())
}

fn rewrite_component_refs(
    components: &mut [Value],
    replacements: &BTreeMap<String, String>,
) -> Result<(), String> {
    for component in components {
        rewrite_component_ref(component, replacements)?;
        if let Some(children) = component
            .get_mut("components")
            .and_then(Value::as_array_mut)
        {
            rewrite_component_refs(children, replacements)?;
        }
    }
    Ok(())
}

fn rewrite_component_ref(
    component: &mut Value,
    replacements: &BTreeMap<String, String>,
) -> Result<(), String> {
    let object = component
        .as_object_mut()
        .ok_or("SBOM component is not an object")?;
    let old = object
        .get("bom-ref")
        .and_then(Value::as_str)
        .ok_or("SBOM component lacks bom-ref")?;
    let canonical = replacements
        .get(old)
        .ok_or_else(|| format!("SBOM component reference has no replacement: {old}"))?
        .clone();
    object.insert("bom-ref".to_string(), Value::String(canonical.clone()));
    object.insert("purl".to_string(), Value::String(canonical));
    Ok(())
}

fn rewrite_ref_value(
    value: &mut Value,
    replacements: &BTreeMap<String, String>,
) -> Result<(), String> {
    let old = value
        .as_str()
        .ok_or("SBOM dependency reference is not a string")?;
    let replacement = replacements
        .get(old)
        .ok_or_else(|| format!("SBOM dependency reference is dangling: {old}"))?
        .clone();
    *value = Value::String(replacement);
    Ok(())
}

fn filter_sbom_edges(sbom: &mut Value, release: &Graph) -> Result<(), String> {
    let mut identities = BTreeMap::new();
    let mut findings = Vec::new();
    if let Some(components) = sbom.get("components").and_then(Value::as_array) {
        collect_sbom_identities(components, &mut identities, &mut findings);
    }
    if let Some(component) = sbom
        .get("metadata")
        .and_then(|value| value.get("component"))
    {
        collect_sbom_identity(component, &mut identities, &mut findings);
    }
    if !findings.is_empty() {
        return Err(findings.join("; "));
    }
    let expected = expected_dependency_edges(release);
    for dependency in sbom
        .get_mut("dependencies")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        let Some(from) = dependency
            .get("ref")
            .and_then(Value::as_str)
            .and_then(|reference| identities.get(reference))
            .cloned()
        else {
            continue;
        };
        if let Some(targets) = dependency
            .get_mut("dependsOn")
            .and_then(Value::as_array_mut)
        {
            targets.retain(|target| {
                target
                    .as_str()
                    .and_then(|reference| identities.get(reference))
                    .is_some_and(|to| expected.contains(&format!("{from}->{to}")))
            });
        }
    }
    Ok(())
}

fn validate_evidence(root: &Path, sbom_path: &Path, notices_path: &Path) -> Result<usize, String> {
    let policy_bytes = fs::read(root.join(POLICY_REL)).map_err(|e| e.to_string())?;
    let policy: Value = serde_json::from_slice(&policy_bytes).map_err(|e| e.to_string())?;
    let lock = fs::read(root.join("Cargo.lock")).map_err(|e| e.to_string())?;
    let release = metadata_graph(root, "windows-release", WINDOWS_TARGET, false, true)?;
    let expected: BTreeSet<String> = release
        .packages
        .values()
        .filter(|p| p.source.is_some())
        .map(|p| format!("{} {}", p.name, p.version))
        .collect();
    let mut findings = Vec::new();
    let sbom_bytes = fs::read(sbom_path).map_err(|e| e.to_string())?;
    let sbom: Value = serde_json::from_slice(&sbom_bytes).map_err(|e| e.to_string())?;
    if sbom.get("specVersion").and_then(Value::as_str) != Some("1.5") {
        findings.push("sbom-schema: specVersion must be 1.5".to_string());
    }
    let mut observed = BTreeMap::<String, usize>::new();
    collect_components(sbom.get("components"), &mut observed);
    if let Some(component) = sbom.get("metadata").and_then(|v| v.get("component")) {
        collect_component(component, &mut observed);
    }
    for package in &expected {
        match observed.get(package).copied().unwrap_or(0) {
            1 => {}
            0 => findings.push(format!("sbom-component: missing {package}")),
            count => findings.push(format!(
                "sbom-component: duplicated {package} {count} times"
            )),
        }
    }
    for package in observed.keys().filter(|p| !p.starts_with("fragcap")) {
        if !expected.contains(package) {
            findings.push(format!("sbom-component: unexpected {package}"));
        }
    }
    validate_sbom_dependencies(&sbom, &release, &mut findings);
    let properties = sbom
        .get("metadata")
        .and_then(|v| v.get("properties"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let props: BTreeMap<String, String> = properties
        .into_iter()
        .filter_map(|p| {
            Some((
                p.get("name")?.as_str()?.to_string(),
                p.get("value")?.as_str()?.to_string(),
            ))
        })
        .collect();
    for (name, value) in [
        ("fragcap:evidence-schema", "1".to_string()),
        ("fragcap:completeness", "complete".to_string()),
        ("fragcap:version", workspace_version(root)?),
        (
            "fragcap:sbom-tool",
            format!(
                "cargo-cyclonedx {}",
                policy_tool_version(&policy, "cargo-cyclonedx")?
            ),
        ),
        (
            "fragcap:notices-tool",
            format!(
                "cargo-about {}",
                policy_tool_version(&policy, "cargo-about")?
            ),
        ),
        ("fragcap:target", WINDOWS_TARGET.to_string()),
        ("fragcap:features", "etw,live,socket-table".to_string()),
        ("fragcap:lock-sha256", sha256_hex(&lock)),
        ("fragcap:policy-sha256", sha256_hex(&policy_bytes)),
    ] {
        if props.get(name) != Some(&value) {
            findings.push(format!("sbom-identity: {name} is missing or stale"));
        }
    }
    let revision = props
        .get("fragcap:source-revision")
        .map(String::as_str)
        .unwrap_or("");
    if revision.is_empty() {
        findings.push("sbom-identity: source revision is missing".to_string());
    }
    let epoch = props
        .get("fragcap:source-date-epoch")
        .and_then(|value| value.parse::<u64>().ok());
    let timestamp = sbom
        .get("metadata")
        .and_then(|value| value.get("timestamp"))
        .and_then(Value::as_str)
        .unwrap_or("");
    match epoch {
        Some(epoch) if timestamp.starts_with(&civil_from_days((epoch / 86_400) as i64)) => {}
        _ => findings.push("sbom-identity: source date epoch and timestamp disagree".to_string()),
    }
    if sbom.get("serialNumber").is_some() {
        findings.push("sbom-identity: nondeterministic serial number is present".to_string());
    }
    let notices = fs::read_to_string(notices_path).map_err(|e| e.to_string())?;
    for package in &expected {
        let marker = format!("COMPONENT: {package}");
        match notices.match_indices(&marker).count() {
            1 => {}
            0 => findings.push(format!("notices-component: missing {package}")),
            count => findings.push(format!(
                "notices-component: duplicated {package} {count} times"
            )),
        }
    }
    if notices
        .lines()
        .any(|line| line.starts_with("COMPONENT: fragcap"))
    {
        findings.push("notices-component: first-party workspace package present".to_string());
    }
    let notice_packages: BTreeSet<String> = notices
        .lines()
        .filter_map(|line| line.strip_prefix("USED BY: "))
        .map(str::to_string)
        .collect();
    for package in expected.symmetric_difference(&notice_packages) {
        findings.push(format!(
            "notices-license: component set differs at {package}"
        ));
    }
    let sbom_text = String::from_utf8_lossy(&sbom_bytes);
    if sbom_text.contains(root.to_string_lossy().as_ref())
        || sbom_text.contains("file://")
        || sbom_text.contains("C:\\Users\\")
        || sbom_text.contains("/home/")
    {
        findings.push("sbom-identity: absolute operator path present".to_string());
    }
    if notices.contains(root.to_string_lossy().as_ref())
        || notices.contains("C:\\Users\\")
        || notices.contains("/home/")
    {
        findings.push("notices-identity: absolute operator path present".to_string());
    }
    for (name, value) in &props {
        if name.starts_with("fragcap:") && !notices.contains(&format!("{name}: {value}")) {
            findings.push(format!("notices-identity: {name} is missing or stale"));
        }
    }
    normalize_findings(&mut findings);
    for finding in &findings {
        eprintln!("supply-chain: {finding}");
    }
    Ok(findings.len())
}

fn validate_sbom_dependencies(sbom: &Value, release: &Graph, findings: &mut Vec<String>) {
    let mut identities = BTreeMap::new();
    if let Some(components) = sbom.get("components").and_then(Value::as_array) {
        collect_sbom_identities(components, &mut identities, findings);
    }
    if let Some(component) = sbom
        .get("metadata")
        .and_then(|value| value.get("component"))
    {
        collect_sbom_identity(component, &mut identities, findings);
    }
    let mut observed = BTreeSet::new();
    let mut rows = BTreeSet::new();
    for dependency in sbom
        .get("dependencies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(reference) = dependency.get("ref").and_then(Value::as_str) else {
            findings.push("sbom-dependency: row lacks ref".to_string());
            continue;
        };
        if !rows.insert(reference.to_string()) {
            findings.push(format!("sbom-dependency: duplicate row {reference}"));
        }
        let Some(from) = identities.get(reference) else {
            findings.push(format!("sbom-dependency: unknown source {reference}"));
            continue;
        };
        for target in dependency
            .get("dependsOn")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(target) = target.as_str() else {
                findings.push(format!(
                    "sbom-dependency: non-string target from {reference}"
                ));
                continue;
            };
            let Some(to) = identities.get(target) else {
                findings.push(format!("sbom-dependency: unknown target {target}"));
                continue;
            };
            observed.insert(format!("{from}->{to}"));
        }
    }
    for reference in identities
        .keys()
        .filter(|reference| !rows.contains(*reference))
    {
        findings.push(format!("sbom-dependency: missing row {reference}"));
    }
    let expected = expected_dependency_edges(release);
    for edge in expected.difference(&observed) {
        findings.push(format!("sbom-dependency: missing edge {edge}"));
    }
    for edge in observed.difference(&expected) {
        findings.push(format!("sbom-dependency: unexpected edge {edge}"));
    }
}

fn expected_dependency_edges(release: &Graph) -> BTreeSet<String> {
    release
        .edges
        .iter()
        .flat_map(|edge| {
            release.packages.values().flat_map(move |from| {
                let tail = edge.strip_prefix(&format!("{}->", from.key));
                release.packages.values().filter_map(move |to| {
                    tail.filter(|value| value.starts_with(&format!("{}|", to.key)))?;
                    Some(format!(
                        "{} {}->{} {}",
                        from.name, from.version, to.name, to.version
                    ))
                })
            })
        })
        .collect()
}

fn collect_sbom_identities(
    components: &[Value],
    identities: &mut BTreeMap<String, String>,
    findings: &mut Vec<String>,
) {
    for component in components {
        collect_sbom_identity(component, identities, findings);
        if let Some(children) = component.get("components").and_then(Value::as_array) {
            collect_sbom_identities(children, identities, findings);
        }
    }
}

fn collect_sbom_identity(
    component: &Value,
    identities: &mut BTreeMap<String, String>,
    findings: &mut Vec<String>,
) {
    let Some(reference) = component.get("bom-ref").and_then(Value::as_str) else {
        findings.push("sbom-dependency: component lacks bom-ref".to_string());
        return;
    };
    let Some(package) = reference.strip_prefix("pkg:cargo/") else {
        findings.push(format!("sbom-dependency: noncanonical ref {reference}"));
        return;
    };
    let Some((name, version)) = package.rsplit_once('@') else {
        findings.push(format!("sbom-dependency: noncanonical ref {reference}"));
        return;
    };
    if identities
        .insert(reference.to_string(), format!("{name} {version}"))
        .is_some()
    {
        findings.push(format!(
            "sbom-dependency: duplicate component ref {reference}"
        ));
    }
}

fn collect_components(value: Option<&Value>, observed: &mut BTreeMap<String, usize>) {
    for component in value.and_then(Value::as_array).into_iter().flatten() {
        collect_component(component, observed);
        collect_components(component.get("components"), observed);
    }
}

fn collect_component(component: &Value, observed: &mut BTreeMap<String, usize>) {
    if let (Some(name), Some(version)) = (
        component.get("name").and_then(Value::as_str),
        component.get("version").and_then(Value::as_str),
    ) {
        *observed.entry(format!("{name} {version}")).or_default() += 1;
    }
}

fn policy_tool_version(policy: &Value, name: &str) -> Result<String, String> {
    policy
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|tool| tool.get("version"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("policy tool {name} is missing"))
}

fn validate_governance(
    item: &Map<String, Value>,
    today: &str,
    label: &str,
    findings: &mut Vec<String>,
) {
    for key in ["owner", "rationale", "removal_condition"] {
        if item
            .get(key)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            findings.push(format!("exception-schema: {label} lacks {key}"));
        }
    }
    validate_governance_dates(item, "created", today, label, findings);
}

fn validate_governance_dates(
    item: &Map<String, Value>,
    start_key: &str,
    today: &str,
    label: &str,
    findings: &mut Vec<String>,
) {
    let start = item.get(start_key).and_then(Value::as_str).unwrap_or("");
    let expires = item.get("expires").and_then(Value::as_str).unwrap_or("");
    if !valid_date(start) || !valid_date(expires) || start > expires {
        findings.push(format!("exception-schema: {label} has invalid dates"));
    } else if expires < today {
        findings.push(format!("exception-expired: {label} expired {expires}"));
    }
}

fn validate_maintenance(
    item: &Map<String, Value>,
    today: &str,
    label: &str,
    findings: &mut Vec<String>,
) {
    for key in ["name", "version", "owner"] {
        if item
            .get(key)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            findings.push(format!("exception-schema: {label} lacks {key}"));
        }
    }
    let reviewed = item
        .get("last_reviewed")
        .and_then(Value::as_str)
        .unwrap_or("");
    let cadence = item
        .get("cadence_days")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if !valid_date(reviewed) || cadence == 0 {
        findings.push(format!(
            "exception-schema: {label} has invalid maintenance cadence"
        ));
    } else if days_between(reviewed, today).is_none_or(|days| days > cadence as i64) {
        findings.push(format!("critical-review-expired: {label}"));
    }
}

fn check_keys(
    object: &Map<String, Value>,
    expected: &[&str],
    label: &str,
    findings: &mut Vec<String>,
) {
    let expected: BTreeSet<&str> = expected.iter().copied().collect();
    for key in object.keys() {
        if !expected.contains(key.as_str()) {
            findings.push(format!("exception-schema: {label} has unknown field {key}"));
        }
    }
    for key in expected {
        if !object.contains_key(key) {
            findings.push(format!("exception-schema: {label} lacks {key}"));
        }
    }
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{key} is missing"))
}

fn string(object: &Map<String, Value>, key: &str) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{key} is missing"))
}

fn optional_string(object: &Map<String, Value>, key: &str) -> Option<String> {
    object.get(key).and_then(Value::as_str).map(str::to_string)
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn compatibility_line(version: &str) -> String {
    let mut parts = version.split('.');
    let major = parts.next().unwrap_or("0");
    if major == "0" {
        format!("0.{}", parts.next().unwrap_or("0"))
    } else {
        major.to_string()
    }
}

fn version_gt(left: &str, right: &str) -> bool {
    let parse = |text: &str| {
        let mut parts = text
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(u64::MAX));
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    };
    parse(left) > parse(right)
}

fn valid_date(value: &str) -> bool {
    if value.len() != 10
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
    {
        return false;
    }
    let year = value[0..4].parse::<i64>().ok();
    let month = value[5..7].parse::<u32>().ok();
    let day = value[8..10].parse::<u32>().ok();
    matches!((year, month, day), (Some(y), Some(m), Some(d)) if y >= 1970 && (1..=12).contains(&m) && (1..=days_in_month(y, m)).contains(&d))
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 31,
    }
}

fn days_between(from: &str, to: &str) -> Option<i64> {
    Some(date_days(to)? - date_days(from)?)
}

fn date_days(value: &str) -> Option<i64> {
    if !valid_date(value) {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let adjusted = year - i64::from(month <= 2);
    let era = adjusted.div_euclid(400);
    let year_of_era = adjusted - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era)
}

fn today_ymd() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86_400;
    civil_from_days(days)
}

fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn sha256_hex(bytes: &[u8]) -> String {
    digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn normalize_findings(findings: &mut Vec<String>) {
    findings.sort();
    findings.dedup();
    if findings.len() > MAX_FINDINGS {
        let omitted = findings.len() - (MAX_FINDINGS - 1);
        findings.truncate(MAX_FINDINGS - 1);
        findings.push(format!(
            "finding-limit: {omitted} additional finding(s) omitted"
        ));
    }
}

fn workspace_version(root: &Path) -> Result<String, String> {
    let manifest = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    manifest
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("version")
                .and_then(|rest| rest.split('=').nth(1))
                .map(|value| value.trim().trim_matches('"').to_string())
        })
        .ok_or("workspace version is missing".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_lines_follow_cargo_semantics() {
        assert_eq!(compatibility_line("2.3.4"), "2");
        assert_eq!(compatibility_line("0.4.3"), "0.4");
    }

    #[test]
    fn version_comparison_is_numeric() {
        assert!(version_gt("1.89", "1.88"));
        assert!(!version_gt("1.8", "1.88"));
    }

    #[test]
    fn calendar_validation_catches_invalid_and_leap_dates() {
        assert!(valid_date("2028-02-29"));
        assert!(!valid_date("2027-02-29"));
        assert!(!valid_date("2026-13-01"));
    }

    #[test]
    fn calendar_distance_matches_known_interval() {
        assert_eq!(days_between("2026-09-05", "2026-12-04"), Some(90));
    }

    #[test]
    fn unix_epoch_converts_to_expected_date() {
        assert_eq!(civil_from_days(0), "1970-01-01");
    }

    #[test]
    fn diagnostics_are_sorted_deduplicated_and_bounded() {
        let mut findings: Vec<String> = (0..300)
            .rev()
            .flat_map(|index| [format!("finding-{index:03}"), format!("finding-{index:03}")])
            .collect();
        normalize_findings(&mut findings);
        assert_eq!(findings.len(), MAX_FINDINGS);
        assert_eq!(findings.first().map(String::as_str), Some("finding-000"));
        assert_eq!(
            findings.last().map(String::as_str),
            Some("finding-limit: 45 additional finding(s) omitted")
        );
    }

    #[test]
    fn lock_checksum_parser_joins_exact_package_identity() {
        let checksums = lock_checksums(
            "[[package]]\nname = \"demo\"\nversion = \"1.2.3\"\nsource = \"registry+example\"\nchecksum = \"abc\"\n",
        )
        .unwrap();
        assert_eq!(
            checksums.get(&(
                "demo".to_string(),
                "1.2.3".to_string(),
                "registry+example".to_string()
            )),
            Some(&"abc".to_string())
        );
    }

    #[test]
    fn graph_digest_is_independent_of_insertion_order() {
        let package = Package {
            id: "id".to_string(),
            key: "demo@1.0.0|registry+example".to_string(),
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            source: Some("registry+example".to_string()),
            checksum: Some("abc".to_string()),
            license: Some("MIT".to_string()),
            rust_version: Some("1.70".to_string()),
            features: vec!["std".to_string()],
        };
        let left = BTreeMap::from([(package.key.clone(), package.clone())]);
        let right = BTreeMap::from([(package.key.clone(), package)]);
        assert_eq!(
            graph_digest(&left, &BTreeSet::new()),
            graph_digest(&right, &BTreeSet::new())
        );
    }

    #[test]
    fn sbom_filter_removes_nonrelease_components_and_edges() {
        let mut sbom = json!({
            "metadata": {"component": {"bom-ref": "root", "name": "fragcap-cli", "version": "0.9.0"}},
            "components": [
                {"bom-ref": "keep", "name": "keep", "version": "1.0.0"},
                {"bom-ref": "drop", "name": "drop", "version": "2.0.0"}
            ],
            "dependencies": [
                {"ref": "root", "dependsOn": ["keep", "drop"]},
                {"ref": "drop", "dependsOn": []}
            ]
        });
        filter_sbom(
            &mut sbom,
            &BTreeSet::from([("keep".to_string(), "1.0.0".to_string())]),
        )
        .unwrap();
        assert_eq!(sbom["components"].as_array().unwrap().len(), 1);
        assert_eq!(sbom["dependencies"].as_array().unwrap().len(), 1);
        assert_eq!(sbom["dependencies"][0]["dependsOn"], json!(["keep"]));
    }

    #[test]
    fn sbom_stamping_removes_local_paths_and_nonrelease_edges() {
        let package = |name: &str| Package {
            id: name.to_string(),
            key: format!("{name}@1.0.0|workspace"),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            source: None,
            checksum: None,
            license: Some("Apache-2.0".to_string()),
            rust_version: Some("1.88".to_string()),
            features: Vec::new(),
        };
        let packages = ["root", "keep", "shared"]
            .into_iter()
            .map(|name| {
                let package = package(name);
                (package.key.clone(), package)
            })
            .collect();
        let graph = Graph {
            name: "windows-release".to_string(),
            packages,
            edges: BTreeSet::from([
                "root@1.0.0|workspace->keep@1.0.0|workspace|keep|normal|all".to_string(),
                "root@1.0.0|workspace->shared@1.0.0|workspace|shared|normal|all".to_string(),
            ]),
            digest: String::new(),
        };
        let mut sbom = json!({
            "metadata": {"component": {
                "bom-ref": "path+file:///workspace/root#1.0.0", "name": "root",
                "version": "1.0.0", "purl": "pkg:cargo/root@1.0.0?download_url=file://."
            }},
            "components": [
                {"bom-ref": "path+file:///workspace/keep#1.0.0", "name": "keep", "version": "1.0.0", "purl": "pkg:cargo/keep@1.0.0?download_url=file://keep"},
                {"bom-ref": "path+file:///workspace/shared#1.0.0", "name": "shared", "version": "1.0.0", "purl": "pkg:cargo/shared@1.0.0?download_url=file://shared"}
            ],
            "dependencies": [
                {"ref": "path+file:///workspace/root#1.0.0", "dependsOn": ["path+file:///workspace/keep#1.0.0", "path+file:///workspace/shared#1.0.0"]},
                {"ref": "path+file:///workspace/keep#1.0.0", "dependsOn": ["path+file:///workspace/shared#1.0.0"]},
                {"ref": "path+file:///workspace/shared#1.0.0", "dependsOn": []}
            ]
        });
        canonicalize_sbom_refs(&mut sbom).unwrap();
        filter_sbom_edges(&mut sbom, &graph).unwrap();
        let text = serde_json::to_string(&sbom).unwrap();
        assert!(!text.contains("file://"));
        assert_eq!(sbom["dependencies"][1]["dependsOn"], json!([]));
        let mut findings = Vec::new();
        validate_sbom_dependencies(&sbom, &graph, &mut findings);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn only_the_exact_governed_tool_exception_is_used() {
        let valid = Map::from_iter([(
            "exceptions".to_string(),
            json!([{
                "id": "S130-TOOL-001", "rule": "tool-yanked", "package": "xml-rs@0.8.19",
                "owner": "maintainers", "rationale": "generator only", "created": "2026-09-05",
                "expires": "2026-12-04", "removal_condition": "upstream release"
            }]),
        )]);
        let mut findings = Vec::new();
        validate_exceptions(&valid, "2026-09-05", &mut findings);
        assert!(findings.is_empty());

        let wildcard = Map::from_iter([(
            "exceptions".to_string(),
            json!([{
                "id": "bad", "rule": "tool-yanked", "package": "xml-rs@*",
                "owner": "maintainers", "rationale": "too broad", "created": "2026-09-05",
                "expires": "2026-12-04", "removal_condition": "unknown"
            }]),
        )]);
        validate_exceptions(&wildcard, "2026-09-05", &mut findings);
        assert!(findings
            .iter()
            .any(|finding| finding.contains("scope must be exact")));
        assert!(findings
            .iter()
            .any(|finding| finding.contains("exception-unused")));
    }

    #[test]
    fn policy_rejects_unknown_fields_and_expired_records() {
        let policy = json!({
            "schema_version": 1,
            "release_root": "fragcap-cli",
            "release_target": WINDOWS_TARGET,
            "release_features": ["etw", "live", "socket-table"],
            "workspace_msrv": "1.88",
            "graph_views": [],
            "allowed_sources": [],
            "tools": [],
            "critical_dependencies": [],
            "duplicate_compatibility_exceptions": [{
                "name": "x", "lines": ["1", "2"], "owner": "o", "rationale": "r",
                "created": "2026-01-01", "expires": "2026-02-01", "removal_condition": "gone"
            }],
            "unsafe_review": {"graph_view": "windows-all", "graph_digest": "x", "owner": "o", "rationale": "r", "reviewed": "2026-01-01", "expires": "2026-02-01"},
            "exceptions": [],
            "surprise": true
        });
        let findings = validate_policy(&policy, &[], "2026-09-05".to_string());
        assert!(findings
            .iter()
            .any(|f| f.contains("unknown field surprise")));
        assert!(findings.iter().any(|f| f.contains("exception-expired")));
    }
}
