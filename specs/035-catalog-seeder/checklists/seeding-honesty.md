# Seeding Honesty Checklist: Tier 1 Catalog Seeder

**Purpose**: Validate that the requirements for truthful accounting (P-4/P-9),
non-destructive tier merging, and resumability are complete, unambiguous, and
testable before planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Truthful accounting (P-4/P-9)

- [x] CHK001 Are the seed-summary categories (fetched, written, excluded, failed) enumerated so that every title lands in exactly one? [Completeness, Spec §FR-005]
- [x] CHK002 Is a conservation rule stated (the category counts reconcile to the source total, so no title is silently lost) as a testable requirement? [Measurability, Spec §SC-002]
- [x] CHK003 Is "excluded by the gate" required to be surfaced as a count rather than a silent omission? [Clarity, Spec §FR-005]
- [x] CHK004 Is the requirement that a single malformed or unfetchable title does not abort the run stated, with that title counted as failed? [Coverage, Exception Flow, Spec §FR-006]
- [x] CHK005 Is it specified that every written row remains fidelity heuristic-unverified? [Consistency, Spec §FR-010]

## Non-destructive tier merge

- [x] CHK006 Is the per-tier merge specified to write only the Tier 1 columns for an application id, leaving launch entries, engine, and technologies intact? [Clarity, Spec §FR-007]
- [x] CHK007 Is the distinction between the Tier 1 merge and the foundation's whole-game replace made explicit, so the seeder does not accidentally use the clobbering path? [Consistency, Spec §FR-007]
- [x] CHK008 Is there an acceptance scenario proving the non-clobber property (seed Tier 1 over a game that already has an engine; the engine survives)? [Coverage, Spec §US3]
- [x] CHK009 Is the no-prune rule stated (a stored title absent from a run is left unchanged; the seeder only inserts and updates)? [Completeness, Spec §FR-009]

## Resumability

- [x] CHK010 Is resumability defined as continuing from a recorded position rather than restarting, with the resume state named (the catalog tier's seed state)? [Clarity, Spec §FR-008]
- [x] CHK011 Is the equivalence between a resumed multi-part seed and a single uninterrupted seed stated as a measurable outcome (same final corpus, no duplicated rows)? [Measurability, Spec §SC-004]
- [x] CHK012 Is the behavior after a mid-run fetch failure specified (already-written rows and the resume point survive)? [Edge Case, Recovery, Spec §Edge Cases]

## Corpus gate & naming

- [x] CHK013 Is the corpus gate defined by explicit criteria (game classification plus a configurable review-count threshold with a documented default)? [Clarity, Spec §FR-004]
- [x] CHK014 Is the nameless-title behavior pinned to a single outcome (written with name omitted, never an empty string, counted as written) rather than left as a choice? [Ambiguity resolved, Spec §FR-013]
- [x] CHK015 Is idempotency-per-appid within one run specified (a title appearing twice is written once, not double-counted misleadingly)? [Edge Case, Spec §Edge Cases]

## Notes

- Check items off as completed: `[x]`
- These items test whether the REQUIREMENTS are well-written, not whether code works.
- P-4 (No Silent Loss) and P-9 (The Instrument Does Not Lie) govern this checklist.
