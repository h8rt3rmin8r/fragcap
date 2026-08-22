# Specification Quality Checklist: Identity join & JSON output consistency

**Purpose**: Validate requirements quality for the three-state identity model,
the read-only snapshot constraint, and deterministic ordering, ahead of
planning
**Created**: 2026-08-22
**Feature**: [spec.md](../spec.md)
**Depth**: Standard
**Audience**: Reviewer (PR)

## Requirement Completeness

- [x] CHK001 - Are all three identity states (registered+positioned,
  registered-only, unregistered) explicitly enumerated with distinguishing
  rules, rather than left to be inferred from examples? [Completeness, Spec
  §Key Entities, §FR-003-FR-005]
- [x] CHK002 - Is the behavior when the local store cannot be opened at all
  specified separately from the behavior when a title simply has no
  registration? [Completeness, Spec §FR-008]
- [x] CHK003 - Are requirements defined for the case where a store query
  succeeds for other titles but fails unexpectedly for one specific title
  (not simply "not found")? [Completeness, Spec §Edge Cases]

## Requirement Clarity

- [x] CHK004 - Is "visibly distinct" for the three identity states quantified
  well enough that a reviewer could reject an implementation that renders two
  states identically? [Clarity, Spec §FR-004, §FR-005]
- [x] CHK005 - Is the sort key ("by title name") precise about case
  sensitivity and the tiebreak field, rather than left as an informal
  preference? [Clarity, Spec §FR-007]
- [x] CHK006 - Is "absent" for a JSON field distinguished from "present but
  empty/zero" with a rule precise enough to drive a serialization decision?
  [Clarity, Spec §FR-011]

## Requirement Consistency

- [x] CHK007 - Do the three-state identity rules for the human table (FR-003
  through FR-005) and for the JSON record (FR-011) reference the same
  underlying state definitions rather than two independently described
  models? [Consistency, Spec §FR-003-FR-005, §FR-011]
- [x] CHK008 - Is the read-only constraint on the listing snapshot (FR-006)
  stated once and referenced consistently everywhere the snapshot is
  discussed, rather than restated with different scope in the edge cases or
  success criteria? [Consistency, Spec §FR-006, §SC-003]
- [x] CHK009 - Are the JSON-mode exit-code and enumeration-warning
  requirements (FR-013, FR-014) consistent with the equivalent human-mode
  requirements, with no unstated behavioral divergence between modes?
  [Consistency, Spec §FR-013, §FR-014]

## Acceptance Criteria Quality

- [x] CHK010 - Can "same order across repeated runs" (FR-007, SC-005) be
  objectively verified without relying on implementation knowledge of how the
  sort was performed? [Measurability, Spec §FR-007, §SC-005]
- [x] CHK011 - Can "never rewritten" for the snapshot table (FR-006, SC-003)
  be verified by an observable effect (what `capture <n>` resolves to) rather
  than by inspecting internal state? [Measurability, Spec §FR-006, §SC-003]

## Scenario Coverage

- [x] CHK012 - Are requirements defined for the zero-installed-titles case in
  both human and JSON mode, including the difference in what each mode emits?
  [Coverage, Spec §Edge Cases, §FR-012]
- [x] CHK013 - Are requirements defined for a listing snapshot that exists but
  is empty (no prior `targets` run), distinguishing it from a snapshot that
  was never created? [Coverage, Spec §Edge Cases]

## Dependencies & Assumptions

- [x] CHK014 - Is the assumption that `steam list` reuses the existing local
  store resolution order (rather than introducing its own `--db` flag)
  explicitly recorded and justified? [Assumption, Spec §Assumptions]
- [x] CHK015 - Is the assumption that the JSON structured form is
  newline-delimited (matching `doctor`) rather than a single array explicitly
  recorded, given the issue text raised both as options? [Assumption, Spec
  §Assumptions]

## Notes

- All items pass on the current spec: the linked issues' own "Open design
  questions" sections already resolved the state model, the read-only
  constraint, and the serialization precedent, and this spec's Assumptions
  section carries those resolutions forward with explicit justification.
- No blocking gaps found; proceeding to `/speckit-plan`.
