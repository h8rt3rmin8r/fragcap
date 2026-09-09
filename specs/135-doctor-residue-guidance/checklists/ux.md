# Human and Machine Guidance Requirements Checklist: Doctor Residue Guidance

**Purpose**: Validate the clarity, completeness, consistency, and safety of S135 presentation requirements before planning
**Created**: 2026-09-09
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are diagnosis requirements defined for healthy, active, stale, cleanup-failed, unknown, unsupported, recoverable, and non-recoverable residue states? [Completeness, Spec FR-005 through FR-008]
- [x] CHK002 Are the four required meanings of the abandoned session-owner regression stated explicitly? [Completeness, Spec FR-004]
- [x] CHK003 Are both primary diagnosis and secondary identity-context requirements defined? [Completeness, Spec FR-001 through FR-003]
- [x] CHK004 Are structured machine facts enumerated rather than described only as exact or complete? [Completeness, Spec FR-016]

## Requirement Clarity

- [x] CHK005 Is plain-language guidance distinguished unambiguously from internal key-value vocabulary? [Clarity, Spec FR-001, FR-009, SC-007]
- [x] CHK006 Is the minimum supported narrow width quantified in display columns? [Clarity, Spec FR-012]
- [x] CHK007 Is the indivisible-token exception narrow enough to preserve values without becoming a general line-width escape? [Clarity, Spec FR-011, SC-003]
- [x] CHK008 Is active-ownership wording separated from abandoned or ambiguous ownership wording? [Clarity, Spec FR-002, FR-004, FR-005, FR-008]

## Requirement Consistency

- [x] CHK009 Do human simplification requirements preserve the exact machine and recovery truth required by P-9? [Consistency, Spec FR-015 through FR-019]
- [x] CHK010 Do recoverable-finding remediations remain consistent with the existing confirmation-gated Doctor action contract? [Consistency, Spec FR-009, FR-018]
- [x] CHK011 Do default, narrow, plain, and color layout requirements preserve one consistent visible information set? [Consistency, Spec FR-010 through FR-015]

## Scenario and Edge-Case Coverage

- [x] CHK012 Are missing, long, whitespace-bearing, and non-ASCII identity cases addressed? [Coverage, Spec Edge Cases]
- [x] CHK013 Are multiple same-kind findings and structured escaping covered without relying on a dynamic human check name? [Coverage, Spec Edge Cases, FR-003, FR-016]
- [x] CHK014 Are partial cleanup and failed cleanup requirements explicit about retained evidence and retry truth? [Coverage, Spec US3, FR-007, FR-018]

## Non-Functional and Scope Boundaries

- [x] CHK015 Are display-cell width, complete-character preservation, and color-sequence handling all specified? [Coverage, Spec FR-014]
- [x] CHK016 Is zero-effect read-only Doctor behavior preserved explicitly? [Security, Spec FR-019]
- [x] CHK017 Are excluded cleanup-policy, inventory-classification, broader UX, and completion claims named? [Scope, Spec FR-022]
- [x] CHK018 Can every success criterion be objectively measured from controlled output and action sets? [Measurability, Spec SC-001 through SC-008]

## Notes

- All requirements-quality checks pass after the clarification pass fixed the supported narrow-width boundary at 40 display columns.
