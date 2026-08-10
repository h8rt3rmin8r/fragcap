# Requirements Quality Checklist: Filter Management

**Purpose**: Validate that the filter-management requirements are complete,
unambiguous, and faithful to specification sections 12.2 and 12.3 before
implementation.
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Endpoint source and narrowing

- [x] CHK001 The endpoint set is sourced from the attribution map, never from
  observed name resolution or traffic inspection. [Completeness, Spec §FR-001]
- [x] CHK002 An endpoint set compiles to a program admitting exactly its union,
  across IPv4 and IPv6, deterministically. [Clarity, Spec §FR-002]
- [x] CHK003 The bootstrap filter remains until the first narrowing, which is
  installed per live handle. [Completeness, Spec §FR-003]
- [x] CHK004 Over-admission of shared-port traffic is accepted and resolved in
  userspace, not tightened in the kernel. [Consistency, Spec §FR-007]

## Maintenance timing

- [x] CHK005 Recompilation is debounced by two seconds. [Clarity, Spec §FR-004]
- [x] CHK006 Reinstallation is rate limited to one per five seconds per handle.
  [Clarity, Spec §FR-004]
- [x] CHK007 Rapid endpoint-set churn coalesces into a single reinstall.
  [Edge Case, Spec §FR-004]
- [x] CHK008 A closing endpoint held by the S10 retention window stays in the set
  until retention lapses. [Edge Case, Spec §FR-001]
- [x] CHK009 A transiently empty set (after narrowing) keeps the last narrowed
  program rather than reverting to bootstrap. [Edge Case, Spec §Clarifications D-d]

## Correctness and accounting

- [x] CHK010 Userspace attribution runs on every packet regardless of the filter;
  the filter is never the scope authority. [Completeness, Spec §FR-005]
- [x] CHK011 A packet a stale filter briefly excludes is counted as a filter gap
  and surfaced. [Completeness, Spec §FR-006]
- [x] CHK012 The gap counter counts occurrences, not fabricated kernel-excluded
  packet counts (P-9). [Consistency, Spec §FR-006]
- [x] CHK013 The first narrowing after bootstrap records no gap. [Edge Case,
  Spec §Acceptance US3-3]
- [x] CHK014 `filter_gaps` is distinct from the kernel, buffer, and sink drop
  counters, and the conservation invariant is unaffected. [Consistency,
  Spec §FR-013]

## Placement and separation

- [x] CHK015 All new logic is platform-neutral in `fragcap-core`; no new core
  dependency is introduced. [Constraint, Spec §FR-010]
- [x] CHK016 The control thread installs filters without merging `PacketSource`
  and `FlowAttributor` and without a `Sync` bound on `PacketSource`. [Constraint,
  Spec §FR-009]
- [x] CHK017 No operator-facing filter flag or profile key is introduced.
  [Scope, Spec §FR-011]

## Testability and glossary

- [x] CHK018 Compilation and policy are pure and tier-1 testable; the wiring is
  tested against a recording source double. [Completeness, Spec §FR-008]
- [x] CHK019 Every introduced term gets a glossary entry in this change, including
  the dangling `Filter gap`. [Constraint, Spec §FR-012]

## Notes

- All items are satisfied by the spec as written; the checklist is a
  pre-implementation gate, and implementation must keep each true.
