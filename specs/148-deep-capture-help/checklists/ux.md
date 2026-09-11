# UX Requirements Checklist: Deep Capture Embedded Workflow Help

**Purpose**: Challenge whether the specification defines a complete, navigable, and testable operator experience

**Created**: 2026-09-11

**Feature**: [spec.md](../spec.md)

## Journey Definition

- [x] CHK001 Is the starting knowledge limited to an installed game name and the endpoint an exact ordinary Deep Capture command? [US1, FR-002, SC-001]
- [x] CHK002 Are all audited entry points named rather than implied by broad wording such as "related commands"? [FR-003]
- [x] CHK003 Is the distinction between registration, calibration, execution, recovery, cleanup, and export explicit? [FR-005]
- [x] CHK004 Does each refusal category require exact pasteable guidance rather than a vague cross-reference? [FR-008]
- [x] CHK005 Do calibration stops require known facts, unknown facts, and a next command? [FR-009]

## Progressive Disclosure

- [x] CHK006 Does the specification preserve concise short help while assigning the complete journey to long help? [FR-001, FR-002]
- [x] CHK007 Are the normal path and advanced, sensitive, storage, networking, and troubleshooting controls required to be visibly distinct? [FR-007]
- [x] CHK008 Are both Steam and direct or publisher launch families represented? [FR-004]
- [x] CHK009 Does Doctor bridge environment readiness to target and calibration readiness? [FR-010]
- [x] CHK010 Are bundle sensitivity, destructive cleanup, and failed-cleanup retention explained together? [FR-011]

## Accessibility And Verification

- [x] CHK011 Are 40, 60, and 80 display columns named as measurable rendering points? [FR-013, SC-004]
- [x] CHK012 Is meaning required to survive color-disabled output? [FR-013]
- [x] CHK013 Are quoted and non-ASCII target selectors covered as an edge case?
- [x] CHK014 Is every displayed command backed by parse validation? [FR-015, SC-002]
- [x] CHK015 Is every audited short help, long help, and refusal category covered by automation? [FR-014, SC-003]

## Notes

- The checklist evaluates requirement quality, not the rendered implementation.
