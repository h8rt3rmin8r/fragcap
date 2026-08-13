# Checklist: Constitution and Contract Requirements Quality

**Purpose**: Validate that the requirements for the non-profile capture path's
constitution-sensitive edges and its command-line contract are complete, clear,
consistent, and measurable before planning. Requirements-quality review, not an
implementation test.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)
**Focus**: P-1 capture posture, P-4 surfaced declines, P-9 honest fidelity, the
byte-identical profile-path invariant, and the input mutual-exclusion/exit-code
contract.

## P-1 Passive Capture Posture

- [x] CHK001 - Is it specified that the non-profile capture reuses the existing
  launch-agnostic engine rather than a new acquisition mechanism, so no new
  process-access surface is introduced? [Clarity, Spec §FR-009, §FR-010]
- [x] CHK002 - Is "no process handle opened, no process memory read" stated for
  the non-profile path explicitly, not only inherited by assumption? [Completeness,
  Spec §FR-009]
- [x] CHK003 - Is the already-running case required to be handled by the shared
  startup-snapshot fold (query-only), so attach-to-running adds no handle?
  [Consistency, Spec §FR-010]

## P-4 Surfaced Declines (No Silent Loss)

- [x] CHK004 - Are the three decline classes (unrecognized, ambiguous, unreadable)
  each required to be a surfaced failure that captures nothing? [Completeness, Spec
  §FR-007]
- [x] CHK005 - Is the decline reason required to be carried from the resolver's
  existing unresolved outcome rather than re-derived, so the surfaced reason is
  faithful? [Clarity, Spec §FR-007]
- [x] CHK006 - Is a not-installed `--steam` app id required to fail with a surfaced
  message naming the missing title, distinct from a resolved-but-declined layout?
  [Coverage, Spec §FR-008]
- [x] CHK007 - Is a nonexistent or non-directory `--install-dir` path required to
  be surfaced distinctly from a directory that scanned and matched nothing? [Edge
  Case, Spec §Edge Cases]

## P-9 Honest Fidelity

- [x] CHK008 - Is the synthesized identity required to be `heuristic-unverified`
  and explicitly never `authored`? [Clarity, Spec §FR-002, §SC-003]
- [x] CHK009 - Is the reason for the fidelity choice (resolved by heuristic, not
  typed by an operator) stated so it is not confused with `watch`'s `authored`
  stamp? [Consistency, Spec §Clarifications]
- [x] CHK010 - Is the synthesized profile required to go through the same
  validating construction as authored/watch/tap, so an invalid identity surfaces
  as a diagnostic not a malformed capture? [Measurability, Spec §FR-002]
- [x] CHK011 - Is the synthesized game identity required to be generic/honest
  (a placeholder id and, for `--steam`, the app id as a fact) rather than a
  fabricated title? [Clarity, Spec §Clarifications, §Key Entities]

## Byte-Identical Profile Path

- [x] CHK012 - Is the `run --profile` path required to be byte-identical to before
  the slice (resolution, overlay, output), so this is an added branch not a
  rework? [Measurability, Spec §FR-006, §SC-006]
- [x] CHK013 - Is the non-profile branch required to be reached only when the
  resolved target has no backing profile, so the branch order is unambiguous?
  [Consistency, Spec §Edge Cases, §FR-001]

## Input Contract

- [x] CHK014 - Are `--profile`, `--install-dir`, and `--steam` required to be
  mutually exclusive with exactly one required? [Completeness, Spec §FR-005]
- [x] CHK015 - Is the exit-code contract specified (usage error exit 2 for
  input misuse, surfaced failure exit 1 for a runtime decline/not-installed)?
  [Clarity, Spec §FR-005, §FR-007, §Clarifications]
- [x] CHK016 - Is it specified that `--steam` resolves to an install directory and
  then takes the same path as `--install-dir`, so the two share one non-profile
  branch rather than diverging? [Consistency, Spec §FR-004]
- [x] CHK017 - Is the source of the identity (the resolved target's existing
  `MatchPredicates`) specified, so the synthesized stage is not re-derived from a
  different signal? [Traceability, Spec §FR-001, §Assumptions]

## Dependencies & Assumptions

- [x] CHK018 - Are the reused contracts (resolver, providers, `for_install`,
  Steam `install_root_for`, the offline capture harness) documented as existing
  dependencies rather than new work? [Assumption, Spec §Assumptions]
- [x] CHK019 - Is "no new dependency, MSRV stays green" stated as a hard
  requirement? [Completeness, Spec §FR-012, §SC-007]

## Notes

- All items pass against the current spec; the operator's focus areas map to
  explicit FRs, clarifications, edge cases, and success criteria. Retained as the
  reviewable record that the constitution edges were specified, not assumed.
