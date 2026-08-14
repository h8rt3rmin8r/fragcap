# Tasks: Local Steam launch-data accumulation

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Branch**: `038-launch-accumulation`

Tests are included: the constitution mandates test-driven discipline, and every
behavior here is offline-testable (synthetic appinfo bytes, a fixture Steam-root
tree, an in-memory store).

Absolute crate roots: `crates/fragcap-steam`, `crates/fragcap-targets`,
`crates/fragcap`, `crates/fragcap-cli`.

## Phase 1: Setup

- [x] T001 [P] Add glossary entries for "application-info cache", "launch-data accumulation", "accumulation account", and "change-number staleness" to the project glossary (docs, per P-6), each with a primary-source reference, and add a short specification subsection under the Steam / hints sections of `docs/fragcap-specification.md` describing local launch accumulation and its passivity.

## Phase 2: Foundational (blocking prerequisites for all stories)

- [x] T002 Add the store migration in `crates/fragcap-targets/src/schema.rs` and `crates/fragcap-targets/src/store.rs`: bump `SCHEMA_VERSION` to 2, add nullable `appinfo_change_number INTEGER` to the `games` DDL, and add the v1 to v2 path in `from_connection` (`ALTER TABLE games ADD COLUMN appinfo_change_number INTEGER`, then stamp user_version 2; keep the fresh-store and future-version arms).
- [x] T003 [P] Write the migration test in `crates/fragcap-targets/src/store.rs` (tests): a store stamped at v1 with the pre-column DDL opens under this build, migrates to v2 with existing rows' `appinfo_change_number` NULL, and reads/writes correctly afterward (backward-safe, SC row for CHK015).
- [x] T004 [P] Create `crates/fragcap-steam/src/appinfo.rs` with the parse types (`SteamLaunchEntry`, `AppInfoApp`, `AppInfoFailure`, `AppInfoParse`) and `parse_appinfo(&[u8]) -> AppInfoParse`: read the header (magic v27/v28/v29, universe, v29 string table), iterate size-framed app sections extracting `appid` and `change_number`, parse each section's binary key-values into a `vdf::VdfValue`, and extract `config/launch/*` into `SteamLaunchEntry` values verbatim. A bad section is counted in `failures` and resynced by its size (FR-008, P-4).
- [x] T005 [P] Add a synthetic appinfo byte generator to `crates/fragcap-steam/src/test_support.rs` (build files for given apps in both the inline-key and string-table header variants) and parser unit tests in `crates/fragcap-steam/src/appinfo.rs` (tests): single Windows executable; multiple os-filtered entries preserved in order (P-9); malformed middle section isolates and resyncs; unrecognized magic yields one failure and no apps; truncation yields the apps read plus a trailing failure.
- [x] T006 Add `read_appinfo(root: &Path) -> Result<AppInfoParse, SteamError>` to `crates/fragcap-steam/src/appinfo.rs` (reads `root/appcache/appinfo.vdf`; missing file is `Ok` with an empty parse; present-but-unreadable is `SteamError::Io`) and export the appinfo surface plus `read_appinfo` from `crates/fragcap-steam/src/lib.rs`.

## Phase 3: User Story 1 - Learn a game's launch executable (Priority: P1)

**Goal**: A first run records an installed game's launch executable(s) from the
local cache into the local store.

**Independent test**: Against a fixture Steam-root tree (appmanifests plus a
synthetic appinfo file) and an empty in-memory store, accumulation ends with the
apps' launch rows stored and a `written`/`considered` summary.

- [x] T007 [P] [US1] Add `Store::merge_launch(appid, change_number, &[LaunchEntry])` to `crates/fragcap-targets/src/store.rs`: ensure the games row exists (`INSERT OR IGNORE`), set `appinfo_change_number`, and replace this appid's `launch_entries` in order, all in one transaction, touching no Tier 1/3, `launcher_mediated`, or `token_required` column (mirrors `merge_catalog`/`merge_engine`).
- [x] T008 [P] [US1] Write the tiers-preserved unit test in `crates/fragcap-targets/src/store.rs` (tests): a game pre-seeded with catalog and engine columns keeps name, metrics, and engine after `merge_launch`, gaining only launch rows and the change-number (CHK016).
- [x] T009 [US1] Create `crates/fragcap/src/accumulate.rs` (behind `#[cfg(feature = "targets")]`) with `LaunchAccumulationSummary` (+ `is_conserved`) and `accumulate_launch_data(root, store, report)` write path: enumerate installed titles via `fragcap_steam::discover_in`, read the appinfo parse, and for each installed appid present with storable entries, map `SteamLaunchEntry` to `fragcap_targets::LaunchEntry` and `merge_launch`; export it from `crates/fragcap/src/lib.rs` behind the feature.
- [x] T010 [US1] Write the first-run end-to-end test in `crates/fragcap/tests/launch_accumulation.rs` (`--features targets`): a fixture root with two installed appmanifests and a synthetic appinfo file, against an empty in-memory store, ends with launch rows for both and `written = 2`, `considered = 2`, conserved.
- [x] T011 [US1] Wire accumulation into `crates/fragcap-cli/src/commands/run.rs`: when a hint-database path is present (existing `--hint-db` / `FRAGCAP_HINT_DB`) and this build carries `targets`, open that store, call `accumulate_launch_data` with a progress printer, and proceed to `build_resolver`; no hint database means no accumulation (unchanged behavior). Add a `run.rs` test that a present hint database triggers the accumulation path.

## Phase 4: User Story 2 - Skip known, refresh only what changed (Priority: P2)

