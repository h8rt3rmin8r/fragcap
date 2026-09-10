# Tasks: Guided Target Discovery and Registration

**Input**: Design documents from `specs/142-guided-target-registration/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-021 and the autopilot TDD protocol. Test tasks precede implementation and must demonstrate the expected red state.

## Phase 1: Setup and Specification

**Purpose**: Establish the tracked slice and complete its reviewable intent.

- [x] T001 Create issue #398 as an S142 child of #380, add it to the repository delivery Project, and set its Slice, Status, and Stage fields
- [x] T002 Author and validate the S142 specification and requirements/security checklists in `specs/142-guided-target-registration/spec.md` and `specs/142-guided-target-registration/checklists/`
- [x] T003 Complete research, data model, CLI contract, quickstart, and implementation plan in `specs/142-guided-target-registration/`

---

## Phase 2: Foundational Contract Tests

**Purpose**: Freeze stored-first selection, plan integrity, input, and event contracts before implementation.

- [x] T004 Run focused baseline tests for target selection, discovery registration, events, help, and guided calibration
- [x] T005 [P] Add failing pure tests for stored precedence, exact candidate matching, ambiguity, canonical plan identity, and candidate drift in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T006 [P] Add failing additive registration-plan and registration-outcome event tests in `crates/fragcap-cli/src/events.rs`
- [x] T007 Add failing controlled integration scaffolding for unregistered discovery candidates and sequential authorization responses in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: New tests fail only because the S142 fallback, plan, and events do not exist.

---

## Phase 3: User Story 1 - Find One Unregistered Installed Game (Priority: P1)

**Goal**: Preserve every stored outcome and select only one exact discovered Steam-id or display-name candidate after a clean stored miss.

**Independent Test**: Stored resolved and ambiguous inputs skip discovery; exact fixture candidates produce one plan; discovered ambiguity and no-match produce no registration or session effects.

### Tests for User Story 1

- [x] T008 [US1] Add failing stored-row precedence, stored ambiguity, exact Steam-id, exact display-name, discovered ambiguity, and no-match cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T009 [US1] Run the new User Story 1 suite and record expected pre-implementation failures

### Implementation for User Story 1

- [x] T010 [US1] Expose the single production discovery composition within the CLI crate and add a controlled fixture provider in `crates/fragcap-cli/src/commands/targets.rs` and `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T011 [US1] Refactor stored target selection into a typed stored-first decision without parsing error text in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T012 [US1] Implement exact Steam-id and display-name candidate selection with complete ambiguity and discovery diagnostics in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T013 [US1] Make the focused User Story 1 tests pass while preserving existing stored target semantics in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: One exact installed candidate can be proposed, and every other selection state is effect-free and explicit.

---

## Phase 4: User Story 2 - Confirm One Exact Registration (Priority: P2)

**Goal**: Bind, display, confirm, revalidate, and idempotently persist exactly one discovered candidate through the existing shared registration operation.

**Independent Test**: Interactive decline, closed/invalid exact input, flush failure, candidate drift, same-identity race, and confirmed insertion produce stable distinct outcomes and never broaden authority.

### Tests for User Story 2

