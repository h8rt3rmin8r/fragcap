# Tasks: Warm-To-Cold Restart

**Input**: Design documents from `specs/113-warm-to-cold-restart/`

**Tests**: TDD is required for policy, consent, timeout, re-preparation, events, and regressions.

## Phase 1: Contracts And Foundations

- [x] T001 Validate requirement and security checklists
- [x] T002 Add failing pure warm-case mapping, image-set, deadline, and outcome tests in `crates/fragcap/src/deep_capture/restart.rs`
- [x] T003 Implement immutable warm restart policy values in `crates/fragcap/src/deep_capture/restart.rs` and export them from `mod.rs`
- [x] T004 Run focused facade tests

## Phase 2: User Story 1 - Reach Cold Safely

- [x] T005 [US1] Add `--restart-warm` argument contract and conflict tests in CLI sources and integration tests
- [x] T006 [US1] Refactor launch detection so direct image presence returns a typed warm case while default policy still refuses it
- [x] T007 [US1] Add failing scripted inventory tests for direct, Steam, and publisher warm-to-cold transitions
- [x] T008 [US1] Implement plan presentation, first confirmation, bounded no-handle waiting, and complete-image cold detection
- [x] T009 [US1] Run focused CLI tests

## Phase 3: User Story 2 - Explicit Non-Success

- [x] T010 [US2] Add failing decline, timeout, inventory error, partial closure, changed-state, and re-preparation tests
- [x] T011 [US2] Implement distinct pre-effect outcomes and human and structured events
- [x] T012 [US2] Prove no process-control primitive or Deep Capture effect exists on every negative path

## Phase 4: User Story 3 - Reprepare And Reauthorize

- [x] T013 [US3] Add failing fresh-resolution and second-authorization ordering tests
- [x] T014 [US3] Re-run ordinary preflight after cold detection and bind second authorization to its prepared plan
- [x] T015 [US3] Preserve existing session and cleanup results after restart succeeds
- [x] T016 [US3] Run controlled CLI integration coverage

## Phase 5: Documentation And Verification

- [x] T017 Update master specification, outline, plan index, AGENTS, and changelog fragments
- [x] T018 Mark tasks complete and rerun cross-artifact analysis
- [x] T019 Run focused tests, `cargo xtask ci`, `cargo xtask msrv`, and `cargo xtask neutral`
- [x] T020 Inspect diff for UTF-8, mojibake, prohibited punctuation, dependency drift, scope, and prohibited process control

## Dependencies And Order

Policy values precede CLI orchestration. The safe transition precedes negative outcomes, which precede session authorization. Documentation and full verification follow implementation. Tests precede implementation inside every phase.
