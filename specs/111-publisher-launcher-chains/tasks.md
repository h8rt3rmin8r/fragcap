# Tasks: Managed Publisher-Launcher Chains

**Input**: Design documents from `specs/111-publisher-launcher-chains/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/publisher-chain-api.md

**Tests**: Required. S111 uses test-driven implementation, including security, scope, lifecycle, and controlled multi-stage cases.

**Organization**: Tasks are grouped by independently testable user story and run chronologically.

## Phase 1: Setup and Baseline

- [x] T001 Confirm clean branch `codex/111-publisher-launcher-chains`, active spec pointer, issue #307 scope, and unchanged dependency graph
- [x] T002 Run focused baseline tests for `fragcap-targets`, `fragcap`, and `fragcap-cli` managed launch and Deep Capture paths
- [x] T003 Review existing target launch-entry, profile stage, process tree, Capture preparation, S109 routing, journal, and cleanup contracts against the S111 plan

## Phase 2: Foundational Value and Contract Work

- [x] T004 [P] Add failing role-preservation and dedup tests for stored Windows launch entries in `crates/fragcap-targets/src/hint_provider.rs`
- [x] T005 Preserve optional stored launch roles in `crates/fragcap-targets/src/model.rs` and `crates/fragcap-targets/src/hint_provider.rs`
- [x] T006 [P] Add failing direct-versus-publisher classification, structural diagnostic, path containment, and environment-retention tests in `crates/fragcap/src/managed_launch.rs`
- [x] T007 Add immutable `PublisherChainLaunch` and `PublisherStage` values plus complete preparation errors in `crates/fragcap/src/managed_launch.rs`
- [x] T008 Replace the direct-only preparation entry point with the shared `prepare_managed_launch` contract while preserving existing direct behavior in `crates/fragcap/src/managed_launch.rs`

## Phase 3: User Story 1 - Launch an exact cold publisher chain (Priority: P1)

**Goal**: Prepare and execute one exact cold publisher chain through the shared managed-launch and Capture path.

**Independent Test**: A controlled launcher, intermediate, and client declaration synthesizes one profile, starts only the exact root with child-only routing, binds the declared descendants, and identifies one terminal client.

- [x] T009 [US1] Add failing publisher profile synthesis tests for exact stage order, ancestry, lifecycle, and terminal identity in `crates/fragcap-cli/src/commands/target_resolve.rs`
- [x] T010 [US1] Synthesize one validated multi-stage Capture profile from publisher roles in `crates/fragcap-cli/src/commands/target_resolve.rs`
- [x] T011 [US1] Add failing shared Capture managed-launch selection tests in `crates/fragcap-cli/src/assemble.rs`
- [x] T012 [US1] Route all stored managed-launch preparation through `prepare_managed_launch` in `crates/fragcap-cli/src/assemble.rs`
- [x] T013 [US1] Extend managed-launch description and execution handling for publisher roots in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T014 [US1] Add a controlled three-stage process timeline test proving intermediate lifecycle and terminal-client acquisition in `crates/fragcap-cli/tests/cli_capture.rs`
- [x] T015 [US1] Add a Windows child-process probe proving the publisher root inherits the exact scoped environment and is the only process fragcap starts directly in `crates/fragcap/src/managed_launch.rs`

## Phase 4: User Story 2 - Refuse or report uncertain chains truthfully (Priority: P2)

**Goal**: Distinguish cold, warm, game-start-clean warm, escaped, ambiguous, missing, failed, and timed-out publisher cases without effects or guessed ownership.

**Independent Test**: Each controlled negative case produces its stable code and cannot produce final-client routing or compatibility success.

- [x] T016 [US2] Add failing cold-inventory classification tests for launcher, intermediate, and client images in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T017 [US2] Classify exact publisher chains before effects and preserve warm and game-start-clean warm outcomes in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T018 [US2] Add failing supported-cold and warm-refusal policy tests in `crates/fragcap/src/deep_capture/policy.rs`
- [x] T019 [US2] Permit only `PublisherLauncherCold` through the existing compatibility gate and improve stable refusal reasons in `crates/fragcap/src/deep_capture/policy.rs`
- [x] T020 [US2] Add controlled escaped-tree, ambiguous, missing-stage, launch-failure, and deadline outcome tests in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T021 [US2] Add same-named unrelated process and inherited operator proxy security regressions in `crates/fragcap-cli/tests/cli_deep_capture.rs`

## Phase 5: User Story 3 - Preserve auditable evidence and cleanup (Priority: P3)

**Goal**: Reconcile declared stages, observed lifecycle, route verification, loss, and cleanup through existing session authorities.

**Independent Test**: Controlled success, overflow, interruption, and cleanup failure cases retain exact counts and act only on journaled session-owned resources.

- [x] T022 [US3] Add reconciliation assertions for launcher, intermediate, client, and unmatched process roles in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T023 [US3] Prove watcher loss, bounded observation loss, route verification, and cleanup counts remain exact in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T024 [US3] Verify publisher launch uses the existing journal-before-effect and recovery sequence without a new duplicate authority in `crates/fragcap/src/deep_capture/session.rs`

## Phase 6: Documentation and Convergence

- [x] T025 [P] Add S111 feature and dated architecture-decision fragments under `changelog.d/`
- [x] T026 Update S111 shipped boundary, issue mapping, dependency inventory, and incomplete-status language in `AGENTS.md`, `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md`
- [x] T027 Run spec-kit analysis, remediate every finding, and rerun until clean
- [x] T028 Run spec-kit convergence after implementation, append and complete any missing tasks, and rerun until converged
- [x] T029 Verify issue #307 closure mapping while leaving #308 through #334 open as applicable
- [x] T030 Run formatting, clippy, locked workspace tests, xtask CI, MSRV, dependency, platform, encoding, mojibake, evidence drift, and final diff checks

## Dependencies & Execution Order

- Phase 1 establishes a clean, green baseline.
- Phase 2 is foundational and blocks all user stories.
- User Story 1 establishes the shared publisher value and Capture execution path.
- User Story 2 depends on User Story 1's exact declaration and handles all refusal boundaries.
- User Story 3 depends on the launched and refused outcomes and proves existing evidence authorities remain complete.
- Documentation, analyze, convergence, and full verification follow implementation.

## Parallel Opportunities

- T004 and T006 touch different crates and can start together after baseline review.
- T025 can proceed after behavior and decisions stabilize while test completion continues in separate files.
- Tests in different integration targets can run concurrently, but edits to shared source files remain sequential.

## Implementation Strategy

Implement test-first in dependency order. Preserve direct and Steam behavior before adding publisher selection. Make the shared Capture path green before enabling `PublisherLauncherCold` in Deep Capture policy. Treat every negative classification as a first-class deliverable, then prove evidence and cleanup reconciliation. No supported outcome is complete until the final socket owner is both the declared terminal client and correlated to packet truth.
