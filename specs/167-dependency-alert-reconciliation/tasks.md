# Tasks: S167 dependency alert reconciliation

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [quickstart.md](quickstart.md).

## Phase 1: Setup

- [x] T001 Confirm branch, active feature pointer, eight alert records, and PR #455 diff in `specs/167-dependency-alert-reconciliation/research.md`.
- [x] T002 Run the Spec Kit analysis gate against `specs/167-dependency-alert-reconciliation/` and resolve blocking findings before lockfile changes.

## Phase 2: User Story 1 - Remove vulnerable versions (Priority: P1)

**Goal**: Every affected tracked dependency inventory resolves a fixed version.

**Independent test**: Parse all five affected lockfiles and prove zero vulnerable resolved instances against the eight advisory thresholds.

- [x] T003 [US1] Record the failing version baseline from `spikes/native-proxy/Cargo.lock`, `spikes/native-proxy/audit/Cargo.lock`, `spikes/http-mitm-proxy/Cargo.lock`, `spikes/http-mitm-proxy/audit/Cargo.lock`, and `site/pnpm-lock.yaml`.
- [x] T004 [P] [US1] Update rustls in `spikes/native-proxy/Cargo.lock` and `spikes/native-proxy/audit/Cargo.lock` using the exact fixed package metadata.
- [x] T005 [P] [US1] Update rustls in `spikes/http-mitm-proxy/Cargo.lock` and `spikes/http-mitm-proxy/audit/Cargo.lock` using the exact fixed package metadata.
- [x] T006 [US1] Resolve every vulnerable source-map-js, KaTeX, and DOMPurify instance in `site/pnpm-lock.yaml`, changing `site/package.json` only as needed for reproducible resolution.
- [x] T007 [US1] Recheck all eight advisory thresholds and confirm unchanged root `Cargo.lock` and product version files.

## Phase 3: User Story 2 - Preserve usability (Priority: P2)

**Goal**: The site and historical spike projects remain usable from their committed dependency inventories.

**Independent test**: Frozen site install and build, existing site tests, and applicable locked spike checks complete.

- [x] T008 [US2] Run frozen installation and existing site test and build scripts from `site/package.json`.
- [x] T009 [US2] Run applicable locked checks against both `spikes/native-proxy/Cargo.lock` and `spikes/http-mitm-proxy/Cargo.lock`, including their audit subprojects.
- [x] T010 [US2] Run repository CI parity and documentation checks, then record exact evidence in `specs/167-dependency-alert-reconciliation/quickstart.md`.

## Phase 4: Review and handoff

- [x] T011 Record the dependency maintenance change and any lasting decision in `changelog.d/S167-dependency-alerts.security.md` and the appropriate decisions fragment.
- [ ] T012 Review the complete diff for scope and text hygiene, then commit, push, and open one S167 pull request.
- [ ] T013 Resolve every applicable hosted CI failure and review finding on the S167 PR, with no more than two Codex review rounds.
- [ ] T014 Reconcile overlapping Dependabot PR #455, confirm final-head green checks, and request owner review and merge.

## Dependencies and execution order

T001 and T002 precede all implementation. T003 precedes T004 through T006. T007 follows both spike and site updates. T008 through T010 follow T007. T011 through T014 follow verification. T004 and T005 can proceed independently because they touch distinct spike projects.

## Implementation strategy

The smallest useful increment is US1: remove the vulnerable versions from all five inventories and prove exact thresholds. US2 establishes that the corrected inventories still build and pass existing checks. The final phase packages both in one owner-reviewed PR.
