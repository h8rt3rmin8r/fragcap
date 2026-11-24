# Tasks: Withdrawn Site Advisory Cleanup

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [site audit contract](contracts/site-audit-boundary.md), and [quickstart.md](quickstart.md).

**Organization**: Two independently testable user stories, initial publication, then a review-driven security boundary correction and final gate. Tests for each audit guard expansion precede its implementation.

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
- [x] T019 Commit, push, and publish [S213 PR #642](https://github.com/h8rt3rmin8r/fragcap/pull/642) with `Closes #630`; retain owner merge authority.

## Phase 6: Review-driven audit-source correction

**Goal**: Resolve the security review finding that pinned pnpm 10.26.0 can read audit ignores from the root manifest even when the workspace YAML is clean, and close adjacent repository-controlled audit input paths.

**Independent Test**: A parsed manifest CVE or GHSA ignore, either repository `.npmrc`, and a direct audit-step environment override fail before the unchanged pinned audit command. A step HOME/XDG_CONFIG_HOME redirect cannot reach a committed global config because the audit resets those paths to a fresh temporary directory. The current manifest and no-`.npmrc` baseline pass.

- [x] T020 [US1] Add controlled negative fixtures for `site/package.json#pnpm.auditConfig.ignoreCves`, `.ignoreGhsas`, root `.npmrc`, `site/.npmrc`, and audit-step-only pnpm/npm environment overrides. Add positive fixtures for benign manifest siblings and the current no-`.npmrc` baseline; observe the new contract fail before implementation.
- [x] T021 [US1] Extend the offline `xtask/src/supply_chain.rs` gate to parse `site/package.json`, reject root `pnpm.auditConfig`, and refuse repository `.npmrc` sources while retaining the workspace YAML guard and unrelated Rust policy.
- [x] T022 [US1] Extend `.github/workflows/audit.yml` with the matching manifest and `.npmrc` checks and an environment assertion in the same hosted step as `pnpm audit --audit-level low`; keep Node/pnpm pins, frozen install, and exact audit command unchanged. Update workflow-wiring checks to fail a step-scoped environment bypass.
- [x] T023 [US1] In the hosted audit step, create one fresh temporary directory and set HOME and XDG_CONFIG_HOME to it after refusing direct configuration variables and before invoking the unchanged pinned audit command. Add a workflow-wiring mutation that fails if this isolation is removed or reordered.
- [x] T024 Re-run affected focused tests, `cargo xtask supply-chain`, and `cargo xtask ci` after the fresh-home change; retain pinned frozen install, full site audit, and static export results from the same unchanged site graph; complete Spec Kit cross-artifact analysis. Update `verification.md` with the distinct evidence tiers and confirm that local green checks do not replace final-head hosted CI.
- [ ] T025 Push the corrected branch, answer the security review finding and any remaining comments without exceeding the owner's review-round cap, wait for terminal green final-head CI and satisfied reviews, then flag the owner for the final merge ritual.

## Dependencies and execution order

T001-T003 establish the original scope and contract. T004 was observed red before T006-T008 made the workspace guard pass. T005 and T010-T011 ran independently of guard implementation. T008-T009 completed the initial US1 implementation; T010-T014 completed US2; T015-T018 qualified the initial PR candidate; T019 published PR #642. The review finding then expanded US1. T020 established failing fixtures before T021-T022 closed the direct additional sources. T023 closes the indirect hosted global-config path. T024 requalifies the changed candidate, and T025 is the final review and CI handoff.

**Parallel opportunities**: T005, T010, and T011 touched separate files. T021 and T022 have distinct implementation files after the shared T020 contract is fixed. No task permits a second advisory ignore as a shortcut.
