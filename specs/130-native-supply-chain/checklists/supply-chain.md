# Supply-Chain Requirements Review Checklist

**Purpose**: Test whether the S130 requirements define a complete, reviewable release-security boundary before implementation.

**Created**: 2026-09-05

**Feature**: `specs/130-native-supply-chain/spec.md`

## Graph Coverage

- [x] CHK001 Are runtime, build, development, optional, all-feature, and Windows-target edges explicitly included? [Completeness, FR-001]
- [x] CHK002 Is host-independent treatment of inactive target edges specified? [Clarity, US1 scenario 3]
- [x] CHK003 Are source identity, checksums, registry, Git, and workspace path boundaries covered? [Completeness, FR-002, edge cases]
- [x] CHK004 Is duplicate-major policy distinguished from harmless patch or minor duplication? [Precision, FR-005]
- [x] CHK005 Does the unsafe-code requirement avoid overstating what metadata-only validation can prove? [Truthfulness, FR-006]

## Security and Compatibility Policy

- [x] CHK006 Are license, advisory, yanked, abandoned, prohibited package, and source findings release-blocking rather than warning-only? [Clarity, FR-002]
- [x] CHK007 Are MSRV and pinned development toolchain responsibilities distinct and mechanically testable? [Consistency, FR-003]
- [x] CHK008 Are direct feature and default-feature changes covered without requiring source execution? [Completeness, FR-004]
- [x] CHK009 Are critical dependency pins, ownership, cadence, compatibility, and emergency expectations defined? [Completeness, FR-007]
- [x] CHK010 Is advisory-data unavailability handled separately from a clean advisory result? [Failure semantics, FR-009]

## Exceptions and Maintenance

- [x] CHK011 Does every exception require identity, owner, exact scope, rationale, dates, and removal condition? [Completeness, FR-008]
- [x] CHK012 Are expired, malformed, duplicate, unused, and over-broad exceptions rejected? [Negative path, FR-008]
- [x] CHK013 Do routine and emergency procedures preserve review, CI, release authorization, compatibility, rollback, and evidence generation? [Completeness, FR-010]
- [x] CHK014 Are the procedure requirements testable rather than advisory prose only? [Measurability, SC-006]

## Release Evidence

- [x] CHK015 Are SBOM and notices derived from one exact locked shipped graph? [Consistency, FR-011, FR-012]
- [x] CHK016 Are schema, product, revision, lock, policy, tool, target, feature, time, ordering, and completeness identities defined? [Completeness, FR-013]
- [x] CHK017 Is independent validation required before both artifact and crate publication? [Ordering, FR-014]
- [x] CHK018 Are portable archive and installer payload requirements both explicit while final package certification remains S131? [Scope, FR-014, assumptions]
- [x] CHK019 Are stale, missing, duplicate, altered, and nondeterministic evidence cases covered? [Edge coverage, US3]

## Automation and Scope

- [x] CHK020 Are offline static and network-backed responsibilities separated? [Operability, FR-015, FR-016]
- [x] CHK021 Are pull request, default branch, manual, and bounded recurring audit triggers specified without creating local scheduled work? [Clarity, FR-016]
- [x] CHK022 Are diagnostics bounded and prohibited sensitive fields excluded? [Security, FR-017]
- [x] CHK023 Does the slice reuse current repository authorities and add no product runtime dependency or behavior? [Scope, FR-018, FR-020]
- [x] CHK024 Are pinned-artifact decisions and issue traceability required? [Governance, FR-019]

## Outcome

- [x] All requirements are specific enough to plan and test.
- [x] No critical ambiguity or unstated release authority remains.
- [x] S130 remains bounded to issue #328 and hands final packaging certification to S131.
