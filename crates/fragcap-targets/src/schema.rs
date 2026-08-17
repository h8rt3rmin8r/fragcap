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
//!
//! Version 3 (slice S051) adds the target entry model: a `targets` table and a
//! `target_id_aliases` table. Conceptually these live only in `local.db`, but the
//! store type is shared with `catalog.db` (the S050 split), so both files carry
//! the tables and `catalog.db` simply leaves them empty. The `targets` CHECK
//! constraints make the classification, classification-source, and fidelity enum
//! sets and the non-numeric-handle rule unstorable-if-violated, so the store
//! cannot hold a row the model would reject (P-9). The migration from version 2
//! is additive: two `CREATE TABLE`s, applied transactionally, leaving every
//! existing row untouched.
//!
//! Version 4 (slice S052) adds the `volume_eligibility` table: the persistent,
//! user-editable allowlist of fixed volumes the cross-volume known-roots walk may
//! enumerate. Like the `targets` tables it is conceptually `local.db` only; the
//! catalog leaves it empty. A row is keyed on a stable volume identity (the volume
//! GUID path, not the drive letter, which is reassignable), and the `reason` CHECK
//! set records why a volume is or is not eligible so each decision is statable
//! (FR-017). The migration from version 3 is one additive `CREATE TABLE`.
//!
//! Version 5 (slice S053) adds the `signature` table: the data-driven detection
//! signature set, moved out of the vendored ruleset that was compiled into
//! `fragcap-profile`. Unlike the `targets` and `volume_eligibility` tables it is a
//! `catalog.db` table (shipped, refreshable catalog data seeded like the `games`
//! rows); `local.db` leaves it empty. The `category`, `kind`, and `confidence`
//! CHECK sets make an out-of-vocabulary signature unstorable (P-9). The migration
//! from version 4 is one additive `CREATE TABLE`.

/// The schema version this build writes and understands.
pub const SCHEMA_VERSION: i64 = 5;

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

CREATE TABLE targets (
    id                    INTEGER PRIMARY KEY,
    stable_id             INTEGER NOT NULL UNIQUE,
    handle                TEXT NOT NULL UNIQUE
                            CHECK (length(handle) > 0 AND handle GLOB '*[^0-9]*'),
    name                  TEXT NOT NULL CHECK (length(name) > 0),
    classification        TEXT NOT NULL CHECK (classification IN
                            ('game', 'launcher', 'tool', 'mod', 'emulator', 'unknown')),
    classification_source TEXT NOT NULL CHECK (classification_source IN
                            ('catalog', 'engine-signature', 'platform', 'user', 'unset')),
    fidelity              TEXT NOT NULL CHECK (fidelity IN
                            ('authored', 'verified', 'heuristic-unverified', 'observed')),
    provenance            TEXT,
    anchor                TEXT,
    launch_entries        TEXT,
    install_root          TEXT,
    evidence              TEXT
);

CREATE TABLE target_id_aliases (
    alias_stable_id INTEGER PRIMARY KEY,
    target_id       INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE
);

CREATE TABLE volume_eligibility (
    volume_id   TEXT PRIMARY KEY CHECK (length(volume_id) > 0),
    mount_point TEXT,
    drive_type  TEXT,
    eligible    INTEGER NOT NULL CHECK (eligible IN (0, 1)),
    reason      TEXT NOT NULL CHECK (reason IN
                  ('seeded-first-run', 'user-added', 'user-excluded')),
    first_seen  TEXT
);

CREATE TABLE signature (
    id          INTEGER PRIMARY KEY,
    category    TEXT NOT NULL CHECK (category IN ('engine', 'anti-cheat', 'drm')),
    kind        TEXT NOT NULL CHECK (kind IN
                  ('filename', 'directory-shape', 'pe-version-string', 'binary-marker')),
    pattern     TEXT NOT NULL CHECK (length(pattern) > 0),
    product     TEXT NOT NULL CHECK (length(product) > 0),
    confidence  TEXT NOT NULL CHECK (confidence IN ('definitive', 'heuristic'))
);
";

/// The additive migration from schema version 4 to version 5: create the detection
/// signature table (slice S053). Backward-safe by construction, an existing v4 store
/// keeps every row and gains one empty table. Applied in one transaction alongside
/// the version stamp.
pub const MIGRATE_4_TO_5: &str = "\
CREATE TABLE signature (
    id          INTEGER PRIMARY KEY,
    category    TEXT NOT NULL CHECK (category IN ('engine', 'anti-cheat', 'drm')),
    kind        TEXT NOT NULL CHECK (kind IN
                  ('filename', 'directory-shape', 'pe-version-string', 'binary-marker')),
    pattern     TEXT NOT NULL CHECK (length(pattern) > 0),
    product     TEXT NOT NULL CHECK (length(product) > 0),
    confidence  TEXT NOT NULL CHECK (confidence IN ('definitive', 'heuristic'))
);
";

/// The additive migration from schema version 3 to version 4: create the volume
/// eligibility allowlist table (slice S052). Backward-safe by construction, an
/// existing v3 store keeps every row and gains one empty table. Applied in one
/// transaction alongside the version stamp.
pub const MIGRATE_3_TO_4: &str = "\
CREATE TABLE volume_eligibility (
    volume_id   TEXT PRIMARY KEY CHECK (length(volume_id) > 0),
    mount_point TEXT,
    drive_type  TEXT,
    eligible    INTEGER NOT NULL CHECK (eligible IN (0, 1)),
    reason      TEXT NOT NULL CHECK (reason IN
                  ('seeded-first-run', 'user-added', 'user-excluded')),
    first_seen  TEXT
);
";

/// The additive migration from schema version 2 to version 3: create the target
/// entry model's two tables (slice S051). Backward-safe by construction, an
/// existing v2 store keeps every row and gains two empty tables. Applied in one
/// transaction alongside the version stamp.
pub const MIGRATE_2_TO_3: &str = "\
CREATE TABLE targets (
    id                    INTEGER PRIMARY KEY,
    stable_id             INTEGER NOT NULL UNIQUE,
    handle                TEXT NOT NULL UNIQUE
                            CHECK (length(handle) > 0 AND handle GLOB '*[^0-9]*'),
    name                  TEXT NOT NULL CHECK (length(name) > 0),
    classification        TEXT NOT NULL CHECK (classification IN
                            ('game', 'launcher', 'tool', 'mod', 'emulator', 'unknown')),
    classification_source TEXT NOT NULL CHECK (classification_source IN
                            ('catalog', 'engine-signature', 'platform', 'user', 'unset')),
    fidelity              TEXT NOT NULL CHECK (fidelity IN
                            ('authored', 'verified', 'heuristic-unverified', 'observed')),
    provenance            TEXT,
    anchor                TEXT,
    launch_entries        TEXT,
    install_root          TEXT,
    evidence              TEXT
);

CREATE TABLE target_id_aliases (
    alias_stable_id INTEGER PRIMARY KEY,
    target_id       INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE
);
";

/// The additive migration from schema version 1 to version 2: add the nullable
/// `appinfo_change_number` column. Backward-safe by construction, an existing v1
/// store keeps every row and gains the column as NULL. Applied in one transaction
/// alongside the version stamp.
pub const MIGRATE_1_TO_2: &str = "ALTER TABLE games ADD COLUMN appinfo_change_number INTEGER;";