**Goal**: Repeat runs skip apps already current and re-read only apps whose
appinfo change-number advanced.

**Independent test**: Re-running against an unchanged fixture reports all skipped;
advancing one app's change-number re-reads exactly that app.

- [x] T012 [P] [US2] Add `Store::stored_change_number(appid) -> Result<Option<u32>, TargetsError>` to `crates/fragcap-targets/src/store.rs` with a unit test (returns the recorded value, `None` for an unseen appid or a never-learned one).
- [x] T013 [US2] Add staleness to `accumulate_launch_data` in `crates/fragcap/src/accumulate.rs`: compare each `AppInfoApp::change_number` against `stored_change_number`; skip when not greater (count `skipped`), refresh when greater (count `written`). Never prune an app absent from the cache.
- [x] T014 [US2] Add facade tests in `crates/fragcap/tests/launch_accumulation.rs`: a second run over the unchanged fixture reports `skipped = 2`, `written = 0`; after bumping one app's change-number, the next run reports `written = 1`, `skipped = 1`, and leaves the other app's rows untouched.

## Phase 5: User Story 3 - An honest account of the walk (Priority: P3)

**Goal**: Every considered app lands in exactly one counted outcome, failures do
not abort, and progress is surfaced.

**Independent test**: A mixed fixture (writable, current, malformed,
appinfo-absent) yields a summary whose buckets sum to `considered`.

- [x] T015 [US3] Complete outcome classification in `crates/fragcap/src/accumulate.rs`: fold `AppInfoParse::failures` into `failed`, classify an installed app absent from appinfo or with no storable entry as `empty` (not failed, FR-009), and assert `is_conserved` internally; add `AccumulationProgress` and drive the `report` callback per considered app.
- [x] T016 [US3] Make `crates/fragcap-cli/src/commands/run.rs` print a bounded progress line from the `report` callback (a considered/total counter, so a slow first run reads as working, FR-010), and surface the final summary counts to the operator.
- [x] T017 [US3] Add the conservation facade test in `crates/fragcap/tests/launch_accumulation.rs`: a fixture mixing a writable app, an already-current app, a malformed appinfo section, and an installed app absent from appinfo yields `written + skipped + failed + empty == considered`, and the malformed section does not block the writable app (FR-007, FR-008, SC-004, SC-005).

## Phase 6: Polish & cross-cutting

- [x] T018 [P] Add changelog fragments: `changelog.d/038-launch-accumulation.added.md` (the feature line) and `changelog.d/038-launch-accumulation.decisions.md` (the dated decision recording the store's first v1 to v2 migration and the per-user local-accumulation model replacing the cancelled maintainer Tier 2 seeder).
- [x] T019 [P] Confirm no new dependency: `git diff --exit-code Cargo.lock` is clean, and `cargo xtask deps` shows the crate graph unchanged.
- [x] T020 Run the full gate in the foreground and resolve any finding: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace --locked`, `cargo xtask lint`, `cargo xtask deps`, `cargo xtask license`, then `cargo xtask msrv` (1.82).

## Dependencies & completion order

- Phase 1 (T001) is independent and may run anytime before commit.
- Phase 2 (T002 to T006) blocks all user stories. Within it: T002 before T003; T004 before T005 and T006. T002 and T004 are parallel (different crates).
- US1 (Phase 3) depends on Phase 2. US1 is the MVP.
- US2 (Phase 4) depends on US1 (extends the same orchestrator and store).
- US3 (Phase 5) depends on US1 and is best after US2 (it finalizes the buckets the staleness path feeds).
- Phase 6 depends on all prior phases.

## Parallel execution examples

- Foundational: T002 (targets migration) and T004 (steam parser) run in parallel;
  T003 and T005 (their tests) follow each respectively.
- US1: T007/T008 (targets `merge_launch` + test) run parallel to T009 scaffolding
  once T004 and T002 land.
- Polish: T018 and T019 are parallel.

## Implementation strategy

- **MVP = User Story 1**: parser + migration + `merge_launch` + facade write path
  + CLI wiring. Delivers standalone value: one installed game learned is one the
  resolver can now name.
- **Increment US2**: staleness so repeat runs are cheap.
- **Increment US3**: the honest account and progress, hardening P-4/P-9.
- Keep each phase green under `cargo xtask ci` before starting the next.

## Implementation notes (deviations from the task text)

- The end-to-end tests (T010, T014, T017) were placed in
  `crates/fragcap-cli/tests/launch_accumulation.rs` rather than
  `crates/fragcap/tests/`. Reason: the facade is tested with default features
  under `cargo test --workspace`, where `targets` is off, so a `targets`-gated
  facade test would not run in the gate. The CLI carries `targets`
  unconditionally, so the same test runs there in CI. The orchestrator under test
  still lives in the facade (`crates/fragcap/src/accumulate.rs`). The synthetic
  appinfo generator is shared via a new off-by-default `test-support` feature on
  `fragcap-steam`, so there is one encoder of the format.
- T011's suggested `run.rs` unit test was not added: the CLI wrapper
  `accumulate_launch` calls `accumulate_from_local_steam`, which discovers the
  real Steam root, so a unit test would be non-deterministic (or find no Steam and
  test nothing). The orchestrator it wraps is covered deterministically by the
  four CLI integration tests against a fixture root.
- The only `Cargo.lock` change is an internal dev-dependency edge
  (`fragcap-cli` -> `fragcap-steam`, an already-present workspace crate); no new
  third-party package. `cargo xtask deps` ignores dev-dependencies, so the crate
  graph is unchanged (FR-014 holds).
