// SPDX-License-Identifier: Apache-2.0

//! Closed, finite Windows release evidence for native Deep Capture.

use ring::digest::{digest, SHA256};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const REGISTRY: &str = "integration/windows-native-matrix-v1.json";
const REFERENCE: &str = "integration/windows-native-reference-v1.json";
const WORKFLOW: &str = ".github/workflows/windows-integration.yml";
const MAX_OUTPUT: usize = 256 * 1024;
const PHYSICAL_APPROVAL: &str = "FRAGCAP_WINDOWS_PHYSICAL_EFFECTS";
const TRUST_OBLIGATION: &str = "trust-cleanup.bin";

#[derive(Clone, Copy)]
struct SummaryPolicy {
    current: bool,
    ancestor: bool,
}

const STATIC_SUMMARY: SummaryPolicy = SummaryPolicy {
    current: false,
    ancestor: false,
};
const RELEASE_SUMMARY: SummaryPolicy = SummaryPolicy {
    current: true,
    ancestor: true,
};

pub fn run(root: &Path, arguments: &[String]) -> io::Result<usize> {
    let registry_bytes = fs::read(root.join(REGISTRY))?;
    let registry: Value = serde_json::from_slice(&registry_bytes).map_err(invalid_data)?;
    let mut problems = validate_registry(root, &registry)?;
    if arguments.is_empty() {
        if root.join(REFERENCE).is_file() {
            problems.extend(validate_summary(
                root,
                &registry,
                &root.join(REFERENCE),
                STATIC_SUMMARY,
            )?);
        }
        return report(problems);
    }

    if arguments.first().map(String::as_str) == Some("--validate-report") {
        let path = value_after(arguments, "--validate-report")
            .ok_or_else(|| invalid_input("--validate-report needs a path"))?;
        problems.extend(validate_report(
            root,
            &registry,
            Path::new(path),
            RELEASE_SUMMARY,
        )?);
        return report(problems);
    }
    if arguments.first().map(String::as_str) == Some("--release") {
        if !root.join(REFERENCE).is_file() {
            problems.push(format!("missing required physical evidence {REFERENCE}"));
        } else {
            problems.extend(validate_summary(
                root,
                &registry,
                &root.join(REFERENCE),
                RELEASE_SUMMARY,
            )?);
        }
        return report(problems);
    }
    if arguments.first().map(String::as_str) != Some("--run") {
        return Err(invalid_input(
            "use --run hosted|physical, --validate-report <path>, or --release",
        ));
    }
    if !problems.is_empty() {
        return report(problems);
    }
    let tier = arguments.get(1).map(String::as_str).unwrap_or_default();
    if !matches!(tier, "hosted" | "physical") {
        return Err(invalid_input("--run requires hosted or physical"));
    }
    if tier == "physical" && std::env::var(PHYSICAL_APPROVAL).as_deref() != Ok("approved") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("physical effects require {PHYSICAL_APPROVAL}=approved"),
        ));
    }
    let binary = value_after(arguments, "--binary")
        .map(PathBuf::from)
        .ok_or_else(|| invalid_input("--run needs --binary <path>"))?;
    let raw = value_after(arguments, "--report")
        .map(PathBuf::from)
        .ok_or_else(|| invalid_input("--run needs --report <path>"))?;
    execute(root, &registry, &registry_bytes, tier, &binary, &raw)
}

fn validate_registry(root: &Path, value: &Value) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    if value["schema_version"] != 1 {
        problems.push("registry schema_version must be 1".into());
    }
    let authorities = exact_set(value, "authorities", &mut problems);
    let capabilities = exact_set(value, "capabilities", &mut problems);
    let effects = exact_set(value, "effects", &mut problems);
    let domains = exact_set(value, "completion_domains", &mut problems);
    let rows = value["rows"]
        .as_array()
        .ok_or_else(|| invalid_data("rows must be an array"))?;
    let mut ids = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for row in rows {
        let id = required(row, "id", "row", &mut problems);
        if !ids.insert(id.clone()) {
            problems.push(format!("duplicate row id {id}"));
        }
        if !valid_id(&id) {
            problems.push(format!("invalid row id {id}"));
        }
        let domain = required(row, "domain", &id, &mut problems);
        if !domains.contains(&domain) {
            problems.push(format!("{id} has unknown completion domain {domain}"));
        }
        covered.insert(domain);
        let authority = required(row, "authority", &id, &mut problems);
        if !authorities.contains(&authority) {
            problems.push(format!("{id} has unknown authority {authority}"));
        }
        if !matches!(row["tier"].as_str(), Some("hosted" | "physical")) {
            problems.push(format!("{id} has unknown tier"));
        }
        if row["expected"] != "passed"
            || row["cleanup"] != "reconciled"
            || row["publication"] != "summary-only"
        {
            problems.push(format!("{id} weakens its closed outcome contract"));
        }
        validate_members(
            row,
            "required_capabilities",
            &capabilities,
            &id,
            &mut problems,
        );
        validate_members(row, "owned_effects", &effects, &id, &mut problems);
        validate_members(row, "prohibited_effects", &effects, &id, &mut problems);
        let source_path = required(&row["evidence"], "path", &id, &mut problems);
        let function = required(&row["evidence"], "function", &id, &mut problems);
        if source_path.contains("..") || Path::new(&source_path).is_absolute() {
            problems.push(format!("{id} evidence path escapes the repository"));
        } else {
            match fs::read_to_string(root.join(&source_path)) {
                Ok(source) if source.contains(&format!("fn {function}(")) => {}
                Ok(_) => problems.push(format!("{id} missing evidence function {function}")),
                Err(_) => problems.push(format!("{id} missing evidence source {source_path}")),
            }
        }
        let command = row["command"].as_array();
        if command.is_none_or(|items| {
            items.is_empty() || items.iter().any(|item| item.as_str().is_none())
        }) {
            problems.push(format!("{id} command must be direct argv"));
        }
        if !matches!(row["timeout_seconds"].as_u64(), Some(1..=900)) {
            problems.push(format!(
                "{id} timeout must be finite and at most 900 seconds"
            ));
        }
    }
    if covered != domains {
        problems.push(format!(
            "completion domain coverage mismatch: expected {domains:?}, observed {covered:?}"
        ));
    }
    validate_authority_sources(root, value, &mut problems);
    validate_workflow(root, &mut problems);
    Ok(problems)
}

