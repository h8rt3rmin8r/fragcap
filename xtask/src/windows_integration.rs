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

pub fn run(root: &Path, arguments: &[String]) -> io::Result<usize> {
    let registry_bytes = fs::read(root.join(REGISTRY))?;
    let registry: Value = serde_json::from_slice(&registry_bytes).map_err(invalid_data)?;
    let mut problems = validate_registry(root, &registry)?;
    if arguments.is_empty() {
        if root.join(REFERENCE).is_file() {
            problems.extend(validate_summary(root, &registry, &root.join(REFERENCE))?);
        }
        return report(problems);
    }

    if arguments.first().map(String::as_str) == Some("--validate-report") {
        let path = value_after(arguments, "--validate-report")
            .ok_or_else(|| invalid_input("--validate-report needs a path"))?;
        problems.extend(validate_summary(root, &registry, Path::new(path))?);
        return report(problems);
    }
    if arguments.first().map(String::as_str) == Some("--release") {
        if !root.join(REFERENCE).is_file() {
            problems.push(format!("missing required physical evidence {REFERENCE}"));
        } else {
            problems.extend(validate_summary(root, &registry, &root.join(REFERENCE))?);
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
    if scratch.exists() {
        remove_owned_dir(&scratch, &allowed_resolved)?;
    }
    fs::create_dir_all(&scratch)?;
    std::env::set_var("FRAGCAP_WINDOWS_SCRATCH", &scratch);
    std::env::set_var("FRAGCAP_SESSION_DIR", &scratch);
    let digest = sha256(&fs::read(&binary)?);
    let registry_digest = sha256(registry_bytes);
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
            "registry_sha256":registry_digest, "revision":source_revision(root),
            "product_version":env!("CARGO_PKG_VERSION"), "binary_sha256":digest,
        "capabilities":capability_snapshot, "started_unix_seconds":unix_seconds(),
        }),
    )?;
    let mut failed = 0_u64;
    let rows = registry["rows"].as_array().unwrap();
    for row in rows.iter().filter(|row| row["tier"] == tier) {
        let id = row["id"].as_str().unwrap();
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
                "cleanup":"reconciled", "output_sha256":sha256(result.output.as_bytes()),
                "output_bytes":result.output.len().min(MAX_OUTPUT),
            }),
        )?;
    }
    let residue = match remove_owned_dir(&scratch, &allowed_resolved) {
        Ok(()) if !scratch.exists() => "none",
        _ => {
            failed += 1;
            "present"
        }
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
    let problems = validate_summary(root, registry, &summary_path)?;
    report(problems)
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
    let rows = records.iter().filter(|record| record["record"] == "row").map(|row| json!({
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

fn validate_summary(root: &Path, registry: &Value, path: &Path) -> io::Result<Vec<String>> {
    let bytes = fs::read(path)?;
    let value: Value = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    let mut problems = Vec::new();
    if bytes.len() > 128 * 1024 {
        problems.push("summary exceeds 128 KiB".into());
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
    if value["product_version"] != env!("CARGO_PKG_VERSION") {
        problems.push("summary product version mismatch".into());
    }
    for field in ["revision", "binary_sha256", "recorded_on"] {
        if value[field].as_str().is_none_or(str::is_empty) {
            problems.push(format!("summary lacks {field}"));
        }
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
        let keys = row
            .as_object()
            .map(|map| map.keys().map(String::as_str).collect::<BTreeSet<_>>())
            .unwrap_or_default();
        let allowed = BTreeSet::from([
            "cleanup",
            "id",
            "output_bytes",
            "output_sha256",
            "status",
            "timed_out",
        ]);
        if keys != allowed {
            problems.push(format!("row {} contains non-public fields", row["id"]));
        }
    }
    if value["terminal"]["status"] != "complete"
        || value["terminal"]["failed"] != 0
        || value["terminal"]["residue"] != "none"
    {
        problems.push("summary terminal is not complete and residue-free".into());
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
    let lowered = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
    for token in forbidden {
        if lowered.contains(&format!("\"{token}\"")) {
            problems.push(format!("summary exposes forbidden field {token}"));
        }
    }
    if tier == "physical"
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
        command.creation_flags(0x0800_0000);
    }
    let mut child = command.spawn()?;
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
    let mut fields = value.split('-');
    let year = fields.next()?.parse::<i64>().ok()?;
    let month = fields.next()?.parse::<u32>().ok()?;
    let day = fields.next()?.parse::<u32>().ok()?;
    if fields.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
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
fn source_revision(root: &Path) -> String {
    child_output(
        Path::new("git"),
        &["rev-parse", "HEAD"],
        root,
        Duration::from_secs(10),
    )
    .ok()
    .filter(|r| r.success)
    .map(|r| r.output.trim().to_owned())
    .unwrap_or_else(|| "unknown".into())
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
        let problems = validate_summary(&root(), &registry(), &path).unwrap();
        assert!(problems.iter().any(|p| p.contains("row identity")));
        assert!(problems
            .iter()
            .any(|p| p.contains("forbidden field stdout")));
        let _ = fs::remove_dir_all(directory);
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
}
