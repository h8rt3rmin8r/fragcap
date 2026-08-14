# Requirements Quality Checklist: Constitutional & architectural risk surface

**Purpose**: Validate that the requirements for wiring the hint database into the
cascade are complete, clear, and consistent on the load-bearing constitutional
and architectural constraints, before planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Dependency Direction (P-2, P-3, specification section 8.3)

- [x] CHK001 - Is the requirement that the resolver's home component gains no
  dependency on the targets database stated as a testable constraint, not just an
  assumption? [Completeness, Spec §FR-010, §SC-006]
- [x] CHK002 - Do the requirements name where the concrete provider must reside
  (a component permitted to depend on both), so the placement is not left to
  implementer discretion? [Clarity, Spec §FR-010, Assumptions]
- [x] CHK003 - Is the injection point (which surface assembles the provider into
  the cascade) specified and constrained to a component that legitimately depends
  on both? [Completeness, Spec §FR-012, Assumptions]
- [x] CHK004 - Do the requirements state that exactly one implementation serves
  precedence 2, so the old stub's removal is a stated requirement rather than an
  implied cleanup? [Consistency, Spec §FR-011]

## P-1 Passive Observation

- [x] CHK005 - Do the requirements confine the provider's inputs to reading a
  stored database row (no process handle, no memory read, no launch)? [Coverage,
  Spec §Key Entities, Assumptions]
- [x] CHK006 - Is it clear that the hint answer names a file to match and does not
  reach into any running process? [Clarity, Spec §Key Entities "Hint answer"]

## P-9 Honest Instrument

- [x] CHK007 - Is the fidelity tier of every hint answer specified unambiguously
  as heuristic-unverified (never authored, verified, or observed)? [Clarity, Spec
  §FR-002, §US4]
- [x] CHK008 - Is the provenance source of a hint answer specified to a single
  concrete label, and required not to name a source the provider did not read?
  [Clarity, Spec §FR-003, §Clarifications]
- [x] CHK009 - Are the requirements explicit that the provider must not guess a
  capture identity from an insufficient row? [Consistency, Spec §FR-007, §US2]
- [x] CHK010 - Do the requirements state that a live observation can still
  override / refine a hint, so an inferred answer never masquerades as observed?
  [Coverage, Spec §FR-002, §US4]

## P-4 No Silent Loss

- [x] CHK011 - Is "usable launch executable" defined precisely enough to decide
  resolve-versus-decline objectively (platform filter, distinct-file-name set)?
  [Measurability, Spec §FR-008, §Clarifications]
- [x] CHK012 - Are the decline conditions enumerated completely (absent row, no
  usable executable, no application id, ambiguous multi-executable)? [Completeness,
  Spec §FR-007, §FR-008]
- [x] CHK013 - Is the requirement that an ambiguous decline records an explanatory
  note stated, so a not-resolved outcome can explain itself? [Completeness, Spec
  §FR-008, §Key Entities "Ambiguity note"]
- [x] CHK014 - Are the requirements clear that carried facts (launcher_mediated,
  engine) are not silently dropped when present? [Coverage, Spec §FR-009]
- [x] CHK015 - Is a missing hint database explicitly required NOT to be an error,
  distinct from a present-but-unreadable database which IS an error? [Consistency,
  Spec §FR-013, §FR-014, §Edge Cases]

## Graceful Degradation (feature off / no DB)

- [x] CHK016 - Do the requirements specify byte-identical behavior to a build
  without this feature when no database is available? [Measurability, Spec §FR-013,
  §SC-003, §US3]
- [x] CHK017 - Is the registration condition (feature present AND operator-supplied
  path AND file present) stated completely, with the empty-slot fallback defined?
  [Completeness, Spec §FR-012, §Clarifications]
- [x] CHK018 - Are the requirements consistent that an absent application id on a
  request yields a decline rather than an error? [Consistency, Spec §FR-007,
  §FR-015]

## Request Input Plumbing

- [x] CHK019 - Is the requirement that the application id reaches the provider
  through the resolution request specified without disturbing the other providers'
  inputs? [Clarity, Spec §FR-015, §Key Entities "Resolution request input"]

## Offline Testability

- [x] CHK020 - Do the requirements state the whole feature is testable with no
  network and no game, using a store the test seeds directly? [Completeness, Spec
  §FR-016, §SC-005]
- [x] CHK021 - Is the set of behaviors requiring test coverage enumerated
  (resolve, each decline case, profile-outranks-hint, hint-outranks-engine-rule,
  no-DB identity)? [Coverage, Spec §US1-4, §SC-005]
- [x] CHK022 - Is the Tier 2 seeder and any new seeding explicitly out of scope,
  so the slice's boundary is unambiguous? [Clarity, Spec §FR-017, §Assumptions]

## Notes

- All items pass against the clarified spec. The clarifications session pinned the
  three items that were previously soft (DB supply mechanism CHK017/019, executable
  selection CHK011, provenance label CHK008).
- This checklist tests requirement quality only; behavioral verification lives in
  the slice's Rust tests, driven by tasks.md.
