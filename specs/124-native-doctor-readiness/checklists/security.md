# Security and Recovery Requirements Checklist: Native Doctor Readiness

**Purpose**: Validate that S124 requirements completely define safe observation and exact recovery before implementation
**Created**: 2026-09-03
**Feature**: [spec.md](../spec.md)

## Observation Authority

- [x] CHK001 Are all permitted readiness and residue authorities enumerated? [Completeness, Spec §FR-006]
- [x] CHK002 Is the boundary between observed ownership and an unrelated occupied endpoint explicit? [Clarity, Spec §FR-010]
- [x] CHK003 Are active-session requirements protected against PID reuse? [Security, Spec §FR-009]
- [x] CHK004 Are invalid, unsupported, and unreadable authorities forbidden from producing a clean result? [Consistency, Spec §FR-012]
- [x] CHK005 Is secret material excluded while identity and state remain observable? [Security, Spec §FR-007]

## Recovery Authority

- [x] CHK006 Is the sole source of recovery actions defined without permitting a second policy? [Clarity, Spec §FR-013]
- [x] CHK007 Are active, ambiguous, unrelated, and out-of-root resources excluded from mutation? [Coverage, Spec §FR-014]
- [x] CHK008 Is confirmation required before every offered mutation? [Security, Spec §FR-014]
- [x] CHK009 Are partial cleanup and retry-evidence requirements explicit? [Recovery, Spec §FR-015]
- [x] CHK010 Are recovery-lock conflicts and interrupted recovery covered as distinct scenarios? [Edge Case]

## Read-Only Guarantees

- [x] CHK011 Are prohibited side effects on the ordinary Doctor path exhaustively stated? [Completeness, Spec §FR-017]
- [x] CHK012 Are constitution P-1 prohibited capabilities explicitly excluded? [Consistency, Spec §FR-018]
- [x] CHK013 Is unrelated process cleanup prohibited even when a port or PID appears familiar? [Security, Spec §FR-010, §FR-018]
- [x] CHK014 Are noninteractive and machine-readable mutation refusals specified? [Coverage, User Story 3]

## Loss and Bounds

- [x] CHK015 Are time, depth, entry, session, and finding bounds required? [Completeness, Spec §FR-019]
- [x] CHK016 Must every bound report visible truncation rather than apparent absence? [No Silent Loss, Spec §FR-012]
- [x] CHK017 Are malformed and oversized owner records and journals covered? [Edge Case]
- [x] CHK018 Is an exhausted inventory prevented from authorizing cleanup? [Security, Spec §FR-012, §FR-014]

## Cross-Surface Consistency

- [x] CHK019 Are Capture and Deep Capture applicability rules defined independently? [Clarity, Spec §FR-001, §FR-002]
- [x] CHK020 Must human and JSON output carry the same facts and verdicts? [Consistency, Spec §FR-004]
- [x] CHK021 Is healthy retained evidence distinguished from blocking residue? [Clarity, Spec §FR-016]
- [x] CHK022 Is current native-backend guidance separated from accurate historical documentation? [Scope, Spec §FR-005]

## Scope and Completion

- [x] CHK023 Is runtime Doctor ownership separated from later release-packaging validation? [Dependency, Spec §FR-022]
- [x] CHK024 Are offline controlled tests required for all state and recovery classes? [Acceptance Criteria, Spec §FR-020]
- [x] CHK025 Is premature Deep Capture completion language prohibited? [Consistency, Spec §FR-021]
