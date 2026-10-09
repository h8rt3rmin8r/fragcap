// SPDX-License-Identifier: Apache-2.0

mod common;

use common::run;

#[cfg(windows)]
#[test]
fn access_cli_inspection_is_read_only_and_repair_binds_the_exact_preview() {
    use fragcap::deep_capture::{OutputRecipient, ResourceJournal, MANIFEST_SCHEMA};
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("retained");
    fragcap::deep_capture::prepare_bundle(&bundle).unwrap();
    let mut journal = ResourceJournal::create(&bundle, "cli-access", "fixture").unwrap();
    journal.finish().unwrap();
    drop(journal);
    let capture = bundle.join("capture.fcapng");
    std::fs::write(&capture, b"synthetic evidence").unwrap();
    std::fs::write(bundle.join("manifest.json"), serde_json::json!({"$schema":MANIFEST_SCHEMA,"manifest_version":2,"product":{"name":"fragcap","version":"fixture"},"session_id":"cli-access","state":"complete","artifacts":[],"omissions":[]}).to_string()).unwrap();
    let path = bundle.to_string_lossy();
    let before = std::fs::read(&capture).unwrap();
    let (code, out, err) = run(&["--json", "bundle", "access-inspect", &path]);
    assert_eq!(std::fs::read(&capture).unwrap(), before);
    if OutputRecipient::desktop_session().is_err() {
        assert_eq!(code, 1);
        assert!(err.contains("cannot establish the ordinary output recipient"));
        assert!(out.is_empty());
        return;
    }
    assert_eq!(code, 0, "{err}");
    let inspection: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(inspection["type"], "bundle-access.inspection");
    let id = inspection["inspection_id"].as_str().unwrap();
    let (code, _, err) = run(&[
        "--json",
        "bundle",
        "access-repair",
        &path,
        "--authorize",
        "stale",
    ]);
    assert_eq!(code, 1);
    assert!(err.contains("inspection changed"));
    assert_eq!(std::fs::read(&capture).unwrap(), before);
    let (code, out, err) = run(&[
        "--json",
        "bundle",
        "access-repair",
        &path,
        "--authorize",
        id,
    ]);
    assert_eq!(code, 0, "{err}");
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["verified"], true);
    assert_eq!(std::fs::read(&capture).unwrap(), before);
}

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
    assert!(
        err.contains("Next command:  fragcap bundle cleanup") && err.contains("--yes"),
        "cleanup refusal provides the exact confirmed retry: {err}"
    );
    let (code, _, json_err) = run(&["--json", "bundle", "cleanup", &path]);
    assert_eq!(code, 2);
    assert!(json_err.contains("pass --yes"));
    assert!(
        !json_err.contains("Next command:"),
        "human guidance changed the JSON-mode error contract: {json_err}"
    );
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
