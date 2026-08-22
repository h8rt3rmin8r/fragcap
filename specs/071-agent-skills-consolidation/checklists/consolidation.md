# Consolidation Checklist: Agent skills consolidation

**Purpose**: Slice-specific quality gate over the prune, the re-sourcing, and the new structural check
**Created**: 2026-08-22
**Feature**: [spec.md](../spec.md)

Unchecked items are real, narrow edges to resolve at plan time, not blockers.
`plan.md` resolves each by name.

## Admission and prune

- [x] CHK001 The admission test is stated positively and can be applied by a
      reader to a skill it has never seen. [Clarity, Spec §FR-001]
- [x] CHK002 Every kept skill has a named principle or a named executing gate
      behind it, cited to a file and line. [Completeness, Spec §E-01, §E-02]
- [x] CHK003 Every dropped skill is enumerated, not summarized as "the rest".
      [Completeness, Spec §FR-005]
- [x] CHK004 The one skill that would turn a gate red if dropped is identified
      before any deletion, not discovered by a failing build. [Risk, Spec §E-01]
- [x] CHK005 The cost of dropping is stated for each affected agent surface,
      including the one that reads the tree directly. [Completeness, Spec §E-22]
- [x] CHK006 A skill dropped for being off-domain has that judgment supported by
      its own content, not by its name. [Ambiguity, Spec §E-08]

## Provenance and integrity

- [x] CHK007 The upstream is public, carries an allowlisted licence, and
      resolves to a specific release. [Completeness, Spec §E-19]
- [x] CHK008 Downloaded bytes are verified against a publisher-supplied digest
      before they enter the tree, not after. [Risk, Spec §FR-002]
- [x] CHK009 The lock's entry schema is unchanged in shape, so an external tool
      that owns it is not broken. [Compatibility, Spec §FR-006]
- [x] CHK010 The three pre-existing hash divergences are accounted for
      explicitly, and their disappearance is recorded rather than left to look
      like a cover-up. [Traceability, Spec §E-06, §FR-016]
- [ ] CHK011 The `computedHash` values written for the four surviving entries
      are produced by a stated method, and the plan says what confidence that
      method carries and what would falsify it. [Measurability, Spec §FR-007]
- [x] CHK012 No vendored file is edited after vendoring, and the reason that
      matters is recorded rather than assumed. [Consistency, Spec §FR-003]

## The gate

- [x] CHK013 Each assertion maps to a drift class actually observed in this
      repository, not to a hypothetical one. [Traceability, Spec §FR-008]
- [x] CHK014 The assertion that would have caught the longest-standing defect is
      identified by name. [Traceability, Spec §E-04]
- [x] CHK015 Every assertion is demonstrated failing for its own reason before
      being reported as passing. [Measurability, Spec §FR-011]
- [ ] CHK016 The gate's source of truth for "tracked by git" is stated, along
      with its behavior when git metadata is unavailable, so the check cannot
      report a false pass in an environment it cannot actually inspect.
      [Ambiguity, Spec §FR-008]
- [x] CHK017 The gate excludes CLI-owned directories without hard-coding a list
      that will go stale. [Consistency, Spec §OOS-001]
- [x] CHK018 The gate does not verify hashes, and the reason is recorded so a
      later contributor does not add it back as an obvious improvement.
      [Completeness, Spec §OOS-003, §FR-009]

## Instruction surface

- [x] CHK019 Every rewritten claim is checked against the tree at commit time,
      not against the tree as it was when the spec was written. [Measurability,
      Spec §SC-007]
- [x] CHK020 A claim corrected as historical record says what superseded it
      rather than being silently deleted. [Traceability, Spec §FR-015]
- [ ] CHK021 The removal procedure's home is decided, and the decision accounts
      for the three files that currently describe the vendoring mechanism in
      partial overlap. [Ambiguity, Spec §FR-013]
- [x] CHK022 A pinned-artifact change carries a dated decision fragment.
      [Compliance, Constitution pinned-artifact rule]

## Notes

CHK011, CHK016, and CHK021 are deliberately unchecked. Each is a narrow decision
that belongs in `plan.md` rather than the specification:

- **CHK011** turns on the fact that the hash algorithm available in this
  repository was reverse-engineered by a prior session rather than published by
  the tool that writes the file. The specification records that (E-06); what
  confidence to place in a freshly written hash, and what would reveal it wrong,
  is a design question.
- **CHK016** is the difference between a gate that inspects git's index and one
  that reparses ignore rules. They disagree in exactly the case that produced
  E-04, and the choice determines whether the gate can report a false pass.
- **CHK021** is a placement question across `skills/README.md`, `AGENTS.md`, and
  `docs/plans/000-repository-foundation.md`, which describe the same mechanism
  at three different depths. Putting the procedure in the wrong one creates the
  fourth partial description.

All other items pass.
