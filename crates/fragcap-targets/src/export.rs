// SPDX-License-Identifier: Apache-2.0

//! Projecting the store into a schema-conformant `kind: "export"` document.
//!
//! The document is built as a [`serde_json::Value`] (so escaping is correct by
//! construction and the output re-parses), then validated against the published
//! schema before it is returned. The exporter never returns a document the
//! validator rejects: a self-validation failure is an internal error, surfaced
//! rather than emitted (P-9, validity by construction).

use fragcap_profile::jsonschema::validate_value;
use serde_json::{json, Map, Value};

use crate::model::{Engine, Game, LaunchEntry};
use crate::store::Store;
use crate::TargetsError;

/// The provenance source stamped on the export and every record this slice. The
/// per-tier provenance detail is a later-slice concern; here the whole store has
/// one origin. Shared with the hint provider (S037) so a resolution answer and an
/// export name the store's origin identically (one honest name, P-9).
pub(crate) const PROVENANCE_SOURCE: &str = "hint-db";

/// Every record carries this fidelity. Engine confidence grades one field within
/// it and never changes it (P-9).
const RECORD_FIDELITY: &str = "heuristic-unverified";

/// Export the whole store as a single pretty-printed `kind: "export"` document,
/// validated against the master schema.
///
/// An empty store exports a valid envelope with an empty `records` array.
pub fn export(store: &Store) -> Result<String, TargetsError> {
    let games = store.games()?;
    let records: Vec<Value> = games.iter().map(record_for_game).collect();

    let mut doc = Map::new();
    doc.insert("schema".to_string(), json!(1));
    doc.insert("kind".to_string(), json!("export"));
    doc.insert("fidelity".to_string(), json!(RECORD_FIDELITY));
    doc.insert(
        "provenance".to_string(),
        json!({ "source": PROVENANCE_SOURCE }),
    );
    doc.insert("records".to_string(), Value::Array(records));
    let value = Value::Object(doc);

    let diagnostics = validate_value(&value);
    if !diagnostics.is_empty() {
        return Err(TargetsError::ExportInvalid(format!("{diagnostics}")));
    }

    let mut text = serde_json::to_string_pretty(&value)
        .map_err(|e| TargetsError::ExportInvalid(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

/// Build one `record` object for a game. Unknown engine and empty launch are
/// represented by omission; `game.id` is never emitted (its slug pattern would
/// reject raw names); top-level launch/engine are never emitted (the schema
/// forbids them on an export envelope, so they live only inside records).
fn record_for_game(game: &Game) -> Value {
    let mut record = Map::new();
    record.insert("fidelity".to_string(), json!(RECORD_FIDELITY));
    record.insert(
        "provenance".to_string(),
        json!({ "source": PROVENANCE_SOURCE }),
    );

    // game identity: app_id (string) + platform, plus name when known.
    let mut game_obj = Map::new();
    game_obj.insert("app_id".to_string(), json!(game.appid.to_string()));
    game_obj.insert("platform".to_string(), json!("steam"));
    if let Some(name) = &game.name {
        game_obj.insert("name".to_string(), json!(name));
    }
    record.insert("game".to_string(), Value::Object(game_obj));

    if !game.launch.is_empty() {
        let entries: Vec<Value> = game.launch.iter().map(launch_entry_value).collect();
        record.insert("launch".to_string(), Value::Array(entries));
    }

    if let Some(mediated) = game.launcher_mediated {
        record.insert("launcher_mediated".to_string(), json!(mediated));
    }

    if let Some(engine) = &game.engine {
        record.insert("engine".to_string(), engine_value(engine));
    }

    Value::Object(record)
}

fn launch_entry_value(entry: &LaunchEntry) -> Value {
    let mut obj = Map::new();
    obj.insert("executable".to_string(), json!(entry.executable()));
    insert_opt(&mut obj, "os", &entry.os);
    insert_opt(&mut obj, "osarch", &entry.osarch);
    insert_opt(&mut obj, "launch_type", &entry.launch_type);
    insert_opt(&mut obj, "beta_branch", &entry.beta_branch);
    insert_opt(&mut obj, "arguments", &entry.arguments);
    insert_opt(&mut obj, "description", &entry.description);
    Value::Object(obj)
}

fn engine_value(engine: &Engine) -> Value {
    let mut obj = Map::new();
    obj.insert("source".to_string(), json!(engine.source.as_str()));
    obj.insert("confidence".to_string(), json!(engine.confidence.as_str()));
    if let Some(name) = &engine.name {
        obj.insert("name".to_string(), json!(name));
    }
    Value::Object(obj)
}

/// Insert a string key only when the value is present; omission keeps the record
/// minimal and is what the schema expects for an absent optional filter.
fn insert_opt(obj: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(v) = value {
        obj.insert(key.to_string(), json!(v));
    }
}
