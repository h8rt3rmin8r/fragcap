---
description: Dependency-ordered tasks for S034 Targets Hint Database (foundation)
---

# Tasks: Targets Hint Database (foundation)

**Slice**: S034 (issue #78) | **Branch**: `034-targets-hint-database`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

**Format**: `[ID] [P?] [Story?] Description with file path`
`[P]` = parallelizable (different files, no incomplete dependency).
`[USn]` = serves User Story n. Setup/Foundational/Polish carry no story label.

MVP = User Story 1 (export a populated store to schema-valid JSON). Test-driven:
conformance and round-trip tests are written with (or before) the code they pin.

## Phase 1: Setup

**Goal**: The crate exists, is wired into the workspace and facade, and the
dependency-direction gate is taught the new edges.

- [x] T001 Create the `crates/fragcap-targets/` crate: `Cargo.toml` (inherits workspace package fields; deps `fragcap-profile.workspace`, `serde_json.workspace`, and `rusqlite = { workspace = true }`), and a minimal `src/lib.rs` with the SPDX header and crate docs. `LICENSE` and `NOTICE` copied byte-for-byte from the repo-root originals; a `README.md`.
- [x] T002 Add to root `Cargo.toml` `[workspace.dependencies]`: `rusqlite = { version = "0.40", default-features = false, features = ["bundled"] }` with the multi-line justification comment (unique property, six-package delta, MSRV verified under 1.82, MIT/public-domain licensing, default-features-off reason), and `fragcap-targets = { path = "crates/fragcap-targets", version = "0.2.0" }`.
- [x] T003 Wire the facade in `crates/fragcap/Cargo.toml`: `fragcap-targets = { workspace = true, optional = true }` and a `targets = ["dep:fragcap-targets"]` feature (off by default); re-export the crate under `#[cfg(feature = "targets")]` in `crates/fragcap/src/lib.rs`.
- [x] T004 Update `xtask/src/deps.rs`: add `("fragcap", "fragcap-targets")` and `("fragcap-targets", "fragcap-profile")` to `EXPECTED`, and add `"fragcap-targets"` to `SIBLINGS`. Confirm `fragcap-core`'s allowlist is untouched.

**Checkpoint**: `cargo build -p fragcap-targets` compiles; `cargo build -p fragcap` (no feature) does not compile rusqlite; `cargo xtask deps` passes.

## Phase 2: Foundational (blocking prerequisites)

**Goal**: The value types and the store exist, with invalid states unrepresentable
and unstorable. Blocks every user story.

- [x] T005 Implement `crates/fragcap-targets/src/model.rs`: `Game`, `LaunchEntry`, `Engine`, `Technology`, enums `EngineSource`/`EngineConfidence`/`TechCategory`/`SeedTier`, and `SeedState`. Validating constructors: `LaunchEntry` rejects an empty executable; `Engine` requires `source` + `confidence` (name optional). Enum <-> string conversions match the schema's exact tokens.
- [x] T006 Implement `crates/fragcap-targets/src/schema.rs`: the embedded DDL for `games`, `launch_entries`, `technologies`, `seed_state` per data-model.md, with CHECK constraints (engine enum sets, engine both-or-neither invariant, non-empty executable, boolean 0/1, technology category), foreign keys, and a `user_version = 1` migration. Opening a newer `user_version` is an error.
- [x] T007 Implement `crates/fragcap-targets/src/lib.rs` error type `TargetsError` distinguishing usage/parse, operational (I/O, malformed record, schema violation), and internal-invariant (self-validation) classes.
- [x] T008 Implement `crates/fragcap-targets/src/store.rs`: `Store::open`/`open_in_memory` (enable `foreign_keys`, apply/verify migration), `upsert_game` (transactional delete-then-insert of the game and its launch/technology rows; rejects out-of-set enum and empty executable before writing), `games` (read all with ordered launch entries and technologies), and `seed_state`/`set_seed_state`.

**Checkpoint**: a unit test can open an in-memory store, upsert a game, read it back, and a CHECK-violating write is refused.

## Phase 3: User Story 1 - Export a populated store to schema-valid JSON (P1)

**Goal**: A store projects its games into a `kind: "export"` document that validates
against the hint-record subschema.

**Independent test**: insert games across the three tiers (one launcher-mediated,
one with an engine, one Tier-1-only), export, assert `validate_value` returns no
diagnostics and the record shapes are correct.

- [x] T009 [P] [US1] Write `crates/fragcap-targets/tests/conformance.rs` and fixtures under `tests/fixtures/`: a well-formed export validates; a malformed one (out-of-set `engine.source`, and a launch entry missing `executable`) is rejected with the expected `SchemaCode`. (Written first; drives T011.)
- [x] T010 [P] [US1] Write `crates/fragcap-targets/tests/round_trip.rs`: insert across tiers, export, assert zero diagnostics via `fragcap_profile::jsonschema::validate_value`, assert engine omitted when unknown, launch array carried whole and ordered, and fidelity always `heuristic-unverified`.
- [x] T011 [US1] Implement `crates/fragcap-targets/src/export.rs`: build the `serde_json::Value` envelope (`schema`/`kind`/`fidelity`/`provenance`/`records`) and per-record mapping per data-model.md (omission rules for name, launch, launcher_mediated, engine; `game.id` never emitted; top-level launch/engine never emitted), validate with `validate_value` before returning, and error on non-empty diagnostics. Make T009 and T010 pass.

**Checkpoint**: `cargo test -p fragcap-targets` green; MVP delivered.

## Phase 4: User Story 2 - Offline import/export round-trip through the CLI (P2)

**Goal**: `fragcap targets import <seed> --db <path>` then `export --db <path>`
round-trips to schema-valid JSON with no network.

**Independent test**: run import then export against a temporary store; assert
valid output reflecting the seed; a malformed seed exits non-zero leaving no store.

- [x] T012 [P] [US2] Add `crates/fragcap-targets/tests/fixtures/seed.json` (hand-authored: The Elder Scrolls Online with `launcher_mediated`, one title with an `engine`, one Tier-1-only title) and a malformed-seed fixture (launch entry missing executable).
- [x] T013 [US2] Implement `crates/fragcap-targets/src/import.rs`: parse the seed JSON, enforce the duplicate-appid-in-document error (whole-import rollback) and existing-appid wholesale replace, reject malformed records before any write, all transactional; return an `ImportSummary`.
- [x] T014 [US2] Add `crates/fragcap-cli/src/commands/targets.rs` with `targets import <SEED> --db <DB>` and `targets export --db <DB>` (0/1/2 exit contract), register it in `crates/fragcap-cli/src/commands/mod.rs` and the clap enum in `crates/fragcap-cli/src/cli.rs`, all under `#[cfg(feature = "targets")]`; enable `fragcap = { ..., features = ["targets"] }` (or a `targets` passthrough feature) in `crates/fragcap-cli/Cargo.toml`.
- [x] T015 [P] [US2] Write `crates/fragcap-cli/tests/targets.rs` (using `tempfile`): import the seed then export, assert the output validates and reflects the seed; assert a malformed seed exits non-zero and writes no store; assert re-import is idempotent.

**Checkpoint**: CLI round-trip works offline; `cargo test -p fragcap-cli --features targets` green.

## Phase 5: User Story 3 - Independent, resumable seeding tiers (P3)

**Goal**: Each tier's columns are independently writable and the store records
per-tier seed state.

**Independent test**: write only Tier 1 columns for a game, later add a Tier 3
engine attribution, assert both persist and export valid; read seed state per tier.

- [x] T016 [P] [US3] Write `crates/fragcap-targets/tests/tiers.rs`: upsert a Tier-1-only game (appid + name), export and assert valid with no launch/engine; enrich the same appid with a Tier 3 engine, re-export and assert the engine appears while Tier 1 data is unchanged; set and read `seed_state` for each tier.

**Checkpoint**: incremental per-tier enrichment proven without a full rebuild.

## Phase 6: Polish & Cross-Cutting

- [x] T017 [P] Add glossary entries (P-6) for any new terms introduced ("hint database", "seeding tier", "seed state") to the appropriate file under `docs/glossary/` (likely `process-and-attribution.md`), following section 4.3, with primary-source references.
- [x] T018 [P] Add the `rusqlite` row to the AGENTS.md "Dependency inventory" table (Crate | runtime, optional | S034 | why) and a short justification paragraph consistent with the existing per-dependency prose.
- [x] T019 [P] Add `changelog.d/034-targets-hint-database.added.md` (one or two present-tense sentences on the store + export) and `changelog.d/034-targets-hint-database.decisions.md` (dated 2026-08-13: the rusqlite addition with default-features-off, the six-package delta, MSRV-1.82 verification, and MIT/public-domain licensing).
- [x] T020 Run `cargo xtask ci` in the foreground, watched to completion (fmt, clippy `--all-features -D warnings`, `test --workspace --locked`, lint, deps, license); fix to green.
- [x] T021 Run `cargo xtask msrv` in the foreground; confirm it builds the workspace (including bundled SQLite) through `rustup run 1.82` and exits 0 (a `2` is a not-run failure, not a pass).

## Dependencies & Execution Order

- Phase 1 (Setup) blocks everything.
- Phase 2 (Foundational) blocks Phases 3-5.
- Phase 3 (US1) is the MVP and precedes US2 (import reuses the store + export path).
- Phase 4 (US2) depends on US1's export and the store.
- Phase 5 (US3) depends only on the store (Phase 2); may run alongside US1/US2.
- Phase 6 polish runs after code lands; T020/T021 run last and gate the halt.

## Parallel Opportunities

- Within Phase 1: T001 and (after it) T002/T003/T004 are mostly sequential (same manifests); keep ordered.
- Phase 3: T009 and T010 (two test files) are `[P]` and precede T011.
- Phase 4: T012 (fixtures) is `[P]` with T013/T015 authoring.
- Phase 6: T017/T018/T019 are `[P]` (different files); T020/T021 are sequential and last.

## Implementation Strategy

Deliver the MVP first: Phases 1-3 give a store that round-trips to schema-valid
JSON, the slice's core contract. Then US2 makes it demonstrable through the CLI,
and US3 proves the tiered/resumable structure. Polish records the dependency
paperwork and runs the full gate set. Halt once before push.
