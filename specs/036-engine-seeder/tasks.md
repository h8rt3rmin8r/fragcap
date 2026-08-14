---
description: Dependency-ordered tasks for S036 Tier 3 Engine Seeder (PCGamingWiki)
---

# Tasks: Tier 3 Engine Seeder (PCGamingWiki)

**Slice**: S036 (issue #78) | **Branch**: `036-engine-seeder`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

**Format**: `[ID] [P?] [Story?] Description with file path`
`[P]` = parallelizable. `[USn]` = serves User Story n.

MVP = User Story 1 (seed engine attributions, offline-tested, with a truthful
summary). Test-driven: the offline `FixtureEngineFeed` drives every test; the live
`HttpEngineFeed` is compiled under the existing `net` feature but never run in CI.
No new dependency is taken (the `net` feature and `http_req` already exist from
S035), so there is no Setup phase.

## Phase 1: Foundational (blocking prerequisites)

**Goal**: The engine-source abstraction, the reused summary, and the per-tier merge
exist. Blocks every user story.

- [ ] T001 Implement `crates/fragcap-targets/src/engine_feed.rs`: the `EngineFeed` trait (`fetch_batch(cursor) -> Result<EngineBatch, TargetsError>`), `EngineBatch { entries, failed, next_cursor }`, `EngineEntry { appid, engine: Option<ResolvedEngine> }`, `ResolvedEngine { name: String, confidence: EngineConfidence }`, and `FixtureEngineFeed` (parse a committed JSON array; per-entry `appid` required u32 else failed; `engine` resolves string/one-element-array -> name, absent/null/""/[] -> None, multi-element array -> None ambiguous, other JSON type -> failed; `confidence` absent -> documented default token, valid token -> that value, wrong-typed/out-of-set -> failed; paginate the in-memory list by an opaque consumed-count offset cursor (not an appid cursor, so duplicate appids straddling a page are not skipped); a non-array document is `TargetsError::Seed`).
- [ ] T002 Add `Store::merge_engine(appid, engine: &Engine)` to `crates/fragcap-targets/src/store.rs`: `INSERT INTO games (appid, engine_name, engine_source, engine_confidence) VALUES (...) ON CONFLICT(appid) DO UPDATE SET` of only the three engine columns; binds source and confidence together (both-or-neither), `engine_name` NULL when `None`; never references name/metrics/launcher/token/launch/technologies; inserts an engine-only row for an unseen appid.
- [ ] T003 Add `seed_engine(store, source: &dyn EngineFeed, now)` to `crates/fragcap-targets/src/seed.rs`, reusing `SeedSummary`: resume from the ENGINE `seed_state` cursor; per page add `batch.failed` to `fetched` and `failed`; per entry `fetched += 1`; `Some` engine + new appid -> `merge_engine` + `written`, `Some` + repeat appid -> `merge_engine` + `duplicates`, `None` -> `excluded`; write the cursor + timestamp per batch under `SeedTier::Engine`; never prune. Re-export `EngineFeed`, `EngineBatch`, `EngineEntry`, `ResolvedEngine`, `FixtureEngineFeed`, and `seed_engine` in `crates/fragcap-targets/src/lib.rs` (trait named `EngineFeed` so it does not clash with the existing `EngineSource` enum).

**Checkpoint**: a unit test can seed an in-memory store from a small `FixtureEngineFeed` and read the engine columns back.

## Phase 2: User Story 1 - Seed engine attributions, offline-tested (P1)

**Goal**: A fixture-backed engine seed fills the engine columns for exactly the
resolved titles, the summary reconciles, and the store exports schema-valid.

**Independent test**: drive `FixtureEngineFeed` with clear engines (varied
confidences), a no-engine title, an ambiguous-engine title, and a malformed entry;
assert engine columns, conservation, and a valid export (including an engine-only
row).

- [ ] T004 [P] [US1] Write `crates/fragcap-targets/tests/fixtures/engine.json`: entries covering a clear single-string engine (each of several confidences incl. an omitted-confidence entry using the default), a one-element-array engine (resolved), a multi-element-array engine (ambiguous -> excluded), an absent/empty engine (no engine -> excluded), an appid not previously in the store (engine-only row), a wrong-typed appid (failed), and an out-of-set confidence token (failed).
- [ ] T005 [US1] Write `crates/fragcap-targets/tests/engine_seed.rs`: seed the fixture into an in-memory store; assert the engine columns are set for exactly the resolved titles (source `pcgamingwiki`, expected confidences) and unset for the excluded ones; assert `fetched == written + excluded + duplicates + failed`; assert the malformed entries are counted failed and did not abort; assert `export` validates via `fragcap_profile::jsonschema::validate_json` and an engine-only row exports (app_id + engine, no name); assert every record's fidelity is heuristic-unverified.

**Checkpoint**: `cargo test -p fragcap-targets` green; MVP delivered.

## Phase 3: User Story 2 - Enrich engine without disturbing catalog or launch (P1)

**Goal**: The per-tier engine merge writes only the engine columns.

**Independent test**: give a title a catalog name and launch entries, seed engine
over it, assert the name and launch survive; assert a stored title absent from the
run is untouched.

- [ ] T006 [US2] Write `crates/fragcap-targets/tests/engine_tiers.rs`: upsert a full game (name + launcher_mediated + launch entries, and optionally a pre-existing engine from a different source) via the S034 path, then `seed_engine` over a fixture naming that appid's engine; assert the engine columns are now pcgamingwiki + the seeded name/confidence, and the name, launcher flag, and launch entries are unchanged (the SC-003 "the name survives" assertion); assert a stored game absent from the run is left untouched (no prune).

## Phase 4: User Story 3 - Resume without restarting (P2)

**Goal**: A partial engine seed records its cursor under the engine tier; a second
run continues from it.

**Independent test**: seed part of an engine universe, then resume; assert the final
result equals a single seed with no duplicate rows.

- [ ] T007 [US3] Write `crates/fragcap-targets/tests/engine_resume.rs`: with a small batch size, run a seed that processes part of a batched fixture and records the ENGINE-tier cursor; run again against the same store; assert the second run resumes from the recorded cursor, the final engine set equals an uninterrupted seed with no duplicate rows, and the engine `seed_state` records the last run.

## Phase 5: Live source (behind `net`, compiled-not-run)

**Goal**: The real HTTP source exists and compiles under the all-features gate.

- [ ] T008 Implement `crates/fragcap-targets/src/http_engine.rs` under `#[cfg(feature = "net")]`: `HttpEngineFeed` implementing `EngineFeed` via `http_req` read-only HTTPS GETs against PCGamingWiki's MediaWiki Cargo query API (`Infobox_game` table: Steam_AppID, page name, Engine; offset cursor); map a blank engine field to `None`, a single engine to a resolved name with the documented default confidence (`high`), and a multi-engine field to `None` (ambiguous). No process handle, no capture (P-1). Add its module cfg and re-export under `#[cfg(feature = "net")]` in `fragcap-targets/src/lib.rs` and the facade `crates/fragcap/src/lib.rs`.

## Phase 6: CLI

- [ ] T009 [US1] Add the `targets seed-engine` subcommand to `crates/fragcap-cli/src/cli.rs` and handle it in `crates/fragcap-cli/src/commands/targets.rs`: `--from <FILE> --db <DB>` drives `FixtureEngineFeed` offline and prints the summary (0/1/2 exit contract); a `--pcgamingwiki` form behind `#[cfg(feature = "net")]` drives `HttpEngineFeed`; the two are a mutually exclusive clap group. A default build reports that live engine seeding needs `net` rather than offering a dead flag.
- [ ] T010 [P] [US1] Write CLI tests (inline in `commands/targets.rs`, `tempfile`): `seed-engine --from <fixture>` then `export` round-trips to schema-valid JSON offline and prints a summary carrying `pcgamingwiki`; a bad `--from` file exits 1.

## Phase 7: Polish & Cross-Cutting

- [ ] T011 [P] Add/confirm glossary entries (P-6) for the engine-seeder terms (engine seeder, engine feed, engine attribution as a Tier 3 write, engine confidence as a within-field grade) in `docs/glossary/process-and-attribution.md`, then regenerate the index (`bash scripts/lint-docs.sh fix`) and confirm `bash scripts/lint-docs.sh check` passes. Reuse S033's engine-attribution glossary entry where it already covers the term rather than duplicating.
- [ ] T012 [P] Add `changelog.d/036-engine-seeder.added.md` and `changelog.d/036-engine-seeder.decisions.md` (dated 2026-08-13: the no-new-dependency reuse of http_req/net; the `merge_engine` per-tier merge; the `EngineFeed` trait name chosen to avoid the `EngineSource` enum clash; the `--pcgamingwiki` live flag naming its true source rather than `--steam`; the documented default confidence `high`).
- [ ] T013 Run `cargo xtask ci` in the foreground, watched to completion (fmt, clippy `--all-features -D warnings` which compiles the `net` graph incl. `HttpEngineFeed`, test --workspace --locked which runs only offline tests, lint, deps, license, docs drift); fix to green.
- [ ] T014 Run `cargo xtask msrv` in the foreground; confirm the default-feature workspace (no `net`, no http_req) builds through `rustup run 1.82` and exits 0.

## Dependencies & Execution Order

- Phase 1 (Foundational) blocks Phases 2-6.
- Phase 2 (US1) is the MVP; US2 (Phase 3) and US3 (Phase 4) build on the seeder and
  merge but are independent of each other.
- Phase 5 (live source) depends only on the trait (Phase 1); it is compiled-not-run.
- Phase 6 (CLI) depends on US1 (offline seed) and, for `--pcgamingwiki`, Phase 5.
- Phase 7 polish runs after code lands; T013/T014 run last and gate the halt.

## Parallel Opportunities

- Phase 2: T004 (fixture) is `[P]` and precedes T005.
- Phases 3 and 4 test files are independent and may be written in parallel.
- Phase 7: T011/T012 are `[P]` (different files); T013/T014 are sequential and last.

## Implementation Strategy

Deliver the MVP first: Phases 1-2 give an offline-tested engine seeder that fills
the engine columns with a truthful summary and a schema-valid export. Then US2 and
US3 prove the non-clobbering merge and resumability, and the live source + CLI make
it operable. Polish records the decisions and runs the full gate set. Halt once
before push.
