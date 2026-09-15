// SPDX-License-Identifier: Apache-2.0

//! Static review readiness, deliberately not an independent audit verdict.

use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    process::{Command, Output, Stdio},
};

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
    let mut problems = validate(root, &scope, &template);
    if problems.is_empty() {
        let inventory = EvidenceInventory::read(root)?;
        problems.extend(validate_executable_evidence(root, &scope, &inventory)?);
    }
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
            exact_keys(test, &["path", "function", "features"], id, &mut problems);
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
    let version = crate::spec::workspace_version(root);
    if !version.is_some_and(|version| template_matches_version(template, &version)) {
        problems.push("committed record must remain the exact not-started template with no invented provenance, findings, or approval".into());
    }
    problems
}

fn template_matches_version(template: &Value, version: &str) -> bool {
    let expected = serde_json::json!({"schema_version":1,"state":"not-started","candidate":{"product_version":version,"source_revision":null,"lockfile_sha256":null,"packages":[]},"reviewer":null,"independence":null,"environment":null,"checks":[],"findings":[],"summary":null});
    *template == expected
}

#[derive(Clone)]
struct EvidenceInventory {
    tracked: BTreeSet<String>,
    packages: Vec<EvidencePackage>,
}

#[derive(Clone)]
struct EvidencePackage {
    name: String,
    features: BTreeSet<String>,
    targets: Vec<EvidenceTarget>,
}

#[derive(Clone)]
struct EvidenceTarget {
    name: String,
    source: String,
    kind: String,
}

impl EvidenceInventory {
    fn read(root: &Path) -> io::Result<Self> {
        let tracked = hidden_output(root, "git", &["ls-files", "-z"])?;
        let tracked = String::from_utf8(tracked.stdout)
            .map_err(io::Error::other)?
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect();
        let output = hidden_output(
            root,
            env!("CARGO"),
            &[
                "metadata",
                "--locked",
                "--offline",
                "--format-version",
                "1",
                "--no-deps",
            ],
        )?;
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        let boundary = root.canonicalize()?;
        let mut packages = Vec::new();
        for package in metadata["packages"].as_array().into_iter().flatten() {
            let mut targets = Vec::new();
            for target in package["targets"].as_array().into_iter().flatten() {
                let kind = target["kind"][0].as_str().unwrap_or("");
                if target["test"].as_bool() != Some(true) || !matches!(kind, "test" | "bin" | "lib")
                {
                    continue;
                }
                let source = Path::new(target["src_path"].as_str().unwrap_or("")).canonicalize()?;
                let source = source.strip_prefix(&boundary).map_err(io::Error::other)?;
                targets.push(EvidenceTarget {
                    name: target["name"].as_str().unwrap_or("").into(),
                    source: source.to_string_lossy().replace('\\', "/"),
                    kind: kind.into(),
                });
            }
            packages.push(EvidencePackage {
                name: package["name"].as_str().unwrap_or("").into(),
                features: package["features"]
                    .as_object()
                    .into_iter()
                    .flat_map(|features| features.keys().cloned())
                    .collect(),
                targets,
            });
        }
        Ok(Self { tracked, packages })
    }

    fn owner<'a>(
        &'a self,
        root: &Path,
        path: &str,
    ) -> Result<(&'a EvidencePackage, &'a EvidenceTarget, String), String> {
        if !self.tracked.contains(path) {
            return Err("evidence is not Git-tracked".into());
        }
        let resolved = root
            .join(path)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let boundary = root.canonicalize().map_err(|error| error.to_string())?;
        let resolved = resolved
            .strip_prefix(boundary)
            .map_err(|error| error.to_string())?;
        if !self
            .tracked
            .contains(&resolved.to_string_lossy().replace('\\', "/"))
        {
            return Err("resolved evidence is not Git-tracked".into());
        }
        for package in &self.packages {
            for target in &package.targets {
                if target.source == path {
                    return Ok((package, target, String::new()));
                }
                // The handoff's only module-level unit evidence is the flat xtask
                // binary module tree. Other specimens are not inferred as targets.
                if package.name == "xtask"
                    && target.source == "xtask/src/main.rs"
                    && target.kind == "bin"
                    && path.starts_with("xtask/src/")
                {
                    let relative = path.strip_prefix("xtask/src/").unwrap();
                    if let Some(module) = relative.strip_suffix(".rs") {
                        if !module.contains('/')
                            && confined_source(root, &target.source)?
                                .lines()
                                .any(|line| line.trim() == format!("mod {module};"))
                        {
                            return Ok((package, target, format!("{module}::tests::")));
                        }
                    }
                }
            }
        }
        Err("evidence is not owned by a supported Cargo test target".into())
    }
}

