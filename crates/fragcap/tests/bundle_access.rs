// SPDX-License-Identifier: Apache-2.0

#![cfg(all(windows, feature = "deep-capture"))]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use fragcap::deep_capture::{
    open_sensitive_file, prepare_bundle_for_recipient, publish_final, verify_bundle_access,
    write_crash_prefix, ApplicationArtifactLease, LifecycleWriter, OutputRecipient,
    ResourceJournal,
};
use serde_json::json;

const SESSION: &str = "s170-synthetic-access";

fn contents(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            assert!(path.is_file(), "fixture contains a non-file: {path:?}");
            let name = path.file_name().unwrap().into();
            (name, fs::read(path).unwrap())
        })
        .collect()
}

fn write_bundle(root: &Path, recipient: &OutputRecipient) {
    prepare_bundle_for_recipient(root, recipient).unwrap();
    write_crash_prefix(root, SESSION).unwrap();
    let mut journal = ResourceJournal::create(root, SESSION, "synthetic-plan").unwrap();
    journal.finish().unwrap();
    drop(journal);
    for stream in ["proxy", "cleanup"] {
        let mut writer =
            LifecycleWriter::create(root.join(format!("{stream}.jsonl")), stream, SESSION).unwrap();
        writer.finish().unwrap();
    }
    let mut application =
        ApplicationArtifactLease::open(root.join("application.jsonl"), SESSION, 4).unwrap();
    application.finish().unwrap();
    drop(application);
    open_sensitive_file(&root.join("tls-keylog.log"))
        .unwrap()
        .write_all(b"synthetic-keylog-fixture-only\n")
        .unwrap();
    fs::write(
        root.join("capture.fcapng"),
        b"synthetic-capture-fixture-only\n",
    )
    .unwrap();
    fs::write(root.join("cleanup.json"), b"{\"status\":\"succeeded\"}\n").unwrap();
    fs::write(root.join("process-trace.jsonl"), b"").unwrap();
    let threaded_path = root.join("compatibility.json");
    std::thread::spawn(move || {
        open_sensitive_file(&threaded_path)
            .unwrap()
            .write_all(b"{\"fixture\":\"independent-writer-thread\"}\n")
            .unwrap();
    })
    .join()
    .unwrap();
    let manifest = json!({
        "$schema": "https://fragcap.dev/schema/deep-capture-manifest.v2.json",
        "manifest_version": 2,
        "product": {"name":"fragcap", "version":env!("CARGO_PKG_VERSION")},
        "session_id": SESSION,
        "state": "complete",
        "artifacts": [],
        "omissions": [],
    });
    publish_final(root, &serde_json::to_vec(&manifest).unwrap()).unwrap();
}

#[test]
fn independent_writers_inherited_empty_and_atomic_files_have_actual_recipient_access() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("bundle");
    let recipient = OutputRecipient::current_account().unwrap();
    assert!(matches!(
        recipient.proof_kind(),
        "ordinary-current" | "session-linked" | "controlled-restricted-equivalent"
    ));
    write_bundle(&root, &recipient);
    let expected = contents(&root);
    assert!(expected.contains_key(Path::new(".sensitive-actions.jsonl")));
    assert!(expected.contains_key(Path::new("proxy.jsonl")));
    assert!(expected.contains_key(Path::new("cleanup.jsonl")));
    assert!(expected.contains_key(Path::new("resource-journal.jsonl")));
    assert!(expected.contains_key(Path::new("manifest.json")));
    assert!(!root.join("manifest.prefix.json").exists());
    assert!(!root.join("manifest.json.tmp").exists());
    let verified = verify_bundle_access(&root, &recipient).unwrap();
    assert!(verified.is_verified(), "{verified:?}");
    assert_eq!(verified.recipient_sid, recipient.sid());
    assert_eq!(verified.proof_kind, recipient.proof_kind());
    let observed: BTreeMap<PathBuf, Vec<u8>> = recipient
        .enumerate_fixture(&root)
        .unwrap()
        .into_iter()
        .map(|path| {
            let bytes = recipient.read_fixture(&path).unwrap();
            (path.file_name().unwrap().into(), bytes)
        })
        .collect();
    assert_eq!(observed, expected);
}

