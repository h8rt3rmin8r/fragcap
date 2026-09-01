// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Component, Path, PathBuf};

use serde_json::{json, Value};

const JOURNAL: &str = ".sensitive-actions.jsonl";
const MAX_JOURNAL_BYTES: u64 = 1024 * 1024;
const MAX_JOURNAL_RECORDS: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SensitiveRetention {
    Retain,
}

impl SensitiveRetention {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retain => "retain-until-explicit-cleanup",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactActionResult {
    pub path: PathBuf,
    pub status: String,
    pub reason: String,
}

pub fn prepare_bundle(path: &Path) -> io::Result<()> {
    prepare_protected_directory(path)?;
    let journal = contained(path, Path::new(JOURNAL))?;
    match journal.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_file() => protect_path(&journal, false)?,
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "sensitive action journal is not a regular file",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut file = open_sensitive_file(&journal)?;
            file.write_all(b"{\"version\":1,\"type\":\"header\"}\n")?;
            file.sync_all()?;
        }
        Err(error) => return Err(error),
    }
    Ok(())
}

fn prepare_protected_directory(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    protect_path(path, true)?;
    Ok(())
}

pub fn open_sensitive_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(path)?;
    protect_path(path, false)?;
    Ok(file)
}

pub fn cleanup_sensitive(bundle: &Path) -> io::Result<Vec<ArtifactActionResult>> {
    let manifest = read_manifest(bundle)?;
    prepare_bundle(bundle)?;
    let paths = sensitive_paths(&manifest)?;
    let mut results = Vec::new();
    for relative in paths {
        contained(bundle, &relative)?;
        append_journal(bundle, "delete", &relative, "intent", "pending")?;
        let full = contained(bundle, &relative)?;
        let (status, reason) = match full.symlink_metadata() {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                ("already-absent", "sensitive artifact was already absent")
            }
            Err(error) => return Err(error),
            Ok(metadata) if !metadata.file_type().is_file() => (
                "failed",
                "only regular files are eligible for sensitive cleanup",
            ),
            Ok(_) => match fs::remove_file(&full) {
                Ok(()) => ("removed", "sensitive artifact removed"),
                Err(error) => {
                    results.push(ArtifactActionResult {
                        path: relative.clone(),
                        status: "failed".into(),
                        reason: error.to_string(),
                    });
                    append_journal(bundle, "delete", &relative, "result", "failed")?;
                    continue;
                }
            },
        };
        append_journal(bundle, "delete", &relative, "result", status)?;
        results.push(ArtifactActionResult {
            path: relative,
            status: status.into(),
            reason: reason.into(),
        });
    }
    Ok(results)
}

pub fn recover_sensitive_actions(bundle: &Path) -> io::Result<Vec<ArtifactActionResult>> {
    let journal = match contained(bundle, Path::new(JOURNAL)) {
        Ok(journal) => journal,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let metadata = match journal.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_file() => metadata,
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "sensitive action journal is not a regular file",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    if metadata.len() > MAX_JOURNAL_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sensitive action journal exceeds limit",
        ));
    }
    let mut pending = BTreeSet::new();
    for (index, line) in BufReader::new(File::open(&journal)?).lines().enumerate() {
        if index >= MAX_JOURNAL_RECORDS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "sensitive action journal exceeds record limit",
            ));
        }
        let value: Value = serde_json::from_str(&line?)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if value.get("op").and_then(Value::as_str) != Some("delete") {
            continue;
        }
        let Some(path) = value.get("path").and_then(Value::as_str) else {
            continue;
        };
        match value.get("phase").and_then(Value::as_str) {
            Some("intent") => {
                pending.insert(PathBuf::from(path));
            }
            Some("result") => {
                pending.remove(Path::new(path));
            }
            _ => {}
        }
    }
    let mut results = Vec::new();
    for relative in pending {
        let full = contained(bundle, &relative)?;
        let (status, reason) = match full.symlink_metadata() {
            Err(error) if error.kind() == io::ErrorKind::NotFound => (
                "already-absent",
                "pending sensitive artifact was already absent".into(),
            ),
            Err(error) => return Err(error),
            Ok(metadata) if !metadata.file_type().is_file() => (
                "failed",
                "only regular files are eligible for sensitive recovery".into(),
            ),
            Ok(_) => match fs::remove_file(&full) {
                Ok(()) => ("removed", "recovered pending sensitive cleanup".into()),
                Err(error) => ("failed", error.to_string()),
            },
        };
        append_journal(bundle, "delete", &relative, "result", status)?;
        results.push(ArtifactActionResult {
            path: relative,
            status: status.into(),
            reason,
        });
    }
    Ok(results)
}

