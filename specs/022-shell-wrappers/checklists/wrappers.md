# Wrappers Checklist: Shell wrappers

**Purpose**: Validate that the requirements for the two wrappers, their house-
standard conformance, the shared contract, the scope discipline, the CI gate, and
the verification boundary are complete, clear, consistent, and measurable before
planning.

**Created**: 2026-08-11

**Feature**: [spec.md](../spec.md)

## Scope Discipline (P-7)

- [x] CHK001 Is each wrapper's scope pinned to the five section-18.1
  responsibilities, with capture logic and capture-output parsing explicitly
  excluded? [Clarity, Spec FR-010, Clarifications]
- [x] CHK002 Is the wrapper input contract the section 17.5 structured event
  stream (not human-readable output), so P-7 is satisfied by construction?
  [Consistency, Spec FR-009, Clarifications]
- [x] CHK003 Is unknown-option pass-through required, so a new binary flag works
  through the wrapper without a wrapper change? [Completeness, Spec FR-009, Edge
  Cases, SC-005]

## PowerShell Wrapper

- [x] CHK004 Is the PowerShell wrapper required to pass the vendored
  `Test-ScriptCompliance.ps1`, with the standard's structural elements named?
  [Measurability, Spec FR-001, SC-002]
- [x] CHK005 Is the elevation self-relaunch specified, including a declined
  elevation as a precondition failure (exit 2)? [Completeness, Spec FR-002, Edge
  Cases]
- [x] CHK006 Is driver detection required to install and download nothing (the
  Licensing rule, P-1), reporting the download location when absent? [Consistency,
  Spec FR-003, SC-007]
- [x] CHK007 Are interface filtering and output-path templating (date/time/profile
  tokens plus directory preparation) specified? [Completeness, Spec FR-004,
  FR-005]

## Bash Wrapper

- [x] CHK008 Is the Bash wrapper required to be built to the ShruggieTech Bash
  standard (four-section layout, self-parsing help, `set -euo pipefail` with IFS,
  the fixtures, verbosity handling, 0/1/2 exit) and pass the authored checker?
  [Measurability, Spec FR-006, SC-002]
- [x] CHK009 Is the WSL2 subsystem boundary specified: invoke the native binary
  through interop and translate paths in both directions? [Clarity, Spec FR-007,
  SC-004]
- [x] CHK010 Is the Linux-host-without-a-Windows-binary case required to report
  unavailable and exit 1, not fail obscurely? [Edge case, Spec FR-008, Edge
  Cases]

## Compliance Checkers and the CI Gate

- [x] CHK011 Is the un-vendored Bash checker resolved (author an in-repo checker;
  no skills-lock change), with the PowerShell checker reused? [Consistency, Spec
  FR-011, Clarifications]
- [x] CHK012 Is a `cargo xtask wrappers` gate required to run both checkers and
  both syntax checks, return 0/1/2, and be wired into `ci` and `ci.yml`, so both
  checkers run in continuous integration (section 18.4)? [Completeness, Spec
  FR-012, SC-001, US3]
- [x] CHK013 Is the pinned-artifact change (`scripts/**`, `ci.yml`) required to be
  recorded as a dated decision? [Consistency, Spec FR-014]

## Verification Boundary

- [x] CHK014 Is the tier-1 versus tier-2 boundary explicit: CI verifies the
  checkers, the syntax validity, the help paths, and the pure translation and
  templating logic, while the full runtime behavior is tier 2? [Measurability,
  Spec Assumptions, SC-003, SC-004, Clarifications]
- [x] CHK015 Is the pure logic (path translation, output templating) required to
  be checkable without a capture driver or a real Windows binary? [Measurability,
  Spec SC-004]

## Terminology

- [x] CHK016 Are glossary entries for the new terms (WSL2 interop, path
  translation, output template) required in the same change (P-6)? [Consistency,
  Spec FR-013]

## Notes

- Every item resolves against the current spec; none is outstanding. The checklist
  keeps the analyze gate anchored to the wrapper-specific risks (scope discipline,
  house-standard conformance, the subsystem boundary, the CI gate, and the
  honest verification boundary) rather than only the generic requirements set.
