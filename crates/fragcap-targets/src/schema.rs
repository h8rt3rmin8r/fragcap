// SPDX-License-Identifier: Apache-2.0

//! The embedded SQLite schema and its migrations.
//!
//! Version 2. The CHECK constraints make an invalid row impossible to store: the
//! engine enum sets, the engine both-or-neither invariant, a non-empty
//! executable, boolean columns restricted to 0/1, and the technology category
//! set. A store that cannot hold an invalid row cannot export one either.
//!
//! Version 2 adds one nullable column, `games.appinfo_change_number`, the
//! per-application Steam appinfo change-number that the local launch-data
//! accumulation (slice S038) compares to decide staleness. It is store-internal
//! bookkeeping: never exported, never surfaced on the [`crate::model::Game`], so
//! the export projection and the published schema are unchanged by it. The
//! migration from version 1 is a single additive `ALTER TABLE`, applied in
//! [`crate::store::Store::open`]; existing rows get NULL and refresh on the first
//! walk.

/// The schema version this build writes and understands.
pub const SCHEMA_VERSION: i64 = 2;

/// The complete DDL for the current schema version, applied inside one
/// transaction to a fresh store.
pub const DDL: &str = "\
CREATE TABLE games (
    appid              INTEGER PRIMARY KEY,
    name               TEXT CHECK (name IS NULL OR length(name) > 0),
    review_count       INTEGER,
    owners             INTEGER,
    peak_ccu           INTEGER,
    launcher_mediated  INTEGER CHECK (launcher_mediated IN (0, 1)),
    token_required     INTEGER CHECK (token_required IN (0, 1)),
    appinfo_change_number INTEGER,
    engine_name        TEXT,
    engine_source      TEXT CHECK (engine_source IN
                          ('pcgamingwiki', 'exe_heuristic', 'depot_filename_rules')),
    engine_confidence  TEXT CHECK (engine_confidence IN
                          ('confirmed', 'high', 'medium', 'low', 'unknown')),
    CHECK ((engine_source IS NULL) = (engine_confidence IS NULL))
);

CREATE TABLE launch_entries (
    appid         INTEGER NOT NULL REFERENCES games(appid) ON DELETE CASCADE,
    launch_index  INTEGER NOT NULL,
    os            TEXT,
    osarch        TEXT,
    launch_type   TEXT,
    beta_branch   TEXT,
    executable    TEXT NOT NULL CHECK (length(executable) > 0),
    arguments     TEXT,
    description   TEXT,
    PRIMARY KEY (appid, launch_index)
);

CREATE TABLE technologies (
    appid       INTEGER NOT NULL REFERENCES games(appid) ON DELETE CASCADE,
    tech_index  INTEGER NOT NULL,
    category    TEXT NOT NULL CHECK (category IN
                  ('engine', 'anti_cheat', 'sdk', 'framework',
                   'emulator', 'container', 'runtime', 'launcher')),
    name        TEXT NOT NULL CHECK (length(name) > 0),
    marker_path TEXT,
    PRIMARY KEY (appid, tech_index)
);

CREATE TABLE seed_state (
    tier          TEXT PRIMARY KEY CHECK (tier IN ('catalog', 'launch', 'engine')),
    last_run_at   TEXT,
    resume_cursor TEXT
);
";

/// The additive migration from schema version 1 to version 2: add the nullable
/// `appinfo_change_number` column. Backward-safe by construction, an existing v1
/// store keeps every row and gains the column as NULL. Applied in one transaction
/// alongside the version stamp.
pub const MIGRATE_1_TO_2: &str = "ALTER TABLE games ADD COLUMN appinfo_change_number INTEGER;";
