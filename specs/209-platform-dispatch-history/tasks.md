# Tasks: Platform Dispatch and Advisory Capture History

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [dispatch-and-history.md](contracts/dispatch-and-history.md)

**Prerequisites**: S209 branch at merged S208 main; #610 and #616 open.

**Tests**: Required for identity, authorization, duplicate-launch prevention, route scope, store migration, retention, comparison, and cleanup.

## Phase 1: Setup and decisions

- [x] T001 Confirm the current association, managed launch, process session, authorization, history, and store boundaries in the files named in plan.md; record deviations in research.md.
- [x] T002 Complete spec quality and ownership-history checklists before implementation.

## Phase 2: Foundation, red tests

- [x] T003 Cover cold and warm Steam, unassociated direct, unavailable association refusal, already-running game, publisher handoff, PID reuse, and unrelated same-name outcomes in crates/fragcap/src/session.rs, managed_launch.rs, and crates/fragcap-cli/src/orchestrator.rs.
- [x] T004 Check authorization names one exact direct alternate and its condition, uncertain delivery never triggers a second dispatch, and the game-only direct route remains exact in crates/fragcap-cli/src/commands/capture.rs, orchestrator.rs, and crates/fragcap/src/deep_capture/direct_route.rs.
- [x] T005 Cover version 16 to 17 migration, private history round-trip, target cascade, bounded pruning, append failure, and comparable/incomparable drift in crates/fragcap-targets/src/store.rs and history.rs.

## Phase 3: User Story 1, associated launch

**Goal**: Associated platforms launch without a routine cold-state prerequisite while exact final-game scope remains intact.

**Independent Test**: Run cold and warm platform plus direct target fixtures with one launch each and exact route owner proof.

- [x] T006 [US1] Add query-only warm platform identity and immutable launch intent to crates/fragcap/src/managed_launch.rs using the existing reviewed process-query rights, with no shutdown capability.
- [x] T007 [US1] Bind the warm root and post-dispatch final-game generation without treating a snapshot parent as a creation event in crates/fragcap/src/session.rs.
- [x] T008 [US1] Prepare cold or warm Steam association and direct unassociated launch through the existing target path in crates/fragcap-cli/src/commands/capture.rs and deep_capture.rs.
- [x] T009 [US1] Perform one dispatch only after route and watcher readiness in crates/fragcap-cli/src/orchestrator.rs; keep warm or unrelated processes outside shutdown_scope.
- [x] T010 [US1] Remove the warm Steam compatibility refusal only for the proven exact-game direct route in crates/fragcap/src/deep_capture/policy.rs and align CLI launch-case selection in deep_capture.rs.

## Phase 4: User Story 2, bounded fallback

**Goal**: A wrong association can use one preauthorized alternate only when no first-game launch is possible.

**Independent Test**: Run missing platform, failed dispatch, uncertain delivery, and existing game cases; assert at most one alternate and unchanged route effects.

- [x] T011 [US2] Put first and alternate launch methods, conditions, and ownership effects in the pre-effect plan in crates/fragcap-cli/src/commands/deep_capture.rs.
- [x] T012 [US2] Implement the one-shot fallback gate and exact failure reasons in crates/fragcap-cli/src/orchestrator.rs and shared managed launch code.
- [x] T013 [US2] Reconcile human, JSON, process trace, route journal, and cleanup outcomes for the first and alternate attempts in crates/fragcap-cli/src/commands/deep_capture.rs.

## Phase 5: User Story 3, advisory history

**Goal**: Every attempt retains bounded chronological facts without affecting authorization.

**Independent Test**: First, repeat, failure, updated target, scope change, route change, rollback, pruning, migration, and append failure read back correctly.

- [x] T014 [US3] Add the version 17 table and transactionally bounded append/read contract in crates/fragcap-targets/src/schema.rs and store.rs.
- [x] T015 [US3] Implement typed history validation and advisory comparison in crates/fragcap-targets/src/compatibility.rs or a dedicated history module.
- [x] T016 [US3] Append terminal completed and failed session evidence, with explicit unknowns and write-failure reporting, in crates/fragcap-cli/src/commands/deep_capture.rs.
- [x] T017 [US3] Expose chronological history and named drift in native human and structured output without feeding it into plan authorization.

## Phase 6: Integration and verification

- [x] T018 Reconcile specification sections, ordered slice map, direct affected CLI help, and the dated changelog fragments in docs/fragcap-specification.md, docs/plans/README.md, CLI text, and changelog.d.
- [x] T019 Run the complete controlled launch and history matrix, cargo xtask ci, package and Windows gates, and inspect exact test-run cleanup and source encoding.
- [x] T020 Re-run Spec-Kit analyze over requirements, tasks, plan, and constitution; close findings, mark tasks complete, and commit only S209 changes.

## Dependencies and execution order

T001 and T002 precede red tests T003 through T005. T006 through T010 establish exact launch identity before T011 through T013 permit fallback. T014 and T015 establish storage and comparison before T016 and T017 write product outcomes. T018 through T020 finalize the implementation and evidence.

## Implementation strategy

Keep process proof and plan effects separate. Never relax the game-only gate to make a warm dispatch appear successful. Write controlled negative cases before widening any accepted launch case. Keep history append as a terminal observation and never as a preflight prerequisite.

## Verification record

2026-11-16: `cargo xtask ci` passed on the S209 worktree after the final code edits. It included formatting, Clippy, workspace tests, repository lint and dependency checks, package certification, documentation checks, native protocol conformance, performance registry, and Windows integration registry. The exact test-clean root was removed. Focused tests also exercised warm platform generation revalidation, direct alternate authority and early direct-route admission, history migration, append failure, retention, and advisory comparisons. No installed ESO run or live signed-driver qualification is claimed.
