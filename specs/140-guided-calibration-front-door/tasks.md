# Tasks: Guided Reachability Calibration Front Door

**Input**: Design documents from `specs/140-guided-calibration-front-door/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-018 and the autopilot TDD protocol. Test tasks precede implementation tasks and must demonstrate the expected red state.

## Phase 1: Setup and Specification

**Purpose**: Establish the tracked slice and complete its reviewable intent.

- [x] T001 Create issue #394 as an S140 child of #380, add it to the repository delivery Project, and set its Slice and Stage fields
- [x] T002 Author and validate the S140 specification and requirements checklists in `specs/140-guided-calibration-front-door/spec.md` and `specs/140-guided-calibration-front-door/checklists/`
- [x] T003 Complete research, data model, CLI contract, quickstart, and implementation plan in `specs/140-guided-calibration-front-door/`

---

## Phase 2: Foundational Contract Tests

**Purpose**: Freeze the new public CLI and structured event shapes before implementation.

- [x] T004 Run focused baseline tests for CLI parsing, help, events, and explicit Deep Capture calibration
- [x] T005 [P] Add failing top-level `calibrate` parsing, target-input exclusivity, safe-default, and hidden-internal-option tests in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/tests/cli_args.rs`
- [x] T006 [P] Add failing short-help and full-help contract coverage for the guided command in `crates/fragcap-cli/tests/cli_help.rs`
- [x] T007 [P] Add failing stable `calibration.guidance` human and JSON serialization tests in `crates/fragcap-cli/src/events.rs`
- [x] T008 Add failing integration-test scaffolding for registered-target no-effect and controlled execution paths in `crates/fragcap-cli/tests/cli_calibrate.rs` and `crates/fragcap-cli/tests/common/mod.rs`

**Checkpoint**: New contract tests fail only because the S140 surface does not exist.

---

## Phase 3: User Story 1 - Calibrate a Registered Cold Target (Priority: P1)

**Goal**: Resolve one registered cold target, select exactly one S139 reachability proposal, and execute it through the existing authorized path.

**Independent Test**: A controlled registered target with missing routing evidence emits one selected reachability decision, presents the existing complete plan, and appends only directly observed routing evidence after exact authorization.

### Tests for User Story 1

- [x] T009 [US1] Add failing direct, Steam, publisher, non-positive evidence-reason, authorization-decline, wrong-identifier, and controlled-success cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T010 [US1] Run the new focused suite and record the expected pre-implementation failures

### Implementation for User Story 1

- [x] T011 [US1] Add `CalibrateArgs`, top-level command parsing, and safe bounded options in `crates/fragcap-cli/src/cli.rs`
- [x] T012 [US1] Expose crate-private existing-target, target-fact, and complete Tool Help image-snapshot authorities without changing low-level behavior in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T013 [US1] Implement pure guided decision projection and exact reachability argument adaptation in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T014 [US1] Register and dispatch the guided command with same-process authorization input in `crates/fragcap-cli/src/commands/mod.rs` and `crates/fragcap-cli/src/lib.rs`
- [x] T015 [US1] Delegate the selected reachability case through the existing low-level executor and preserve its terminal exit in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T016 [US1] Make the focused User Story 1 tests pass and retain all explicit low-level calibration tests in `crates/fragcap-cli/tests/cli_calibrate.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`

**Checkpoint**: One cold registered target reaches exactly one plan-bound reachability attempt from one target argument.

---

## Phase 4: User Story 2 - Receive Safe Guidance Without Effects (Priority: P2)

**Goal**: Report ready, warm, and refused proposals truthfully before effects.

**Independent Test**: Current positive, warm, ambiguous, invalid, unavailable, and zero-step-not-ready cases produce deterministic decisions and zero delegated session calls.

### Tests for User Story 2

