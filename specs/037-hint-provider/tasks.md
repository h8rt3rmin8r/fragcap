# Tasks: Hint database resolution provider (S037)

**Input**: Design documents from `specs/037-hint-provider/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/hint-provider.md

**Tests**: Included. The spec requires offline end-to-end coverage and enumerates
the behaviors to test; this project is test-driven.

**Organization**: By user story. The foundational phase (the `fragcap-profile`
contract changes) blocks every story because the request input, the target
origin, and the ambiguity note are shared vocabulary.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on an incomplete task)
- **[Story]**: US1-US4 for story-phase tasks; setup/foundational/polish carry none

## Path Conventions

Rust workspace under `crates/`. Docs under `docs/`. Changelog fragments under
`changelog.d/`.

---

## Phase 1: Setup

- [X] T001 Confirm the baseline gate is green before any change: run `cargo xtask ci` and note the pass, so a later failure is attributable to this slice.

---

## Phase 2: Foundational (BLOCKING - shared resolver vocabulary in fragcap-profile)

**Purpose**: Add the request input, the target origin, the ambiguity note, and the
`ProviderError::Hint` variant to `fragcap-profile`, and remove the stub. Every
story depends on these.

- [X] T002 Add `steam_app_id: Option<u32>` to `ResolutionRequest` in `crates/fragcap-profile/src/resolver.rs`: initialize it to `None` in `for_reference`, `for_observation`, and `for_install`; add the `with_steam_app_id(self, app_id: u32)` builder (mirroring `with_install_root`) and the `steam_app_id(&self)` accessor. Document each with the same idiom as the neighbors.
- [X] T003 Add `HintTarget` to `crates/fragcap-profile/src/target.rs` (fields `app_id: u32`, `image_name: String`, `identity: MatchPredicates`, `launcher_mediated: Option<bool>`, `engine: Option<String>`; `new(...)` constructor and accessors; derive `Clone, Debug, PartialEq, Eq`). Add the `TargetOrigin::HintDatabase(HintTarget)` variant. Update the `Target::profile()`, `Target::into_profile()`, and `Target::identity()` match arms (`HintDatabase(_) => None` for profile; `HintDatabase(t) => Some(t.identity())` for identity). Write the module doc note that a hint carries no `image_path` because the store knows no per-install path (P-9).
- [X] T004 Add `HintAmbiguity { app_id: u32, candidates: usize }` with accessors and a `Display` impl to `crates/fragcap-profile/src/resolver.rs` (mirroring `WalkerAmbiguity`). Add `hint_ambiguous: Option<HintAmbiguity>` to `ResolutionNotes` with `note_hint_ambiguous(app_id, candidates)`, and to `Unresolved` with the `hint_ambiguous()` accessor; thread it through `TargetResolver::resolve`.
- [X] T005 Add `ProviderError::Hint(String)` to `crates/fragcap-profile/src/resolver.rs` (a hard hint failure carrying a message; `fragcap-profile` names no `fragcap-targets` type). Extend the `Display` and `source()` impls for the new variant.
- [X] T006 Remove the `HintProvider` stub struct and its `TargetProvider` impl from `crates/fragcap-profile/src/providers.rs`, and update the `the_stub_providers_decline_at_their_precedence` test to drop the `HintProvider` assertions (the engine-rule decline stays). Update the module doc comment that described the hint stub to say the concrete provider now lives in `fragcap-targets` (mirroring the S030 platform-walker note).
- [X] T007 Export `HintTarget` (and `HintAmbiguity`) from `crates/fragcap-profile/src/lib.rs` so `fragcap-targets` and the facade can name them.
- [X] T008 [P] Unit tests in `crates/fragcap-profile/src/target.rs`: a `HintDatabase` target reports `profile() == None`, `into_profile() == None`, and `identity() == Some(...)` keyed on the executable; the carried `launcher_mediated`/`engine` round-trip through the accessors.
- [X] T009 [P] Unit test in `crates/fragcap-profile/src/resolver.rs`: a request built with `with_steam_app_id` exposes it through `steam_app_id()`, and a stub provider recording `note_hint_ambiguous` surfaces it through `Unresolved::hint_ambiguous()` when nothing resolves.

**Checkpoint**: `cargo test -p fragcap-profile` green; `fragcap-profile` compiles with no `fragcap-targets` dependency.

---

## Phase 3: US1 - A seeded title resolves from the hint database (P1)

**Goal**: A row with a launch executable resolves at heuristic-unverified with
`hint-db` provenance, and outranks the engine rule.

**Independent test**: Seed an in-memory store with one game row carrying one launch
executable, resolve that appid through `HintDatabaseProvider`, and assert the
stamped target.

- [X] T010 [US1] Add `Store::game(&self, appid: u32) -> Result<Option<Game>, TargetsError>` to `crates/fragcap-targets/src/store.rs`: a single-row `WHERE appid = ?` load reusing `load_launch` and `load_technologies`, returning `None` for an absent row.
- [X] T011 [P] [US1] Unit test in `crates/fragcap-targets/src/store.rs`: `game(appid)` returns a fully-hydrated row (with its launch entries) for a present appid and `None` for an absent one, over `Store::open_in_memory`.
- [X] T012 [US1] Create `crates/fragcap-targets/src/hint_provider.rs` with `HintDatabaseProvider { store: Store }`, `new(store)`, `precedence() == Precedence::HintDatabase`, and the resolve path of `provide`: read `steam_app_id`; look up the row; select the single Windows-applicable distinct executable; build `HintTarget` (identity `exe = name`, carry `launcher_mediated`/`engine`); return `Target::new(FidelityTier::HeuristicUnverified, Provenance::new("hint-db".into(), None), TargetOrigin::HintDatabase(t))`. Map a post-open `TargetsError` read failure to `ProviderError::Hint(e.to_string())`. Include the executable-selection helper as a pure function.
- [X] T013 [US1] Register the module and export `HintDatabaseProvider` from `crates/fragcap-targets/src/lib.rs`.
- [X] T014 [P] [US1] Unit test in `hint_provider.rs`: a row with one launch executable resolves at `HeuristicUnverified` with `hint-db` provenance, identity matching the executable, and carried `launcher_mediated`/`engine`; a row whose one executable repeats across several launch configs (different args/osarch) is one candidate, not ambiguous.
- [X] T015 [US1] Export `HintDatabaseProvider` from the facade's `#[cfg(feature = "targets")] pub mod targets` and `HintTarget` from `pub mod profile` in `crates/fragcap/src/lib.rs`.
- [X] T016 [P] [US1] Cascade-ordering test (in `crates/fragcap-targets/tests/` or the facade): a `TargetResolver` with `HintDatabaseProvider` and `EngineRuleProvider` resolves an appid request to the hint answer regardless of registration order (precedence 2 outranks 3).

