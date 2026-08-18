# Checklist: CLI surface rework requirements quality

**Purpose**: Unit-test the requirements for completeness, clarity, and consistency before planning
**Created**: 2026-08-17
**Feature**: [spec.md](../spec.md)
**Focus**: Verb-collapse coverage, removal completeness, namespace-to-store correctness, usage-error semantics, presentation, documentation coherence, constitution touchpoints (P-6, P-10/P-11)

## Verb collapse coverage

- [ ] CHK001 - Are all five section-9.1 captures enumerated in the requirements with a distinct expressible form each? [Completeness, Spec §FR-003, §SC-001]
- [ ] CHK002 - Is each of the five captures traceable to a required test, not just to an acceptance narrative? [Measurability, Spec §SC-001]
- [ ] CHK003 - Is the orthogonality of capture-behaviour flags (mode, wait, launch, ring, bounds, sinks, scoping) to the target input stated as a requirement rather than implied? [Clarity, Spec §FR-003]
- [ ] CHK004 - Are the two target inputs (`--target`, `--process`) defined precisely, including what a selector resolves against and what a raw image name matches? [Clarity, Spec §FR-002]
- [ ] CHK005 - Is the survival of the path-anchor capability (path substring and path regex) on `capture` specified, so no `watch` disambiguation is lost? [Coverage, Spec §FR-004]

## Removal completeness

- [ ] CHK006 - Is the removal of `run`, `tap`, and `watch` stated with the explicit "no aliases, no deprecation shims" constraint? [Clarity, Spec §FR-001]
- [ ] CHK007 - Is the profile-file surface removal enumerated as a coherent set (the `profile` command, the profile directory, `--profile-dir`, the file provider, the `--profile` selector) rather than left partial? [Completeness, Spec §FR-007, §FR-008]
- [ ] CHK008 - Is there a requirement that no stale reference to a removed verb or command remains anywhere in shipped output or docs? [Coverage, Spec §FR-017]
- [ ] CHK009 - Is the retention of `schema validate` (distinct from the retired `profile validate`) specified, with its advanced-docs placement? [Consistency, Spec §FR-009]
- [ ] CHK010 - Are the retired verbs and the retired `profile` command each required to be rejected as unknown (a testable negative)? [Measurability, Spec §SC-002]

## Namespace-to-store correctness

- [ ] CHK011 - Is each relocated command's destination store unambiguous (catalog operations write `catalog.db`, target operations write `local.db`)? [Clarity, Spec §FR-010, §FR-011]
- [ ] CHK012 - Is the complete set of commands moving under `catalog` specified, including `seed-signatures` which the issue prose omits? [Completeness, Spec §FR-010, Assumptions]
- [ ] CHK013 - Is `targets add --steam <app_id>` defined as equivalent to the former `steam profile <app_id>`, with the equivalence testable? [Consistency, Spec §FR-012, §SC-005]
- [ ] CHK014 - Is the residual `steam` namespace scope defined (what stays) rather than only what leaves? [Coverage, Spec §Clarifications, Assumptions]
- [ ] CHK015 - Is `catalog update`'s scope bounded (namespace + net-gated fetch, no new remote artifact shipped) so its acceptance is not open-ended? [Clarity, Spec §Clarifications, §FR-013]

## Usage-error semantics

- [ ] CHK016 - Is the "exactly one of `--target`/`--process`, required" rule specified with the exit-2 outcome for neither and for both? [Completeness, Spec §FR-002]
- [ ] CHK017 - Is the `--launch`-without-a-launchable-anchor case defined as a usage error rather than a silent no-op? [Edge Case, Spec §FR-005]
- [ ] CHK018 - Are the usage-error outcomes stated as a specific, verifiable exit code (2) rather than "an error"? [Measurability, Spec §FR-002]

## Presentation and bare invocation

- [ ] CHK019 - Are the four help groups and their exact command membership specified, with "nothing hidden" as an explicit constraint? [Completeness, Spec §FR-014]
- [ ] CHK020 - Is the bare-invocation behaviour (targets listing plus footer) defined including the empty-store case? [Edge Case, Spec §FR-015, §Edge Cases]
- [ ] CHK021 - Is the footer-suppression rule for explicit `targets` stated so bare `fragcap` and `fragcap targets` differ only by that line? [Clarity, Spec §FR-016, §SC-004]

## Documentation coherence and constitution touchpoints

- [ ] CHK022 - Is "every shipped documentation example names a command that exists after the change" stated as a verifiable requirement with a scan-based success criterion? [Measurability, Spec §FR-017, §SC-006]
- [ ] CHK023 - Is the P-6 obligation (glossary entry in the same change for any new term) reflected as a requirement, and are the slice's new terms identified? [Traceability, Spec §FR-018]
- [ ] CHK024 - Is the P-10/P-11 spec-impact discipline accounted for (the changelog fragment's `spec-impact` line and any master-spec section touched by the surface change)? [Traceability, Assumption]
- [ ] CHK025 - Is the full-gate success criterion (`cargo xtask ci` plus spec/impact checks) present so "done" is objectively bounded? [Measurability, Spec §SC-007]

## Consistency and boundary

- [ ] CHK026 - Do the Assumptions and Clarifications agree with the Functional Requirements (no requirement contradicts a recorded decision, e.g. the dropped `--install-dir`/`--steam` paths)? [Consistency, Spec §Clarifications, §Assumptions]
- [ ] CHK027 - Is the scope boundary explicit that capture internals (pipeline, attribution, ring, launch mechanics) are unchanged, so the slice is bounded to the surface? [Clarity, Spec §Assumptions]
- [ ] CHK028 - Are dependencies on S050/S051/S053 documented with what each supplies to this slice? [Dependency, Spec §Dependencies]

## Notes

- Items are requirement-quality questions, not implementation tests. An unchecked
  item means the spec should be tightened before `/speckit-plan`, not that code is
  wrong.
