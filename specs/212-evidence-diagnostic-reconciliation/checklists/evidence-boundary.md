# Evidence Boundary Checklist: Evidence and Diagnostic Debt Reconciliation

**Purpose**: Assess requirement quality for public tracking, diagnostic truth, and private-work separation.
**Created**: 2026-11-24
**Feature**: [spec.md](../spec.md)

## Tracker completeness

- [x] CHK001 Are the affected epic and related issue population, including metadata and dependency decisions, defined? [Completeness, Spec FR-001]
- [x] CHK002 Is completed evidence distinguished from remaining product work and from later optional field observation? [Consistency, Spec FR-002]
- [x] CHK003 Is closure defined for an issue containing both completed investigation and an independent defect? [Coverage, Spec Edge Cases]
- [x] CHK004 Does the success gate require a disposition for every inventoried issue rather than a selective sample? [Measurability, Spec SC-001]

## Diagnostic truth

- [x] CHK005 Are clean zero-byte EOF, zero-byte read error, partial EOF, partial read error, size refusal, complete head, and normal absence of the failure field distinct? [Coverage, Spec US2]
- [x] CHK006 Is the count tied to bytes actually observed, with no inferred request or client motive? [Clarity, Spec FR-005]
- [x] CHK007 Is downstream absence distinguished from an empty or null count, and is the bundle failure array omitted when there is no valid counted head-read failure? [Consistency, Spec FR-006]
- [x] CHK008 Does a controlled bundle test derive a sanitized failure row from the same native `proxy.jsonl` record instead of comparing independently constructed synthetic records? [Evidence, Spec FR-007/SC-004]
- [x] CHK009 Are content-retention and forwarding boundaries stated separately from diagnostic storage? [Completeness, Spec FR-006/FR-008]

## Public and private separation

- [x] CHK010 Is the prohibited public detail set explicit across tracker, source, Spec Kit, commits, and PR text? [Completeness, Spec FR-003]
- [x] CHK011 Is every dirty file and substantive hunk accounted for without requiring experimental code to be merged? [Measurability, Spec FR-004/SC-002]
- [x] CHK012 Is the original dirty worktree's recoverability required until extraction is verified? [Coverage, Spec FR-004]
- [x] CHK013 Do specification and planning updates preserve historical unverified gates instead of promoting them to success? [Consistency, Spec FR-009/Edge Cases]

## Finite audit exception

- [x] CHK014 Are the pre-existing audit failure, exact allowed parent, nine advisory IDs, expiry, and permanent replacement issue distinguished? [Completeness, Spec FR-010]
- [x] CHK015 Do acceptance criteria require both positive full/isolated audits and failing direct-parent/expiry controls without claiming universal safety? [Measurability, Spec FR-011/SC-006]
- [x] CHK016 Are the seven governed CI repair files separate from the seven Rust diagnostic source paths and the original 20-file inventory? [Consistency, Spec Scope/SC-002]

## Notes

- This checklist evaluates whether requirements cover the evidence and privacy boundary. It does not substitute for implementation tests or a public diff review.
