# Security and Recovery Requirements Checklist: Crash-Safe Lifecycle Authority

**Purpose**: Review whether the written requirements fully constrain target-scoped routing, durable effects, exact recovery, and crash-readable evidence
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md)

**Note**: This checklist evaluates requirements quality, not implementation behavior.

## Effect Authorization

- [x] CHK001 Are all permitted routing effect classes and permanently prohibited fallbacks explicitly bounded? [Completeness, Spec FR-004 and FR-007]
- [x] CHK002 Is the relationship between immutable plan review and later effect execution unambiguous? [Clarity, Spec FR-001 and FR-002]
- [x] CHK003 Are changed-plan, unsupported-target, and target-owned file conflict cases specified? [Coverage, Spec Edge Cases]

## Durable Recovery

- [x] CHK004 Is obligation-before-effect ordering stated for every external resource class? [Completeness, Spec FR-009 and FR-010]
- [x] CHK005 Are ownership reuse and unrelated-resource protection objectively defined? [Security, Spec FR-011 and FR-013]
- [x] CHK006 Are partial, corrupt, contradictory, repeated, and interrupted recovery cases covered? [Edge Cases, Spec FR-014]
- [x] CHK007 Is repeated recovery required to remain idempotent and restartable? [Clarity, Spec FR-013]
- [x] CHK008 Is evidence retention distinguished from cleanup of temporary effects? [Consistency, Spec FR-017]

## Evidence and Loss

- [x] CHK009 Are proxy and cleanup chronology authorities distinct from derived summaries? [Consistency, Spec FR-025 and FR-026]
- [x] CHK010 Are incomplete stream prefixes and writer failures specified without false completion? [Coverage, Spec FR-020 and FR-024]
- [x] CHK011 Are connection, resource, and count reconciliation requirements measurable across all artifacts? [Measurability, Spec FR-021 through FR-023]
- [x] CHK012 Are finite identity bounds and exact overflow accounting specified independently from traffic volume? [Performance, Spec FR-027 and FR-028]

## Dependencies and Completion

- [x] CHK013 Is the #306 to #320 to #336 dependency order explicit? [Dependency, Spec Assumptions]
- [x] CHK014 Are #305, #319, and #321 boundaries excluded without removing the shared seams S109 must supply? [Scope, Spec Assumptions]
- [x] CHK015 Is the final verification gate measurable and consistent with the constitution? [Acceptance Criteria, Spec SC-012]

## Notes

- Standard depth, written for pull-request reviewers before implementation.
- Security, recovery, boundedness, and cross-artifact authority are mandatory gates.
