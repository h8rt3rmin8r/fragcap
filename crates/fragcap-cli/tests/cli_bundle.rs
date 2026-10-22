// SPDX-License-Identifier: Apache-2.0

mod common;

use common::run;

#[test]
fn retired_access_verbs_are_rejected_before_bundle_effects() {
    let root = tempfile::tempdir().unwrap();
    let evidence = root.path().join("evidence.txt");
    std::fs::write(&evidence, b"unchanged synthetic evidence").unwrap();
    let path = root.path().to_string_lossy();
    for json in [false, true] {
        for verb in ["access-inspect", "access-repair"] {
            let mut args = vec!["bundle", verb, &path];
            if verb == "access-repair" {
                args.extend(["--authorize", "old-inspection"]);
            }
            if json {
                args.insert(0, "--json");
            }
            let (code, out, err) = run(&args);
            assert_eq!(code, 2, "retired {verb}: {err}");
            assert!(err.contains("unrecognized subcommand"), "{err}");
            assert!(out.is_empty(), "retired {verb}: {out}");
            assert_eq!(
                std::fs::read(&evidence).unwrap(),
                b"unchanged synthetic evidence"
            );
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
        }
    }
    let (code, out, err) = run(&["bundle", "--help"]);
    assert_eq!(code, 0, "{err}");
    assert!(!out.contains("access-inspect") && !out.contains("access-repair"));
    assert!(out.contains("access-authorize") && out.contains("cleanup") && out.contains("collect"));
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

#[test]
fn collection_preview_never_offers_apply_for_missing_or_unowned_bundle() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing bundle");
    let empty = root.path().join("empty bundle");
    std::fs::create_dir(&empty).unwrap();
    for bundle in [&missing, &empty] {
        let path = bundle.to_string_lossy();
        let (code, human, err) = run(&["bundle", "collect", "--bundle", &path]);
        assert_eq!(code, 0, "{err}");
        let (json_code, json, json_err) = run(&["--json", "bundle", "collect", "--bundle", &path]);
        assert_eq!(json_code, 0, "{json_err}");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["eligible_sessions"], 0, "{json}");
        if bundle == &missing {
            assert_eq!(value["complete"], false, "{json}");
            assert_eq!(value["unresolved_sessions"], 1, "{json}");
        } else {
            assert_eq!(value["retained_sessions"], 1, "{json}");
        }
        assert!(!human.contains("Apply reviewed proposal:"), "{human}");
        let row = &value["results"][0];
        let reason = row["reason"].as_str().unwrap();
        assert!(human.contains(reason), "{human}");
        if let Some(detail) = row["detail"].as_str() {
            let normalized = human.split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(
                normalized.contains(detail),
                "missing detail {detail:?}: {human}"
            );
        }
        assert!(human.contains("Next step:"), "{human}");
        if bundle == &missing {
            assert!(human.contains("Check the bundle path"), "{human}");
        } else {
            assert!(human.contains("No eligible contents"), "{human}");
        }
        assert_eq!(bundle.exists(), bundle == &empty);
    }
}

#[test]
fn exact_retained_preview_offers_only_its_authorized_collection() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("completed");
    fragcap::deep_capture::prepare_bundle(&bundle).unwrap();
    let mut journal =
        fragcap::deep_capture::ResourceJournal::create(&bundle, "s178-fixture", "plan").unwrap();
    journal.finish().unwrap();
    drop(journal);
    std::fs::write(bundle.join("capture.fcapng"), b"synthetic packet bytes\n").unwrap();
    std::fs::write(
        bundle.join("manifest.json"),
        br#"{"manifest_version":1,"session_id":"s178-fixture","state":"complete","artifacts":[{"path":"capture.fcapng","sensitivity":"ordinary"}]}"#,
    )
    .unwrap();
    let path = bundle.to_string_lossy();

    let (code, ordinary, err) = run(&["bundle", "collect", "--bundle", &path]);
    assert_eq!(code, 0, "{err}");
    assert!(!ordinary.contains("Apply reviewed proposal:"), "{ordinary}");
    assert!(bundle.join("capture.fcapng").exists());
    std::fs::create_dir_all(std::env::var_os("FRAGCAP_SESSION_DIR").unwrap()).unwrap();

    let (code, human, err) = run(&["bundle", "collect", "--bundle", &path, "--include-retained"]);
    assert_eq!(code, 0, "{err}");
    assert!(human.contains("Apply reviewed proposal:"), "{human}");
    assert!(human.contains("--include-retained"), "{human}");
    let (code, json, err) = run(&[
        "--json",
        "bundle",
        "collect",
        "--bundle",
        &path,
        "--include-retained",
    ]);
    assert_eq!(code, 0, "{err}");
    let preview: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(preview["complete"], true, "{json}");
    assert_eq!(preview["eligible_sessions"], 1, "{json}");
    let id = preview["proposal_id"].as_str().unwrap();
    assert!(human.contains(id), "{human}");
    let (code, applied, err) = run(&[
        "--json",
        "bundle",
        "collect",
        "--bundle",
        &path,
        "--include-retained",
        "--authorize",
        id,
    ]);
    assert_eq!(code, 0, "{applied} {err}");
    let result: serde_json::Value = serde_json::from_str(&applied).unwrap();
    assert_eq!(result["complete"], true, "{applied}");
    assert!(!bundle.join("capture.fcapng").exists());
    assert!(
        bundle.exists(),
        "routine collection preserves its container"
    );
}

#[test]
fn missing_bundle_cleanup_and_export_name_the_path_before_guiding_action() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing bundle");
    let share = root.path().join("share");
    let path = missing.to_string_lossy();
    let destination = share.to_string_lossy();
    let (code, out, err) = run(&["bundle", "cleanup", &path]);
    assert_eq!(code, 2, "{out}\n{err}");
    assert!(err.contains(path.as_ref()), "{err}");
    assert!(!err.contains(" --yes"), "{err}");
    let (code, out, err) = run(&["bundle", "export", &path, "--out", &destination]);
    assert_ne!(code, 0, "{out}");
    assert!(err.contains(path.as_ref()), "{err}");
    assert!(err.contains("Check the bundle path"), "{err}");
    assert!(!share.exists());
}

#[test]
fn invalid_cleanup_manifest_never_recommends_confirmation() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("invalid");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::write(bundle.join("manifest.json"), b"not json").unwrap();
    let path = bundle.to_string_lossy();
    let (code, out, err) = run(&["bundle", "cleanup", &path]);
    assert_eq!(code, 2, "{out} {err}");
    assert!(err.contains(path.as_ref()), "{err}");
    assert!(!err.contains("--yes"), "{err}");
    assert_eq!(
        std::fs::read(bundle.join("manifest.json")).unwrap(),
        b"not json"
    );
}

#[test]
fn collection_help_names_deletion_and_retained_scope() {
    for flag in ["-h", "--help"] {
        let (code, out, err) = run(&["bundle", "collect", flag]);
        assert_eq!(code, 0, "{err}");
        assert!(out.contains("deletes eligible bundle contents"), "{out}");
        assert!(out.contains("--include-retained"), "{out}");
        assert!(out.contains("retained"), "{out}");
    }
}
