# Tasks: S161 Interactive CLI Input Ownership

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [interactive-input.md](contracts/interactive-input.md)

**Tests**: Required for the confirmed active-release defect and prompt effect boundary.

## Phase 1: Analyze and baseline

- [x] T001 Run the Spec-Kit analyze gate against spec, plan, tasks, and constitution; resolve every material finding before implementation.
- [x] T002 Confirm S161 branch, issue #437, clean base, and affected source paths in `crates/fragcap-cli/src/lib.rs`, `doctor/fix.rs`, `commands/targets.rs`, and `commands/deep_capture.rs`.

## Phase 2: Failing regressions

- [x] T003 [US1] Add a bounded regression for production stdin-owner lifetime and Doctor negative, positive, multi-action, and I/O-failure outcomes in `crates/fragcap-cli/src/lib.rs` and `crates/fragcap-cli/src/doctor/fix.rs`; observe a pre-fix failure.
- [x] T004 [US2] Add controlled target socket-holder and warm-restart prompt regressions, including failure paths and authorization preservation, in their owning CLI modules; observe a pre-fix failure where feasible.

## Phase 3: User Story 1 - Doctor action responses (P1)

- [x] T005 [US1] Replace command-wide stdin guard with per-authorization-read locking in `crates/fragcap-cli/src/lib.rs`, preserving bounded exact plan input.
- [x] T006 [US1] Propagate Doctor prompt write, flush, EOF, and read failures through `ActionConfirm`, `drive_actions`, and `run_fix` in `crates/fragcap-cli/src/doctor/fix.rs`; keep affirmative, default-No, action ordering, and no-effect outcomes.
- [x] T007 [US1] Run focused Doctor and stdin-owner tests; confirm bounded completion, `skipped` only on completed No, and zero later performer calls after error.

## Phase 4: User Story 2 - Other CLI responses (P2)

- [x] T008 [US2] Flush and validate the target socket-holder question in `crates/fragcap-cli/src/commands/targets.rs`; retain EOF-to-unsure and propagate I/O errors.
- [x] T009 [US2] Distinguish warm-restart EOF/read errors from completed decline in `crates/fragcap-cli/src/commands/deep_capture.rs`; retain existing consent and refusal semantics.
- [x] T010 [US2] Run focused target, warm-restart, Deep Capture, and calibration authorization regressions.

## Phase 5: User Story 3 - Evidence and documentation (P3)

- [x] T011 [US3] Update `docs/fragcap-specification.md` with the corrected prompt failure contract, add S161 ordering in `docs/plans/README.md`, and record the user-visible correction in `changelog.d/`.
- [x] T012 [US3] Check that slice and PR wording separates the confirmed stdin deadlock from the unproven Print Screen cause; report any focused comparison not performed as unverified.

## Phase 6: Verification and delivery

- [x] T013 Run a controlled interactive Doctor decline with the new executable through the integrated headless terminal, when the environment offers an action; otherwise report this field comparison as unverified rather than a CI pass.
- [x] T014 Run `cargo xtask ci` in the foreground, fix scoped failures, inspect the diff and UTF-8/LF hygiene, and mark verified tasks complete.

## Dependencies and execution order

T001 is the blocking pre-implementation gate. T002 precedes tests. T003 and T004 precede production edits T005, T006, T008, and T009. T005 precedes the three prompt fixes. T007 and T010 precede documentation claims. T013 and T014 follow implementation. Commit, push, and review occur after the implementation task list is complete.

## Done when

Every scoped prompt has a verified response or honest failure, authorization semantics remain exact, repository-controlled gates pass, the issue has a reviewable PR, and no unsupported Print Screen fix is claimed.
