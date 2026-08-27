# Tasks: Marker Cap Warning Subject

**Input**: Design documents from `specs/084-marker-cap-warning/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required. This is a warning correctness fix where TDD is the safest route.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Establish S084 metadata and specification alignment.

- [X] T001 Create S084 specification artifacts in `specs/084-marker-cap-warning/`
- [X] T002 Update `.specify/feature.json` to point at `specs/084-marker-cap-warning`
- [X] T003 Add changelog fragment for issue #206 in `changelog.d/206-marker-cap-warning.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin the warning contract before implementation.

- [X] T004 Update binary-marker scan warning contract in `docs/fragcap-specification.md`
- [X] T005 Add failing scan outcome warning tests in `crates/fragcap-profile/src/signature.rs`

---

## Phase 3: User Story 1 - Locate The Incomplete Scan (Priority: P1)

**Goal**: Every marker-cap warning names the scanned root that produced it.

**Independent Test**: Scan capped candidate sets for one or more roots and observe distinct warning subjects.

### Tests for User Story 1

- [X] T006 [US1] Add discovery forwarding test for root-named marker-cap warnings in `crates/fragcap-targets/tests/user_pointed.rs`
- [X] T007 [US1] Add CLI discover test for root-named marker-cap warnings in `crates/fragcap-cli/tests/cli_targets.rs`

### Implementation for User Story 1

- [X] T008 [US1] Store the scanned root on `ScanOutcome` in `crates/fragcap-profile/src/signature.rs`
- [X] T009 [US1] Update marker-cap warning text in `crates/fragcap-profile/src/signature.rs`
- [X] T010 [US1] Adjust affected discovery and CLI assertions in `crates/fragcap-targets/tests/user_pointed.rs` and `crates/fragcap-cli/tests/cli_targets.rs`

---

## Phase 4: User Story 2 - Explain The Operator Consequence (Priority: P2)

**Goal**: The warning says what was skipped and that technology detection may be incomplete for the named root.

**Independent Test**: Assert the warning includes binary-marker skipped candidates, exact skipped count, named root, and incomplete technology consequence.

### Tests for User Story 2

- [X] T011 [US2] Add consequence wording assertions in `crates/fragcap-profile/src/signature.rs`
- [X] T012 [US2] Add or adjust standalone technologies warning assertions in `crates/fragcap-cli/src/commands/technologies.rs`

### Implementation for User Story 2

- [X] T013 [US2] Keep warning prefixes at callers while consuming the improved shared warning body in `crates/fragcap-cli/src/commands/technologies.rs` and `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Validate the slice and finish the integration packet.

- [X] T014 Run focused S084 tests from `specs/084-marker-cap-warning/quickstart.md`
- [X] T015 Run `cargo fmt --all -- --check`
- [X] T016 Run `cargo xtask ci`
- [X] T017 Review `git diff --check` and changed-file punctuation for repository conventions
- [X] T018 Mark tasks complete in `specs/084-marker-cap-warning/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup.
- **User Story 1 (Phase 3)**: Depends on the shared warning contract.
- **User Story 2 (Phase 4)**: Depends on the shared warning contract and can be validated independently from caller framing.
- **Polish (Phase 5)**: Depends on both user stories.

### Parallel Opportunities

- T006 and T007 can be written independently after T005.
- T011 and T012 can be written independently after T005.
- Verification commands must run sequentially in the foreground.

## Implementation Strategy

### MVP First

1. Complete setup and contract updates.
2. Add failing scan outcome tests.
3. Store the scanned root and update shared warning text.
4. Validate User Story 1 independently.

### Incremental Delivery

1. Deliver root-named marker-cap warnings.
2. Add consequence wording coverage.
3. Confirm existing callers consume the shared warning body.
4. Run focused tests and full repository CI.