#[test]
fn denied_user_equivalent_has_a_readable_sibling_positive_control() {
    use fragcap::deep_capture::{
        protect_denied_user_fixture_path, verify_denied_user_fixture_access,
    };

    let temp = tempfile::tempdir().unwrap();
    protect_denied_user_fixture_path(temp.path(), true).unwrap();
    let control = temp.path().join("readable-control.txt");
    fs::write(&control, b"denied-user-equivalent-positive-control\n").unwrap();
    protect_denied_user_fixture_path(&control, false).unwrap();
    let positive =
        verify_denied_user_fixture_access(temp.path(), std::slice::from_ref(&control)).unwrap();
    assert!(
        positive.is_verified(),
        "ancestor or probe failure: {positive:?}"
    );

    let recipient = OutputRecipient::current_account().unwrap();
    let root = temp.path().join("bundle");
    write_bundle(&root, &recipient);
    let files: Vec<_> = contents(&root).keys().map(|name| root.join(name)).collect();
    let denied = verify_denied_user_fixture_access(&root, &files).unwrap();
    assert_eq!(denied.proof_kind, "controlled-denied-user-equivalent");
    assert!(
        !denied.is_verified(),
        "unrelated context read a private bundle: {denied:?}"
    );
    assert!(
        denied.paths.iter().all(|path| path.state != "verified"),
        "{denied:?}"
    );
    assert!(verify_bundle_access(&root, &recipient)
        .unwrap()
        .is_verified());
}

fn security_descriptor(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Security::{
        GetFileSecurityW, DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
    };

    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let information = DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION;
    let mut size = 0;
    unsafe { GetFileSecurityW(name.as_ptr(), information, ptr::null_mut(), 0, &mut size) };
    assert_ne!(
        size,
        0,
        "descriptor size: {}",
        std::io::Error::last_os_error()
    );
    let mut descriptor = vec![0_u8; size as usize];
    assert_ne!(
        unsafe {
            GetFileSecurityW(
                name.as_ptr(),
                information,
                descriptor.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        },
        0,
        "descriptor read: {}",
        std::io::Error::last_os_error(),
    );
    descriptor
}

fn old_group_only_policy(path: &Path, directory: bool) {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::{
        SetFileSecurityW, DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
    };
    use windows_sys::Win32::System::Memory::LocalFree;

    // This repository-owned equivalent reproduces group-dependent access while
    // preserving the current object owner. The producing ordinary token can
    // read via its exact logon-session group, but the selected group-denied
    // recipient cannot. Ordinary profile and volume group access is preserved.
    // No administrative owner assignment or elevation is required by the test.
    let inheritance = if directory { "OICI" } else { "" };
    let logon_sid = fragcap::deep_capture::current_fixture_logon_sid().unwrap();
    let sddl = format!(
        "D:P(A;{inheritance};FA;;;BA)(A;{inheritance};FA;;;{logon_sid})(A;{inheritance};FA;;;SY)"
    );
    let source: Vec<u16> = sddl.encode_utf16().chain(Some(0)).collect();
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut descriptor = ptr::null_mut();
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                source.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        },
        0,
    );
    let result = unsafe {
        SetFileSecurityW(
            name.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            descriptor,
        )
    };
    let error = std::io::Error::last_os_error();
    unsafe { LocalFree(descriptor as isize) };
    assert_ne!(result, 0, "old-policy fixture assignment failed: {error}");
}

#[test]
fn historical_group_only_sidecars_are_denied_then_repaired_without_content_changes() {
    use fragcap::deep_capture::{inspect_bundle_access, repair_bundle_access};

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("bundle");
    let recipient = OutputRecipient::group_denied_fixture().unwrap();
    assert_eq!(recipient.proof_kind(), "controlled-group-denied-equivalent");
    write_bundle(&root, &recipient);
    let before = contents(&root);
    let modified_before: BTreeMap<PathBuf, std::time::SystemTime> = before
        .keys()
        .map(|name| {
            (
                name.clone(),
                fs::metadata(root.join(name)).unwrap().modified().unwrap(),
            )
        })
        .collect();
    for name in [
        ".sensitive-actions.jsonl",
        "cleanup.jsonl",
        "proxy.jsonl",
        "resource-journal.jsonl",
    ] {
        let path = root.join(name);
        old_group_only_policy(&path, false);
        assert!(
            recipient.read_fixture(&path).is_err(),
            "old policy unexpectedly readable: {path:?}"
        );
    }
    let denied = verify_bundle_access(&root, &recipient).unwrap();
    assert!(!denied.is_verified(), "{denied:?}");
    assert_eq!(denied.state, "unresolved");
    assert_eq!(
        denied
            .paths
            .iter()
            .filter(|path| path.state == "unresolved")
            .count(),
        4
    );
    assert!(denied
        .paths
        .iter()
        .any(|path| path.path == Path::new("manifest.json") && path.state == "verified"));
    let inspection = inspect_bundle_access(&root, &recipient, None).unwrap();
    assert!(inspection.repairable, "{inspection:?}");
    assert_eq!(inspection.session_id, SESSION);
    assert_eq!(inspection.paths.len(), before.len() + 1);
    let repaired = repair_bundle_access(&root, &recipient, None, &inspection.id).unwrap();
    assert!(repaired.verified, "{repaired:?}");
    assert_eq!(contents(&root), before);
    for (name, bytes) in &before {
        assert_eq!(recipient.read_fixture(&root.join(name)).unwrap(), *bytes);
        assert_eq!(
            fs::metadata(root.join(name)).unwrap().modified().unwrap(),
            modified_before[name]
        );
    }
    let next = inspect_bundle_access(&root, &recipient, None).unwrap();
    let retried = repair_bundle_access(&root, &recipient, None, &next.id).unwrap();
    assert!(retried.verified, "{retried:?}");
    assert!(
        retried.paths.iter().all(|path| path.status == "unchanged"),
        "{retried:?}"
    );
    assert_eq!(contents(&root), before);
}

