# Tasks: Guided Steam Client Setup

**Input**: Design documents from `specs/143-guided-steam-topology/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, and `contracts/`

**Tests**: Every behavior task follows test-first development and records the initial expected failure.

## Phase 1: Specification and Tracking

**Purpose**: Establish the reviewed scope and repository tracking before implementation.

- [x] T001 Create and link GitHub issue #400 to parent #380 and the fragcap Delivery project as slice S143
- [x] T002 Author `specs/143-guided-steam-topology/spec.md` and validate `specs/143-guided-steam-topology/checklists/requirements.md`
- [x] T003 Resolve security-sensitive authority boundaries in `specs/143-guided-steam-topology/checklists/security.md`
- [x] T004 Produce `specs/143-guided-steam-topology/plan.md`, `research.md`, `data-model.md`, `contracts/calibrate-steam-client-command.md`, and `quickstart.md`

---

## Phase 2: Foundational Store Contract

**Purpose**: Provide one race-safe target mutation that every command path can rely on.

- [x] T005 [P] Add failing conditional authored-client store tests for applied, changed, missing, and present-declaration cases in `crates/fragcap-targets/src/store.rs`
- [x] T006 Implement the typed conditional authored-client transaction in `crates/fragcap-targets/src/store.rs`
- [x] T007 Export the conditional outcome through `crates/fragcap-targets/src/lib.rs` and `crates/fragcap/src/lib.rs`
- [x] T008 Run the focused `fragcap-targets` tests and confirm the foundational contract passes

**Checkpoint**: The target store can atomically fill only an absent client declaration without overwriting concurrent or prior authority.

---

## Phase 3: User Story 1 - Review and Author a Steam Client (Priority: P1)

**Goal**: Let an operator review a bounded Steam executable proposal and explicitly author it as the socket-holding client.

**Independent Test**: A controlled discovered Steam target with no launch declaration emits the complete setup plan, accepts its exact identifier, persists the authored client, and then reaches the separately authorized calibration path.

### Tests for User Story 1

- [x] T009 [P] [US1] Add failing plan identity, executable validation, and positive confirmation unit tests in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T010 [P] [US1] Add failing human and JSON event contract tests for setup plan and outcome events in `crates/fragcap-cli/src/events.rs`
- [x] T011 [US1] Add a failing controlled end-to-end positive setup test in `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 1

- [x] T012 [US1] Implement the deterministic Steam client setup plan, candidate join, executable refusal, and confirmation boundary in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T013 [US1] Add `calibration.steam_client_plan` and `calibration.steam_client` presentation contracts in `crates/fragcap-cli/src/events.rs`
- [x] T014 [US1] Insert setup before proposal construction, re-resolve the updated target by stable identifier, and preserve separate session authorization in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T015 [US1] Run the focused plan, event, and positive CLI tests and confirm the story passes

**Checkpoint**: A positive operator assertion can safely complete one missing Steam client declaration and continue calibration.

---

## Phase 4: User Story 2 - Preserve Existing and Declined Authority (Priority: P2)

**Goal**: Ensure the convenience flow never edits a present declaration and a negative, unsure, absent, or malformed response causes no mutation.

**Independent Test**: Stored resolved, unresolved, malformed, and empty launch declarations bypass setup, while declined and invalid setup input leaves an absent declaration unchanged.

### Tests for User Story 2

- [x] T016 [US2] Add failing bypass, decline, and malformed-structured-input CLI tests in `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 2

- [x] T017 [US2] Complete ineligible-target and no-effect outcome handling in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T018 [US2] Run the focused authority-preservation CLI tests and confirm the story passes

**Checkpoint**: Every non-positive or previously authored state is preserved exactly.

---

## Phase 5: User Story 3 - Refuse Drift and Races (Priority: P3)

**Goal**: Refuse setup when discovery metadata or the stored target changes after the plan is reviewed.

**Independent Test**: Controlled discovery drift, target-row mutation, target deletion, and ambiguous candidate reproduction each terminate explicitly without writing the proposed client.

### Tests for User Story 3

- [x] T019 [US3] Add failing discovery-drift, target-change, missing-target, and ambiguous-candidate integration tests in `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 3

- [x] T020 [US3] Add controlled discovery drift support and exact fresh-plan comparison in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T021 [US3] Map every conditional-store and re-resolution failure to explicit events and terminal outcomes in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T022 [US3] Run the focused drift and race tests and confirm the story passes

**Checkpoint**: Reviewed authority cannot survive changed discovery or storage state.

---

## Phase 6: Documentation, Analysis, and Verification

**Purpose**: Reconcile the implemented behavior with the repository architecture and release evidence.

- [x] T023 [P] Update `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, `site/content/docs/reference/cli.mdx`, and `AGENTS.md`
- [x] T024 [P] Add `changelog.d/S143-guided-steam-topology.changed.md` and `changelog.d/S143-guided-steam-topology.decisions.md`
- [x] T025 Validate traceability and implementation convergence against every S143 requirement, contract, and checklist
- [x] T026 Run formatting, focused suites, `cargo xtask ci`, dependency checks, and UTF-8, LF, mojibake, and forbidden-punctuation checks
- [x] T027 Inspect the complete diff for scope, security, and documentation accuracy, then mark every S143 task complete

---

## Dependencies and Execution Order

- Phase 1 is complete and gates all implementation.
- Phase 2 blocks every user story because the CLI must not use an unconditional target update.
- User Story 1 establishes the plan and successful handoff used by User Stories 2 and 3.
- User Story 2 completes non-mutating authority preservation before race handling is considered complete.
- User Story 3 closes the revalidation and concurrent-writer boundaries.
- Phase 6 begins after all three stories pass independently.

## Implementation Strategy

1. Land the conditional store contract under failing tests.
2. Deliver the positive Steam-only setup path and validate its independent authorization sequence.
3. Prove every present or non-positive state remains unchanged.
4. Prove fresh discovery and complete-row races terminate without mutation.
5. Reconcile documentation, analyze coverage, run the complete gate, and review the final diff before publication.