fn validate_authority_sources(root: &Path, value: &Value, problems: &mut Vec<String>) {
    let Some(sources) = value["authority_sources"].as_array() else {
        problems.push("authority_sources must be an array".into());
        return;
    };
    for source in sources {
        let path = required(source, "path", "authority source", problems);
        let text = fs::read_to_string(root.join(&path)).unwrap_or_default();
        for marker in source["markers"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !text.contains(marker) {
                problems.push(format!("authority source {path} lacks marker {marker}"));
            }
        }
    }
}

fn validate_workflow(root: &Path, problems: &mut Vec<String>) {
    let text = fs::read_to_string(root.join(WORKFLOW)).unwrap_or_default();
    for marker in [
        "windows-latest",
        "--run hosted",
        "--validate-report",
        "upload-artifact",
        "stage\\fragcap.exe",
    ] {
        if !text.contains(marker) {
            problems.push(format!("Windows workflow lacks required marker {marker}"));
        }
    }
    if text.contains("schedule:") || text.contains("integration/windows-native-reference-v1.json") {
        problems
            .push("Windows workflow may not be scheduled or overwrite physical evidence".into());
    }
}

fn execute(
    root: &Path,
    registry: &Value,
    registry_bytes: &[u8],
    tier: &str,
    binary: &Path,
    raw: &Path,
) -> io::Result<usize> {
    if !cfg!(windows) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "execution requires Windows",
        ));
    }
    let binary = fs::canonicalize(binary)?;
    if !binary.is_file() {
        return Err(invalid_input("staged binary is not a file"));
    }
    let version = child_output(&binary, &["--version"], root, Duration::from_secs(30))?;
    if !version.success {
        return Err(invalid_data("staged binary did not report its version"));
    }
    let capability_snapshot = capabilities();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let wireshark = PathBuf::from(program_files).join("Wireshark");
        if wireshark.is_dir() {
            let mut paths = vec![wireshark];
            paths.extend(std::env::split_paths(
                &std::env::var_os("PATH").unwrap_or_default(),
            ));
            std::env::set_var("PATH", std::env::join_paths(paths).map_err(invalid_data)?);
        }
    }
    std::env::set_var("FRAGCAP_WINDOWS_BINARY", &binary);
    std::env::set_var(
        "FRAGCAP_WINDOWS_EXPECT_NPCAP",
        if capability_snapshot.contains("npcap-present") {
            "present"
        } else {
            "absent"
        },
    );
    let allowed = root.join("target/windows-integration");
    fs::create_dir_all(&allowed)?;
    let allowed_resolved = fs::canonicalize(&allowed)?;
    if raw
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(invalid_input(
            "report paths may not contain parent traversal",
        ));
    }
    let raw = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        root.join(raw)
    };
    if !raw.starts_with(&allowed) {
        return Err(invalid_input(
            "reports and scratch effects must remain under target/windows-integration",
        ));
    }
    let scratch = raw.with_extension("scratch");
    std::env::set_var("FRAGCAP_WINDOWS_SCRATCH", &scratch);
    prepare_scratch(root, &scratch, &allowed_resolved)?;
    std::env::set_var("FRAGCAP_SESSION_DIR", &scratch);
    std::env::set_var("FRAGCAP_LOCAL_DB", scratch.join("local.db"));
    std::env::set_var("FRAGCAP_CATALOG_DB", scratch.join("catalog.db"));
    let digest = sha256(&fs::read(&binary)?);
    let registry_digest = sha256(registry_bytes);
    let revision = source_revision(root)?;
    let diagnostics = raw.with_extension("rows");
    if diagnostics.exists() {
        remove_owned_dir(&diagnostics, &allowed_resolved)?;
    }
    fs::create_dir_all(&diagnostics)?;
    if let Some(parent) = raw.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = File::create(&raw)?;
    write_record(
        &mut output,
        &json!({
            "record":"header", "schema_version":1, "tier":tier,
            "registry_sha256":registry_digest, "revision":revision,
            "product_version":env!("CARGO_PKG_VERSION"), "binary_sha256":digest,
        "capabilities":capability_snapshot, "started_unix_seconds":unix_seconds(),
        }),
    )?;
    let mut failed = 0_u64;
    let rows = registry["rows"].as_array().unwrap();
    for row in rows.iter().filter(|row| row["tier"] == tier) {
        let id = row["id"].as_str().unwrap();
        let effects = strings(row, "owned_effects")
            .union(&strings(row, "prohibited_effects"))
            .cloned()
            .collect::<BTreeSet<_>>();
        let before = effect_snapshot(root, &binary, &scratch, &effects)?;
        let effects_before_sha256 = inventory_digest(&before);
        let required = strings(row, "required_capabilities");
        let missing = required
            .difference(&capability_snapshot)
            .cloned()
            .collect::<Vec<_>>();
        let start = Instant::now();
        let mut result = if missing.is_empty() {
            let argv = row["command"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>();
            child_output(
                Path::new(argv[0]),
                &argv[1..],
                root,
                Duration::from_secs(row["timeout_seconds"].as_u64().unwrap()),
            )?
        } else {
            ChildResult {
                success: false,
                timed_out: false,
                output: format!("missing capabilities: {}", missing.join(",")),
            }
        };
        let capabilities_after = capabilities();
        if capabilities_after != capability_snapshot {
            result.success = false;
            result
                .output
                .push_str("\npreflight capability snapshot changed during the row");
        }
        if scratch.join(TRUST_OBLIGATION).is_file() {
            let recovery = recover_pending_trust(root)?;
            result.output.push_str("\ntrust recovery:\n");
            result.output.push_str(&recovery.output);
            result.success &= recovery.success && !scratch.join(TRUST_OBLIGATION).exists();
        }
        let (cleanup, effects_after_sha256) =
            match effect_snapshot(root, &binary, &scratch, &effects) {
                Ok(after) if after == before => ("reconciled", inventory_digest(&after)),
                Ok(_) => {
                    result.success = false;
                    result
                        .output
                        .push_str("\neffect inventory changed across the row");
                    ("failed", "changed".into())
                }
                Err(error) => {
                    result.success = false;
                    result
                        .output
                        .push_str(&format!("\neffect inventory failed after the row: {error}"));
                    ("failed", "unavailable".into())
                }
            };
        let status = if result.success { "passed" } else { "failed" };
        if !result.success {
            failed += 1;
        }
        fs::write(
            diagnostics.join(format!("{id}.log")),
            result.output.as_bytes(),
        )?;
        write_record(
            &mut output,
            &json!({
                "record":"row", "id":id, "status":status,
                "timed_out":result.timed_out, "duration_ms":start.elapsed().as_millis() as u64,
                "cleanup":cleanup, "output_sha256":sha256(result.output.as_bytes()),
                "output_bytes":result.output.len().min(MAX_OUTPUT),
                "effects_before_sha256":effects_before_sha256,
                "effects_after_sha256":effects_after_sha256,
            }),
        )?;
    }
    let residue = if failed == 0 {
        match remove_owned_dir(&scratch, &allowed_resolved) {
            Ok(()) if !scratch.exists() => "none",
            _ => {
                failed += 1;
                "present"
            }
        }
    } else {
        "present"
    };
    let required = rows.iter().filter(|row| row["tier"] == tier).count() as u64;
    write_record(
        &mut output,
        &json!({
            "record":"terminal", "status":if failed == 0 {"complete"} else {"failed"},
            "required":required, "passed":required.saturating_sub(failed), "failed":failed,
            "residue":residue, "finished_unix_seconds":unix_seconds(),
        }),
    )?;
    drop(output);
    let summary = derive_summary(&raw)?;
    let summary_path = raw.with_extension("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).map_err(invalid_data)?,
    )?;
    let problems = validate_summary(root, registry, &summary_path, RELEASE_SUMMARY)?;
    report(problems)
}