#[test]
fn owned_parent_repair_restores_enumeration_without_changing_a_sibling_bundle() {
    use fragcap::deep_capture::{inspect_bundle_access, repair_bundle_access};

    let temp = tempfile::tempdir().unwrap();
    let application = temp.path().join("fragcap");
    let sessions = application.join("sessions");
    let root = sessions.join("selected");
    let sibling = sessions.join("sibling");
    let recipient = OutputRecipient::group_denied_fixture().unwrap();
    write_bundle(&root, &recipient);
    write_bundle(&sibling, &recipient);
    let selected_before = contents(&root);
    let sibling_before = contents(&sibling);
    let sibling_descriptors: BTreeMap<PathBuf, Vec<u8>> = std::iter::once(sibling.clone())
        .chain(sibling_before.keys().map(|name| sibling.join(name)))
        .map(|path| (path.clone(), security_descriptor(&path)))
        .collect();
    old_group_only_policy(&sessions, true);
    old_group_only_policy(&application, true);
    assert!(recipient.enumerate_fixture(&sessions).is_err());
    assert!(recipient.enumerate_fixture(&application).is_err());
    let inspection = inspect_bundle_access(&root, &recipient, Some(&sessions)).unwrap();
    assert_eq!(
        inspection
            .paths
            .iter()
            .filter(|path| path.traversal_only)
            .count(),
        2
    );
    let repaired =
        repair_bundle_access(&root, &recipient, Some(&sessions), &inspection.id).unwrap();
    assert!(repaired.verified, "{repaired:?}");
    assert!(recipient.enumerate_fixture(&sessions).is_ok());
    assert!(recipient.enumerate_fixture(&application).is_ok());
    assert_eq!(contents(&root), selected_before);
    assert_eq!(contents(&sibling), sibling_before);
    for (path, descriptor) in sibling_descriptors {
        assert_eq!(
            security_descriptor(&path),
            descriptor,
            "sibling descriptor changed: {path:?}"
        );
    }
}

#[test]
fn stale_unknown_and_outside_hard_link_populations_change_no_descriptors_or_bytes() {
    use fragcap::deep_capture::{inspect_bundle_access, repair_bundle_access};

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("bundle");
    let recipient = OutputRecipient::current_account().unwrap();
    write_bundle(&root, &recipient);
    let inspection = inspect_bundle_access(&root, &recipient, None).unwrap();
    let unknown = root.join("unrelated.txt");
    fs::write(&unknown, b"unrelated-content\n").unwrap();
    let before = contents(&root);
    let descriptor_before = security_descriptor(&root);
    assert!(repair_bundle_access(&root, &recipient, None, &inspection.id).is_err());
    assert_eq!(contents(&root), before);
    assert_eq!(security_descriptor(&root), descriptor_before);
    fs::remove_file(unknown).unwrap();

    let fresh = inspect_bundle_access(&root, &recipient, None).unwrap();
    old_group_only_policy(&root.join("proxy.jsonl"), false);
    let changed_before = security_descriptor(&root.join("proxy.jsonl"));
    assert!(repair_bundle_access(&root, &recipient, None, &fresh.id).is_err());
    assert_eq!(
        security_descriptor(&root.join("proxy.jsonl")),
        changed_before
    );
    let outside = temp.path().join("external-alias.jsonl");
    fs::hard_link(root.join("proxy.jsonl"), &outside).unwrap();
    assert!(inspect_bundle_access(&root, &recipient, None).is_err());
    assert_eq!(security_descriptor(&outside), changed_before);
    assert_eq!(fs::read(outside).unwrap(), before[Path::new("proxy.jsonl")]);
}
