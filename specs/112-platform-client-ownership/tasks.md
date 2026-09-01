# Tasks: Cold Platform-Client Ownership

**Input**: Design documents from `specs/112-platform-client-ownership/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/platform-launch-api.md

**Tests**: S112 requires TDD for preparation, authority transitions, security refusals, loss, compatibility evidence, and regression coverage.

**Organization**: Tasks are grouped by independently testable user story and executed chronologically.

## Phase 1: Setup And Contracts

**Purpose**: Establish the S112 artifact and source boundaries before behavior changes.

- [x] T001 Validate all S112 requirement and security checklist items in `specs/112-platform-client-ownership/checklists/`
- [x] T002 Record platform adapter, ownership state, and evidence contracts in `specs/112-platform-client-ownership/contracts/platform-launch-api.md` and `specs/112-platform-client-ownership/data-model.md`
- [x] T003 Identify the existing Steam discovery, managed-launch, Capture preparation, session, and Deep Capture fact seams in `crates/fragcap-steam/src/`, `crates/fragcap/src/`, and `crates/fragcap-cli/src/`

---

## Phase 2: Foundational Platform Plan

**Purpose**: Build the immutable side-effect-free platform value used by every story.

**Critical**: User story implementation begins only after the plan can be tested without launching a process.

- [x] T004 [P] Add failing Steam adapter preparation tests for exact root, application identity, invalid paths, and unsupported state in `crates/fragcap/src/managed_launch.rs`
- [x] T005 [P] Add regression tests proving S112 reuses the existing routing and propagation closed sets without new compatibility tokens in `crates/fragcap-targets/src/compatibility.rs`
- [x] T006 Implement the reusable platform adapter contract and immutable platform launch value in `crates/fragcap/src/managed_launch.rs`
- [x] T007 Expose the exact installed Steam root through the existing read-only discovery boundary in `crates/fragcap-steam/src/lib.rs`
- [x] T008 Implement the Steam adapter's exact root-start and retained application dispatch preparation in `crates/fragcap/src/managed_launch.rs`
- [x] T009 Run focused platform plan tests and confirm the new tests pass in `crates/fragcap/src/managed_launch.rs`

**Checkpoint**: One cold Steam plan is fully prepared and inspectable with no effects.

---

## Phase 3: User Story 1 - Own A Cold Platform Launch (Priority: P1)

**Goal**: Start the exact cold platform root under session routing, observe ownership, then dispatch the title once.

**Independent Test**: A scripted process timeline proves zero dispatches before platform binding, exactly one afterward, and terminal acquisition only beneath the bound root.

### Tests for User Story 1

- [x] T010 [P] [US1] Add failing platform-rooted profile synthesis tests in `crates/fragcap-cli/src/commands/target_resolve.rs`
- [x] T011 [P] [US1] Add failing observe-before-dispatch and at-most-once orchestrator tests in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T012 [P] [US1] Add failing Deep Capture preparation tests proving owned Steam selection while ordinary Capture remains protocol-based in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/assemble.rs`

### Implementation for User Story 1

- [x] T013 [US1] Synthesize a validated exact platform-rooted Capture profile from the resolved Steam target in `crates/fragcap-cli/src/commands/target_resolve.rs`
- [x] T014 [US1] Add explicit owned-platform preparation to Deep Capture without changing ordinary Capture in `crates/fragcap-cli/src/commands/capture.rs` and `crates/fragcap-cli/src/assemble.rs`
- [x] T015 [US1] Start the prepared platform root and authorize retained title dispatch only after platform role binding in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T016 [US1] Carry exact ownership and finite terminal acquisition bounds through `crates/fragcap/src/session.rs` and `crates/fragcap-cli/src/assemble.rs`
- [x] T017 [US1] Run the focused managed-launch, session, target-resolution, assembly, and orchestrator tests

**Checkpoint**: The cold platform path owns root start, title dispatch, and terminal client acquisition.

---

## Phase 4: User Story 2 - Refuse Warm And Escaped Paths (Priority: P2)

**Goal**: Make every unowned or incomplete platform path a named non-success outcome.

**Independent Test**: Offline snapshots and event permutations prove pre-effect warm refusal and no terminal ownership for escaped, competing, exited, lost, failed, or timed-out paths.

### Tests for User Story 2

