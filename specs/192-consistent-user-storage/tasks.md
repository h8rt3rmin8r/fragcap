# Tasks: Consistent User Storage

**Slice**: S192 | **Branch**: `codex/s192-consistent-user-storage` | **Issue**: #556
**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [storage contract](contracts/storage-location.md)

## Phase 1: Storage identity and selection

- [x] T001 Verify S192 identity and run Spec-Kit requirement analysis over the completed artifacts.
- [x] T002 [US1] Write focused non-ASCII, absent-path, and explicit-override cases in `crates/fragcap-cli/src/paths.rs` and CLI integration tests; use the controlled package script for same-account independent-process readback with altered AppData variables.
- [x] T003 [US1] Implement exact current-user profile-root selection and shared default path functions in `crates/fragcap-cli/src/paths.rs`.
- [x] T004 [US1] Reconcile target, Capture, Deep Capture, calibration, Doctor, and session-registry default-path consumers without changing explicit override precedence.

## Phase 2: Legacy transition and retained history

- [x] T005 [US2] Add synthetic single-root, WAL, active, conflicting, repeated-start, and failed-transition regressions for migration.
- [x] T006 [US2] Implement bounded exact legacy-root inventory, SQLite-consistent staged copy, durable transition record, and non-destructive conflict handling.
- [x] T007 [US2] Retain and index old session bundles and registries so read-only history and Doctor recovery discover unresolved obligations at their original paths.

## Phase 3: Effective-path reports and cleanup

- [x] T008 [US3] Add selected/effective path and redirection cases for Doctor human and JSON output and terminal session reports.
- [x] T009 [US3] Implement handle-derived effective destination reporting for stores, session root, registry, and retained bundle, with distinct absent and unavailable states.
- [x] T010 [US3] Align retention, fresh-start preview/execution, all-users inventory, and MSI current-user adapter with exact new and retained legacy roots.

## Phase 4: Controlled package and documentation

- [x] T011 Add an independent-process same-account package row with altered AppData variables, opened-file readback, and no private game data in `scripts/Test-PackageCertification.ps1`. Record actual packaged-host behavior for post-release field verification.
- [x] T012 Reconcile default path, transition, Doctor schema, installed-test guidance, master specification, chronological plan, and user-visible changelog.
- [x] T013 Run focused tests, full applicable workspace checks, the static package contract check, PowerShell compliance, mojibake scan, and `git diff --check`; resolve findings.
- [x] T014 Run Spec-Kit convergence against the implemented code and close any remaining buildable tasks before commit.

## Dependencies

T001 precedes implementation. T002 precedes T003 and T004. T005 precedes T006 and T007. T008 precedes T009. T011 follows the complete runtime path. T012 and T013 follow all product changes; T014 checks the final implementation against the written intent.
