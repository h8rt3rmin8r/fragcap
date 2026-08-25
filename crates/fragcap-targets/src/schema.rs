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

//! Version 7 (slice S065) adds the nullable `detection_scan` column to `targets`:
//! whether the row's install directory was scanned and whether that scan was
//! complete. NULL means no scan is recorded, which is what every pre-S065 row and
//! every row from a source that ran no detection carries. The CHECK set makes an
//! out-of-vocabulary coverage claim unstorable, so the listing cannot render a state
//! the store never wrote (P-9).
//!
//! Version 8 (slice S066) adds two nullable `TEXT` columns to `targets`,
//! `folder_name` and `executable_hint`: a target's raw platform installdir (or
//! directory-scan folder name) and its raw observed launch executable, both stored
//! verbatim and neither reconstructed from `name` or from the other (issue #173).
//! Neither carries a CHECK constraint: both are free-form observed strings with no
//! closed vocabulary to enforce, unlike `detection_scan`'s enum. Backward-safe by
//! construction: an existing row reads both as NULL, which is exactly "not
//! recorded".
//!
//! Version 9 (issue #217) adds `deep_capture_facts`, the local compatibility fact
//! table. It is keyed to `targets(id)` so Deep Capture facts do not create a
//! second target-resolution path. CHECK constraints keep the fact keys,
//! launch-case tokens, proxy provenance, final-owner details, stale flag, and
//! key-specific value vocabularies closed where the value domain is known: the
//! store can record `unknown`, but it cannot record an out-of-vocabulary guess.
//! The migration from version 8 is one additive `CREATE TABLE`.

/// The schema version this build writes and understands.
pub const SCHEMA_VERSION: i64 = 9;

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
    evidence              TEXT,
    detection_scan        TEXT CHECK (detection_scan IS NULL OR detection_scan IN
                            ('complete', 'incomplete')),
    folder_name           TEXT,
    executable_hint       TEXT
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

CREATE TABLE listing_snapshot (
    position   INTEGER PRIMARY KEY CHECK (position > 0),
    stable_id  INTEGER NOT NULL,
    handle     TEXT NOT NULL CHECK (length(handle) > 0)
);