fn prepare_scratch(root: &Path, scratch: &Path, allowed_resolved: &Path) -> io::Result<()> {
    prepare_scratch_with(
        scratch,
        allowed_resolved,
        std::env::var(PHYSICAL_APPROVAL).as_deref() == Ok("approved"),
        || recover_pending_trust(root),
    )
}

fn prepare_scratch_with(
    scratch: &Path,
    allowed_resolved: &Path,
    physical_approved: bool,
    recover: impl FnOnce() -> io::Result<ChildResult>,
) -> io::Result<()> {
    if scratch.exists() {
        if scratch.join(TRUST_OBLIGATION).is_file() {
            if !physical_approved {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "retained trust cleanup requires physical-effects approval",
                ));
            }
            let recovery = recover()?;
            if !recovery.success || scratch.join(TRUST_OBLIGATION).exists() {
                return Err(invalid_data(format!(
                    "retained trust cleanup failed; scratch was preserved ({})",
                    sha256(recovery.output.as_bytes())
                )));
            }
        }
        remove_owned_dir(scratch, allowed_resolved)?;
    }
    fs::create_dir_all(scratch)?;
    Ok(())
}

fn recover_pending_trust(root: &Path) -> io::Result<ChildResult> {
    child_output(
        Path::new("cargo"),
        &[
            "test",
            "-p",
            "fragcap-proxy",
            "--test",
            "certificates",
            "reconcile_pending_current_user_trust",
            "--",
            "--exact",
            "--ignored",
        ],
        root,
        Duration::from_secs(180),
    )
}

fn effect_snapshot(
    root: &Path,
    binary: &Path,
    scratch: &Path,
    effects: &BTreeSet<String>,
) -> io::Result<BTreeSet<String>> {
    let mut snapshot = BTreeSet::new();
    if effects.contains("child-process") {
        snapshot.insert("child-process=job-contained".into());
    }
    if effects.contains("routing-environment") {
        for name in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "no_proxy",
        ] {
            snapshot.insert(format!("env:{name}={:?}", std::env::var_os(name)));
        }
    }
    if effects.iter().any(|effect| {
        matches!(
            effect.as_str(),
            "current-user-trust"
                | "listener"
                | "sensitive-file"
                | "session-journal"
                | "session-lease"
                | "temporary-key"
        )
    }) {
        let doctor = child_output(binary, &["--json", "doctor"], root, Duration::from_secs(60))?;
        for line in doctor.output.lines() {
            let Ok(record) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            let name = record["name"].as_str().unwrap_or_default();
            let file_effect = effects.iter().any(|effect| {
                matches!(
                    effect.as_str(),
                    "sensitive-file" | "session-journal" | "session-lease" | "temporary-key"
                )
            });
            let relevant = (effects.contains("current-user-trust") && name == "local CA trust")
                || (effects.contains("listener")
                    && matches!(
                        name,
                        "IPv4 loopback listener"
                            | "IPv6 loopback listener"
                            | "native resource bundle/session-owner"
                    ))
                || (file_effect && name == "native resource bundle/session-owner");
            if relevant {
                snapshot.insert(format!(
                    "doctor:{name}:{}:{}",
                    record["status"].as_str().unwrap_or_default(),
                    record["detail"].as_str().unwrap_or_default()
                ));
            }
        }
        inventory_paths(scratch, scratch, &mut snapshot)?;
    }
    if effects.contains("system-proxy") {
        for name in ["ProxyEnable", "ProxyServer", "AutoConfigURL"] {
            registry_observation(
                root,
                &[
                    "query",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                    "/v",
                    name,
                ],
                &format!("system-proxy:{name}"),
                &mut snapshot,
            )?;
        }
    }
    if effects.contains("firewall-rule") {
        for (name, key) in [
            (
                "machine",
                r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\FirewallRules",
            ),
            (
                "user",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\FirewallPolicy\FirewallRules",
            ),
        ] {
            registry_observation(
                root,
                &["query", key],
                &format!("firewall:{name}"),
                &mut snapshot,
            )?;
        }
    }
    Ok(snapshot)
}

