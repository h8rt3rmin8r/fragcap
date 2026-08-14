# Seeding Honesty Checklist: Tier 3 Engine Seeder

**Purpose**: Validate that the requirements for truthful accounting (P-4/P-9),
non-destructive tier merging, and resumability are complete, unambiguous, and
testable before planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Truthful accounting (P-4/P-9)

- [x] CHK001 Are the seed-summary categories (fetched, written, excluded, duplicates, failed) enumerated so that every title lands in exactly one? [Completeness, Spec §FR-005]
- [x] CHK002 Is a conservation rule stated (the category counts reconcile to the source total, so no title is silently lost) as a testable requirement? [Measurability, Spec §SC-002]
- [x] CHK003 Is "no or ambiguous engine" required to be surfaced as an excluded count rather than a silent omission? [Clarity, Spec §FR-004, §FR-005]
- [x] CHK004 Is the requirement that a single malformed or unfetchable title does not abort the run stated, with that title counted as failed? [Coverage, Exception Flow, Spec §FR-006]
- [x] CHK005 Is it specified that every written row remains fidelity heuristic-unverified and that an engine seed never changes a record's fidelity? [Consistency, Spec §FR-010]

## Non-destructive tier merge

- [x] CHK006 Is the per-tier engine merge specified to write only the engine columns for an application id, leaving name, catalog metrics, launcher flags, launch entries, and technologies intact? [Clarity, Spec §FR-007]
- [x] CHK007 Is the distinction between the engine merge and both the foundation's whole-game replace and the Tier 1 catalog merge made explicit, so the seeder writes engine without clobbering or omitting? [Consistency, Spec §FR-007]
- [x] CHK008 Is there an acceptance scenario proving the non-clobber property (seed engine over a catalog-seeded, launch-bearing game; the name and launch entries survive)? [Coverage, Spec §US2, §SC-003]
- [x] CHK009 Is the no-prune rule stated (a stored title absent from a run is left unchanged; the seeder only inserts and updates)? [Completeness, Spec §FR-009]
- [x] CHK010 Is source-and-confidence written together (the both-or-neither invariant) specified so a half-present engine cannot be stored? [Clarity, Spec §FR-007]

## Resumability

- [x] CHK011 Is resumability defined as continuing from a recorded position rather than restarting, with the resume state named (the engine tier's seed state)? [Clarity, Spec §FR-008]
- [x] CHK012 Is the equivalence between a resumed multi-part seed and a single uninterrupted seed stated as a measurable outcome (same final result, no duplicated rows)? [Measurability, Spec §SC-004]
- [x] CHK013 Is the behavior after a mid-run fetch failure specified (already-written engines and the resume point survive)? [Edge Case, Recovery, Spec §Edge Cases]

## Keep/exclude decision & naming

- [x] CHK014 Is the write-vs-exclude decision defined by an explicit criterion (write iff a single unambiguous engine name resolves; else exclude)? [Clarity, Spec §FR-001, §FR-004]
- [x] CHK015 Is the never-guess rule pinned to a single outcome (a missing or ambiguous engine leaves the columns unset, not a placeholder) rather than left as a choice? [Ambiguity resolved, Spec §FR-004]
- [x] CHK016 Is idempotency-per-appid within one run specified (a title appearing twice with an engine is written once, the repeat counted as a duplicate)? [Edge Case, Spec §Edge Cases]
- [x] CHK017 Is the engine-only-row case specified (an engine for an appid the store has not seen is written and is schema-valid on export)? [Coverage, Spec §FR-007, §Edge Cases]

## Notes

- Check items off as completed: `[x]`
- These items test whether the REQUIREMENTS are well-written, not whether code works.
- P-4 (No Silent Loss) and P-9 (The Instrument Does Not Lie) govern this checklist.
