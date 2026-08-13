---
description: "Task list for S029 Engine-Rule Provider (Unreal First)"
---

# Tasks: Engine-Rule Provider (Unreal First)

**Input**: Design documents from `specs/029-engine-rule-provider/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. This repository works under test-driven discipline
(constitution, autopilot protocol); the determinism, precedence, and
fidelity-honesty tests are required, not optional.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 / US2, mapping to the spec's user stories

## Path Conventions

Rust workspace. New code in `crates/fragcap-profile/src/`; docs in `docs/`.
The `EngineRuleProvider` stub already exists in `providers.rs` at
`Precedence::EngineRule`; this slice fills it in and adds the surrounding
surface.

---

## Phase 1: Setup

- [X] T001 Declare a new `pub mod engine_rule;` in
  `crates/fragcap-profile/src/lib.rs` pointing at a new empty
  `crates/fragcap-profile/src/engine_rule.rs`, so later tasks add to a compiling
  module; add no re-exports yet.

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The shared surface both stories build on: the request input, the new
target origin, the ambiguity note, and the module skeleton with its
engine-agnostic scaffolding. No story is testable until these exist.

- [X] T002 In `crates/fragcap-profile/src/engine_rule.rs`, add the `Engine` enum
  (`Unreal`, `Unity`, `RenPy`; derive `Clone, Copy, Debug, PartialEq, Eq`) with
  `as_str()` returning `"unreal"`/`"unity"`/`"renpy"` (data-model, contract).
- [X] T003 In `crates/fragcap-profile/src/target.rs`, add `EngineRuleTarget`
  (`engine: Engine`, `image_name: String`, `image_path: String`,
  `identity: MatchPredicates`; derive `Clone, Debug, PartialEq, Eq`) with
  `engine()`, `image_name()`, `image_path()`, `identity()` accessors, and add the
  `EngineRule(EngineRuleTarget)` variant to `TargetOrigin`; leave `profile()` and
  `into_profile()` returning `None` for it (data-model, FR-003).
- [X] T004 In `crates/fragcap-profile/src/resolver.rs`, add
  `install_root: Option<&'a Path>` to `ResolutionRequest`, a
  `for_install(install_root, search, bundled)` constructor (setting reference,
  identity, tree to `None`), and an `install_root()` accessor; set
  `install_root: None` in `for_reference` and `for_observation` (data-model D-2,
  contract).
- [X] T005 In `crates/fragcap-profile/src/resolver.rs`, extend `ResolutionNotes`
  with `engine_rule_ambiguous: Option<EngineRuleAmbiguity>` (a small
  `{ engine: Engine, candidates: usize }` record) and a
  `note_engine_rule_ambiguous(engine, candidates)` recorder; carry it through
  `Unresolved` with an `engine_rule_ambiguous()` accessor (FR-009, P-4, P-9).
- [X] T006 In `crates/fragcap-profile/src/engine_rule.rs`, add the internal
  `EngineResolution` enum (`Resolved(EngineRuleTarget)`, `NoMatch`,
  `Ambiguous { engine, candidates }`) and the `resolve_engine(install_root:
  &Path) -> EngineResolution` entry point that dispatches to the per-engine
  recognizers in a fixed, total order (Unreal, Unity, RenPy), returning the first
  engine whose layout is present; recognizers are stubs returning `NoMatch` until
  their story tasks fill them (data-model, FR-006).
- [X] T007 In `crates/fragcap-profile/src/providers.rs`, fill in
  `EngineRuleProvider::provide`: read `request.install_root()`, decline `Ok(None)`
  when absent; call `resolve_engine`; map `Resolved` to
  `Ok(Some(Target::new(FidelityTier::HeuristicUnverified,
  Provenance::new("engine-rule".to_string(), None),
  TargetOrigin::EngineRule(t))))`, `NoMatch` to `Ok(None)`, and `Ambiguous` to
  recording the note then `Ok(None)`; never return `Err` (contract, FR-003,
  FR-004).
