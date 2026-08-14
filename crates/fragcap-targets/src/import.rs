// SPDX-License-Identifier: Apache-2.0

//! Loading a local seed document into the store, offline.
//!
//! The seed is a `kind: "export"` document (the same shape [`crate::export`]
//! produces). Import validates the whole document against the master schema
//! first, then extracts every record into a [`Game`], enforcing the
//! duplicate-appid rule, and writes the batch in one transaction. Nothing is
//! written until every record has parsed and validated, so a malformed seed
//! leaves the store untouched (P-4). An appid already in the store is replaced
//! wholesale, so re-importing the same seed is idempotent.

use std::collections::HashSet;

use fragcap_profile::jsonschema::validate_value;
use serde_json::Value;

use crate::model::{Engine, EngineConfidence, EngineSource, Game, LaunchEntry};
use crate::store::Store;
use crate::TargetsError;

/// What an import did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportSummary {
    /// How many games were inserted or replaced.
    pub imported: usize,
}

/// Import a seed document into the store.
///
/// Fails without writing if the seed is not valid JSON, does not validate
/// against the schema, is not an `export` envelope, or carries a duplicate appid.
pub fn import(store: &mut Store, seed_text: &str) -> Result<ImportSummary, TargetsError> {
    let value: Value = serde_json::from_str(seed_text)
        .map_err(|e| TargetsError::Seed(format!("not valid JSON: {e}")))?;

    // Structural validation first: a malformed record (bad engine source, a
    // launch entry missing its executable) is rejected here, before any write.
    let diagnostics = validate_value(&value);
    if !diagnostics.is_empty() {
        return Err(TargetsError::Seed(format!(
            "seed does not conform to the target schema:\n{diagnostics}"
        )));
    }

    if value.get("kind").and_then(Value::as_str) != Some("export") {
        return Err(TargetsError::Seed(
            "seed must be a kind: \"export\" document".to_string(),
        ));
    }

    let records = value
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| TargetsError::Seed("export envelope has no records array".to_string()))?;

    let mut seen: HashSet<u32> = HashSet::new();
    let mut games: Vec<Game> = Vec::with_capacity(records.len());
    for (i, record) in records.iter().enumerate() {
        let game = game_from_record(record, i)?;
        if !seen.insert(game.appid) {
            return Err(TargetsError::Seed(format!(
                "duplicate appid {} in seed document",
                game.appid
            )));
        }
        games.push(game);
    }

    store.upsert_all(&games)?;
    Ok(ImportSummary {
        imported: games.len(),
    })
}

/// Parse one schema `record` into a [`Game`]. The schema validation above
/// guarantees the shapes; this maps them and enforces that a record carries an
/// application id (the store's identity), which the schema does not require.
fn game_from_record(record: &Value, index: usize) -> Result<Game, TargetsError> {
    let game_obj = record.get("game");
    let app_id = game_obj
        .and_then(|g| g.get("app_id"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            TargetsError::Seed(format!("record {index} has no game.app_id to key on"))
        })?;
    let appid: u32 = app_id.parse().map_err(|_| {
        TargetsError::Seed(format!("record {index} app_id {app_id:?} is not a u32"))
    })?;

    let mut game = Game::new(appid);
    game.name = game_obj
        .and_then(|g| g.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);

    game.launcher_mediated = record.get("launcher_mediated").and_then(Value::as_bool);

    if let Some(entries) = record.get("launch").and_then(Value::as_array) {
        for entry in entries {
            game.launch.push(launch_entry_from(entry)?);
        }
    }

    if let Some(engine) = record.get("engine") {
        game.engine = Some(engine_from(engine)?);
    }

    Ok(game)
}

fn launch_entry_from(value: &Value) -> Result<LaunchEntry, TargetsError> {
    let executable = value
        .get("executable")
        .and_then(Value::as_str)
        .ok_or_else(|| TargetsError::Seed("launch entry has no executable".to_string()))?;
    let mut entry = LaunchEntry::new(executable)?;
    entry.os = opt_str(value, "os");
    entry.osarch = opt_str(value, "osarch");
    entry.launch_type = opt_str(value, "launch_type");
    entry.beta_branch = opt_str(value, "beta_branch");
    entry.arguments = opt_str(value, "arguments");
    entry.description = opt_str(value, "description");
    Ok(entry)
}

fn engine_from(value: &Value) -> Result<Engine, TargetsError> {
    let source = value
        .get("source")
        .and_then(Value::as_str)
        .ok_or_else(|| TargetsError::Seed("engine has no source".to_string()))?;
    let confidence = value
        .get("confidence")
        .and_then(Value::as_str)
        .ok_or_else(|| TargetsError::Seed("engine has no confidence".to_string()))?;
    Ok(Engine {
        name: opt_str(value, "name"),
        source: EngineSource::parse(source)?,
        confidence: EngineConfidence::parse(confidence)?,
    })
}

fn opt_str(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}
