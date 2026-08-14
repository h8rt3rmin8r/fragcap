# Distribution & Release Readiness Checklist: S039

**Purpose**: Validate that the requirements for the unsigned installer, the
bundled barebones hint database, the hint-db default and bootstrap, the release
artifact and checksum changes, and the docs/glossary/spec honesty are complete,
clear, consistent, and measurable before planning and implementation.

**Created**: 2026-08-14

**Feature**: [spec.md](../spec.md)

**Note**: These items test the quality of the requirements, not the eventual
implementation.

## Requirement Completeness

- [x] CHK001 - Are the three release artifact forms (portable archive, installer, loose hint database) each explicitly required, with the archive's contents enumerated? [Completeness, Spec FR-015, FR-016]
- [x] CHK002 - Is a checksum required for each of the three artifacts individually, rather than for the archive alone? [Completeness, Spec FR-017]
- [x] CHK003 - Are the installer's placed files (program, read-only database template, license, notice) each enumerated? [Completeness, Spec FR-010]
- [x] CHK004 - Is the requirement to acquire the installer toolchain on the build runner stated rather than assumed present? [Completeness, Spec FR-018]
- [x] CHK005 - Is the pinned-artifact obligation (a dated changelog decision for the release-workflow change) stated as a requirement? [Completeness, Spec FR-029]
- [x] CHK006 - Are the new distribution terms required to receive glossary entries in the same change, with the generated index regenerated? [Completeness, Spec FR-021]
- [x] CHK007 - Are the specification amendments (artifacts section, no-bundling scope, per-user data default) each named as required edits? [Completeness, Spec FR-022, FR-023]
- [x] CHK008 - Is the committed empty seed document required as the deterministic, offline source of the barebones database? [Completeness, Spec FR-007]

## Requirement Clarity

- [x] CHK009 - Is "barebones database" unambiguously defined as an empty current-schema store rather than a small seeded set? [Clarity, Spec Clarifications, FR-007]
- [x] CHK010 - Is the precedence among explicit flag, environment override, and the new default stated without ambiguity? [Clarity, Spec FR-001, FR-002]
- [x] CHK011 - Is "best-effort" for the Defender exclusion defined concretely (a refusal does not fail the install)? [Clarity, Spec FR-012]
- [x] CHK012 - Is the unsigned-installer guidance requirement specific about what the documentation must say (the warning is expected, how to proceed, checksum as the integrity check, signing tracked separately)? [Clarity, Spec FR-019]
- [x] CHK013 - Is the scope of the first-run bootstrap (which capture entry point triggers it) explicitly bounded? [Clarity, Spec Clarifications]
- [x] CHK014 - Is "takes effect in newly opened terminals" stated so the system-path behavior is not mistaken for immediate effect in the running shell? [Clarity, Spec FR-009]

## Requirement Consistency

- [x] CHK015 - Is the bundled hint database consistently distinguished from a game profile, so "the archive ships no game profiles" does not conflict with shipping the database? [Consistency, Spec FR-022]
- [x] CHK016 - Is the no-bundling obligation consistently scoped to the capture driver only, so the bundled database does not read as a violation? [Consistency, Spec FR-023, FR-025]
- [x] CHK017 - Do the default-path and bootstrap requirements preserve the existing explicit-path semantics (absent is non-fatal, unopenable is loud) without contradiction? [Consistency, Spec FR-002, FR-004]
- [x] CHK018 - Is the read-only template placement consistent between the installer and the portable archive so one bootstrap code path serves both? [Consistency, Spec FR-010, FR-015]

## Acceptance Criteria Quality

- [x] CHK019 - Is the bootstrap outcome objectively verifiable (a valid current-schema database exists at the default; a second run leaves it unchanged)? [Measurability, Spec SC-001]
- [x] CHK020 - Is the barebones database's validity expressed as a measurable round-trip (import produces a store the export round-trips to a valid empty document)? [Measurability, Spec SC-003, FR-008]
- [x] CHK021 - Is artifact completeness expressed as a checkable set (three artifacts, three checksums)? [Measurability, Spec SC-004]
- [x] CHK022 - Are the no-new-dependency and unchanged-version constraints stated as checkable outcomes? [Measurability, Spec SC-007, SC-008]
- [x] CHK023 - Is the documentation-gate outcome (terms defined, index reproduces exactly) measurable? [Measurability, Spec SC-006]

## Scenario & Edge-Case Coverage

- [x] CHK024 - Are requirements defined for when the per-user application-data base is unresolvable (default simply absent, run proceeds)? [Coverage, Edge Case, Spec Edge Cases]
- [x] CHK025 - Are requirements defined for a bootstrap write failure (warn, non-fatal, capture proceeds)? [Coverage, Exception Flow, Spec FR-005]
- [x] CHK026 - Are requirements defined for the installer toolchain being absent on the runner (explicit install; a miss is a build failure, not a silent omission)? [Coverage, Spec FR-018, Edge Cases]
- [x] CHK027 - Are requirements defined for the platform refusing the Defender exclusion (tamper protection) without failing the install? [Coverage, Exception Flow, Spec FR-012]
- [x] CHK028 - Is the major-upgrade path addressed (stable upgrade identity; unexercised until a second release)? [Coverage, Spec FR-011, Edge Cases]
- [x] CHK029 - Is the uninstall path's completeness specified (files, path entry, and exclusion all removed)? [Coverage, Spec FR-012, US2]

## Non-Functional & Constitution Constraints

- [x] CHK030 - Is the P-1 boundary of the Defender exclusion stated as a requirement (scoped to fragcap's own install dir; no process handle, no target memory/traffic/stack)? [Security, Spec FR-024, FR-012]
- [x] CHK031 - Is the "capture driver never bundled/downloaded/installed, only linked" obligation a stated requirement? [Security, Spec FR-013, FR-025]
- [x] CHK032 - Is the minimum-supported-toolchain-stays-green constraint stated, tied to the bootstrap adding no new dependency? [Non-Functional, Spec FR-026, SC-008]
- [x] CHK033 - Is the text-encoding constraint (no byte-order mark, line-feed endings, no em or en dashes) required across new files including the installer definition and code comments? [Non-Functional, Spec FR-028]
- [x] CHK034 - Is the manual-verification honesty posture for installer runtime behavior stated as a requirement rather than left implicit? [Non-Functional, Spec SC-005, Assumptions]

## Dependencies & Assumptions

- [x] CHK035 - Is the assumption that the hint database is a single at-rest file (safe to copy) documented? [Assumption, Spec Assumptions]
- [x] CHK036 - Is the dependency on the existing offline import path (no new production code for the DB) documented? [Assumption, Spec FR-007]
- [x] CHK037 - Is the version-bump-at-release-cut boundary documented so the slice is not expected to bump the version? [Assumption, Spec FR-027, Assumptions]
- [x] CHK038 - Is the default-on local accumulation consequence (touching local Steam data by default, sharing still opt-in) documented as an assumption/decision? [Assumption, Spec Clarifications]

## Notes

- Every item traces to a spec section or marker (100 percent traceability).
- All items resolve against the current spec, so the checklist is recorded as
  passing; it stands as the requirements-quality gate for planning and analyze.
