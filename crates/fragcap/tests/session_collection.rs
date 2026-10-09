// SPDX-License-Identifier: Apache-2.0

#![cfg(all(windows, feature = "deep-capture"))]

use std::fs;
use std::path::{Path, PathBuf};

use fragcap::deep_capture::{
    collect_session_contents, collected_session_container, inspect_session_collection,
    prepare_bundle, purge_session_container, ResourceJournal, ResourceKind, ResourceState,
    ResourceTransition,
};
use serde_json::json;

fn fixture(parent: &Path) -> (PathBuf, PathBuf) {
    let bundle = parent.join("session");
    let store = parent.join(".session-retirements");
    prepare_bundle(&bundle).unwrap();
    let mut journal = ResourceJournal::create(&bundle, "collection-fixture", "plan").unwrap();
    journal.finish().unwrap();
    drop(journal);
    fs::write(
        bundle.join("capture.fcapng"),
        b"fixture-only-packet-bytes\n",
    )
    .unwrap();
    fs::write(
        bundle.join("manifest.json"),
        serde_json::to_vec(&json!({
            "manifest_version":1,"session_id":"collection-fixture","state":"complete",
            "artifacts":[{"path":"capture.fcapng","sensitivity":"ordinary"}]
        }))
        .unwrap(),
    )
    .unwrap();
    (bundle, store)
}

#[test]
fn preview_is_read_only_and_collection_preserves_a_genuinely_empty_root() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    assert!(!store.exists());
    assert!(preview.owned && !preview.empty && preview.bytes > 0);
    let result = collect_session_contents(&bundle, &store, &preview.id).unwrap();
    assert!(result.complete && result.preserved_container && !result.purged);
    assert_eq!(result.removed_bytes, preview.bytes);
    assert_eq!(result.removed_files, preview.files);
    assert_eq!(fs::read_dir(&bundle).unwrap().count(), 0);
    assert!(collected_session_container(&bundle, &store).unwrap());
    let empty = inspect_session_collection(&bundle, &store).unwrap();
    assert!(empty.empty && empty.owned);
    let repeated = collect_session_contents(&bundle, &store, &empty.id).unwrap();
    assert!(repeated.complete);
    assert_eq!(repeated.removed_bytes, 0);
    assert!(bundle.exists());
}

#[test]
fn purge_requires_separate_empty_container_authorization() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let populated = inspect_session_collection(&bundle, &store).unwrap();
    assert!(purge_session_container(&bundle, &store, &populated.id).is_err());
    collect_session_contents(&bundle, &store, &populated.id).unwrap();
    let empty = inspect_session_collection(&bundle, &store).unwrap();
    let result = purge_session_container(&bundle, &store, &empty.id).unwrap();
    assert!(result.complete && result.purged && !result.preserved_container);
    assert!(!bundle.exists());
    let retry = inspect_session_collection(&bundle, &store).unwrap();
    assert!(
        purge_session_container(&bundle, &store, &retry.id)
            .unwrap()
            .complete
    );
}

#[test]
fn unknown_files_and_hard_link_aliases_refuse_before_transaction_or_deletion() {
    for alias in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        if alias {
            fs::hard_link(bundle.join("capture.fcapng"), temp.path().join("outside")).unwrap();
        } else {
            fs::write(bundle.join("user-owned.txt"), b"preserve").unwrap();
        }
        assert!(inspect_session_collection(&bundle, &store).is_err());
        assert!(!store.exists());
        assert!(bundle.join("capture.fcapng").exists());
    }
}

#[test]
fn changed_proposal_and_forged_trailer_cannot_authorize_removal() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    fs::write(bundle.join("capture.fcapng"), b"changed fixture content").unwrap();
    assert!(collect_session_contents(&bundle, &store, &preview.id).is_err());
    assert!(!store.exists());
    let journal = bundle.join("resource-journal.jsonl");
    let source = fs::read_to_string(&journal).unwrap();
    let broken = source.lines().map(|line| {
        if line.contains("resource-journal.trailer") {
            json!({"type":"resource-journal.trailer","schema_version":1,"session_id":"other","records":0}).to_string()
        } else {line.to_string()}
    }).collect::<Vec<_>>().join("\n") + "\n";
    fs::write(&journal, broken).unwrap();
    assert!(inspect_session_collection(&bundle, &store).is_err());
    assert!(bundle.join("capture.fcapng").exists());
}

#[test]
fn arbitrary_empty_directory_has_no_purge_ownership() {
    let temp = tempfile::tempdir().unwrap();
    let bundle = temp.path().join("unowned");
    fs::create_dir(&bundle).unwrap();
    let store = temp.path().join(".session-retirements");
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    assert!(preview.empty && !preview.owned);
    assert!(purge_session_container(&bundle, &store, &preview.id).is_err());
    assert!(bundle.exists());
}

