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

use crate::entry::{ClassificationSource, DetectionScan, TargetClassification, TargetEntry};
use crate::model::{
    Engine, EngineConfidence, EngineSource, Game, LaunchEntry, SeedState, SeedTier, TechCategory,
    Technology,
};
use crate::schema::{
    DDL, MIGRATE_1_TO_2, MIGRATE_2_TO_3, MIGRATE_3_TO_4, MIGRATE_4_TO_5, MIGRATE_5_TO_6,
    MIGRATE_6_TO_7, MIGRATE_7_TO_8, SCHEMA_VERSION,
};
use crate::volume::{EligibilityReason, Volume, VolumeEligibility};
use crate::TargetsError;
use fragcap_profile::{
    FidelityTier, Signature, SignatureCategory, SignatureConfidence, SignatureKind,
};

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
        let mut version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        // Fresh database: apply the whole schema and stamp its version inside one
        // transaction, so a failure partway through the DDL rolls the whole thing
        // back rather than leaving a half-created file whose user_version is still
        // zero (which the next open would treat as fresh and then fail on the
        // tables that do exist).
        if version == 0 {
            let tx = conn.transaction()?;
            tx.execute_batch(DDL)?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            tx.commit()?;
            version = SCHEMA_VERSION;
        }

        // Additive forward migrations, applied in sequence so a v1 store steps
        // through 1 -> 2 -> 3. Each step is its own transaction stamping the
        // intermediate version, so an interrupted upgrade leaves a store whose
        // tables and user_version agree at some version this build can resume from.
        if version == 1 {
            // 1 -> 2: add the nullable appinfo_change_number column (slice S038).
            // Existing rows read it as NULL and refresh on the first walk.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_1_TO_2)?;
            tx.pragma_update(None, "user_version", 2i64)?;
            tx.commit()?;
            version = 2;
        }
        if version == 2 {
            // 2 -> 3: create the target entry model's two tables (slice S051).
            // Existing catalog rows are untouched; the new tables start empty.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_2_TO_3)?;
            tx.pragma_update(None, "user_version", 3i64)?;
            tx.commit()?;
            version = 3;
        }
        if version == 3 {
            // 3 -> 4: create the volume eligibility allowlist table (slice S052).
            // Existing rows are untouched; the new table starts empty.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_3_TO_4)?;
            tx.pragma_update(None, "user_version", 4i64)?;
            tx.commit()?;
            version = 4;
        }
        if version == 4 {
            // 4 -> 5: create the detection signature table (slice S053). Existing
            // rows are untouched; the new table starts empty.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_4_TO_5)?;
            tx.pragma_update(None, "user_version", 5i64)?;
            tx.commit()?;
            version = 5;
        }
        if version == 5 {
            // 5 -> 6: create the listing snapshot table (slice S055). Existing rows
            // are untouched; the new table starts empty.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_5_TO_6)?;
            tx.pragma_update(None, "user_version", 6i64)?;
            tx.commit()?;
            version = 6;
        }

        if version == 6 {
            // 6 -> 7: add the nullable detection_scan column to targets (slice
            // S065). Existing rows read it as NULL, which is the "no scan recorded"
            // state, so nothing is backfilled and no row changes meaning.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_6_TO_7)?;
            tx.pragma_update(None, "user_version", 7i64)?;
            tx.commit()?;
            version = 7;
        }

        if version == 7 {
            // 7 -> 8: add the nullable folder_name and executable_hint columns to
            // targets (slice S066). Existing rows read both as NULL, which is the
            // "not recorded" state, so nothing is backfilled.
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATE_7_TO_8)?;
            tx.pragma_update(None, "user_version", 8i64)?;
            tx.commit()?;
            version = 8;
        }

        if version != SCHEMA_VERSION {
            // A file whose schema version is newer than this build understands is
            // an error, not a silent operation on an incompatible layout.
            return Err(TargetsError::SchemaVersion { found: version });
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

    /// Merge one title's Tier 3 (engine attribution) columns: insert an engine-only
    /// row if the appid is new, or update only the engine columns if it exists.
    ///
    /// This is the engine analogue of [`Store::merge_catalog`]: the engine seeder
    /// knows only appid and an engine, and neither the whole-game replace
    /// ([`Store::upsert_game`]) nor `merge_catalog` (which never writes engine) fits.
    /// `merge_engine` leaves `name`, the catalog metrics, `launcher_mediated`,
    /// `token_required`, and the launch and technology rows untouched, so an engine
    /// enrichment over a catalog-seeded, launch-bearing game preserves Tiers 1 and 2.
    ///
    /// `engine.source` and `engine.confidence` are bound together (both non-optional
    /// on [`Engine`]), satisfying the both-or-neither CHECK by construction;
    /// `engine.name` binds NULL when absent. An insert for an unseen appid creates a
    /// row whose other columns are NULL, which is schema-valid on export.
    pub fn merge_engine(&mut self, appid: u32, engine: &Engine) -> Result<(), TargetsError> {
        self.conn.execute(
            "INSERT INTO games (appid, engine_name, engine_source, engine_confidence)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(appid) DO UPDATE SET
                engine_name       = excluded.engine_name,
                engine_source     = excluded.engine_source,
                engine_confidence = excluded.engine_confidence",
            params![
                appid,
                engine.name.as_deref().filter(|s| !s.is_empty()),
                engine.source.as_str(),
                engine.confidence.as_str(),
            ],
        )?;
        Ok(())
    }

    /// Merge one title's Tier 2 (launch) columns from the local Steam appinfo
    /// cache: record the appinfo change-number and replace this appid's launch
    /// entries, leaving every other tier untouched.
    ///
    /// This is the launch analogue of [`Store::merge_catalog`] and
    /// [`Store::merge_engine`]: the accumulator knows only an appid, its appinfo
    /// change-number, and its launch entries. It ensures a row exists (inserting an
    /// appid-only row if the appid is new, so the `launch_entries` foreign key
    /// holds), sets `appinfo_change_number`, and rewrites the launch entries in
    /// order. It leaves `name`, the catalog metrics, the engine columns,
    /// `launcher_mediated`, and `token_required` exactly as they were, so a launch
    /// enrichment over a catalog-seeded or engine-seeded game preserves Tiers 1 and
    /// 3 (slice S038). All in one transaction: either the change-number and the new
    /// entries both land, or neither does.
    pub fn merge_launch(
        &mut self,
        appid: u32,
        change_number: u32,
        entries: &[LaunchEntry],
    ) -> Result<(), TargetsError> {
        let tx = self.conn.transaction()?;
        // Ensure the row exists without disturbing any existing column. An unseen
        // appid becomes an appid-only row (every other column NULL, schema-valid on
        // export); an existing row is left as it is.
        tx.execute(
            "INSERT INTO games (appid) VALUES (?1) ON CONFLICT(appid) DO NOTHING",
            params![appid],
        )?;
        tx.execute(
            "UPDATE games SET appinfo_change_number = ?2 WHERE appid = ?1",
            params![appid, change_number as i64],
        )?;
        // Wholesale replace this appid's launch entries; the change-number and the
        // entries always move together.
        tx.execute(
            "DELETE FROM launch_entries WHERE appid = ?1",
            params![appid],
        )?;
        for (i, entry) in entries.iter().enumerate() {
            tx.execute(
                "INSERT INTO launch_entries
                    (appid, launch_index, os, osarch, launch_type, beta_branch,
                     executable, arguments, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    appid,
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
        tx.commit()?;
        Ok(())
    }

    /// The appinfo change-number recorded the last time launch data was stored for
    /// this application, or `None` when no row exists or the application was never
    /// learned from appinfo. The staleness comparison input for accumulation
    /// (slice S038): a cache change-number greater than this means the launch data
    /// is stale and must be refreshed.
    pub fn stored_change_number(&self, appid: u32) -> Result<Option<u32>, TargetsError> {
        let value: Option<Option<i64>> = self
            .conn
            .query_row(
                "SELECT appinfo_change_number FROM games WHERE appid = ?1",
                params![appid],
                |row| row.get(0),
            )
            .optional()?;
        // Outer None: no row. Inner None: row exists but column is NULL. Both mean
        // "never learned from appinfo", so both collapse to None.
        Ok(value.flatten().map(|v| v as u32))
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

    /// Read one game by application id, with its launch entries (ordered) and
    /// technologies (ordered), or `None` when no row exists.
    ///
    /// The single-row path the hint provider (S037) resolves through: it reads the
    /// one row it was asked about rather than loading the whole corpus. An absent
    /// row is `Ok(None)`, not an error; a read that fails after the store opened is
    /// an error the caller surfaces.
    pub fn game(&self, appid: u32) -> Result<Option<Game>, TargetsError> {
        let Some(mut game) = self.load_game(appid)? else {
            return Ok(None);
        };
        game.launch = self.load_launch(appid)?;
        game.technologies = self.load_technologies(appid)?;
        Ok(Some(game))
    }

    fn load_game(&self, appid: u32) -> Result<Option<Game>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT appid, name, review_count, owners, peak_ccu,
                    launcher_mediated, token_required,
                    engine_name, engine_source, engine_confidence
             FROM games WHERE appid = ?1",
        )?;
        let mut rows = stmt.query(params![appid])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let engine_source: Option<String> = row.get(8)?;
        let engine_confidence: Option<String> = row.get(9)?;
        let engine_name: Option<String> = row.get(7)?;
        let r = GameRow {
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
        };
        Ok(Some(game_from_row(r)?))
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
            out.push(game_from_row(row?)?);
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

    // -- Target entry model (slice S051) ----------------------------------------

    /// Whether a handle is already taken in the `targets` table.
    pub fn handle_exists(&self, handle: &str) -> Result<bool, TargetsError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM targets WHERE handle = ?1",
            params![handle],
            |row| row.get(0),
        )?;
        Ok(n > 0)
    }

    /// Insert a target entry, returning its assigned row id.
    ///
    /// This is the low-level insert; the merge-on-anchor guarantee is provided by
    /// the caller checking [`Store::target_by_anchor`] first (the CLI does), so a
    /// re-registration reports the existing target rather than reaching here. If a
    /// duplicate identity or handle does reach the insert, the UNIQUE violation is
    /// surfaced as a clear [`TargetsError::Model`] rather than a raw SQLite error;
    /// a CHECK violation (an out-of-set enum, an empty name, a purely numeric
    /// handle) is likewise a model error, so an invalid entry is a named storage
    /// failure rather than a silently stored lie (P-9).
    pub fn insert_target(&mut self, entry: &TargetEntry) -> Result<i64, TargetsError> {
        let result = self.conn.execute(
            "INSERT INTO targets
                (stable_id, handle, name, classification, classification_source,
                 fidelity, provenance, anchor, launch_entries, install_root, evidence,
                 detection_scan, folder_name, executable_hint)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                entry.stable_id,
                entry.handle,
                entry.name,
                entry.classification.as_str(),
                entry.classification_source.as_str(),
                entry.fidelity.as_str(),
                entry.provenance.as_ref().map(json_text),
                entry.anchor,
                entry.launch_entries.as_ref().map(json_text),
                entry.install_root,
                entry.evidence.as_ref().map(json_text),
                entry.detection_scan.map(|d| d.as_str()),
                entry.folder_name,
                entry.executable_hint,
            ],
        );
        match result {
            Ok(_) => Ok(self.conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(e, msg))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                let detail = msg.unwrap_or_else(|| "constraint violation".to_string());
                if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE {
                    Err(TargetsError::Model(format!(
                        "a target with this identity or handle already exists \
                         (stable_id {}, handle {:?}): {detail}",
                        entry.stable_id, entry.handle
                    )))
                } else {
                    Err(TargetsError::Model(format!("invalid target: {detail}")))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Read a target by its row id, or `None`.
    pub fn target(&self, id: i64) -> Result<Option<TargetEntry>, TargetsError> {
        self.conn
            .query_row(
                "SELECT id, stable_id, handle, name, classification,
                        classification_source, fidelity, provenance, anchor,
                        launch_entries, install_root, evidence, detection_scan,
                        folder_name, executable_hint
                 FROM targets WHERE id = ?1",
                params![id],
                read_target_row,
            )
            .optional()?
            .transpose_targets()
    }

    /// Every target, ordered by row id, for the ephemeral row-index selector and
    /// listing.
    pub fn targets(&self) -> Result<Vec<TargetEntry>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, stable_id, handle, name, classification,
                    classification_source, fidelity, provenance, anchor,
                    launch_entries, install_root, evidence, detection_scan,
                        folder_name, executable_hint
             FROM targets ORDER BY id",
        )?;
        let rows = stmt.query_map([], read_target_row)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r??);
        }
        Ok(out)
    }

    /// Resolve a target by exact handle, or `None`.
    pub fn target_by_handle(&self, handle: &str) -> Result<Option<TargetEntry>, TargetsError> {
        self.conn
            .query_row(
                "SELECT id, stable_id, handle, name, classification,
                        classification_source, fidelity, provenance, anchor,
                        launch_entries, install_root, evidence, detection_scan,
                        folder_name, executable_hint
                 FROM targets WHERE handle = ?1",
                params![handle],
                read_target_row,
            )
            .optional()?
            .transpose_targets()
    }

    /// Resolve targets by case-insensitive exact name. More than one match is an
    /// ambiguity the caller must refuse to guess (P-9); zero is a clean no-match.
    ///
    /// The fold is Unicode-aware (Rust's `to_lowercase`), not SQLite's `NOCASE`
    /// collation, which folds ASCII only: a stored `Élan` must match a selector
    /// `élan`, and two names differing only in non-ASCII case must be detected as
    /// an ambiguity rather than slip past it. The target table is small (a user's
    /// registered titles), so folding every row in memory is inexpensive.
    pub fn targets_by_name(&self, name: &str) -> Result<Vec<TargetEntry>, TargetsError> {
        let needle = name.to_lowercase();
        Ok(self
            .targets()?
            .into_iter()
            .filter(|t| t.name.to_lowercase() == needle)
            .collect())
    }

    /// Resolve targets whose `name`, `folder_name`, or `executable_hint` contains
    /// `needle`, case-insensitively (slice S066, issue #173). Each matching target
    /// is returned once even when it matches on more than one field. This is the
    /// selector's third, lowest-priority tier: it is consulted only when the exact
    /// handle and exact name tiers both miss, so an existing exact match's
    /// resolution is unaffected by this method's existence.
    ///
    /// The same Unicode-aware fold as [`Store::targets_by_name`], for the same
    /// reason: a stored `Ivy` must match a selector `ivy` regardless of script.
    pub fn targets_by_substring(&self, needle: &str) -> Result<Vec<TargetEntry>, TargetsError> {
        let needle = needle.to_lowercase();
        if needle.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self
            .targets()?
            .into_iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&needle)
                    || t.folder_name
                        .as_deref()
                        .is_some_and(|f| f.to_lowercase().contains(&needle))
                    || t.executable_hint
                        .as_deref()
                        .is_some_and(|e| e.to_lowercase().contains(&needle))
            })
            .collect())
    }

    /// Resolve a target by its stable identifier, consulting superseded aliases so
    /// a reference to a former (unanchored) id still lands on the merged entry.
    pub fn target_by_stable_id(&self, stable_id: i64) -> Result<Option<TargetEntry>, TargetsError> {
        if let Some(t) = self
            .conn
            .query_row(
                "SELECT id, stable_id, handle, name, classification,
                        classification_source, fidelity, provenance, anchor,
                        launch_entries, install_root, evidence, detection_scan,
                        folder_name, executable_hint
                 FROM targets WHERE stable_id = ?1",
                params![stable_id],
                read_target_row,
            )
            .optional()?
            .transpose_targets()?
        {
            return Ok(Some(t));
        }
        // Not an active id: try the alias table.
        let target_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT target_id FROM target_id_aliases WHERE alias_stable_id = ?1",
                params![stable_id],
                |row| row.get(0),
            )
            .optional()?;
        match target_id {
            Some(id) => self.target(id),
            None => Ok(None),
        }
    }

    /// Look up an existing target by its anchor's stable identifier, for merge.
    pub fn target_by_anchor(&self, anchor: &str) -> Result<Option<TargetEntry>, TargetsError> {
        let stable_id = crate::identifier::anchored_id(anchor);
        self.target_by_stable_id(stable_id)
    }

    /// Promote an unanchored target to an anchor: adopt the anchored stable id and
    /// retain the former value as a superseded alias (never reissued). Idempotent
    /// on the alias insert.
    pub fn supersede_with_anchor(
        &mut self,
        target_id: i64,
        anchor: &str,
    ) -> Result<i64, TargetsError> {
        let old: i64 = self.conn.query_row(
            "SELECT stable_id FROM targets WHERE id = ?1",
            params![target_id],
            |row| row.get(0),
        )?;
        let canonical = crate::identifier::canonicalize_anchor(anchor);
        let new = crate::identifier::anchored_id(&canonical);

        // If another target already owns this anchored identity, this is a
        // collision with an existing registration, not a supersede: report it
        // clearly before the transaction rather than failing on a raw UNIQUE error.
        if let Some(existing) = self.target_by_stable_id(new)? {
            if existing.id != Some(target_id) {
                return Err(TargetsError::Model(format!(
                    "anchor {canonical:?} is already registered to target {} (handle {})",
                    existing.stable_id, existing.handle
                )));
            }
        }

        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE targets SET stable_id = ?1, anchor = ?2 WHERE id = ?3",
            params![new, canonical, target_id],
        )?;
        // Keep the former id resolvable. INSERT OR IGNORE so re-running is a no-op.
        tx.execute(
            "INSERT OR IGNORE INTO target_id_aliases (alias_stable_id, target_id)
             VALUES (?1, ?2)",
            params![old, target_id],
        )?;
        tx.commit()?;
        Ok(new)
    }

    // --- Target mutation and listing snapshot (slice S055) ---------------------

    /// Delete one target by row id. Returns whether a row was removed. The
    /// `target_id_aliases` rows for this target cascade via the ON DELETE CASCADE
    /// foreign key, so a former (superseded) id stops resolving with the entry.
    pub fn delete_target(&mut self, id: i64) -> Result<bool, TargetsError> {
        let removed = self
            .conn
            .execute("DELETE FROM targets WHERE id = ?1", params![id])?;
        Ok(removed > 0)
    }

    /// Overwrite the mutable fields of the target whose `stable_id` matches
    /// `entry.stable_id`. Used by import to merge an incoming record onto an
    /// existing row in place (identity preserved, no duplicate). The handle,
    /// classification, fidelity, and the JSON-carried fields are replaced; a
    /// CHECK violation is a named model error (P-9). Returns whether a row matched.
    pub fn update_target(&mut self, entry: &TargetEntry) -> Result<bool, TargetsError> {
        let result = self.conn.execute(
            "UPDATE targets SET
                handle = ?2, name = ?3, classification = ?4,
                classification_source = ?5, fidelity = ?6, provenance = ?7,
                anchor = ?8, launch_entries = ?9, install_root = ?10, evidence = ?11,
                detection_scan = ?12, folder_name = ?13, executable_hint = ?14
             WHERE stable_id = ?1",
            params![
                entry.stable_id,
                entry.handle,
                entry.name,
                entry.classification.as_str(),
                entry.classification_source.as_str(),
                entry.fidelity.as_str(),
                entry.provenance.as_ref().map(json_text),
                entry.anchor,
                entry.launch_entries.as_ref().map(json_text),
                entry.install_root,
                entry.evidence.as_ref().map(json_text),
                entry.detection_scan.map(|d| d.as_str()),
                entry.folder_name,
                entry.executable_hint,
            ],
        );
        match result {
            Ok(rows) => Ok(rows > 0),
            Err(rusqlite::Error::SqliteFailure(e, msg))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                let detail = msg.unwrap_or_else(|| "constraint violation".to_string());
                Err(TargetsError::Model(format!(
                    "invalid target update: {detail}"
                )))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Merge a batch of imported targets in one transaction: each element updates
    /// the row whose `stable_id` matches in place, or inserts a new row. This is
    /// all-or-nothing (FR-019): if any element violates a constraint (for example a
    /// new stable id whose handle collides with an existing row) the whole batch
    /// rolls back, so a partially applied import cannot happen. Returns the count of
    /// rows inserted and updated.
    pub fn merge_targets(&mut self, entries: &[TargetEntry]) -> Result<(u64, u64), TargetsError> {
        let tx = self.conn.transaction()?;
        let mut inserted = 0u64;
        let mut updated = 0u64;
        for entry in entries {
            let exists = tx
                .query_row(
                    "SELECT 1 FROM targets WHERE stable_id = ?1",
                    params![entry.stable_id],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
            // Both statements bind ?1..?12 in the same order, so one parameter list
            // serves both. The JSON renderings are bound to locals so they outlive
            // the `params!` borrow.
            let provenance = entry.provenance.as_ref().map(json_text);
            let launch_entries = entry.launch_entries.as_ref().map(json_text);
            let evidence = entry.evidence.as_ref().map(json_text);
            let bound = params![
                entry.stable_id,
                entry.handle,
                entry.name,
                entry.classification.as_str(),
                entry.classification_source.as_str(),
                entry.fidelity.as_str(),
                provenance,
                entry.anchor,
                launch_entries,
                entry.install_root,
                evidence,
                entry.detection_scan.map(|d| d.as_str()),
                entry.folder_name,
                entry.executable_hint,
            ];
            let result = if exists {
                tx.execute(
                    "UPDATE targets SET
                        handle = ?2, name = ?3, classification = ?4,
                        classification_source = ?5, fidelity = ?6, provenance = ?7,
                        anchor = ?8, launch_entries = ?9, install_root = ?10, evidence = ?11,
                        detection_scan = ?12, folder_name = ?13, executable_hint = ?14
                     WHERE stable_id = ?1",
                    bound,
                )
            } else {
                tx.execute(
                    "INSERT INTO targets
                        (stable_id, handle, name, classification, classification_source,
                         fidelity, provenance, anchor, launch_entries, install_root, evidence,
                         detection_scan, folder_name, executable_hint)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    bound,
                )
            };
            match result {
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(e, msg))
                    if e.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    let detail = msg.unwrap_or_else(|| "constraint violation".to_string());
                    return Err(TargetsError::Model(format!(
                        "import element (stable_id {}, handle {:?}) violates a constraint, \
                         nothing was applied: {detail}",
                        entry.stable_id, entry.handle
                    )));
                }
                Err(e) => return Err(e.into()),
            }
            if exists {
                updated += 1;
            } else {
                inserted += 1;
            }
        }
        tx.commit()?;
        Ok((inserted, updated))
    }

    /// Promote a target after a capture observed its real socket-holding process:
    /// rewrite its `launch_entries` to the resolved chain and raise its `fidelity`.
    /// The caller passes only an observed result; nothing here fabricates a holder
    /// (P-9). Returns whether the row was found.
    pub fn promote_target_launch(
        &mut self,
        id: i64,
        launch_entries: &serde_json::Value,
        fidelity: FidelityTier,
    ) -> Result<bool, TargetsError> {
        let rows = self.conn.execute(
            "UPDATE targets SET launch_entries = ?2, fidelity = ?3 WHERE id = ?1",
            params![id, json_text(launch_entries), fidelity.as_str()],
        )?;
        Ok(rows > 0)
    }

    /// Replace the listing snapshot with the ordered rows a listing just displayed.
    /// Each row is `(stable_id, handle)`; positions are assigned 1..n in the given
    /// order. A row-index selector resolves against this so a number names the row
    /// the user saw (slice S055). Transactional: the old snapshot is cleared and the
    /// new one written atomically.
    pub fn write_listing_snapshot(&mut self, rows: &[(i64, &str)]) -> Result<(), TargetsError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM listing_snapshot", [])?;
        for (index, (stable_id, handle)) in rows.iter().enumerate() {
            tx.execute(
                "INSERT INTO listing_snapshot (position, stable_id, handle)
                 VALUES (?1, ?2, ?3)",
                params![(index as i64) + 1, stable_id, handle],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// The stable id at a 1-based position in the most recent listing snapshot, or
    /// `None` if the position is out of range (or no listing has been written). The
    /// row-index branch of selector resolution reads this.
    pub fn listing_snapshot_nth(&self, position: usize) -> Result<Option<i64>, TargetsError> {
        let stable_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT stable_id FROM listing_snapshot WHERE position = ?1",
                params![position as i64],
                |row| row.get(0),
            )
            .optional()?;
        Ok(stable_id)
    }

    /// How many rows the most recent listing snapshot holds, or zero if no
    /// listing has been written yet.
    ///
    /// Read only to report a row-index miss. A numeric selector is resolved
    /// unconditionally as a row index (see `selector::is_row_index`), so when it
    /// misses, the size of the space it was resolved against is the fact the
    /// operator needs and the one the resolver already knows: issue #181 records
    /// an operator being told only "no target matches" after passing a Steam app
    /// id that was read as row 1333350 of a 33-row listing.
    pub fn listing_snapshot_len(&self) -> Result<usize, TargetsError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM listing_snapshot", [], |row| {
                row.get(0)
            })?;
        Ok(n.max(0) as usize)
    }

    // --- Volume eligibility (slice S052) ---------------------------------------

    /// Whether the volume eligibility allowlist has ever been seeded. Empty means
    /// no first run has happened yet, which is what [`Store::seed_volume_eligibility`]
    /// keys the one-time permissive seeding on (FR-016a).
    pub fn volume_eligibility_is_empty(&self) -> Result<bool, TargetsError> {
        let count: i64 =
            self.conn
                .query_row("SELECT count(*) FROM volume_eligibility", [], |row| {
                    row.get(0)
                })?;
        Ok(count == 0)
    }

    /// Permissively seed the allowlist with the given fixed volumes, once. If the
    /// table already holds any row this is a no-op (the seeding is first-run only),
    /// so a later-appearing volume is never auto-added (FR-016a). Each seeded volume
    /// is recorded eligible with reason `seeded-first-run`.
    pub fn seed_volume_eligibility(&mut self, volumes: &[Volume]) -> Result<(), TargetsError> {
        if !self.volume_eligibility_is_empty()? {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        for v in volumes {
            tx.execute(
                "INSERT OR IGNORE INTO volume_eligibility
                    (volume_id, mount_point, drive_type, eligible, reason, first_seen)
                 VALUES (?1, ?2, ?3, 1, 'seeded-first-run', datetime('now'))",
                params![v.identity, v.mount_point, v.drive_type.as_str()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Record an explicit eligibility decision for one volume: a user opt-in
    /// (`eligible = true`, reason `user-added`) or a user exclusion
    /// (`eligible = false`, reason `user-excluded`). Upserts by volume identity.
    pub fn set_volume_eligibility(
        &mut self,
        volume: &Volume,
        eligible: bool,
        reason: EligibilityReason,
    ) -> Result<(), TargetsError> {
        self.conn.execute(
            "INSERT INTO volume_eligibility
                (volume_id, mount_point, drive_type, eligible, reason, first_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
             ON CONFLICT(volume_id) DO UPDATE SET
                mount_point = excluded.mount_point,
                drive_type  = excluded.drive_type,
                eligible    = excluded.eligible,
                reason      = excluded.reason",
            params![
                volume.identity,
                volume.mount_point,
                volume.drive_type.as_str(),
                eligible as i64,
                reason.as_str(),
            ],
        )?;
        Ok(())
    }

    /// The recorded eligibility decision for one volume identity, or `None` if the
    /// volume is unseen (not in the allowlist, hence not walked).
    pub fn volume_eligibility(
        &self,
        volume_id: &str,
    ) -> Result<Option<VolumeEligibility>, TargetsError> {
        let row = self
            .conn
            .query_row(
                "SELECT volume_id, mount_point, drive_type, eligible, reason, first_seen
                 FROM volume_eligibility WHERE volume_id = ?1",
                params![volume_id],
                read_eligibility_row,
            )
            .optional()?;
        row.transpose_eligibility()
    }

    /// Every volume recorded eligible. A volume absent from the result is either
    /// recorded ineligible or unseen; either way the walk does not touch it.
    pub fn eligible_volumes(&self) -> Result<Vec<VolumeEligibility>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT volume_id, mount_point, drive_type, eligible, reason, first_seen
             FROM volume_eligibility WHERE eligible = 1 ORDER BY volume_id",
        )?;
        let rows = stmt.query_map([], read_eligibility_row)?;
        let mut out = Vec::new();
        for r in rows {
            // `r?` unwraps the SQLite read; the inner `?` surfaces a reason-parse
            // failure as a TargetsError rather than dropping the row.
            out.push(r??);
        }
        Ok(out)
    }

    /// Replace the detection signature table with `signatures`, transactionally
    /// (slice S053). Idempotent: re-seeding the same set yields the same table, so a
    /// catalog refresh through the seed path leaves no stale rows. The table is a
    /// `catalog.db` table; seeding it on a `local.db` is allowed but pointless (the
    /// matcher reads the catalog).
    pub fn seed_signatures(&mut self, signatures: &[Signature]) -> Result<(), TargetsError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM signature", [])?;
        for s in signatures {
            tx.execute(
                "INSERT INTO signature (category, kind, pattern, product, confidence)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    s.category.as_str(),
                    s.kind.as_str(),
                    s.pattern,
                    s.product,
                    s.confidence.as_str(),
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Load every detection signature row (slice S053). Returns the raw signatures;
    /// the caller compiles them into a [`fragcap_profile::SignatureSet`], which is
    /// where the applied/inert/skipped partition and its P-4 surfacing live. A row
    /// whose stored enum text is somehow out of vocabulary (the CHECK constraints
    /// make it unstorable) is surfaced as an error, never dropped.
    pub fn load_signatures(&self) -> Result<Vec<Signature>, TargetsError> {
        let mut stmt = self.conn.prepare(
            "SELECT category, kind, pattern, product, confidence
             FROM signature ORDER BY id",
        )?;
        let rows = stmt.query_map([], read_signature_row)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r??);
        }
        Ok(out)
    }
}

/// Serialize a carried-whole JSON value to its stored text.
fn json_text(v: &serde_json::Value) -> String {
    v.to_string()
}

/// Read one `signature` row into a [`Signature`]. Enum parsing that can fail is
/// deferred so the rusqlite closure stays infallible in the column reads; the CHECK
/// constraints make an out-of-vocabulary value unstorable, so a parse failure means
/// a corrupt file, surfaced rather than dropped (P-4).
fn read_signature_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<Result<Signature, TargetsError>> {
    let category_text: String = row.get(0)?;
    let kind_text: String = row.get(1)?;
    let pattern: String = row.get(2)?;
    let product: String = row.get(3)?;
    let confidence_text: String = row.get(4)?;
    let parsed = (|| {
        let category = SignatureCategory::parse(&category_text).ok_or_else(|| {
            TargetsError::Model(format!("unknown signature category {category_text:?}"))
        })?;
        let kind = SignatureKind::parse(&kind_text)
            .ok_or_else(|| TargetsError::Model(format!("unknown signature kind {kind_text:?}")))?;
        let confidence = SignatureConfidence::parse(&confidence_text).ok_or_else(|| {
            TargetsError::Model(format!("unknown signature confidence {confidence_text:?}"))
        })?;
        Ok(Signature {
            category,
            kind,
            pattern,
            product,
            confidence,
        })
    })();
    Ok(parsed)
}

/// Read one `volume_eligibility` row. Reason parsing that can fail is deferred so
/// the rusqlite closure stays infallible in the column reads.
fn read_eligibility_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<Result<VolumeEligibility, TargetsError>> {
    let volume_id: String = row.get(0)?;
    let mount_point: Option<String> = row.get(1)?;
    let drive_type: Option<String> = row.get(2)?;
    let eligible: i64 = row.get(3)?;
    let reason_text: String = row.get(4)?;
    let first_seen: Option<String> = row.get(5)?;
    let parsed = match EligibilityReason::parse(&reason_text) {
        Some(reason) => Ok(VolumeEligibility {
            volume_id,
            mount_point,
            drive_type,
            eligible: eligible != 0,
            reason,
            first_seen,
        }),
        None => Err(TargetsError::Model(format!(
            "unknown eligibility reason {reason_text:?}"
        ))),
    };
    Ok(parsed)
}

/// Transpose the nested `Option<Result<..>>` an eligibility read produces, matching
/// the pattern the target reads use.
trait TransposeEligibility {
    fn transpose_eligibility(self) -> Result<Option<VolumeEligibility>, TargetsError>;
}

impl TransposeEligibility for Option<Result<VolumeEligibility, TargetsError>> {
    fn transpose_eligibility(self) -> Result<Option<VolumeEligibility>, TargetsError> {
        match self {
            Some(Ok(e)) => Ok(Some(e)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

/// Read one `targets` row into a [`TargetEntry`]. Enum and JSON parsing that can
/// fail is deferred to [`TransposeTargets`] so the rusqlite closure stays
/// infallible in the column reads.
fn read_target_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<TargetEntry, TargetsError>> {
    let id: i64 = row.get(0)?;
    let stable_id: i64 = row.get(1)?;
    let handle: String = row.get(2)?;
    let name: String = row.get(3)?;
    let classification: String = row.get(4)?;
    let classification_source: String = row.get(5)?;
    let fidelity: String = row.get(6)?;
    let provenance: Option<String> = row.get(7)?;
    let anchor: Option<String> = row.get(8)?;
    let launch_entries: Option<String> = row.get(9)?;
    let install_root: Option<String> = row.get(10)?;
    let evidence: Option<String> = row.get(11)?;
    let detection_scan: Option<String> = row.get(12)?;
    let folder_name: Option<String> = row.get(13)?;
    let executable_hint: Option<String> = row.get(14)?;

    Ok((|| {
        Ok(TargetEntry {
            id: Some(id),
            stable_id,
            handle,
            name,
            classification: TargetClassification::parse(&classification)?,
            classification_source: ClassificationSource::parse(&classification_source)?,
            fidelity: FidelityTier::parse(&fidelity)
                .ok_or_else(|| TargetsError::Model(format!("unknown fidelity {fidelity:?}")))?,
            provenance: parse_json_opt(provenance)?,
            anchor,
            launch_entries: parse_json_opt(launch_entries)?,
            install_root,
            evidence: parse_json_opt(evidence)?,
            // An out-of-set stored value is an error, never read permissively: the
            // CHECK constraint makes it unwritable by this build, so encountering
            // one means the file was written by something else and its coverage
            // claim cannot be trusted (P-9).
            detection_scan: detection_scan
                .as_deref()
                .map(DetectionScan::parse)
                .transpose()?,
            folder_name,
            executable_hint,
        })
    })())
}

/// Parse an optional stored JSON text back to a value, mapping a parse failure to
/// a model error rather than panicking.
fn parse_json_opt(text: Option<String>) -> Result<Option<serde_json::Value>, TargetsError> {
    match text {
        None => Ok(None),
        Some(s) => serde_json::from_str(&s)
            .map(Some)
            .map_err(|e| TargetsError::Model(format!("stored JSON is invalid: {e}"))),
    }
}

/// Flatten a `query_row`/`optional` result whose row closure itself returns a
/// fallible parse, so callers get `Result<Option<TargetEntry>, TargetsError>`.
trait TransposeTargets {
    fn transpose_targets(self) -> Result<Option<TargetEntry>, TargetsError>;
}

impl TransposeTargets for Option<Result<TargetEntry, TargetsError>> {
    fn transpose_targets(self) -> Result<Option<TargetEntry>, TargetsError> {
        match self {
            None => Ok(None),
            Some(Ok(t)) => Ok(Some(t)),
            Some(Err(e)) => Err(e),
        }
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

/// Fold a raw `games` row into a `Game`, reconstructing its `Engine` from the
/// both-or-neither source/confidence columns. The launch and technology rows are
/// loaded separately; this leaves them empty.
fn game_from_row(r: GameRow) -> Result<Game, TargetsError> {
    let engine = match (r.engine_source, r.engine_confidence) {
        (Some(s), Some(c)) => Some(Engine {
            name: r.engine_name,
            source: EngineSource::parse(&s)?,
            confidence: EngineConfidence::parse(&c)?,
        }),
        // The both-or-neither CHECK makes a half-present engine unstorable; treat
        // any residue as absent rather than guessing.
        _ => None,
    };
    Ok(Game {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Engine, EngineConfidence, EngineSource, Game, LaunchEntry};
    use crate::schema::DDL;

    /// Fabricate a schema-version-1 store on an in-memory connection: apply the
    /// current DDL, then undo every change made after version 1 (drop the version-2
    /// column, the version-3 target tables, and the version-4 eligibility table) and
    /// stamp user_version 1. Returns the raw connection so a test can hand it to
    /// `from_connection` and observe the forward migration step through
    /// 1 -> 2 -> 3 -> 4.
    fn v1_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory");
        conn.execute_batch(DDL).expect("apply DDL");
        conn.execute_batch("ALTER TABLE games DROP COLUMN appinfo_change_number;")
            .expect("drop column to simulate v1");
        conn.execute_batch("DROP TABLE target_id_aliases; DROP TABLE targets;")
            .expect("drop v3 tables to simulate v1");
        conn.execute_batch("DROP TABLE volume_eligibility;")
            .expect("drop v4 table to simulate v1");
        conn.execute_batch("DROP TABLE signature;")
            .expect("drop v5 table to simulate v1");
        conn.execute_batch("DROP TABLE listing_snapshot;")
            .expect("drop v6 table to simulate v1");
        conn.pragma_update(None, "user_version", 1_i64)
            .expect("stamp v1");
        conn
    }

    #[test]
    fn opens_and_migrates_a_v1_store_backward_safely() {
        let conn = v1_connection();
        // A row written under v1, with no notion of the new column.
        conn.execute(
            "INSERT INTO games (appid, name, review_count) VALUES (730, 'CS2', 1234)",
            [],
        )
        .expect("insert v1 row");

        // Opening migrates 1 -> 2 in place, preserving the row.
        let mut store = Store::from_connection(conn).expect("migrate");
        assert_eq!(
            store.stored_change_number(730).expect("read"),
            None,
            "an existing row gets the new column as NULL"
        );
        let g = store.game(730).expect("read").expect("present");
        assert_eq!(g.name.as_deref(), Some("CS2"));
        assert_eq!(g.review_count, Some(1234));

        // And the store is fully usable at v2 afterward.
        store
            .merge_launch(730, 42, &[LaunchEntry::new("cs2.exe").expect("exe")])
            .expect("merge");
        assert_eq!(store.stored_change_number(730).expect("read"), Some(42));
        assert_eq!(g.name.as_deref(), Some("CS2"));
    }

    /// Build a target entry for tests with the given handle, anchor, and fidelity.
    fn sample_target(handle: &str, anchor: Option<&str>, fidelity: FidelityTier) -> TargetEntry {
        let stable_id = match anchor {
            Some(a) => crate::identifier::anchored_id(a),
            None => crate::identifier::unanchored_id(),
        };
        TargetEntry {
            id: None,
            stable_id,
            handle: handle.to_string(),
            name: handle.to_string(),
            classification: TargetClassification::Game,
            classification_source: ClassificationSource::User,
            fidelity,
            provenance: None,
            anchor: anchor.map(|a| a.to_string()),
            launch_entries: Some(serde_json::json!([{ "executable": "game.exe" }])),
            install_root: None,
            evidence: None,
            detection_scan: None,
            folder_name: None,
            executable_hint: None,
        }
    }

    #[test]
    fn a_fresh_store_is_at_the_current_version_with_empty_tables() {
        let store = Store::open_in_memory().expect("store");
        let version: i64 = store
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .expect("version");
        assert_eq!(
            version, SCHEMA_VERSION,
            "a store this build wrote is stamped at the version this build declares"
        );
        assert!(store.targets().expect("targets").is_empty());
        assert!(store
            .volume_eligibility_is_empty()
            .expect("eligibility empty"));
        assert!(store.load_signatures().expect("signatures").is_empty());
        assert!(
            store.listing_snapshot_nth(1).expect("snapshot").is_none(),
            "fresh store has no listing snapshot"
        );
    }

    #[test]
    fn a_v6_store_gains_the_coverage_column_and_reads_it_as_no_scan_recorded() {
        // The migration is additive: an existing row keeps every value and reads the
        // new column as NULL, which is exactly "no scan is recorded". Nothing is
        // backfilled, because backfilling would invent a scan that never ran (P-9).
        let store = Store::open_in_memory().expect("store");
        let conn = store.conn;
        // Roll the file back to v6 by dropping every column added after it (and the
        // version stamp), the shape a store written by the v6 build actually has.
        conn.execute_batch(
            "ALTER TABLE targets DROP COLUMN detection_scan;
             ALTER TABLE targets DROP COLUMN folder_name;
             ALTER TABLE targets DROP COLUMN executable_hint;",
        )
        .expect("drop columns");
        conn.pragma_update(None, "user_version", 6i64)
            .expect("stamp v6");
        conn.execute(
            "INSERT INTO targets
                (stable_id, handle, name, classification, classification_source, fidelity)
             VALUES (11, 'older_row', 'Older Row', 'game', 'user', 'authored')",
            [],
        )
        .expect("insert a pre-migration row");

        let store = Store::from_connection(conn).expect("migrate forward");
        let version: i64 = store
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .expect("version");
        assert_eq!(version, SCHEMA_VERSION);

        let targets = store.targets().expect("targets");
        assert_eq!(targets.len(), 1, "the existing row survives");
        assert_eq!(targets[0].handle, "older_row");
        assert_eq!(
            targets[0].detection_scan, None,
            "a row written before the column reads as no scan recorded, never as clean"
        );
    }

    #[test]
    fn a_v7_store_gains_the_two_new_columns_and_reads_them_as_not_recorded() {
        // The migration is additive: an existing row keeps every value and reads
        // both new columns as NULL, exactly "not recorded". Nothing is backfilled.
        let store = Store::open_in_memory().expect("store");
        let conn = store.conn;
        conn.execute_batch(
            "ALTER TABLE targets DROP COLUMN folder_name;
             ALTER TABLE targets DROP COLUMN executable_hint;",
        )
        .expect("drop columns");
        conn.pragma_update(None, "user_version", 7i64)
            .expect("stamp v7");
        conn.execute(
            "INSERT INTO targets
                (stable_id, handle, name, classification, classification_source, fidelity)
             VALUES (12, 'older_row', 'Older Row', 'game', 'user', 'authored')",
            [],
        )
        .expect("insert a pre-migration row");

        let store = Store::from_connection(conn).expect("migrate forward");
        let version: i64 = store
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .expect("version");
        assert_eq!(version, SCHEMA_VERSION);

        let targets = store.targets().expect("targets");
        assert_eq!(targets.len(), 1, "the existing row survives");
        assert_eq!(targets[0].folder_name, None);
        assert_eq!(targets[0].executable_hint, None);
    }

    #[test]
    fn folder_name_and_executable_hint_round_trip() {
        let mut store = Store::open_in_memory().expect("store");
        let mut entry = sample_target("t1", None, FidelityTier::Observed);
        entry.folder_name = Some("Escape from Ivy & Piper".to_string());
        entry.executable_hint = Some("TrappedWithIvyAndPiper-EA.exe".to_string());
        store.insert_target(&entry).expect("insert");

        let read_back = store
            .target_by_handle("t1")
            .expect("query")
            .expect("present");
        assert_eq!(
            read_back.folder_name.as_deref(),
            Some("Escape from Ivy & Piper")
        );
        assert_eq!(
            read_back.executable_hint.as_deref(),
            Some("TrappedWithIvyAndPiper-EA.exe")
        );
    }

    #[test]
    fn update_target_writes_folder_name_and_executable_hint_too() {
        // Review of PR #193 (Copilot): update_target's UPDATE previously stopped
        // at detection_scan, so any caller reaching a row through it (including an
        // import-style merge onto an existing stable_id) silently lost both new
        // fields even though insert_target and merge_targets already carried them.
        let mut store = Store::open_in_memory().expect("store");
        let mut entry = sample_target("t2", None, FidelityTier::Observed);
        store.insert_target(&entry).expect("insert");

        entry.folder_name = Some("Escape from Ivy & Piper".to_string());
        entry.executable_hint = Some("TrappedWithIvyAndPiper-EA.exe".to_string());
        assert!(store.update_target(&entry).expect("update"));

        let read_back = store
            .target_by_handle("t2")
            .expect("query")
            .expect("present");
        assert_eq!(
            read_back.folder_name.as_deref(),
            Some("Escape from Ivy & Piper")
        );
        assert_eq!(
            read_back.executable_hint.as_deref(),
            Some("TrappedWithIvyAndPiper-EA.exe")
        );
    }

    #[test]
    fn the_coverage_column_round_trips_and_refuses_an_out_of_set_value() {
        let mut store = Store::open_in_memory().expect("store");
        let mut entry = sample_target("scanned", Some("steam:1"), FidelityTier::Authored);
        entry.detection_scan = Some(DetectionScan::Incomplete);
        store.insert_target(&entry).expect("insert");
        let read = store
            .target_by_handle("scanned")
            .expect("read")
            .expect("present");
        assert_eq!(read.detection_scan, Some(DetectionScan::Incomplete));

        // The CHECK set makes an out-of-vocabulary coverage claim unstorable, so the
        // listing cannot render a state the store never wrote (P-9).
        let err = store
            .conn
            .execute(
                "UPDATE targets SET detection_scan = 'probably' WHERE handle = 'scanned'",
                [],
            )
            .expect_err("an out-of-set coverage value is refused");
        assert!(
            format!("{err}").to_lowercase().contains("constraint"),
            "refused by the CHECK constraint: {err}"
        );
    }

    #[test]
    fn a_v1_store_migrates_to_the_current_version_with_catalog_rows_intact() {
        let conn = v1_connection();
        conn.execute(
            "INSERT INTO games (appid, name, review_count) VALUES (730, 'CS2', 1234)",
            [],
        )
        .expect("insert v1 row");
        let store = Store::from_connection(conn).expect("migrate forward");
        let version: i64 = store
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .expect("version");
        assert_eq!(
            version, SCHEMA_VERSION,
            "a store this build wrote is stamped at the version this build declares"
        );
        assert!(store.targets().expect("targets").is_empty());
        assert!(store
            .volume_eligibility_is_empty()
            .expect("eligibility empty"));
        assert!(store.load_signatures().expect("signatures").is_empty());
        assert!(
            store.game(730).expect("read").is_some(),
            "catalog row survives"
        );
    }

    #[test]
    fn listing_snapshot_round_trips_and_replaces() {
        let mut store = Store::open_in_memory().expect("store");
        store
            .write_listing_snapshot(&[(1001, "alpha"), (1002, "bravo"), (1003, "charlie")])
            .expect("write snapshot");
        assert_eq!(store.listing_snapshot_nth(1).expect("nth"), Some(1001));
        assert_eq!(store.listing_snapshot_nth(3).expect("nth"), Some(1003));
        assert_eq!(store.listing_snapshot_nth(4).expect("nth"), None);
        assert_eq!(store.listing_snapshot_nth(0).expect("nth"), None);
        // A new listing replaces the prior snapshot wholesale.
        store
            .write_listing_snapshot(&[(2001, "delta")])
            .expect("rewrite snapshot");
        assert_eq!(store.listing_snapshot_nth(1).expect("nth"), Some(2001));
        assert_eq!(store.listing_snapshot_nth(2).expect("nth"), None);
    }

    #[test]
    fn promote_target_launch_resolves_the_chain_and_raises_fidelity() {
        use crate::authoring::{launch_entries_for, launch_is_unresolved, resolved_client_launch};
        use crate::readiness::{capture_readiness, CaptureReadiness};
        use crate::SocketHolderAnswer;

        let mut store = Store::open_in_memory().expect("store");
        // An unsure-authored target: unresolved chain, needs a target.
        let mut entry = sample_target("some_game", None, FidelityTier::Authored);
        entry.launch_entries = Some(launch_entries_for(SocketHolderAnswer::Unsure, "game.exe"));
        let id = store.insert_target(&entry).expect("insert");
        let before = store.target(id).expect("read").expect("present");
        assert!(launch_is_unresolved(&before));
        assert_eq!(capture_readiness(&before), CaptureReadiness::NeedsTarget);

        // A capture observed the real holder: promote.
        let resolved = resolved_client_launch("game.exe");
        assert!(store
            .promote_target_launch(id, &resolved, FidelityTier::Verified)
            .expect("promote"));

        let after = store.target(id).expect("read").expect("present");
        assert_eq!(after.fidelity, FidelityTier::Verified);
        assert!(!launch_is_unresolved(&after));
        assert_eq!(capture_readiness(&after), CaptureReadiness::Ready);
    }

    #[test]
    fn merge_targets_is_all_or_nothing_on_a_constraint_violation() {
        let mut store = Store::open_in_memory().expect("store");
        store
            .insert_target(&sample_target(
                "existing",
                Some("steam:1"),
                FidelityTier::Authored,
            ))
            .expect("insert baseline");

        // A batch whose first element is a clean insert and whose second reuses the
        // existing handle under a new stable id: the whole batch must roll back.
        let good = sample_target("fresh", Some("steam:2"), FidelityTier::Authored);
        let mut collides = sample_target("existing", Some("steam:3"), FidelityTier::Authored);
        collides.stable_id = crate::identifier::anchored_id("steam:3");
        let batch = vec![good, collides];

        assert!(
            store.merge_targets(&batch).is_err(),
            "handle collision fails the batch"
        );
        // Nothing from the batch persisted: only the baseline row remains.
        let handles: Vec<_> = store
            .targets()
            .expect("targets")
            .into_iter()
            .map(|t| t.handle)
            .collect();
        assert_eq!(
            handles,
            vec!["existing".to_string()],
            "the batch rolled back"
        );
    }

    #[test]
    fn delete_target_removes_the_row_and_reports_it() {
        let mut store = Store::open_in_memory().expect("store");
        let entry = sample_target("some_game", Some("steam:42"), FidelityTier::Authored);
        let id = store.insert_target(&entry).expect("insert");
        assert!(store.delete_target(id).expect("delete"));
        assert!(store.target(id).expect("read").is_none());
        assert!(
            !store.delete_target(id).expect("delete again"),
            "second delete is a no-op"
        );
    }

    #[test]
    fn seed_signatures_round_trips_and_is_idempotent() {
        let mut store = Store::open_in_memory().expect("store");
        let sigs = vec![
            Signature {
                category: SignatureCategory::Engine,
                kind: SignatureKind::Filename,
                pattern: "UnityPlayer.dll".to_string(),
                product: "Unity".to_string(),
                confidence: SignatureConfidence::Definitive,
            },
            Signature {
                category: SignatureCategory::Drm,
                kind: SignatureKind::BinaryMarker,
                pattern: "denuvo-marker".to_string(),
                product: "Denuvo".to_string(),
                confidence: SignatureConfidence::Definitive,
            },
        ];
        store.seed_signatures(&sigs).expect("seed");
        assert_eq!(store.load_signatures().expect("load"), sigs);

        // Re-seeding the same set yields the same table (no duplication, no stale
        // rows): a catalog refresh through the seed path is idempotent.
        store.seed_signatures(&sigs).expect("re-seed");
        assert_eq!(store.load_signatures().expect("load"), sigs);

        // Re-seeding a different set replaces wholesale.
        let replacement = vec![Signature {
            category: SignatureCategory::AntiCheat,
            kind: SignatureKind::Filename,
            pattern: "vgk.sys".to_string(),
            product: "Vanguard".to_string(),
            confidence: SignatureConfidence::Definitive,
        }];
        store.seed_signatures(&replacement).expect("replace");
        assert_eq!(store.load_signatures().expect("load"), replacement);
    }

    #[test]
    fn insert_and_read_a_target_round_trips() {
        let mut store = Store::open_in_memory().expect("store");
        let mut t = sample_target("portal_2", Some("steam:620"), FidelityTier::Authored);
        let id = store.insert_target(&t).expect("insert");
        t.id = Some(id);
        assert!(store.handle_exists("portal_2").expect("exists"));
        let read = store.target(id).expect("read").expect("present");
        assert_eq!(read, t);
        assert_eq!(
            store
                .target_by_handle("portal_2")
                .expect("by handle")
                .unwrap()
                .id,
            Some(id)
        );
    }

    #[test]
    fn a_purely_numeric_handle_is_rejected_by_the_store() {
        let mut store = Store::open_in_memory().expect("store");
        let t = sample_target("2048", None, FidelityTier::Observed);
        assert!(
            store.insert_target(&t).is_err(),
            "the handle GLOB CHECK rejects a purely numeric handle"
        );
    }

    #[test]
    fn supersede_adopts_the_anchor_and_keeps_the_old_id_resolvable() {
        let mut store = Store::open_in_memory().expect("store");
        let t = sample_target("my_game", None, FidelityTier::Observed);
        let old_id = t.stable_id;
        let row = store.insert_target(&t).expect("insert");
        let new_id = store
            .supersede_with_anchor(row, "steam:2221490")
            .expect("supersede");
        assert_eq!(new_id, crate::identifier::anchored_id("steam:2221490"));
        // The active id resolves.
        assert_eq!(
            store
                .target_by_stable_id(new_id)
                .expect("active")
                .unwrap()
                .id,
            Some(row)
        );
        // The old id still resolves via the alias table.
        assert_eq!(
            store
                .target_by_stable_id(old_id)
                .expect("alias")
                .unwrap()
                .id,
            Some(row)
        );
    }

    #[test]
    fn name_lookup_is_case_insensitive_and_reports_all_matches() {
        let mut store = Store::open_in_memory().expect("store");
        let mut a = sample_target("portal_2", Some("steam:620"), FidelityTier::Authored);
        a.name = "Portal 2".to_string();
        let mut b = sample_target("portal_2_2", Some("steam:200"), FidelityTier::Observed);
        b.name = "Portal 2".to_string();
        store.insert_target(&a).expect("a");
        store.insert_target(&b).expect("b");
        let matches = store.targets_by_name("portal 2").expect("by name");
        assert_eq!(matches.len(), 2, "both same-named targets are returned");
    }

    #[test]
    fn merge_launch_preserves_tier_1_and_tier_3() {
        let mut store = Store::open_in_memory().expect("store");
        let mut g = Game::new(620);
        g.name = Some("Portal 2".to_string());
        g.review_count = Some(1000);
        g.launcher_mediated = Some(true);
        g.token_required = Some(false);
        g.engine = Some(Engine {
            name: Some("Source".to_string()),
            source: EngineSource::Pcgamingwiki,
            confidence: EngineConfidence::High,
        });
        store.upsert_game(&g).expect("seed");

        store
            .merge_launch(620, 7, &[LaunchEntry::new("portal2.exe").expect("exe")])
            .expect("merge launch");

        let read = store.game(620).expect("read").expect("present");
        assert_eq!(read.name.as_deref(), Some("Portal 2"));
        assert_eq!(read.review_count, Some(1000));
        assert_eq!(read.launcher_mediated, Some(true));
        assert_eq!(read.token_required, Some(false));
        assert_eq!(
            read.engine.as_ref().unwrap().name.as_deref(),
            Some("Source")
        );
        assert_eq!(read.launch.len(), 1);
        assert_eq!(read.launch[0].executable(), "portal2.exe");
        assert_eq!(store.stored_change_number(620).expect("read"), Some(7));
    }

    #[test]
    fn stored_change_number_reads_and_defaults_to_none() {
        let mut store = Store::open_in_memory().expect("store");
        assert_eq!(store.stored_change_number(1).expect("read"), None);

        let mut g = Game::new(1);
        g.name = Some("X".to_string());
        store.upsert_game(&g).expect("seed");
        assert_eq!(
            store.stored_change_number(1).expect("read"),
            None,
            "a row with no appinfo learned reads None"
        );

        store
            .merge_launch(1, 99, &[LaunchEntry::new("x.exe").expect("exe")])
            .expect("merge");
        assert_eq!(store.stored_change_number(1).expect("read"), Some(99));
    }

    #[test]
    fn merge_launch_inserts_an_appid_only_row_for_an_unseen_app() {
        let mut store = Store::open_in_memory().expect("store");
        store
            .merge_launch(555, 3, &[LaunchEntry::new("g.exe").expect("exe")])
            .expect("merge");
        let g = store.game(555).expect("read").expect("present");
        assert!(g.name.is_none());
        assert!(g.engine.is_none());
        assert_eq!(g.launch.len(), 1);
        assert_eq!(store.stored_change_number(555).expect("read"), Some(3));
    }

    #[test]
    fn game_reads_one_row_with_its_launch_entries() {
        let mut store = Store::open_in_memory().expect("in-memory store");
        let mut game = Game::new(730);
        game.name = Some("CS2".to_string());
        game.launcher_mediated = Some(true);
        game.launch = vec![LaunchEntry::new("cs2.exe").expect("non-empty")];
        store.upsert_game(&game).expect("upsert");

        let read = store.game(730).expect("read").expect("present");
        assert_eq!(read.appid, 730);
        assert_eq!(read.name.as_deref(), Some("CS2"));
        assert_eq!(read.launcher_mediated, Some(true));
        assert_eq!(read.launch.len(), 1);
        assert_eq!(read.launch[0].executable(), "cs2.exe");
    }

    #[test]
    fn game_returns_none_for_an_absent_row() {
        let store = Store::open_in_memory().expect("in-memory store");
        assert!(store.game(12345).expect("read").is_none());
    }
}
