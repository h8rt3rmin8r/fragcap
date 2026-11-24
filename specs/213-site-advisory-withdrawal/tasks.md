# Tasks: Withdrawn Site Advisory Cleanup

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [site audit contract](contracts/site-audit-boundary.md), and [quickstart.md](quickstart.md).

**Organization**: Two independently testable user stories, followed by one final gate. Tests for the audit guard precede its implementation.

## Phase 1: Setup

- [x] T001 Confirm [issue #630](https://github.com/h8rt3rmin8r/fragcap/issues/630), the official withdrawn [GHSA record](https://github.com/advisories/GHSA-fc5q-39xc-vjpf), and the exact site-exception inventory in `research.md`.
- [x] T002 Create and validate `specs/213-site-advisory-withdrawal/spec.md`, `checklists/requirements.md`, and the ignored `.specify/feature.json` pointer on the clean S213 branch.

## Phase 2: Foundation

- [x] T003 Complete `specs/213-site-advisory-withdrawal/checklists/audit-boundary.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/site-audit-boundary.md`, and `quickstart.md`; run the blocking pre-implementation analysis and resolve its findings.

## Phase 3: User Story 1 - Trust the site audit result (P1)

**Goal**: Remove the withdrawn site's exception and prove the full audit still blocks known workspace bypass settings.

**Independent Test**: Controlled guard fixtures pass and fail as declared, and the pinned full site audit succeeds with no ignore.

- [x] T004 [US1] Add LF/CRLF and indentless `packages` baselines (including a harmless comment and nested override), four bypass-config negative tests, and unsupported root YAML, standalone-sequence, and bare-CR canaries in `xtask/src/supply_chain.rs`; observe the new no-exception contract fail before replacing the old validator.
- [x] T005 [P] [US1] Remove the CVE ignore from `site/pnpm-workspace.yaml` and delete `supply-chain/site-audit-exception-v1.json` without changing `site/pnpm-lock.yaml`.
- [x] T006 [US1] Replace the exception-specific offline validator in `xtask/src/supply_chain.rs` with a small site workspace guard for top-level `auditConfig` and `audit` keys, unsupported root YAML syntax, and bare CR, retaining unrelated Rust policy checks.
- [x] T007 [US1] Replace the exception validator step in `.github/workflows/audit.yml` with the matching hosted workspace guard before the unchanged unconditional `pnpm audit --audit-level low` step.
- [x] T008 [US1] Update the workflow-wiring assertions in `xtask/src/supply_chain.rs` to require guard order and syntax, the frozen-lockfile install, the unfiltered full audit command, and the absence of the retired exception fields.
- [x] T009 [US1] Run focused xtask tests, `cargo xtask supply-chain`, pinned `pnpm install --frozen-lockfile`, and unfiltered `pnpm audit --audit-level low`; record exact results in the S213 verification handoff.

## Phase 4: User Story 2 - Read an accurate current record (P2)

**Goal**: Current guidance names the withdrawal while the v0.11.2 history remains true.

**Independent Test**: Read the current and historical records against the dated GHSA source and the final audit behavior.

- [x] T010 [P] [US2] Correct the current site and Rust audit distinction in `docs/maintainers/supply-chain.md`, linking the official withdrawal without rewriting the v0.11.2 publication claim.
- [x] T011 [P] [US2] Add the chronological S213 entry and dated correction to `docs/plans/README.md`; preserve the original v0.11.2 release record as a historical observation.
- [x] T012 [US2] Align `docs/fragcap-specification.md` sections 24.2, 24.3, and 27.3 with the unfiltered site audit and S213 slice, including the next draft revision record.
- [x] T013 [US2] Verify `docs/maintainers/v0.11.2-release-handoff.md` remains unchanged and that dated current guidance distinguishes its publication-time exception from the later withdrawal.
- [x] T014 [P] [US2] Add `changelog.d/S213-site-advisory-withdrawal.changed.md` for the current audit behavior and `changelog.d/S213-site-advisory-withdrawal.decisions.md` for the pinned workflow change, each with an exact `spec-impact` first line.

## Phase 5: Polish and cross-cutting gates

- [x] T015 [P] Run the pinned site build and static export from `site/package.json`; inspect generated output cleanup under the repository test contract.
- [x] T016 Run `cargo xtask ci` to completion, check Rust formatting and repository documentation rules, and resolve any applicable failure.
- [x] T017 Re-run the Spec Kit analysis of `specs/213-site-advisory-withdrawal/spec.md`, `plan.md`, and `tasks.md` after implementation; resolve every critical or high mismatch before PR handoff.
- [x] T018 Inspect the final diff against `contracts/site-audit-boundary.md`: no site lock churn without separate proof, no altered #640 Rust exception, no private identifier, no mojibake, and no stale current GHSA claim.
- [ ] T019 Commit, push, and publish the S213 PR with `Closes #630`; resolve all comments within the owner's two-round Codex cap, wait for green final-head CI, and hand merge authority to the owner.

## Dependencies and execution order

T001-T003 establish the scope and contract. T004 must be observed red before T006-T008 make the new guard pass. T005 and T010-T011 can run in parallel with guard implementation because they touch separate files. T007 follows the contract from T006. T008 and T009 complete US1. T010-T014 complete US2 independently of the site build, but their current claims must match the final US1 implementation. T015-T018 are final acceptance gates before T019.

**Parallel opportunities**: T005, T010, and T011 touch separate files. T014 can proceed after the specification section choices are settled. No task permits a second advisory ignore as a shortcut.
