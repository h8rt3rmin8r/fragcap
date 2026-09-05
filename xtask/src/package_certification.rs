// SPDX-License-Identifier: Apache-2.0

//! Closed certification authority for final Windows release packages.

use ring::digest::{digest, SHA256};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

const CONTRACT: &str = "integration/windows-package-contract-v1.json";
const PACKAGE_WORKFLOW: &str = ".github/workflows/package-certification.yml";
const RELEASE_WORKFLOW: &str = ".github/workflows/release.yml";
const WIX_SOURCE: &str = "crates/fragcap-cli/wix/main.wxs";
const HARNESS: &str = "scripts/Test-PackageCertification.ps1";
const MAX_CONTRACT_BYTES: u64 = 128 * 1024;

pub fn run(root: &Path, arguments: &[String]) -> io::Result<usize> {
    let bytes = fs::read(root.join(CONTRACT))?;
    let contract: Value = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    let mut problems = validate_contract(&contract, bytes.len() as u64);
    problems.extend(validate_repository(root, &contract));
    if arguments.is_empty() {
        return report(problems);
    }
    if arguments.first().map(String::as_str) != Some("validate-report")
        || !(2..=3).contains(&arguments.len())
    {
        return Err(invalid_input(
            "use validate-report <report.json> [artifact-directory]",
        ));
    }
    problems.extend(validate_report(
        &contract,
        &bytes,
        Path::new(&arguments[1]),
    )?);
    if let Some(directory) = arguments.get(2) {
        problems.extend(validate_transferred_artifacts(
            Path::new(&arguments[1]),
            Path::new(directory),
        )?);
    }
    report(problems)
}

fn validate_contract(value: &Value, byte_len: u64) -> Vec<String> {
    let mut problems = Vec::new();
    if byte_len > MAX_CONTRACT_BYTES {
        problems.push("package contract exceeds 128 KiB".into());
    }
    exact_keys(
        value,
        &[
            "schema_version",
            "release_identity",
            "primary_artifacts",
            "shared_entries",
            "checksum_sidecars",
            "prohibited_tokens",
            "pe_imports",
            "installer_effects",
            "user_owned_fixtures",
            "lifecycle_cases",
            "predecessor",
            "tooling",
            "report_limits",
            "workflow_order",
        ],
        "contract",
        &mut problems,
    );
    if value["schema_version"] != 1 {
        problems.push("contract schema_version must be 1".into());
    }
    validate_release_identity(&value["release_identity"], &mut problems);
    validate_artifacts(value, &mut problems);
    validate_imports(&value["pe_imports"], &mut problems);
    validate_lifecycle(value, &mut problems);
    validate_predecessor(&value["predecessor"], &mut problems);
    validate_tooling(&value["tooling"], &mut problems);
    validate_limits(&value["report_limits"], &mut problems);
    exact_strings(
        value,
        "workflow_order",
        &[
            "identity",
            "build",
            "supply-chain-evidence",
            "assemble",
            "checksums",
            "certify",
            "upload-certified",
            "revalidate",
            "github-release",
            "crates-publish",
        ],
        &mut problems,
    );
    problems
}

fn validate_release_identity(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &[
            "product",
            "target",
            "architecture",
            "pe_machine",
            "features",
            "deep_capture_backend",
        ],
        "release_identity",
        problems,
    );
    for (key, expected) in [
        ("product", "fragcap"),
        ("target", "x86_64-pc-windows-msvc"),
        ("architecture", "x86_64"),
        ("pe_machine", "8664"),
        ("deep_capture_backend", "fragcap-native"),
    ] {
        if value[key] != expected {
            problems.push(format!("release_identity.{key} must be {expected}"));
        }
    }
    let features = string_set(&value["features"], "release_identity.features", problems);
    let expected = BTreeSet::from([
        "etw".to_string(),
        "live".to_string(),
        "native-deep-capture".to_string(),
        "socket-table".to_string(),
    ]);
    if features != expected {
        problems.push(format!("release feature set mismatch: {features:?}"));
    }
}

