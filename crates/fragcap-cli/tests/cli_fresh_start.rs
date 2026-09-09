// SPDX-License-Identifier: Apache-2.0

mod common;

use std::fs;
use std::sync::{Mutex, MutexGuard};

use common::run;

static CURRENT_USER_ENVIRONMENT: Mutex<()> = Mutex::new(());

struct CurrentUserEnvironment<'a> {
    _lock: MutexGuard<'a, ()>,
    appdata: Option<std::ffi::OsString>,
    local_appdata: Option<std::ffi::OsString>,
}

impl<'a> CurrentUserEnvironment<'a> {
    fn set(temp: &tempfile::TempDir) -> Self {
        let lock = CURRENT_USER_ENVIRONMENT.lock().unwrap();
        let appdata = std::env::var_os("APPDATA");
        let local_appdata = std::env::var_os("LOCALAPPDATA");
        let base = temp.path().join("User").join("AppData");
        std::env::set_var("APPDATA", base.join("Roaming"));
        std::env::set_var("LOCALAPPDATA", base.join("Local"));
        Self {
            _lock: lock,
            appdata,
            local_appdata,
        }
    }
}

impl Drop for CurrentUserEnvironment<'_> {
    fn drop(&mut self) {
        match self.appdata.take() {
            Some(value) => std::env::set_var("APPDATA", value),
            None => std::env::remove_var("APPDATA"),
        }
        match self.local_appdata.take() {
            Some(value) => std::env::set_var("LOCALAPPDATA", value),
            None => std::env::remove_var("LOCALAPPDATA"),
        }
    }
}

fn roots(temp: &tempfile::TempDir) -> (String, String) {
    let appdata = temp.path().join("User").join("AppData");
    let roaming = appdata.join("Roaming").join("fragcap");
    let local = appdata.join("Local").join("fragcap");
    fs::create_dir_all(roaming.join("profiles")).unwrap();
    fs::create_dir_all(local.join("logs")).unwrap();
    fs::write(roaming.join("local.db"), b"learned target").unwrap();
    fs::write(roaming.join("profiles").join("game.toml"), b"profile").unwrap();
    fs::write(local.join("logs").join("fragcap.log"), b"log").unwrap();
    (
        roaming.to_string_lossy().into_owned(),
        local.to_string_lossy().into_owned(),
    )
}

#[test]
fn preview_is_read_only_and_names_exact_roots_and_categories() {
    let temp = tempfile::tempdir().unwrap();
    let _environment = CurrentUserEnvironment::set(&temp);
    let (roaming, local) = roots(&temp);

    let (code, out, err) = run(&["--json", "fresh-start", "--preview"]);

    assert_eq!(code, 0, "preview failed: {err}");
    let value: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(value["type"], "fresh-start.inventory");
    assert_eq!(value["scope"], "current-user");
    assert!(value["inventory_id"]
        .as_str()
        .unwrap()
        .starts_with("fresh-start-v1:"));
    let rendered = value.to_string();
    assert!(rendered.contains("local-database"));
    assert!(rendered.contains("profiles"));
    assert!(rendered.contains("logs"));
    assert!(PathLike::new(&roaming).exists());
    assert!(PathLike::new(&local).exists());
}

#[test]
fn execution_requires_exact_confirmation_and_preserves_on_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    let _environment = CurrentUserEnvironment::set(&temp);
    let (roaming, local) = roots(&temp);

    let (code, _out, err) = run(&[
        "fresh-start",
        "--yes",
        "--confirm",
        "fresh-start-v1:0000000000000000000000000000000000000000000000000000000000000000",
    ]);

    assert_eq!(code, 1, "changed inventory is an expected refusal: {err}");
    assert!(PathLike::new(&roaming).join("local.db").exists());
    assert!(PathLike::new(&local).join("logs").exists());
}

#[test]
fn direct_command_cannot_replace_current_user_roots() {
    let temp = tempfile::tempdir().unwrap();
    let (roaming, local) = roots(&temp);

    let (code, _out, err) = run(&[
        "fresh-start",
        "--preview",
        "--roaming-root",
        &roaming,
        "--local-root",
        &local,
    ]);

    assert_eq!(code, 2);
    assert!(err.contains("reserved for the installer adapter"));
    assert!(PathLike::new(&roaming).exists());
    assert!(PathLike::new(&local).exists());
}

#[test]
fn exact_preview_identifier_authorizes_direct_current_user_cleanup() {
    let temp = tempfile::tempdir().unwrap();
    let _environment = CurrentUserEnvironment::set(&temp);
    let (roaming, local) = roots(&temp);
    let (preview_code, preview, preview_err) = run(&["--json", "fresh-start", "--preview"]);
    assert_eq!(preview_code, 0, "preview failed: {preview_err}");
    let value: serde_json::Value = serde_json::from_str(preview.trim()).unwrap();
    let identifier = value["inventory_id"].as_str().unwrap();

    let (code, out, err) = run(&["--json", "fresh-start", "--yes", "--confirm", identifier]);

    assert_eq!(code, 0, "cleanup failed: {err}\n{out}");
    assert!(!PathLike::new(&roaming).exists());
    assert!(!PathLike::new(&local).exists());
}

