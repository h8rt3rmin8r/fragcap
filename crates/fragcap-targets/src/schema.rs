// SPDX-License-Identifier: Apache-2.0

//! The embedded SQLite schema and its single migration.
//!
//! Version 1. The CHECK constraints make an invalid row impossible to store: the
//! engine enum sets, the engine both-or-neither invariant, a non-empty
//! executable, boolean columns restricted to 0/1, and the technology category
//! set. A store that cannot hold an invalid row cannot export one either.

/// The schema version this build writes and understands.
pub const SCHEMA_VERSION: i64 = 1;

/// The complete DDL for schema version 1, applied inside one transaction to a
/// fresh store.
pub const DDL: &str = "\
CREATE TABLE games (
    appid              INTEGER PRIMARY KEY,
    name               TEXT,
    review_count       INTEGER,
    owners             INTEGER,
    peak_ccu           INTEGER,
    launcher_mediated  INTEGER CHECK (launcher_mediated IN (0, 1)),
    token_required     INTEGER CHECK (token_required IN (0, 1)),
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
