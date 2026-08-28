# CLI Contract Checklist: CLI Reference Gate

**Purpose**: Test whether the S093 requirements completely and unambiguously define the command-tree, option, sink, example, and output-routing documentation contract
**Created**: 2026-08-28
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [X] CHK001 Are recursive command and subcommand coverage requirements defined? [Completeness, Spec FR-002, FR-003]
- [X] CHK002 Are local options, global options, short aliases, enumerated values, and declared defaults all included? [Completeness, Spec FR-005, FR-007]
- [X] CHK003 Are sink schemes, aliases, modifiers, values, and transport constraints included? [Completeness, Spec FR-009, FR-010]
- [X] CHK004 Are worked invocations and output-routing prose included as separate contract surfaces? [Completeness, Spec FR-012, FR-013]
- [X] CHK005 Are feature-gated options and intentional exclusions covered? [Completeness, Spec FR-004, FR-008]

## Requirement Clarity

- [X] CHK006 Is public visibility distinguished structurally from hidden harness controls? [Clarity, Spec FR-004]
- [X] CHK007 Is command ownership defined so propagated globals are not duplicated on every subcommand? [Clarity, Spec FR-005]
- [X] CHK008 Is a parser-declared default distinguished from an application-level fallback invisible to the parser? [Clarity, Spec FR-007]
- [X] CHK009 Is the current shipped command variant distinguished from optional network-capable flags? [Clarity, Spec FR-008]
- [X] CHK010 Is parsing-only validation explicitly separated from command dispatch and side effects? [Clarity, Spec FR-013, FR-016]

## Requirement Consistency

- [X] CHK011 Do the command-section and exact-once requirements agree without requiring token-level uniqueness in explanatory prose? [Consistency, Spec FR-003]
- [X] CHK012 Do the sink requirements preserve parser authority without creating a second accepted grammar? [Consistency, Spec FR-009]
- [X] CHK013 Do store-path requirements agree across capture, Deep Capture, target, catalog, technology, and extcap commands? [Consistency, Spec FR-011]
- [X] CHK014 Do JSON routing requirements distinguish command results, lifecycle events, capture bytes, warnings, and errors? [Consistency, Spec FR-012]
- [X] CHK015 Does the scope exclude command grammar changes while still authorizing reference corrections? [Consistency, Spec FR-018, FR-020]

## Acceptance Criteria Quality

- [X] CHK016 Can complete command coverage be measured as a two-sided set comparison? [Measurability, Spec SC-001]
- [X] CHK017 Can complete option coverage be measured per owning command path? [Measurability, Spec SC-002]
- [X] CHK018 Can a synthetic drift mutation prove that new commands, flags, values, and defaults fail the gate? [Measurability, Spec SC-003]
- [X] CHK019 Can worked invocation coverage be measured over every executable reference example? [Measurability, Spec SC-004]
- [X] CHK020 Can sink coverage be measured against parser-derived tokens? [Measurability, Spec SC-005]

## Scenario and Edge-Case Coverage

- [X] CHK021 Are commands with no local flags still required to have one section? [Coverage, Spec Edge Cases]
- [X] CHK022 Are hidden commands and options excluded without a manually drifting name list? [Coverage, Spec FR-004]
- [X] CHK023 Are parser-generated help and version controls addressed explicitly? [Coverage, Spec Edge Cases]
- [X] CHK024 Are feature variants required to prove conditional option availability? [Coverage, Spec FR-008]
- [X] CHK025 Are quoted paths, comments, and line continuations covered by the example-validation requirements? [Coverage, Spec FR-014]

## Safety and Operational Boundaries

- [X] CHK026 Does the spec prohibit executing examples during validation? [Safety, Spec FR-013, FR-016]
- [X] CHK027 Does the spec prohibit capture, store, network, trust, proxy, game, and elevation dependencies? [Safety, Spec FR-016]
- [X] CHK028 Is the no-runtime-dependency constraint explicit? [Dependency, Spec FR-017]
- [X] CHK029 Is integration into the existing docs and CI gates required without workflow modification? [Dependency, Spec FR-015, FR-020]
- [X] CHK030 Is rendered accessibility work explicitly reserved for issue #249? [Scope, Spec Assumptions]

## Notes

- All 30 requirement-quality checks pass before planning.
- The checklist is reviewer-oriented and treats a false green documentation gate as the primary risk.
