# Tasks: Guided Protocol Calibration Attempt

**Input**: Design documents from `specs/141-guided-protocol-calibration/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-020 and the autopilot TDD protocol. Test tasks precede implementation tasks and must demonstrate the expected red state.

## Phase 1: Setup and Specification

**Purpose**: Establish the tracked slice and complete its reviewable intent.

- [x] T001 Create issue #396 as an S141 child of #380, add it to the repository delivery Project, and set its Slice and Stage fields
- [x] T002 Author and validate the S141 specification and requirements checklists in `specs/141-guided-protocol-calibration/spec.md` and `specs/141-guided-protocol-calibration/checklists/`
- [x] T003 Complete research, data model, CLI contract, quickstart, and implementation plan in `specs/141-guided-protocol-calibration/`

---

## Phase 2: Foundational Contract Tests

**Purpose**: Freeze candidate, event, and executor-result contracts before implementation.

- [x] T004 Run focused baseline tests for facade policy, CLI parsing, help, events, S140 guidance, and low-level protocol calibration
- [x] T005 [P] Add failing facade tests for deterministic concrete final-client observed-candidate derivation and exclusion in `crates/fragcap/src/deep_capture/policy.rs`
- [x] T006 [P] Add failing repeatable `--protocol` parsing, invalid-routing, normalization, and help tests in `crates/fragcap-cli/src/cli.rs`, `crates/fragcap-cli/tests/cli_args.rs`, and `crates/fragcap-cli/tests/cli_help.rs`
- [x] T007 [P] Add failing additive guided event array tests in `crates/fragcap-cli/src/events.rs`
- [x] T008 Add failing controlled integration scaffolding for one protocol attempt and observed continuation in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: New contract tests fail only because the S141 surface and typed terminal handoff do not exist.

---

## Phase 3: User Story 1 - Run One Exact Protocol Attempt (Priority: P1)

**Goal**: Preserve reachability precedence and execute exactly one first useful S139 protocol step through the existing authorized path.

**Independent Test**: A controlled target with current routing evidence and one unresolved protocol emits one selected protocol decision, presents the complete existing plan, and appends only directly observed facts after exact authorization.

### Tests for User Story 1

- [x] T009 [US1] Add failing reachability-precedence, one-step, multi-candidate-order, current-positive suppression, authorization-decline, wrong-identifier, and controlled-success cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T010 [US1] Run the new focused suite and record the expected pre-implementation failures

### Implementation for User Story 1

- [x] T011 [US1] Add repeatable concrete protocol candidate arguments and mappings in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T012 [US1] Supply normalized candidates to S139 and project the first selected reachability or TLS step without adding a second sequencing policy in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T013 [US1] Adapt one selected protocol step into existing low-level arguments with HAR and key logging disabled in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T014 [US1] Refactor the low-level executor to return a crate-private terminal observation outcome while preserving `run` behavior in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T015 [US1] Make the focused User Story 1 tests pass and retain every low-level calibration test in `crates/fragcap-cli/tests/cli_calibrate.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`

**Checkpoint**: One invocation reaches no more than one exact authorized reachability or protocol attempt.

---

## Phase 4: User Story 2 - Carry Only Observed Candidates Forward (Priority: P2)

**Goal**: Derive automatic continuation candidates only from concrete final-client terminal observations.

**Independent Test**: Mixed final-client, launcher, intermediate, unknown, unrouted, uncorrelated, duplicate, and reordered observations produce one stable eligible candidate set.

### Tests for User Story 2

- [x] T016 [US2] Add failing facade eligibility, ordering, and deduplication cases in `crates/fragcap/src/deep_capture/policy.rs`
- [x] T017 [US2] Add failing controlled reachability continuation and non-observation cases in `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 2

- [x] T018 [US2] Export one pure candidate derivation helper beside compatibility fact policy in `crates/fragcap/src/deep_capture/policy.rs` and `crates/fragcap/src/deep_capture/api.rs`
- [x] T019 [US2] Consume typed terminal observations, merge eligible candidates with unresolved requests, and generate one bounded continuation in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T020 [US2] Re-read target facts and re-run S139 after the session so only current exact facts determine completed and remaining coverage in `crates/fragcap-cli/src/commands/calibrate.rs`

