# Feature Specification: Targets Hint Database (foundation)

**Feature Branch**: `034-targets-hint-database`

**Created**: 2026-08-13

**Status**: Draft

**Slice**: S034 (issue #78, first of several; part of #77; consumes #75/S033)

**Input**: Foundation slice of the targets hint database (#78): an embedded
SQLite store, the three-tier seeding model, and a schema-conformant JSON export.
No network fetching this slice.

## Overview

The targets hint database (issue #78) is the provider registered at precedence 2
of the resolution cascade (#77) that today returns no answer. It is meant to be a
large, auto-generated corpus of known game binaries and launch patterns that
seeds target resolution at the `heuristic-unverified` tier: never a source of
truth, always overridable by a live runtime observation (P-9).

S033 already defined the JSON shape the database must emit: the loose
hint-record subschema gained a `launch` array, a `launcher_mediated` flag, and an
`engine` object, all published in `docs/schema/target-schema.v1.json`. What does
not yet exist is the store itself.

This slice builds the foundation only, and deliberately stops before any network
access: the persistence layer, the model that says which of three seeding tiers
owns which columns, and the path that projects stored rows into schema-valid
export JSON. Getting the store, the mapping, and the export right is the
prerequisite every later seeding slice (Web API, PICS, PCGamingWiki) and the
cascade-wiring slice build on. A maintainer can exercise the whole path offline:
load a committed seed fixture into a store and export it to JSON that an
unmodified schema validator accepts.

## Clarifications

### Session 2026-08-13

- Q: Where does the store and its embedded-database dependency live? -> A: A new
  `fragcap-targets` crate, depending only on `fragcap-profile` (for schema
  validation), exposed through the `fragcap` facade behind an optional `targets`
  feature so default builds do not compile the embedded database. Not
  `fragcap-steam` (the corpus is broader than Steam; it also carries PCGamingWiki
  engine data) and never `fragcap-core` (P-2).
- Q: With no fetching this slice, how is the export demonstrable? -> A: A `fragcap
  targets` CLI with an offline `import <json>` and an `export` subcommand, plus a
  small committed seed fixture, so `import` then `export` round-trips to
  schema-valid JSON with no network.
- Q: Is a database row's engine-confidence grade a new resolution fidelity? -> A:
  No. Every exported record carries fidelity `heuristic-unverified`. Engine
  confidence (`confirmed`/`high`/`medium`/`low`/`unknown`) grades the engine
  field within that one tier; it never promotes or demotes a record's overall
  trust (P-9).
- Q: Should the launch executable be reduced to a single process name at store
  time? -> A: No. The full launch array is persisted whole, with its filters
  (os, osarch, launch type, beta branch) intact. Reducing an array to the one
  entry that will actually hold sockets at runtime is the resolver's job (#77),
  not the store's.
- Q: How are duplicate application ids handled on import? -> A: Two rules. A
  duplicate appid within a single import document is an error (the whole import is
  rejected, no partial store written). Importing a game whose appid already exists
  in the store replaces that game wholesale (its rows are deleted and reinserted,
  never partially merged), so re-importing the same seed is idempotent.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Export a populated store to schema-valid JSON (Priority: P1)

A maintainer has a hint-database store holding some games, their launch entries,
and engine attributions. They export it and receive a single JSON document, in
the loose `export` shape, that an unmodified target-schema validator accepts.

**Why this priority**: The export is the database's only external contract. Every
later seeding slice is worthless if the rows it writes cannot be projected into
the schema the cascade and downstream tooling read. This is the MVP: a store that
round-trips its contents to valid JSON.

**Independent Test**: Populate an in-memory or temporary store through the library
API with a handful of games (one with an engine, one without, one
launcher-mediated, one with several launch entries carrying different filters),
export it, and assert the resulting JSON validates against the embedded schema
with zero diagnostics.

**Acceptance Scenarios**:

1. **Given** a store with games carrying launch entries and engine attributions,
   **When** the maintainer exports it, **Then** the output is a single `export`
   envelope whose every record validates against the hint-record subschema and is
   stamped fidelity `heuristic-unverified`.
2. **Given** a game whose engine is unknown (no engine attribution), **When** the
   store is exported, **Then** that record omits the `engine` object entirely
   rather than emitting a null or a placeholder.
3. **Given** a game with several launch entries, **When** the store is exported,
   **Then** the record's `launch` array carries every entry with its filters
   intact, in stored order, never collapsed to one.

### User Story 2 - Offline import/export round-trip through the CLI (Priority: P2)

A maintainer runs `fragcap targets import <seed.json> --db <path>` to load a
committed seed fixture into a fresh store, then `fragcap targets export --db
<path>` to project it back out, and gets schema-valid JSON. No network access
occurs at any point.

**Why this priority**: This makes the foundation demonstrable by a human without
writing Rust, and it is the shape the later network seeders slot into (each
seeder is an `import`-like writer for its own tier). It depends on US1's export
path.

**Independent Test**: In a temporary directory, run the `import` command against
the committed seed fixture, then the `export` command, and assert the exported
JSON validates and reflects the seeded rows. Assert no network syscall is made
(the commands take only local paths).

**Acceptance Scenarios**:

1. **Given** a committed seed fixture and no existing database file, **When** the
   maintainer runs `targets import` then `targets export`, **Then** a store is
   created, populated, and projected to schema-valid JSON.
2. **Given** a malformed seed fixture (e.g. a launch entry missing its
   executable), **When** the maintainer runs `targets import`, **Then** the
   command fails with a diagnostic naming the problem and writes no partial store
   (exit non-zero), rather than importing a record the schema would reject.

### User Story 3 - Model three independent, resumable seeding tiers (Priority: P3)

The store's schema separates the columns each of three seeding tiers owns, and
records per-tier seeding state, so that a later slice can run one tier's fetch and
fill only its columns without disturbing another tier's data or forcing a full
rebuild.

**Why this priority**: Resumability is a structural requirement from the issue
(#83): the corpus is large and each tier has a different access cost and cadence.
The columns and the seed-state must be designed now even though no fetcher writes
them this slice, or the later seeders inherit a schema that cannot express partial
progress.

**Independent Test**: Through the library API, write only Tier 1 columns (appid,
name) for a game, then later add Tier 3 columns (engine attribution) to the same
game, and assert both persist and export correctly, with per-tier seed-state
readable for each.

**Acceptance Scenarios**:

1. **Given** a game with only Tier 1 columns populated, **When** it is exported,
   **Then** the record is valid (identity present, launch and engine absent).
2. **Given** a game later enriched with a Tier 3 engine attribution, **When** it
   is exported, **Then** the record now carries the engine object and remains
   valid, with Tier 1 data unchanged.
3. **Given** any tier's seeding has run, **When** seed-state is read, **Then** it
   reports which tier last ran and a resume cursor for it.

### Edge Cases

- A store opened against a file written by a newer schema version: the store
  reports the version mismatch rather than silently operating on an incompatible
  layout.
- A game with an empty launch array: the exported record omits `launch` rather
  than emitting `[]` (an empty array is valid per schema, but omission keeps the
  record minimal and unambiguous; this is an assumption, recorded below).
- A game whose engine attribution carries an out-of-set `source` or `confidence`:
  rejected at write time, never persisted, so the store can never export a record
  the schema would reject.
- Duplicate appids within one import document: rejected as an error, no partial
  store written. An appid already present in the store: replaced wholesale (rows
  deleted and reinserted), never silently merged into a half-updated row.
- The `targets` feature is disabled: the CLI subcommand is absent (or reports it
  was built without database support) rather than presenting a command that
  cannot work.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an embedded, single-file database store for
  target hints that requires no external database service and no network access to
  open, create, or query.
- **FR-002**: The store MUST persist, per game: a Steam application id (unique
  identity), a name, secondary Steam-catalog metrics (review counts, owners, peak
  concurrent players), a `launcher_mediated` flag, a `token_required` flag, and an
  optional engine attribution (engine name, engine source, engine confidence).
- **FR-003**: The store MUST persist a game's launch entries as an ordered
  collection, each carrying its executable (required, non-empty), optional
  arguments and description, and its filters (operating system, os architecture,
  launch type, beta branch). The collection MUST NOT be reduced to a single
  process name at store time.
- **FR-004**: The store MUST persist per-game technology findings (a category, a
  technology name, and its evidence/marker) as an ordered collection.
- **FR-005**: The store MUST record per-tier seeding state (which of the three
  tiers last ran, and a resume cursor for it) so a later fetch can resume without
  a full rebuild. No fetch writes this state in this slice; the structure exists
  for later slices.
- **FR-006**: Engine source MUST be one of `pcgamingwiki`, `exe_heuristic`,
  `depot_filename_rules`; engine confidence MUST be one of `confirmed`, `high`,
  `medium`, `low`, `unknown`. A write carrying any other value MUST be refused,
  not coerced.
- **FR-007**: The seeding model MUST assign column ownership by tier: Tier 1
  (public catalog) owns appid, name, and catalog metrics; Tier 2 (application
  launch metadata) owns launch entries, `launcher_mediated`, and `token_required`;
  Tier 3 (community engine data) owns the engine attribution with source
  `pcgamingwiki`. Each tier's columns MUST be independently writable, leaving the
  others intact.
- **FR-008**: The system MUST export a store's contents as a single JSON document
  in the loose `export` shape: an envelope carrying a records array, one record
  per game. Each record MUST carry fidelity `heuristic-unverified`, a provenance
  source, the game's identity, and, when present, the launch array,
  `launcher_mediated`, and engine object. A record whose engine is unknown MUST
  omit the engine object.
- **FR-009**: Every exported document MUST validate against the published target
  schema's hint-record subschema. The exporter MUST validate its own output before
  returning it and MUST NOT return a document the validator rejects
  (validity by construction).
- **FR-010**: Engine confidence MUST NOT act as a resolution fidelity tier. Every
  exported record's fidelity is `heuristic-unverified` regardless of engine
  confidence; confidence grades only the engine field (P-9).
- **FR-011**: The system MUST provide a command-line surface to import a local seed
  document into a store and to export a store to JSON, both operating only on local
  paths with no network access. A committed seed fixture MUST exercise the
  import-then-export round-trip to a schema-valid result.
- **FR-012**: An import of a malformed seed (a record the schema would reject, e.g.
  a launch entry missing its executable, or an out-of-set engine source) MUST fail
  with a diagnostic and MUST NOT leave a partially populated store.
- **FR-013**: The database capability MUST be optional at build time (a `targets`
  feature) so that a default build of the project neither compiles the embedded
  database engine nor requires a C toolchain for it.
- **FR-014**: Introducing the store MUST NOT make `fragcap-core` depend on the new
  crate or on the embedded-database dependency; the dependency direction stays
  concrete toward abstract (P-2).
- **FR-015**: Import MUST be idempotent per application id: a duplicate appid
  within one import document is rejected as an error with no partial store written,
  and importing an appid already present in the store replaces that game's rows
  wholesale (delete then insert) rather than merging into a half-updated row.

### Key Entities

- **Game**: One target title. Identity is a Steam application id. Carries a name,
  catalog metrics, the `launcher_mediated` and `token_required` flags, and an
  optional engine attribution. Owns zero or more launch entries and zero or more
  technology findings.
- **Launch entry**: One way the title can be started, as disclosed by its launch
  metadata. Carries a required executable, optional arguments and description, and
  filters (os, os architecture, launch type, beta branch). Ordered within a game.
- **Engine attribution**: The engine a title runs on, with the source that
  determined it and a confidence grade. Optional; absent means unknown.
- **Technology finding**: A detected technology (category, name, evidence) present
  for a title. Ordered within a game.
- **Seed state**: Per-tier record of seeding progress (last tier run, resume
  cursor). Structural; unwritten by any fetch in this slice.
- **Export document**: The loose `export` envelope projecting the store's games
  into schema-conformant records.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A maintainer can take a committed seed fixture and produce
  schema-valid export JSON from it, end to end, with no network access, in a single
  command sequence (import then export).
- **SC-002**: 100% of exported documents produced from any valid store state
  validate against the published hint-record subschema (verified by the exporter's
  own validation and by an independent conformance test).
- **SC-003**: A store can be enriched one tier at a time: populating one tier's
  columns and later another tier's columns for the same title both persist and
  export correctly, with no full rebuild required.
- **SC-004**: A malformed seed (out-of-set engine value, or a launch entry missing
  its executable) is rejected at import with a clear diagnostic and leaves no
  partial store, in 100% of such cases.
- **SC-005**: A default build of the project (without the `targets` feature)
  compiles without the embedded database engine, and the project's full check set
  (including the minimum-supported-toolchain build) passes with the feature
  enabled.

## Assumptions

- The embedded database engine is compiled from a bundled source (no reliance on a
  system-installed database library), so the build is deterministic on a bare
  Windows runner. The bundled engine's licensing is recorded as a decision.
- An empty launch collection and an unknown engine are both represented by
  omission in the export (the schema permits an empty `launch` array, but omission
  is chosen for a minimal, unambiguous record).
- The seed fixture is a small hand-authored set (a few titles, including one
  launcher-mediated title such as The Elder Scrolls Online and one carrying an
  engine attribution), sufficient to exercise every export branch; it is not a real
  catalog dump.
- The export envelope's provenance source is a fixed identifier naming the hint
  database as the origin (the actual per-tier provenance detail is a later-slice
  concern); this slice stamps a single database-origin source.
- The `targets` feature is enabled for the CLI binary so the subcommand is present
  in shipped builds; it is off by default at the library/facade level so unrelated
  consumers do not pay for the database engine.
- The store operates on the current schema version only; cross-version migration of
  a database file is out of scope for this slice beyond detecting a mismatch.