pub fn export_share_copy(source: &Path, destination: &Path) -> io::Result<PathBuf> {
    let destination = normalized_absolute(destination)?;
    match destination.symlink_metadata() {
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "share destination already exists",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let source = source.canonicalize()?;
    let resolved_destination = resolve_location(&destination)?;
    if resolved_destination == source || resolved_destination.starts_with(&source) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "share destination must be outside the source bundle",
        ));
    }
    let manifest = read_manifest(&source)?;
    let parent = destination.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "share destination has no parent",
        )
    })?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(format!(
        ".{}.fragcap-staging",
        destination
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("share")
    ));
    match staging.symlink_metadata() {
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "share staging directory already exists",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    prepare_protected_directory(&staging)?;
    let sensitive = sensitive_paths(&manifest)?;
    let mut included = Vec::new();
    let mut omitted = Vec::new();
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest has no artifacts"))?;
    let result = (|| -> io::Result<()> {
        for artifact in artifacts {
            let Some(path) = artifact.get("path").and_then(Value::as_str) else {
                continue;
            };
            if path == "manifest.json" {
                continue;
            }
            let relative = super::validate_relative_path(path)?;
            let from = contained(&source, &relative)?;
            if sensitive.contains(&relative) {
                omitted.push(json!({"path": path, "reason": "sensitive"}));
                continue;
            }
            let metadata = from.symlink_metadata()?;
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "share source artifact is not a regular file",
                ));
            }
            let to = contained(&staging, &relative)?;
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to)?;
            included.push(json!({"path": path, "bytes": metadata.len()}));
        }
        if manifest.get("manifest_version").and_then(Value::as_u64) == Some(2) {
            let sharing_omissions = omitted
                .iter()
                .filter_map(|entry| {
                    let path = entry.get("path")?;
                    let role = manifest["artifacts"]
                        .as_array()?
                        .iter()
                        .find(|artifact| artifact.get("path") == Some(path))?
                        .get("role")?;
                    Some(json!({
                        "role":role,
                        "reason":"sharing-excludes-sensitive-artifact",
                        "severity":"info"
                    }))
                })
                .collect::<Vec<_>>();
            let mut shared = manifest.clone();
            shared["state"] = json!(if omitted.is_empty() {
                "complete"
            } else {
                "partial"
            });
            shared["sharing"] = json!({
                "source_bundle": source,
                "transformation": "sensitive-artifacts-omitted",
                "status": "complete"
            });
            if let Some(artifacts) = shared.get_mut("artifacts").and_then(Value::as_array_mut) {
                for artifact in artifacts {
                    let role = artifact.get("role").and_then(Value::as_str);
                    if role == Some("manifest") {
                        continue;
                    }
                    let Some(path) = artifact.get("path").and_then(Value::as_str) else {
                        continue;
                    };
                    if omitted.iter().any(|entry| entry["path"] == path) {
                        let object = artifact.as_object_mut().expect("artifact is an object");
                        object.remove("path");
                        object.insert("completeness".to_string(), json!("omitted"));
                        object.insert("finalization".to_string(), json!("complete"));
                        object.insert(
                            "omission_reason".to_string(),
                            json!("sharing-excludes-sensitive-artifact"),
                        );
                        object.insert("loss".to_string(), json!({"state":"not-applicable"}));
                        object.insert("correlation".to_string(), json!({"state":"not-applicable"}));
                    }
                }
            }
            shared["omissions"]
                .as_array_mut()
                .expect("version 2 manifest omissions are validated")
                .extend(sharing_omissions);
            super::validate_v2(&shared)?;
            let bytes = serde_json::to_vec_pretty(&shared).map_err(io::Error::other)?;
            let mut file = open_sensitive_file(&staging.join("manifest.json"))?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        } else {
            let from = contained(&source, Path::new("manifest.json"))?;
            let metadata = from.symlink_metadata()?;
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "source manifest is not a regular file",
                ));
            }
            fs::copy(&from, staging.join("manifest.json"))?;
            included.push(json!({"path":"manifest.json","bytes":metadata.len()}));
        }
        let sharing = serde_json::to_vec_pretty(&json!({
            "version": 1,
            "source": source,
            "included": included,
            "omitted": omitted,
            "status": "complete"
        }))
        .map_err(io::Error::other)?;
        let mut file = open_sensitive_file(&staging.join("sharing-manifest.json"))?;
        file.write_all(&sharing)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&staging, &destination)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;
    Ok(destination.join("sharing-manifest.json"))
}

