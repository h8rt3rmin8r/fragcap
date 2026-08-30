# Lifecycle and Safety Requirements Checklist: Library-First Deep Capture Sessions

**Purpose**: Test the completeness, clarity, consistency, and measurability of the public lifecycle, adapter, failure, and safety requirements before implementation
**Created**: 2026-08-29
**Feature**: [spec.md](../spec.md)

**Review depth**: Formal pre-implementation gate
**Audience**: Pull request reviewers and API maintainers

## Public Lifecycle Completeness

- [x] CHK001 Are all lifecycle phases available to a non-CLI consumer and named in their required order? [Completeness, Spec FR-001, FR-006]
- [x] CHK002 Is the boundary between side-effect-free preflight and side-effecting execution explicit? [Clarity, Spec FR-002]
- [x] CHK003 Are invalid ordering, repeated calls, and terminal-session reuse defined? [Edge Case, Spec FR-015]
- [x] CHK004 Are the caller's confirmation responsibility and the library's plan responsibility distinguishable? [Consistency, Spec Assumptions]

## Adapter Contract Completeness

- [x] CHK005 Are every privileged, external, persistent, temporal, and event-delivery concern assigned a substitutable boundary? [Completeness, Spec FR-004]
- [x] CHK006 Is each adapter's obligation to preserve observations, deadlines, and typed failures specified? [Clarity, Spec FR-003, FR-007]
- [x] CHK007 Is replacement-backend compatibility defined without requiring CLI or artifact-contract changes? [Acceptance Criteria, Spec US2]
- [x] CHK008 Is the relationship to ordinary Capture explicit enough to prevent a second packet pipeline? [Consistency, Spec FR-009]

## Failure and Recovery Coverage

- [x] CHK009 Are exception requirements present for every side-effecting stage and independent finalization action? [Coverage, Spec FR-012, FR-016]
- [x] CHK010 Are partial evidence, partial artifacts, fact-write disagreement, and cleanup failure represented without erasing earlier truth? [Completeness, Spec US3]
- [x] CHK011 Is terminal-state agreement required across every in-memory, event, and artifact authority? [Consistency, Spec FR-014]
- [x] CHK012 Are event-delivery failure semantics defined separately from session-operation failure? [Edge Case, Spec FR-016]

## Security and Evidence Quality

- [x] CHK013 Are the explicit-consent, target-scope, reversibility, audit, and prohibited-technique requirements complete? [Coverage, Spec FR-007, FR-008]
- [x] CHK014 Is silence prohibited from becoming affirmative compatibility evidence across observations, facts, and artifacts? [Clarity, Spec FR-013]
- [x] CHK015 Are target ownership and append-only conflicting evidence consistent with the single target-store model? [Consistency, Spec FR-011]
- [x] CHK016 Is controlled verification explicitly independent of a driver, elevation, game, remote service, and real trust mutation? [Measurability, Spec FR-021, SC-006]

## Compatibility and Scope Boundaries

- [x] CHK017 Are all shipped CLI, event, artifact, persistence, and refusal surfaces covered by the compatibility requirement? [Completeness, Spec FR-017]
- [x] CHK018 Is the definition of a thin CLI specific enough to identify forbidden duplicated policy? [Clarity, Spec FR-018, SC-005]
- [x] CHK019 Are the native backend, direct-executable launch, protocol expansion, target-storage, and command-surface exclusions explicit? [Scope, Spec FR-022]
- [x] CHK020 Can every measurable outcome be checked through controlled tests or repository gates? [Acceptance Criteria, Spec SC-001 through SC-008]

## Notes

- All 20 requirements-quality checks passed before planning.
- Focus areas are public lifecycle architecture, failure recovery, evidence fidelity, security boundaries, and compatibility.
