# Detection Truthfulness Checklist: S065

**Purpose**: Validate that the S065 requirements are complete, unambiguous, and
consistent before implementation, with particular attention to the constitution
principles this slice exists to serve (P-9 truthfulness, P-4 no silent loss,
P-1 passive observation) and to the two-surface rule the last campaign slice
failed on.

**Created**: 2026-08-20

**Feature**: [spec.md](../spec.md)

**Depth**: release gate. **Audience**: the reviewer of the pull request, and the
author before the pre-push halt.

## P-9: the observation recorded is the observation made

- [x] CHK001 Is the discriminator for the DRM wrapper named precisely enough
      that a reader can tell a true positive from a false one without guessing?
      [Clarity, Spec FR-003]
- [x] CHK002 Is the basis for the `.bind` claim recorded as measurement rather
      than as inference, with the sample it rests on? [Traceability, Spec
      Assumptions]
- [x] CHK003 Does the spec state what happens when a marker cannot be read, as
      distinct from a marker that is absent? [Gap, Spec Edge Cases]
- [x] CHK004 Are the confidence tiers of the new engine signatures justified,
      so a corroborating signal is not stamped definitive? [Clarity, Spec
      FR-008, FR-009]
- [x] CHK005 Is it specified that a target registered before this change reads
      as never scanned rather than as scanned clean? [Completeness, Spec FR-015]
- [x] CHK006 Is the removal of the Steamworks SDK rows stated as a removal, not
      left as a choice for the implementer? [Clarity, Spec FR-001]

## P-4: the accounting sums

- [x] CHK007 Is the invariant that applied plus inert plus skipped equals loaded
      restated as a requirement, not assumed from the existing code?
      [Completeness, Spec FR-006]
- [x] CHK008 Is the disposition of an unrecognized binary-marker pattern form
      specified exactly (inert, not skipped, not dropped)? [Clarity, Spec
      FR-006]
- [x] CHK009 Are the bounds on the section-table scan specified in both
      dimensions the spec claims (depth and candidate count)? [Completeness,
      Spec FR-004]
- [x] CHK010 Is the count of candidates excluded by a bound required to be
      reachable by a caller, rather than merely computed? [Measurability, Spec
      FR-005]
- [x] CHK011 Are the three parked byte-marker rows explicitly declared out of
      scope with a reason, so their inertness is a decision rather than an
      oversight? [Assumption, Spec Assumptions]
- [x] CHK012 Does the spec forbid silent truncation of a rendered value in terms
      that a reviewer can check against the renderer? [Measurability, Spec
      FR-017]

## P-1: the reader stays passive

- [x] CHK013 Is the section reader constrained to file bytes already on disk,
      with process handles, process memory, and inspection APIs each named as
      forbidden? [Completeness, Spec FR-007]
- [x] CHK014 Is the bounded-prefix read specified, so a large executable is not
      read whole? [Clarity, Spec FR-004]
- [x] CHK015 Is the behavior for a file that is named like an executable but is
      not one specified as a non-event rather than an error? [Edge Case, Spec
      Edge Cases]

## The dual-detector decision

- [x] CHK016 Is the chosen option stated with the rejected option and the reason
      for rejection, so the decision survives without this conversation?
      [Traceability, Spec Clarifications, FR-010]
- [x] CHK017 Is the invariant stated as directed (one set is a subset of the
      other) rather than as a two-way equality that the code cannot honor?
      [Consistency, Spec FR-011]
- [x] CHK018 Does the check requirement forbid a hand-maintained list and a
      literal count, which is the failure mode a prior slice shipped?
      [Measurability, Spec FR-011]
- [x] CHK019 Is it clear which side of the invariant a new engine must be added
      to first, so a future contributor knows what the check is telling them?
      [Clarity, Spec FR-011]

## Three-state coverage plumbing

- [x] CHK020 Are all three coverage states named and distinguished from one
      another in requirement text, not only in the narrative? [Completeness,
      Spec FR-014]
- [x] CHK021 Is it specified that every source that can produce a target records
      the state, so one unplumbed source cannot leave rows silently blank?
      [Coverage, Spec FR-015]
- [x] CHK022 Is the meaning of an absent value pinned (never scanned), rather
      than left to the reader? [Ambiguity, Spec FR-015]
- [x] CHK023 Is the behavior on opening a store written by an earlier build
      specified? [Edge Case, Spec FR-015]
- [x] CHK024 Is rejection of an out-of-set stored value required, rather than a
      permissive fallback? [Clarity, Spec FR-015]

## Two surfaces, one answer

- [x] CHK025 Does the spec name both surfaces the split must appear on, so
      implementing one and claiming both is not possible? [Completeness, Spec
      FR-016]
- [x] CHK026 Is the machine-readable surface required to carry the coverage
      state as well as the category partition? [Coverage, Spec FR-016]
- [x] CHK027 Is a round trip through export and import required, so the machine
      surface is not write-only? [Measurability, Spec FR-015]

## Rendering and width

- [x] CHK028 Is the width rule stated as a decision with a rejected alternative,
      rather than as a target to aim at? [Clarity, Spec FR-017,
      Clarifications]
- [x] CHK029 Is the representative row defined concretely enough to build a test
      from it? [Measurability, Spec FR-017, SC-006]
- [x] CHK030 Is the retirement of the readiness fallback sentences reconciled
      with #174's own wording, which proposed moving them? [Conflict, Spec
      FR-013, FR-018, Clarifications]

## Scope boundaries

- [x] CHK031 Are the adjacent issues that are deliberately not in this slice
      named, so a reviewer does not read their absence as an omission? [Scope,
      Spec Assumptions]
- [x] CHK032 Do the success criteria include at least one that is checkable
      without the operator's machine, so the slice is not gated on hardware
      nobody else has? [Measurability, Spec SC-005, SC-006, SC-007]

## Notes

- Check items off as completed: `[x]`
- CHK018, CHK021, and CHK025 exist because those three failure modes each
  shipped in a slice of this campaign and were caught by a reviewer rather than
  by the slice's own tests.