- [X] T008 Add a test-only temp-tree helper in the `engine_rule` test module
  (create under `std::env::temp_dir()` with a unique name, `write` / `write_exe`
  helpers creating parents, recursive remove on `Drop`), in the spirit of
  `fragcap-steam`'s `TempTree` (research D9). No committed `fixtures/`.

**Checkpoint**: `cargo build -p fragcap-profile` compiles; the provider answers
`Ok(None)` for every input because all recognizers are stubs.

---

## Phase 3: User Story 1 - Resolve an Unreal title (Priority: P1)

**Goal**: The Unreal recognizer resolves a `*-Win64-Shipping.exe` under
`Binaries/Win64` from a stub install root, at `heuristic-unverified` fidelity,
flowing through the S027 resolver.

**Independent test**: Build an Unreal twin-exe temp tree, resolve it through a
`TargetResolver` holding the real `EngineRuleProvider`, and assert the shipping
executable is named at `HeuristicUnverified` with provenance `engine-rule`.

- [X] T009 [P] [US1] In `crates/fragcap-profile/src/engine_rule.rs` tests, write
  failing tests over a sample of at least three distinct Unreal layouts (differing
  game names, per SC-001): (a) each Unreal tree resolves to its shipping exe with
  the right fidelity/provenance and an `identity` carrying `exe` and
  `path_contains = "Binaries\\Win64"`; (b) a tree with no `Binaries/Win64`
  yields `NoMatch`; (c) a `Binaries/Win64` directory with no shipping exe yields
  `NoMatch` (no fabricated target); (d) two shipping exes yield
  `Ambiguous { Unreal, 2 }` (FR-002, FR-006, SC-001, edge cases).
- [X] T010 [US1] Implement the Unreal recognizer in
  `crates/fragcap-profile/src/engine_rule.rs`: locate a directory whose trailing
  components are `Binaries/Win64` (case-insensitive, either separator) beneath the
  install root, collect files ending `-Win64-Shipping.exe` (case-insensitive)
  that exist as files; zero -> `NoMatch`, one -> `Resolved` (build
  `MatchPredicates` via `Default` + `set_exe(ImagePattern::new(name))` +
  `set_path_contains("Binaries\\Win64")`), more than one -> `Ambiguous`
  (research D3, D6, D7, D8).
- [X] T011 [P] [US1] In `crates/fragcap-profile/src/engine_rule.rs` tests, add a
  determinism test: the same Unreal fixture resolves to the identical path across
  repeated calls and after creating sibling files in a different order (SC-003,
  FR-006).
- [X] T012 [US1] In `crates/fragcap-profile/src/providers.rs` tests, add
  provider-level tests: `EngineRuleProvider` on a `for_install` request over an
  Unreal tree yields the stamped target; on a no-match tree declines; on an
  ambiguous tree declines and records the note (contract).
- [X] T013 [US1] In `crates/fragcap-profile/src/resolver.rs` tests (or providers
  tests), add a cascade test: a `TargetResolver` holding a `ProfileProvider` and
  the `EngineRuleProvider` returns the engine-rule answer for an Unreal install
  with no matching profile, and the profile answer when a matching profile exists
  (profile outranks engine rule), independent of registration order (SC-004,
  FR-001).

**Checkpoint**: Unreal is fully resolved through the resolver; US1 is
independently testable and shippable on its own.

---

## Phase 4: User Story 2 - Resolve Unity and Ren'Py titles (Priority: P2)

**Goal**: The Unity and Ren'Py recognizers resolve their player executables from
install layout, under the same fidelity, provenance, and decline rules.

**Independent test**: Build Unity and Ren'Py temp trees and assert each resolves
its player executable at `HeuristicUnverified` with provenance `engine-rule`, and
a tree matching neither declines.

- [X] T014 [P] [US2] In `crates/fragcap-profile/src/engine_rule.rs` tests, write
  failing tests for the Unity layout (`*_Data` directory + `UnityPlayer.dll` +
  player exe resolves to the player exe) and the Ren'Py layout (`renpy` directory
  + `.rpa` archive + launcher exe resolves to the launcher) (FR-008).
