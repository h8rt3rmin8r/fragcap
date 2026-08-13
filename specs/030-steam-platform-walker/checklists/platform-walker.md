# Platform-Walker Refactor: Requirements Quality Checklist

**Purpose**: Validate that the S030 spec states the platform-walker refactor's
requirements completely, clearly, and consistently before planning and
implementation. Each item tests the requirements text, not the eventual code.
**Created**: 2026-08-12
**Feature**: [spec.md](../spec.md)

## Dependency Direction and Placement

- [ ] CHK001 - Does the spec state the walker provider lives in the Steam crate
      and that the profile crate must not gain a dependency on it? [Consistency,
      Spec §FR-002]
- [ ] CHK002 - Is it specified who assembles the resolver with the real walker
      (the crate that legitimately depends on both), so the stub's removal has a
      defined new home? [Completeness, Spec §FR-001, §FR-002]
- [ ] CHK003 - Is the dependency-direction check named as an acceptance signal
      rather than left implicit? [Measurability, Spec §SC-005]

## Composition with the Engine Rule

- [ ] CHK004 - Does the spec make explicit that the walker supplies the install
      directory to the resolver so the higher-precedence engine rule can resolve
      layout titles? [Clarity, Spec §FR-003]
- [ ] CHK005 - Is the precedence relationship stated (engine rule outranks the
      walker; the walker answers directly only when the engine rule declines)?
      [Consistency, Spec §FR-003, §FR-004]
- [ ] CHK006 - Is the mechanism by which an install directory reaches an
      earlier-precedence provider addressed or explicitly deferred to planning
      (a provider cannot mutate the request for a provider consulted before it)?
      [Ambiguity, Spec §FR-003]
- [ ] CHK007 - Is it specified how the target Steam title is identified in
      production (what tells the resolver which title to walk)? [Gap, Spec §FR-003]

## Fidelity and Provenance Honesty (P-9)

- [ ] CHK008 - Is every walker answer required to be stamped
      `heuristic-unverified` and forbidden a higher tier? [Consistency, Spec §FR-005]
- [ ] CHK009 - Is the provenance fixed to `steam-library` and explicitly
      forbidden from claiming `steam-appinfo` (a source not read)? [Clarity,
      Spec §FR-005]
- [ ] CHK010 - Is the distinction between what the walker reads (library
      manifests, install-directory files) and what it does not (application info)
      stated, so the provenance is defensible? [Completeness, Spec §Overview]

## Decline, Ambiguity, and Degradation (P-4)

- [ ] CHK011 - Are the decline conditions enumerated (not installed, no plausible
      client, genuinely ambiguous)? [Completeness, Spec §FR-006]
- [ ] CHK012 - Is "genuinely ambiguous" characterized enough to be testable
      (several near-identical candidates), or is the precise threshold explicitly
      deferred to planning? [Ambiguity, Spec §FR-006]
- [ ] CHK013 - Is it required that an ambiguity is a surfaced outcome rather than
      a silent drop, and that a decline lets the cascade continue to runtime
      observation? [Coverage, Spec §FR-006]
- [ ] CHK014 - Are non-fatal enumeration problems (unreadable path, malformed
      manifest) required to warn and continue rather than abort or silently drop?
      [Completeness, Spec §FR-008]

## Passive Observation (P-1)

- [ ] CHK015 - Does the spec require filesystem and registry reads only and
      forbid process handles, memory reads, and network access? [Coverage,
      Spec §FR-007]
- [ ] CHK016 - Is the `steam://` managed launch required to stay a convenience
      adapter and explicitly not a precondition of resolution or capture?
      [Consistency, Spec §FR-010]

## Scope Boundary (appinfo/PICS deferral)

- [ ] CHK017 - Is the appinfo/PICS deferral stated explicitly with its rationale,
      rather than implied by omission? [Clarity, Spec §Clarifications, §Assumptions]
- [ ] CHK018 - Is it clear what the deferral excludes (the launch array, the
      launcher-mediated flag, any new dependency) and where that work belongs?
      [Coverage, Spec §Assumptions]
- [ ] CHK019 - Are Epic/GOG/folder-of-exes and environment inheritance explicitly
      out of scope? [Clarity, Spec §Input]

## Data Model (the walker answer)

- [ ] CHK020 - Is a walker target origin, distinct from the profile, engine-rule,
      and observed origins, identified as needed, with the fields it carries
      (client identity plus the Steam app id and title)? [Completeness,
      Spec §Key Entities, §Assumptions]
- [ ] CHK021 - Is the walker's resolved target required to be consumable by the
      capture pipeline like any other target and to validate against the master
      schema where materialized? [Consistency, Spec §FR-009]

## Terminology (P-6 glossary-first)

- [ ] CHK022 - Does the spec require a full glossary definition of "platform
      walker" (currently only a referenced term) in this slice? [Traceability,
      Spec §FR-011]
- [ ] CHK023 - Are "platform walker" and "walker-resolved target" used
      consistently without unlabeled synonyms? [Consistency, Spec §Key Entities]

## Acceptance Measurability

- [ ] CHK024 - Are the success criteria concrete enough to test (engine-title via
      engine rule, non-engine title via walker, not-installed and ambiguous both
      degrade)? [Measurability, Spec §SC-001..§SC-003]
- [ ] CHK025 - Is the profile-outranks-walker-and-engine-rule ordering stated as
      a verifiable criterion? [Measurability, Spec §SC-004]
- [ ] CHK026 - Is fixture coverage (a fake Steam library composed with the
      engine-rule install-layout fixtures) required by the acceptance criteria
      rather than left implicit? [Coverage, Spec §Assumptions, §SC-001..§SC-003]
- [ ] CHK027 - Is the repository-gate pass (`cargo xtask ci`) stated as a done
      condition? [Completeness, Spec §SC-006]