fn registry_observation(
    root: &Path,
    arguments: &[&str],
    name: &str,
    snapshot: &mut BTreeSet<String>,
) -> io::Result<()> {
    let result = child_output(
        Path::new("reg.exe"),
        arguments,
        root,
        Duration::from_secs(30),
    )?;
    snapshot.insert(format!(
        "{name}:{}:{}",
        result.success,
        sha256(result.output.as_bytes())
    ));
    Ok(())
}

fn inventory_digest(snapshot: &BTreeSet<String>) -> String {
    let bytes = snapshot
        .iter()
        .flat_map(|entry| entry.bytes().chain(std::iter::once(b'\n')))
        .collect::<Vec<_>>();
    sha256(&bytes)
}

fn inventory_paths(
    root: &Path,
    directory: &Path,
    snapshot: &mut BTreeSet<String>,
) -> io::Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(invalid_data)?;
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            snapshot.insert(format!("path:link:{}", relative.display()));
        } else if metadata.is_dir() {
            snapshot.insert(format!("path:dir:{}", relative.display()));
            inventory_paths(root, &path, snapshot)?;
        } else {
            let bytes = fs::read(&path)?;
            snapshot.insert(format!(
                "path:file:{}:bytes={}:sha256={}",
                relative.display(),
                bytes.len(),
                sha256(&bytes)
            ));
        }
    }
    Ok(())
}

fn derive_summary(raw: &Path) -> io::Result<Value> {
    let records = fs::read_to_string(raw)?
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).map_err(invalid_data))
        .collect::<io::Result<Vec<_>>>()?;
    let header = records
        .first()
        .ok_or_else(|| invalid_data("report has no header"))?;
    let terminal = records
        .last()
        .ok_or_else(|| invalid_data("report has no terminal"))?;
    validate_raw_record(header, "header", HEADER_KEYS)?;
    validate_raw_record(terminal, "terminal", RAW_TERMINAL_KEYS)?;
    if records.len() < 2 {
        return Err(invalid_data("report has no terminal"));
    }
    let mut row_ids = BTreeSet::new();
    for record in &records[1..records.len() - 1] {
        validate_raw_record(record, "row", RAW_ROW_KEYS)?;
        let id = record["id"]
            .as_str()
            .ok_or_else(|| invalid_data("raw row lacks a string id"))?;
        if !row_ids.insert(id) {
            return Err(invalid_data(format!("raw report repeats row {id}")));
        }
    }
    let rows = records[1..records.len() - 1].iter().map(|row| json!({
        "id":row["id"], "status":row["status"], "timed_out":row["timed_out"],
        "cleanup":row["cleanup"], "output_sha256":row["output_sha256"], "output_bytes":row["output_bytes"]
    })).collect::<Vec<_>>();
    Ok(json!({
        "schema_version":1, "tier":header["tier"], "registry_sha256":header["registry_sha256"],
        "revision":header["revision"], "product_version":header["product_version"],
        "binary_sha256":header["binary_sha256"], "capabilities":header["capabilities"],
        "recorded_on":civil_date(), "rows":rows,
        "terminal":{"status":terminal["status"],"required":terminal["required"],"passed":terminal["passed"],"failed":terminal["failed"],"residue":terminal["residue"]}
    }))
}

const HEADER_KEYS: &[&str] = &[
    "binary_sha256",
    "capabilities",
    "product_version",
    "record",
    "registry_sha256",
    "revision",
    "schema_version",
    "started_unix_seconds",
    "tier",
];
const RAW_ROW_KEYS: &[&str] = &[
    "cleanup",
    "duration_ms",
    "effects_after_sha256",
    "effects_before_sha256",
    "id",
    "output_bytes",
    "output_sha256",
    "record",
    "status",
    "timed_out",
];
const RAW_TERMINAL_KEYS: &[&str] = &[
    "failed",
    "finished_unix_seconds",
    "passed",
    "record",
    "required",
    "residue",
    "status",
];

fn validate_raw_record(value: &Value, kind: &str, allowed: &[&str]) -> io::Result<()> {
    if value["record"] != kind {
        return Err(invalid_data(format!("raw report expected {kind} record")));
    }
    if !has_exact_keys(value, allowed) {
        return Err(invalid_data(format!(
            "raw {kind} record has an invalid field set"
        )));
    }
    let valid_types = match kind {
        "header" => {
            value["schema_version"] == 1 && value["started_unix_seconds"].as_u64().is_some()
        }
        "row" => {
            value["duration_ms"].as_u64().is_some()
                && is_lower_hex(
                    value["effects_before_sha256"].as_str().unwrap_or_default(),
                    64,
                )
                && is_effect_digest(value["effects_after_sha256"].as_str().unwrap_or_default())
        }
        "terminal" => value["finished_unix_seconds"].as_u64().is_some(),
        _ => false,
    };
    if !valid_types {
        return Err(invalid_data(format!(
            "raw {kind} record has invalid value types"
        )));
    }
    Ok(())
}