#[test]
fn readonly_journal_failure_resumes_after_manifest_and_policy_have_been_removed() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let policy = json!({"schema_version":1,"storage_class":"managed-history","created_at_unix_ms":1,"max_age_seconds":2592000,"max_sessions":20,"max_bytes":2147483648u64,"preserve_empty_containers":true});
    fs::write(
        bundle.join(".session-retention.json"),
        serde_json::to_vec(&policy).unwrap(),
    )
    .unwrap();
    let journal = bundle.join("resource-journal.jsonl");
    let mut permissions = fs::metadata(&journal).unwrap().permissions();
    let original_permissions = permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&journal, permissions).unwrap();
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    let result = collect_session_contents(&bundle, &store, &preview.id).unwrap();
    assert!(!result.complete);
    assert!(result.removed_bytes > 0 && result.removed_bytes < preview.bytes);
    assert!(!bundle.join("manifest.json").exists());
    assert!(!bundle.join(".session-retention.json").exists());
    let resumed = inspect_session_collection(&bundle, &store).unwrap();
    assert!(resumed.resumed);
    assert_eq!(resumed.retention, Some(policy));
    fs::set_permissions(&journal, original_permissions).unwrap();
    let result = collect_session_contents(&bundle, &store, &resumed.id).unwrap();
    assert!(result.complete);
    assert_eq!(result.removed_bytes, resumed.bytes);
    assert!(collected_session_container(&bundle, &store).unwrap());
}

#[test]
fn failure_accounts_for_every_not_attempted_object() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let capture = bundle.join("capture.fcapng");
    let mut permissions = fs::metadata(&capture).unwrap().permissions();
    let original_permissions = permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&capture, permissions).unwrap();
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    let result = collect_session_contents(&bundle, &store, &preview.id).unwrap();
    assert!(!result.complete);
    assert_eq!(result.paths.len(), preview.files);
    assert_eq!(
        result.paths.iter().filter(|p| p.status == "failed").count(),
        1
    );
    assert!(result.paths.iter().any(|p| p.status == "not-attempted"));
    assert!(bundle.join("manifest.json").exists());
    fs::set_permissions(&capture, original_permissions).unwrap();
}

#[test]
fn nested_manifest_files_are_removed_before_exact_child_directory() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    fs::create_dir(bundle.join("nested")).unwrap();
    fs::write(bundle.join("nested/evidence.bin"), b"nested-fixture").unwrap();
    let path = bundle.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest["artifacts"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"nested/evidence.bin","sensitivity":"ordinary"}));
    fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    let result = collect_session_contents(&bundle, &store, &preview.id).unwrap();
    assert!(result.complete, "{result:?}");
    assert_eq!(fs::read_dir(&bundle).unwrap().count(), 0);
}

#[test]
fn failed_closed_artifact_production_is_collectible_but_retained_trust_is_not() {
    for kind in [ResourceKind::Artifact, ResourceKind::Trust] {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let mut journal = ResourceJournal::resume(&bundle.join("resource-journal.jsonl")).unwrap();
        journal
            .append(ResourceTransition::new(
                "fixture-resource",
                kind,
                "synthetic-only",
                "session:collection-fixture",
                "fixture-action",
                ResourceState::Pending,
                "declared",
            ))
            .unwrap();
        let terminal = if kind == ResourceKind::Artifact {
            ResourceState::Failed
        } else {
            ResourceState::Retained
        };
        journal
            .append(ResourceTransition::new(
                "fixture-resource",
                kind,
                "synthetic-only",
                "session:collection-fixture",
                "fixture-action",
                terminal,
                "closed fixture",
            ))
            .unwrap();
        journal.finish().unwrap();
        drop(journal);
        let manifest_path = bundle.join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["state"] = json!("failed");
        fs::write(manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        if kind == ResourceKind::Artifact {
            let preview = inspect_session_collection(&bundle, &store).unwrap();
            assert!(
                collect_session_contents(&bundle, &store, &preview.id)
                    .unwrap()
                    .complete
            );
        } else {
            assert!(inspect_session_collection(&bundle, &store).is_err());
            assert!(!store.exists());
        }
    }
}

#[test]
fn pending_sensitive_cleanup_requires_its_own_reconciliation() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    fs::write(bundle.join(".sensitive-actions.jsonl"),b"{\"version\":1,\"type\":\"header\"}\n{\"version\":1,\"op\":\"delete\",\"path\":\"tls-keylog.log\",\"phase\":\"intent\",\"status\":\"pending\"}\n").unwrap();
    assert!(inspect_session_collection(&bundle, &store).is_err());
    assert!(bundle.join("capture.fcapng").exists());
    assert!(!store.exists());
}

