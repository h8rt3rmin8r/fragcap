# Tasks: v0.10.14 Release Stabilization and Candidate

**Input**: Design documents from `specs/195-v0-10-14-release/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/release-candidate.md

**Organization**: Tasks are chronological and grouped by the three independently verifiable user stories.

## Phase 1: Setup And Source Inventory

**Purpose**: Bind the slice to exact merged source, issues, branch, and release authorities before mutation.

- [x] T001 Verify clean merged `main` at `1e44519a8d34406fcd3178365c1dde68b689439d`, then create `release/0.10.14`.
- [x] T002 Create milestone v0.10.14, timing defect #566, release-delivery issue #567, and map both to S195 in the project.
- [x] T003 Run the official release-preparation dry-run and retain any concrete readiness failure.
- [x] T004 Complete specification, clarification, requirements checklists, research, design, plan, and task records under `specs/195-v0-10-14-release/`.
- [x] T005 Perform the read-only Spec-Kit consistency analysis and resolve every finding before implementation.

---

## Phase 2: User Story 1 - Deterministic Source Readiness (Priority: P1)

**Goal**: Correct #566 without changing production timeout behavior.

**Independent Test**: The focused controlled setup-to-calibration regression passes repeatedly on Windows, while exact deadline-boundary coverage remains unchanged and successful.

- [x] T006 [US1] Preserve the failed hosted attempt and successful rerun as the defect receipt, then compare `crates/fragcap-cli/tests/cli_calibrate.rs` with adjacent controlled cases and exact deadline tests.
- [x] T007 [US1] Change only the test-controlled launch allowance in `crates/fragcap-cli/tests/cli_calibrate.rs` from one second to seven seconds and document why the duration remains one second.
- [x] T008 [US1] Run the focused setup-to-calibration integration test and the exact finite launch-deadline unit test.

**Checkpoint**: #566 has a deterministic controlled assertion, and production deadline authority is unchanged.

---

## Phase 3: User Story 2 - Coherent v0.10.14 Candidate (Priority: P1)

**Goal**: Assemble S189-S194 and the S195 correction into one internally consistent source candidate.

**Independent Test**: Candidate-bound identities agree on v0.10.14, all release fragments are consumed exactly once, public records remain on v0.10.13, and all dependency authorities pass.

- [x] T009 [US2] Repair the dated-lead format in `changelog.d/S193-native-blind-relay.decisions.md` and `changelog.d/S194-scope-calibration-journey.decisions.md` without weakening the validator.
- [x] T010 [US2] Add the S195 timing and candidate decision/fix fragments under `changelog.d/`.
- [ ] T011 [US2] Commit the complete Spec-Kit record, #566 correction, and fragment readiness repair so cargo-release can run from a clean branch.
- [ ] T012 [US2] Execute cargo-release 0.10.14 for the workspace and verify it changes only first-party version identity.
- [ ] T013 [US2] Update embedded v0.10.14 test and conformance identities, then regenerate repository-owned goldens.
- [ ] T014 [US2] Refresh first-party identity in every isolated lockfile and refresh the supply-chain inventory without accepting third-party drift.
- [ ] T015 [US2] Assemble the 2026-11-06 v0.10.14 changelog and create concise `release-notes/v0.10.14.md`, consuming every fragment once.
- [ ] T016 [US2] Reconcile `docs/fragcap-specification.md`, `docs/plans/README.md`, `CONTRIBUTING.md`, and current security documentation, removing stale #547-open prose while preserving verified v0.10.13 public markers.
- [ ] T017 [US2] Run notes, specification, documentation, changelog, conformance, graph, and generated-file validation and correct every candidate inconsistency.

**Checkpoint**: One v0.10.14 candidate exists, and no publication claim has advanced.

---

## Phase 4: User Story 3 - Official Reviewed PR (Priority: P2)

**Goal**: Deliver the exact candidate through the official PR with complete CI and bounded review reconciliation.

**Independent Test**: The final PR head has terminal successful applicable checks, exact-head package evidence, every actual review comment answered, zero unresolved threads, and no more than two Codex rounds.

- [ ] T018 [US3] Run complete local `cargo xtask ci`, MSRV, neutral, documentation build, notes, supply-chain, dependency-policy, isolated-graph, text, encoding, and mojibake checks.
- [ ] T019 [US3] Complete the post-implementation Spec-Kit convergence review and resolve every finding.
- [ ] T020 [US3] Record exact local verification and explicit unverified boundaries in `specs/195-v0-10-14-release/verification.md`, mark completed tasks, and finalize the candidate commit.
- [ ] T021 [US3] Push `release/0.10.14`, open the official PR with `Closes #566` and a non-closing reference to #567, and move both project items to PR review.
- [ ] T022 [US3] Monitor every applicable hosted check and obtain exact final-head Windows package-certification evidence.
- [ ] T023 [US3] Inspect all hosted code and security review surfaces, respond to every comment, apply required corrections, and resolve every thread.
- [ ] T024 [US3] If useful after round-one reconciliation, request exactly one second hosted Codex round, process it fully, and never request a third.
- [ ] T025 [US3] Reconfirm the final head, terminal successful checks, review count, comment disposition, issue/project state, and candidate-only evidence boundary, then hand the PR to the owner for final merge ritual.

**Checkpoint**: S195 is ready for owner review and merge. No merge, tag, publication, installed-product test, or #567 closure has occurred.

---

## Dependencies And Execution Order

- Phase 1 fixes exact source and scope before any mutation.
- User Story 1 must complete before version preparation so the candidate includes the correction.
- User Story 2 depends on User Story 1 and produces the immutable candidate inputs required by hosted qualification.
- User Story 3 depends on the completed candidate and restarts applicable checks after every source change.
- T024 is optional, bounded to one explicit request, and follows complete round-one reconciliation.

## Implementation Strategy

1. Establish exact source, tracking, dry-run failure, and complete Spec-Kit records.
2. Correct and independently verify the timing regression.
3. Assemble, validate, and commit the full patch candidate.
4. Push once locally complete, then reconcile CI and hosted review to the final exact head.
5. Stop for the owner before merge and publication.
