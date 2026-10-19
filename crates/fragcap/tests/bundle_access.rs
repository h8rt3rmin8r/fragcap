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

fn fixture_policy(path: &Path, sddl: &str) {
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
fn missing_nested_containers_have_recipient_access_without_changing_existing_objects() {
    let temp = tempfile::tempdir().unwrap();
    let ancestor = temp.path().join("non-inheriting-recipient");
    fs::create_dir(&ancestor).unwrap();
    let recipient = OutputRecipient::group_denied_fixture().unwrap();
    let logon_sid = fragcap::deep_capture::current_fixture_logon_sid().unwrap();
    fixture_policy(
        &ancestor,
        &format!(
            "D:P(A;;FA;;;{})(A;OICI;FA;;;{logon_sid})(A;OICI;FA;;;SY)",
            recipient.sid()
        ),
    );
    recipient.enumerate_fixture(&ancestor).unwrap();
    // Ordinary enumeration of the existing parent proves no inheritable grant.
    let sibling = ancestor.join("producer-only-sibling");
    fs::create_dir(&sibling).unwrap();
    fs::write(sibling.join("retained.txt"), b"unchanged sibling bytes").unwrap();
    assert!(recipient.enumerate_fixture(&sibling).is_err());
    let existing = [
        ancestor.clone(),
        sibling.clone(),
        sibling.join("retained.txt"),
    ];
    let descriptors = existing
        .iter()
        .map(|path| (path.clone(), security_descriptor(path)))
        .collect::<BTreeMap<_, _>>();
    let root = ancestor.join("first").join("second").join("bundle");
    write_bundle(&root, &recipient);
    for directory in [
        ancestor.join("first"),
        ancestor.join("first/second"),
        root.clone(),
    ] {
        assert!(!recipient.enumerate_fixture(&directory).unwrap().is_empty());
    }
    let verified = verify_bundle_access(&root, &recipient).unwrap();
    assert!(verified.is_verified(), "{verified:?}");
    for (name, bytes) in contents(&root) {
        assert_eq!(recipient.read_fixture(&root.join(name)).unwrap(), bytes);
    }
    for (path, descriptor) in descriptors {
        assert_eq!(
            security_descriptor(&path),
            descriptor,
            "changed existing ACL: {path:?}"
        );
    }
    assert_eq!(
        fs::read(sibling.join("retained.txt")).unwrap(),
        b"unchanged sibling bytes"
    );
    assert!(recipient.enumerate_fixture(&sibling).is_err());
}

#[test]
#[ignore = "requires a genuine elevated same-account UAC desktop; run explicitly in the controlled Windows tier"]
fn elevated_desktop_production_recipient_reads_every_final_artifact() {
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TokenElevationType, TokenElevationTypeFull,
    };
    let query = |class| {
        let mut value = 0_u32;
        let mut size = 0;
        assert_ne!(
            unsafe {
                GetTokenInformation(-4, class, (&mut value as *mut u32).cast(), 4, &mut size)
            },
            0,
            "{}",
            std::io::Error::last_os_error()
        );
        assert_eq!(size, 4);
        value
    };
    assert_ne!(
        query(TokenElevation),
        0,
        "required row needs a genuinely elevated producer"
    );
    assert_eq!(
        query(TokenElevationType),
        TokenElevationTypeFull as u32,
        "required row needs actual UAC split elevation"
    );
    let recipient =
        OutputRecipient::desktop_session().expect("production desktop recipient acquisition");
    assert_eq!(recipient.proof_kind(), "session-shell");
    recipient.revalidate().unwrap();
    let public = std::env::var_os("FRAGCAP_TEST_PUBLIC_ROOT")
        .expect("dedicated fixture needs a shared synthetic test root");
    let temp = tempfile::tempdir_in(public).unwrap();
    let control = temp.path().join("ordinary-control.txt");
    fs::write(&control, b"synthetic ordinary positive control").unwrap();
    let root = temp.path().join("first/second/bundle");
    write_bundle(&root, &recipient);
    for directory in [
        temp.path().to_path_buf(),
        temp.path().join("first"),
        temp.path().join("first/second"),
        root.clone(),
    ] {
        assert!(!recipient.enumerate_fixture(&directory).unwrap().is_empty());
    }
    let expected = contents(&root);
    assert!(expected.contains_key(Path::new(".sensitive-actions.jsonl")));
    assert!(expected.contains_key(Path::new("resource-journal.jsonl")));
    assert!(expected.contains_key(Path::new("manifest.json")));
    assert!(expected.contains_key(Path::new("process-trace.jsonl")));
    assert!(!root.join("manifest.prefix.json").exists());
    assert!(!root.join("manifest.json.tmp").exists());
    assert!(verify_bundle_access(&root, &recipient)
        .unwrap()
        .is_verified());
    let actual: BTreeMap<_, _> = recipient
        .enumerate_fixture(&root)
        .unwrap()
        .into_iter()
        .map(|file| {
            (
                PathBuf::from(file.file_name().unwrap()),
                recipient.read_fixture(&file).unwrap(),
            )
        })
        .collect();
    assert_eq!(actual, expected);
    assert_unrelated_ordinary_access_is_denied(&root, &control);
    denied_user_equivalent_has_a_readable_sibling_positive_control();
}

