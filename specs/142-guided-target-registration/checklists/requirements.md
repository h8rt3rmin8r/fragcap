# Requirements Checklist: Guided Target Discovery and Registration

**Purpose**: Review the completeness, clarity, consistency, measurability, and coverage of the S142 requirements before planning and implementation.
**Created**: 2026-09-10
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are stored resolution, discovery fallback, candidate selection, registration authorization, revalidation, persistence, and guided handoff all specified? [Completeness, Spec §FR-001-017]
- [x] CHK002 Are every permitted selector and the precedence between row numbers and Steam application identifiers defined? [Completeness, Spec §FR-003-005]
- [x] CHK003 Are the exact candidate fields bound to preview and revalidation defined? [Completeness, Spec §FR-008-009]
- [x] CHK004 Are human and structured confirmation requirements both defined, including output flush and complete-line behavior? [Completeness, Spec §FR-010-013]
- [x] CHK005 Are post-registration continuation and unsupported-topology outcomes both specified? [Completeness, Spec §FR-017-020]

## Requirement Clarity

- [x] CHK006 Is an "exact" discovery match limited to objective identifier and name rules, with every fuzzy alternative excluded? [Clarity, Spec §FR-003]
- [x] CHK007 Is the registration boundary distinguished from store infrastructure maintenance and from every later session effect? [Clarity, Spec §FR-013, Assumptions]
- [x] CHK008 Is candidate drift defined as a complete canonical-authority mismatch rather than a subjective equivalence test? [Clarity, Spec §FR-014]
- [x] CHK009 Are registration outcomes named independently from calibration readiness and completion? [Clarity, Spec §FR-018-020]

## Requirement Consistency

- [x] CHK010 Do discovery and persistence requirements preserve P-10's single `TargetSource` and `register_candidate` authorities? [Consistency, Spec §FR-002, FR-015]
- [x] CHK011 Do target honesty requirements preserve S133 precision without applying automatic-registration thresholds to an explicit confirmed choice? [Consistency, Spec §FR-006-008, Assumptions]
- [x] CHK012 Do continuation requirements preserve S139-S141 proposal ordering, one-session limit, and exact fact authority? [Consistency, Spec §FR-017-018, FR-022]

## Acceptance Criteria Quality

- [x] CHK013 Can zero pre-confirmation target-row and session effects be objectively demonstrated? [Measurability, Spec §SC-001-002]
- [x] CHK014 Can plan-integrity and stale-confirmation behavior be measured by deterministic identifier changes and unchanged target counts? [Measurability, Spec §SC-003, SC-005]
- [x] CHK015 Can durable handoff and authorization separation be proven through exact identifiers and consumed input lines? [Measurability, Spec §SC-004, SC-006]

## Scenario and Edge-Case Coverage

- [x] CHK016 Are primary, stored-target, ambiguous, no-match, decline, invalid-input, drift, race, and unsupported-topology flows specified? [Coverage, Spec §User Stories, Edge Cases]
- [x] CHK017 Are discovery warnings and incomplete coverage preserved rather than hidden behind no-match behavior? [Coverage, Spec §FR-007]
- [x] CHK018 Is the no-durable-identifier state of an unregistered path candidate explicit? [Edge Case, Spec §FR-005, Edge Cases]
- [x] CHK019 Are flush failure, input closure, interruption, and concurrent prior registration addressed? [Exception Flow, Spec §FR-010-016]

## Non-Functional Requirements and Scope

- [x] CHK020 Are constant-time exact comparison, deterministic canonicalization, domain separation, and revalidation requirements explicit? [Security, Spec §FR-009, FR-012, FR-014]
- [x] CHK021 Are prohibited process, trust, routing, extraction, orchestration, schema, and dependency expansions listed? [Scope, Spec §FR-022]
- [x] CHK022 Is parent issue #380 intentionally left open with later workflow-state, multi-attempt, override, and completion work named? [Dependency, Spec §FR-022, Assumptions]

## Notes

- Completed during the specification review. No unresolved placeholder or ambiguity remains.
