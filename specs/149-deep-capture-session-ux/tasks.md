# Tasks: Deep Capture Session UX Completion

**Input**: [spec.md](spec.md), [plan.md](plan.md), and supporting design artifacts.

## Phase 1: Setup

- [x] T001 Verify native authority, repository identity, ignore rules, output contracts, and controlled acceptance in AGENTS.md, CONTRIBUTING.md, .gitignore, and crates/fragcap-cli/src/commands/deep_capture.rs.
- [x] T002 Complete specify, clarify, checklist, plan, and tasks in specs/149-deep-capture-session-ux/ before implementation.

## Phase 2: Foundational

- [x] T003 Pass read-only consistency analysis across specs/149-deep-capture-session-ux/spec.md, plan.md, and tasks.md.
- [x] T004 Write and observe failing pure presentation tests in crates/fragcap-cli/src/session_ux.rs, then use existing shared display-cell wrapping and stderr-width selection in crates/fragcap-cli/src/display.rs.

## Phase 3: User Story 1 - Understand Authorization

**Independent test**: Synthetic trust/sensitive selection plus existing controlled zero-effect authorization tests.

- [x] T005 [US1] Implement exact plan-derived consequence summary and integrate it with checked canonical plan emission in crates/fragcap-cli/src/session_ux.rs and commands/deep_capture.rs (FR-001 through FR-003).
- [x] T006 [US1] Cover consequence visibility, declined/invalid/closed/interrupted responses, and failed writes in crates/fragcap-cli/tests/cli_deep_capture.rs (FR-001 through FR-003, FR-007, FR-008).

## Phase 4: User Story 2 - Follow Observed Progress

**Independent test**: Synthetic typed lifecycle events and fixed inspection counters, plus integrated controlled sessions.

- [x] T007 [US2] Implement fixed saturating inspection counters and typed lifecycle human projection in crates/fragcap-cli/src/session_ux.rs and commands/deep_capture.rs (FR-004, FR-005).
- [x] T008 [US2] Add truthful fact-persistence and finalization progress at existing adapter calls in crates/fragcap-cli/src/commands/deep_capture.rs and verify observation-only semantics in unit and command tests (FR-004, FR-005, FR-007).

## Phase 5: User Story 3 - Recover From Partial Results

**Independent test**: Synthetic complete/partial/interrupted/failed terminal cases, exact artifact presence, and independent cleanup results.

- [x] T009 [US3] Implement verbosity-gated terminal text in crates/fragcap-cli/src/emit.rs and actual post-run terminal evidence/recovery summary in session_ux.rs and commands/deep_capture.rs (FR-006, FR-007).
- [x] T010 [US3] Cover released, failed, timed-out, missing evidence, retained sensitive evidence, quiet/silent/JSON, and narrow Unicode contracts in crates/fragcap-cli/src/session_ux.rs and tests/cli_deep_capture.rs (FR-006 through FR-008).

## Phase 6: Polish and Verification

- [x] T011 Record every #332 criterion with exact executable references in specs/149-deep-capture-session-ux/acceptance.md; update docs/fragcap-specification.md, docs/fragcap-spec-outline.md, docs/plans/README.md, and changelog.d/S149-deep-capture-session-ux.added.md without completion claims (FR-009).
- [x] T012 Run controlled tests and complete foreground cargo xtask ci; inspect git diff and text hygiene, then record evidence in specs/149-deep-capture-session-ux/acceptance.md (FR-007 through FR-009, SC-001 through SC-005).
- [x] T013 Commit, push codex/s149-deep-capture-session-ux, publish PR with issue traceability, handle received review findings, and complete at most one manually triggered second review round (SC-005). See the external delivery gate below for current-head checks and operator handoff.

## External Delivery Gate

PR #410 is the live authority for current-head checks and review state. The first Codex finding was corrected and resolved, and the one authorized second review completed without new findings on `fe07abc`. Subsequent active-harness synchronization changes only the already-reviewed Rustls pin and lock checksums, not product source. SC-005 remains required: wait until every current-head required check is green and every arrived finding is handled before requesting the operator's final review and merge ritual. Completed repository tasks do not assert completion of still-running external checks, independent review issue #333, or the native feature gate #334. The agent never merges this PR.

## Dependencies and Execution Order

Setup and analyze block implementation. T004 precedes all presentation changes. US1, US2, and US3 can be tested independently but integration edits to commands/deep_capture.rs are serial. T011 and T012 follow all stories, and T013 follows verified code.

## Parallel Opportunities

Pure contract tests and acceptance-document drafting occupy separate files, but this run remains serial because no independent research agent is required. Within each story, tests precede corresponding implementation.

## Implementation Strategy

Deliver US1 first as the consent MVP, then observed progress, then terminal recovery guidance. Preserve exact policy authority throughout; never close a criterion on intent or a skipped test.
