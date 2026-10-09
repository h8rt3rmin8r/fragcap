# Tasks: S172 v0.10.6 release preparation

## Phase 1 - Setup

- [x] T001 Record the exact merged base and candidate scope in `spec.md`.
- [x] T002 Resolve scope, dependency disposition and authorization in `spec.md` Clarifications.
- [x] T003 Complete `checklists/requirements.md` and `checklists/release.md`.

## Phase 2 - Foundation

- [x] T004 Research release identity and stale failure evidence in `research.md`.
- [x] T005 Complete `plan.md`, `data-model.md`, `contracts/release-candidate.md` and `quickstart.md`.
- [ ] T006 Run blocking spec-kit analysis, record its result in `analysis.md` and commit the specification gate.

## Phase 3 - Accurate Candidate (US1)

**Independent test**: All first-party/release-bound versions and generated fixtures agree on 0.10.6; historical/public evidence retains its recorded identity.

- [ ] T007 [US1] Preview and execute version-only preparation for `Cargo.toml` and `Cargo.lock`.
- [ ] T008 [US1] Refresh `fuzz/Cargo.lock` and `performance/native-proxy/Cargo.lock` offline and inspect only first-party version changes.
- [ ] T009 [US1] Update release-bound sink/Windows assertions, conformance matrix/report, review template and master Applies-To in the plan inventory.
- [ ] T010 [US1] Regenerate `fixtures/goldens/` and CLI capture/extcap golden suites through their update switches.
- [ ] T011 [US1] Snapshot and reconcile candidate digests in `supply-chain/policy-v1.json` without changing third-party versions or audit dates.
- [ ] T012 [US1] Add S172 dated decision/security fragments, assemble all current fragments into `CHANGELOG.md` and author `release-notes/v0.10.6.md`.

## Phase 4 - Reconciled Gates (US2)

**Independent test**: Updated current source passes the existing HTTPS fixture and site frozen build/browser gates; no security/protocol assertion is weakened.

- [ ] T013 [US2] Inspect #470's exact patch and integrate only `site/package.json` and `site/pnpm-lock.yaml`.
- [ ] T014 [US2] Run current-source `https_proxy` controlled coverage and retain the corrected fixture in `crates/fragcap-proxy/tests/https_proxy.rs`; repair only a reproduced failure.
- [ ] T015 [US2] Run site frozen install, unit, production export and accessibility gates; record results in `verification.md`.
- [ ] T016 [US2] Run focused release gates, `cargo xtask ci`, MSRV and neutral gates and record exact evidence in `verification.md`.

## Phase 5 - Reviewable Handoff (US3)

**Independent test**: Candidate handoff accurately separates local verification from hosted and public states, and the final feature branch is cleanly committed.

- [ ] T017 [US3] Create `docs/maintainers/v0.10.6-release-handoff.md` with local results and exact later authorization/verification steps.
- [ ] T018 [US3] Append chronological S172 entry to `docs/plans/README.md` and label `spec.md` and `verification.md` candidate status precisely.

## Phase 6 - Final Verification and Commit

- [ ] T019 Run strict encoding, LF, final newline, mojibake and `git diff --check` for every changed text file; record evidence in `verification.md`.
- [ ] T020 Commit the locally verified candidate and provide the exact pre-push command in the `docs/maintainers/v0.10.6-release-handoff.md` contract.

## Dependencies and Execution

T001-T006 precede implementation. T013 precedes candidate gate runs, and T007-T012 prepare US1. T014-T016 validate the combined candidate; T017-T020 finish local delivery. US1 identity edits and US2 dependency inspection use separate file sets and may be researched in parallel, but mutations and Cargo operations run sequentially. US3 depends on recorded results. The smallest useful increment is an internally consistent US1 candidate; the authorized slice completes all three stories.

## Requirement Coverage

FR-001: T001,T007,T012; FR-002: T007-T011; FR-003: T012; FR-004: T009,T017,T018; FR-005: T013,T015,T017; FR-006: T014,T016; FR-007: T003,T006,T014-T016,T019; FR-008: T017,T020; FR-009: T005,T014,T017. SC-001: T007-T013,T020; SC-002: T014-T016,T019; SC-003: T012,T017; SC-004: T017-T020.
