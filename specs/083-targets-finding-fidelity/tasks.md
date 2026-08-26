# Tasks: Targets Finding Fidelity

**Input**: Design documents from `specs/083-targets-finding-fidelity/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required. This is a correctness fix where TDD is the safest route.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Establish S083 metadata and specification alignment.

- [X] T001 Create S083 specification artifacts in `specs/083-targets-finding-fidelity/`
- [X] T002 Update `.specify/feature.json` to point at `specs/083-targets-finding-fidelity`
- [X] T003 Add changelog fragment for issue #211 in `changelog.d/211-targets-finding-fidelity.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin the public contract before implementation.

- [X] T004 Update target listing contract in `docs/fragcap-specification.md`
- [X] T005 Add focused readiness tests for fidelity-marked product summaries in `crates/fragcap-targets/src/readiness.rs`

---

## Phase 3: User Story 1 - Distinguish Guesses In The Listing (Priority: P1)

**Goal**: The human target listing visibly distinguishes verified-or-stronger technology products from uncertain ones.

**Independent Test**: Store targets with verified and heuristic-unverified findings, run the listing, and observe different rendered cells.

### Tests for User Story 1

- [X] T006 [US1] Add CLI integration test for unverified ENGINE and SENSITIVITIES markers in `crates/fragcap-cli/tests/cli_targets.rs`
- [X] T007 [US1] Add readiness unit test for duplicate products choosing strongest fidelity in `crates/fragcap-targets/src/readiness.rs`

### Implementation for User Story 1

- [X] T008 [US1] Implement fidelity-aware product summaries in `crates/fragcap-targets/src/readiness.rs`
- [X] T009 [US1] Recheck table width assumptions and comments in `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 4: User Story 2 - Preserve Machine Fidelity Agreement (Priority: P2)

**Goal**: Export and import preserve the per-finding fidelity that the listing summarizes.

**Independent Test**: Export, import, and re-export evidence with verified and heuristic-unverified finding fidelity.

### Tests for User Story 2

- [X] T010 [US2] Add export/import fidelity preservation test in `crates/fragcap-targets/src/targets_export.rs`
- [X] T011 [US2] Add CLI export guard for finding fidelity in `crates/fragcap-cli/tests/cli_targets.rs`

### Implementation for User Story 2

- [X] T012 [US2] Adjust export/import implementation only if tests reveal fidelity is not preserved in `crates/fragcap-targets/src/targets_export.rs`

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Validate the slice and finish the integration packet.

- [X] T013 Run focused S083 tests from `specs/083-targets-finding-fidelity/quickstart.md`
- [X] T014 Run `cargo fmt --all -- --check`
- [X] T015 Run `cargo xtask ci`
- [X] T016 Review `git diff --check` and changed-file punctuation for repository conventions
- [X] T017 Mark tasks complete in `specs/083-targets-finding-fidelity/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup.
- **User Story 1 (Phase 3)**: Depends on foundational tests and specification contract.
- **User Story 2 (Phase 4)**: Can run after foundational work and is independent of the US1 implementation except for shared understanding of fidelity.
- **Polish (Phase 5)**: Depends on both user stories.

### Parallel Opportunities

- T006 and T007 can be written in either order but both target the same behavior at different boundaries.
- T010 and T011 can be written independently from US1 after T004.
- Verification commands must run sequentially in the foreground.

## Implementation Strategy

### MVP First

1. Complete setup and contract updates.
2. Add failing readiness and CLI listing tests.
3. Implement fidelity-aware summaries.
4. Validate User Story 1 independently.

### Incremental Delivery

1. Deliver human listing fidelity markers.
2. Add export/import preservation guards.
3. Run focused tests and full repository CI.
