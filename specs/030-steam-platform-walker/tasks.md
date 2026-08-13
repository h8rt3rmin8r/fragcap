---
description: "Task list for S030 Steam Platform-Walker Refactor"
---

# Tasks: Steam Platform-Walker Refactor

**Input**: Design documents from `specs/030-steam-platform-walker/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. This repository works under test-driven discipline
(constitution, autopilot protocol); the composition, degradation, and
dependency-direction tests are required, not optional.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 / US2 / US3, mapping to the spec's user stories

## Path Conventions

Rust workspace. Walker provider in `crates/fragcap-steam/src/`; cascade
vocabulary (origin, notes) in `crates/fragcap-profile/src/`; resolver assembly in
`crates/fragcap-cli/src/commands/`; docs in `docs/`.

---

## Phase 1: Setup

- [X] T001 Declare a new `pub mod walker;` in
  `crates/fragcap-steam/src/lib.rs` pointing at a new empty
  `crates/fragcap-steam/src/walker.rs`, so later tasks add to a compiling module;
  add no re-exports yet.

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The cascade vocabulary the walker needs (origin, notes) and the
shared classifier surface. No story is testable until these exist.

- [X] T002 In `crates/fragcap-profile/src/target.rs`, add `WalkerTarget`
  (`platform: String`, `image_name: String`, `image_path: String`,
  `identity: MatchPredicates`; derive `Clone, Debug, PartialEq, Eq`) with `new`
  and `platform()`/`image_name()`/`image_path()`/`identity()` accessors, and add
  the `PlatformWalker(WalkerTarget)` variant to `TargetOrigin`; make `profile()`
  and `into_profile()` return `None` for it (data-model, FR-001).
- [X] T003 In `crates/fragcap-profile/src/resolver.rs`, extend `ResolutionNotes`
  with `walker_ambiguous: Option<WalkerAmbiguity>` (a `{ candidates: usize }`
  record) and `walker_unreadable: Option<PathBuf>`, recorders
  `note_walker_ambiguous(candidates)` and `note_walker_unreadable(path)` (first
  path wins); carry both through `Unresolved` with `walker_ambiguous()` and
  `walker_unreadable()` accessors; mention the unreadable path in the `Unresolved`
  display (FR-006, P-4).
- [X] T004 In `crates/fragcap-profile/src/providers.rs`, remove the no-op
  `PlatformWalkerProvider` stub and its test; in
  `crates/fragcap-profile/src/lib.rs` drop it from the re-export and add
  `WalkerTarget` (data-model, FR-002). Update any `TargetOrigin` exhaustive match
  in the crate for the new variant.
- [X] T005 In `crates/fragcap-steam/src/scaffold.rs`, make `scan`, `is_non_game`,
  and `is_launcher` (and `ExecutableImage` as needed) `pub(crate)` so
  `walker.rs` shares the exact classifier predicates rather than duplicating the
  token lists (research D7).
- [X] T006 In `crates/fragcap-steam/src/library.rs` (or `lib.rs`), add
  `install_root_in(root, app_id) -> Result<Option<PathBuf>, SteamError>` (portable,
  over a given Steam root) and `install_root_for(app_id)` (locates Steam then
  delegates); re-export both from `lib.rs` (contract, D6).

**Checkpoint**: `cargo build -p fragcap-profile -p fragcap-steam` compiles; the
walker module is empty; the profile crate no longer carries a walker stub.

---

## Phase 3: User Story 2 - Walker resolves a single-client non-engine title (Priority: P1)

**Goal**: The walker classifies a Steam install directory and answers with a
single client when exactly one plausible client remains, declining otherwise.

**Independent test**: Build a temp install directory with one clear client plus
installers and helpers (no engine markers) and assert `client_for` resolves the
client; empty/only-launchers declines; several clients is ambiguous; unreadable is
surfaced.

- [X] T007 [P] [US2] In `crates/fragcap-steam/src/walker.rs` tests, write failing
  tests for `client_for`: (a) one clear client among installers/helpers resolves;
  (b) only launchers / nothing yields `NoMatch`; (c) two plausible clients yield
  `Ambiguous { candidates: 2 }`; (d) an unreadable directory yields
  `Unreadable { path }` (FR-004, FR-006, D3, D8).
- [X] T008 [US2] Implement `client_for(install_dir) -> ClientResolution` in
  `crates/fragcap-steam/src/walker.rs`: `scan` (Err -> `Unreadable`), drop
  `is_non_game` (fall back to all if that empties), drop `is_launcher`; zero
  clients -> `NoMatch`, exactly one -> `Resolved(WalkerTarget{"steam", name, path,
  identity(exe=name)})`, more than one -> `Ambiguous` (data-model, D3, D7, D8).
- [X] T009 [US2] Implement `SteamWalkerProvider` in
  `crates/fragcap-steam/src/walker.rs`: `impl fragcap_profile::TargetProvider` at
  `Precedence::PlatformWalker`; read `request.install_root()`, decline when absent;
  map `Resolved` to `Target::new(HeuristicUnverified,
  Provenance::new("steam-library".to_string(), None),
  TargetOrigin::PlatformWalker(t))`, `NoMatch` to `Ok(None)`, `Ambiguous`/
  `Unreadable` to recording the note then `Ok(None)`; never `Err`; re-export
  `SteamWalkerProvider` from `lib.rs` (contract, FR-001, FR-005).
- [X] T010 [P] [US2] In `crates/fragcap-steam/src/walker.rs` tests, add
  provider-level tests over a temp install dir: single client yields a target
  stamped `heuristic-unverified` + `steam-library` + `PlatformWalker` origin;
  no-match declines; ambiguous declines and records the note (contract, SC-002).

**Checkpoint**: The walker resolves and declines correctly in isolation; US2 is
independently testable.

---

## Phase 4: User Story 1 - Engine title through the cascade via the walker (Priority: P1)

**Goal**: A Steam-installed engine title resolves via the engine rule when the
walker supplies the install directory; a profile outranks both.

**Independent test**: Build a fake Steam library whose title install dir is an
Unreal twin-exe layout; enrich a request with the install dir; assert the engine
rule resolves the shipping exe through a resolver holding the engine-rule and
walker providers, and that a matching profile outranks both.

- [X] T011 [P] [US1] Add an integration test file
  `crates/fragcap/tests/walker_cascade.rs (facade, the crate that depends on all three, per AGENTS.md)` with a fake-Steam-library helper
  (library manifests + install dirs) composed with an Unreal install layout, in
  the spirit of `test_support::TempTree` and the S029 `UnrealTree` (D10).
- [X] T012 [US1] In `walker_cascade.rs`, test the composition through the full
  resolver: `install_root_in` resolves the title's install dir; a request enriched
  via `with_install_root` runs through a `TargetResolver` holding
  `EngineRuleProvider` + `SteamWalkerProvider`. Assert (a) a Steam-installed Unreal
  title resolves the shipping executable via the engine rule (higher precedence) at
  `heuristic-unverified` (SC-001), and (b) a Steam-installed single-client
  non-engine title (engine rule declines) resolves via the walker at
  `heuristic-unverified` with provenance `steam-library` through the same resolver
  (SC-002, resolver-level).
- [X] T013 [US1] In `walker_cascade.rs`, test precedence: an authored/verified
  profile for the same title outranks both the engine rule and the walker,
  independent of registration order (SC-004).

**Checkpoint**: The walker plus engine-rule composition is proven end to end.

---

## Phase 5: User Story 3 - Graceful degradation to runtime observation (Priority: P1)

**Goal**: Not-installed, ambiguous, and unreadable installs decline at the walker
and the cascade resolves via runtime observation.

**Independent test**: Resolver holding the walker + observation providers over a
fake library; a not-installed title, an ambiguous install, and an unreadable
install each decline at the walker and resolve via observation when a matching
process tree is present; the decline reasons are surfaced through `Unresolved`
when observation is absent.

- [X] T014 [P] [US3] In `walker_cascade.rs`, test degradation: for a not-installed
  title, an ambiguous install, and an unreadable install, a resolver holding
  `SteamWalkerProvider` + `ObservationProvider` resolves via observation when a
  matching process tree is present (SC-003).
- [X] T015 [US3] In `walker_cascade.rs`, test the surfaced reasons: with no
  observation provider, an ambiguous install yields `Unresolved` with
  `walker_ambiguous()` set, and an unreadable install yields `Unresolved` with
  `walker_unreadable()` set (FR-006, P-4).

**Checkpoint**: Degradation and honest decline reasons are proven.

---

## Phase 6: Production wiring

- [X] T016 In `crates/fragcap-cli/src/commands/run.rs`, replace the retired
  `PlatformWalkerProvider` stub in the resolver vec with
  `fragcap::steam::SteamWalkerProvider::new()`; update the import. Confirm `run`'s
  existing behavior is unchanged (the walker declines without an `install_root`,
  as the engine rule already does), so profile-backed capture is byte-identical.
- [X] T017 In `crates/fragcap-cli/src/commands/watch.rs`, make the same swap in the
  watch resolver assembly; update the import.

---

## Phase 7: Docs and changelog

- [X] T018 [P] Add a full "Platform walker" glossary entry in
  `docs/glossary/process-and-attribution.md` (near the cascade cluster): what it
  is, that it walks a storefront's installed library and classifies an install
  directory, that it declines rather than guess a client and degrades to runtime
  observation, that it stamps `heuristic-unverified` with provenance
  `steam-library`; cross-link `[Provider]`, `[Resolution cascade]`,
  `[Engine rule]`, `[Target]`; promote the referenced mention (P-6, FR-011). Then
  run `bash scripts/lint-docs.sh fix` to regenerate the index.
- [X] T019 [P] In `docs/fragcap-specification.md`, add a section 15.7 platform-
  walker subsection (composition with the engine rule, decline-and-degrade, honest
  provenance, appinfo/PICS deferred) and reframe section 16 to present Steam as
  one adapter feeding the cascade, cross-referenced to 15.7 (FR-011).
- [X] T020 Add `changelog.d/030-steam-platform-walker.added.md` (feature line) and
  `changelog.d/030-steam-platform-walker.decisions.md` (dated) recording the
  architecture-affecting choices: walker provider in fragcap-steam (deps
  direction); new `TargetOrigin::PlatformWalker`; walker declines rather than
  guess a client by size (P-9, per the library research); honest provenance
  `steam-library` not `steam-appinfo`; appinfo/PICS deferred; production
  non-profile capture path deferred (the run.rs boundary); no new dependency.

---

## Phase 8: Verification

- [X] T021 Run the full gate in the foreground: `cargo xtask ci` (fmt, clippy,
  test, lint, deps, license, wrappers, docs check) plus `cargo xtask msrv` /
  `cargo xtask neutral` where available (report an exit-2 skip honestly). The
  `deps` check is load-bearing: it must confirm `fragcap-profile` gained no
  dependency on `fragcap-steam`. Resolve any failure before the pre-push halt.

---

## Dependencies & completion order

- **Setup (T001)** before everything.
- **Foundational (T002-T006)** blocks all stories: the origin, notes, stub
  removal, shared predicates, and the enumeration helper.
- **US2 (T007-T010)** is the walker's own resolution; depends only on Foundational.
- **US1 (T011-T013)** proves composition; depends on Foundational + the provider
  (T009) and the enumeration helper (T006).
- **US3 (T014-T015)** proves degradation; depends on Foundational + the provider.
- **Production wiring (T016-T017)** after the provider exists.
- **Docs (T018-T020)** after the behavior is settled; **Verification (T021)** last.

## Parallel opportunities

- T007 and T010 (US2 tests) are `[P]` before T008/T009 land.
- T011 (fixture helper) is `[P]` with the US2 tests.
- T014 is `[P]` within US3.
- T018 and T019 (docs) are `[P]` across different files.

## Implementation strategy

The MVP is US2 (the walker resolves a single-client Steam title) plus US3
(degradation), which together satisfy the walker's honest contract. US1 proves the
composition with the engine rule that #77 was built toward. Production capture of a
resolved non-profile target is the explicit follow-up (see the plan's scope
boundary), surfaced at the pre-push halt.

---

## Post-review follow-up (PR #86)

Three changes landed after the first review, all verified by `cargo xtask ci`:

- [X] T022 (Codex P1) `client_for` no longer restores the dropped set when the
  non-game filter empties it, so an installer-only install declines instead of
  targeting the installer (section 15.7.2). File: `walker.rs`; test
  `an_install_of_only_non_game_executables_declines`.
- [X] T023 (Codex P1) `install_root_in`/`install_root_for` return
  `InstallLookup { install_dir, warnings }` so enumeration warnings (malformed
  manifest, unreadable library, duplicate app id) are surfaced, not discarded, and
  a malformed manifest for the requested app is not silently indistinguishable
  from an uninstalled title (FR-008). Files: `library.rs`, `lib.rs`, facade
  re-export, `walker_cascade.rs` callers; test
  `install_root_in_resolves_and_carries_warnings`.
- [X] T024 (Codex P2) the shared `scan` is strict: directory-entry iterator errors
  and per-entry metadata errors surface as `SteamError::Io` rather than being
  skipped, so an incomplete scan becomes `Unreadable` rather than a false single
  answer (mirrors the S029 engine-rule handling; also improves the scaffold).
  File: `scaffold.rs`.
