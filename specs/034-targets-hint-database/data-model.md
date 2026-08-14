# Phase 1 Data Model: Targets Hint Database (foundation)

Two layers: the SQLite persistence schema, and the Rust value types the library
API exchanges. The mapping to the published JSON schema
(`docs/schema/target-schema.v1.json`) is given last and is the binding contract.

## SQLite schema (one migration, version 1)

A single embedded file. `PRAGMA user_version` carries the schema version; opening
a file whose `user_version` is newer than the code's is an error (edge case in
spec), not a silent operation.

```sql
CREATE TABLE games (
    appid              INTEGER PRIMARY KEY,   -- Steam application id (Tier 1)
    name               TEXT,                  -- display name (Tier 1)
    review_count       INTEGER,               -- catalog metric (Tier 1, nullable)
    owners             INTEGER,               -- catalog metric (Tier 1, nullable)
    peak_ccu           INTEGER,               -- catalog metric (Tier 1, nullable)
    launcher_mediated  INTEGER,               -- 0/1/NULL boolean (Tier 2)
    token_required     INTEGER,               -- 0/1/NULL boolean (Tier 2)
    engine_name        TEXT,                  -- engine label (Tier 3, nullable even when engine present)
    engine_source      TEXT,                  -- enum, NULL iff no engine (Tier 3)
    engine_confidence  TEXT                   -- enum, NULL iff no engine (Tier 3)
    -- CHECK constraints enforce the enum sets and the engine invariant below.
);

CREATE TABLE launch_entries (               -- Tier 2; ordered, persisted whole
    appid         INTEGER NOT NULL REFERENCES games(appid) ON DELETE CASCADE,
    launch_index  INTEGER NOT NULL,          -- position within the game's launch array
    os            TEXT,
    osarch        TEXT,
    launch_type   TEXT,
    beta_branch   TEXT,
    executable    TEXT NOT NULL,             -- required, non-empty (CHECK length > 0)
    arguments     TEXT,
    description   TEXT,
    PRIMARY KEY (appid, launch_index)
);

CREATE TABLE technologies (                 -- ordered per game
    appid       INTEGER NOT NULL REFERENCES games(appid) ON DELETE CASCADE,
    tech_index  INTEGER NOT NULL,
    category    TEXT NOT NULL,               -- schema technology category enum
    name        TEXT NOT NULL,
    marker_path TEXT,
    PRIMARY KEY (appid, tech_index)
);

CREATE TABLE seed_state (                   -- per-tier resumability (structural this slice)
    tier          TEXT PRIMARY KEY,          -- 'catalog' | 'launch' | 'engine'
    last_run_at   TEXT,                      -- ISO-8601, nullable
    resume_cursor TEXT                       -- opaque resume token, nullable
);
```

Constraints that make an invalid row impossible to store (so the store can never
export a document the schema rejects):

- `engine_source IN ('pcgamingwiki','exe_heuristic','depot_filename_rules')` or
  NULL; `engine_confidence IN ('confirmed','high','medium','low','unknown')` or
  NULL.
- Engine invariant: `engine_source` and `engine_confidence` are both NULL or both
  non-NULL (an engine attribution needs both; `engine_name` may be NULL either
  way). Enforced by a table CHECK plus the write-path guard.
- `launcher_mediated`/`token_required` are 0, 1, or NULL.
- `launch_entries.executable` length > 0 (CHECK).
- `category` restricted to the schema's technology category enum.
- Foreign keys ON; `PRAGMA foreign_keys = ON` at open.

`ON DELETE CASCADE` gives the wholesale-replace on re-import (R6): delete the
`games` row for an appid and its launch/technology rows go with it, then reinsert.

## Rust value types (library API surface)

```text
Game { appid: u32, name: Option<String>, review_count: Option<u64>,
       owners: Option<u64>, peak_ccu: Option<u64>,
       launcher_mediated: Option<bool>, token_required: Option<bool>,
       engine: Option<Engine>,
       launch: Vec<LaunchEntry>, technologies: Vec<Technology> }

Engine { name: Option<String>, source: EngineSource, confidence: EngineConfidence }
  EngineSource      = Pcgamingwiki | ExeHeuristic | DepotFilenameRules
  EngineConfidence  = Confirmed | High | Medium | Low | Unknown

LaunchEntry { os, osarch, launch_type, beta_branch: Option<String>,
              executable: String,  // non-empty, validated on construction
              arguments, description: Option<String> }

Technology { category: TechCategory, name: String, marker_path: Option<String> }

SeedTier   = Catalog | Launch | Engine
SeedState  { tier: SeedTier, last_run_at: Option<String>, resume_cursor: Option<String> }
```

Making `Engine` a single `Option<Engine>` with mandatory `source`+`confidence`
encodes the engine invariant in the type: you cannot construct an engine
attribution missing either, which is exactly what the schema requires. Building a
`LaunchEntry` with an empty executable is rejected at construction, so no invalid
launch entry reaches the store or the export.

## Column to JSON mapping (the binding export contract)

The export is a single `kind: "export"` document. Envelope:

| JSON path | Value | Source |
| --- | --- | --- |
| `schema` | `1` | constant |
| `kind` | `"export"` | constant |
| `fidelity` | `"heuristic-unverified"` | constant (P-9: the whole DB is this tier) |
| `provenance.source` | `"hint-db"` | constant this slice |
| `records` | array, one per game | `games` rows |

Each element of `records` (a schema `record`), for one game:

| JSON path | Value | Column / rule |
| --- | --- | --- |
| `fidelity` | `"heuristic-unverified"` | constant |
| `provenance.source` | `"hint-db"` | constant this slice |
| `game.app_id` | `appid` as string | `games.appid` |
| `game.name` | name | `games.name` (omit key if NULL) |
| `game.platform` | `"steam"` | constant when appid present |
| `launch` | array of launch entries | `launch_entries` for appid, ordered by `launch_index`; omit key when none |
| `launch[i].executable` | executable | required, always present |
| `launch[i].{os,osarch,launch_type,beta_branch,arguments,description}` | filters/args | omit each key when its column is NULL |
| `launcher_mediated` | bool | `games.launcher_mediated`; omit key when NULL |
| `engine` | object | present only when `engine_source` is non-NULL; else omit |
| `engine.name` | engine label | `games.engine_name`; omit when NULL |
| `engine.source` | enum | `games.engine_source` |
| `engine.confidence` | enum | `games.engine_confidence` |

Notes bound by the schema:

- `game.id` is deliberately NOT emitted (its `^[a-z0-9_-]+$` pattern would reject
  raw names; a record does not require it).
- Top-level `launch`/`launcher_mediated`/`engine` are NEVER emitted on the
  envelope; the schema `allOf` forbids them for `kind: export`. They appear only
  inside records.
- An unknown engine and an empty launch collection are both represented by
  omission (spec assumption), producing a minimal record.
- `technologies` are stored but are NOT part of this slice's record projection
  unless a later decision maps them; the record `$def` has no `technologies`
  member (only the top-level target does), so per-record technology export is out
  of scope here. Recorded so a later slice does not mistake the omission for a
  bug.

## Validation

Before returning any export text, the exporter builds the `serde_json::Value` and
calls `fragcap_profile::jsonschema::validate_value(&value)`. A non-empty
`SchemaDiagnostics` is an internal error (the exporter is wrong), surfaced rather
than swallowed. The conformance test drives the same path over fixtures and also
asserts specific malformed inputs are rejected with the expected `SchemaCode`.
