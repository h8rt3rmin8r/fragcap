// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
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
    fs::create_dir_all(path)?;
    protect_path(path, true)?;
    let journal = path.join(JOURNAL);
    if !journal.exists() {
        let mut file = open_sensitive_file(&journal)?;
        file.write_all(b"{\"version\":1,\"type\":\"header\"}\n")?;
        file.sync_all()?;
    }
    Ok(())
}

pub fn open_sensitive_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
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
        append_journal(bundle, "delete", &relative, "intent", "pending")?;
        let full = contained(bundle, &relative)?;
        let (status, reason) = if !full.exists() {
            ("already-absent", "sensitive artifact was already absent")
        } else if full.symlink_metadata()?.file_type().is_symlink() {
            (
                "failed",
                "symbolic links are not eligible for sensitive cleanup",
            )
        } else {
            match fs::remove_file(&full) {
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
            }
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
    let journal = bundle.join(JOURNAL);
    if !journal.exists() {
        return Ok(Vec::new());
    }
    if journal.metadata()?.len() > MAX_JOURNAL_BYTES {
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
        let (status, reason) = if !full.exists() {
            (
                "already-absent",
                "pending sensitive artifact was already absent".into(),
            )
        } else if full.symlink_metadata()?.file_type().is_symlink() {
            (
                "failed",
                "symbolic links are not eligible for sensitive recovery".into(),
            )
        } else {
            match fs::remove_file(&full) {
                Ok(()) => ("removed", "recovered pending sensitive cleanup".into()),
                Err(error) => ("failed", error.to_string()),
            }
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
    if destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "share destination already exists",
        ));
    }
    let source = source.canonicalize()?;
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
    if staging.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "share staging directory already exists",
        ));
    }
    prepare_bundle(&staging)?;
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
            let relative = PathBuf::from(path);
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
        fs::rename(&staging, destination)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;
    Ok(destination.join("sharing-manifest.json"))
}

fn read_manifest(bundle: &Path) -> io::Result<Value> {
    serde_json::from_slice(&fs::read(bundle.join("manifest.json"))?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
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
            let path = PathBuf::from(path);
            validate_relative(&path)?;
            paths.insert(path);
        }
    }
    Ok(paths)
}

fn contained(root: &Path, relative: &Path) -> io::Result<PathBuf> {
    validate_relative(relative)?;
    Ok(root.join(relative))
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
    let journal = bundle.join(JOURNAL);
    let current_bytes = journal.metadata().map(|m| m.len()).unwrap_or(0);
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
        let sharing: Value = serde_json::from_slice(&fs::read(sharing_manifest).unwrap()).unwrap();
        assert_eq!(sharing["status"], "complete");
        assert_eq!(sharing["included"].as_array().unwrap().len(), 1);
        assert_eq!(sharing["omitted"].as_array().unwrap().len(), 1);
        assert_eq!(cleanup_sensitive(&bundle).unwrap()[0].status, "removed");
        assert_eq!(
            cleanup_sensitive(&bundle).unwrap()[0].status,
            "already-absent"
        );
        assert!(bundle.join("capture.fcapng").exists());
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
    fn cleanup_of_an_unknown_bundle_leaves_no_directory_behind() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing");
        assert!(cleanup_sensitive(&missing).is_err());
        assert!(!missing.exists());
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