CREATE TABLE deep_capture_facts (
    id               INTEGER PRIMARY KEY,
    target_id        INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    fact_key         TEXT NOT NULL CHECK (fact_key IN
                       ('proxy-environment-honored', 'proxy-routing',
                        'proxy-propagation', 'launch-case', 'final-socket-owner-role',
                        'publisher-launcher-present',
                        'requires-platform-cold-start-for-proxy',
                        'direct-exe-supported', 'steam-protocol-supported',
                        'tls-trust-behavior', 'protocol-behavior', 'inspectability',
                        'proxy-variable-tested')),
    fact_value       TEXT NOT NULL CHECK (length(fact_value) > 0),
    launch_case      TEXT CHECK (launch_case IS NULL OR launch_case IN
                       ('steam-protocol-warm', 'steam-protocol-cold',
                        'direct-exe-warm', 'direct-exe-cold', 'publisher-launcher',
                        'publisher-launcher-warm',
                        'publisher-launcher-game-start-clean-warm',
                        'publisher-launcher-cold')),
    evidence_source  TEXT NOT NULL CHECK (evidence_source IN
                       ('observed-run', 'user-confirmed', 'imported-catalog',
                        'stale-observation')),
    observed_at      TEXT,
    fragcap_version  TEXT,
    target_version   TEXT,
    proxy_backend    TEXT CHECK (proxy_backend IS NULL OR length(proxy_backend) > 0),
    proxy_backend_version TEXT,
    proxy_mode       TEXT CHECK (proxy_mode IS NULL OR length(proxy_mode) > 0),
    final_owner_executable TEXT
                       CHECK (final_owner_executable IS NULL OR length(final_owner_executable) > 0),
    final_owner_handoff INTEGER NOT NULL DEFAULT 0 CHECK (final_owner_handoff IN (0, 1)),
    stale            INTEGER NOT NULL DEFAULT 0 CHECK (stale IN (0, 1)),
    note             TEXT,
    CHECK (
        (fact_key IN ('proxy-environment-honored', 'publisher-launcher-present',
                      'requires-platform-cold-start-for-proxy',
                      'direct-exe-supported', 'steam-protocol-supported')
            AND fact_value IN ('yes', 'no', 'unknown'))
        OR (fact_key = 'proxy-routing'
            AND fact_value IN ('reached-client', 'launcher-only-routing',
                               'escaped-tree', 'no-proxy-traffic',
                               'not-applicable', 'inconclusive'))
        OR (fact_key = 'proxy-propagation'
            AND fact_value IN ('confirmed', 'not-confirmed', 'not-tested'))
        OR (fact_key = 'launch-case'
            AND fact_value IN ('steam-protocol-warm', 'steam-protocol-cold',
                               'direct-exe-warm', 'direct-exe-cold',
                               'publisher-launcher', 'publisher-launcher-warm',
                               'publisher-launcher-game-start-clean-warm',
                               'publisher-launcher-cold'))
        OR (fact_key = 'final-socket-owner-role'
            AND fact_value IN ('client', 'launcher', 'platform', 'platform-service',
                               'helper', 'proxy', 'wrapper', 'unknown'))
        OR (fact_key = 'tls-trust-behavior'
            AND fact_value IN ('accepts-local-ca', 'certificate-pinned', 'unknown'))
        OR (fact_key = 'protocol-behavior'
            AND fact_value IN ('http', 'https', 'websocket', 'non-http-tls', 'quic',
                               'udp', 'plaintext', 'unknown'))
        OR (fact_key = 'inspectability'
            AND fact_value IN ('full', 'metadata-only', 'unsupported', 'unknown'))
        OR (fact_key = 'proxy-variable-tested'
            AND fact_value IN ('HTTP_PROXY', 'HTTPS_PROXY', 'ALL_PROXY', 'NO_PROXY',
                               'http_proxy', 'https_proxy', 'all_proxy', 'no_proxy'))
    )
);
";