fn read_manifest(bundle: &Path) -> io::Result<Value> {
    let bytes = fs::read(bundle.join("manifest.json"))?;
    let loose: Value = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if loose.get("manifest_version").is_some() {
        return Ok(super::ManifestDocument::parse(&bytes)?.value().clone());
    }
    Ok(loose)
}

fn sensitive_paths(manifest: &Value) -> io::Result<BTreeSet<PathBuf>> {
    let mut paths = BTreeSet::new();
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest has no artifacts"))?;
    for artifact in artifacts {
        let sensitivity = artifact
            .get("sensitivity")
            .and_then(Value::as_str)
            .unwrap_or("ordinary");
        if !matches!(sensitivity, "sensitive" | "secret" | "secret-adjacent") {
            continue;
        }
        if let Some(path) = artifact.get("path").and_then(Value::as_str) {
            let path = super::validate_relative_path(path)?;
            paths.insert(path);
        }
    }
    Ok(paths)
}

fn contained(root: &Path, relative: &Path) -> io::Result<PathBuf> {
    validate_relative(relative)?;
    let root = root.canonicalize()?;
    let candidate = root.join(relative);
    let resolved = resolve_location(&candidate)?;
    if !resolved.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "artifact path resolves outside the bundle",
        ));
    }
    Ok(candidate)
}

fn normalized_absolute(path: &Path) -> io::Result<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "share destination is empty",
        ));
    }
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "share destination must not contain a parent component",
                ));
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    Ok(normalized)
}

fn resolve_location(path: &Path) -> io::Result<PathBuf> {
    let mut ancestor = path.to_path_buf();
    let mut missing = Vec::<OsString>::new();
    loop {
        match ancestor.symlink_metadata() {
            Ok(_) => {
                let mut resolved = ancestor.canonicalize()?;
                for component in missing.iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let component = ancestor.file_name().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "path has no existing ancestor")
                })?;
                missing.push(component.to_os_string());
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "path has no existing ancestor")
                    })?
                    .to_path_buf();
            }
            Err(error) => return Err(error),
        }
    }
}

fn validate_relative(path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "artifact path is not a normalized relative path",
        ));
    }
    Ok(())
}

fn append_journal(
    bundle: &Path,
    op: &str,
    path: &Path,
    phase: &str,
    status: &str,
) -> io::Result<()> {
    let journal = contained(bundle, Path::new(JOURNAL))?;
    let metadata = journal.symlink_metadata()?;
    if !metadata.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sensitive action journal is not a regular file",
        ));
    }
    let current_bytes = metadata.len();
    if current_bytes > MAX_JOURNAL_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sensitive action journal exceeds limit",
        ));
    }
    let record =
        serde_json::to_vec(&json!({"version":1,"op":op,"path":path,"phase":phase,"status":status}))
            .map_err(io::Error::other)?;
    if current_bytes.saturating_add(record.len() as u64 + 1) > MAX_JOURNAL_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sensitive action journal would exceed byte limit",
        ));
    }
    let record_count = BufReader::new(File::open(&journal)?)
        .lines()
        .take(MAX_JOURNAL_RECORDS + 1)
        .count();
    if record_count >= MAX_JOURNAL_RECORDS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sensitive action journal would exceed record limit",
        ));
    }
    let mut file = OpenOptions::new().append(true).open(journal)?;
    file.write_all(&record)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

#[cfg(unix)]
fn protect_path(path: &Path, directory: bool) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(
        path,
        fs::Permissions::from_mode(if directory { 0o700 } else { 0o600 }),
    )
}

#[cfg(windows)]
fn protect_path(path: &Path, directory: bool) -> io::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::{
        SetFileSecurityW, DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
    };
    use windows_sys::Win32::System::Memory::LocalFree;
    let descriptor = if directory {
        "D:P(A;OICI;FA;;;OW)(A;OICI;FA;;;SY)"
    } else {
        "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
    };
    let sddl: Vec<u16> = descriptor
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut security = ptr::null_mut();
    let mut length = 0;
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut security,
            &mut length,
        )
    } == 0
    {
        return Err(io::Error::from_raw_os_error(
            unsafe { GetLastError() } as i32
        ));
    }
    let wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let applied = unsafe {
        SetFileSecurityW(
            wide.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            security,
        )
    };
    let error = unsafe { GetLastError() };
    unsafe { LocalFree(security as isize) };
    if applied == 0 {
        Err(io::Error::from_raw_os_error(error as i32))
    } else {
        Ok(())
    }
}

