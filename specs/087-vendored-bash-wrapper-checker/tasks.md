# Tasks: Vendored Bash Wrapper Checker

**Input**: Design documents from `specs/087-vendored-bash-wrapper-checker/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required. This is a CI gate correctness change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Establish S087 metadata and specification alignment.

- [X] T001 Create S087 specification artifacts in `specs/087-vendored-bash-wrapper-checker/`
- [X] T002 Update `.specify/feature.json` to point at `specs/087-vendored-bash-wrapper-checker`
- [X] T003 Add changelog fragments for issue #199 in `changelog.d/`

---

## Phase 2: Foundational

**Purpose**: Pin the product and gate contract before implementation.

- [X] T004 Update wrapper-gate behavior in `docs/fragcap-specification.md`
- [X] T005 Measure current Bash scripts against the vendored checker

---

## Phase 3: User Story 1 - One Bash Standard Authority (Priority: P1)

**Goal**: All gated Bash scripts are checked by the vendored Bash checker and the Rust duplicate is gone.

**Independent Test**: Run `cargo xtask wrappers` and confirm each Bash script reports a vendored Bash checker result.

### Tests for User Story 1

- [X] T006 [US1] Add focused xtask assertions that all Bash scripts use the vendored Bash checker in `xtask/src/wrappers.rs`

### Implementation for User Story 1

- [X] T007 [US1] Replace the hand-authored `check_bash` path with vendored Bash checker invocations in `xtask/src/wrappers.rs`
- [X] T008 [US1] Remove the old Rust Bash structural checker and its rule-unit tests from `xtask/src/wrappers.rs`
- [X] T009 [US1] Preserve `bash -n`, `--help`, and `fragcap.sh --dry-run` checks in `xtask/src/wrappers.rs`

---

## Phase 4: User Story 2 - Static Analysis Cannot Be Skipped (Priority: P2)

**Goal**: A missing Bash-runnable ShellCheck is reported as an unable-to-run wrapper gate.

**Independent Test**: Run `cargo xtask wrappers` in an environment without Bash-runnable ShellCheck and confirm exit 2.

### Tests for User Story 2

- [X] T010 [US2] Add focused xtask assertions for the Bash checker and script registry in `xtask/src/wrappers.rs`

### Implementation for User Story 2

- [X] T011 [US2] Add a Bash-visible ShellCheck preflight in `xtask/src/wrappers.rs`
- [X] T012 [US2] Map missing Bash-visible ShellCheck to the existing unable-to-run error path in `xtask/src/wrappers.rs`

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Validate the slice and finish the integration packet.

- [X] T013 Run focused S087 tests from `specs/087-vendored-bash-wrapper-checker/quickstart.md`
- [X] T014 Run `cargo fmt --all -- --check`
- [X] T015 Run `cargo xtask ci`
- [X] T016 Review `git diff --check` and changed-file punctuation for repository conventions
- [X] T017 Mark tasks complete in `specs/087-vendored-bash-wrapper-checker/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup.
- **User Story 1 (Phase 3)**: Depends on foundational contract updates.
- **User Story 2 (Phase 4)**: Depends on wrapper checker delegation.
- **Polish (Phase 5)**: Depends on all user stories.

### Parallel Opportunities

- T003 and T004 can be prepared in parallel after setup.
- T006 through T012 stay in one file and should be implemented together.
- Verification commands must run sequentially in the foreground.

## Implementation Strategy

### MVP First

1. Complete setup and contract updates.
2. Replace Bash structural checks with vendored checker invocations.
3. Add the Bash-visible ShellCheck preflight.
4. Validate the wrapper gate independently.

### Incremental Delivery

1. Deliver checker authority swap.
2. Deliver no-skipped-static-analysis behavior.
3. Run focused tests and the full repository gate.
