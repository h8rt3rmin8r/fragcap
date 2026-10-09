# Tasks: S169 packet-flow acquisition and correlation

**Input**: [plan.md](plan.md), [spec.md](spec.md), research, data model and contracts.

**Tests**: Required controlled regressions before corrective code; full repository gate.

## Phase 1: Setup

- [x] T001 Read governing documents, verify #468 and create exact branch/selector/spec in specs/169-packet-flow-acquisition-and-correlation/spec.md.
- [x] T002 Clarify authority and scope, complete requirements checklists in specs/169-packet-flow-acquisition-and-correlation/checklists/.

## Phase 2: Foundation

- [x] T003 Dispatch acquisition/correlation research and resolve owning-layer decisions in specs/169-packet-flow-acquisition-and-correlation/research.md.
- [x] T004 Generate plan, data model, contract and quickstart in specs/169-packet-flow-acquisition-and-correlation/.
- [x] T005 Run blocking read-only consistency analysis across spec.md, plan.md and tasks.md; record resolved gate in specs/169-packet-flow-acquisition-and-correlation/analysis.md.

## Phase 3: User Story 1, correlated routed client

**Goal**: Obtain exact scoped loopback flows and select the unique observed bound owner.

**Independent test**: Both-family real synthetic endpoint identities through production parser, attribution, pipeline and native join.

- [x] T006 [P] [US1] Reproduce mandatory interface/locality and fixed-filter defects with failing unit tests in crates/fragcap-cli/src/assemble.rs and crates/fragcap-core/src/filter.rs.
- [x] T007 [P] [US1] Reproduce proxy-side endpoint-order ownership with failing tests in crates/fragcap/src/session.rs.
- [x] T008 [US1] Add exact listener handoff and required loopback selection/locality in crates/fragcap-cli/src/commands/capture.rs, commands/deep_capture.rs, assemble.rs and orchestrator.rs (FR-001, FR-002, FR-004).
- [x] T009 [US1] Add private per-handle fixed endpoints and additive configuration in crates/fragcap-core/src/filter.rs and pipeline/mod.rs; prove churn, refusal/retry and physical-source isolation (FR-002, FR-004, FR-006).
- [x] T010 [P] [US1] Correct both-orientation stage-bound ownership in crates/fragcap/src/session.rs, preserving creation time, retention, unrelated behavior and unresolved competing owners (FR-003).
- [x] T011 [P] [US1] Add real synthetic endpoint/synthesized packet pipeline and process/writer reconciliation in crates/fragcap/tests/loopback_correlation.rs (FR-004, FR-005, FR-006).
- [x] T012 [P] [US1] Extend final native join, omitted/withheld/reused/ambiguous/window tests in crates/fragcap/src/deep_capture/native.rs and inspect compatibility/manifest integration coverage (FR-003, FR-005, FR-006).

## Phase 4: User Story 2, missing-evidence diagnosis

**Goal**: Keep complete exchanges with missing ownership inconclusive and provide supported inspection guidance.

**Independent test**: Existing complete-exchange absent-flow scenario names the exact evidence boundary and next action.

- [x] T013 [US2] Establish diagnosis regressions in crates/fragcap-cli/src/commands/calibrate/assessment.rs and session_ux.rs (FR-007).
- [x] T014 [US2] Project distinct bounded correlation-reason counts and Doctor inspection action in crates/fragcap-cli/src/commands/calibrate/assessment.rs, commands/calibrate.rs and session_ux.rs (FR-007).
- [x] T015 [US2] Preserve observation cutoff, unavailable/ambiguous ownership and no-positive-fact behavior in owning native/CLI tests (FR-003, FR-004, FR-005).

## Phase 5: Completion

- [x] T016 Update master specification, docs/plans/README.md, site troubleshooting and changelog.d/S169.*.md with source corrections and verification limits (FR-001, FR-007).
- [x] T017 Audit every #468 criterion in specs/169-packet-flow-acquisition-and-correlation/issue-acceptance.md and record focused/full gate evidence in verification.md (SC-001 through SC-005).
- [x] T018 Run cargo xtask ci, review full diff, verify text hygiene and commit scoped files (FR-008).
- [ ] T019 Automatically push/open official PR, attach it, resolve all reviews and green exact-head CI, request no more than one second round and hand off for owner merge (FR-008).

## Dependencies and parallel execution

T001-T005 block all implementation. Acquisition T006 -> T008 -> T009 runs independently from ownership T007 -> T010 -> T011 and native T012 plus root T013-T014. Root/agents coordinate shared signatures first; native evidence owns native.rs; root edits assessment/calibrate/session_ux/docs and manifest after acquisition handoff, acquisition owns assemble/orchestrator/capture/deep_capture/filter/pipeline, correlation owns session.rs and loopback_correlation.rs. T015 follows integration; T016-T019 follow focused proof. No concurrent edits to an owning file.

## Implementation strategy

Deliver US1 acquisition/ownership as the functional increment, then US2 evidence guidance. All scope and negative authority regressions remain required. Source changes do not demonstrate the owner machine's root cause, real driver acquisition or universal title compatibility.