fn validate_artifacts(value: &Value, problems: &mut Vec<String>) {
    let expected_artifacts = BTreeSet::from([
        "portable-zip".to_string(),
        "standalone-catalog".to_string(),
        "windows-msi".to_string(),
    ]);
    let mut artifact_ids = BTreeSet::new();
    let Some(artifacts) = value["primary_artifacts"].as_array() else {
        problems.push("primary_artifacts must be an array".into());
        return;
    };
    for artifact in artifacts {
        exact_keys(
            artifact,
            &["id", "filename", "signature", "size_ceiling_bytes"],
            "primary artifact",
            problems,
        );
        let id = required_string(artifact, "id", "primary artifact", problems);
        if !artifact_ids.insert(id.clone()) {
            problems.push(format!("duplicate primary artifact {id}"));
        }
        if !matches!(
            artifact["size_ceiling_bytes"].as_u64(),
            Some(1..=67_108_864)
        ) {
            problems.push(format!("{id} has an invalid size ceiling"));
        }
        let signature = artifact["signature"].as_str().unwrap_or_default();
        if (id == "windows-msi" && signature != "not_signed")
            || (id != "windows-msi" && signature != "not_applicable")
        {
            problems.push(format!("{id} has an invalid signature policy"));
        }
    }
    if artifact_ids != expected_artifacts {
        problems.push(format!("primary artifact set mismatch: {artifact_ids:?}"));
    }
    let sidecars = string_set(&value["checksum_sidecars"], "checksum_sidecars", problems);
    if sidecars != expected_artifacts {
        problems.push(format!("checksum sidecar set mismatch: {sidecars:?}"));
    }

    let expected_entries = BTreeSet::from([
        "LICENSE".to_string(),
        "NOTICE".to_string(),
        "THIRD-PARTY-NOTICES.txt".to_string(),
        "catalog.db".to_string(),
        "fragcap.cdx.json".to_string(),
        "fragcap.exe".to_string(),
    ]);
    let mut entries = BTreeSet::new();
    let mut folded = BTreeSet::new();
    let Some(shared) = value["shared_entries"].as_array() else {
        problems.push("shared_entries must be an array".into());
        return;
    };
    for entry in shared {
        exact_keys(
            entry,
            &["path", "role", "size_ceiling_bytes", "signature"],
            "shared entry",
            problems,
        );
        let path = required_string(entry, "path", "shared entry", problems);
        if !entries.insert(path.clone()) || !folded.insert(path.to_ascii_lowercase()) {
            problems.push(format!("duplicate shared entry {path}"));
        }
        if path.contains('/') || path.contains('\\') || path.contains("..") {
            problems.push(format!("shared entry is not a canonical basename: {path}"));
        }
        if !matches!(entry["size_ceiling_bytes"].as_u64(), Some(1..=67_108_864)) {
            problems.push(format!("{path} has an invalid size ceiling"));
        }
        let expected_signature = if path == "fragcap.exe" {
            "not_signed"
        } else {
            "not_applicable"
        };
        if entry["signature"] != expected_signature {
            problems.push(format!("{path} has an invalid signature policy"));
        }
    }
    if entries != expected_entries {
        problems.push(format!("shared package entry set mismatch: {entries:?}"));
    }
}

fn validate_imports(value: &Value, problems: &mut Vec<String>) {
    exact_keys(value, &["ordinary", "delayed"], "pe_imports", problems);
    let ordinary = string_set(&value["ordinary"], "pe_imports.ordinary", problems);
    let delayed = string_set(&value["delayed"], "pe_imports.delayed", problems);
    if delayed != BTreeSet::from(["wpcap.dll".to_string()]) {
        problems.push(format!("delayed import set mismatch: {delayed:?}"));
    }
    for required in ["kernel32.dll", "crypt32.dll", "ws2_32.dll"] {
        if !ordinary.contains(required) {
            problems.push(format!("ordinary import allowlist lacks {required}"));
        }
    }
    for name in ordinary.iter().chain(delayed.iter()) {
        if name != &name.to_ascii_lowercase() || !name.ends_with(".dll") {
            problems.push(format!("PE import is not canonical: {name}"));
        }
        if ["mitmdump", "mitmproxy", "openssl", "python"]
            .iter()
            .any(|token| name.contains(token))
        {
            problems.push(format!("PE import is prohibited: {name}"));
        }
    }
}

fn validate_lifecycle(value: &Value, problems: &mut Vec<String>) {
    let expected = BTreeSet::from([
        "clean-install".to_string(),
        "downgrade-refusal".to_string(),
        "repair".to_string(),
        "same-version-reinstall".to_string(),
        "uninstall".to_string(),
        "upgrade".to_string(),
    ]);
    let mut observed = BTreeSet::new();
    let Some(cases) = value["lifecycle_cases"].as_array() else {
        problems.push("lifecycle_cases must be an array".into());
        return;
    };
    for case in cases {
        exact_keys(
            case,
            &["id", "timeout_seconds", "terminal"],
            "lifecycle case",
            problems,
        );
        let id = required_string(case, "id", "lifecycle case", problems);
        if !observed.insert(id.clone()) {
            problems.push(format!("duplicate lifecycle case {id}"));
        }
        if !matches!(case["timeout_seconds"].as_u64(), Some(1..=600)) {
            problems.push(format!("{id} timeout must be between 1 and 600 seconds"));
        }
        let expected_terminal = if id == "downgrade-refusal" {
            "refused_as_expected"
        } else {
            "passed"
        };
        if case["terminal"] != expected_terminal {
            problems.push(format!("{id} has an invalid terminal outcome"));
        }
    }
    if observed != expected {
        problems.push(format!("lifecycle case set mismatch: {observed:?}"));
    }
    exact_strings(
        value,
        "installer_effects",
        &[
            "defender-exclusion-if-created",
            "installed-files",
            "product-registration",
            "system-path-entry",
        ],
        problems,
    );
    exact_strings(
        value,
        "user_owned_fixtures",
        &[
            "capture",
            "deep-capture-bundle",
            "extcap-registration",
            "local-database",
            "preexisting-defender-exclusion",
            "writable-catalog",
        ],
        problems,
    );
}

