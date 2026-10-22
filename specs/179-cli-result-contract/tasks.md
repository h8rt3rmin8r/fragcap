# Tasks: Consistent CLI Result Output

**Input**: Design documents from `specs/179-cli-result-contract/`.

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, and `contracts/cli-results.md`.

**Tests**: Required by issue #488, the feature specification, and the repository autopilot protocol; write the contract tests before each corresponding implementation task.

## Phase 1: Setup

**Purpose**: Bind the active S179 feature and existing CLI surface without adding dependencies.

- [x] T001 Verify branch `codex/s179-cli-result-contract`, `.specify/feature.json`, repository CI gate, and no extension hooks in `specs/179-cli-result-contract/plan.md`.
- [x] T002 Inventory each top-level command's current stdout, stderr, JSON, quiet, silent, prompt, and machine-artifact behavior in `specs/179-cli-result-contract/contracts/cli-results.md`.

---

## Phase 2: Foundational Output Policy

**Purpose**: Establish result framing and precedence before command-specific projections.

- [x] T003 Write failing global flag and channel matrix assertions in `crates/fragcap-cli/tests/cli_output_matrix.rs` for default, quiet, silent, JSON, and JSON plus silent.
- [x] T004 Implement CLI-local output selection and complete JSON-line result writing in `crates/fragcap-cli/src/lib.rs` and `crates/fragcap-cli/src/commands/result.rs`, preserving required prompts and machine artifacts.
- [x] T005 Update `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/emit.rs` help and tests so the global flag descriptions and event precedence match the chosen contract.

---

## Phase 3: User Story 1 - Structured Command Results (Priority: P1)

**Goal**: Scripts receive typed target, catalog, and schema facts instead of human prose under JSON mode.

**Independent Test**: The target/catalog/schema rows of `cli_output_matrix.rs` parse every selected machine result and compare actual counts and diagnostics.

- [x] T006 [US1] Add failing target list, show, no-match, and export-preservation assertions in `crates/fragcap-cli/tests/cli_output_matrix.rs` and `crates/fragcap-cli/tests/cli_targets.rs`.
- [x] T007 [US1] Project target list, show, and action results from domain values without parsing human output in `crates/fragcap-cli/src/commands/targets.rs`.
- [x] T008 [US1] Add failing catalog import, seed, skip-reason, and export-preservation assertions in `crates/fragcap-cli/tests/cli_output_matrix.rs` and `crates/fragcap-cli/tests/cli_catalog.rs`.
- [x] T009 [US1] Project typed import and seed results, including actual tier counters, in `crates/fragcap-cli/src/commands/catalog.rs`.
- [x] T010 [US1] Add failing valid, malformed, and four-defect schema diagnostics assertions in `crates/fragcap-cli/tests/cli_output_matrix.rs` and `crates/fragcap-cli/tests/cli_schema.rs`.
- [x] T011 [US1] Emit separate schema diagnostic and summary records while preserving schema print bytes and exit classes in `crates/fragcap-cli/src/commands/schema.rs`.

---

## Phase 4: User Story 2 - Predictable Verbosity (Priority: P2)

**Goal**: Silent human output omits optional successful prose while machine streams, consent, action obligations, and errors remain.

**Independent Test**: Quiet/silent and JSON/silent rows assert stdout and stderr for controlled success, failure, and required-action paths.

- [x] T012 [US2] Add failing human silent and JSON plus silent assertions for targets, catalog, schema, Doctor, and Capture in `crates/fragcap-cli/tests/cli_output_matrix.rs`.
- [x] T013 [US2] Gate ordinary human result writes without hiding required prompts or output artifacts in `crates/fragcap-cli/src/commands/targets.rs`, `crates/fragcap-cli/src/commands/catalog.rs`, and `crates/fragcap-cli/src/commands/schema.rs`.
- [x] T014 [US2] Reconcile Doctor read-only report, existing Bundle/Fresh Start/Steam result modes, and Capture lifecycle precedence in `crates/fragcap-cli/src/commands/doctor.rs`, `crates/fragcap-cli/src/lib.rs`, and `crates/fragcap-cli/src/emit.rs`.

---

## Phase 5: User Story 3 - Complete Command Matrix (Priority: P3)

**Goal**: Every top-level command has an honest accepted flag policy or an early, named usage refusal.

**Independent Test**: The closed command/flag matrix names every top-level family and verifies output grammar and channel ownership without a live target.

- [x] T015 [US3] Add remaining command-family, technologies, export, extcap, build-identity, and unsupported-combination assertions in `crates/fragcap-cli/tests/cli_output_matrix.rs`.
- [x] T016 [US3] Add typed technology findings and coverage output, and reconcile any uncovered command result family, in `crates/fragcap-cli/src/commands/technologies.rs` and `crates/fragcap-cli/src/lib.rs`.
- [x] T017 [US3] Verify machine document and protocol compatibility against existing fixtures in `crates/fragcap-cli/tests/cli_output_matrix.rs` and `crates/fragcap-cli/tests/cli_capture.rs`.

---

## Phase 6: Documentation and Verification

**Purpose**: Reconcile public contracts and run the complete gate before publication.

- [x] T018 Update the section 17 output contract in `docs/fragcap-specification.md`, add S179 chronologically in `docs/plans/README.md`, and add `changelog.d/S179-cli-result-contract.fixed.md`.
- [x] T019 Run Spec-Kit analyze against `specs/179-cli-result-contract/{spec,plan,tasks}.md`, resolve all findings, and recheck `specs/179-cli-result-contract/checklists/`.
- [x] T020 Run targeted matrix tests and `cargo xtask ci`, then check formatting, UTF-8 without BOM, LF, mojibake, source-line rules, and `git diff --check` for S179 files.
- [ ] T021 Commit the S179 branch, push it under the owner's explicit authorization, open the official PR with `Closes #488`, and attach the PR to this task.
- [ ] T022 Watch exact-head CI and both permitted third-party review rounds, fix and answer every finding, resolve every review thread, and stop for the owner's final review and merge ritual.

## Dependencies and Execution Order

- Setup T001-T002 precedes foundation T003-T005. The contract test T003 must fail before its implementation.
- User Story 1 follows foundation, with tests T006, T008, and T010 preceding their matching implementation tasks T007, T009, and T011.
- User Story 2 follows the shared result projection; T012 precedes T013-T014. User Story 3 closes the remaining surface after T015's tests.
- Documentation and analysis follow the implemented contract. T020 must pass before T021; T022 applies to the published exact head.

## Implementation Strategy

Deliver a small result framing helper first, then command-local typed records, then flag precedence and closed matrix coverage. Preserve every existing protocol or document shape and keep controlled fixtures independent from Npcap, games, elevation, and owner evidence.