#[test]
fn an_open_writer_with_delete_sharing_refuses_before_any_collection_effect() {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let writer = fs::OpenOptions::new()
        .write(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(bundle.join("capture.fcapng"))
        .unwrap();
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    assert!(collect_session_contents(&bundle, &store, &preview.id).is_err());
    assert!(!store.exists());
    assert!(bundle.join("capture.fcapng").exists());
    drop(writer);
    assert!(
        collect_session_contents(&bundle, &store, &preview.id)
            .unwrap()
            .complete
    );
}

#[test]
fn retention_promises_cannot_be_silently_changed_by_a_conflicting_sidecar() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let path = bundle.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest["sensitive_artifacts"] = json!({"retention":"retain-until-explicit-cleanup"});
    fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    fs::write(bundle.join(".session-retention.json"),serde_json::to_vec(&json!({"schema_version":1,"storage_class":"managed-history","created_at_unix_ms":1,"max_age_seconds":2592000,"max_sessions":20,"max_bytes":2147483648u64,"preserve_empty_containers":true})).unwrap()).unwrap();
    assert!(inspect_session_collection(&bundle, &store).is_err());
    assert!(!store.exists());
    assert!(bundle.join("capture.fcapng").exists());
}

#[test]
fn retirement_identity_never_authorizes_a_replacement_container() {
    let temp = tempfile::tempdir().unwrap();
    let (bundle, store) = fixture(temp.path());
    let preview = inspect_session_collection(&bundle, &store).unwrap();
    collect_session_contents(&bundle, &store, &preview.id).unwrap();
    fs::rename(&bundle, temp.path().join("original-preserved-container")).unwrap();
    fs::create_dir(&bundle).unwrap();
    fs::write(bundle.join("replacement.txt"), b"preserve replacement").unwrap();
    assert!(inspect_session_collection(&bundle, &store).is_err());
    assert!(purge_session_container(&bundle, &store, &preview.id).is_err());
    assert_eq!(
        fs::read(bundle.join("replacement.txt")).unwrap(),
        b"preserve replacement"
    );
}

fn junction(link: &Path, target: &Path) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::IO::DeviceIoControl;
    fs::create_dir(link).unwrap();
    let target = target
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .trim_start_matches("\\\\?\\")
        .to_string();
    let substitute = format!("\\??\\{target}").encode_utf16().collect::<Vec<_>>();
    let print = target.encode_utf16().collect::<Vec<_>>();
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&0xa0000003u32.to_le_bytes());
    buffer.extend_from_slice(&(8 + (substitute.len() + print.len() + 2) * 2).to_le_bytes()[..2]);
    buffer.extend_from_slice(&0u16.to_le_bytes());
    buffer.extend_from_slice(&0u16.to_le_bytes());
    buffer.extend_from_slice(&((substitute.len() * 2) as u16).to_le_bytes());
    buffer.extend_from_slice(&(((substitute.len() + 1) * 2) as u16).to_le_bytes());
    buffer.extend_from_slice(&((print.len() * 2) as u16).to_le_bytes());
    for word in substitute
        .into_iter()
        .chain(Some(0))
        .chain(print)
        .chain(Some(0))
    {
        buffer.extend_from_slice(&word.to_le_bytes());
    }
    let name = link
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            0x40000000,
            7,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            0,
        )
    };
    assert_ne!(
        handle,
        INVALID_HANDLE_VALUE,
        "{}",
        std::io::Error::last_os_error()
    );
    let mut returned = 0;
    let result = unsafe {
        DeviceIoControl(
            handle,
            0x000900a4,
            buffer.as_ptr() as *const _,
            buffer.len() as u32,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    let error = std::io::Error::last_os_error();
    unsafe { CloseHandle(handle) };
    assert_ne!(result, 0, "{error}");
}

#[test]
fn junction_children_ancestors_and_retirement_stores_never_reach_unrelated_data() {
    for location in ["child", "ancestor", "store"] {
        let temp = tempfile::tempdir().unwrap();
        let (bundle, store) = fixture(temp.path());
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let preserved = outside.join("preserve.txt");
        fs::write(&preserved, b"outside fixture").unwrap();
        let selected = match location {
            "child" => {
                junction(&bundle.join("linked"), &outside);
                bundle.clone()
            }
            "ancestor" => {
                let link = temp.path().join("linked-root");
                junction(&link, temp.path());
                link.join("session")
            }
            "store" => {
                junction(&store, &outside);
                bundle.clone()
            }
            _ => unreachable!(),
        };
        assert!(
            inspect_session_collection(&selected, &store).is_err(),
            "{location}"
        );
        assert_eq!(fs::read(&preserved).unwrap(), b"outside fixture");
        assert!(bundle.join("capture.fcapng").exists());
    }
}