fn validate_predecessor(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &["version", "url", "size_bytes", "sha256"],
        "predecessor",
        problems,
    );
    if value["version"] != "0.8.0"
        || value["size_bytes"] != 3_436_544
        || value["sha256"] != "eaf2554b1da3721400c1b00f5ea0a298455f59454b0084e617ed2efcdcf83901"
        || !value["url"].as_str().is_some_and(|url| {
            url.starts_with("https://github.com/h8rt3rmin8r/fragcap/releases/download/v0.8.0/")
        })
    {
        problems.push("predecessor identity is not the reviewed v0.8.0 MSI".into());
    }
}

fn validate_tooling(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &[
            "cargo_wix",
            "npcap_sdk",
            "npcap_sdk_sha256",
            "wixtoolset_chocolatey",
        ],
        "tooling",
        problems,
    );
    for (key, expected) in [
        ("cargo_wix", "0.3.9"),
        ("npcap_sdk", "1.16"),
        (
            "npcap_sdk_sha256",
            "f0a8be7778ee3ae1b99bbbecb27a3ff0f6c111a4093f1c78c5c5a099607184db",
        ),
        ("wixtoolset_chocolatey", "3.14.1.20250415"),
    ] {
        if value[key] != expected {
            problems.push(format!("tooling.{key} must be {expected}"));
        }
    }
}

fn validate_limits(value: &Value, problems: &mut Vec<String>) {
    exact_keys(
        value,
        &[
            "max_findings",
            "max_path_chars",
            "max_report_bytes",
            "max_string_chars",
        ],
        "report_limits",
        problems,
    );
    for (key, range) in [
        ("max_findings", 1..=256),
        ("max_path_chars", 64..=1024),
        ("max_report_bytes", 1024..=1_048_576),
        ("max_string_chars", 64..=4096),
    ] {
        if value[key]
            .as_u64()
            .is_none_or(|number| !range.contains(&number))
        {
            problems.push(format!("report_limits.{key} is outside its bound"));
        }
    }
}

fn validate_repository(root: &Path, contract: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    validate_markers(
        root,
        WIX_SOURCE,
        &[
            "AllowSameVersionUpgrades=\"yes\"",
            "Schedule=\"afterInstallInitialize\"",
            "FRAGCAP_DEFENDER_EXCLUSION_OWNER",
            "if(-not ((Get-MpPreference).ExclusionPath -contains $p)){Remove-ItemProperty",
            "SetDefenderRemove\" Before=\"RemoveFiles\">REMOVE=\"ALL\"",
            "Name=\"fragcap.exe\"",
            "Name=\"catalog.db\"",
            "Name=\"LICENSE\"",
            "Name=\"NOTICE\"",
            "Name=\"fragcap.cdx.json\"",
            "Name=\"THIRD-PARTY-NOTICES.txt\"",
        ],
        &mut problems,
    );
    validate_markers(
        root,
        PACKAGE_WORKFLOW,
        &[
            "workflow_call:",
            "pull_request:",
            "windows-latest",
            "cargo xtask package-certification",
            "Test-PackageCertification.ps1",
            "validate-report",
            "cargo-wix --locked --version 0.3.9",
            "wixtoolset --version=3.14.1.20250415",
            "npcap-sdk-1.16.zip",
            "eaf2554b1da3721400c1b00f5ea0a298455f59454b0084e617ed2efcdcf83901",
            "upload-artifact@v4",
        ],
        &mut problems,
    );
    validate_markers(
        root,
        RELEASE_WORKFLOW,
        &[
            "uses: ./.github/workflows/package-certification.yml",
            "needs: package-certification",
            "cargo xtask package-certification validate-report certification/report.json dist",
            "Assert the tag matches the workspace version",
        ],
        &mut problems,
    );
    validate_markers(
        root,
        HARNESS,
        &[
            "SupportsShouldProcess=$true",
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "CreateNoWindow = $true",
            "ProcessWindowStyle]::Hidden",
            "Get-AuthenticodeSignature",
            "Get-FileHash",
            "dumpbin.exe",
            "-Recurse -File",
            "Uninstall registered certification product during final cleanup",
            "New-NetFirewallRule",
            "Get-NetTCPConnection",
            "Get-NetUDPEndpoint",
            "unexpected_process_paths",
            "fragcap\\captures\\preserved.fcapng",
            "Wireshark\\extcap\\fragcap.exe",
            "ProductName",
            "ProductVersion",
            "Manufacturer",
            "UpgradeCode",
            "entries",
            "pe_inspections",
            "clean-install",
            "same-version-reinstall",
            "downgrade-refusal",
            "## End of script",
        ],
        &mut problems,
    );
    validate_markers(
        root,
        "crates/fragcap-cli/build.rs",
        &[
            "FRAGCAP_SOURCE_REVISION",
            "FRAGCAP_OFFICIAL_BUILD",
            "FRAGCAP_BUILD_TARGET",
        ],
        &mut problems,
    );
    validate_markers(
        root,
        "crates/fragcap-cli/src/cli.rs",
        &["__build-identity", "BuildIdentity"],
        &mut problems,
    );
    let readme = fs::read_to_string(root.join("README.md")).unwrap_or_default();
    if readme.contains("`hint.db`") {
        problems.push("README still names hint.db as a release download".into());
    }
    let declared_features = string_set(
        &contract["release_identity"]["features"],
        "release features",
        &mut problems,
    );
    if declared_features.contains("net") {
        problems.push("official package contract may not enable the net feature".into());
    }
    problems
}