- [x] T014 [US2] Add failing plan canonicalization, field-sensitivity, domain-separation, and constant-time exact-line tests in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T015 [US2] Add failing decline, invalid, closed, drift, idempotent race, and confirmed registration integration cases in `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation for User Story 2

- [x] T016 [US2] Implement the deterministic complete registration-plan model and versioned identifier in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T017 [US2] Add checked plan emission, flush-before-input, human default-no, and exact structured confirmation in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T018 [US2] Re-run discovery, compare canonical authority, and pass only the unchanged candidate to `fragcap::targets::register_candidate` in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T019 [US2] Recover the exact inserted or already-present target by canonical anchor or install root and refuse conflicts in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T020 [US2] Add registration-plan and registration-outcome variants and stable human/JSON rendering in `crates/fragcap-cli/src/events.rs`
- [x] T021 [US2] Make the focused User Story 2 tests pass and prove zero pre-confirmation target-row/session effects in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: One unchanged candidate is registered or reused only after exact operator consent.

---

## Phase 5: User Story 3 - Continue Guided Calibration by Durable Identity (Priority: P3)

**Goal**: Re-enter the unchanged S139-S141 path with the resulting stable target while retaining separate registration and session authorization.

**Independent Test**: A confirmed candidate yields an existing guidance outcome under its durable id, unsupported topology stays a limitation, and a session path consumes a second distinct input line.

### Tests for User Story 3

- [x] T022 [US3] Add failing durable-id handoff, unresolved-topology no-effect, and separate later authorization cases in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T023 [P] [US3] Add failing reference and event compatibility assertions in `crates/fragcap-cli/tests/cli_reference.rs` and `crates/fragcap-cli/src/events.rs`

### Implementation for User Story 3

- [x] T024 [US3] Refactor the existing guided body to accept one pre-resolved durable target after the S142 front door in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T025 [US3] Continue registration success into the existing proposal without promoting hints, while preserving the later warm, execution, fact, continuation, and separate authorization flow for authoritative targets in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T026 [US3] Make the focused User Story 3 and all existing S140-S141 tests pass in `crates/fragcap-cli/tests/cli_calibrate.rs`

**Checkpoint**: Registration removes the manual prerequisite without changing calibration authority or session count.

---

## Phase 6: Documentation, Audit, and Completion

**Purpose**: Reconcile shipped truth, verify scope, and prepare the pull request.

- [x] T027 [P] Update the architecture record and outline in `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md`
- [x] T028 [P] Record S142 sequencing and bounded #380 follow-up in `docs/plans/README.md` and `AGENTS.md`
- [x] T029 [P] Update the public CLI reference in `site/content/docs/reference/cli.mdx`
- [x] T030 [P] Add changed and decisions fragments in `changelog.d/S142-guided-target-registration.changed.md` and `changelog.d/S142-guided-target-registration.decisions.md`
- [x] T031 Run the S142 quickstart, complete a requirement-to-test audit, and mark all checklists and tasks complete in `specs/142-guided-target-registration/`
- [x] T032 Run `/speckit-converge`, append and implement any remaining traceable work, or record a clean convergence result
- [x] T033 Run `cargo xtask ci`, encoding, punctuation, mojibake, diff, dependency, and worktree checks
- [x] T034 Commit, push the authorized feature branch, open a PR closing #398, and set the Project Stage to PR review
- [ ] T035 Resolve every first-round review finding, trigger at most one `@Codex review` second round, resolve every second-round finding, and wait for all CI checks to pass

---

## Dependencies & Execution Order

- Phase 1 is complete and gates all later phases.
- Phase 2 freezes selection, plan, input, and event contracts and must demonstrate red before implementation.
- User Story 1 supplies the exact candidate needed by User Story 2.
- User Story 2 supplies the durable target needed by User Story 3.
- Phase 6 follows all user stories; convergence is blocking before the repository gate.

## Parallel Opportunities

- T005 and T006 touch independent unit-test surfaces.
- T022 and T023 separate command integration and compatibility tests.
- T027 through T030 touch distinct documentation and changelog files after behavior is final.

## Implementation Strategy

1. Freeze stored precedence, exact candidate selection, plan identity, input, and event expectations.
2. Reuse the production discovery composition and retain complete ambiguity/loss reporting.
3. Bind, confirm, revalidate, and register one candidate through the shared operation.
4. Re-enter S139-S141 by durable identity with a separate later authorization.
5. Reconcile documentation, converge, run the full repository gate, and complete the authorized review cycle.

## Scope Guardrails

- No automatic bulk registration, fuzzy selection, or fabricated launch topology.
- No process control, hidden or system-wide trust, pinning bypass, target process access, or target key extraction.
- No new dependency, lockfile package, storage migration, artifact schema, internal multi-attempt loop, or workflow persistence.
- No change to existing low-level calibration semantics.
- Issue #398 closes on merge; parent #380 remains open.

## Convergence Result

The post-implementation specification, plan, task, contract, documentation, and test audit found no uncovered requirement or inconsistent authority boundary. No follow-up task was appended.
