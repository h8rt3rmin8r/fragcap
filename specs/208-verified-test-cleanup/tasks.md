# Tasks: Verified Test Cleanup

**Input**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), and [test-cleanup.md](contracts/test-cleanup.md)

**Prerequisites**: S208 branch at merged S207 main, #605 and #606 open

**Tests**: Required for fixture ownership, failure reporting, bounded residue, and the repository gate.

**Organization**: Stories follow the independent fixture, aggregate run, and guidance outcomes in the specification.

## Phase 1: Setup

- [x] T001 Audit temp, AppData, repository scratch, package, and child-output writers and record proven versus separately owned cases in `specs/208-verified-test-cleanup/research.md`.
- [x] T002 Establish the exact fixture and aggregate-run contracts in `specs/208-verified-test-cleanup/contracts/test-cleanup.md` before implementation.

## Phase 2: Foundation

- [x] T003 Add controlled lifecycle regressions for success, assertion panic, cleanup failure, concurrent roots, and exact-path refusal in `crates/fragcap/tests/deep_capture_session.rs` and `xtask/src/main.rs`; observe pre-fix failures.

## Phase 3: User Story 1 - Automated fixtures leave no owned residue (P1)

**Goal**: Replace demonstrated persistent writers with unique, fallibly closed fixtures.

**Independent Test**: Run affected Deep Capture, skills, CLI bundle, and technology tests under an isolated effective temp root, then verify zero newly owned residue.

- [x] T004 [US1] Keep Deep Capture session bundle roots and residual owner markers alive through assertions and close them on success and panic in `crates/fragcap/tests/deep_capture_session.rs`.
- [x] T005 [US1] Replace the fixed skills test directory with a unique owned and verified fixture in `xtask/src/skills.rs` and add only the needed dependency in `xtask/Cargo.toml`.
- [x] T006 [US1] Give shared CLI tests a locked, multi-call owned environment in `crates/fragcap-cli/tests/common/mod.rs` and adapt persistent call sites in `crates/fragcap-cli/tests/cli_bundle.rs`, `cli_targets.rs`, and `cli_steam.rs`.
- [x] T007 [US1] Replace raw technology-command test trees with fallibly closed unique fixtures in `crates/fragcap-cli/src/commands/technologies.rs`.
- [x] T008 [US1] Repair any additional source-proven success-path leaks from T001 in their exact test writers, without rewriting already owned fixtures.

## Phase 4: User Story 2 - Run-level cleanup is a gate (P2)

**Goal**: Report exact effective roots, fail on new residue, and preserve uncertain native evidence.

**Independent Test**: A controlled successful child leaves no run root; a deliberately leaking or cleanup-failing child makes the wrapper fail with the exact path while an unrelated same-prefix path remains untouched.

- [x] T009 [US2] Implement unique run-root creation, exact child environment, bounded inventory, interrupted-owner marker, and fallible close in `xtask/src/main.rs`.
- [x] T010 [US2] Add `test-clean` invocation to the ordinary `ci` path in `xtask/src/main.rs` and migrate hosted test steps in `.github/workflows/ci.yml`, `.github/workflows/platform.yml`, and `.github/workflows/windows-integration.yml`.
- [x] T011 [US2] Add controlled runner tests for success, leak, cleanup failure, interruption record, concurrent invocation, redirected temp root, same-prefix refusal, and unresolved native evidence preservation in `xtask/src/main.rs`.
- [x] T012 [US2] Run a Windows before/after inventory using the exact S208 run root, record effective canonical paths and any residuals in the PR evidence, and leave historic and native material untouched.

## Phase 5: User Story 3 - Guidance agrees (P3)

**Goal**: Make cleanup an explicit completion obligation across all application-test classes.

**Independent Test**: The constitution, agent, contributor, reviewer, and testing guidance agree on pass, failure, interruption, and unresolved-native-session handling and pass documentation checks.

- [x] T013 [US3] Amend the versioned constitution and Sync Impact Report in `.specify/memory/constitution.md` for #605.
- [x] T014 [US3] Reconcile `AGENTS.md`, `CONTRIBUTING.md`, and applicable review guidance with exact effective-root and residual handoff in the same files, adding any new public term to `docs/glossary/rust-and-tooling.md` first.
- [x] T015 [US3] Reconcile testing strategy and ordered slice record in `docs/fragcap-specification.md` and `docs/plans/README.md` without rewriting historical test claims.

## Phase 6: Polish and verification

- [x] T016 Add user-visible and dated process decisions in `changelog.d/S208-verified-test-cleanup.feature.md` and `changelog.d/S208-verified-test-cleanup.decisions.md`.
- [x] T017 Mark completed S208 tasks in `specs/208-verified-test-cleanup/tasks.md` and run the full repository format, lint, docs, wrapper, dependency, and test gates from `AGENTS.md`.
- [x] T018 Review changed files for UTF-8 without BOM, LF, final newline, mojibake, privacy leaks, and exact issue acceptance, then commit only S208 files.

## Dependencies and execution order

T001 and T002 precede the red tests in T003. T004 through T008 complete the fixture story before the aggregate runner asserts zero residue. T009 through T012 establish the run gate. T013 through T015 align governance after the executable contract is fixed. T016 through T018 finalize evidence and handoff.

## Parallel opportunities

The independently owned source files in T005 and T007 can be changed separately after T003. The governance review in T013 can be drafted while fixture implementation is underway, but constitutional text is finalized against the implemented contract.

## Implementation strategy

Finish the P1 fixture correction first, validate it in isolation, then add the P2 run gate and P3 guidance. A passing test assertion is never substituted for cleanup verification.