fn validate_report(
    contract: &Value,
    contract_bytes: &[u8],
    path: &Path,
) -> io::Result<Vec<String>> {
    let limits = &contract["report_limits"];
    let max_bytes = limits["max_report_bytes"].as_u64().unwrap_or(0);
    let bytes = fs::read(path)?;
    let mut problems = Vec::new();
    if bytes.len() as u64 > max_bytes {
        problems.push(format!("certification report exceeds {max_bytes} bytes"));
        return Ok(problems);
    }
    let report: Value = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    exact_keys(
        &report,
        &[
            "schema_version",
            "contract_sha256",
            "release_identity",
            "build_identity",
            "artifacts",
            "entries",
            "pe_inspections",
            "smoke",
            "lifecycle",
            "findings",
            "complete",
        ],
        "certification report",
        &mut problems,
    );
    if report["schema_version"] != 1 {
        problems.push("report schema_version must be 1".into());
    }
    if report["contract_sha256"] != sha256(contract_bytes) {
        problems.push("report contract digest does not match current contract".into());
    }
    if report["release_identity"] != contract["release_identity"] {
        problems.push("report release identity does not match the package contract".into());
    }
    validate_build_identity(contract, &report["build_identity"], &mut problems);
    validate_report_rows(contract, &report, &mut problems);
    let findings = report["findings"].as_array();
    if findings.is_none_or(|rows| !rows.is_empty()) {
        problems.push("certification report contains findings or lacks a findings array".into());
    }
    if report["complete"] != true {
        problems.push("certification report is not complete".into());
    }
    let text = String::from_utf8_lossy(&bytes);
    for forbidden in [
        "C:\\Users\\",
        "C:/Users/",
        "RUNNER_TEMP",
        "github.workspace",
        "@example.",
    ] {
        if text.contains(forbidden) {
            problems.push(format!(
                "certification report contains host-sensitive text: {forbidden}"
            ));
        }
    }
    Ok(problems)
}

fn validate_transferred_artifacts(report_path: &Path, directory: &Path) -> io::Result<Vec<String>> {
    let report: Value = serde_json::from_slice(&fs::read(report_path)?).map_err(invalid_data)?;
    let mut problems = Vec::new();
    let mut expected_names = BTreeSet::new();
    for row in report["artifacts"].as_array().into_iter().flatten() {
        let Some(filename) = row["filename"].as_str() else {
            problems.push("transferred artifact report row lacks a filename".into());
            continue;
        };
        if Path::new(filename)
            .file_name()
            .and_then(|name| name.to_str())
            != Some(filename)
            || !expected_names.insert(filename.to_string())
        {
            problems.push(format!(
                "unsafe or duplicate transferred artifact name: {filename}"
            ));
            continue;
        }
        let path = directory.join(filename);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                problems.push(format!(
                    "could not read transferred artifact {filename}: {error}"
                ));
                continue;
            }
        };
        if row["size_bytes"].as_u64() != Some(bytes.len() as u64)
            || row["sha256"].as_str() != Some(sha256(&bytes).as_str())
        {
            problems.push(format!(
                "transferred artifact {filename} differs from the certified size or digest"
            ));
        }
    }
    let observed_names = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|kind| kind.is_file())
                .and_then(|_| entry.file_name().into_string().ok())
        })
        .collect::<BTreeSet<_>>();
    if observed_names != expected_names {
        problems.push(format!(
            "transferred artifact set mismatch: {observed_names:?}"
        ));
    }
    Ok(problems)
}

