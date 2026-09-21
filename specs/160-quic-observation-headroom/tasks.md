# Tasks: S160 QUIC observation headroom

**Input**: Design documents from `specs/160-quic-observation-headroom/`

**Prerequisites**: `spec.md`, `research.md`, `data-model.md`, `contracts/queue-and-report.md`, `plan.md`

**Testing rule**: Add the deterministic failing regression before each
behavioral correction. Do not weaken loss, memory, cleanup, security, or
first-attempt hosted acceptance.

## Phase 1: Setup and defect authority

- [x] T001 Record the concrete merged-main failure in GitHub issue #435 under the v0.10.2 stabilization milestone
- [x] T002 Create and validate the complete S160 spec-kit design set in specs/160-quic-observation-headroom/
- [x] T003 Run the blocking spec-kit analysis gate and resolve every finding across specs/160-quic-observation-headroom/

## Phase 2: Foundational regression seams

- [x] T004 Add a deterministic private post-readiness writer stall seam in crates/fragcap/src/deep_capture/application.rs without changing production startup behavior
- [x] T005 Add a pure accepted-versus-lost payload projection boundary in performance/native-proxy/src/workloads.rs for mutation-oriented tests

## Phase 3: User Story 1 - Trust the first Windows result (Priority: P1)

**Goal**: The canonical finite QUIC campaign has enough shared bounded queue headroom to remain lossless when its ready consumer is descheduled.

**Independent Test**: Hold the consumer after readiness, admit exactly the shared default capacity, prove the first excess event is counted without blocking, release the consumer, and finish with ordered zero-current ownership.

- [x] T006 [US1] Add the ready-consumer exact-capacity and one-past-capacity regression in crates/fragcap/src/deep_capture/application.rs, with the retained main failure proving the prior 4,096-event authority was insufficient
- [x] T007 [US1] Define and document the shared 16,384-event default capacity in crates/fragcap/src/deep_capture/application.rs
- [x] T008 [US1] Replace the ordinary native application artifact literal with the shared capacity in crates/fragcap/src/deep_capture/native.rs while leaving lifecycle capacity unchanged
- [x] T009 [US1] Replace both canonical performance artifact literals with the shared capacity in performance/native-proxy/src/workloads.rs
- [x] T010 [US1] Update the reviewed queue ceiling and its validator expectations in performance/native-proxy-budgets-v1.json and xtask/src/performance.rs as required
- [x] T011 [US1] Add exact producer-attempt accounting in crates/fragcap-proxy/src/application.rs and crates/fragcap/src/deep_capture/application.rs, advance new campaign and worker records to performance report schema version 2, publish attempted events and exact queue capacity per sample, then make excess total burst size a hard invariant in performance/native-proxy/src/workloads.rs and performance/native-proxy/src/main.rs
- [x] T012 [US1] Run focused application queue and performance-authority tests without running the installed product

## Phase 4: User Story 2 - Read truthful conservation evidence (Priority: P1)

**Goal**: Performance samples include accepted and lost payload observations exactly once under the existing conservation equation.

**Independent Test**: Project controlled retained, omitted, queue-dropped, and storage-dropped values, then mutate each value independently and prove every mismatch is rejected.

- [x] T013 [US2] Add a regression for retained payload plus queue loss and storage loss in performance/native-proxy/src/workloads.rs that fails against the prior saturating-subtraction projection
- [x] T014 [US2] Partition writer-observed bytes into retained, omitted, and storage-dropped dispositions, then add only pre-writer queue loss into total observed bytes in performance/native-proxy/src/workloads.rs
- [x] T015 [US2] Preserve historical version 1 report validation, refuse unknown versions, and add version 2 evaluator mutation cases for all four payload dispositions in performance/native-proxy/src/main.rs and xtask/src/performance.rs
- [x] T016 [US2] Run the isolated performance harness tests and static report validator without executing the full local campaign

## Phase 5: User Story 3 - Keep the correction bounded (Priority: P2)

**Goal**: The corrected queue remains finite, nonblocking, loss-accounted, memory-bounded, and operationally unchanged outside capacity.

**Independent Test**: Exercise exact capacity and excess admission, validate unchanged artifact records and cleanup, and enforce existing memory, artifact, task, and shutdown ceilings.

- [x] T017 [US3] Extend application queue assertions for peak, current, accepted, dropped, byte-loss, order, and terminal drain in crates/fragcap/src/deep_capture/application.rs
- [x] T018 [P] [US3] Record the superseding queue and accounting decision in changelog.d/S160-quic-observation-headroom.decisions.md and changelog.d/S160-quic-observation-headroom.fixed.md
- [x] T019 [P] [US3] Reconcile the current contract in docs/security/deep-capture-performance.md and docs/fragcap-specification.md
- [x] T020 [US3] Append S160 chronologically to docs/plans/README.md with the explicit S150 and S158 deviation rationale

## Phase 6: Cross-cutting verification and delivery

- [x] T021 Run cargo fmt --all -- --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test --all --locked, cargo xtask ci, cargo xtask msrv, and cargo xtask neutral in the foreground
- [x] T022 Run UTF-8 without BOM, LF, trailing-whitespace, mojibake, and git diff hygiene checks over every changed text file
- [x] T023 Re-run spec-kit analysis and convergence, reconcile every S160 task, and confirm issue #435 plus the repository project reflect delivery state
- [ ] T024 Commit S160, push codex/s160-quic-observation-headroom, open and attach the official pull request
- [ ] T025 Preserve and inspect every first-attempt hosted conclusion, especially Windows native performance, without using a rerun as acceptance
- [ ] T026 Address every review comment and hosted defect within at most two review rounds, replying and resolving each thread
- [ ] T027 Confirm every required check is green on the final head, update issue #435 with exact evidence, and request the operator's final review and merge ritual

## Dependencies and execution order

- T001 through T003 establish the defect and design authority before code.
- T004 and T005 create independent test seams and may proceed in parallel after T003.
- T006 precedes T007 through T011.
- T013 precedes T014 through T016 and may proceed independently from T006 through T012 after T005.
- T017 follows T006 through T011. T018 and T019 may proceed in parallel after behavior settles; T020 follows them.
- T021 through T023 require all implementation and documentation tasks.
- T024 through T027 require local convergence and the established pull-request delivery workflow.

## Parallel execution examples

- **US1 and US2**: After T004 and T005, queue-capacity work in the facade can proceed independently from pure performance projection tests.
- **US3**: Changelog and current-contract documentation can be authored in parallel after final behavior is fixed.

## Implementation strategy

1. Establish the exact post-readiness failure with deterministic tests.
2. Centralize the finite capacity and route every ordinary producer through it.
3. Correct report projection without changing artifact data.
4. Reconcile contracts, run every local source gate, then use first-attempt hosted Windows evidence as acceptance.

The minimum viable correction is User Stories 1 and 2 together. Capacity without
truthful conservation leaves misleading evidence; projection without headroom
leaves main nondeterministic.