- [x] T018 [P] [US2] Add failing warm, missing, unsupported, and uncertain preflight refusal tests in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T019 [P] [US2] Add failing platform-exit, dispatch-failure, escaped-client, ambiguity, watcher-loss, and timeout session tests in `crates/fragcap/src/session.rs` and `crates/fragcap-cli/src/orchestrator.rs`

### Implementation for User Story 2

- [x] T020 [US2] Enforce conservative warm platform refusal and exact preparation errors before effects in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T021 [US2] Add named platform ownership and escaped-descendant reconciliation outcomes in `crates/fragcap/src/session.rs`
- [x] T022 [US2] Surface platform start, dispatch, exit, escape, loss, and deadline outcomes through existing CLI events and summaries in `crates/fragcap-cli/src/orchestrator.rs` and `crates/fragcap-cli/src/output.rs`
- [x] T023 [US2] Run focused negative-path and no-effect ordering tests

**Checkpoint**: No warm, escaped, ambiguous, incomplete, or lost path can be mislabeled cold or terminal.

---

## Phase 5: User Story 3 - Preserve Separate Compatibility Evidence (Priority: P3)

**Goal**: Retain routing reachability and platform-to-client propagation as separate truthful facts.

**Independent Test**: Controlled observation permutations vary root ownership, client ancestry, and proxy correlation independently and produce exact separate fact values.

### Tests for User Story 3

- [x] T024 [P] [US3] Add failing routing-versus-propagation evidence permutation tests in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T025 [P] [US3] Add failing credential-free controlled integration coverage in `crates/fragcap-cli/tests/cli_deep_capture.rs`

### Implementation for User Story 3

- [x] T026 [US3] Reconcile owned platform ancestry with final-client proxy observations without collapsing routing and propagation in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T027 [US3] Persist and render separate platform routing, propagation, process, socket, loss, and omission evidence through existing artifacts in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T028 [US3] Run the focused compatibility and controlled integration tests

**Checkpoint**: Positive propagation requires both exact owned ancestry and final-client proxy evidence.

---

## Phase 6: Documentation And Full Verification

**Purpose**: Reconcile the architecture of record and prove the complete slice.

- [x] T029 [P] Update S112 architecture and ordering in `AGENTS.md`, `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md`
- [x] T030 [P] Add user-visible and architecture-decision fragments in `changelog.d/S112-platform-client-ownership.added.md` and `changelog.d/S112-platform-client-ownership.decisions.md`
- [x] T031 Mark every completed task in `specs/112-platform-client-ownership/tasks.md` and re-run cross-artifact analysis
- [x] T032 Run `cargo xtask ci` and resolve every failure
- [x] T033 Run `cargo xtask msrv` and `cargo xtask neutral` and resolve every failure
- [x] T034 Inspect the complete diff for encoding, mojibake, prohibited punctuation, scope, dependencies, and credential-free fixtures

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup And Contracts**: Starts immediately.
- **Foundational Platform Plan**: Depends on artifact setup and blocks every user story.
- **User Story 1**: Depends on the immutable platform plan.
- **User Story 2**: Depends on the owned launch transitions from User Story 1.
- **User Story 3**: Depends on exact process ownership from User Stories 1 and 2.
- **Documentation And Full Verification**: Depends on all desired user stories.

### User Story Dependencies

- **User Story 1**: The minimum viable S112 path.
- **User Story 2**: Extends User Story 1 with mandatory security and truthfulness failures.
- **User Story 3**: Uses the ownership result but remains independently testable through observation permutations.

### Parallel Opportunities

- T004 and T005 touch independent crates.
- T010, T011, and T012 establish independent failing boundaries before shared implementation.
- T018 and T019 cover independent preflight and runtime failures.
- T024 and T025 cover unit and integration evidence independently.
- T029 and T030 touch separate documentation artifacts after implementation settles.

## Implementation Strategy

1. Complete the value-only platform preparation and its tests.
2. Add the observe-before-dispatch transition through the shared Capture path.
3. Complete every negative ownership path before accepting positive evidence.
4. Reconcile routing and propagation after exact ownership exists.
5. Update the architecture of record, run the full gates, and inspect the final diff.

## Notes

- Tests precede implementation for each story.
- The real Steam path is optional tier-2 validation; CI uses synthetic roots, process timelines, identifiers, and loopback traffic.
- No committed fixture may contain a real account, installed-library inventory, credential, or game capture.
