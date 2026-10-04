# First-run UX Requirements Quality Checklist

**Purpose**: Review the written operator-facing setup and failure requirements.
**Created**: 2026-10-03

## Setup and target detail

- [x] CHK001 Is the Steam launch hint named as heuristic rather than a proven client? [Clarity, Spec FR-004]
- [x] CHK002 Is the guided path defined for one candidate, no evidence, and ambiguous evidence? [Coverage, Spec FR-005]
- [x] CHK003 Is the active launch entry and role distinguished from metadata in target detail? [Consistency, Spec FR-007]

## Failure and recovery

- [x] CHK004 Are the causal stage, evidence counts, fact disposition, and workflow state required in a failed report? [Completeness, Spec FR-008]
- [x] CHK005 Are unknowns and written-but-empty artifacts handled without a success claim? [Clarity, Spec FR-008]
- [x] CHK006 Is repeated generic cleanup progress excluded while exact cleanup results remain? [Consistency, Spec FR-009]
- [x] CHK007 Are setup, reachability, protocol, and retry states covered by the published first-run guide? [Coverage, Spec FR-010]
