// SPDX-License-Identifier: Apache-2.0

mod common;

use common::run;

fn bundle(root: &std::path::Path) -> std::path::PathBuf {
    let bundle = root.join("bundle");
    fragcap::deep_capture::prepare_bundle(&bundle).unwrap();
    std::fs::write(bundle.join("capture.fcapng"), b"ordinary").unwrap();
    std::fs::write(bundle.join("tls-keylog.log"), b"secret").unwrap();
    std::fs::write(
        bundle.join("manifest.json"),
        br#"{"artifacts":[{"path":"capture.fcapng","sensitivity":"ordinary"},{"path":"tls-keylog.log","sensitivity":"secret-adjacent"}]}"#,
    )
    .unwrap();
    bundle
}

#[test]
fn cleanup_requires_confirmation_and_removes_only_sensitive_evidence() {
    let root = tempfile::tempdir().unwrap();
    let bundle = bundle(root.path());
    let path = bundle.to_string_lossy();
    let (code, _, err) = run(&["bundle", "cleanup", &path]);
    assert_eq!(code, 2);
    assert!(err.contains("pass --yes"));
    assert!(bundle.join("tls-keylog.log").exists());

    let (code, out, err) = run(&["bundle", "cleanup", &path, "--yes"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("removed"));
    assert!(!bundle.join("tls-keylog.log").exists());
    assert!(bundle.join("capture.fcapng").exists());
}

#[test]
fn export_is_a_separate_copy_with_an_exhaustive_manifest() {
    let root = tempfile::tempdir().unwrap();
    let bundle = bundle(root.path());
    let share = root.path().join("share");
    let source = bundle.to_string_lossy();
    let destination = share.to_string_lossy();
    let before = std::fs::read(bundle.join("tls-keylog.log")).unwrap();
    let (code, out, err) = run(&["bundle", "export", &source, "--out", &destination]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("sharing-manifest.json"));
    assert!(share.join("capture.fcapng").exists());
    assert!(!share.join("tls-keylog.log").exists());
    assert!(!share.join(".sensitive-actions.jsonl").exists());
    assert_eq!(
        std::fs::read(bundle.join("tls-keylog.log")).unwrap(),
        before
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(share.join("sharing-manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["status"], "complete");
    assert_eq!(manifest["omitted"][0]["path"], "tls-keylog.log");
}