**Checkpoint**: US1 independently testable; a seeded title resolves from the DB.

---

## Phase 4: US2 - The database defers rather than guesses (P1)

**Goal**: Sparse and ambiguous rows decline honestly so the cascade continues.

**Independent test**: Seed catalog-only, engine-only, and multi-executable rows;
assert each declines (the last with a recorded note).

- [X] T017 [US2] Complete the decline branches of `provide` in `crates/fragcap-targets/src/hint_provider.rs`: absent appid -> `Ok(None)`; absent row -> `Ok(None)`; empty Windows-applicable distinct-executable set -> `Ok(None)`; two or more -> `notes.note_hint_ambiguous(appid, n)` then `Ok(None)`. Ensure macOS/Linux-only launch entries are filtered out before the count.
- [X] T018 [P] [US2] Unit tests in `hint_provider.rs`: a Tier-1-only row (appid + name) declines; an engine-only row (engine set, empty launch) declines; a row with two distinct Windows executables declines and records `note_hint_ambiguous` with count 2; a request with no appid declines; a row whose only launch entry is macOS-only declines.
- [X] T019 [P] [US2] Cascade test: when the hint provider declines an ambiguous row and nothing lower resolves, `ResolutionError::Unresolved` exposes `hint_ambiguous()` with the appid and candidate count.

**Checkpoint**: every insufficient row falls through instead of guessing.

---

## Phase 5: US3 - No database is not an error (P1)

**Goal**: Feature off or no DB present leaves precedence 2 empty; a present-but-
unopenable DB is a loud error.

**Independent test**: Assemble the CLI resolver with no hint DB and confirm the
outcome matches the no-provider cascade; confirm an unopenable path is a `CliError`.

