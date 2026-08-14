// SPDX-License-Identifier: Apache-2.0

//! The embedded SQLite store.
//!
//! [`Store::open`] creates or opens a single database file, enables foreign
//! keys, and applies or verifies the version-1 migration. [`Store::upsert_game`]
//! writes one game and its launch and technology rows transactionally, replacing
//! any existing rows for that appid wholesale (delete then insert, via
//! `ON DELETE CASCADE`), so a re-import is idempotent and never a partial merge.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{
    Engine, EngineConfidence, EngineSource, Game, LaunchEntry, SeedState, SeedTier, TechCategory,
    Technology,
};
use crate::schema::{DDL, SCHEMA_VERSION};
use crate::TargetsError;

/// The hint store: a connection to one SQLite file (or an in-memory database for
/// tests), migrated to the current schema version.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open or create a store at `path`, enabling foreign keys and applying or
    /// verifying the version-1 migration.
    ///
    /// A file whose schema version is newer than this build understands is an
    /// error, not a silent operation on an incompatible layout.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, TargetsError> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    /// Open an in-memory store, migrated to the current version. For tests.
    pub fn open_in_memory() -> Result<Self, TargetsError> {
        let conn = Connection::open_in_memory()?;
        Self::from_connection(conn)
    }

    fn from_connection(mut conn: Connection) -> Result<Self, TargetsError> {
        // Set outside any transaction: `foreign_keys` is a no-op within one.
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match version {
            0 => {
                // Fresh database: apply the schema and stamp its version inside
                // one transaction, so a failure partway through the DDL rolls the
                // whole thing back rather than leaving a half-created file whose
                // user_version is still zero (which the next open would treat as
                // fresh and then fail on the tables that do exist).
                let tx = conn.transaction()?;
                tx.execute_batch(DDL)?;
                tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
                tx.commit()?;
            }
            v if v == SCHEMA_VERSION => {}
            found => return Err(TargetsError::SchemaVersion { found }),
        }
        Ok(Store { conn })
    }

    /// Insert or wholesale-replace one game and its launch and technology rows,
    /// transactionally. Rejects an out-of-set enum or an empty executable before
    /// writing; both are unrepresentable in the model, and the SQLite CHECK
    /// constraints are the backstop.
    pub fn upsert_game(&mut self, game: &Game) -> Result<(), TargetsError> {
        let tx = self.conn.transaction()?;
        write_game(&tx, game)?;
        tx.commit()?;
        Ok(())
    }

    /// Insert or wholesale-replace many games in one transaction: either every
    /// game lands or none does. The importer uses this so a failure partway
    /// through a batch leaves the store untouched (P-4).
    pub fn upsert_all(&mut self, games: &[Game]) -> Result<(), TargetsError> {
        let tx = self.conn.transaction()?;
        for game in games {
            write_game(&tx, game)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Merge one title's Tier 1 (public catalog) columns: insert the row if the
    /// appid is new, or update only `name` and the catalog metrics if it exists.
    ///
    /// This is deliberately not [`Store::upsert_game`]: the catalog seeder knows
    /// only appid, name, and metrics, and the whole-game replace would erase any
    /// launch entries (Tier 2) and engine attribution (Tier 3) a later seeder had
    /// written. `merge_catalog` leaves `launcher_mediated`, `token_required`, the
    /// engine columns, and the launch and technology rows untouched. An empty or
    /// absent name is stored as NULL, never `""` (the S034 guard).
    pub fn merge_catalog(
        &mut self,
        appid: u32,
        name: Option<&str>,
        review_count: Option<u64>,
        owners: Option<u64>,
        peak_ccu: Option<u64>,
    ) -> Result<(), TargetsError> {
        let name = name.filter(|s| !s.is_empty());
        self.conn.execute(
            "INSERT INTO games (appid, name, review_count, owners, peak_ccu)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(appid) DO UPDATE SET
                name         = excluded.name,
                review_count = excluded.review_count,
                owners       = excluded.owners,
                peak_ccu     = excluded.peak_ccu",
            params![
                appid,
                name,
                review_count.map(|v| v as i64),
                owners.map(|v| v as i64),
                peak_ccu.map(|v| v as i64),
            ],
        )?;
        Ok(())
    }

    /// Read every game, each with its launch entries (ordered) and technologies
    /// (ordered), sorted by appid.
    pub fn games(&self) -> Result<Vec<Game>, TargetsError> {
        let mut games = self.load_games()?;
        for game in &mut games {
            game.launch = self.load_launch(game.appid)?;
            game.technologies = self.load_technologies(game.appid)?;
        }
        Ok(games)
    }

    fn load_games(&self) -> Result<Vec<Game>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT appid, name, review_count, owners, peak_ccu,
                    launcher_mediated, token_required,
                    engine_name, engine_source, engine_confidence
             FROM games ORDER BY appid",
        )?;
        let rows = stmt.query_map([], |row| {
            let engine_source: Option<String> = row.get(8)?;
            let engine_confidence: Option<String> = row.get(9)?;
            let engine_name: Option<String> = row.get(7)?;
            Ok(GameRow {
                appid: row.get::<_, i64>(0)? as u32,
                name: row.get(1)?,
                review_count: row.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                owners: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                peak_ccu: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                launcher_mediated: row.get(5)?,
                token_required: row.get(6)?,
                engine_name,
                engine_source,
                engine_confidence,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            let r = row?;
            let engine = match (r.engine_source, r.engine_confidence) {
                (Some(s), Some(c)) => Some(Engine {
                    name: r.engine_name,
                    source: EngineSource::parse(&s)?,
                    confidence: EngineConfidence::parse(&c)?,
                }),
                // The both-or-neither CHECK makes a half-present engine
                // unstorable; treat any residue as absent rather than guessing.
                _ => None,
            };
            out.push(Game {
                appid: r.appid,
                name: r.name,
                review_count: r.review_count,
                owners: r.owners,
                peak_ccu: r.peak_ccu,
                launcher_mediated: r.launcher_mediated,
                token_required: r.token_required,
                engine,
                launch: Vec::new(),
                technologies: Vec::new(),
            });
        }
        Ok(out)
    }

    fn load_launch(&self, appid: u32) -> Result<Vec<LaunchEntry>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT os, osarch, launch_type, beta_branch, executable, arguments, description
             FROM launch_entries WHERE appid = ?1 ORDER BY launch_index",
        )?;
        let rows = stmt.query_map(params![appid], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (os, osarch, launch_type, beta_branch, executable, arguments, description) = row?;
            let mut entry = LaunchEntry::new(executable)?;
            entry.os = os;
            entry.osarch = osarch;
            entry.launch_type = launch_type;
            entry.beta_branch = beta_branch;
            entry.arguments = arguments;
            entry.description = description;
            out.push(entry);
        }
        Ok(out)
    }

    fn load_technologies(&self, appid: u32) -> Result<Vec<Technology>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT category, name, marker_path
             FROM technologies WHERE appid = ?1 ORDER BY tech_index",
        )?;
        let rows = stmt.query_map(params![appid], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (category, name, marker_path) = row?;
            out.push(Technology {
                category: TechCategory::parse(&category)?,
                name,
                marker_path,
            });
        }
        Ok(out)
    }

    /// Read the seeding state for one tier, if any has been recorded.
    pub fn seed_state(&self, tier: SeedTier) -> Result<Option<SeedState>, TargetsError> {
        let row = self
            .conn
            .query_row(
                "SELECT last_run_at, resume_cursor FROM seed_state WHERE tier = ?1",
                params![tier.as_str()],
                |row| {
                    Ok(SeedState {
                        tier,
                        last_run_at: row.get(0)?,
                        resume_cursor: row.get(1)?,
                    })
                },
            )
            .optional()?;
        Ok(row)
    }

    /// Record the seeding state for one tier, replacing any prior value.
    pub fn set_seed_state(&mut self, state: &SeedState) -> Result<(), TargetsError> {
        self.conn.execute(
            "INSERT INTO seed_state (tier, last_run_at, resume_cursor)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(tier) DO UPDATE SET
                last_run_at = excluded.last_run_at,
                resume_cursor = excluded.resume_cursor",
            params![state.tier.as_str(), state.last_run_at, state.resume_cursor],
        )?;
        Ok(())
    }
}

