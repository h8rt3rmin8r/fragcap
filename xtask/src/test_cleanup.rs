// SPDX-License-Identifier: Apache-2.0

//! Exact run-owned temporary output for repository test commands.

use std::fs;
use std::path::{Path, PathBuf};

const MARKER: &str = "test-owner.json";
const MAX_RESIDUAL_ENTRIES: usize = 64;

struct RunRoot {
    owner: tempfile::TempDir,
    canonical: PathBuf,
}

impl RunRoot {
    fn new() -> Result<Self, String> {
        Self::new_in(&std::env::temp_dir())
    }

    fn new_in(parent: &Path) -> Result<Self, String> {
        let parent = fs::canonicalize(parent)
            .map_err(|error| format!("canonical temp parent {}: {error}", parent.display()))?;
        let owner = tempfile::Builder::new()
            .prefix("fragcap-test-run-")
            .tempdir_in(parent)
            .map_err(|error| format!("create unique test root: {error}"))?;
        let canonical = owner
            .path()
            .canonicalize()
            .map_err(|error| format!("canonical test root: {error}"))?;
        let marker = serde_json::json!({
            "schema": 1,
            "canonical_root": canonical,
            "owner_pid": std::process::id(),
            "run_token": canonical.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
        });
        fs::write(owner.path().join(MARKER), format!("{marker}\n"))
            .map_err(|error| format!("write exact test owner marker: {error}"))?;
        Ok(Self { owner, canonical })
    }

    fn path(&self) -> &Path {
        &self.canonical
    }

    fn finish(self) -> Result<(), String> {
        self.finish_with(tempfile::TempDir::close)
    }