- [x] T017 [US2] Add failing current-positive, warm guidance, explicit warm retry, limitation aggregation, unavailable snapshot, and invalid zero-step tests in `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 2

- [x] T018 [US2] Implement current-positive readiness verification and durable ordinary Deep Capture handoff in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T019 [US2] Implement effect-free warm guidance plus explicit delegation to the existing close-and-retry authority in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T020 [US2] Implement aggregate limitation and invalid-proposal refusals before low-level argument construction in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T021 [US2] Preserve no-match, row-index, durable-id, ambiguity, and absent-store diagnostics through the existing resolver in `crates/fragcap-cli/src/commands/calibrate.rs`

**Checkpoint**: Every no-effect state has one typed action and no session effect.

---

## Phase 5: User Story 3 - Automate and Continue the Guided Path (Priority: P3)

**Goal**: Emit stable human and JSON decisions and only parseable durable next commands.

**Independent Test**: Structured ready, warm, refused, selected, and completed records parse with the same field set, and every emitted next command selects the same durable target.

### Tests for User Story 3

- [x] T022 [P] [US3] Add failing event-envelope field and nullability cases in `crates/fragcap-cli/src/events.rs`
- [x] T023 [P] [US3] Add failing JSON-line, quiet/silent, durable next-command parsing, and existing event coexistence tests in `crates/fragcap-cli/tests/cli_calibrate.rs` and `crates/fragcap-cli/tests/cli_reference.rs`

### Implementation for User Story 3

- [x] T024 [US3] Implement the stable `calibration.guidance` event and human projection in `crates/fragcap-cli/src/events.rs`
- [x] T025 [US3] Emit selected, no-effect, and successful terminal guidance without suppressing delegated Deep Capture events in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T026 [US3] Generate only `--id` reassessment, warm-retry, and ordinary Deep Capture commands and validate them through the command parser in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: Human and machine consumers receive the same stable decision facts.

---

## Phase 6: Documentation, Audit, and Completion

**Purpose**: Reconcile shipped truth, verify scope, and prepare the pull request.

- [x] T027 [P] Extend the calibration glossary entry and architecture record in `docs/glossary/capture-and-networking.md`, `docs/fragcap-specification.md`, and `docs/fragcap-spec-outline.md`
- [x] T028 [P] Record S140 sequencing and bounded #380 follow-up in `docs/plans/README.md` and `AGENTS.md`
- [x] T029 [P] Add changed and decisions fragments in `changelog.d/S140-guided-calibration-front-door.changed.md` and `changelog.d/S140-guided-calibration-front-door.decisions.md`
- [x] T030 Run the S140 quickstart, complete a requirement-to-test audit, and mark all checklists and tasks complete in `specs/140-guided-calibration-front-door/`
- [x] T031 Run `/speckit-converge`, append and implement any remaining traceable work, or record a clean convergence result
- [x] T032 Run `cargo xtask ci`, encoding, punctuation, mojibake, diff, dependency, and worktree checks
- [x] T033 Commit, push the authorized feature branch, open a PR closing #394, and set the Project Stage to PR review
- [ ] T034 Resolve every first-round review finding, trigger at most one `@Codex review` second round, resolve every second-round finding, and wait for all CI checks to pass

---

## Dependencies & Execution Order

- Phase 1 is complete and gates all later phases.
- Phase 2 freezes the public contract and must demonstrate red before implementation.
- User Story 1 supplies target context and delegated execution needed by User Stories 2 and 3.
- User Story 2 completes all no-effect safety paths before User Story 3 finalizes presentation.
- Phase 6 follows all user stories; convergence is blocking before the repository gate.

## Parallel Opportunities

- T005, T006, and T007 touch independent contract-test surfaces.
- T022 and T023 separate event-unit and command-integration tests.
- T027, T028, and T029 touch distinct documentation and changelog files after behavior is final.

## Implementation Strategy

1. Freeze parsing, help, event, and controlled-run expectations.
2. Build the minimal cold registered-target path through S139 and the existing executor.
3. Add ready, warm, and refusal projections with explicit zero-effect checks.
4. Stabilize human and structured guidance plus durable next commands.
5. Reconcile documentation, converge, run the full repository gate, and complete the authorized review cycle.

## Scope Guardrails

- No automatic target registration, TLS or protocol selection, multi-attempt loop, or workflow persistence.
- No process control or process handle.
- No new dependency, lockfile package, storage migration, or public Rust API export.
- No change to existing low-level calibration semantics.
- Issue #394 closes on merge; parent #380 remains open.

---

## Phase 7: Convergence

- [x] T035 Expose explicit no-process-control truth in warm human and JSON guidance per FR-010 (partial)
- [x] T036 Emit stable terminal guided outcomes for delegated refusals and nonzero exits while preserving detailed Deep Capture events per FR-014 (partial)
- [x] T037 Preserve the PowerShell-quoted effective local-store path in every generated next command and verify same-store durable identity resolution per SC-003 (review)
- [x] T038 Require a limitation-free, matching ready launch state before post-session completion and cover limited, mismatched, and warm zero-step reassessments per FR-013a (review)
