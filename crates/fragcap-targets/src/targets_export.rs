// SPDX-License-Identifier: Apache-2.0

//! Target-entry export and import (slice S055).
//!
//! A dedicated JSON array of target-entry objects, carrying each entry's identity
//! so an export round-trips through an import with identical identifiers and no
//! duplicate rows (operator decision, 2026-08-18). This is deliberately NOT the
//! published capture schema (`target-schema.v1.json`), whose export records are
//! catalog games and omit the entry identity that merge-on-id needs.
//!
//! The mapping is explicit (hand-written, like the catalog exporter) so the JSON
//! key set is a reviewed contract rather than a serde-derive accident. `id` (the
//! rowid) is never exported; `stable_id` is the durable, merge-on identity.

use serde_json::{json, Map, Value};

use crate::entry::{ClassificationSource, TargetClassification, TargetEntry};
use crate::TargetsError;
use fragcap_profile::FidelityTier;

/// Export target entries as a single pretty-printed JSON array, ordered by handle
/// for stable output. An empty slice exports `[]`.
pub fn export_targets(entries: &[TargetEntry]) -> String {
    let mut ordered: Vec<&TargetEntry> = entries.iter().collect();
    ordered.sort_by(|a, b| a.handle.cmp(&b.handle));
    let array = Value::Array(ordered.into_iter().map(entry_to_value).collect());
    let mut text = serde_json::to_string_pretty(&array).expect("array serializes");
    text.push('\n');
    text
}

/// Parse a target-entry array. Every element is validated structurally (required
/// fields present, enum values legal) before any is returned, so a nonconforming
/// file yields an error and the caller applies nothing (all-or-nothing, FR-019).
pub fn import_targets(json: &str) -> Result<Vec<TargetEntry>, TargetsError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TargetsError::Model(format!("import is not valid JSON: {e}")))?;
    let array = value
        .as_array()
        .ok_or_else(|| TargetsError::Model("import must be a JSON array of targets".to_string()))?;
    array.iter().map(value_to_entry).collect()
}

/// One entry as its export object. Optional fields are emitted only when present,
/// keeping the record minimal.
fn entry_to_value(entry: &TargetEntry) -> Value {
    let mut map = Map::new();
    map.insert("stable_id".to_string(), json!(entry.stable_id));
    map.insert("handle".to_string(), json!(entry.handle));
    map.insert("name".to_string(), json!(entry.name));
    map.insert(
        "classification".to_string(),
        json!(entry.classification.as_str()),
    );
    map.insert(
        "classification_source".to_string(),
        json!(entry.classification_source.as_str()),
    );
    map.insert("fidelity".to_string(), json!(entry.fidelity.as_str()));
    if let Some(anchor) = &entry.anchor {
        map.insert("anchor".to_string(), json!(anchor));
    }
    if let Some(launch) = &entry.launch_entries {
        map.insert("launch_entries".to_string(), launch.clone());
    }
    if let Some(root) = &entry.install_root {
        map.insert("install_root".to_string(), json!(root));
    }
    if let Some(evidence) = &entry.evidence {
        map.insert("evidence".to_string(), evidence.clone());
    }
    if let Some(provenance) = &entry.provenance {
        map.insert("provenance".to_string(), provenance.clone());
    }
    Value::Object(map)
}

/// One export object back to an entry, validating the required fields. The rowid is
/// always `None` (import assigns or merges by `stable_id`, never by rowid).
fn value_to_entry(value: &Value) -> Result<TargetEntry, TargetsError> {
    let obj = value
        .as_object()
        .ok_or_else(|| TargetsError::Model("each target must be a JSON object".to_string()))?;
    let stable_id = obj
        .get("stable_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            TargetsError::Model("a target is missing an integer stable_id".to_string())
        })?;
    let handle = required_str(obj, "handle")?;
    let name = required_str(obj, "name")?;
    let classification = TargetClassification::parse(&required_str(obj, "classification")?)?;
    let classification_source =
        ClassificationSource::parse(&required_str(obj, "classification_source")?)?;
    let fidelity = FidelityTier::parse(&required_str(obj, "fidelity")?)
        .ok_or_else(|| TargetsError::Model(format!("target {handle:?} has an unknown fidelity")))?;
    Ok(TargetEntry {
        id: None,
        stable_id,
        handle,
        name,
        classification,
        classification_source,
        fidelity,
        provenance: obj.get("provenance").cloned(),
        anchor: optional_str(obj, "anchor"),
        launch_entries: obj.get("launch_entries").cloned(),
        install_root: optional_str(obj, "install_root"),
        evidence: obj.get("evidence").cloned(),
    })
}

fn required_str(obj: &Map<String, Value>, key: &str) -> Result<String, TargetsError> {
    obj.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| TargetsError::Model(format!("a target is missing a non-empty {key}")))
}

fn optional_str(obj: &Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn sample() -> TargetEntry {
        TargetEntry {
            id: None,
            stable_id: 306130,
            handle: "the_elder_scrolls_online".to_string(),
            name: "The Elder Scrolls Online".to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity: FidelityTier::Authored,
            provenance: None,
            anchor: Some("steam:306130".to_string()),
            launch_entries: Some(json!([{ "executable": "eso64.exe", "role": "client" }])),
            install_root: None,
            evidence: None,
        }
    }

    #[test]
    fn export_import_round_trips_identically() {
        let entries = vec![sample()];
        let doc = export_targets(&entries);
        let parsed = import_targets(&doc).expect("import");
        assert_eq!(parsed, entries);
    }

    #[test]
    fn import_rejects_a_non_array_and_a_bad_enum() {
        assert!(import_targets("{}").is_err(), "a non-array is rejected");
        assert!(
            import_targets(r#"[{"stable_id":1,"handle":"g","name":"G","classification":"nonsense","classification_source":"user","fidelity":"authored"}]"#)
                .is_err(),
            "an out-of-set classification is rejected"
        );
        assert!(
            import_targets(r#"[{"handle":"g","name":"G","classification":"game","classification_source":"user","fidelity":"authored"}]"#)
                .is_err(),
            "a missing stable_id is rejected"
        );
    }

    #[test]
    fn round_trip_through_a_store_preserves_identity_without_duplicates() {
        let mut source = Store::open_in_memory().expect("store");
        source.insert_target(&sample()).expect("insert");
        let doc = export_targets(&source.targets().expect("targets"));

        let mut dest = Store::open_in_memory().expect("store");
        let imported = import_targets(&doc).expect("import");
        for entry in &imported {
            dest.insert_target(entry).expect("insert imported");
        }
        // A second import updates in place rather than duplicating.
        for entry in &import_targets(&doc).expect("reimport") {
            if dest
                .target_by_stable_id(entry.stable_id)
                .expect("lookup")
                .is_some()
            {
                dest.update_target(entry).expect("update");
            } else {
                dest.insert_target(entry).expect("insert");
            }
        }
        let dest_targets = dest.targets().expect("targets");
        assert_eq!(dest_targets.len(), 1, "no duplicate rows");
        assert_eq!(dest_targets[0].stable_id, 306130);
    }
}
