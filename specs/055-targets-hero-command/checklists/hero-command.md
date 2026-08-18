# Checklist: Targets Hero Command and Interactive Authoring

**Purpose**: Validate the quality, clarity, and completeness of the S055
requirements before planning. These are unit tests for the spec's English, not
for the implementation.
**Created**: 2026-08-18
**Feature**: [spec.md](../spec.md)

## Hero Acceptance Criteria (the five §9.5 tests)

- [ ] CHK001 - Is "runs successfully on a fresh install with no arguments and no prior config" expressed as an objectively checkable requirement rather than an aspiration? [Measurability, Spec §FR-001, SC-001]
- [ ] CHK002 - Is "operates on the user's own software rather than an example" stated as a requirement that a test could distinguish from a demo/fixture path? [Clarity, Spec §US1, SC-001]
- [ ] CHK003 - Is "demonstrates attribution (the core value proposition)" tied to a concrete listed-row-leads-to-a-named-process outcome rather than left abstract? [Clarity, Spec §SC-002]
- [ ] CHK004 - Is "completes in a few seconds" quantified or bounded enough to be verifiable, or is "a few seconds" flagged as an accepted qualitative target? [Ambiguity, Spec §FR-007, SC-001]
- [ ] CHK005 - Is "ends by naming the next command to run" specified for every listing path (populated and empty)? [Completeness, Spec §FR-006]
- [ ] CHK006 - Are all five hero criteria individually traceable to at least one functional requirement or success criterion? [Traceability, Spec §Success Criteria]

## Row-Index Listing Snapshot and capture <n>

- [ ] CHK007 - Is it unambiguous which invocations write the listing snapshot (bare `fragcap`, `fragcap targets`, `targets list`)? [Clarity, Spec §FR-004]
- [ ] CHK008 - Is the resolution contract for a row-index selector defined against the snapshot rather than the live ordering? [Consistency, Spec §FR-004, Clarifications]
- [ ] CHK009 - Is the behavior of a row index beyond the snapshot's range specified (out-of-range usage error), and distinguished from a clean handle/name no-match? [Edge Case, Spec §FR-005]
- [ ] CHK010 - Is the interaction with the already-shipped S054 `capture <n>` behavior documented so the change is intentional, not silent? [Consistency, Spec §Clarifications, §Dependencies]
- [ ] CHK011 - Is the snapshot's lifetime/replacement rule stated (a new listing replaces the prior snapshot; `--id` is unaffected)? [Completeness, Spec §Assumptions]
- [ ] CHK012 - Is deterministic ordering ("by handle") defined precisely enough that two runs over the same store produce identical numbering? [Measurability, Spec §FR-003]

## Interactive Add and the Y/n/unsure Honesty Posture

- [ ] CHK013 - Is the `unsure` branch specified as a required, first-class outcome rather than an implied fallback? [Completeness, Spec §FR-011]
- [ ] CHK014 - Is "never guess the socket holder and present the guess as fact" stated as a testable no-fabrication requirement? [Clarity, Spec §FR-012]
- [ ] CHK015 - Is the post-`unsure` promotion-to-`verified` requirement defined, including the case where a capture never observes a holder (row stays unresolved)? [Coverage, Spec §FR-013, Edge Cases]
- [ ] CHK016 - Are the outcomes of `Y` and `n` answers each specified (resolved launch chain), not just the `unsure` path? [Completeness, Spec §FR-011]
- [ ] CHK017 - Is the ordering requirement (scan evidence shown inline before dependent prompts) specified? [Clarity, Spec §FR-009]
- [ ] CHK018 - Is the derived-default-handle-with-disambiguation behavior specified so a collision cannot overwrite an existing target? [Consistency, Spec §FR-010]
- [ ] CHK019 - Is "browse for an executable" defined well enough to be unambiguous about what interaction is expected? [Ambiguity, Spec §Assumptions]