**Checkpoint**: Generated candidates are observed, bounded, target-scoped, and never facts by themselves.

---

## Phase 5: User Story 3 - Report Coverage and Continue Safely (Priority: P3)

**Goal**: Expose stable requested, observed, completed, and remaining coverage in human and JSON modes.

**Independent Test**: No-candidate, already-positive, successful, refused, failed, warm, and missing-observation cases retain one stable field set and parseable next commands.

### Tests for User Story 3

- [x] T021 [P] [US3] Add failing guided event serialization and human rendering cases in `crates/fragcap-cli/src/events.rs`
- [x] T022 [P] [US3] Add failing no-candidate, already-positive, warm-preservation, limitation, terminal failure, and parseable continuation cases in `crates/fragcap-cli/tests/cli_calibrate.rs` and `crates/fragcap-cli/tests/cli_reference.rs`

### Implementation for User Story 3

- [x] T023 [US3] Extend `calibration.guidance` additively with requested, observed, completed, and remaining arrays in `crates/fragcap-cli/src/events.rs`
- [x] T024 [US3] Implement coverage-unknown, requested-coverage-complete, selected, completed, not-observed, refused, and failed guidance without suppressing existing Deep Capture events in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T025 [US3] Preserve candidates across warm retry and generate only durable target and effective-store continuation commands in `crates/fragcap-cli/src/commands/calibrate.rs`

**Checkpoint**: Human and machine consumers can distinguish requests, observations, facts, and remaining work.

---

## Phase 6: Documentation, Audit, and Completion

**Purpose**: Reconcile shipped truth, verify scope, and prepare the pull request.

- [x] T026 [P] Update the architecture record and outline in `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md`
- [x] T027 [P] Record S141 sequencing and bounded #380 follow-up in `docs/plans/README.md` and `AGENTS.md`
- [x] T028 [P] Update the public CLI reference in `site/content/docs/reference/cli.mdx`
- [x] T029 [P] Add changed and decisions fragments in `changelog.d/S141-guided-protocol-calibration.changed.md` and `changelog.d/S141-guided-protocol-calibration.decisions.md`
- [x] T030 Run the S141 quickstart, complete a requirement-to-test audit, and mark all checklists and tasks complete in `specs/141-guided-protocol-calibration/`
- [x] T031 Run `/speckit-converge`, append and implement any remaining traceable work, or record a clean convergence result
- [x] T032 Run `cargo xtask ci`, encoding, punctuation, mojibake, diff, dependency, and worktree checks
- [x] T033 Commit, push the authorized feature branch, open a PR closing #396, and set the Project Stage to PR review
- [ ] T034 Resolve every first-round review finding, trigger at most one `@Codex review` second round, resolve every second-round finding, and wait for all CI checks to pass

---

## Dependencies & Execution Order

- Phase 1 is complete and gates all later phases.
- Phase 2 freezes the additive contracts and must demonstrate red before implementation.
- User Story 1 supplies the typed terminal outcome needed by User Story 2.
- User Story 2 supplies coverage sets needed by User Story 3.
- Phase 6 follows all user stories; convergence is blocking before the repository gate.

## Parallel Opportunities

- T005, T006, and T007 touch independent contract-test surfaces.
- T021 and T022 separate event-unit and command-integration tests.
- T026 through T029 touch distinct documentation and changelog files after behavior is final.

## Implementation Strategy

1. Freeze candidate parsing, policy, event, and executor-outcome expectations.
2. Preserve routing precedence and execute one S139-selected protocol attempt.
3. Derive only eligible observed candidates and reassess durable facts.
4. Stabilize coverage guidance and parseable continuation commands.
5. Reconcile documentation, converge, run the full repository gate, and complete the authorized review cycle.

## Scope Guardrails

- No automatic target registration, internal multi-attempt loop, persisted workflow state, or resume.
- No hidden or system-wide trust, pinning bypass, target process access, or target key extraction.
- No new dependency, lockfile package, storage migration, or artifact schema.
- No change to existing low-level calibration semantics.
- Issue #396 closes on merge; parent #380 remains open.