- [X] T020 [US3] Add the `--hint-db <PathBuf>` option to `RunArgs` in `crates/fragcap-cli/src/cli.rs` (optional, documented), and add `hint_db_path(flag: Option<&Path>) -> Option<PathBuf>` to `crates/fragcap-cli/src/paths.rs` reading the flag then the `FRAGCAP_HINT_DB` environment override (mirroring `PROFILE_DIR_ENV`).
- [X] T021 [US3] Add a resolver-assembly helper (for example `crate::assemble::build_resolver`) in `crates/fragcap-cli/src/` that builds the provider vector: `ProfileProvider`, `EngineRuleProvider`, `SteamWalkerProvider`, `ObservationProvider`, and, under `#[cfg(feature = "targets")]` only, `HintDatabaseProvider` when `hint_db_path` resolves to a present file (opening the store, mapping a `Store::open` failure to a `CliError`, FR-014). Use it from `crates/fragcap-cli/src/commands/run.rs`.
- [X] T022 [US3] Update `crates/fragcap-cli/src/commands/run.rs`: drop the `HintProvider` import from `fragcap::profile`, resolve via the helper, and on the `--steam <app_id>` path parse the appid to `u32` and attach it with `with_steam_app_id`. Update `crates/fragcap-cli/src/attach.rs` to drop the removed `HintProvider` import (its observation-only resolver omits precedence 2, behavior identical to the old no-answer stub).
- [X] T023 [P] [US3] CLI/integration tests: with no `--hint-db` and no `FRAGCAP_HINT_DB`, resolution behavior is identical to the pre-slice cascade (same target or same not-resolved outcome); a `--hint-db` pointing at a present seeded store resolves an appid; a `--hint-db` pointing at an unopenable file is a `CliError`; a `--hint-db` pointing at a non-existent path is not an error and leaves precedence 2 empty.

**Checkpoint**: graceful degradation proven; missing DB silent, broken DB loud.

---

## Phase 6: US4 - A live process still overrides a stale hint (P2)

**Goal**: A hint answer is always heuristic-unverified and carries the identity the
live path re-matches; a profile outranks it.

**Independent test**: Assert the fidelity stamp and that a profile provider wins
over the hint provider for the same appid request.

- [X] T024 [P] [US4] Cascade test: a `TargetResolver` with `ProfileProvider` and `HintDatabaseProvider` resolves a request that both can answer to the profile answer, regardless of registration order (precedence 1 outranks 2); the hint answer's fidelity is never `Observed` or `Authored`.
- [X] T025 [P] [US4] Test that the hint target's carried identity is a valid capture identity: it round-trips through the same `MatchPredicates` the non-profile `run` path serializes, so a live process is bound by the executable name the hint named (the override/refine path).

**Checkpoint**: an inferred hint never masquerades as observed; live truth can override.

---

## Phase 7: Polish & Cross-Cutting

- [X] T026 [P] Add a "Hint provider" glossary term (and honest-answer wording) to `docs/glossary/process-and-attribution.md`, cross-linked to Provider, Hint database, Engine rule, Fidelity tier; regenerate the index via the docs linter (P-6).
- [X] T027 [P] Add a changelog fragment `changelog.d/S037.feature.md` describing the wiring, and, if any choice proves architecture-affecting (the `--hint-db`/`FRAGCAP_HINT_DB` supply mechanism), a dated `changelog.d/S037.decisions.md` fragment.
- [X] T028 Run the full gate in the foreground and watch to completion: `cargo xtask ci` (fmt, clippy --all-targets --all-features, test --workspace --locked, lint, deps, license) and `cargo xtask msrv`. Resolve any failure within the slice.
- [X] T029 Final review pass: confirm no em-dashes/en-dashes, UTF-8/LF, no process handle or transmit call introduced, and that `cargo xtask deps` shows no `fragcap-profile -> fragcap-targets` edge.

---

## Dependencies & Execution Order

- **Setup (T001)** -> **Foundational (T002-T009)** blocks everything.
- **US1 (T010-T016)** depends on Foundational; it is the MVP.
- **US2 (T017-T019)** depends on US1 (extends the same `provide`).
- **US3 (T020-T023)** depends on US1 (needs the concrete provider to inject) and
  Foundational (needs the request input and stub removal).
- **US4 (T024-T025)** depends on US1 and the profile provider (unchanged).
- **Polish (T026-T029)** last; T028 is the authoritative gate.

## Parallel Opportunities

- Foundational test tasks T008, T009 are parallel to each other.
- Within US1: T011, T014, T016 are parallel once T010/T012/T013 land.
- US2 tests T018, T019 parallel; US4 tests T024, T025 parallel.
- Docs/changelog T026, T027 parallel with each other.

## MVP Scope

**US1 alone** (T001-T016) is a viable MVP: a seeded title resolves from the hint
database at the correct fidelity and precedence. US2 (honest declines), US3
(graceful degradation), and US4 (override) complete the slice's done-gate.

## Independent Test Criteria

- **US1**: an in-memory store row with one launch executable resolves at
  heuristic-unverified with `hint-db` provenance.
- **US2**: catalog-only, engine-only, and multi-executable rows each decline (the
  last with a note).
- **US3**: no-DB resolution equals the pre-slice cascade; unopenable DB is a
  `CliError`; missing DB is silent.
- **US4**: a profile outranks a hint; the hint stamp is never observed/authored.