fn hidden_output(root: &Path, executable: &str, args: &[&str]) -> io::Result<Output> {
    let mut command = Command::new(executable);
    command
        .current_dir(root)
        .args(args)
        .stdin(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{executable} {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(output)
}

fn discovered_test(listed: &str, name: &str) -> Result<(), String> {
    if listed.lines().any(|line| line == format!("{name}: test")) {
        Ok(())
    } else {
        Err("test is absent under its declared features/enclosing configuration".into())
    }
}

fn validate_executable_evidence(
    root: &Path,
    scope: &Value,
    inventory: &EvidenceInventory,
) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    type References = Vec<(String, String, String)>;
    let mut groups: BTreeMap<(String, Vec<String>), References> = BTreeMap::new();
    for area in scope["areas"].as_array().into_iter().flatten() {
        for test in area["tests"].as_array().into_iter().flatten() {
            let path = test["path"].as_str().unwrap_or("");
            let function = test["function"].as_str().unwrap_or("");
            let (package, target, prefix) = match inventory.owner(root, path) {
                Ok(owner) => owner,
                Err(reason) => {
                    problems.push(format!("test {path}::{function}: {reason}"));
                    continue;
                }
            };
            let Some(features) = test["features"].as_array() else {
                problems.push(format!(
                    "test {path}::{function}: features must be an explicit array"
                ));
                continue;
            };
            let features: Vec<_> = features
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            if features.len() != test["features"].as_array().unwrap().len()
                || features.iter().collect::<BTreeSet<_>>().len() != features.len()
                || features
                    .iter()
                    .any(|feature| !package.features.contains(feature))
            {
                problems.push(format!(
                    "test {path}::{function}: invalid, duplicate or unknown package features"
                ));
                continue;
            }
            groups
                .entry((package.name.clone(), features))
                .or_default()
                .push((
                    target.name.clone(),
                    format!("{prefix}{function}"),
                    path.into(),
                ));
        }
    }
    if !problems.is_empty() {
        return Ok(problems);
    }
    for ((package, features), references) in groups {
        let mut args = vec![
            "test",
            "--locked",
            "--offline",
            "--no-default-features",
            "-p",
            &package,
            "--tests",
            "--no-run",
            "--message-format=json",
        ];
        let selected = features.join(",");
        if !features.is_empty() {
            args.extend(["--features", &selected]);
        }
        let output = hidden_output(root, env!("CARGO"), &args)?;
        let mut executables = BTreeMap::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Ok(artifact) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if artifact["reason"] == "compiler-artifact" && artifact["profile"]["test"] == true {
                if let (Some(name), Some(executable)) = (
                    artifact["target"]["name"].as_str(),
                    artifact["executable"].as_str(),
                ) {
                    executables.insert(name.to_string(), executable.to_string());
                }
            }
        }
        let mut listed = BTreeMap::new();
        for (target, function, path) in references {
            if !listed.contains_key(&target) {
                let Some(executable) = executables.get(&target) else {
                    problems.push(format!(
                        "test {path}::{function}: Cargo did not build target {target}"
                    ));
                    continue;
                };
                let output = hidden_output(root, executable, &["--list", "--format", "terse"])?;
                listed.insert(
                    target.clone(),
                    String::from_utf8(output.stdout).map_err(io::Error::other)?,
                );
            }
            if let Err(reason) = discovered_test(&listed[&target], &function) {
                problems.push(format!("test {path}::{function}: {reason}"));
            }
        }
    }
    Ok(problems)
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
            json!({"schema_version":1,"review_state":"not-performed","areas":AREAS.map(|id|json!({"id":id,"method":"Inspect exact source and run controlled evidence independently.","sources":["README.md"],"tests":[{"path":"xtask/src/lint.rs","function":"clean_source_produces_no_findings","features":[]}]}))}),
            json!({"schema_version":1,"state":"not-started","candidate":{"product_version":"0.10.0","source_revision":null,"lockfile_sha256":null,"packages":[]},"reviewer":null,"independence":null,"environment":null,"checks":[],"findings":[],"summary":null}),
        )
    }
    #[test]
    fn committed_handoff_is_complete_and_unperformed() {
        let scope = read(root(), REGISTRY).unwrap();
        let template = read(root(), TEMPLATE).unwrap();
        assert!(validate(root(), &scope, &template).is_empty());
        let inventory = EvidenceInventory::read(root()).unwrap();
        for area in scope["areas"].as_array().unwrap() {
            for test in area["tests"].as_array().unwrap() {
                assert!(inventory
                    .owner(root(), test["path"].as_str().unwrap())
                    .is_ok());
            }
        }
        // Full harness discovery belongs to the standalone CI command, not a
        // recursive build that may try to relink this running Windows harness.
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

    #[test]
    fn evidence_requires_tracked_cargo_ownership() {
        let inventory = EvidenceInventory {
            tracked: ["xtask/src/main.rs", "xtask/src/lint.rs"]
                .map(str::to_string)
                .into_iter()
                .collect(),
            packages: vec![EvidencePackage {
                name: "xtask".into(),
                features: BTreeSet::new(),
                targets: vec![EvidenceTarget {
                    name: "xtask".into(),
                    source: "xtask/src/main.rs".into(),
                    kind: "bin".into(),
                }],
            }],
        };
        assert!(inventory.owner(root(), "xtask/src/lint.rs").is_ok());
        for path in [
            "target/generated.rs",
            "docs/specimen.rs",
            "xtask/src/not_a_module.rs",
        ] {
            assert!(inventory.owner(root(), path).is_err());
        }
        let mut specimen = inventory.clone();
        specimen.tracked.insert("README.md".into());
        assert!(specimen.owner(root(), "README.md").is_err());
        let mut untracked = inventory;
        untracked.tracked.remove("xtask/src/lint.rs");
        assert!(untracked.owner(root(), "xtask/src/lint.rs").is_err());
    }

    #[test]
    fn compiled_discovery_rejects_disabled_enclosing_configuration() {
        let listed = "enabled: test\nmodule::tests::enabled: test\nhelper: benchmark\n";
        assert!(discovered_test(listed, "enabled").is_ok());
        assert!(discovered_test(listed, "module::tests::enabled").is_ok());
        // File-level feature gates and enclosing cfg(any()) both omit their test.
        for name in ["file_feature_disabled", "module::tests::disabled", "helper"] {
            assert!(discovered_test(listed, name).is_err());
        }
    }

    #[cfg(any())]
    mod disabled_evidence {
        #[test]
        fn enclosing_cfg_disabled_evidence() {}
    }

    #[test]
    fn actual_harness_omits_enclosing_cfg_disabled_test() {
        // The old adjacent-attribute checker accepts this source declaration.
        assert!(crate::threat_model::validate_test_source(
            include_str!("review_handoff.rs"),
            "enclosing_cfg_disabled_evidence"
        )
        .is_ok());
        let executable = std::env::current_exe().unwrap();
        let output = hidden_output(
            root(),
            executable.to_str().unwrap(),
            &["--list", "--format", "terse"],
        )
        .unwrap();
        let listed = String::from_utf8(output.stdout).unwrap();
        assert!(discovered_test(
            &listed,
            "review_handoff::tests::actual_harness_omits_enclosing_cfg_disabled_test"
        )
        .is_ok());
        assert!(discovered_test(
            &listed,
            "review_handoff::tests::disabled_evidence::enclosing_cfg_disabled_evidence"
        )
        .is_err());
    }

    #[test]
    fn actual_file_feature_gate_requires_declared_deep_capture() {
        let inventory = EvidenceInventory::read(root()).unwrap();
        let mut scope = json!({"areas":[{"tests":[{
            "path":"crates/fragcap/tests/public_api.rs",
            "function":"cli_product_contract_does_not_bypass_the_stable_module",
            "features":[]
        }]}]});
        let missing = validate_executable_evidence(root(), &scope, &inventory).unwrap();
        assert!(missing
            .iter()
            .any(|problem| problem.contains("absent under its declared features")));
        scope["areas"][0]["tests"][0]["features"] = json!(["deep-capture"]);
        assert!(validate_executable_evidence(root(), &scope, &inventory)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn candidate_version_must_follow_workspace_identity() {
        let (_, mut template) = fixture();
        template["candidate"]["product_version"] = json!("0.9.0");
        assert!(!validate(root(), &fixture().0, &template).is_empty());
        assert!(template_matches_version(&fixture().1, "0.10.0"));
        assert!(!template_matches_version(&fixture().1, "0.11.0"));
    }
}
