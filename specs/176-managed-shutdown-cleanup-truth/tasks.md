# Tasks: Managed Shutdown and Cleanup Truth

**Branch**: `codex/s176-managed-shutdown-cleanup-truth`

## Phase 1: Setup and governance

- [x] T001 Amend `.specify/memory/constitution.md`, `docs/fragcap-specification.md`, and `AGENTS.md` before S176 specification to name exact, pre-authorized Deep Capture process control.
- [x] T002 Create `specs/176-managed-shutdown-cleanup-truth/spec.md`, clarifications, checklists, `plan.md`, `research.md`, `data-model.md`, `contracts/shutdown-and-history.md`, and `quickstart.md`.
- [x] T003 Run the Spec-Kit analyze gate against `spec.md`, `plan.md`, `tasks.md`, constitution, and intended source behavior; resolve all findings before implementation.

## Phase 2: Historical cleanup truth (US3)

- [x] T004 [US3] Add readable-journal plus failed-manifest and post-timeout-exit regressions in Doctor inventory and CLI tests.
- [x] T005 [US3] Parse bounded terminal manifest and cleanup failed-resource details in `crates/fragcap-cli/src/doctor/probe.rs`.
- [x] T006 [US3] Separate historical outcome from present journal health in `crates/fragcap-cli/src/doctor/residue.rs` and render consistent human/JSON findings in `crates/fragcap-cli/src/doctor/checks.rs`.

## Phase 3: Exact owned shutdown (US1 and US2)

- [x] T007 [US2] Add controlled identity and no-unowned-control tests in the focused Windows shutdown adapter.
- [x] T008 [US2] Retain exact minimal-rights root and descendant ownership proof in `crates/fragcap/src/managed_launch.rs`, `crates/fragcap/src/managed_shutdown.rs`, and `crates/fragcap-cli/src/orchestrator.rs` without serializing handles.
- [x] T009 [US1] Bind automatic shutdown scope, deadlines, and unsaved-progress warning into the pre-effect plan in `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T010 [US1] Implement bounded process-bound graceful, force, and verified-exit route release in `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs` with explicit refusal and partial-cleanup outcomes.
- [x] T011 [US1] Record attempted close, grace, force, refusal, exit, and proxy order in existing lifecycle evidence and controlled tests.
- [x] T011a [US1] Retain the original ETW watcher through route-owner release, reconcile shutdown-time descendants and the final queued events, and cover a helper started during graceful close.
- [x] T012 [US2] Cover direct, Steam, publisher, unrelated Steam title, PID reuse, identity mismatch, ancestry loss, no window, force failure, and interruption in controlled Windows and existing session-order tests.

## Phase 4: Polish and verification

- [x] T013 Add a release fragment and dated architecture decision under `changelog.d/`, then update `docs/plans/README.md` with S176 ordering and scope.
- [x] T014 Run `cargo xtask ci` and focused Windows tests in the foreground; resolve failures and check UTF-8/LF, mojibake, and diff hygiene.
- [x] T015 Record real-title calibration as unverified: `AGENTS.md` assigns it to the operator against a published release, and this candidate branch is unreleased.
