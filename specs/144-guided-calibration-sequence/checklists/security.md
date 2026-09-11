# Security Requirements Checklist: Bounded Guided Calibration Sequence

**Purpose**: Review S144 authorization, progression, evidence, and termination requirements before implementation
**Created**: 2026-09-11
**Feature**: [spec.md](../spec.md)

## Authorization Boundaries

- [x] CHK001 Is every effectful attempt covered by one fresh complete S134 plan and separate confirmation? [Completeness, Spec FR-010 through FR-012]
- [x] CHK002 Are registration, target setup, warm restart, and session decisions explicitly non-transferable? [Clarity, Spec FR-011]
- [x] CHK003 Do decline, closed input, invalid input, interruption, and drift stop later attempts? [Coverage, Spec FR-013 and FR-014]
- [x] CHK004 Is blanket sequence authorization explicitly excluded? [Scope, Clarifications]

## Fresh Authority and Evidence

- [x] CHK005 Are the target, process snapshot, facts, and proposal refreshed before each attempt? [Completeness, Spec FR-003]
- [x] CHK006 Is progression conditioned on a fresh current positive fact rather than session exit alone? [Clarity, Spec FR-017 through FR-019]
- [x] CHK007 Are observed candidates limited to eligible final-client evidence and kept distinct from facts? [Consistency, Spec FR-015]
- [x] CHK008 Are warm, unavailable, limited, changed, and deleted authorities required to stop before another plan? [Exception Flow, Spec FR-004]

## Finite Execution and Artifact Safety

- [x] CHK009 Is each exact case limited to one execution per invocation? [Measurability, Spec FR-006 and FR-008]
- [x] CHK010 Is the complete attempt bound closed and mechanically testable? [Completeness, Spec FR-007 and SC-004]
- [x] CHK011 Are explicit bundle destinations unique, deterministic, path-safe, and still subject to empty-root validation? [Coverage, Spec FR-022 through FR-024]
- [x] CHK012 Are partial evidence and repeated proposals prevented from causing automatic retries? [Edge Case, Spec FR-019]

## Honesty and Prohibited Capabilities

- [x] CHK013 Are completed and remaining coverage derived from current exact evidence after every attempt? [Consistency, Spec FR-017, FR-020, FR-021]
- [x] CHK014 Are failure, refusal, incomplete, decline, interruption, and no-progress outcomes distinguishable? [Clarity, Spec FR-021]
- [x] CHK015 Are persistent state, process control, broad proxy effects, pinning bypass, topology invention, and parent completion excluded? [Scope, Spec FR-028]
- [x] CHK016 Are tests required for every authorization, progression, stopping, bound, and artifact boundary? [Measurability, Spec FR-026 and SC-001 through SC-008]

## Review Result

- The specification defines complete per-attempt authority, current-evidence progression, finite termination, and artifact-destination boundaries for S144.
