# Tasks: S173 Bundle access stabilization

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [contracts/access.md](contracts/access.md).

## Phase 1: Setup

- [x] T001 Validate S173 issue/directory/branch identity and complete spec-kit artifacts in specs/173-bundle-access-stabilization/.

## Phase 2: Foundation

- [ ] T002 Establish hidden disposable Windows elevated test prerequisites and record actual token stages in specs/173-bundle-access-stabilization/verification.md (FR-009).
- [ ] T003 Add scoped process-query lint negative/positive regressions in xtask/src/lint.rs before enabling the exact shell-token exception (FR-002, FR-010).

## Phase 3: US1 automatic ordinary output

**Goal**: Elevated same-account output works automatically and is actually readable in the ordinary context.

**Independent test**: Genuine elevated production resolver and full synthetic writer population, plus ordinary/access-denial fixtures.

- [ ] T004 [US1] Add token-stage, shell/account/session/elevation/integrity refusal regressions in crates/fragcap/src/deep_capture/access/windows.rs (FR-001, FR-002, FR-003).
- [ ] T005 [US1] Implement exact shell-token acquisition and stage-preserving errors in crates/fragcap/src/deep_capture/access/windows.rs; add Windows messaging feature in crates/fragcap/Cargo.toml and audited narrow lint exception in xtask/src/lint.rs (FR-001, FR-002, FR-003).
- [ ] T006 [US1] Add genuinely elevated production-path new-output regression with ordinary full reads, all parent/journal/atomic artifacts and unrelated-authority denial in crates/fragcap/tests/bundle_access.rs (FR-004, FR-009, SC-001, SC-003).
- [ ] T007 [US1] Correct default recipient guidance in crates/fragcap-cli/src/commands/bundle.rs and verify Deep Capture/calibration shared resolver integration (FR-001, FR-003).

## Phase 4: US2 retirement

**Goal**: Retired commands are unavailable while maintained legacy operations work.

**Independent test**: Both verbs rejected before effects, retained cleanup/collection/Doctor tests pass and maintained help contains no retired instructions.

- [ ] T008 [US2] Add retired-verb parser/integration failures in crates/fragcap-cli/tests/cli_bundle.rs and crates/fragcap-cli/src/cli.rs (FR-006, SC-002).
- [ ] T009 [US2] Remove repair API/module and exclusive support/tests from crates/fragcap/src/deep_capture/mod.rs, access_repair.rs, access.rs, access/windows.rs and crates/fragcap/tests/bundle_access.rs; preserve shared legacy pinning (FR-006, FR-007).
- [ ] T010 [US2] Remove command branches and repair-only helpers/tests in crates/fragcap-cli/src/cli.rs and commands/bundle.rs; preserve legacy cleanup/collection/Doctor regressions (FR-006, FR-007).
- [ ] T011 [US2] Reconcile current guidance and regressions in crates/fragcap-cli/src/session_ux.rs, workflow_help.rs and maintained docs while preserving historical records (FR-008, SC-002).

## Phase 5: US3 explicit recipient

**Goal**: Genuine different-account output retains authenticated semantics.

**Independent test**: Existing actual handoff/token tests including wrong-path, identity, elevation and timeout.

- [ ] T012 [US3] Run and retain authenticated ordinary recipient handoff regressions in crates/fragcap/src/deep_capture/access/windows.rs and explicit CLI argument contracts in crates/fragcap-cli/src/cli.rs (FR-005, SC-003).

## Phase 6: Verification and delivery

- [ ] T013 Record the narrow architecture/lint deviation in AGENTS.md, docs/fragcap-specification.md and changelog.d/S173-bundle-access.decisions.md; add user-visible fragment and chronological docs/plans/README.md entry (FR-008, FR-010).
- [ ] T014 Run full gates, genuine elevated regression and issue acceptance mapping; record evidence/limits in specs/173-bundle-access-stabilization/verification.md and issue-acceptance.md (FR-009, FR-010, SC-001, SC-002, SC-003, SC-004).
- [ ] T015 Commit, push and publish official PR closing #476/#477; reconcile every review and exact final-head CI with at most a second requested review round; record handoff in specs/173-bundle-access-stabilization/verification.md (SC-004).

## Dependencies and execution

T001 precedes foundation and US1. T004 precedes T005. T008 precedes T009-T011. US3 uses shared recipient code and is verified after its changes. T013-T015 follow all stories. Shared files are edited serially. Documentation and independent fixture analysis can be considered alongside token diagnostics, but no implementation precedes passing analysis.

## Implementation strategy

US1 establishes the working automatic default and its actual evidence first. US2 removes historical repair while retaining the corrected new-output boundary. US3 confirms the independent helper contract. The slice delivers both issues in one complete PR; a partial story is not final completion.