fn validate_report_rows(contract: &Value, report: &Value, problems: &mut Vec<String>) {
    let version = report["build_identity"]["version"]
        .as_str()
        .unwrap_or_default();
    let mut expected_artifacts = contract["primary_artifacts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            Some((
                row["id"].as_str()?.to_string(),
                (
                    row["filename"].as_str()?.replace("{version}", version),
                    row["size_ceiling_bytes"].as_u64()?,
                    row["signature"].as_str()?.to_string(),
                ),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let primary = expected_artifacts.clone();
    for id in contract["checksum_sidecars"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if let Some((filename, _, _)) = primary.get(id) {
            expected_artifacts.insert(
                format!("{id}-checksum"),
                (format!("{filename}.sha256"), 1024, "not_applicable".into()),
            );
        }
    }
    let artifact_rows = report["artifacts"].as_array();
    let observed_artifacts = artifact_rows
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let id = row["id"].as_str().unwrap_or_default();
            exact_keys(
                row,
                &["id", "filename", "size_bytes", "sha256", "signature", "complete"],
                "artifact report row",
                problems,
            );
            let expected = expected_artifacts.get(id);
            if row["complete"] != true
                || !is_sha256(row["sha256"].as_str().unwrap_or_default())
                || expected.is_none_or(|(filename, ceiling, signature)| {
                    row["filename"] != *filename
                        || row["size_bytes"]
                            .as_u64()
                            .is_none_or(|size| size == 0 || size > *ceiling)
                        || row["signature"] != *signature
                })
            {
                problems.push("artifact report row has invalid identity, size, digest, signature, or completion".into());
            }
            row["id"].as_str().map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    let expected_artifact_ids = expected_artifacts.keys().cloned().collect::<BTreeSet<_>>();
    if observed_artifacts != expected_artifact_ids {
        problems.push(format!(
            "artifact report set mismatch: {observed_artifacts:?}"
        ));
    }
    if artifact_rows.is_none_or(|rows| rows.len() != expected_artifact_ids.len()) {
        problems.push("artifact report contains a duplicate or missing row".into());
    }
    let expected_entries = contract["shared_entries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            Some((
                row["path"].as_str()?.to_string(),
                (
                    row["role"].as_str()?.to_string(),
                    row["size_ceiling_bytes"].as_u64()?,
                    row["signature"].as_str()?.to_string(),
                ),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let entry_rows = report["entries"].as_array();
    let observed_entries = entry_rows
        .into_iter()
        .flatten()
        .filter_map(|row| {
            exact_keys(
                row,
                &["path", "role", "size_bytes", "sha256", "signature", "complete"],
                "package entry report row",
                problems,
            );
            let path = row["path"].as_str().unwrap_or_default();
            let expected = expected_entries.get(path);
            if row["complete"] != true
                || !is_sha256(row["sha256"].as_str().unwrap_or_default())
                || expected.is_none_or(|(role, ceiling, signature)| {
                    row["role"] != *role
                        || row["size_bytes"]
                            .as_u64()
                            .is_none_or(|size| size > *ceiling)
                        || row["signature"] != *signature
                })
            {
                problems.push("package entry report row has invalid identity, size, digest, signature, or completion".into());
            }
            row["path"].as_str().map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    let expected_entry_paths = expected_entries.keys().cloned().collect::<BTreeSet<_>>();
    if observed_entries != expected_entry_paths
        || entry_rows.is_none_or(|rows| rows.len() != expected_entry_paths.len())
    {
        problems.push(format!(
            "package entry report set mismatch: {observed_entries:?}"
        ));
    }
    let expected_lifecycle = contract["lifecycle_cases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            row["id"].as_str().map(|id| {
                (
                    id.to_string(),
                    row["terminal"].as_str().unwrap_or_default().to_string(),
                )
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let lifecycle_rows = report["lifecycle"].as_array();
    let observed_lifecycle = lifecycle_rows
        .into_iter()
        .flatten()
        .filter_map(|row| {
            if row["complete"] != true
                || row["cleanup"] != "reconciled"
                || row["elapsed_seconds"]
                    .as_u64()
                    .is_none_or(|seconds| seconds > 600)
            {
                problems
                    .push("lifecycle report row is incomplete, unbounded, or unreconciled".into());
            }
            row["id"].as_str().map(|id| {
                (
                    id.to_string(),
                    row["terminal"].as_str().unwrap_or_default().to_string(),
                )
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    if observed_lifecycle != expected_lifecycle {
        problems.push(format!(
            "lifecycle report set mismatch: {observed_lifecycle:?}"
        ));
    }
    if lifecycle_rows.is_none_or(|rows| rows.len() != expected_lifecycle.len()) {
        problems.push("lifecycle report contains a duplicate or missing row".into());
    }
    let pe = report["pe_inspections"].as_array();
    let expected_surfaces = BTreeSet::from(["installed-msi", "portable-zip"]);
    if pe.is_none_or(|rows| {
        rows.len() != 2
            || rows
                .iter()
                .filter_map(|row| row["surface"].as_str())
                .collect::<BTreeSet<_>>()
                != expected_surfaces
            || rows.iter().any(|row| {
                exact_keys(
                    row,
                    &[
                        "surface",
                        "machine",
                        "ordinary_imports",
                        "delayed_imports",
                        "file_version",
                        "product_version",
                        "product_name",
                        "original_filename",
                        "signature",
                        "complete",
                    ],
                    "PE inspection row",
                    problems,
                );
                row["complete"] != true
                    || row["signature"] != "not_signed"
                    || row["machine"] != contract["release_identity"]["pe_machine"]
                    || row["ordinary_imports"] != contract["pe_imports"]["ordinary"]
                    || row["delayed_imports"] != contract["pe_imports"]["delayed"]
                    || row["file_version"] != format!("{version}.0")
                    || row["product_version"] != version
                    || row["product_name"] != "fragcap"
                    || row["original_filename"] != "fragcap.exe"
            })
    }) {
        problems
            .push("report must contain two complete identity-bound unsigned PE inspections".into());
    }
    exact_keys(
        &report["smoke"],
        &[
            "backend",
            "network",
            "process_observation",
            "network_observation",
            "samples",
            "complete",
        ],
        "smoke report",
        problems,
    );
    if report["smoke"]["complete"] != true
        || report["smoke"]["backend"] != "fragcap-native"
        || report["smoke"]["network"] != "loopback-only"
        || report["smoke"]["process_observation"] != "complete"
        || report["smoke"]["network_observation"] != "firewall-contained-and-socket-observed"
        || report["smoke"]["samples"]
            .as_u64()
            .is_none_or(|samples| samples == 0)
    {
        problems.push("packaged native smoke is incomplete".into());
    }
}

fn validate_build_identity(contract: &Value, identity: &Value, problems: &mut Vec<String>) {
    exact_keys(
        identity,
        &[
            "schema_version",
            "product",
            "version",
            "source_revision",
            "target",
            "architecture",
            "features",
            "deep_capture_backend",
            "official",
        ],
        "build_identity",
        problems,
    );
    if identity["schema_version"] != 1
        || identity["product"] != contract["release_identity"]["product"]
        || identity["target"] != contract["release_identity"]["target"]
        || identity["architecture"] != contract["release_identity"]["architecture"]
        || identity["features"] != contract["release_identity"]["features"]
        || identity["deep_capture_backend"] != contract["release_identity"]["deep_capture_backend"]
        || identity["official"] != true
    {
        problems.push("report build identity does not match the official package contract".into());
    }
    let version = identity["version"].as_str().unwrap_or_default();
    if version.split('.').count() != 3
        || version.split('.').any(|part| part.parse::<u16>().is_err())
    {
        problems.push("report build identity version must have three numeric fields".into());
    }
    let revision = identity["source_revision"].as_str().unwrap_or_default();
    if !matches!(revision.len(), 40 | 64)
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        problems
            .push("report build identity source revision is not a full lowercase digest".into());
    }
}

fn validate_markers(root: &Path, relative: &str, markers: &[&str], problems: &mut Vec<String>) {
    let path = root.join(relative);
    let Ok(text) = fs::read_to_string(&path) else {
        problems.push(format!("missing package authority {relative}"));
        return;
    };
    for marker in markers {
        if !text.contains(marker) {
            problems.push(format!("{relative} lacks required marker {marker}"));
        }
    }
}

fn exact_keys(value: &Value, expected: &[&str], label: &str, problems: &mut Vec<String>) {
    let Some(object) = value.as_object() else {
        problems.push(format!("{label} must be an object"));
        return;
    };
    let observed = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if observed != expected {
        problems.push(format!("{label} keys mismatch: {observed:?}"));
    }
}

fn exact_strings(value: &Value, key: &str, expected: &[&str], problems: &mut Vec<String>) {
    let observed = string_set(&value[key], key, problems);
    let expected = expected
        .iter()
        .map(|item| (*item).to_string())
        .collect::<BTreeSet<_>>();
    if observed != expected {
        problems.push(format!("{key} set mismatch: {observed:?}"));
    }
}

fn string_set(value: &Value, label: &str, problems: &mut Vec<String>) -> BTreeSet<String> {
    let Some(items) = value.as_array() else {
        problems.push(format!("{label} must be an array"));
        return BTreeSet::new();
    };
    let mut set = BTreeSet::new();
    for item in items {
        let Some(item) = item.as_str() else {
            problems.push(format!("{label} contains a non-string"));
            continue;
        };
        if !set.insert(item.to_string()) {
            problems.push(format!("{label} contains duplicate {item}"));
        }
    }
    set
}

fn required_string(value: &Value, key: &str, label: &str, problems: &mut Vec<String>) -> String {
    match value[key].as_str() {
        Some(text) if !text.is_empty() => text.to_string(),
        _ => {
            problems.push(format!("{label}.{key} must be a non-empty string"));
            String::new()
        }
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn sha256(bytes: &[u8]) -> String {
    digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn report(problems: Vec<String>) -> io::Result<usize> {
    for problem in &problems {
        eprintln!("package-certification: {problem}");
    }
    if problems.is_empty() {
        println!("package-certification: final Windows package contract is complete");
    }
    Ok(problems.len())
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn invalid_input(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> Value {
        serde_json::from_slice(include_bytes!(
            "../../integration/windows-package-contract-v1.json"
        ))
        .expect("contract")
    }

    #[test]
    fn checked_in_contract_is_closed_and_complete() {
        assert_eq!(validate_contract(&contract(), 1), Vec::<String>::new());
    }

    #[test]
    fn unknown_contract_key_is_rejected() {
        let mut value = contract();
        value
            .as_object_mut()
            .expect("object")
            .insert("escape".into(), Value::Bool(true));
        assert!(validate_contract(&value, 1)
            .iter()
            .any(|problem| problem.contains("keys mismatch")));
    }

    #[test]
    fn recursive_checksum_and_missing_lifecycle_are_rejected() {
        let mut value = contract();
        value["checksum_sidecars"]
            .as_array_mut()
            .expect("sidecars")
            .push(Value::String("portable-zip.sha256".into()));
        value["lifecycle_cases"]
            .as_array_mut()
            .expect("cases")
            .pop();
        let problems = validate_contract(&value, 1);
        assert!(problems
            .iter()
            .any(|problem| problem.contains("checksum sidecar set mismatch")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("lifecycle case set mismatch")));
    }

    #[test]
    fn oversized_entry_and_unknown_import_are_rejected() {
        let mut value = contract();
        value["shared_entries"][0]["size_ceiling_bytes"] = Value::from(100_000_000_u64);
        value["pe_imports"]["ordinary"]
            .as_array_mut()
            .expect("imports")
            .push(Value::String("python312.dll".into()));
        let problems = validate_contract(&value, 1);
        assert!(problems
            .iter()
            .any(|problem| problem.contains("invalid size ceiling")));
        assert!(problems
            .iter()
            .any(|problem| problem.contains("PE import is prohibited")));
    }

    #[test]
    fn malformed_digest_is_not_sha256() {
        assert!(!is_sha256("ABC"));
        assert!(!is_sha256(&"G".repeat(64)));
        assert!(is_sha256(&"a".repeat(64)));
    }

    #[test]
    fn transferred_artifacts_must_match_the_certified_bytes() {
        let root = std::env::temp_dir().join(format!(
            "fragcap-transferred-artifacts-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let report_path = root.join("report.json");
        let artifact_root = root.join("dist");
        fs::create_dir_all(&artifact_root).unwrap();
        fs::write(artifact_root.join("fragcap.zip"), b"certified").unwrap();
        let report = serde_json::json!({"artifacts": [{"filename": "fragcap.zip", "size_bytes": 9, "sha256": sha256(b"certified")}]});
        fs::write(&report_path, serde_json::to_vec(&report).unwrap()).unwrap();
        assert!(validate_transferred_artifacts(&report_path, &artifact_root)
            .unwrap()
            .is_empty());
        fs::write(artifact_root.join("fragcap.zip"), b"substituted").unwrap();
        assert!(validate_transferred_artifacts(&report_path, &artifact_root)
            .unwrap()
            .iter()
            .any(|problem| problem.contains("differs from the certified")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn complete_report_reconciles_and_duplicate_row_fails() {
        let contract_bytes = include_bytes!("../../integration/windows-package-contract-v1.json");
        let contract = contract();
        let mut artifacts = contract["primary_artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|artifact| {
                serde_json::json!({"id": artifact["id"], "filename": artifact["filename"].as_str().unwrap().replace("{version}", "0.9.0"), "size_bytes": 1, "sha256": "a".repeat(64), "signature": artifact["signature"], "complete": true})
            })
            .collect::<Vec<_>>();
        for artifact in contract["primary_artifacts"].as_array().unwrap() {
            artifacts.push(serde_json::json!({"id": format!("{}-checksum", artifact["id"].as_str().unwrap()), "filename": format!("{}.sha256", artifact["filename"].as_str().unwrap().replace("{version}", "0.9.0")), "size_bytes": 80, "sha256": "b".repeat(64), "signature": "not_applicable", "complete": true}));
        }
        let entries = contract["shared_entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| serde_json::json!({"path": entry["path"], "role": entry["role"], "size_bytes": 1, "sha256": "c".repeat(64), "signature": entry["signature"], "complete": true}))
            .collect::<Vec<_>>();
        let lifecycle = contract["lifecycle_cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|case| {
                serde_json::json!({"id": case["id"], "terminal": case["terminal"], "cleanup": "reconciled", "elapsed_seconds": 1, "complete": true})
            })
            .collect::<Vec<_>>();
        let build_identity = serde_json::json!({"schema_version": 1, "product": "fragcap", "version": "0.9.0", "source_revision": "a".repeat(40), "target": "x86_64-pc-windows-msvc", "architecture": "x86_64", "features": contract["release_identity"]["features"], "deep_capture_backend": "fragcap-native", "official": true});
        let portable_pe = serde_json::json!({"surface": "portable-zip", "machine": "8664", "ordinary_imports": contract["pe_imports"]["ordinary"], "delayed_imports": contract["pe_imports"]["delayed"], "file_version": "0.9.0.0", "product_version": "0.9.0", "product_name": "fragcap", "original_filename": "fragcap.exe", "signature": "not_signed", "complete": true});
        let mut installed_pe = portable_pe.clone();
        installed_pe["surface"] = Value::String("installed-msi".into());
        let mut value = serde_json::json!({"schema_version": 1, "contract_sha256": sha256(contract_bytes), "release_identity": contract["release_identity"], "build_identity": build_identity, "artifacts": artifacts, "entries": entries, "pe_inspections": [portable_pe, installed_pe], "smoke": {"backend": "fragcap-native", "network": "loopback-only", "process_observation": "complete", "network_observation": "firewall-contained-and-socket-observed", "samples": 1, "complete": true}, "lifecycle": lifecycle, "findings": [], "complete": true});
        let path = std::env::temp_dir().join(format!(
            "fragcap-package-report-{}.json",
            std::process::id()
        ));
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(validate_report(&contract, contract_bytes, &path)
            .unwrap()
            .is_empty());
        let valid = value.clone();
        macro_rules! rejects {
            ($label:literal, $mutation:expr) => {{
                let mut candidate = valid.clone();
                $mutation(&mut candidate);
                fs::write(&path, serde_json::to_vec(&candidate).unwrap()).unwrap();
                assert!(
                    !validate_report(&contract, contract_bytes, &path)
                        .unwrap()
                        .is_empty(),
                    "{} mutation must block certification",
                    $label
                );
            }};
        }
        rejects!("missing", |candidate: &mut Value| {
            candidate["entries"].as_array_mut().unwrap().pop();
        });
        rejects!("extra", |candidate: &mut Value| {
            candidate["artifacts"].as_array_mut().unwrap().push(
                serde_json::json!({"id": "extra", "filename": "extra", "size_bytes": 1, "sha256": "a".repeat(64), "signature": "not_applicable", "complete": true}),
            );
        });
        rejects!("altered", |candidate: &mut Value| {
            candidate["entries"][0]["sha256"] = Value::String("altered".into());
        });
        rejects!("stale", |candidate: &mut Value| {
            candidate["contract_sha256"] = Value::String("d".repeat(64));
        });
        rejects!("mis-versioned", |candidate: &mut Value| {
            candidate["build_identity"]["version"] = Value::String("0.9".into());
        });
        rejects!("mis-featured", |candidate: &mut Value| {
            candidate["build_identity"]["features"] = serde_json::json!(["live"]);
        });
        rejects!("prohibited", |candidate: &mut Value| {
            candidate["smoke"]["network"] = Value::String("python-fetch".into());
        });
        rejects!("unsigned-policy", |candidate: &mut Value| {
            candidate["pe_inspections"][0]["signature"] = Value::String("valid".into());
        });
        rejects!("checksum", |candidate: &mut Value| {
            candidate["artifacts"][3]["sha256"] = Value::String("wrong".into());
        });
        rejects!("traversal", |candidate: &mut Value| {
            candidate["entries"][0]["path"] = Value::String("../fragcap.exe".into());
        });
        rejects!("lifecycle", |candidate: &mut Value| {
            candidate["lifecycle"][0]["terminal"] = Value::String("failed".into());
        });
        rejects!("timeout", |candidate: &mut Value| {
            candidate["lifecycle"][0]["elapsed_seconds"] = Value::from(601);
        });
        rejects!("residue", |candidate: &mut Value| {
            candidate["lifecycle"][0]["cleanup"] = Value::String("residue".into());
        });
        value = valid;
        let duplicate = value["artifacts"][0].clone();
        value["artifacts"].as_array_mut().unwrap().push(duplicate);
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(validate_report(&contract, contract_bytes, &path)
            .unwrap()
            .iter()
            .any(|problem| problem.contains("duplicate or missing row")));
        fs::remove_file(path).unwrap();
    }
}