- [X] T015 [US2] Implement the Unity recognizer in
  `crates/fragcap-profile/src/engine_rule.rs`: require a `*_Data` directory and a
  `UnityPlayer.dll` in the root; resolve the player executable matching the
  `*_Data` stem; build its `identity` with `exe` set; apply the same
  zero/one/many decision (FR-008, FR-006).
- [X] T016 [US2] Implement the Ren'Py recognizer in
  `crates/fragcap-profile/src/engine_rule.rs`: require a `renpy` directory and at
  least one `.rpa` archive under the root; resolve the launcher executable in the
  root; build its `identity` with `exe` set; apply the same zero/one/many
  decision (FR-008, FR-006).
- [X] T017 [P] [US2] In `crates/fragcap-profile/src/engine_rule.rs` tests, add a
  rule-order test: a tree carrying signatures for more than one engine resolves by
  the fixed engine order, identically across runs (FR-006).

**Checkpoint**: All three engines resolve; the provider offers real breadth.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T018 [P] Re-export `Engine` and `EngineRuleTarget` from
  `crates/fragcap-profile/src/lib.rs` (and confirm they flow through the facade
  re-export in `crates/fragcap/src/lib.rs` as the other cascade types do), so a
  caller can match `TargetOrigin::EngineRule` and read the resolved client
  (contract).
- [X] T019 [P] Add a full "Engine rule" glossary entry in
  `docs/glossary/process-and-attribution.md` (alphabetical placement), defining
  what an engine rule is, that it keys on documented install layout only, that it
  stamps `heuristic-unverified`, and cross-linking `[Provider]`, `[Provenance]`,
  `[Fidelity tier]`, `[Resolution cascade]`; promote the existing named-example
  mentions to reference it (P-6, FR-010).
- [X] T020 [P] Extend the target-resolution cascade section (15.7) of
  `docs/fragcap-specification.md` with an engine-rule subsection: the three rules
  and their layout signatures, the fidelity and provenance, the install-root
  input, the ambiguity-declines behavior, and composition with the S030 walker
  (FR-010).
- [X] T021 Add a `changelog.d/029-engine-rule-provider.added.md` feature line and
  a `changelog.d/029-engine-rule-provider.decisions.md` fragment recording the
  architecture-affecting choices (new `TargetOrigin::EngineRule` variant and
  `EngineRuleTarget`; the `install_root` request input; ambiguity declines to
  runtime observation with a surfaced note; targeting fidelity stays separate
  from attribution fidelity; no new dependency, provider stays in
  fragcap-profile).
- [X] T022 Run the full gate in the foreground: `cargo xtask ci` (fmt, clippy,
  test, lint, deps, license) plus `bash scripts/lint-docs.sh`, and
  `cargo xtask msrv` / `cargo xtask neutral` where available (report an exit-2
  skip honestly). Resolve any failure before the pre-push halt.

---

## Dependencies & completion order

- **Setup (T001)** before everything.
- **Foundational (T002-T008)** blocks both stories; the module skeleton,
  request input, origin variant, note, and provider wiring must exist before any
  recognizer is testable.
- **US1 (T009-T013)** is the MVP and the mandatory acceptance gate. Depends only
  on Foundational.
- **US2 (T014-T017)** depends only on Foundational; independent of US1 (different
  recognizers in the same file, so sequence the edits but they do not depend on
  US1 behavior). Can be deferred to a follow-up if complexity forces it.
- **Polish (T018-T022)** after the stories it documents; T022 is last.

## Parallel opportunities

- T009 and T011 (US1 tests) are `[P]` with each other before T010 lands, then
  T010 makes them pass.
- T014 and T017 (US2 tests) are `[P]`.
- T018, T019, T020 (re-exports and docs) are `[P]` across different files.

## Implementation strategy

MVP is US1 (Unreal) resolved through the resolver: it satisfies the mandatory
acceptance target (SC-001) and the honesty and determinism criteria on its own.
US2 (Unity, Ren'Py) is the same-cost breadth increment. Ship both this slice per
the plan; fall back to US1-only, deferring US2, only if Unity/Ren'Py detection
proves subtler than documented.
