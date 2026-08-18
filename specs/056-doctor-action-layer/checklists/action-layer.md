# Requirements Quality Checklist: doctor --fix action layer

**Purpose**: Unit-test the S056 requirements for completeness, clarity,
consistency, and measurability before planning. Tests the requirements, not the
implementation.
**Created**: 2026-08-18
**Feature**: [spec.md](../spec.md)

## Classifier Purity Invariant

- [ ] CHK001 - Is "the classifier remains a pure Inputs to Report function" stated
  as a hard invariant with the specific prohibitions (no added probe, no changed
  status logic, existing tests unmodified)? [Clarity, Spec §FR-002]
- [ ] CHK002 - Is the boundary between the pure classifier and the action layer
  defined precisely enough that a reviewer can tell which code each requirement
  governs? [Clarity, Spec Overview, §FR-002]
- [ ] CHK003 - Is "doctor without --fix is behaviorally unchanged" made objectively
  verifiable (same checks, rendering, exit code)? [Measurability, Spec §FR-001,
  §SC-001]

## Action-Bound-to-Report Safety Invariant

- [ ] CHK004 - Is the rule that --fix acts only on actions carried by a check
  present and actionable in the current report stated unambiguously? [Clarity,
  Spec §FR-003]
- [ ] CHK005 - Is the requirement that the structured action and human-readable
  remediation "cannot drift" defined in a way that is testable rather than
  aspirational? [Measurability, Spec §FR-004]
- [ ] CHK006 - Is the negative case specified: an action definition whose check is
  absent is never offered or performed? [Coverage, Edge Case, Spec §FR-003,
  §US2-3, §SC-003]
- [ ] CHK007 - Is "actionable" itself defined (which statuses or findings carry an
  action, e.g. does a Warn carry one)? [Ambiguity, Gap, Spec §FR-003]

## Refusal and Confirmation Gating

- [ ] CHK008 - Are the exact refusal conditions for --fix (with --json; with
  non-TTY stdout) and their exit code (usage error, exit 2) specified? [Clarity,
  Spec §FR-007, §FR-008]
- [ ] CHK009 - Is the interaction between --yes and the non-TTY refusal specified
  (does --yes bypass the terminal requirement)? [Consistency, Spec §FR-009]
- [ ] CHK010 - Is "--yes without --fix is a usage error" stated? [Completeness,
  Spec §FR-009]
- [ ] CHK011 - Is the per-action confirmation flow (name before performing, act
  only on confirm, skip on decline, continue) fully specified in order? [Coverage,
  Spec §FR-006, §US1]
- [ ] CHK012 - Is the confirmation seam required to be testable with a scripted
  double (no real terminal), and is that requirement traceable? [Measurability,
  Spec §FR-017, Key Entities]

## Action Catalog Completeness and Consistency

- [ ] CHK013 - Is every finding in the action table matched to exactly one action,
  with no finding left without an action and no action without a finding?
  [Completeness, Consistency, Spec §US3]
- [ ] CHK014 - Is the npcap action's single-primary-action shape (no nested
  sub-menu) and its default-vs-net behavior specified? [Clarity, Spec §FR-012,
  Clarifications]
- [ ] CHK015 - Is the "relaunch elevated" action's handoff behavior (elevated child
  re-runs, parent stops) specified rather than left implicit? [Clarity, Spec
  §FR-014, Clarifications]
- [ ] CHK016 - Is the discovery action pinned to tiers 1 and 2 (the same discovery
  the S055 listing runs) rather than an unbounded discovery? [Clarity, Spec
  §FR-015]
- [ ] CHK017 - Is the catalog action scoped to the missing-store case, with
  stale-store detection explicitly out of scope? [Scope, Consistency, Spec §FR-016,
  Assumptions, Clarifications]

## Network Gating and Degradation

- [ ] CHK018 - Are the network-dependent actions identified, and is the gating
  capability named consistently with existing project gating? [Consistency, Spec
  §FR-012, §FR-016, Assumptions]
- [ ] CHK019 - Is the degraded (no-network) behavior of each network action
  specified so a default build still tells the operator what to do? [Completeness,
  Spec §FR-012, §FR-016]

## Licensing, Honesty, and Passivity Constraints

- [ ] CHK020 - Is the npcap license gate stated as a precondition on the fetch
  action, with the safe default (degrade to opening the page) and the
  stop-and-ask-on-ambiguity path? [Completeness, Spec §FR-013]
- [ ] CHK021 - Is the requirement to record the license determination in a
  changelog.d decisions fragment traceable to a measurable outcome? [Measurability,
  Spec §FR-013, §SC-005]
- [ ] CHK022 - Is "fragcap never bundles, vendors, or hosts npcap" restated for the
  fetch action (Licensing non-negotiable)? [Consistency, Spec §FR-012]
- [ ] CHK023 - Is a failed action required to be reported as failed and never as
  success (P-9), and is action-outcome reporting defined for all outcomes
  (performed/skipped/degraded/failed)? [Coverage, Spec §FR-011, Key Entities]
- [ ] CHK024 - Is the P-1 boundary stated: --fix changes only the local environment
  through named confirmed actions, never traffic and never a target process?
  [Clarity, Spec Assumptions]

## Testability and House Non-Negotiables

- [ ] CHK025 - Is the split between Tier 1 (decision/selection/refusal logic, in CI)
  and Tier 2 (platform side effects, not in CI) specified so nothing platform-bound
  is silently skipped? [Coverage, Spec §FR-017, §SC-006, Assumptions]
- [ ] CHK026 - Are new terms introduced by the slice required to get glossary
  entries in the same change (P-6)? [Completeness, Spec §FR-018]
- [ ] CHK027 - Are the house non-negotiables (UTF-8 without BOM, LF endings, no
  em/en dashes) applicable to every artifact this slice adds, and is that
  expectation captured? [Consistency, Gap]

## Acceptance Criteria Quality

- [ ] CHK028 - Are all Success Criteria technology-agnostic and objectively
  verifiable (no reliance on a specific capture driver, elevation, or network to
  evaluate)? [Measurability, Spec §SC-001..§SC-006]
- [ ] CHK029 - Does every functional requirement have at least one acceptance
  scenario or success criterion tracing to it? [Traceability, Spec §US1..§US3,
  §SC-001..§SC-006]

## Notes

- Items are requirement-quality questions, not implementation tests. An unchecked
  item means the spec text needs sharpening before planning, not that code is
  wrong.
