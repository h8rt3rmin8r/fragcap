# Tasks: Steam Non-Game Filter

**Input**: Design documents from `specs/086-steam-non-game-filter/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required. This is a discovery correctness fix where TDD is the safest route.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Establish S086 metadata and specification alignment.

- [X] T001 Create S086 specification artifacts in `specs/086-steam-non-game-filter/`
- [X] T002 Update `.specify/feature.json` to point at `specs/086-steam-non-game-filter`
- [X] T003 Add changelog fragment for issue #212 in `changelog.d/212-steam-non-game-filter.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin the product and source contract before implementation.

- [X] T004 Update Steam discovery behavior in `docs/fragcap-specification.md`
- [X] T005 Add failing fixture tests for excluded and preserved Steam app types in `crates/fragcap/tests/steam_source.rs`

---

## Phase 3: User Story 1 - Exclude Non-Capturable Steam App Types (Priority: P1)

**Goal**: `Music`, `Tool`, `Application`, `Config`, and `Video` Steam apps do not become discovery candidates.

**Independent Test**: Run the Steam source fixture test and assert those app ids are absent, counted not-a-game, and conserved.

### Tests for User Story 1

- [X] T006 [US1] Add non-game app type exclusion assertions in `crates/fragcap/tests/steam_source.rs`

### Implementation for User Story 1

- [X] T007 [US1] Add a case-insensitive non-game app type predicate in `crates/fragcap/src/discovery.rs`
- [X] T008 [US1] Use the predicate before app-id parsing and candidate construction in `crates/fragcap/src/discovery.rs`

---

## Phase 4: User Story 2 - Preserve Game-Like Steam Entries (Priority: P2)

**Goal**: `Demo`, `Game`, and unknown app types remain eligible candidates.

**Independent Test**: Run the Steam source fixture test and assert preserved app ids still appear with existing candidate fields.

### Tests for User Story 2

- [X] T009 [US2] Add preserved app type assertions in `crates/fragcap/tests/steam_source.rs`

### Implementation for User Story 2

- [X] T010 [US2] Keep `Demo`, `Game`, and absent app types outside the exclusion predicate in `crates/fragcap/src/discovery.rs`

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Validate the slice and finish the integration packet.

- [X] T011 Run focused S086 tests from `specs/086-steam-non-game-filter/quickstart.md`
- [X] T012 Run `cargo fmt --all -- --check`
- [X] T013 Run `cargo xtask ci`
- [X] T014 Review `git diff --check` and changed-file punctuation for repository conventions
- [X] T015 Mark tasks complete in `specs/086-steam-non-game-filter/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup.
- **User Story 1 (Phase 3)**: Depends on foundational contract updates.
- **User Story 2 (Phase 4)**: Depends on the shared exclusion predicate.
- **Polish (Phase 5)**: Depends on all user stories.

### Parallel Opportunities

- T004 and T005 can be prepared in parallel after setup.
- T006 and T009 can be written together because they affect the same fixture module and validate opposite sides of the same predicate.
- Verification commands must run sequentially in the foreground.

## Implementation Strategy

### MVP First

1. Complete setup and contract updates.
2. Add failing app-type fixture tests.
3. Implement the non-game predicate and wire it into Steam discovery.
4. Validate User Story 1 independently.

### Incremental Delivery

1. Deliver non-game exclusion.
2. Validate game-like preservation.
3. Run focused tests and the full repository gate.
