# Implementation-Readiness Checklist: Steam integration and managed launch

**Purpose**: Validate that the S17 requirements are complete, clear, and consistent in
the areas most exposed to the constitution's non-negotiables and to verification honesty,
before planning and implementation.
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Constitution Compliance (P-1, P-2, P-3, no-bundling)

- [x] CHK001 - Does the spec state explicitly that section 16.5 environment inheritance is
  out of scope for this slice, with the process-handle rationale recorded? [Completeness,
  Spec §Deferred / §Clarifications]
- [x] CHK002 - Is it unambiguous that no requirement in this slice opens a process handle
  carrying memory-read rights (OpenProcess/ReadProcessMemory/WriteProcessMemory)? [Clarity,
  Spec §FR-012 / §Deferred]
- [x] CHK003 - Does the spec require that fragcap-core takes no Steam or platform
  dependency (P-3 dependency direction)? [Completeness, Spec §FR-012]
- [x] CHK004 - Are the no-bundle / no-download / no-install obligations stated for Steam
  itself, mirroring the existing npcap posture? [Completeness, Spec §FR-015]
- [x] CHK005 - Is "reads local Steam metadata and invokes the already-installed protocol
  handler only" distinguished from installing or launching Steam? [Clarity, Spec §FR-015]

## Parser Robustness (VDF)

- [x] CHK006 - Is the required behavior on a malformed manifest specified as "report and
  skip", not "abort" and not "silently mis-parse"? [Clarity, Spec §FR-004, §Edge Cases]
- [x] CHK007 - Does the spec bound the VDF syntax subset the parser must handle rather than
  leaving "parse VDF" open-ended? [Clarity, Spec §FR-003, §Assumptions]
- [x] CHK008 - Are the malformed-input outcomes for both the library-folders manifest and
  the per-title application manifest covered, not just one? [Coverage, Spec §FR-004,
  §US3-AC2]
- [x] CHK009 - Is the duplicate-app_id-across-libraries outcome defined deterministically
  (first wins, collision reported)? [Edge Case, Spec §Edge Cases]

## Scaffolding Correctness (section 15.4)

- [x] CHK010 - Is "scaffolded profile passes section 15.4 validation unedited" stated as a
  hard requirement, not an aspiration? [Measurability, Spec §FR-008, §SC-001]
- [x] CHK011 - Does the spec forbid inferring `descends_from` from a static install-scan,
  and require image-name predicates instead? [Clarity, Spec §FR-006, §Clarifications]
- [x] CHK012 - Is the ambiguous-image-match obligation addressed (add path_contains/
  path_regex where two proposed stages would share a basename)? [Completeness, Spec §FR-006]
- [x] CHK013 - Is the heuristic-header-comment requirement present and its content
  specified (classification is heuristic, verify against an observed session)? [Clarity,
  Spec §FR-007]
- [x] CHK014 - Are the degenerate scan outcomes defined (no non-launcher image; every image
  launcher-tokened; so a client stage is always proposed)? [Edge Case, Spec §Edge Cases]
- [x] CHK015 - Is the not-installed app_id path specified as "error naming the app_id,
  writes no profile"? [Completeness, Spec §FR-009, §US1-AC3]

## Managed-Launch Safety

- [x] CHK016 - Is the launch ordering ("after watcher attach and capture-handle open")
  stated as a requirement, and is it testable without a live Steam process?
  [Measurability, Spec §FR-010, §US2 Independent Test]
- [x] CHK017 - Is the missing-platform/app_id refusal specified as a named configuration
  error raised before capture starts? [Clarity, Spec §FR-011, §SC-004]
- [x] CHK018 - Is the non-Windows `--launch` outcome defined (refused, named, before
  capture)? [Coverage, Spec §Edge Cases]
- [x] CHK019 - Are both previously stubbed surfaces (steam stub, --launch "deferred to S17"
  refusal) required to be removed with no stub path left reachable? [Completeness, Spec
  §FR-013]

## Neutral-Target Build & Platform Gating

- [x] CHK020 - Does the spec require the workspace still builds on the neutral non-Windows
  target with the Steam crate present? [Completeness, Spec §FR-014, §SC-006]
- [x] CHK021 - Is the platform-gating mechanism correctly left as a plan-level choice
  rather than under-specified in the spec? [Consistency, Spec §Clarifications / §Assumptions]

## Verification Honesty

- [x] CHK022 - Does the spec distinguish tier-1 (offline, unit-testable: VDF parse,
  classifier, launch-config validation) from tier-2/manual (physical Steam launch)?
  [Clarity, Spec §Assumptions, §SC-005]
- [x] CHK023 - Is it explicit that a live Steam launch must not be asserted as run in CI?
  [Traceability, Spec §Assumptions]

## Notes

- These items test whether the S17 requirements are written well enough to implement and
  verify against the constitution, not whether the implementation works.
- Cross-referenced against `.specify/memory/constitution.md` P-1 (technique denylist),
  P-2 (neutral core build), P-3 (dependency direction), and the npcap-style no-bundling
  rule generalized to Steam.