fn assert_unrelated_ordinary_access_is_denied(root: &Path, control: &Path) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        EqualSid, GetTokenInformation, ImpersonateLoggedOnUser, LogonUserW, RevertToSelf,
        TokenElevation, TokenUser, LOGON32_LOGON_INTERACTIVE, LOGON32_PROVIDER_DEFAULT, TOKEN_USER,
    };
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
    fn user(token: isize) -> Vec<usize> {
        let mut size = 0;
        unsafe { GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut size) };
        assert!(size > 0 && size < 65536);
        let mut bytes = vec![0_usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
        assert_ne!(
            unsafe {
                GetTokenInformation(token, TokenUser, bytes.as_mut_ptr().cast(), size, &mut size)
            },
            0
        );
        bytes
    }
    let name = std::env::var("FRAGCAP_TEST_UNRELATED_USER")
        .expect("dedicated fixture needs a disposable unrelated standard user");
    let password = std::env::var("FRAGCAP_TEST_UNRELATED_PASSWORD")
        .expect("dedicated fixture needs disposable credentials");
    let root = root.to_path_buf();
    let control = control.to_path_buf();
    std::thread::spawn(move || {
        let mut token = 0;
        assert_ne!(
            unsafe {
                LogonUserW(
                    wide(&name).as_ptr(),
                    wide(".").as_ptr(),
                    wide(&password).as_ptr(),
                    LOGON32_LOGON_INTERACTIVE,
                    LOGON32_PROVIDER_DEFAULT,
                    &mut token,
                )
            },
            0,
            "unrelated fixture logon: {}",
            std::io::Error::last_os_error()
        );
        let actual = user(token);
        let producer = user(-4);
        let differs = unsafe {
            EqualSid(
                (*actual.as_ptr().cast::<TOKEN_USER>()).User.Sid,
                (*producer.as_ptr().cast::<TOKEN_USER>()).User.Sid,
            )
        } == 0;
        let mut elevation = 1_u32;
        let mut size = 0;
        let queried = unsafe {
            GetTokenInformation(
                token,
                TokenElevation,
                (&mut elevation as *mut u32).cast(),
                4,
                &mut size,
            )
        } != 0;
        let files = contents(&root)
            .keys()
            .map(|name| root.join(name))
            .collect::<Vec<_>>();
        if unsafe { ImpersonateLoggedOnUser(token) } == 0 {
            let error = std::io::Error::last_os_error();
            unsafe { CloseHandle(token) };
            panic!("unrelated ordinary fixture impersonation: {error}");
        }
        let positive = fs::read(&control)
            .map(|v| v == b"synthetic ordinary positive control")
            .unwrap_or(false);
        let enumeration_denied =
            fs::read_dir(&root).is_err_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied);
        let all_denied = files.iter().all(|file| {
            fs::read(file).is_err_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied)
        });
        let restored = unsafe { RevertToSelf() } != 0;
        unsafe { CloseHandle(token) };
        assert!(restored, "restore unrelated fixture thread");
        assert!(
            differs && queried && elevation == 0,
            "unrelated fixture must have a distinct genuinely ordinary token"
        );
        assert!(
            positive,
            "ordinary principal must read the sibling positive control"
        );
        assert!(
            enumeration_denied && all_denied,
            "unrelated ordinary principal read private retained evidence"
        );
    })
    .join()
    .unwrap();
}
