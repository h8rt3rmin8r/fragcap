---

description: "Task list for S027 Target Resolution Cascade -- Resolver Core"
---

# Tasks: Target Resolution Cascade -- Resolver Core

**Input**: Design documents from `specs/027-target-resolver-core/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. This repository works under test-driven discipline
(constitution, autopilot protocol); the permutation and honesty tests are
required, not optional.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 / US2 / US3, mapping to the spec's user stories

## Path Conventions

Rust workspace. New code in `crates/fragcap-profile/src/`; CLI integration in
`crates/fragcap-cli/src/`; docs in `docs/`.

---

## Phase 1: Setup

- [ ] T001 Declare the three new modules (`target`, `resolver`, `providers`) in
  `crates/fragcap-profile/src/lib.rs` as empty `pub mod` files created alongside,
  so later tasks add to compiling stubs; add no re-exports yet.

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The metadata surfacing and the resolver skeleton that every user
story builds on. No story is testable until these exist.

- [ ] T002 Add `FidelityTier` (variants `Observed`, `HeuristicUnverified`,
  `Verified`, `Authored`, declared ascending so `Ord` makes Authored greatest),
  with `ACCEPTED`, `parse`, and `as_str`, in
  `crates/fragcap-profile/src/schema.rs` (data-model D-2).
- [ ] T003 Add `Provenance { source, seeded_at }` and `Kind { Profile, Package,
  Hint, Export }` with accessors in `crates/fragcap-profile/src/schema.rs`.
- [ ] T004 [P] Unit tests in `schema.rs`: `FidelityTier::parse` maps the four
  schema names and rejects others; `as_str` round-trips; ordering is
  `Authored > Verified > HeuristicUnverified > Observed` (FR-003, SC-006).
- [ ] T005 Extend the `Profile` struct with `kind`, `fidelity`,
  `provenance: Option<Provenance>`, and `notes: Option<String>`; update
  `Profile::new`; add `kind()`, `fidelity()`, `provenance()`, `notes()`
  accessors; extend `Profile::ACCEPTED` with `kind`/`fidelity`/`provenance`/
  `notes` in `crates/fragcap-profile/src/schema.rs` (FR-009).
- [ ] T006 In `crates/fragcap-profile/src/parse.rs`, read `kind`, `fidelity`,
  `provenance`, and `notes` off the parsed `Value` (the structural validator has
  already confirmed them) and pass them into `Profile::new`; explicitly accept
  `kind: package` alongside `profile` and keep refusing `hint`/`export`
  (FR-009, D-8).
- [ ] T007 Add the `Target`, `TargetOrigin`, and `ObservedTarget` types with
  accessors (`fidelity`, `provenance`, `origin`, `profile`) in
  `crates/fragcap-profile/src/target.rs` (data-model D-5).
- [ ] T008 Add the resolver engine in `crates/fragcap-profile/src/resolver.rs`:
  `Precedence` (five positions, highest-first), `ResolutionRequest` (with
  `for_reference` and `for_observation` constructors), the `TargetProvider`
  trait, `ProviderError`, `Unresolved`, and `TargetResolver::new` (sorts
  providers by `precedence()`) and `resolve` (query highest-first; first
  `Ok(Some)` wins; `Err` aborts; all `Ok(None)` -> `Unresolved`) (FR-001, FR-004,
  FR-011).
- [ ] T009 Re-export the new public types from
  `crates/fragcap-profile/src/lib.rs` per contracts/resolver-api.md.

**Checkpoint**: `cargo build -p fragcap-profile` compiles; `Profile` exposes its
metadata; the resolver engine exists with no live providers yet.

---

## Phase 3: User Story 1 -- Resolve by fidelity-ranked precedence (P1)

**Goal**: The cascade returns the highest-precedence stamped answer,
deterministically. **Independent test**: two providers can both answer; the
higher-precedence one wins, identically for every registration order.

- [ ] T010 [US1] Add the three no-answer stub providers `HintProvider`,
  `EngineRuleProvider`, `PlatformWalkerProvider` (each reports its `Precedence`
  and returns `Ok(None)`) in `crates/fragcap-profile/src/providers.rs` (FR-008).
- [ ] T011 [US1] Add `ProfileProvider` (`Precedence::Profile`) wrapping
  `resolve()`: found -> `Ok(Some(Target))` stamped with the profile's declared
  fidelity and its provenance (or a `ProfileSource`-named synthesized one);
  `NotFound` -> `Ok(None)` (attach to `Unresolved`); `Load`/`InvalidReference` ->
  `Err(ProviderError::Profile(..))` in
  `crates/fragcap-profile/src/providers.rs` (FR-006, D-6, D-7).
- [ ] T012 [P] [US1] Tests in `resolver.rs` (or a `tests/resolver.rs`): highest
  precedence wins; a lower provider answers when higher ones are silent; an
  `Err` aborts and lower providers are not consulted; all-`Ok(None)` yields
  `Unresolved` (FR-001, FR-011).
- [ ] T013 [P] [US1] The permutation test: for every permutation of the provider
  vec, `resolve` returns the same answer for the same inputs (FR-004, SC-001).
- [ ] T014 [P] [US1] A resolved target carries exactly one fidelity tier and a
  provenance; a `ProfileProvider` answer's fidelity equals the profile's declared
  tier, not a fixed value (FR-002, FR-006, SC-002).

**Checkpoint**: US1 fully testable with the profile provider plus stubs.

---

## Phase 4: User Story 2 -- Fall back to runtime observation (P2)

**Goal**: A game with no higher answer resolves to an `observed` target once a
matching process exists. **Independent test**: a tree with a matching live node
and no higher provider yields an `observed` target.

- [ ] T015 [US2] Add `pub fn first_live_match(preds: &MatchPredicates, tree:
  &ProcessTree) -> Option<NodeId>` in
  `crates/fragcap-profile/src/matching.rs`, refactoring the private
  `predicates_hold` into a callable form so the P-9 command-line-unavailable rule
  stays in one place (D-5).
- [ ] T016 [P] [US2] Tests for `first_live_match`: matches on exe/path/regex;
  an `Unavailable` command line never satisfies `cmdline_contains`; returns the
  first live node in creation order (P-9, parity with `bind_stages`).
- [ ] T017 [US2] Add `ObservationProvider` (`Precedence::RuntimeObservation`):
  with `identity` + `tree`, returns `Ok(Some(Target { origin: Observed, fidelity:
  Observed, provenance source "runtime-observation" }))` for the first match via
  `first_live_match`, using only `image_name()`/`image()`; else `Ok(None)` in
  `crates/fragcap-profile/src/providers.rs` (FR-007).
- [ ] T018 [P] [US2] Tests: observation yields an `observed` target with the
  node's pid/name/path and provenance `runtime-observation`; no match ->
  `Ok(None)` -> resolver `Unresolved`; observation never higher than `observed`;
  it opens no handle (asserted by `cargo xtask lint` in Phase 6) (FR-007, SC-003,
  SC-006, SC-007).

**Checkpoint**: US1 and US2 both work through one resolver.

---

## Phase 5: User Story 3 -- Read the trust level of any answer (P3)

**Goal**: Fidelity, provenance, and kind are readable on a loaded profile and on
every resolved target; the two fidelity axes stay distinct.

- [ ] T019 [P] [US3] Tests in `parse.rs`: a valid profile exposes `kind()`,
  `fidelity()`, `provenance()` (None when omitted), and `notes()` matching the
  document; `kind: package` loads and reports `Package`; `hint`/`export` still
  refused; an existing profile still parses with identical `game`/`capture`/
  `stages` (FR-009, SC-004).
- [ ] T020 [P] [US3] A separation test asserting the targeting `FidelityTier`
  and the attribution `Fidelity` (`fragcap_core::attribution::Fidelity`) are
  distinct types on distinct values, neither derived from the other (FR-010).

---

## Phase 6: Integration (CLI, behavior-preserving)

- [ ] T021 Rewire `crates/fragcap-cli/src/commands/run.rs` to build a
  `TargetResolver` and call `resolve` for the profile path, extract the `Profile`
  from the profile-backed `Target`, and pass it to `assemble::effective_config`
  exactly as today; map `Unresolved{profile NotFound}` and
  `ProviderError::Profile(..)` onto the existing `CliError` variants so exit codes
  and messages are unchanged (D-6, SC-008).
- [ ] T022 Verify capture output is byte-identical: run the corpus pipeline tests
  and any CLI resolution tests; confirm goldens unchanged (SC-008).

---

## Phase 7: Polish and cross-cutting

- [ ] T023 [P] Add master spec section 15.7 "Target Resolution Cascade" in
  `docs/fragcap-specification.md`, cross-referencing 7.1 (acquisition), 15.6
  (artifacts and fidelity), and 15.3 (the narrower profile-reference lookup)
  (FR-013).
- [ ] T024 [P] Add glossary entries `provider`, `target resolver`, `resolution
  cascade`, and `target` in `docs/glossary/process-and-attribution.md`, with
  See-also links, keeping the generated index reproducible (FR-013, P-6).
- [ ] T025 Add a changelog feature fragment
  `changelog.d/027-target-resolver-core.added.md` and a decisions fragment
  `changelog.d/027-target-resolver-core.decisions.md` recording the
  two-fidelity-axis separation, the precedence-vs-fidelity model, and the
  fragcap-profile placement.
- [ ] T026 Run the full gate in the foreground and fix to green: `cargo xtask ci`
  (fmt, clippy, test, lint, deps, license), then `cargo xtask msrv` and
  `cargo xtask neutral` (report exit-2 skips honestly).

---

## Dependencies and order

- Setup (T001) -> Foundational (T002-T009) -> stories.
- US1 (T010-T014) depends only on Foundational. US2 (T015-T018) depends on
  Foundational (and reuses `matching`). US3 (T019-T020) depends on Foundational
  (the Profile surfacing). US1/US2/US3 are otherwise independent.
- Integration (T021-T022) depends on US1 (the resolver + profile provider).
- Polish (T023-T026) last; T026 is the verification gate.

## Parallel opportunities

- T004, T012, T013, T014, T016, T018, T019, T020, T023, T024 are `[P]` (distinct
  files or test-only, no incomplete dependencies).

## MVP scope

User Story 1 (the precedence engine + profile provider + stubs) is the minimum
viable increment: it delivers the cascade and its determinism. US2 adds the
observation arbiter; US3 makes trust readable.