## Non-Interactive Fallback (CI testability)

- [ ] CHK020 - Is the terminal-vs-not-terminal branch explicitly specified as the trigger for interactive vs flag-driven authoring? [Clarity, Spec §FR-015, Clarifications]
- [ ] CHK021 - Is a required-but-missing value under the non-interactive path specified as a usage error rather than a blocking prompt? [Edge Case, Spec §FR-015]
- [ ] CHK022 - Are the `Y`/`n`/`unsure` outcomes reachable and assertable without a live terminal, so each is independently testable? [Coverage, Spec §US2 Independent Test]

## Export / Import Round-Trip Integrity

- [ ] CHK023 - Is the export document shape defined unambiguously (single JSON array of schema-conforming target objects for one and for many)? [Clarity, Spec §FR-018, Clarifications]
- [ ] CHK024 - Is merge-on-stable-identifier specified for import, including the update-in-place-vs-duplicate outcome? [Completeness, Spec §FR-019]
- [ ] CHK025 - Is round-trip identity ("identical identifiers, no duplicate rows") stated as a measurable success criterion? [Measurability, Spec §FR-020, SC-005]
- [ ] CHK026 - Is the behavior for a non-conforming import file specified (rejected with diagnostics, not partially applied)? [Edge Case, Spec §FR-019]
- [ ] CHK027 - Is the master target schema referenced by export/import identified as the existing published schema (no schema change in this slice)? [Assumption, Spec §Assumptions]

## Empty Case and Actionable Next Commands

- [ ] CHK028 - Is the empty case (no targets AND discovery finds nothing) distinguished from a non-empty listing, with its own required output? [Coverage, Spec §FR-006, Edge Cases]
- [ ] CHK029 - Is "print the commands that fix that" specified concretely enough (which commands: add, scan) to be verifiable? [Clarity, Spec §FR-006]
- [ ] CHK030 - Does the empty case still satisfy hero criterion 5 (ends by naming a next command) per an explicit requirement? [Traceability, Spec §SC-006]

## CAPTURE / KNOWN Column Semantics

- [ ] CHK031 - Is the CAPTURE status vocabulary bounded and defined (`ready` vs `needs a target`), with the derivation rule stated? [Clarity, Spec §FR-002, §Key Entities]
- [ ] CHK032 - Is the KNOWN column constrained to neutral evidence, with an explicit requirement that it not read as a blocker or an endorsement? [Consistency, Spec §FR-021]
- [ ] CHK033 - Is "every listed row is capturable in principle" stated so CAPTURE reports closeness, never validity? [Clarity, Spec §FR-021]

## Selector Robustness Across Lifecycle Commands

- [ ] CHK034 - Is ambiguous-selector behavior on `remove`/`export` specified (list matches, refuse to act) consistently with `show`? [Consistency, Spec §FR-017, Edge Cases]
- [ ] CHK035 - Is `remove`'s isolation requirement ("exactly that target, others untouched") stated measurably? [Measurability, Spec §FR-017]

## Governance Lock-Step (P-6 / P-11)

- [ ] CHK036 - Is the requirement to add a glossary entry for any new term in the same change stated (P-6)? [Completeness, Spec §FR-022]
- [ ] CHK037 - Is the requirement to update any affected master-specification section in lock-step stated (P-11)? [Completeness, Spec §FR-022]
- [ ] CHK038 - Are the non-negotiables that bound this slice (P-1 no-fabrication, P-9 refuse-to-guess) reflected as requirements, not just prose? [Traceability, Spec §FR-012, §FR-017]

## Dependencies & Assumptions

- [ ] CHK039 - Are the S051/S052/S053/S054/fragcap-steam dependencies each named with the specific capability this slice consumes? [Dependency, Spec §Dependencies]
- [ ] CHK040 - Is every Assumption stated as falsifiable (a reviewer could confirm or reject it), rather than open-ended? [Assumption, Spec §Assumptions]
