# Specification Quality Checklist: Scope separation and false-positive risk

**Purpose**: Validate requirements quality for the two riskiest properties of
this slice: title-scope vs machine-scope separation, and false-positive
avoidance in the launch-entry classifier
**Created**: 2026-08-22
**Feature**: [spec.md](../spec.md)
**Depth**: Standard
**Audience**: Reviewer (PR)

## Requirement Completeness

- [x] CHK001 - Is the rule for combining a directory-scan finding and a
  launch-entry finding for the same product fully specified (which wins,
  and on what basis)? [Completeness, Spec §FR-005]
- [x] CHK002 - Is the behavior when the machine probe cannot run at all
  (not just "finds nothing") specified separately from "finds nothing"?
  [Completeness, Spec §FR-008, §Edge Cases]

## Requirement Clarity

- [x] CHK003 - Is "specific, unambiguous tokens" for the launch-entry
  classifier concrete enough that a reviewer could reject an implementation
  that substring-matches the word "anti-cheat"? [Clarity, Spec §FR-004]
- [x] CHK004 - Is the negative counter-example (the EAC-disabled launch
  variant) specific enough to be encoded as a literal regression test rather
  than left as a general principle? [Clarity, Spec §User Story 2 Scenario 3]

## Requirement Consistency

- [x] CHK005 - Do FR-002 (EOSSDK must never be evidence) and FR-004 (no
  broad substring match) apply the same underlying principle consistently
  across both the directory-scan source and the launch-entry source, rather
  than stating the rule once and leaving the other source's obligation
  implicit? [Consistency, Spec §FR-002, §FR-004]
- [x] CHK006 - Is FR-007's "never merged into, or used to infer" consistent
  with User Story 3 Scenario 3's example, with no scenario elsewhere
  implying the opposite (e.g. a title's row changing because the machine
  probe fired)? [Consistency, Spec §FR-007, §User Story 3]

## Scenario Coverage

- [x] CHK007 - Is the case where a title has both a positive and an
  explicitly-disabling launch entry (like Halo: MCC) addressed, rather than
  only the disabling entry in isolation? [Coverage, Spec §Edge Cases]
- [x] CHK008 - Is the case where the appinfo cache is entirely empty
  addressed for the classifier specifically (not just generally for appinfo
  reading elsewhere in the codebase)? [Coverage, Spec §Edge Cases]

## Dependencies & Assumptions

- [x] CHK009 - Is the decision to exclude BattlEye and Vanguard from the
  machine-wide probe explicitly justified by a stated principle (not
  measured, avoid unverified claims), rather than left as an unexplained
  scope cut? [Assumption, Spec §Assumptions]
- [x] CHK010 - Is the decision to exclude source D (Steam Deck compat
  tokens) justified against the actual acceptance criteria, rather than
  assumed out of scope without reasoning? [Assumption, Spec §Assumptions]

## Notes

- All items pass. FR-002/FR-004's shared principle, the MCC counter-example,
  and the BattlEye/Vanguard exclusion are all drawn directly from the
  issue's own text rather than invented, which is why they check out cleanly.
- No blocking gaps found; proceeding to `/speckit-plan`.
