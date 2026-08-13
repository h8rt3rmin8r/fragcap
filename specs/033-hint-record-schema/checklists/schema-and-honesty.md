# Checklist: Schema Correctness and Honesty Requirements Quality

**Purpose**: Validate that the requirements for the hint-record schema revision's
correctness and honesty edges are complete, clear, consistent, and measurable
before planning. Requirements-quality review, not an implementation test.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)
**Focus**: additive v1 extension, strict-vs-loose boundary, P-9 vocabulary
reconciliation, closed-property discipline, launch-array-never-flattened.

## Additive, Backward-Compatible Extension

- [x] CHK001 - Is "additive, no version bump" stated as a requirement, with every
  pre-existing artifact required to still validate? [Completeness, Spec §FR-007]
- [x] CHK002 - Is the two-copy byte-identical invariant (embedded + published,
  drift check green) specified? [Consistency, Spec §FR-007, §SC-004]
- [x] CHK003 - Is each new field required to be optional, so a minimal hint that
  omits all three still validates? [Clarity, Spec §Edge Cases, §FR-002..004]

## Strict-vs-Loose Boundary

- [x] CHK004 - Is it specified exactly which variants carry the three fields (hint
  top-level and export records) and which do not (strict profile/package, export
  envelope top-level)? [Completeness, Spec §FR-006, §Clarifications]
- [x] CHK005 - Is a strict profile/package carrying any new field required to be
  rejected (not silently accepted)? [Clarity, Spec §FR-006, §US3]
- [x] CHK006 - Is the rationale for the boundary stated (hint-seeding metadata vs
  authored capture format), so it is not read as arbitrary? [Consistency, Spec
  §Overview, §US3]

## P-9 Vocabulary Reconciliation

- [x] CHK007 - Is the record `fidelity` enum required to be unchanged, and the
  engine `confidence` a separate field never added to or required to match it?
  [Clarity, Spec §FR-005, §SC-005]
- [x] CHK008 - Is the engine `source` distinguished from the record's provenance
  `source` (same field name, different vocabulary, no cross-constraint)?
  [Consistency, Spec §FR-005, §Edge Cases]
- [x] CHK009 - Is "engine confidence is a within-tier gradation, not a fifth
  fidelity tier" stated so a low-confidence engine cannot silently move record
  trust? [Ambiguity, Spec §Overview, §Clarifications]
- [x] CHK010 - Is a failed engine lookup required to leave the engine absent
  rather than present-but-lying? [Clarity, Spec §Clarifications, §Edge Cases]

## Closed-Property Discipline

- [x] CHK011 - Is each new object required to reject unknown keys (closed property
  set preserved)? [Completeness, Spec §FR-001, §FR-004, §FR-008]
- [x] CHK012 - Is an out-of-enum engine `source` or `confidence` required to be
  rejected with a named diagnostic? [Measurability, Spec §FR-008, §SC-002]
- [x] CHK013 - Is a launch entry missing the required `executable` required to be
  rejected with a named diagnostic? [Measurability, Spec §FR-001, §FR-008, §SC-002]
- [x] CHK014 - Are the filter fields (`os`/`osarch`/`launch_type`/`beta_branch`)
  specified as free strings rather than enums, with the reason (external evolving
  vocabularies)? [Clarity, Spec §Clarifications, §FR-001]

## Launch Array Never Flattened

- [x] CHK015 - Is the launch data required to stay an array and never be reduced
  to a single "process name" at seeding time? [Clarity, Spec §FR-002, §Overview]
- [x] CHK016 - Is it specified that reducing the array to the socket holder is the
  resolver's (#77) runtime job, not this schema's? [Consistency, Spec §FR-002,
  §Overview]
- [x] CHK017 - Is an empty launch array required to validate (found none vs never
  looked up), distinct from an absent array? [Edge Case, Spec §Edge Cases]

## Coverage and Scope

- [x] CHK018 - Are the required conformance fixtures enumerated (valid full hint;
  out-of-enum source; out-of-enum confidence; missing executable; strict-variant
  rejection; pre-existing fixtures unchanged)? [Completeness, Spec §FR-009, §SC-003]
- [x] CHK019 - Is the out-of-scope boundary explicit (no SQLite, no seeding, no
  external fetch, all #78)? [Assumption, Spec §Overview, §Assumptions]
- [x] CHK020 - Is "no new dependency, MSRV green, UTF-8/LF/no-dashes" stated as a
  hard requirement? [Completeness, Spec §FR-011, §SC-006]

## Notes

- All items pass against the current spec; the operator's focus areas map to
  explicit FRs, clarifications, edge cases, and success criteria. Retained as the
  reviewable record that the honesty and schema-correctness edges were specified,
  not assumed.
