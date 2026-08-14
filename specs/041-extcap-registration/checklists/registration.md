# Checklist: extcap registration correctness

**Purpose**: Validate that the requirements for register/unregister and the
installer option are complete, unambiguous, and consistent before implementation.
**Created**: 2026-08-14
**Feature**: [spec.md](../spec.md)

## Registration Behavior

- [x] CHK001 Is the exact target (name and location) of the registered binary
  specified as the same one the readiness check probes? [Consistency, Spec FR-001]
- [x] CHK002 Is directory creation on a missing extcap directory required rather
  than left implicit? [Completeness, Spec FR-001, Edge Cases]
- [x] CHK003 Is register defined as refresh-on-existing (overwrite with the
  current binary) rather than skip? [Clarity, Spec FR-002, Clarifications]
- [x] CHK004 Is unregister-when-absent defined as success, not error?
  [Clarity, Spec FR-003]
- [x] CHK005 Is the `--dir` override defined as a directory (not a file path),
  with a documented platform default? [Ambiguity, Spec FR-004, Clarifications]

## Readiness Agreement

- [x] CHK006 Is the post-register readiness verdict (installed) and the
  post-unregister verdict (not registered) specified? [Completeness, Spec FR-005]
- [x] CHK007 Is it explicit that this slice does not change how the readiness
  check locates the extcap directory, so tool and check cannot drift?
  [Consistency, Spec Assumptions]

## Analyzer Protocol Non-Regression

- [x] CHK008 Are all four analyzer protocol invocations enumerated as
  must-still-work? [Completeness, Spec FR-006]
- [x] CHK009 Is the requirement stated for BOTH invocation forms (bare top-level
  and explicit `extcap` subcommand)? [Coverage, Spec FR-006, US4]
- [x] CHK010 Is a regression test over those invocations required rather than
  assumed? [Measurability, Spec SC-003]

## Installer Option

- [x] CHK011 Is the installer component defined as optional, with the
  not-selected path (install proceeds, not registered) specified?
  [Completeness, Spec FR-007]
- [x] CHK012 Does the installer component reuse the same registration behavior /
  target as the command, rather than a second mechanism? [Consistency,
  Spec Assumptions]

## Error and Safety

- [x] CHK013 Are the failure cases (undetermined location, undetermined binary
  path, unwritable directory) required to report an error and never claim
  success? [Coverage, Spec FR-008, SC-005]
- [x] CHK014 Is it explicit that only fragcap's own binary is registered and
  npcap/Wireshark are never downloaded, installed, or modified? [Constitution
  P-1/Licensing, Spec FR-010]
- [x] CHK015 Is elevation explicitly not required for the default per-user
  target? [Clarity, Spec Clarifications]

## Documentation

- [x] CHK016 Is same-change CLI-reference documentation of the new subcommand
  required (P-6-adjacent, CLI surface)? [Traceability, Spec FR-009]

## Notes

All items pass on review. The two highest-risk areas, the analyzer-protocol
non-regression (CHK008 to CHK010) and the tool/readiness agreement on the target
location (CHK001, CHK007), are pinned in FR-001, FR-005, FR-006 and the
Assumptions.
