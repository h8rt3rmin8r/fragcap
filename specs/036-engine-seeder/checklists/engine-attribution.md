# Engine Attribution Checklist: Tier 3 Engine Seeder

**Purpose**: Validate that the P-9 requirements specific to engine attribution
(confidence as a within-field grade, not a fidelity tier; PCGamingWiki as a named
provenance; the field-typing discipline) are complete and testable before
planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Confidence is a within-field grade, not a fidelity tier (P-9)

- [x] CHK001 Is the engine confidence explicitly required to be a within-field quality grade of one heuristic field, and explicitly NOT a fifth fidelity tier? [Clarity, Spec §Overview, §FR-010]
- [x] CHK002 Is it stated that a low or unknown confidence does not lower the record's overall trust, which stays heuristic-unverified? [Consistency, Spec §Overview, §FR-010]
- [x] CHK003 Is the confidence value constrained to the schema's closed set of tokens (an out-of-set token is a failure, not a silent downgrade)? [Measurability, Spec §FR-013, §Edge Cases]

## Provenance is named honestly

- [x] CHK004 Is `engine_source` required to be "pcgamingwiki" for every row this seeder writes, distinct from the record-level provenance? [Clarity, Spec §FR-001]
- [x] CHK005 Is the live-source flag / naming required to name PCGamingWiki rather than misattribute the data to Steam (the tier is keyed by Steam appid but sourced from PCGamingWiki)? [Ambiguity resolved, Spec §Assumptions; Plan §Key Design Decisions]

## Field-typing discipline (no coercion)

- [x] CHK006 Is a present-but-wrong-typed field (appid or confidence of the wrong JSON type) required to be counted failed, never coerced to a default? [Clarity, Spec §FR-013]
- [x] CHK007 Is the difference between "failed" (unparsable) and "excluded" (no/ambiguous engine) drawn so the summary does not misattribute why an engine is absent? [Consistency, Spec §FR-004, §FR-005, §FR-013]
- [x] CHK008 Is an acceptance scenario present that drives a malformed entry among good ones and asserts it is counted failed while the good titles are written? [Coverage, Spec §US1 scenario 2]

## Absence is honest

- [x] CHK009 Is "a missing or ambiguous engine is left absent, never guessed" stated as a hard requirement rather than a preference? [Clarity, Spec §FR-004]
- [x] CHK010 Is the export of an engine-bearing row required to carry the engine name, source, and confidence, with the record still heuristic-unverified? [Coverage, Spec §US1 scenario 3, §FR-010]

## Notes

- Check items off as completed: `[x]`
- These items test whether the REQUIREMENTS are well-written, not whether code works.
- P-9 (The Instrument Does Not Lie) governs this checklist; it is the slice's
  sharpest edge because engine attribution is community data, never verified.
