---
description: Dependency-ordered tasks for S035 Tier 1 Catalog Seeder
---

# Tasks: Tier 1 Catalog Seeder

**Slice**: S035 (issue #78) | **Branch**: `035-catalog-seeder`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

**Format**: `[ID] [P?] [Story?] Description with file path`
`[P]` = parallelizable. `[USn]` = serves User Story n.

MVP = User Story 1 (seed the corpus, offline-tested, with a truthful summary).
Test-driven: the offline `FixtureCatalog` drives every test; the live `HttpCatalog`
is compiled under `net` but never run in CI.

## Phase 1: Setup

**Goal**: The `net` feature and the optional HTTP client are wired, with MSRV and
the default build unaffected.

- [x] T001 Add `http_req` to root `Cargo.toml` `[workspace.dependencies]`: `http_req = { version = "0.13", default-features = false, features = ["native-tls"] }` with the multi-line justification comment (unique property, 18-package delta, all MIT/Apache, native-tls chosen over rustls to avoid the CDLA-Permissive-2.0 webpki-roots and over ureq to avoid the ICU4X graph, MSRV non-binding because `net` is off by default).
- [x] T002 Add the `net` feature to `crates/fragcap-targets/Cargo.toml` (`net = ["dep:http_req"]`) and the optional dep `http_req = { workspace = true, optional = true }`.
- [x] T003 Add the `net` passthrough to `crates/fragcap/Cargo.toml` (`net = ["fragcap-targets/net"]`, off by default) and `crates/fragcap-cli/Cargo.toml` (`net = ["fragcap/net"]`, off by default).

**Checkpoint**: `cargo build -p fragcap` (no feature) compiles no http_req; `cargo build -p fragcap-targets --features net` compiles it; `cargo xtask deps` passes.

## Phase 2: Foundational (blocking prerequisites)

**Goal**: The source abstraction, the gate, the summary, and the per-tier merge
exist. Blocks every user story.

- [x] T004 Implement `crates/fragcap-targets/src/catalog.rs`: the `CatalogSource` trait, `CatalogBatch`, `CatalogEntry`, `Classification`, and `FixtureCatalog` (parse a committed JSON catalog document; paginate its in-memory list by an opaque cursor; a malformed document is `TargetsError::Seed`).
- [x] T005 Implement `crates/fragcap-targets/src/gate.rs`: `CorpusGate { min_reviews }` with `admits(&entry)` (game classification AND review_count present AND >= threshold); a missing signal excludes.
- [x] T006 Add `Store::merge_catalog(appid, name, review_count, owners, peak_ccu)` to `crates/fragcap-targets/src/store.rs`: `INSERT ... ON CONFLICT(appid) DO UPDATE SET` of only the Tier 1 columns; maps empty/absent name to NULL (S034 guard); never references launcher_mediated/token_required/engine/launch/technologies.
- [x] T007 Implement `crates/fragcap-targets/src/seed.rs`: `SeedSummary { fetched, written, excluded, failed }` and `seed_catalog(store, source, gate, now)`: resume from the catalog `seed_state` cursor, count every entry, merge admitted ones, catch per-entry errors as `failed` without aborting, write the cursor + timestamp per batch, never prune. Re-export the new surface in `crates/fragcap-targets/src/lib.rs`.

**Checkpoint**: a unit test can seed an in-memory store from a small `FixtureCatalog` and read the rows back.

## Phase 3: User Story 1 - Seed the corpus, offline-tested (P1)

**Goal**: A fixture-backed seed fills the store with exactly the in-corpus titles,
the summary reconciles, and the store exports schema-valid.

**Independent test**: drive `FixtureCatalog` with in-corpus games, out-of-corpus
entries, and a failing entry; assert rows, conservation, and a valid export.

- [x] T008 [P] [US1] Write `crates/fragcap-targets/tests/fixtures/catalog.json`: a mix of in-corpus games (game + reviews over threshold), out-of-corpus entries (non-game, and game below threshold), a nameless in-corpus game, and an entry designed to fail per-entry processing.
- [x] T009 [US1] Write `crates/fragcap-targets/tests/catalog_seed.rs`: seed the fixture, assert the store holds exactly the in-corpus titles (including the nameless one, name absent), assert `fetched == written + excluded + failed`, assert the failing entry is counted as failed and did not abort the run, and assert `export` validates via `fragcap_profile::jsonschema::validate_json`.

**Checkpoint**: `cargo test -p fragcap-targets` green; MVP delivered.

## Phase 4: User Story 2 - Resume without restarting (P2)

**Goal**: A partial seed records its cursor; a second run continues from it.

**Independent test**: seed part of a catalog, then resume; assert the final corpus
equals a single seed with no duplicate rows.

- [x] T010 [US2] Write `crates/fragcap-targets/tests/catalog_resume.rs`: run a seed that processes part of a batched fixture and records the cursor; run again against the same store; assert the second run resumes from the recorded cursor and the final corpus equals an uninterrupted seed, with the catalog `seed_state` recording the last run.

## Phase 5: User Story 3 - Refresh Tier 1 without disturbing other tiers (P2)

**Goal**: The per-tier merge updates only catalog columns.

**Independent test**: give a title an engine and launch entries, seed Tier 1 over it
with a new name, assert the engine and launch survive.

- [x] T011 [US3] Write `crates/fragcap-targets/tests/catalog_tiers.rs`: upsert a full game (engine + launch entries) via the S034 path, then `seed_catalog` over a catalog carrying that appid with a new name; assert the name updated and the engine and launch entries are unchanged; assert a stored game absent from the run is left untouched (no prune).

## Phase 6: Live source (behind `net`, compiled-not-run)

**Goal**: The real HTTP source exists and compiles under the all-features gate.

- [x] T012 Implement `crates/fragcap-targets/src/http_catalog.rs` under `#[cfg(feature = "net")]`: `HttpCatalog` implementing `CatalogSource` via `http_req` read-only HTTPS GETs against the public Steam Web API (app-list paginated by last-appid as the cursor; map responses to `CatalogEntry`). No process handle, no capture (P-1). Re-export under `#[cfg(feature = "net")]` in `fragcap-targets` and the facade `crates/fragcap/src/lib.rs`.

## Phase 7: CLI

- [x] T013 [US1] Add the `targets seed` subcommand to `crates/fragcap-cli/src/cli.rs` and handle it in `crates/fragcap-cli/src/commands/targets.rs`: `--from <FILE> --db <DB> [--min-reviews N]` drives `FixtureCatalog` offline and prints the summary (0/1/2 exit contract); a `--steam` form behind `#[cfg(feature = "net")]` drives `HttpCatalog`; the two are a mutually exclusive clap group. A default build reports that live seeding needs `net` rather than offering a dead flag.
- [x] T014 [P] [US1] Write CLI tests (inline in `commands/targets.rs`, `tempfile`): `seed --from <fixture>` then `export` round-trips to schema-valid JSON offline and prints a summary; a bad `--from` file exits 1.

## Phase 8: Polish & Cross-Cutting

- [x] T015 [P] Add glossary entries (P-6) for new terms (catalog seeder, corpus gate, catalog source, seed summary) to `docs/glossary/process-and-attribution.md`, then regenerate the index (`bash scripts/lint-docs.sh fix`) and confirm `bash scripts/lint-docs.sh check` passes.
- [x] T016 [P] Add the `http_req` row to the AGENTS.md "Dependency inventory" table and a justification paragraph consistent with the existing per-dependency prose (the ICU4X and CDLA rejections; MSRV non-binding because net is off by default).
- [x] T017 [P] Add `changelog.d/035-catalog-seeder.added.md` and `changelog.d/035-catalog-seeder.decisions.md` (dated 2026-08-13: the http_req/native-tls choice, the ureq/ICU4X and minreq/CDLA rejections, the 18-package delta, MSRV analysis, and the per-tier-merge decision).
- [x] T018 Run `cargo xtask ci` in the foreground, watched to completion (fmt, clippy `--all-features -D warnings` which compiles the `net` graph, test --workspace --locked which runs only offline tests, lint, deps, license, docs); fix to green.
- [x] T019 Run `cargo xtask msrv` in the foreground; confirm the default-feature workspace (no `net`, no http_req) builds through `rustup run 1.82` and exits 0.

## Dependencies & Execution Order

- Phase 1 (Setup) blocks everything.
- Phase 2 (Foundational) blocks Phases 3-7.
- Phase 3 (US1) is the MVP; US2 (Phase 4) and US3 (Phase 5) build on the seeder and
  merge but are independent of each other.
- Phase 6 (live source) depends only on the trait (Phase 2); it is compiled-not-run.
- Phase 7 (CLI) depends on US1 (offline seed) and, for `--steam`, Phase 6.
- Phase 8 polish runs after code lands; T018/T019 run last and gate the halt.

## Parallel Opportunities

- Phase 3: T008 (fixture) is `[P]` and precedes T009.
- Phases 4 and 5 test files are independent and may be written in parallel.
- Phase 8: T015/T016/T017 are `[P]` (different files); T018/T019 are sequential and
  last.

## Implementation Strategy

Deliver the MVP first: Phases 1-3 give an offline-tested seeder that fills the
corpus with a truthful summary and a schema-valid export. Then US2 and US3 prove
resumability and the non-clobbering merge, and the live source + CLI make it
operable. Polish records the dependency and runs the full gate set. Halt once
before push.
