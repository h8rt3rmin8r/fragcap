# Tasks: Deep Capture Embedded Workflow Help

**Input**: Design documents from `specs/148-deep-capture-help/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/embedded-workflow-help.md, quickstart.md

**Tests**: Required by FR-014, FR-015, and the autopilot TDD protocol. Test tasks precede their corresponding implementation tasks.

## Phase 1: Setup

**Purpose**: Establish the S148 branch, artifacts, and current command-surface inventory.

- [x] T001 Confirm the audited Clap paths and existing human refusal touchpoints against `crates/fragcap-cli/src/cli.rs`, `crates/fragcap-cli/src/commands/deep_capture.rs`, `crates/fragcap-cli/src/commands/calibrate.rs`, `crates/fragcap-cli/src/commands/doctor.rs`, and `crates/fragcap-cli/src/commands/bundle.rs`
- [x] T002 Verify all requirements, UX, and security checklist items are complete in `specs/148-deep-capture-help/checklists/`

---

## Phase 2: Foundational Registry And Contract Tests

**Purpose**: Create failing semantic tests for the shared journey, parser-backed examples, and closed refusal inventory before product changes.

- [x] T003 Add failing audited short-help, long-help, section-order, narrow-width, no-color, and example-parsing tests in `crates/fragcap-cli/tests/cli_help.rs`
- [x] T004 [P] Add failing ordinary Deep Capture first-run refusal guidance assertions in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T005 [P] Add failing guided calibration known, unknown, pause, and continuation guidance assertions in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T006 [P] Add failing Doctor first-run bridge assertions in `crates/fragcap-cli/tests/cli_doctor.rs`
- [x] T007 [P] Add failing bundle sensitivity, destructive cleanup, export, and failed-cleanup retention assertions in `crates/fragcap-cli/tests/cli_bundle.rs`

**Checkpoint**: Focused tests fail for missing S148 behavior and preserve existing baseline behavior.

---

## Phase 3: User Story 1 - Discover The First Successful Path (Priority: P1)

**Goal**: Make one ordered offline journey discoverable from an installed game name through ordinary Deep Capture.

**Independent Test**: Every audited help path renders, the stages are reachable in order, and every displayed command example parses through the production command model.

- [x] T008 [US1] Implement the versioned stage, audited-path, command-example, and refusal-category registry in `crates/fragcap-cli/src/workflow_help.rs`
- [x] T009 [US1] Expose the registry to the CLI command model and integration tests in `crates/fragcap-cli/src/lib.rs`
- [x] T010 [US1] Add the concise root summary and registry-backed long first-run journey in `crates/fragcap-cli/src/cli.rs`
- [x] T011 [US1] Add ordered target discovery, registration, detail, calibration, and launch-family examples to the relevant subcommand help in `crates/fragcap-cli/src/cli.rs`
- [x] T012 [US1] Make the workflow and example parsing tests pass in `crates/fragcap-cli/tests/cli_help.rs`

**Checkpoint**: User Story 1 is independently complete and validated without executing a session.

---

## Phase 4: User Story 2 - Choose Safe Defaults Without Hiding Advanced Controls (Priority: P2)

**Goal**: Separate ordinary, advanced, sensitive, storage, networking, and troubleshooting controls while preserving truthful security boundaries.

**Independent Test**: Deep Capture and calibration long help preserve required section order and all meaning at 40, 60, and 80 columns with color disabled.

- [x] T013 [US2] Assign stable required, common, advanced, sensitive, custom-storage, networking, and troubleshooting help headings in `crates/fragcap-cli/src/cli.rs`
- [x] T014 [US2] Add observed-compatibility, non-universal-decryption, no-pinning-bypass, mTLS, key-log, bypass, and custom-storage boundaries to long help in `crates/fragcap-cli/src/cli.rs`
- [x] T015 [US2] Make section-order, width, color, and safety-language tests pass in `crates/fragcap-cli/tests/cli_help.rs`

**Checkpoint**: User Story 2 is independently complete and every advanced control remains visible under its consequence-based heading.

---

## Phase 5: User Story 3 - Recover From Every First-Run Stop (Priority: P3)

**Goal**: End every closed pre-session first-run category with exact, parse-backed next-step guidance.

**Independent Test**: Controlled command tests exercise all nine refusal categories and calibration terminal states without applying real session effects.

- [x] T016 [US3] Apply registry-backed target, launch, process, compatibility, recovery, bundle, and authorization next-step guidance in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T017 [US3] Align guided calibration output with established facts, remaining unknowns, and registry-validated next commands in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T018 [US3] Bridge environment readiness to exact target registration or calibration guidance in `crates/fragcap-cli/src/commands/doctor.rs`
- [x] T019 [US3] Explain sensitive export, destructive cleanup, and failed-cleanup recovery-record retention in `crates/fragcap-cli/src/commands/bundle.rs`
- [x] T020 [US3] Make all focused refusal guidance tests pass in `crates/fragcap-cli/tests/cli_deep_capture.rs`, `crates/fragcap-cli/tests/cli_calibrate.rs`, `crates/fragcap-cli/tests/cli_doctor.rs`, and `crates/fragcap-cli/tests/cli_bundle.rs`

**Checkpoint**: User Story 3 is independently complete, human guidance is actionable, and machine contracts are unchanged.

---

## Phase 6: Documentation, Convergence, And Repository Validation

**Purpose**: Record the shipped boundary, validate prose mechanically, and prove complete repository health.

- [x] T021 Update the CLI, testing, Doctor, roadmap, and slice-history architecture in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T022 Add S148 change and decision fragments under `changelog.d/`
- [x] T023 Mark every completed task and run the spec-to-code convergence audit against `specs/148-deep-capture-help/`
- [x] T024 Run encoding, mojibake, Markdown, formatting, focused tests, and `cargo xtask ci` validation across the repository

---

## Dependencies And Execution Order

- Phase 1 precedes every implementation phase.
- Phase 2 must complete before product changes so every behavior follows TDD.
- User Story 1 establishes the registry required by User Stories 2 and 3.
- User Story 2 and User Story 3 may proceed after the registry exists, but changes to `cli.rs` and shared tests remain sequential.
- Phase 6 follows all user stories.

## Parallel Opportunities

- T004 through T007 can be authored in parallel because they touch distinct integration-test files.
- After T008 and T009, refusal updates in T016 through T019 can be reasoned about independently, but implementation remains sequential where shared registry contracts are involved.
- Documentation files in T021 and changelog fragments in T022 can be prepared independently after product behavior stabilizes.

## Implementation Strategy

1. Establish red tests for the complete registry, help pages, parser-backed examples, and refusal categories.
2. Deliver the P1 workflow registry and ordered long-help path as the minimum useful increment.
3. Add consequence-based option organization and truthful sensitive-control boundaries.
4. Route each closed first-run refusal category to exact next commands without changing structured output or effects.
5. Converge against every requirement, document the bounded shipped behavior, and run the complete repository gate.