#[cfg(not(any(unix, windows)))]
fn protect_path(_: &Path, _: bool) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_is_exact_and_idempotent_and_share_preserves_source() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        prepare_bundle(&bundle).unwrap();
        fs::write(bundle.join("capture.fcapng"), b"ordinary").unwrap();
        fs::write(bundle.join("tls-keylog.log"), b"secret").unwrap();
        fs::write(bundle.join("manifest.json"), br#"{"artifacts":[{"path":"capture.fcapng","sensitivity":"ordinary"},{"path":"tls-keylog.log","sensitivity":"secret-adjacent"}]}"#).unwrap();
        let source_before = ["capture.fcapng", "tls-keylog.log", "manifest.json", JOURNAL]
            .map(|path| (path, fs::read(bundle.join(path)).unwrap()));
        let share = root.path().join("share");
        let sharing_manifest = export_share_copy(&bundle, &share).unwrap();
        for (path, bytes) in source_before {
            assert_eq!(fs::read(bundle.join(path)).unwrap(), bytes);
        }
        assert!(!share.join("tls-keylog.log").exists());
        assert!(share.join("capture.fcapng").exists());
        assert!(share.join("manifest.json").is_file());
        assert!(!share.join(JOURNAL).exists());
        let sharing: Value = serde_json::from_slice(&fs::read(sharing_manifest).unwrap()).unwrap();
        assert_eq!(sharing["status"], "complete");
        assert_eq!(sharing["included"].as_array().unwrap().len(), 2);
        assert_eq!(sharing["omitted"].as_array().unwrap().len(), 1);
        assert_eq!(cleanup_sensitive(&bundle).unwrap()[0].status, "removed");
        assert_eq!(
            cleanup_sensitive(&bundle).unwrap()[0].status,
            "already-absent"
        );
        assert!(bundle.join("capture.fcapng").exists());
    }

    #[test]
    fn version_one_share_copy_retains_a_readable_manifest() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        prepare_bundle(&bundle).unwrap();
        fs::write(bundle.join("capture.fcapng"), b"ordinary").unwrap();
        fs::write(bundle.join("tls-keylog.log"), b"secret").unwrap();
        let manifest = json!({
            "manifest_version":1,
            "artifacts":[
                {"path":"capture.fcapng","sensitivity":"ordinary"},
                {"path":"tls-keylog.log","sensitivity":"secret-adjacent"},
                {"path":"manifest.json","sensitivity":"ordinary"}
            ]
        });
        let source_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        fs::write(bundle.join("manifest.json"), &source_bytes).unwrap();

        let share = root.path().join("share");
        let sharing_path = export_share_copy(&bundle, &share).unwrap();

        assert_eq!(fs::read(share.join("manifest.json")).unwrap(), source_bytes);
        assert!(super::super::ManifestDocument::read(&share.join("manifest.json")).is_ok());
        assert!(!share.join("tls-keylog.log").exists());
        let sharing: Value = serde_json::from_slice(&fs::read(sharing_path).unwrap()).unwrap();
        assert!(sharing["included"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["path"] == "manifest.json"));
    }

    #[test]
    fn version_two_share_copy_rewrites_authority_without_mutating_source() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        prepare_bundle(&bundle).unwrap();
        fs::write(bundle.join("capture.fcapng"), b"ordinary").unwrap();
        fs::write(bundle.join("application.jsonl"), b"sensitive").unwrap();
        let artifact = |role: &str, path: &str, sensitivity: &str| {
            json!({
                "role":role,"path":path,
                "authority":{"kind":"primary-evidence","owner":role,"source_role":null},
                "sensitivity":sensitivity,"content_type":"application/octet-stream",
                "required":false,"finalization":"complete","completeness":"complete",
                "loss":{"state":"none"},"correlation":{"state":"not-applicable"}
            })
        };
        let source = json!({
            "$schema":super::super::MANIFEST_SCHEMA,"manifest_version":2,
            "product":{"name":"fragcap","version":"test"},"session_id":"share-test",
            "state":"complete","artifacts":[
                artifact("pcapng","capture.fcapng","ordinary"),
                artifact("application-jsonl","application.jsonl","sensitive"),
                {"role":"manifest","path":"manifest.json","authority":{"kind":"bundle-index","owner":"bundle-index","source_role":null},"sensitivity":"ordinary","content_type":"application/json","required":true,"finalization":"complete","completeness":"complete","loss":{"state":"none"},"correlation":{"state":"not-applicable"}}
            ],"omissions":[]
        });
        let source_bytes = serde_json::to_vec_pretty(&source).unwrap();
        fs::write(bundle.join("manifest.json"), &source_bytes).unwrap();

        let share = root.path().join("share");
        export_share_copy(&bundle, &share).unwrap();

        assert_eq!(
            fs::read(bundle.join("manifest.json")).unwrap(),
            source_bytes
        );
        assert!(share.join("capture.fcapng").is_file());
        assert!(!share.join("application.jsonl").exists());
        let shared = super::super::ManifestDocument::read(&share.join("manifest.json")).unwrap();
        assert_eq!(shared.value()["state"], "partial");
        let application = shared.value()["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|artifact| artifact["role"] == "application-jsonl")
            .unwrap();
        assert_eq!(application["completeness"], "omitted");
        assert!(application.get("path").is_none());
    }

    #[test]
    fn pending_delete_replays_and_malformed_or_unbounded_journals_fail_safe() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        prepare_bundle(&bundle).unwrap();
        fs::write(bundle.join("tls-keylog.log"), b"secret").unwrap();
        fs::write(bundle.join("manifest.json"), br#"{"artifacts":[]}"#).unwrap();
        append_journal(
            &bundle,
            "delete",
            Path::new("tls-keylog.log"),
            "intent",
            "pending",
        )
        .unwrap();
        let result = recover_sensitive_actions(&bundle).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, "removed");
        assert!(recover_sensitive_actions(&bundle).unwrap().is_empty());

        fs::write(bundle.join(JOURNAL), b"not-json\n").unwrap();
        assert_eq!(
            recover_sensitive_actions(&bundle).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        fs::write(bundle.join(JOURNAL), "{}\n".repeat(4097)).unwrap();
        assert_eq!(
            recover_sensitive_actions(&bundle).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn deletion_failure_is_reported_and_does_not_skip_other_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        prepare_bundle(&bundle).unwrap();
        fs::create_dir(bundle.join("cannot-remove-as-file")).unwrap();
        fs::write(bundle.join("tls-keylog.log"), b"secret").unwrap();
        fs::write(bundle.join("capture.fcapng"), b"ordinary").unwrap();
        fs::write(bundle.join("manifest.json"), br#"{"artifacts":[{"path":"cannot-remove-as-file","sensitivity":"sensitive"},{"path":"tls-keylog.log","sensitivity":"secret-adjacent"},{"path":"capture.fcapng","sensitivity":"ordinary"}]}"#).unwrap();

        let results = cleanup_sensitive(&bundle).unwrap();
        assert!(results.iter().any(|result| {
            result.path == Path::new("cannot-remove-as-file") && result.status == "failed"
        }));
        assert!(results.iter().any(|result| {
            result.path == Path::new("tls-keylog.log") && result.status == "removed"
        }));
        assert!(bundle.join("capture.fcapng").exists());
    }

    #[test]
    fn traversal_is_refused() {
        assert!(validate_relative(Path::new("../secret")).is_err());
        assert!(validate_relative(Path::new("/secret")).is_err());
    }

    #[test]
    fn linked_parent_escape_is_refused_before_cleanup_is_journaled() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        let outside = root.path().join("outside");
        prepare_bundle(&bundle).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("secret"), b"outside").unwrap();
        fs::write(
            bundle.join("manifest.json"),
            br#"{"artifacts":[{"path":"linked/secret","sensitivity":"sensitive"}]}"#,
        )
        .unwrap();
        if let Err(error) = symlink_directory(&outside, &bundle.join("linked")) {
            if error.kind() == io::ErrorKind::PermissionDenied {
                return;
            }
            panic!("could not create test directory link: {error}");
        }
        let journal_before = fs::read(bundle.join(JOURNAL)).unwrap();

        assert_eq!(
            cleanup_sensitive(&bundle).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(fs::read(bundle.join(JOURNAL)).unwrap(), journal_before);
        assert_eq!(fs::read(outside.join("secret")).unwrap(), b"outside");

        append_journal(
            &bundle,
            "delete",
            Path::new("linked/secret"),
            "intent",
            "pending",
        )
        .unwrap();
        assert_eq!(
            recover_sensitive_actions(&bundle).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(fs::read(outside.join("secret")).unwrap(), b"outside");
    }

    #[test]
    fn share_destination_inside_source_is_refused_without_mutation() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        prepare_bundle(&bundle).unwrap();
        fs::write(bundle.join("capture.fcapng"), b"ordinary").unwrap();
        fs::write(
            bundle.join("manifest.json"),
            br#"{"artifacts":[{"path":"capture.fcapng","sensitivity":"ordinary"}]}"#,
        )
        .unwrap();
        let journal_before = fs::read(bundle.join(JOURNAL)).unwrap();
        let destination = bundle.join("share");

        assert_eq!(
            export_share_copy(&bundle, &destination).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(!destination.exists());
        assert_eq!(fs::read(bundle.join(JOURNAL)).unwrap(), journal_before);
        assert_eq!(
            fs::read(bundle.join("capture.fcapng")).unwrap(),
            b"ordinary"
        );
    }

    #[test]
    fn linked_or_dangling_sensitive_files_and_journals_are_never_followed() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("bundle");
        let outside = root.path().join("outside.log");
        prepare_bundle(&bundle).unwrap();
        fs::write(&outside, b"outside").unwrap();
        fs::write(
            bundle.join("manifest.json"),
            br#"{"artifacts":[{"path":"linked.log","sensitivity":"sensitive"}]}"#,
        )
        .unwrap();
        if let Err(error) = symlink_file(&outside, &bundle.join("linked.log")) {
            if error.kind() == io::ErrorKind::PermissionDenied {
                return;
            }
            panic!("could not create test file link: {error}");
        }
        let journal_before = fs::read(bundle.join(JOURNAL)).unwrap();
        assert!(cleanup_sensitive(&bundle).is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"outside");
        assert_eq!(fs::read(bundle.join(JOURNAL)).unwrap(), journal_before);

        fs::remove_file(bundle.join("linked.log")).unwrap();
        symlink_file(&root.path().join("absent"), &bundle.join("linked.log")).unwrap();
        assert!(cleanup_sensitive(&bundle).is_err());
        assert_eq!(fs::read(bundle.join(JOURNAL)).unwrap(), journal_before);

        fs::remove_file(bundle.join(JOURNAL)).unwrap();
        symlink_file(&outside, &bundle.join(JOURNAL)).unwrap();
        assert!(prepare_bundle(&bundle).is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"outside");
    }

    #[test]
    fn cleanup_of_an_unknown_bundle_leaves_no_directory_behind() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing");
        assert!(cleanup_sensitive(&missing).is_err());
        assert!(!missing.exists());
    }

    #[cfg(unix)]
    fn symlink_directory(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(unix)]
    fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn symlink_directory(target: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_dir(target, link)
    }

    #[cfg(windows)]
    fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }

    #[cfg(windows)]
    #[test]
    fn windows_sensitive_file_has_a_protected_owner_system_dacl() {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;
        use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
        use windows_sys::Win32::Security::{GetFileSecurityW, DACL_SECURITY_INFORMATION};
        use windows_sys::Win32::System::Memory::LocalFree;

        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("secret.log");
        drop(open_sensitive_file(&path).unwrap());
        let wide: Vec<u16> = OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut needed = 0;
        unsafe {
            GetFileSecurityW(
                wide.as_ptr(),
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                0,
                &mut needed,
            )
        };
        let mut descriptor = vec![0_u8; needed as usize];
        assert_ne!(
            unsafe {
                GetFileSecurityW(
                    wide.as_ptr(),
                    DACL_SECURITY_INFORMATION,
                    descriptor.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            },
            0
        );
        let mut sddl = ptr::null_mut();
        let mut length = 0;
        assert_ne!(
            unsafe {
                ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    descriptor.as_mut_ptr().cast(),
                    1,
                    DACL_SECURITY_INFORMATION,
                    &mut sddl,
                    &mut length,
                )
            },
            0
        );
        let text = String::from_utf16(unsafe { std::slice::from_raw_parts(sddl, length as usize) })
            .unwrap();
        unsafe { LocalFree(sddl as isize) };
        assert!(text.starts_with("D:P"), "{text}");
        assert!(text.contains(";;;OW)") && text.contains(";;;SY)"), "{text}");
        assert!(
            !text.contains(";;;WD)") && !text.contains(";;;BU)"),
            "{text}"
        );
    }
}
