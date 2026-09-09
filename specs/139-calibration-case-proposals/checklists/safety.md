# Safety Requirements Checklist: Guided Calibration Case Proposals

**Purpose**: Review the requirement boundary for a pure pre-effect Deep Capture proposal authority
**Created**: 2026-09-09
**Feature**: [spec.md](../spec.md)

## Scope Completeness

- [x] CHK001 Are target resolution, registration, execution, trust, artifacts, persistence, and ordinary eligibility explicitly excluded? [Completeness, Spec FR-017]
- [x] CHK002 Is the complete set of supported real-target topology classes defined without implying future platform support? [Completeness, Spec FR-001]
- [x] CHK003 Is the source and completeness requirement for process state explicit? [Completeness, Spec FR-005]

## Truth and Ambiguity

- [x] CHK004 Are missing, stale, legacy, mismatched, negative, and conflicting evidence separately specified? [Clarity, Spec FR-012]
- [x] CHK005 Are ambiguous topology candidates preserved rather than silently ranked? [Consistency, Spec FR-001 through FR-004]
- [x] CHK006 Is cold state prohibited when snapshot authority is incomplete or unavailable? [Coverage, Spec FR-005 and FR-014]
- [x] CHK007 Is protocol discovery limited to observed or explicitly supplied closed-set candidates? [Coverage, Spec FR-010]

## Effect Safety

- [x] CHK008 Does the spec prohibit process control while still defining operator-owned warm-state guidance? [Security, Spec FR-006]
- [x] CHK009 Does the spec gate trust-bearing work on exact final-client reachability? [Security, Spec FR-009]
- [x] CHK010 Are all external effects excluded from proposal generation and its controlled tests? [Security, Spec FR-013, FR-016, and FR-017]

## Determinism and Traceability

- [x] CHK011 Is equivalent-input determinism defined across ordering, duplicates, and image-name case? [Measurability, Spec FR-015 and SC-004]
- [x] CHK012 Does every runnable attempt retain the complete exact compatibility case identity? [Traceability, Spec FR-008]
- [x] CHK013 Does every retest carry one stable evidence reason? [Traceability, Spec FR-012 and SC-005]

## Notes

- Standard-depth PR-review checklist. The requirements fully cover the selected safety and truthfulness risks.
