// SPDX-License-Identifier: Apache-2.0

//! Controlled global result-mode regressions for issue #488.

mod common;

use common::run;
use serde_json::Value;

fn records(text: &str) -> Vec<Value> {
    text.lines()
        .map(|line| serde_json::from_str(line).expect("one complete JSON record per line"))
        .collect()
}

#[test]
fn target_results_and_silence_use_selected_shape() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local.db").to_string_lossy().into_owned();
    let (code, out, err) = run(&["--json", "targets", "list", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    let rows = records(&out);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["event"], "targets.list");
    assert_eq!(rows[0]["count"], 0);
    assert_eq!(rows[0]["targets"], serde_json::json!([]));

    let (code, out, err) = run(&["--silent", "targets", "list", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_empty(), "{out}");

    let (code, out, err) = run(&["--json", "--silent", "targets", "list", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["event"], "targets.list");

    let (code, out, err) = run(&["--json", "targets", "add", "Sample", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["event"], "targets.add");
    let handle = records(&out)[0]["target"]["handle"]
        .as_str()
        .unwrap()
        .to_owned();
    let (code, out, err) = run(&["--json", "targets", "show", &handle, "--db", &db]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["target"]["handle"], handle);
    let (code, out, err) = run(&["--json", "targets", "show", "missing-target", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["status"], "no_match");
    let (code, out, err) = run(&["--silent", "targets", "show", &handle, "--db", &db]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_empty(), "{out}");
}

#[test]
fn schema_validation_keeps_each_defect_and_document_print() {
    let dir = tempfile::tempdir().unwrap();
    let invalid = dir.path().join("invalid.json");
    std::fs::write(&invalid, r#"{"schema":1,"kind":"profile","fidelity":"invalid","extra":true,"game":{"id":"game"},"stage":[{"role":"client","lifecycle":"session","match":{}}]}"#).unwrap();
    let path = invalid.to_string_lossy().into_owned();
    let (code, out, err) = run(&["--json", "schema", "validate", &path]);
    assert_eq!(code, 2, "{err}");
    let rows = records(&out);
    assert!(rows.len() >= 5, "{out}");
    assert_eq!(rows.last().unwrap()["event"], "schema.summary");
    assert_eq!(rows.last().unwrap()["diagnostics"], rows.len() - 1);
    for row in &rows[..rows.len() - 1] {
        assert_eq!(row["event"], "schema.diagnostic");
        for key in ["code", "path", "line", "column", "message"] {
            assert!(row.get(key).is_some(), "missing {key}: {row}");
        }
    }

    let (code, out, err) = run(&["--json", "schema", "print"]);
    assert_eq!(code, 0, "{err}");
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert!(doc.get("$schema").is_some());
    assert!(doc.get("event").is_none());

    let malformed = dir.path().join("malformed.json");
    std::fs::write(&malformed, "{ broken\n").unwrap();
    let (code, out, err) = run(&["--json", "schema", "validate", &malformed.to_string_lossy()]);
    assert_eq!(code, 2, "{err}");
    let rows = records(&out);
    assert_eq!(rows[0]["code"], "invalid-json");
    assert!(rows[0]["line"].as_u64().is_some());
    assert_eq!(rows[1]["diagnostics"], 1);
}

#[test]
fn catalog_seed_reports_tier_counters_and_export_stays_a_document() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("catalog.db").to_string_lossy().into_owned();
    let (code, out, err) = run(&[
        "--json",
        "catalog",
        "seed",
        "--tier",
        "signature",
        "--db",
        &db,
    ]);
    assert_eq!(code, 0, "{err}");
    let rows = records(&out);
    assert_eq!(rows[0]["event"], "catalog.seed.tier");
    assert_eq!(rows[0]["tier"], "signature");
    assert!(rows[0]["written"].as_u64().unwrap() > 0);
    assert_eq!(rows.last().unwrap()["event"], "catalog.seed.summary");

    let (code, out, err) = run(&["--json", "catalog", "export", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    let doc: Value = serde_json::from_str(&out).unwrap();
    assert!(doc.get("records").is_some());
    assert!(doc.get("event").is_none());

    let (code, out, err) = run(&["--json", "catalog", "seed", "--db", &db]);
    assert_eq!(code, 0, "{err}");
    let rows = records(&out);
    assert!(rows
        .iter()
        .any(|row| row["status"] == "skipped" && row["reason"].is_string()));

    let seed = dir.path().join("seed.json");
    std::fs::write(&seed, r#"{"schema":1,"kind":"export","fidelity":"heuristic-unverified","provenance":{"source":"hint-db"},"records":[{"fidelity":"heuristic-unverified","provenance":{"source":"hint-db"},"game":{"app_id":"306130","platform":"steam","name":"Test Game"}}]}"#).unwrap();
    let (code, out, err) = run(&[
        "--json",
        "catalog",
        "import",
        &seed.to_string_lossy(),
        "--db",
        &db,
    ]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["event"], "catalog.import");
    assert_eq!(records(&out)[0]["imported"], 1);

    let (code, out, err) = run(&[
        "--silent",
        "catalog",
        "seed",
        "--tier",
        "signature",
        "--db",
        &db,
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_empty(), "{out}");
}

#[test]
fn existing_machine_channels_survive_json_plus_silent() {
    let (doctor_code, doctor_out, doctor_err) = run(&["--json", "--silent", "doctor"]);
    assert!(doctor_code <= 1, "{doctor_err}");
    assert!(!records(&doctor_out).is_empty(), "doctor check records");
    let (human_code, out, _err) = run(&["--silent", "doctor"]);
    if human_code == 0 {
        assert!(out.is_empty(), "{out}");
    } else {
        assert!(out.contains("fail"), "{out}");
    }

    let (code, out, err) = run(&[
        "--json",
        "--silent",
        "capture",
        "--process",
        "game.exe",
        "--replay-source",
        "missing.pcap",
    ]);
    assert_ne!(code, 0);
    assert!(out.is_empty(), "{out}");
    assert!(
        records(&err).iter().any(|row| row["event"].is_string()),
        "{err}"
    );
}

#[test]
fn remaining_families_preserve_machine_artifacts_or_typed_results() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = dir.path().join("catalog.db").to_string_lossy().into_owned();
    let (code, _, err) = run(&["catalog", "seed", "--tier", "signature", "--db", &catalog]);
    assert_eq!(code, 0, "{err}");
    let (code, out, err) = run(&[
        "--json",
        "technologies",
        "--path",
        &dir.path().to_string_lossy(),
        "--catalog-db",
        &catalog,
    ]);
    assert_eq!(code, 0, "{err}");
    let row = &records(&out)[0];
    assert_eq!(row["event"], "technologies.detect");
    assert!(row["findings"].is_array());
    assert!(row["coverage_warnings"].is_array());

    let (code, out, err) = run(&[
        "--silent",
        "technologies",
        "--path",
        &dir.path().to_string_lossy(),
        "--catalog-db",
        &catalog,
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_empty(), "{out}");

    let (code, out, err) = run(&["--json", "--silent", "extcap", "--extcap-interfaces"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("extcap {version="), "{out}");
    assert!(!out.contains("\"event\""), "{out}");

    let extcap_dir = dir.path().join("extcap");
    let (code, out, err) = run(&[
        "--json",
        "extcap",
        "uninstall",
        "--dir",
        &extcap_dir.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["status"], "absent");

    let (code, out, err) = run(&["--json", "--silent", "__build-identity"]);
    assert_eq!(code, 0, "{err}");
    let identity: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(identity["product"], "fragcap");

    let (code, out, err) = run(&["--json", "doctor", "--fix"]);
    assert_eq!(code, 2);
    assert!(out.is_empty(), "{out}");
    assert_eq!(records(&err)[0]["event"], "error");

    let (code, out, err) = run(&["--json", "--silent", "bundle", "collect"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(records(&out)[0]["mode"], "preview");

    let (code, out, err) = run(&["--json", "replay"]);
    assert_ne!(code, 0);
    assert!(out.is_empty(), "{out}");
    assert_eq!(records(&err)[0]["event"], "error");
}
