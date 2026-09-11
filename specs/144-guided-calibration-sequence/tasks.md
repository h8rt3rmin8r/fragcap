# Tasks: Bounded Guided Calibration Sequence

**Input**: Design documents from `specs/144-guided-calibration-sequence/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-026 and the autopilot TDD protocol. Test tasks precede implementation tasks and must demonstrate the expected red state.

## Phase 1: Setup and Specification

**Purpose**: Establish the tracked slice and complete its reviewable intent.

- [x] T001 Create issue #402 as an S144 child of #380, add it to the repository delivery Project, and set its Slice and Stage fields
- [x] T002 Author and validate the S144 specification plus requirements and security checklists in `specs/144-guided-calibration-sequence/`
- [x] T003 Complete research, data model, command contract, quickstart, and implementation plan in `specs/144-guided-calibration-sequence/`

---

## Phase 2: Foundational Contract Tests

**Purpose**: Freeze sequence identity, finite bounds, event progress, and bundle ownership before implementation.

- [x] T004 Run focused baseline tests for guided calibration, low-level authorization, controlled sessions, events, and public CLI references
- [x] T005 Add failing unit tests for exact attempted-case equality, one-per-case insertion, fourteen-attempt bound, and deterministic candidate accumulation in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T006 Add failing unit tests for first-path compatibility, deterministic sibling bundle derivation, unsafe path refusal, and collision preservation in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T007 Add failing event serialization and human rendering tests for nullable attempt, maximum, phase, and protocol fields in `crates/fragcap-cli/src/events.rs`
- [x] T008 Add failing controlled integration scaffolding for multiple authorization responses and multiple distinct attempt bundles in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: New tests fail only because sequence state, additive guidance fields, and multi-attempt execution do not exist.

---

## Phase 3: User Story 1 - Advance Through Useful Work (Priority: P1)

**Goal**: Run reachability and each still-useful requested or observed protocol case in deterministic order within one invocation.

**Independent Test**: A controlled target with missing routing and two requested protocols records three exact current facts under three distinct authorization plans and finishes with no remaining requested coverage.

### Tests for User Story 1

- [x] T009 [US1] Add failing missing-routing-to-protocol and multiple-requested-protocol controlled cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T010 [US1] Add failing observed-candidate growth, requested-observed overlap, current-positive suppression, and deterministic ordering cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T011 [US1] Run the new focused suite and record the expected pre-implementation failures

### Implementation for User Story 1

- [x] T012 [US1] Add transient guided sequence and exact attempted-case values with the closed supported bound in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T013 [US1] Refactor the single-attempt tail into fresh-authority selection and post-session reassessment helpers in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T014 [US1] Implement deterministic in-process progression through distinct useful S139 steps in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T015 [US1] Accumulate only existing eligible final-client observed candidates while preserving requested and observed sets separately in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T016 [US1] Make the focused User Story 1 tests pass while retaining every S140-S143 front-door and low-level session test

**Checkpoint**: Current positive evidence can advance one invocation through all distinct current useful work.

---

## Phase 4: User Story 2 - Authorize Every Attempt Separately (Priority: P2)

**Goal**: Preserve one fresh complete S134 plan and one exact input decision for every session in the sequence.

**Independent Test**: Three selected cases emit three different plans, consume three separate response lines, and stop before the first unconfirmed case without applying its effects.

### Tests for User Story 2

- [x] T017 [US2] Add failing separate-plan, separate-response, interactive-decline, structured-invalid, closed-input, and interruption cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T018 [US2] Add failing target, process, fact, proposal, and bundle drift cases between attempts in `crates/fragcap-cli/src/commands/calibrate.rs` and `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 2

- [x] T019 [US2] Reopen the store, re-resolve the durable target, snapshot processes, read facts, and rebuild S139 before every selected attempt in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T020 [US2] Extend the crate-private run outcome with an execution disposition that distinguishes declined, completed, interrupted, and terminal-failure sessions, then delegate each selected case independently without caching plan or response authority in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T021 [US2] Update the production authorization-input documentation to describe ordered per-plan responses in `crates/fragcap-cli/src/lib.rs`
- [x] T022 [US2] Make the focused User Story 2 tests pass and retain existing low-level S134 decline, drift, and terminal error semantics

