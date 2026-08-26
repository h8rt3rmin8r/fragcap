# Warning Requirements Checklist: Marker Cap Warning Subject

**Purpose**: Validate that warning requirements are complete, clear, and testable before implementation
**Created**: 2026-08-26
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are subject-naming requirements defined for the capped binary-marker warning? [Completeness, Spec §FR-001]
- [x] CHK002 Are count-preservation requirements defined for skipped executable candidates? [Completeness, Spec §FR-002]
- [x] CHK003 Are consequence requirements defined for what incomplete technology detection means to the operator? [Completeness, Spec §FR-004]
- [x] CHK004 Are shared-warning requirements defined so callers do not drift? [Completeness, Spec §FR-005, Spec §FR-006]

## Requirement Clarity

- [x] CHK005 Is the scan subject identified as the scanned root rather than a row number or surrounding table context? [Clarity, Spec §FR-001]
- [x] CHK006 Is the skipped quantity specified as the exact candidate count? [Clarity, Spec §FR-002]
- [x] CHK007 Is the operator consequence described without naming source constants or hidden implementation details? [Clarity, Spec §FR-003, Spec §FR-004]

## Scenario Coverage

- [x] CHK008 Are standalone technology scans and target discovery scans both covered? [Coverage, Spec §US1, Spec §US2]
- [x] CHK009 Are multi-root discovery warnings covered so duplicate-looking warnings remain distinguishable? [Coverage, Spec §US1]
- [x] CHK010 Are mixed reduced-coverage causes covered, including unreadable subtree plus marker-cap truncation? [Coverage, Spec §US1]

## Edge Case Coverage

- [x] CHK011 Are path readability concerns covered for relative roots and spaces? [Edge Case]
- [x] CHK012 Is the single-line warning constraint documented? [Edge Case]
- [x] CHK013 Is the absence of a cap-adjustment remedy explicitly bounded? [Edge Case, Spec §Assumptions]