#[test]
fn json_cleanup_keeps_recovery_progress_out_of_structured_stdout() {
    use fragcap::deep_capture::{ResourceJournal, ResourceKind, ResourceState, ResourceTransition};

    let temp = tempfile::tempdir().unwrap();
    let _environment = CurrentUserEnvironment::set(&temp);
    let (roaming, _local) = roots(&temp);
    let bundle = PathLike::new(&roaming).join("sessions").join("refused");
    fs::create_dir_all(&bundle).unwrap();
    let mut journal = ResourceJournal::create(&bundle, "session", "plan").unwrap();
    journal
        .append(ResourceTransition::new(
            "artifact",
            ResourceKind::Artifact,
            "session/session-output",
            "",
            "remove-artifact",
            ResourceState::Applied,
            "retained",
        ))
        .unwrap();
    drop(journal);
    let (preview_code, preview, preview_err) = run(&["--json", "fresh-start", "--preview"]);
    assert_eq!(preview_code, 0, "preview failed: {preview_err}");
    let preview: serde_json::Value = serde_json::from_str(preview.trim()).unwrap();

    let (code, out, _err) = run(&[
        "--json",
        "fresh-start",
        "--yes",
        "--confirm",
        preview["inventory_id"].as_str().unwrap(),
    ]);

    assert_eq!(code, 1);
    let result: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(result["type"], "fresh-start.cleanup");
    assert_eq!(result["status"], "partial");
}

#[test]
fn installer_adapter_requires_preview_confirmation_and_removes_only_current_user_roots() {
    let temp = tempfile::tempdir().unwrap();
    let _environment = CurrentUserEnvironment::set(&temp);
    let (roaming, local) = roots(&temp);
    let excluded = temp.path().join("exported.fcapng");
    let report = temp.path().join("reports").join("fresh-start.json");
    fs::write(&excluded, b"outside evidence").unwrap();

    let (preview_code, preview, preview_err) = run(&[
        "--json",
        "fresh-start",
        "--preview",
        "--installer-adapter",
        "--roaming-root",
        &roaming,
        "--local-root",
        &local,
    ]);
    assert_eq!(preview_code, 0, "preview failed: {preview_err}");
    let preview: serde_json::Value = serde_json::from_str(preview.trim()).unwrap();
    let identifier = preview["inventory_id"].as_str().unwrap();

    let (code, out, err) = run(&[
        "--json",
        "fresh-start",
        "--yes",
        "--confirm",
        identifier,
        "--installer-adapter",
        "--roaming-root",
        &roaming,
        "--local-root",
        &local,
        "--report",
        &report.to_string_lossy(),
    ]);

    assert_eq!(code, 0, "cleanup failed: {err}\n{out}");
    assert!(!PathLike::new(&roaming).exists());
    assert!(!PathLike::new(&local).exists());
    assert_eq!(fs::read(excluded).unwrap(), b"outside evidence");
    let report_value: serde_json::Value =
        serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
    assert_eq!(report_value["status"], "complete");
    assert_eq!(report_value["scope"], "current-user");
}

#[test]
fn installer_adapter_cannot_select_all_users() {
    let temp = tempfile::tempdir().unwrap();
    let (roaming, local) = roots(&temp);
    let report = temp.path().join("report.json");

    let (code, _out, err) = run(&[
        "fresh-start",
        "--scope",
        "all-users",
        "--yes",
        "--installer-adapter",
        "--roaming-root",
        &roaming,
        "--local-root",
        &local,
        "--report",
        &report.to_string_lossy(),
    ]);

    assert_eq!(code, 2);
    assert!(err.contains("installer adapter requires current-user scope"));
    assert!(PathLike::new(&roaming).exists());
    assert!(PathLike::new(&local).exists());
}

#[test]
fn installer_adapter_rejects_roots_for_another_profile() {
    let current = tempfile::tempdir().unwrap();
    let _environment = CurrentUserEnvironment::set(&current);
    let other = tempfile::tempdir().unwrap();
    let (roaming, local) = roots(&other);

    let (code, _out, err) = run(&[
        "fresh-start",
        "--preview",
        "--installer-adapter",
        "--roaming-root",
        &roaming,
        "--local-root",
        &local,
    ]);

    assert_eq!(code, 1);
    assert!(err.contains("do not match the initiating user's canonical data roots"));
    assert!(PathLike::new(&roaming).exists());
    assert!(PathLike::new(&local).exists());
}

type PathLike = std::path::Path;