fn is_effect_digest(value: &str) -> bool {
    is_lower_hex(value, 64) || matches!(value, "changed" | "unavailable")
}

fn validate_summary(
    root: &Path,
    registry: &Value,
    path: &Path,
    policy: SummaryPolicy,
) -> io::Result<Vec<String>> {
    let bytes = fs::read(path)?;
    let value: Value = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    validate_summary_value(root, registry, &bytes, &value, policy)
}

fn validate_report(
    root: &Path,
    registry: &Value,
    path: &Path,
    policy: SummaryPolicy,
) -> io::Result<Vec<String>> {
    let bytes = fs::read(path)?;
    if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
        return validate_summary_value(root, registry, &bytes, &value, policy);
    }
    let summary = derive_summary(path)?;
    let summary_bytes = serde_json::to_vec(&summary).map_err(invalid_data)?;
    validate_summary_value(root, registry, &summary_bytes, &summary, policy)
}

fn validate_summary_value(
    root: &Path,
    registry: &Value,
    bytes: &[u8],
    value: &Value,
    policy: SummaryPolicy,
) -> io::Result<Vec<String>> {
    let mut problems = Vec::new();
    const SUMMARY_KEYS: &[&str] = &[
        "binary_sha256",
        "capabilities",
        "product_version",
        "recorded_on",
        "registry_sha256",
        "revision",
        "rows",
        "schema_version",
        "terminal",
        "tier",
    ];
    const SUMMARY_ROW_KEYS: &[&str] = &[
        "cleanup",
        "id",
        "output_bytes",
        "output_sha256",
        "status",
        "timed_out",
    ];
    const SUMMARY_TERMINAL_KEYS: &[&str] = &["failed", "passed", "required", "residue", "status"];
    if bytes.len() > 128 * 1024 {
        problems.push("summary exceeds 128 KiB".into());
    }
    if !has_exact_keys(value, SUMMARY_KEYS) {
        problems.push("summary contains unknown or missing top-level fields".into());
    }
    if !has_exact_keys(&value["terminal"], SUMMARY_TERMINAL_KEYS) {
        problems.push("summary terminal contains unknown or missing fields".into());
    }
    if value["schema_version"] != 1 {
        problems.push("summary schema_version must be 1".into());
    }
    let tier = value["tier"].as_str().unwrap_or_default();
    if !matches!(tier, "hosted" | "physical") {
        problems.push("summary tier is invalid".into());
    }
    let registry_digest = sha256(&fs::read(root.join(REGISTRY))?);
    if value["registry_sha256"] != registry_digest {
        problems.push("summary registry digest mismatch".into());
    }
    if policy.current && value["product_version"] != env!("CARGO_PKG_VERSION") {
        problems.push("summary product version mismatch".into());
    }
    for field in ["revision", "binary_sha256", "recorded_on"] {
        if value[field].as_str().is_none_or(str::is_empty) {
            problems.push(format!("summary lacks {field}"));
        }
    }
    for field in ["registry_sha256", "binary_sha256"] {
        if !is_lower_hex(value[field].as_str().unwrap_or_default(), 64) {
            problems.push(format!("summary {field} is not a SHA-256 digest"));
        }
    }
    if !is_lower_hex(value["revision"].as_str().unwrap_or_default(), 40) {
        problems.push("summary revision is not a full lowercase Git object id".into());
    }
    if parse_civil(value["recorded_on"].as_str().unwrap_or_default()).is_none() {
        problems.push("summary recorded_on is not a civil date".into());
    }
    let allowed_capabilities = strings(registry, "capabilities");
    let summary_capabilities = value["capabilities"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if value["capabilities"].as_array().is_none_or(|items| {
        items.len() != summary_capabilities.len()
            || items.iter().any(|item| item.as_str().is_none())
    }) || !summary_capabilities.is_subset(&allowed_capabilities)
    {
        problems.push("summary capabilities are invalid".into());
    }
    if policy.ancestor
        && !revision_is_ancestor(root, value["revision"].as_str().unwrap_or_default())
    {
        problems.push("summary revision is not a commit ancestor of the candidate".into());
    }
    let expected = registry["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["tier"] == tier)
        .map(|row| row["id"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    let rows = value["rows"].as_array().cloned().unwrap_or_default();
    let observed = rows
        .iter()
        .filter_map(|row| row["id"].as_str())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if rows.len() != observed.len() || observed != expected {
        problems.push("summary row identity or completeness mismatch".into());
    }
    for row in &rows {
        if row["status"] != "passed" || row["cleanup"] != "reconciled" || row["timed_out"] != false
        {
            problems.push(format!("row {} is not satisfied", row["id"]));
        }
        if !has_exact_keys(row, SUMMARY_ROW_KEYS) {
            problems.push(format!("row {} contains non-public fields", row["id"]));
        }
        if !is_lower_hex(row["output_sha256"].as_str().unwrap_or_default(), 64)
            || row["output_bytes"].as_u64().is_none()
        {
            problems.push(format!("row {} has invalid evidence metadata", row["id"]));
        }
    }
    if value["terminal"]["status"] != "complete"
        || value["terminal"]["failed"] != 0
        || value["terminal"]["residue"] != "none"
    {
        problems.push("summary terminal is not complete and residue-free".into());
    }
    let required = value["terminal"]["required"].as_u64();
    let passed = value["terminal"]["passed"].as_u64();
    let failed = value["terminal"]["failed"].as_u64();
    if required != Some(expected.len() as u64) || passed != required || failed != Some(0) {
        problems.push("summary terminal counts do not reconcile".into());
    }
    let forbidden = [
        "stdout",
        "stderr",
        "command",
        "path",
        "certificate",
        "payload",
        "username",
        "hostname",
        "secret",
    ];
    let lowered = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    for token in forbidden {
        if lowered.contains(&format!("\"{token}\"")) {
            problems.push(format!("summary exposes forbidden field {token}"));
        }
    }
    if policy.current
        && tier == "physical"
        && !date_is_current(
            value["recorded_on"].as_str().unwrap_or_default(),
            registry["physical_evidence_max_age_days"]
                .as_u64()
                .unwrap_or(0),
        )
    {
        problems.push("physical evidence is expired or has an invalid date".into());
    }
    Ok(problems)
}

fn revision_is_ancestor(root: &Path, revision: &str) -> bool {
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return false;
    }
    child_output(
        Path::new("git"),
        &["cat-file", "-e", &format!("{revision}^{{commit}}")],
        root,
        Duration::from_secs(10),
    )
    .is_ok_and(|result| result.success)
        && child_output(
            Path::new("git"),
            &["merge-base", "--is-ancestor", revision, "HEAD"],
            root,
            Duration::from_secs(10),
        )
        .is_ok_and(|result| result.success)
}

struct ChildResult {
    success: bool,
    timed_out: bool,
    output: String,
}

fn child_output(
    program: &Path,
    args: &[&str],
    root: &Path,
    timeout: Duration,
) -> io::Result<ChildResult> {
    let mut command = Command::new(program);
    command
        .current_dir(root)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.env("FRAGCAP_WINDOWS_MATRIX", "1");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::{CREATE_NO_WINDOW, CREATE_SUSPENDED};
        command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
    }
    #[cfg(windows)]
    let job = WindowsJob::new()?;
    let mut child = command.spawn()?;
    #[cfg(windows)]
    if let Err(error) = job.assign(&child).and_then(|()| resume_child(child.id())) {
        let _ = job.terminate();
        let _ = child.wait();
        return Err(error);
    }
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out_thread = thread::spawn(move || read_bounded(stdout));
    let err_thread = thread::spawn(move || read_bounded(stderr));
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            #[cfg(windows)]
            job.terminate()?;
            #[cfg(not(windows))]
            child.kill()?;
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(50));
    };
    let mut output = out_thread.join().unwrap_or_default();
    output.push_str(&err_thread.join().unwrap_or_default());
    let mut end = output.len().min(MAX_OUTPUT);
    while !output.is_char_boundary(end) {
        end -= 1;
    }
    output.truncate(end);
    Ok(ChildResult {
        success: status.success() && !timed_out,
        timed_out,
        output,
    })
}

