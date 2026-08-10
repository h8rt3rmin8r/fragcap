# Requirements Quality Checklist: Stage Matching and Session Lifecycle

**Purpose**: Validate that the S12 requirements are complete, clear, consistent, and measurable before implementation
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Stage Matching Completeness

- [ ] CHK001 - Are all five match predicates and their exact semantics specified, including case sensitivity and the field each reads? [Completeness, Spec §FR-002]
- [ ] CHK002 - Is the "all specified predicates must hold" conjunction rule stated unambiguously? [Clarity, Spec §FR-001]
- [ ] CHK003 - Is the behavior of `cmdline_contains` against an unobserved command line explicitly defined? [Edge Case, Spec §FR-002]
- [ ] CHK004 - Is `descends_from` defined as resolving over the synthetic tree rather than the OS parent chain? [Clarity, Spec §FR-004]
- [ ] CHK005 - Is precedence defined for a process that matches more than one stage? [Completeness, Spec §FR-005]
- [ ] CHK006 - Is the evaluation model for `descends_from` when the ancestor is not yet bound stated, and does it rest on a named guarantee? [Ambiguity, Spec Clarifications]

## Session Lifecycle Completeness

- [ ] CHK007 - Are all five states and every transition between them enumerated, including the Watching-to-Complete timeout edge? [Completeness, Spec §FR-007]
- [ ] CHK008 - Is the requirement that the capture handle open before any target exists stated? [Completeness, Spec §FR-008]
- [ ] CHK009 - Is the no-traffic-lost property at the Watching-to-Capturing boundary specified as a requirement, not just described? [Clarity, Spec §FR-010]
- [ ] CHK010 - Is the acquisition timeout's optionality and its effect when unset defined? [Completeness, Spec Clarifications]
- [ ] CHK011 - Are the three lifecycle classes and their distinct exit-handling rules specified, including that a service is never awaited? [Completeness, Spec §FR-006]

## Stop Conditions Coverage

- [ ] CHK012 - Are all six stop conditions enumerated? [Coverage, Spec §FR-012]
- [ ] CHK013 - Is "first to occur" stated so the conditions are not treated as simultaneous? [Clarity, Spec §FR-012]
- [ ] CHK014 - Is the same orderly-shutdown outcome required for every stop condition, including operator interrupt as a normal stop? [Consistency, Spec §FR-013]
- [ ] CHK015 - Is the case of a terminal stage that never binds addressed? [Edge Case, Spec Edge Cases]

## Loss Accounting (P-4)

- [ ] CHK016 - Is a named counter required for packets discarded during Watching? [Completeness, Spec §FR-009]
- [ ] CHK017 - Is the conservation relation (observed equals retained plus watching-discards plus other named discards) stated as a measurable outcome? [Measurability, Spec §SC-003]
- [ ] CHK018 - Is it clear that the watching discard is surfaced in statistics, not merely counted internally? [Clarity, Spec §FR-009]

## Testability and Boundaries

- [ ] CHK019 - Is the tier-1 testability requirement (scripted watcher, no driver/elevation/game) stated for both matching and lifecycle? [Completeness, Spec §FR-014, §SC-005]
- [ ] CHK020 - Are the acceptance scenarios for the ambiguous shared-image chain concrete enough to test? [Measurability, Spec User Story 1]
- [ ] CHK021 - Is scope-out explicit (CLI, filter management, live wiring)? [Clarity, Spec Input/Assumptions]

## Consistency and Dependencies

- [ ] CHK022 - Do the crate-placement requirements avoid asserting a forbidden attribution/capture sibling edge? [Consistency, Spec §FR-016]
- [ ] CHK023 - Are the invariants relied upon from S05 validation (unique roles, one terminal stage, acyclic descends_from, no ambiguous image match) documented as assumptions rather than re-checked? [Assumption, Spec Assumptions]
- [ ] CHK024 - Is the binding-storage decision (core setter vs side-map) resolved rather than left open? [Ambiguity, Spec Clarifications]
- [ ] CHK025 - Is every new domain term flagged for a glossary entry in the same change? [Traceability, Spec §FR-015]