**Checkpoint**: One invocation is convenient, but every effect retains an independent exact authorization boundary.

---

## Phase 5: User Story 3 - Stop Safely and Report Continuation (Priority: P3)

**Goal**: Stop on every unsafe or incomplete boundary with finite execution, truthful coverage, distinct artifacts, and an exact next command when supported.

**Independent Test**: Partial evidence, failure, warm change, limitation, repetition, and bundle collision each start no later session and retain exact completed and remaining work.

### Tests for User Story 3

- [x] T023 [US3] Add failing partial-evidence, terminal-failure, warm-transition, limitation, target-loss, and repeated-case stop tests in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T024 [US3] Add failing one-per-case and fourteen-attempt exhaustion tests in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T025 [US3] Add failing explicit sibling bundle layout, later collision, default unique bundle, and first-path compatibility tests in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T026 [US3] Add failing ordered attempt progress, terminal coverage, and parseable continuation tests in `crates/fragcap-cli/src/events.rs`, `crates/fragcap-cli/tests/cli_calibrate.rs`, and `crates/fragcap-cli/tests/cli_reference.rs`

### Implementation for User Story 3

- [x] T027 [US3] Add additive attempt, maximum, phase, and protocol fields to calibration guidance in `crates/fragcap-cli/src/events.rs` and `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T028 [US3] Derive path-safe deterministic sibling bundle arguments for later explicit-path attempts in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T029 [US3] Implement current-positive progression gates, repetition and bound guards, and terminal stop classification in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T030 [US3] Revalidate ordinary prerequisites at sequence completion and emit exact completed, remaining, and continuation guidance in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T031 [US3] Make the focused User Story 3 tests pass without changing storage, artifact schemas, or low-level eligibility

**Checkpoint**: The sequence is finite, separately authorized, artifact-safe, and honest about every stop.

---

## Phase 6: Documentation, Audit, and Completion

**Purpose**: Reconcile shipped truth, verify scope, and prepare the pull request.

- [x] T032 Update the architecture record and outline in `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md`
- [x] T033 Record the S144 deviation, sequence boundary, and bounded #380 follow-up in `docs/plans/README.md` and `AGENTS.md`
- [x] T034 Update the public CLI reference in `site/content/docs/reference/cli.mdx`
- [x] T035 Add changed and decisions fragments in `changelog.d/S144-guided-calibration-sequence.changed.md` and `changelog.d/S144-guided-calibration-sequence.decisions.md`
- [x] T036 Run the S144 quickstart, complete a requirement-to-test audit, and mark all checklists and tasks complete in `specs/144-guided-calibration-sequence/`
- [x] T037 Run `/speckit-converge`, append and implement any remaining traceable work, or record a clean convergence result
- [x] T038 Run `cargo xtask ci`, encoding, punctuation, mojibake, diff, dependency, and worktree checks
- [ ] T039 Commit, push the authorized feature branch, open a PR closing #402, and set the Project Stage to PR review
- [ ] T040 Resolve every first-round review finding, trigger at most one `@Codex review` second round, resolve every second-round finding, and wait for all CI checks to pass

---

## Dependencies & Execution Order

- Phase 1 is complete and gates all later phases.
- Phase 2 freezes additive contracts and must demonstrate red before implementation.
- User Story 1 supplies the sequence needed by User Story 2.
- User Story 2 supplies independent plan boundaries needed by User Story 3.
- Phase 6 follows all user stories; convergence is blocking before the repository gate.

## Implementation Strategy

1. Freeze attempt identity, bounds, progress, and bundle expectations.
2. Extract fresh proposal and completion helpers from the existing single-attempt tail.
3. Implement deterministic progression using current positive facts only.
4. Preserve one complete authorization and one unique bundle per attempt.
5. Stabilize every stop and terminal coverage outcome.
6. Reconcile documentation, converge, run the full repository gate, and complete the authorized review cycle.

## Scope Guardrails

- No persistent workflow or resume state, non-Steam topology authoring, or ambiguous topology choice.
- No hidden or system-wide trust, blanket authorization, automatic process control, pinning bypass, target process access, or target key extraction.
- No new dependency, lockfile package, storage migration, or artifact schema.
- No change to the low-level S134 plan, session, fact, cleanup, or ordinary eligibility authorities.
- Issue #402 closes on merge; parent #380 remains open.