    fn finish_with(
        self,
        close: impl FnOnce(tempfile::TempDir) -> std::io::Result<()>,
    ) -> Result<(), String> {
        let path = self.canonical.clone();
        let inspection = inspect_root(&path);
        match inspection {
            Ok(entries) if entries.is_empty() => {
                close(self.owner).map_err(|error| {
                    format!("test cleanup failed at {}: {error}", path.display())
                })?;
                if path.exists() {
                    return Err(format!(
                        "test cleanup failed at {}: root remained after close",
                        path.display()
                    ));
                }
                Ok(())
            }
            Ok(entries) => {
                let _ = self.owner.keep();
                Err(format!(
                    "test output remained under {}: {}",
                    path.display(),
                    entries
                        .iter()
                        .map(|entry| entry.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
            Err(error) => {
                let _ = self.owner.keep();
                Err(format!("test root {} preserved: {error}", path.display()))
            }
        }
    }
}

fn inspect_root(path: &Path) -> Result<Vec<PathBuf>, String> {
    if !path.is_absolute() {
        return Err("test root must be an absolute path".into());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("canonical test root {}: {error}", path.display()))?;
    if canonical != path {
        return Err(format!("test root path changed: {}", path.display()));
    }
    let marker_path = path.join(MARKER);
    let metadata = fs::symlink_metadata(&marker_path)
        .map_err(|error| format!("owner marker {}: {error}", marker_path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "owner marker is not a regular file: {}",
            marker_path.display()
        ));
    }
    let marker: serde_json::Value = serde_json::from_slice(
        &fs::read(&marker_path)
            .map_err(|error| format!("read owner marker {}: {error}", marker_path.display()))?,
    )
    .map_err(|error| format!("parse owner marker {}: {error}", marker_path.display()))?;
    let expected_token = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if marker["schema"] != 1
        || marker["canonical_root"].as_str() != Some(path.to_string_lossy().as_ref())
        || marker["run_token"].as_str() != Some(expected_token)
        || marker["owner_pid"].as_u64().is_none()
    {
        return Err(format!(
            "owner marker does not bind exact root: {}",
            path.display()
        ));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)
        .map_err(|error| format!("list exact test root {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("read test root entry: {error}"))?;
        if entry.file_name() != MARKER {
            if entries.len() == MAX_RESIDUAL_ENTRIES {
                return Err(format!(
                    "test residue inventory exceeded {MAX_RESIDUAL_ENTRIES} entries under {}",
                    path.display()
                ));
            }
            entries.push(entry.path());
        }
    }
    entries.sort();
    Ok(entries)
}

pub fn run(args: &[String]) -> u8 {
    let root = match RunRoot::new() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("test-clean: {error}");
            return 2;
        }
    };
    eprintln!(
        "test-clean: effective canonical temp root: {}",
        root.path().display()
    );
    let root_path = root.path().to_path_buf();
    let mut child = super::hidden_command(env!("CARGO"));
    child
        .current_dir(super::repo_root())
        .arg("test")
        .args(args)
        .env("TMP", &root_path)
        .env("TEMP", &root_path)
        .env("TMPDIR", &root_path)
        .env("FRAGCAP_TEST_RUN_ROOT", &root_path);
    let result = child.status();
    let cleanup = root.finish();
    if let Err(error) = cleanup {
        eprintln!("test-clean: {error}");
        return 1;
    }
    match result {
        Ok(status) if status.success() => 0,
        Ok(status) => {
            eprintln!("test-clean: cargo test failed with {status}");
            1
        }
        Err(error) => {
            eprintln!("test-clean: could not start cargo test: {error}");
            2
        }
    }
}

pub fn inspect(arguments: &[String]) -> u8 {
    if arguments.len() != 1 {
        eprintln!("test-clean-inspect requires one exact absolute run root");
        return 2;
    }
    let path = Path::new(&arguments[0]);
    match inspect_root(path) {
        Ok(entries) => {
            eprintln!("test-clean-inspect: exact root: {}", path.display());
            if entries.is_empty() {
                eprintln!("test-clean-inspect: no child residue observed");
                0
            } else {
                for entry in entries {
                    eprintln!("test-clean-inspect: residual: {}", entry.display());
                }
                eprintln!("test-clean-inspect: ownership inactivity and native recovery obligations still require review; no deletion performed");
                1
            }
        }
        Err(error) => {
            eprintln!("test-clean-inspect: {error}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_run_root_closes_and_verifies_absence() {
        let parent = tempfile::tempdir().unwrap();
        let root = RunRoot::new_in(parent.path()).unwrap();
        let path = root.path().to_path_buf();
        assert!(inspect_root(&path).unwrap().is_empty());
        root.finish().unwrap();
        assert!(!path.exists());
        parent.close().unwrap();
    }

    #[test]
    fn residue_is_reported_and_preserved_without_touching_a_sibling() {
        let parent = tempfile::tempdir().unwrap();
        let sibling = parent.path().join("fragcap-test-run-unrelated");
        fs::create_dir(&sibling).unwrap();
        let root = RunRoot::new_in(parent.path()).unwrap();
        let path = root.path().to_path_buf();
        fs::write(path.join("leak.txt"), b"controlled").unwrap();
        let error = root.finish().unwrap_err();
        assert!(error.contains("leak.txt"), "{error}");
        assert!(path.exists());
        assert!(sibling.exists());
        assert!(inspect_root(&sibling).is_err());
        fs::remove_dir_all(path).unwrap();
        parent.close().unwrap();
    }

    #[test]
    fn close_failure_names_and_preserves_exact_root() {
        let parent = tempfile::tempdir().unwrap();
        let root = RunRoot::new_in(parent.path()).unwrap();
        let path = root.path().to_path_buf();
        let error = root
            .finish_with(|owner| {
                let _ = owner.keep();
                Err(std::io::Error::other("injected close failure"))
            })
            .unwrap_err();
        assert!(error.contains("injected close failure"), "{error}");
        assert!(error.contains(&path.display().to_string()), "{error}");
        assert!(path.exists());
        fs::remove_dir_all(path).unwrap();
        parent.close().unwrap();
    }

    #[test]
    fn interrupted_owner_marker_is_inspectable_without_deletion() {
        let parent = tempfile::tempdir().unwrap();
        let root = RunRoot::new_in(parent.path()).unwrap();
        let path = root.path().to_path_buf();
        let _ = root.owner.keep();
        assert!(inspect_root(&path).unwrap().is_empty());
        assert!(path.join(MARKER).exists());
        fs::remove_dir_all(path).unwrap();
        parent.close().unwrap();
    }

    #[test]
    fn changed_marker_refuses_cleanup_of_exact_root() {
        let parent = tempfile::tempdir().unwrap();
        let root = RunRoot::new_in(parent.path()).unwrap();
        let path = root.path().to_path_buf();
        fs::write(
            path.join(MARKER),
            b"{\"schema\":1,\"canonical_root\":\"other\"}",
        )
        .unwrap();
        let error = root.finish().unwrap_err();
        assert!(error.contains("owner marker does not bind"), "{error}");
        assert!(path.exists());
        fs::remove_dir_all(path).unwrap();
        parent.close().unwrap();
    }

    #[test]
    fn concurrent_roots_are_distinct_and_cannot_clean_each_other() {
        let parent = tempfile::tempdir().unwrap();
        let first = RunRoot::new_in(parent.path()).unwrap();
        let second = RunRoot::new_in(parent.path()).unwrap();
        let second_path = second.path().to_path_buf();
        assert_ne!(first.path(), second.path());
        first.finish().unwrap();
        assert!(second_path.exists());
        second.finish().unwrap();
        assert!(!second_path.exists());
        parent.close().unwrap();
    }

    #[test]
    fn native_shaped_residue_is_preserved_for_review() {
        let parent = tempfile::tempdir().unwrap();
        let root = RunRoot::new_in(parent.path()).unwrap();
        let path = root.path().to_path_buf();
        fs::create_dir(path.join("native-session")).unwrap();
        fs::write(path.join("native-session/manifest.json"), b"{}").unwrap();
        assert!(root.finish().is_err());
        assert!(path.join("native-session/manifest.json").exists());
        fs::remove_dir_all(path).unwrap();
        parent.close().unwrap();
    }
}
