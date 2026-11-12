# Tasks: Packet Correlation Integrity

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [acceptance contract](contracts/acceptance.md), [quickstart.md](quickstart.md).

## Phase 1: Authority and reproduction

- [x] T001 Confirm merged S205 baseline, issue states, exact verification commands, and no pending recovery; keep native readback source-preserving.
- [ ] T002 Reproduce #596 in a controlled cold chain with before/after client, undeclared helper, unrelated traffic, and exact per-flow packet/proxy timing.
- [x] T003 Audit #594 positive/negative TCP and UDP decisions, fact-store write, later admission, and physical package order feasibility.

## Phase 2: Tests and correction

- [x] T004 Add failing #596 gate/correlation regressions for demonstrated cause, cross-flow isolation, and conservation.
- [x] T005 Correct only the demonstrated causal gate, omission, or proxy-window defect; preserve unrelated pass-through and bounded work.
- [ ] T006 Add #594 tests for original-present/absent independent packet authority and exact later admission/refusal; fix demonstrated disagreement.
- [ ] T007 Check normal/JSON/manifest/process-trace/compatibility parity and security/privacy invariants.

## Phase 3: Qualification and PR

- [ ] T008 Run focused tests, `cargo xtask ci`, hosted signed-driver matrix, and package certification when package behavior changes.
- [x] T009 Observe both packaged physical packet orders for #594 or explicitly retain the missing row; record #596 controlled evidence separately.
- [x] T010 Update master specification, chronological plans index, evidence, and unreleased changelog for delivered behavior.
- [ ] T011 Run Spec-Kit analysis and quality checks, commit, push, and open the authorized official PR.
- [ ] T012 Resolve all CI and review comments, request at most one explicit second Codex round, and hand a green PR to the owner without merging.
