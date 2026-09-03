# Tasks: Complete Process Lifecycle Evidence

**Input**: Design documents from `specs/123-process-lifecycle-evidence/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-018 and the autopilot TDD protocol.

## Phase 1: Setup and Contracts

- [x] T001 Validate the active feature selector, branch, and clean dependency baseline in `.specify/feature.json` and `Cargo.lock`
- [x] T002 [P] Finalize the process trace JSON Lines contract in `specs/123-process-lifecycle-evidence/contracts/process-trace.md`
- [x] T003 [P] Validate process-evidence security requirements in `specs/123-process-lifecycle-evidence/checklists/security.md`

## Phase 2: Foundational Evidence Vocabulary

- [x] T004 Add failing deterministic all-flow snapshot tests in `crates/fragcap-core/src/flow.rs`
- [x] T005 Add a deterministic immutable all-flow snapshot to `crates/fragcap-core/src/flow.rs`
- [x] T006 Add failing process trace framing, process-instance, PID-reuse, and ordering tests in `crates/fragcap/src/deep_capture/process.rs`
- [x] T007 Define bounded capture-process evidence, process instance, limitation, socket-owner interval, summary, and reader types in `crates/fragcap/src/deep_capture/process.rs`
- [x] T008 Export the process evidence API through `crates/fragcap/src/deep_capture/mod.rs`

## Phase 3: User Story 1 - Audit the Managed Process Chronology (Priority: P1)

**Goal**: Preserve the exact launch, relevant process, stage, socket-owner, exit, and terminal chronology.

**Independent Test**: A controlled cold launch produces a versioned, ordered, reconciled process trace with one complete trailer.

- [x] T009 [US1] Add failing orchestrator evidence-retention tests in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T010 [US1] Retain startup snapshots and bounded raw process events in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T011 [US1] Retain managed launch receipt, stage transitions, watcher report, and terminal state in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T012 [US1] Return capture process evidence through `CaptureOutcome` in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T013 [US1] Thread process evidence through the composed capture entry in `crates/fragcap-cli/src/commands/capture.rs`
- [x] T014 [US1] Reconcile relevant process instances and ancestry in `crates/fragcap/src/deep_capture/process.rs`
- [x] T015 [US1] Serialize header, chronology, terminal record, and trailer in `crates/fragcap/src/deep_capture/process.rs`

## Phase 4: User Story 2 - Distinguish Missing Evidence from Process Truth (Priority: P2)

**Goal**: Preserve every limitation and prevent PID reuse, ordering, and missing-event cases from fabricating identity.

**Independent Test**: Permutations and injected loss produce stable partial traces with exact counts and no cross-lifetime ownership.

- [x] T016 [P] [US2] Add PID-reuse and out-of-order permutation cases in `crates/fragcap/src/deep_capture/process.rs`
- [x] T017 [P] [US2] Add watcher, retention, missing-exit, and missing-parent loss cases in `crates/fragcap/src/deep_capture/process.rs`
- [x] T018 [US2] Resolve exits and parent instances only within observed process lifetimes in `crates/fragcap/src/deep_capture/process.rs`
- [x] T019 [US2] Emit typed limitations and derive trace completeness from all loss authorities in `crates/fragcap/src/deep_capture/process.rs`
- [x] T020 [US2] Parse and validate complete, partial, unavailable, and crash-prefix streams in `crates/fragcap/src/deep_capture/process.rs`

## Phase 5: User Story 3 - Reconcile Process, Packet, and Application Anchors (Priority: P3)

**Goal**: Use existing flow registry truth to align process, packet, application, compatibility, and manifest claims.

**Independent Test**: Every process-side flow anchor matches packet and application evidence, and every unresolved owner weakens completeness explicitly.

- [x] T021 [P] [US3] Add flow-owner transition and unretained packet-evidence tests in `crates/fragcap/src/deep_capture/process.rs`
- [x] T022 [US3] Collapse retained packet observations into deterministic socket-owner intervals in `crates/fragcap/src/deep_capture/process.rs`
- [x] T023 [US3] Replace placeholder process trace construction in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T024 [US3] Make controlled target lifecycle evidence use the same process trace contract in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T025 [US3] Derive compatibility process evidence state from the process trace trailer in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T026 [US3] Derive manifest process-trace finalization, completeness, loss, and correlation from the process trace reader in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T027 [US3] Add bundle-level process trace and manifest reconciliation tests in `crates/fragcap-cli/tests/cli_deep_capture.rs`

## Phase 6: Documentation and Verification

- [x] T028 [P] Add process instance and process lifecycle stream glossary entries in `docs/glossary/capture-and-networking.md` and `docs/glossary/index.md`
- [x] T029 [P] Record S123 shipped scope in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T030 [P] Add S123 feature and dated decision fragments in `changelog.d/`
- [x] T031 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T032 Run focused process trace, orchestrator, and Deep Capture bundle tests from `specs/123-process-lifecycle-evidence/quickstart.md`
- [x] T033 Run `cargo xtask ci`, text hygiene, dependency lock, and forbidden-capability checks
- [x] T034 Mark every completed task in `specs/123-process-lifecycle-evidence/tasks.md` and perform the final scope audit

## Dependencies and Execution Order

- Phase 1 establishes the reviewed contract.
- Phase 2 blocks every user story.
- User Story 1 establishes chronology, User Story 2 hardens identity and loss truth, and User Story 3 adds cross-artifact reconciliation.
- Documentation can proceed after the contract stabilizes; full verification follows all implementation work.

## Parallel Opportunities

- T002 and T003 are independent requirements checks.
- T016 and T017 cover distinct reconciliation failure classes.
- T028, T029, and T030 touch separate documentation groups after the contract is final.

## Implementation Strategy

1. Make the flow and process evidence unit tests fail before adding production behavior.
2. Complete the bounded capture report and chronology before cross-artifact integration.
3. Add loss and reuse cases before manifest claims.
4. Run focused tests after each story, then the complete repository gate.
