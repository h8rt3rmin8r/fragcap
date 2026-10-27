# Tasks: S181 Dependabot reconciliation and green-main recovery

## Phase 1 - Setup

- [x] T001 Establish approved branch and spec in `specs/181-dependabot-main-reconciliation/spec.md` (FR-001/FR-008).
- [x] T002 Complete clarification, checklists, research, and design in `specs/181-dependabot-main-reconciliation/` (FR-001 through FR-009).

## Phase 2 - Foundation

- [x] T003 Complete blocking cross-artifact analysis and record it in `specs/181-dependabot-main-reconciliation/analysis.md` (FR-001 through FR-009).
- [x] T004 Establish failing workflow/advisory baselines and add focused workflow regression cases in `xtask/src/supply_chain.rs` (FR-002/FR-004/FR-005/FR-006).

## Phase 3 - US1: Resolve findings

- [x] T005 [US1] Incorporate #510's Zod patch and advance DOMPurify through package-manager-generated `site/package.json`, `site/pnpm-workspace.yaml`, and `site/pnpm-lock.yaml` (FR-002/FR-003).
- [x] T006 [US1] Patch affected resolutions in `fuzz/Cargo.lock`, `performance/native-proxy/Cargo.lock`, and four `spikes/**/Cargo.lock` files; inspect fresh audits of all seven graphs (FR-001/FR-002/FR-007).
- [x] T007 [US1] Add complete site and isolated Rust advisory checks in `.github/workflows/audit.yml`, with regression coverage in `xtask/src/supply_chain.rs` (FR-006/FR-007).

## Phase 4 - US2: Clean environment parity

- [x] T008 [US2] Add locked dependency preparation and unconditional root contracts in `.github/workflows/docs.yml` (FR-004/FR-005).
- [x] T009 [US2] Add manual isolated Cargo home/target cold-cache qualification in `.github/workflows/docs.yml` and guard failure propagation in `xtask/src/supply_chain.rs` (FR-004/FR-005).
- [x] T010 [US2] Document complete audit/preparation procedure in `docs/maintainers/supply-chain.md`, `docs/fragcap-specification.md`, `docs/plans/README.md`, and dated `changelog.d/S181.*.md` fragments (FR-004 through FR-007).

## Phase 5 - US3: Deliver green main

- [x] T011 [US3] Complete local repository and affected-project qualification; record actual results in `specs/181-dependabot-main-reconciliation/verification.md` (FR-007/FR-008).
- [ ] T012 [US3] Publish consolidated PR, resolve exact-source checks and review findings, and integrate under owner delegation; preserve evidence in #511 (FR-008).
- [ ] T013 [US3] Verify terminal applicable final-main checks, fresh audit/links, cold-cache docs dispatch, and Pages deployment; record evidence in #511 (FR-004/FR-008).
- [ ] T014 [US3] Reconcile bot reevaluation and zero open alerts/PRs, close superseded #510 and completed #511 with final-main evidence (FR-001/FR-003/FR-009).

## Final Phase - Reconciliation

- [ ] T015 Recheck encoding, final diff, task/evidence truth, final main identity, and clean checkout in `specs/181-dependabot-main-reconciliation/verification.md` (FR-007 through FR-009).

## Dependencies and implementation strategy

T001-T003 precede implementation. T004 establishes regressions before T005-T010 corrections. US1 dependency resolution and US2 workflow preparation can be researched independently; shared workflow and supply-chain files are edited sequentially. T011 precedes publication. T012 precedes final-main/scanner evidence T013-T014. T015 closes the audit trail. Each story has independent evidence described in [spec.md](spec.md), and delivery requires all three stories together.

This source snapshot records implementation through local qualification. Delivery operations T012-T015 are subsequently executed and reconciled with checked completion receipts in issue #511, the authoritative live planning record, so source publication cannot claim its own later merge or scanner state.

Parallel research examples: inspect site resolution while inspecting documentation discovery; inspect historical Cargo graphs while reading workflow guards. Qualification shares tool caches and is sequenced where resource or lock contention would obscure results.