#[cfg(windows)]
fn resume_child(process_id: u32) -> io::Result<()> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    let mut present = unsafe { Thread32First(snapshot, std::ptr::addr_of_mut!(entry)) } != 0;
    while present {
        if entry.th32OwnerProcessID == process_id {
            unsafe { CloseHandle(snapshot) };
            // The child is still suspended. Request only the right needed to resume its
            // primary thread after Job Object assignment; no memory right is requested.
            let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if thread == 0 {
                return Err(io::Error::last_os_error());
            }
            let resumed = unsafe { ResumeThread(thread) };
            unsafe { CloseHandle(thread) };
            if resumed == u32::MAX {
                return Err(io::Error::last_os_error());
            }
            return Ok(());
        }
        present = unsafe { Thread32Next(snapshot, std::ptr::addr_of_mut!(entry)) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "suspended child primary thread was not found",
    ))
}

#[cfg(windows)]
struct WindowsJob(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl WindowsJob {
    fn new() -> io::Result<Self> {
        use windows_sys::Win32::System::JobObjects::{
            CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle == 0 {
            return Err(io::Error::last_os_error());
        }
        let job = Self(handle);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(limits).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(job)
    }

    fn assign(&self, child: &std::process::Child) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

        if unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as isize) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn terminate(&self) -> io::Result<()> {
        use windows_sys::Win32::System::JobObjects::TerminateJobObject;

        if unsafe { TerminateJobObject(self.0, 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for WindowsJob {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

fn read_bounded(stream: impl Read) -> String {
    let mut bytes = Vec::new();
    let _ = stream.take((MAX_OUTPUT + 1) as u64).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

fn remove_owned_dir(path: &Path, allowed: &Path) -> io::Result<()> {
    let resolved = fs::canonicalize(path)?;
    if !resolved.starts_with(allowed) || resolved == allowed {
        return Err(invalid_input(format!(
            "refusing recursive removal outside the matrix workspace: {}",
            resolved.display()
        )));
    }
    fs::remove_dir_all(resolved)
}

fn capabilities() -> BTreeSet<String> {
    let mut values = BTreeSet::from(["windows".into()]);
    if std::net::TcpListener::bind("127.0.0.1:0").is_ok() {
        values.insert("ipv4-loopback".into());
    }
    if std::net::TcpListener::bind("[::1]:0").is_ok() {
        values.insert("ipv6-loopback".into());
    }
    let system = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let npcap = system.join("System32/Npcap/wpcap.dll").is_file();
    values.insert(if npcap {
        "npcap-present".into()
    } else {
        "npcap-absent".into()
    });
    let tshark = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .map(|p| p.join("Wireshark/tshark.exe"))
        .is_some_and(|p| p.is_file());
    if tshark {
        values.insert("tshark-present".into());
    }
    if is_elevated() {
        values.insert("elevated".into());
    } else {
        values.insert("non-elevated".into());
    }
    values
}

fn is_elevated() -> bool {
    let Ok(result) = child_output(
        Path::new("whoami.exe"),
        &["/groups", "/fo", "csv", "/nh"],
        Path::new("."),
        Duration::from_secs(10),
    ) else {
        return false;
    };
    result.success
        && result
            .output
            .lines()
            .any(|line| line.contains("S-1-16-12288") || line.contains("S-1-16-16384"))
}

fn exact_set(value: &Value, field: &str, problems: &mut Vec<String>) -> BTreeSet<String> {
    let values = value[field].as_array().cloned().unwrap_or_default();
    let set = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if set.len() != values.len() {
        problems.push(format!("{field} contains duplicates or non-strings"));
    }
    set
}

fn validate_members(
    value: &Value,
    field: &str,
    allowed: &BTreeSet<String>,
    owner: &str,
    problems: &mut Vec<String>,
) {
    for item in strings(value, field) {
        if !allowed.contains(&item) {
            problems.push(format!("{owner} has unknown {field} value {item}"));
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

fn required(value: &Value, field: &str, owner: &str, problems: &mut Vec<String>) -> String {
    value[field]
        .as_str()
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            problems.push(format!("{owner} lacks {field}"));
            String::new()
        })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

fn has_exact_keys(value: &Value, expected: &[&str]) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn write_record(file: &mut File, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *file, value).map_err(invalid_data)?;
    file.write_all(b"\n")?;
    file.flush()
}

fn sha256(bytes: &[u8]) -> String {
    digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn civil_date() -> String {
    if let Ok(value) = std::env::var("FRAGCAP_WINDOWS_EVIDENCE_DATE") {
        return value;
    }
    let days = (unix_seconds() / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}
fn date_is_current(date: &str, max_age: u64) -> bool {
    let Some(recorded) = parse_civil(date) else {
        return false;
    };
    let today = (unix_seconds() / 86_400) as i64;
    recorded <= today && today - recorded <= max_age as i64
}
fn parse_civil(value: &str) -> Option<i64> {
    if value.len() != 10
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || value
            .bytes()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return None;
    }
    let mut fields = value.split('-');
    let year = fields.next()?.parse::<i64>().ok()?;
    let month = fields.next()?.parse::<u32>().ok()?;
    let day = fields.next()?.parse::<u32>().ok()?;
    if fields.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let days = days_from_civil(year, month, day);
    (civil_from_days(days) == (year, month, day)).then_some(days)
}
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let doe = shifted - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}
fn source_revision(root: &Path) -> io::Result<String> {
    let status = child_output(
        Path::new("git"),
        &["status", "--porcelain", "--untracked-files=no"],
        root,
        Duration::from_secs(10),
    )?;
    if !status.success || !status.output.trim().is_empty() {
        return Err(invalid_data(
            "Windows evidence requires a clean tracked source revision",
        ));
    }
    let revision = child_output(
        Path::new("git"),
        &["rev-parse", "HEAD"],
        root,
        Duration::from_secs(10),
    )?;
    if !revision.success {
        return Err(invalid_data("could not resolve the source revision"));
    }
    Ok(revision.output.trim().to_owned())
}
fn value_after<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}
fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
fn invalid_input(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error.to_string())
}
fn report(problems: Vec<String>) -> io::Result<usize> {
    for problem in &problems {
        eprintln!("windows-integration: {problem}");
    }
    if problems.is_empty() {
        println!("windows-integration: closed registry and retained evidence are valid");
    }
    Ok(problems.len())
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn repository_windows_matrix_is_closed() {
        assert!(validate_registry(&root(), &registry()).unwrap().is_empty());
    }
    #[test]
    fn duplicate_row_and_unknown_capability_are_rejected() {
        let mut value = registry();
        let duplicate = value["rows"][0].clone();
        value["rows"].as_array_mut().unwrap().push(duplicate);
        value["rows"][0]["required_capabilities"] = json!(["invented"]);
        let problems = validate_registry(&root(), &value).unwrap();
        assert!(problems.iter().any(|p| p.contains("duplicate row")));
        assert!(problems
            .iter()
            .any(|p| p.contains("unknown required_capabilities")));
    }
    #[test]
    fn incomplete_and_free_form_summaries_are_rejected() {
        let directory =
            std::env::temp_dir().join(format!("fragcap-s129-summary-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("summary.json");
        fs::write(
            &path,
            br#"{"schema_version":1,"tier":"physical","stdout":"secret"}"#,
        )
        .unwrap();
        let problems = validate_summary(&root(), &registry(), &path, STATIC_SUMMARY).unwrap();
        assert!(problems.iter().any(|p| p.contains("row identity")));
        assert!(problems
            .iter()
            .any(|p| p.contains("forbidden field stdout")));
        let mut reference: Value =
            serde_json::from_slice(&fs::read(root().join(REFERENCE)).unwrap()).unwrap();
        reference["note"] = json!(r"C:\Users\operator\private");
        let bytes = serde_json::to_vec(&reference).unwrap();
        let problems =
            validate_summary_value(&root(), &registry(), &bytes, &reference, STATIC_SUMMARY)
                .unwrap();
        assert!(problems
            .iter()
            .any(|problem| problem.contains("unknown or missing top-level fields")));
        let _ = fs::remove_dir_all(directory);
    }
    #[test]
    fn raw_jsonl_and_derived_summary_validate_identically() {
        let directory =
            std::env::temp_dir().join(format!("fragcap-s129-jsonl-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let reference: Value =
            serde_json::from_slice(&fs::read(root().join(REFERENCE)).unwrap()).unwrap();
        let raw_path = directory.join("physical.jsonl");
        let mut raw = File::create(&raw_path).unwrap();
        write_record(
            &mut raw,
            &json!({
                "record":"header", "schema_version":1, "tier":reference["tier"],
                "registry_sha256":reference["registry_sha256"], "revision":reference["revision"],
                "product_version":reference["product_version"],
                "binary_sha256":reference["binary_sha256"],
                "capabilities":reference["capabilities"], "started_unix_seconds":1
            }),
        )
        .unwrap();
        for row in reference["rows"].as_array().unwrap() {
            let mut record = row.clone();
            record["record"] = json!("row");
            record["duration_ms"] = json!(1);
            record["effects_before_sha256"] = row["output_sha256"].clone();
            record["effects_after_sha256"] = row["output_sha256"].clone();
            write_record(&mut raw, &record).unwrap();
        }
        let mut terminal = reference["terminal"].clone();
        terminal["record"] = json!("terminal");
        terminal["finished_unix_seconds"] = json!(2);
        write_record(&mut raw, &terminal).unwrap();
        drop(raw);
        assert!(
            validate_report(&root(), &registry(), &raw_path, STATIC_SUMMARY)
                .unwrap()
                .is_empty()
        );
        assert!(validate_summary(
            &root(),
            &registry(),
            &root().join(REFERENCE),
            STATIC_SUMMARY,
        )
        .unwrap()
        .is_empty());
        let mut append = fs::OpenOptions::new().append(true).open(&raw_path).unwrap();
        write_record(&mut append, &terminal).unwrap();
        assert!(derive_summary(&raw_path).is_err());
        let _ = fs::remove_dir_all(directory);
    }
    #[test]
    fn currency_and_revision_binding_are_release_only() {
        let bytes = fs::read(root().join(REFERENCE)).unwrap();
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        value["recorded_on"] = json!("2000-01-01");
        value["revision"] = json!("0000000000000000000000000000000000000000");
        value["product_version"] = json!("0.0.0");
        let mutated = serde_json::to_vec(&value).unwrap();
        let static_problems =
            validate_summary_value(&root(), &registry(), &mutated, &value, STATIC_SUMMARY).unwrap();
        assert!(!static_problems.iter().any(|problem| {
            problem.contains("expired")
                || problem.contains("commit ancestor")
                || problem.contains("product version")
        }));
        let release_problems =
            validate_summary_value(&root(), &registry(), &mutated, &value, RELEASE_SUMMARY)
                .unwrap();
        assert!(release_problems
            .iter()
            .any(|problem| problem.contains("expired")));
        assert!(release_problems
            .iter()
            .any(|problem| problem.contains("commit ancestor")));
        assert!(release_problems
            .iter()
            .any(|problem| problem.contains("product version")));
    }
    #[test]
    fn path_inventory_detects_effect_residue() {
        let directory =
            std::env::temp_dir().join(format!("fragcap-s129-effects-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let mut before = BTreeSet::new();
        inventory_paths(&directory, &directory, &mut before).unwrap();
        fs::write(directory.join("residue.bin"), b"residue").unwrap();
        let mut after = BTreeSet::new();
        inventory_paths(&directory, &directory, &mut after).unwrap();
        assert_ne!(after, before);
        let original = after;
        fs::write(directory.join("residue.bin"), b"changed").unwrap();
        let mut changed = BTreeSet::new();
        inventory_paths(&directory, &directory, &mut changed).unwrap();
        assert_ne!(changed, original);
        let _ = fs::remove_dir_all(directory);
    }
    #[test]
    fn retained_trust_is_recovered_or_preserved_before_scratch_replacement() {
        let allowed = root().join("target/windows-integration");
        fs::create_dir_all(&allowed).unwrap();
        let allowed_resolved = fs::canonicalize(&allowed).unwrap();
        let scratch = allowed.join(format!("recovery-order-{}", std::process::id()));
        fs::create_dir_all(&scratch).unwrap();
        fs::write(scratch.join(TRUST_OBLIGATION), b"retained authority").unwrap();
        let denied = prepare_scratch_with(&scratch, &allowed_resolved, false, || {
            panic!("recovery must not run without approval")
        });
        assert_eq!(denied.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        assert!(scratch.join(TRUST_OBLIGATION).is_file());
        let failed = prepare_scratch_with(&scratch, &allowed_resolved, true, || {
            Ok(ChildResult {
                success: false,
                timed_out: false,
                output: "controlled recovery failure".into(),
            })
        });
        assert_eq!(failed.unwrap_err().kind(), io::ErrorKind::InvalidData);
        assert!(scratch.join(TRUST_OBLIGATION).is_file());
        remove_owned_dir(&scratch, &allowed_resolved).unwrap();
    }
    #[test]
    fn bounded_reader_and_timeout_are_finite() {
        let input = vec![b'x'; MAX_OUTPUT + 10];
        assert_eq!(read_bounded(input.as_slice()).len(), MAX_OUTPUT + 1);
        let result = child_output(
            Path::new("rustc"),
            &["--version"],
            &root(),
            Duration::from_secs(30),
        )
        .unwrap();
        assert!(result.success);
        assert!(result.output.contains("rustc"));
    }

    #[cfg(windows)]
    #[test]
    fn timeout_terminates_descendants_before_reader_join() {
        let directory =
            std::env::temp_dir().join(format!("fragcap-s129-tree-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let marker = directory.join("survived.txt");
        let script = format!(
            "ping.exe -n 20 127.0.0.1 && echo survived > \\\"{}\\\"",
            marker.display()
        );
        let started = Instant::now();
        let result = child_output(
            Path::new("cmd.exe"),
            &["/d", "/s", "/c", &script],
            &root(),
            Duration::from_millis(250),
        )
        .unwrap();
        assert!(result.timed_out);
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(!marker.exists());
        let _ = fs::remove_dir_all(directory);
    }
}
