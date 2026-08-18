# Checklist: Docs convergence and first-run quality (S057)

**Purpose**: Unit-test the requirements for the S057 landing/getting-started rewrite and site-wide docs convergence before implementation.
**Created**: 2026-08-18
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Are the exact retired tokens the slice must eliminate enumerated (verbs, commands, selector, directory, slugs) so "no page references them" is objectively checkable? [Completeness, Spec §FR-016, SC-002]
- [ ] CHK002 - Is the full set of documentation surfaces in scope enumerated (site MDX pages, landing page, brand page, doctor sample) rather than only "getting started"? [Completeness, Spec §US1/US2/US3]
- [ ] CHK003 - Are requirements defined for the non-Steam / manually-installed game path, not only the Steam-App-ID path? [Coverage, Spec Edge Cases, §FR-010]
- [ ] CHK004 - Does the spec state what replaces each removed link target (landing capability bullet, index Guides list, architecture link) so no dangling reference remains? [Completeness, Spec §FR-015, Clarifications]
- [ ] CHK005 - Are the QA issues #130, #131, #132, #133 (docs half), #134, #135 each mapped to a concrete functional requirement? [Traceability, Spec §FR-007..FR-012, SC-006]

## Requirement Clarity

- [ ] CHK006 - Is the landing page's "primary persuasive asset" specified concretely (a `fragcap targets` listing with defined columns and the capture hint) rather than as an adjective? [Clarity, Spec §FR-002, Clarifications]
- [ ] CHK007 - Is "ends with a capture file on disk" defined as an observable endpoint (a named `.fcapng` produced by `fragcap capture <n>`)? [Clarity/Measurability, Spec §FR-006, SC-001]
- [ ] CHK008 - Is the corrected doctor identity row set stated exactly (version, binary, catalog db, local db) so the sample and the binary can be compared? [Clarity, Spec §FR-017, FR-019]
- [ ] CHK009 - Is the coherent npcap narrative (prerequisite + installer exit dialog + `doctor --fix` action) specified precisely enough to write one non-contradictory story? [Clarity, Spec §FR-012]

## Requirement Consistency

- [ ] CHK010 - Do the getting-started and landing requirements use the same current command surface (`fragcap targets`, `fragcap capture`) with no residual `run`/`--profile`? [Consistency, Spec §FR-006, FR-002]
- [ ] CHK011 - Is the CLI reference requirement bound to a single source of truth (the clap grammar in cli.rs) so the doc cannot drift from the binary? [Consistency, Spec §FR-013]
- [ ] CHK012 - Is the npcap posture in the docs consistent with the constitution's amended Licensing rule 2 (detection-only, plus user-confirmed vendor-installer fetch)? [Consistency, Spec §FR-012]

## Acceptance Criteria Quality

- [ ] CHK013 - Is every acceptance criterion objectively verifiable (grep result, site build, doctor row set, ci exit code) rather than subjective? [Measurability, Spec §SC-001..SC-006]
- [ ] CHK014 - Is the doctor companion change's non-regression stated as a checkable invariant (exit status and all other rows unchanged)? [Measurability, Spec §FR-018, US4]

## Scenario & Edge Coverage

- [ ] CHK015 - Are requirements defined for a reader who already has npcap/Wireshark (skip-the-walkthrough path)? [Coverage, Spec Edge Cases, §FR-007]
- [ ] CHK016 - Is the `doctor --json` output covered by the profile-row removal, not only the human report? [Coverage, Spec Edge Cases, §FR-017]
- [ ] CHK017 - Does the spec address the glossary index reproducibility / dangling cross-link risk from removing two pages? [Edge Case, Spec Edge Cases, §FR-021]

## Dependencies & Assumptions

- [ ] CHK018 - Is the retention of `reference/target-schema.mdx` (and the internal Profile type) stated so the removal is scoped to the stale subset only? [Assumption, Spec Assumptions]
- [ ] CHK019 - Is the IGDB deferral documented with its rationale (no plumbing exists; P-11) rather than silently dropped? [Assumption, Spec Assumptions, Out of scope]
- [ ] CHK020 - Is the dependency on S056 (the `doctor --fix` npcap behavior the #133 narrative reconciles against) recorded? [Dependency, Spec Dependencies]

## Governance & Completion

- [ ] CHK021 - Does the spec require P-6 glossary discipline for any new term the rewrites introduce? [Governance, Spec §FR-021]
- [ ] CHK022 - Is the pinned-artifact discipline respected (no edit to scripts/**, workflows, without a dated decision) given the linter script is pinned? [Governance, Constitution non-negotiables]
- [ ] CHK023 - Is "cargo xtask ci green" plus "docs linter passes" stated as a hard completion gate? [Completion, Spec §FR-020, SC-005]

## Notes

- Items are requirements-quality tests, not implementation tests; each is resolved by confirming the spec says enough, not by running the tool.
- CHK022 flags that `scripts/lint-docs.sh` is a pinned artifact: S057 must not modify it, so no dated CHANGELOG decision is needed for it.
