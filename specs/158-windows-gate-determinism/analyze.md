# Specification Analysis Evidence: S158

**Date**: 2026-09-19

## Blocking Gate

Read-only analyze phase passed before implementation. No `.specify/extensions.yml` hooks are installed. Required spec, plan, tasks, constitution and design artifacts exist and were inspected. All 15 functional requirements and 9 measurable outcomes have task coverage. Twenty-three tasks preserve prerequisite, TDD, story, documentation, verification and review ordering. No unresolved clarification, conflicting requirement, untraceable implementation task, duplicated authority or constitutional violation remains.

## Coverage

| Requirements | Tasks | Acceptance |
| --- | --- | --- |
| FR-001 through FR-005, SC-001, SC-002 | T005 through T007 | Deterministic pre-ready hold, failed start settlement, unchanged bounded queue and exact output |
| FR-006 through FR-009, SC-003, SC-004 | T008 through T011 | Structured complete and incomplete drain, stage-owned deadlines, cancellation and cleanup |
| FR-010, FR-011, SC-005, SC-006 | T012 through T014 | Poison recovery, unwind-safe environment restoration and exact Windows regression |
| FR-012, FR-013, SC-009 | T015 through T018 | Specification and changelog reconciliation, full gates and no sensitive local product execution |
| FR-014, FR-015, SC-007, SC-008 | T019 through T023 | First-attempt hosted Windows evidence, complete review response and final green head |

## Boundary Decisions

The application queue is not enlarged, producer calls do not block or retry, and required generic UDP or QUIC records are not removed. The fix establishes consumer readiness before publication and preserves truthful later saturation.

The public four-deadline authorization plan is not expanded. Capture execution owns `observation`; capture stop, proxy stop and structured observation drain share the remaining displayed `shutdown` budget. Complete cached evidence can return with zero work remaining, while incomplete work cannot.

Mutex poison recovery does not suppress or reclassify the original test failure. It prevents that failure from manufacturing unrelated later failures. Scope-owned environment restoration is required even on unwind.

No installed product, real game, real trust mutation or sensitive live capture is part of local verification. Controlled source tests and hosted disposable Windows jobs remain the execution boundary.

## Implementation Convergence

Local implementation and verification pass. The writer readiness and startup-failure regressions, structured complete, incomplete and failed drain cases, genuine capture and shutdown deadline cases, cancellation and cleanup coverage, poisoned-lock recovery and exact unwind restoration all pass.

The implementation deliberately deviates from the initial research wording that would have changed `ProxyLease::observations`. That method belongs to the curated stable API, so S158 preserves it and adds a defaulted `drain_observations` method plus two additive stable types. Existing adapter implementations and callers remain source-compatible while the session coordinator uses the stronger structured contract.

The complete local gate set passed: formatting, all-target and all-feature clippy, locked workspace tests, the aggregate CI authority, MSRV 1.88, neutral core crates, strict UTF-8 without BOM, forbidden-dash checks and mojibake detection. No installed product, real game, real trust mutation or sensitive live capture ran locally.

Next action: publish the official pull request and require first-attempt hosted Windows evidence on its final head.
