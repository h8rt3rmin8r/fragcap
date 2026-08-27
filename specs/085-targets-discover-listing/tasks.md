# Tasks: Targets Discover Listing

**Input**: Design documents from `specs/085-targets-discover-listing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required. This is a CLI rendering correctness fix where TDD is the safest route.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Establish S085 metadata and specification alignment.

- [X] T001 Create S085 specification artifacts in `specs/085-targets-discover-listing/`
- [X] T002 Update `.specify/feature.json` to point at `specs/085-targets-discover-listing`
- [X] T003 Add changelog fragment for issue #207 in `changelog.d/207-targets-discover-listing.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin the human rendering contract before implementation.

- [X] T004 Update discovery rendering contract in `docs/fragcap-specification.md`
- [X] T005 Add failing unit tests for discovery table and account rendering in `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 3: User Story 1 - Read Discovery Results As A Listing (Priority: P1)

**Goal**: Discovery candidates print as a headed aligned table with no tabs.

**Independent Test**: Run fixture-backed `targets discover` and assert labelled stores, headers, aligned rows, no tabs, and the expected Steam identity.

### Tests for User Story 1

- [X] T006 [US1] Add CLI fixture assertions for labelled stores, headers, and no tabs in `crates/fragcap-cli/tests/cli_targets.rs`

### Implementation for User Story 1

- [X] T007 [US1] Replace discovery preamble with labelled store-path output in `crates/fragcap-cli/src/commands/targets.rs`
- [X] T008 [US1] Render discovery candidates as a width-computed table in `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 4: User Story 2 - Keep Candidate Evidence Attached (Priority: P2)

**Goal**: Candidate evidence stays directly under the owning row with fidelity intact.

**Independent Test**: Print a discovery result with evidence and assert the indented evidence line includes category, product, and fidelity.

### Tests for User Story 2

- [X] T009 [US2] Add evidence-under-row rendering test in `crates/fragcap-cli/src/commands/targets.rs`

### Implementation for User Story 2

- [X] T010 [US2] Preserve evidence rendering under each discovery row in `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 5: User Story 3 - Read The Discovery Account (Priority: P3)

**Goal**: Discovery account totals and outcome buckets print as labelled lines with zero outcomes grouped.

**Independent Test**: Print an account with non-zero and zero outcomes and assert the distinct labels and grouped zero line.

### Tests for User Story 3

- [X] T011 [US3] Replace account-line assertions with labelled-block assertions in `crates/fragcap-cli/src/commands/targets.rs`

### Implementation for User Story 3

- [X] T012 [US3] Render discovery account as a labelled block with grouped zero outcomes in `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validate the slice and finish the integration packet.

- [X] T013 Run focused S085 tests from `specs/085-targets-discover-listing/quickstart.md`
- [X] T014 Run `cargo fmt --all -- --check`
- [X] T015 Run `cargo xtask ci`
- [X] T016 Review `git diff --check` and changed-file punctuation for repository conventions
- [X] T017 Mark tasks complete in `specs/085-targets-discover-listing/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup.
- **User Story 1 (Phase 3)**: Depends on the rendering contract.
- **User Story 2 (Phase 4)**: Depends on candidate table rendering.
- **User Story 3 (Phase 5)**: Depends on the shared discovery printer.
- **Polish (Phase 6)**: Depends on all user stories.

### Parallel Opportunities

- T006 and T009 can be written independently after T005.
- T011 can be written independently after T005.
- Verification commands must run sequentially in the foreground.

## Implementation Strategy

### MVP First

1. Complete setup and contract updates.
2. Add failing discovery table tests.
3. Replace tab rows with the headed aligned table.
4. Validate User Story 1 independently.

### Incremental Delivery

1. Deliver the candidate listing.
2. Preserve and test evidence placement.
3. Replace the account line with the labelled block.
4. Run focused tests and full repository CI.