/// The additive migration from schema version 8 to version 9: create the Deep
/// Capture compatibility fact table (issue #217). Backward-safe by construction,
/// an existing v8 store keeps every target row and gains one empty table. Applied
/// in one transaction alongside the version stamp.
pub const MIGRATE_8_TO_9: &str = "\
CREATE TABLE deep_capture_facts (
    id               INTEGER PRIMARY KEY,
    target_id        INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    fact_key         TEXT NOT NULL CHECK (fact_key IN
                       ('proxy-environment-honored', 'proxy-routing',
                        'proxy-propagation', 'launch-case', 'final-socket-owner-role',
                        'publisher-launcher-present',
                        'requires-platform-cold-start-for-proxy',
                        'direct-exe-supported', 'steam-protocol-supported',
                        'tls-trust-behavior', 'protocol-behavior', 'inspectability',
                        'proxy-variable-tested')),
    fact_value       TEXT NOT NULL CHECK (length(fact_value) > 0),
    launch_case      TEXT CHECK (launch_case IS NULL OR launch_case IN
                       ('steam-protocol-warm', 'steam-protocol-cold',
                        'direct-exe-warm', 'direct-exe-cold', 'publisher-launcher',
                        'publisher-launcher-warm',
                        'publisher-launcher-game-start-clean-warm',
                        'publisher-launcher-cold')),
    evidence_source  TEXT NOT NULL CHECK (evidence_source IN
                       ('observed-run', 'user-confirmed', 'imported-catalog',
                        'stale-observation')),
    observed_at      TEXT,
    fragcap_version  TEXT,
    target_version   TEXT,
    proxy_backend    TEXT CHECK (proxy_backend IS NULL OR length(proxy_backend) > 0),
    proxy_backend_version TEXT,
    proxy_mode       TEXT CHECK (proxy_mode IS NULL OR length(proxy_mode) > 0),
    final_owner_executable TEXT
                       CHECK (final_owner_executable IS NULL OR length(final_owner_executable) > 0),
    final_owner_handoff INTEGER NOT NULL DEFAULT 0 CHECK (final_owner_handoff IN (0, 1)),
    stale            INTEGER NOT NULL DEFAULT 0 CHECK (stale IN (0, 1)),
    note             TEXT,
    CHECK (
        (fact_key IN ('proxy-environment-honored', 'publisher-launcher-present',
                      'requires-platform-cold-start-for-proxy',
                      'direct-exe-supported', 'steam-protocol-supported')
            AND fact_value IN ('yes', 'no', 'unknown'))
        OR (fact_key = 'proxy-routing'
            AND fact_value IN ('reached-client', 'launcher-only-routing',
                               'escaped-tree', 'no-proxy-traffic',
                               'not-applicable', 'inconclusive'))
        OR (fact_key = 'proxy-propagation'
            AND fact_value IN ('confirmed', 'not-confirmed', 'not-tested'))
        OR (fact_key = 'launch-case'
            AND fact_value IN ('steam-protocol-warm', 'steam-protocol-cold',
                               'direct-exe-warm', 'direct-exe-cold',
                               'publisher-launcher', 'publisher-launcher-warm',
                               'publisher-launcher-game-start-clean-warm',
                               'publisher-launcher-cold'))
        OR (fact_key = 'final-socket-owner-role'
            AND fact_value IN ('client', 'launcher', 'platform', 'platform-service',
                               'helper', 'proxy', 'wrapper', 'unknown'))
        OR (fact_key = 'tls-trust-behavior'
            AND fact_value IN ('accepts-local-ca', 'certificate-pinned', 'unknown'))
        OR (fact_key = 'protocol-behavior'
            AND fact_value IN ('http', 'https', 'websocket', 'non-http-tls', 'quic',
                               'udp', 'plaintext', 'unknown'))
        OR (fact_key = 'inspectability'
            AND fact_value IN ('full', 'metadata-only', 'unsupported', 'unknown'))
        OR (fact_key = 'proxy-variable-tested'
            AND fact_value IN ('HTTP_PROXY', 'HTTPS_PROXY', 'ALL_PROXY', 'NO_PROXY',
                               'http_proxy', 'https_proxy', 'all_proxy', 'no_proxy'))
    )
);
";

/// The additive migration from schema version 7 to version 8: add the nullable
/// `folder_name` and `executable_hint` columns to `targets` (slice S066). Two
/// `ALTER TABLE` statements in one transaction, following the S051 precedent of a
/// multi-statement additive migration. Backward-safe by construction: an existing
/// v7 store keeps every row and reads both new columns as NULL, exactly "not
/// recorded".
pub const MIGRATE_7_TO_8: &str = "\
ALTER TABLE targets ADD COLUMN folder_name TEXT;
ALTER TABLE targets ADD COLUMN executable_hint TEXT;
";

/// The additive migration from schema version 5 to version 6: create the listing
/// snapshot table (slice S055). It pins the ordered rows the most recent listing
/// displayed so a 1-based row-index selector resolves to what the user saw.
/// Backward-safe by construction, an existing v5 store keeps every row and gains
/// one empty table. Applied in one transaction alongside the version stamp.
/// The additive migration from schema version 6 to version 7: add the nullable
/// `detection_scan` column to `targets` (slice S065). Backward-safe by construction,
/// an existing v6 store keeps every row and reads the column as NULL, which is
/// exactly the "no scan recorded" state, so no backfill is needed and no row changes
/// meaning. Applied in one transaction alongside the version stamp.
pub const MIGRATE_6_TO_7: &str = "\
ALTER TABLE targets ADD COLUMN detection_scan TEXT
    CHECK (detection_scan IS NULL OR detection_scan IN ('complete', 'incomplete'));
";

pub const MIGRATE_5_TO_6: &str = "\
CREATE TABLE listing_snapshot (
    position   INTEGER PRIMARY KEY CHECK (position > 0),
    stable_id  INTEGER NOT NULL,
    handle     TEXT NOT NULL CHECK (length(handle) > 0)
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
