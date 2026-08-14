# Schema & Honesty Checklist: Targets Hint Database (foundation)

**Purpose**: Validate that the requirements for schema conformance and observational
honesty (P-9) are complete, unambiguous, and consistent before planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Export / Schema Conformance

- [x] CHK001 Is the exact output shape (a single `export` envelope with a records array, one record per game) specified rather than left open? [Clarity, Spec §FR-008]
- [x] CHK002 Are the required fields of each exported record enumerated (fidelity, provenance source, identity) with the conditional fields (launch, launcher_mediated, engine) distinguished from them? [Completeness, Spec §FR-008]
- [x] CHK003 Is "validates against the published hint-record subschema" stated as a binding requirement with a named validator and a zero-diagnostics acceptance bar? [Measurability, Spec §FR-009, SC-002]
- [x] CHK004 Is validity-by-construction required (the exporter validates its own output and never returns a rejected document), not merely tested after the fact? [Clarity, Spec §FR-009]
- [x] CHK005 Is the behavior for an unknown engine specified as omission of the engine object (not null, not placeholder)? [Edge Case, Spec §FR-008]
- [x] CHK006 Is the representation of an empty launch collection in the export specified (omission vs empty array), and the choice justified? [Ambiguity, Spec Assumptions]

## Honesty (P-9)

- [x] CHK007 Does the spec state that every exported record carries fidelity heuristic-unverified regardless of engine confidence? [Consistency, Spec §FR-010]
- [x] CHK008 Is it explicit that engine confidence grades only the engine field and is never a fifth fidelity tier? [Clarity, Spec §FR-010, Clarifications]
- [x] CHK009 Is the requirement that the launch array is persisted and exported whole (never flattened to a single process name) stated as a hard constraint? [Completeness, Spec §FR-003, Clarifications]
- [x] CHK010 Are out-of-set engine source/confidence values required to be refused at write time (not coerced or defaulted), so no invalid record can ever be stored? [Clarity, Spec §FR-006]
- [x] CHK011 Does a malformed import fail loudly with no partial store, rather than silently dropping or normalizing the offending record? [Consistency with P-4/P-9, Spec §FR-012, §FR-015]
- [x] CHK012 Is the provenance source that the export stamps defined (a single database-origin identifier this slice), avoiding an undefined or implied value? [Gap, Spec Assumptions]

## Consistency & Traceability

- [x] CHK013 Do the enum sets for engine source and confidence in the spec match the published schema's sets exactly (no drift, no extra/missing member)? [Consistency, Spec §FR-006]
- [x] CHK014 Are the launch-entry filter fields (os, osarch, launch type, beta branch) and the required executable named consistently between the store model and the schema shape? [Consistency, Spec §FR-003]
- [x] CHK015 Is there a requirement that a conformance fixture proves both a valid export passes and a malformed one (bad engine source, launch entry missing executable) is rejected with the right code? [Coverage, Spec §DONE WHEN, SC-004]

## Notes

- Check items off as completed: `[x]`
- These items test whether the REQUIREMENTS are well-written, not whether code works.
- P-9 (The Instrument Does Not Lie) and P-4 (No Silent Loss) are the governing principles behind the Honesty section.