/// Write one game and its launch and technology rows onto an open connection or
/// transaction, replacing any existing rows for that appid wholesale. Shared by
/// [`Store::upsert_game`] (its own transaction) and [`Store::upsert_all`] (a
/// batch transaction).
fn write_game(conn: &Connection, game: &Game) -> Result<(), TargetsError> {
    // A present-but-empty name would store cleanly yet export as `game.name: ""`,
    // which the schema (minLength 1) rejects: the store would then hold a row it
    // could not export. Refuse it here, before any write, with the CHECK
    // constraint on `games.name` as the backstop. An absent name (None) is fine.
    if game.name.as_deref() == Some("") {
        return Err(TargetsError::Model(
            "game name must not be empty (use None for an unknown name)".to_string(),
        ));
    }

    // Wholesale replace: delete the existing game; its launch and technology
    // rows follow through ON DELETE CASCADE.
    conn.execute("DELETE FROM games WHERE appid = ?1", params![game.appid])?;

    let (engine_name, engine_source, engine_confidence) = match &game.engine {
        Some(Engine {
            name,
            source,
            confidence,
        }) => (
            name.clone(),
            Some(source.as_str()),
            Some(confidence.as_str()),
        ),
        None => (None, None, None),
    };

    conn.execute(
        "INSERT INTO games
            (appid, name, review_count, owners, peak_ccu,
             launcher_mediated, token_required,
             engine_name, engine_source, engine_confidence)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            game.appid,
            game.name,
            game.review_count.map(|v| v as i64),
            game.owners.map(|v| v as i64),
            game.peak_ccu.map(|v| v as i64),
            game.launcher_mediated,
            game.token_required,
            engine_name,
            engine_source,
            engine_confidence,
        ],
    )?;

    for (i, entry) in game.launch.iter().enumerate() {
        conn.execute(
            "INSERT INTO launch_entries
                (appid, launch_index, os, osarch, launch_type, beta_branch,
                 executable, arguments, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                game.appid,
                i as i64,
                entry.os,
                entry.osarch,
                entry.launch_type,
                entry.beta_branch,
                entry.executable(),
                entry.arguments,
                entry.description,
            ],
        )?;
    }

    for (i, tech) in game.technologies.iter().enumerate() {
        conn.execute(
            "INSERT INTO technologies (appid, tech_index, category, name, marker_path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                game.appid,
                i as i64,
                tech.category.as_str(),
                tech.name,
                tech.marker_path,
            ],
        )?;
    }

    Ok(())
}

/// A raw `games` row before its engine columns are folded into an `Engine`.
struct GameRow {
    appid: u32,
    name: Option<String>,
    review_count: Option<u64>,
    owners: Option<u64>,
    peak_ccu: Option<u64>,
    launcher_mediated: Option<bool>,
    token_required: Option<bool>,
    engine_name: Option<String>,
    engine_source: Option<String>,
    engine_confidence: Option<String>,
}
