# Performance Requirements Checklist: Native Deep Capture Performance Envelope

**Purpose**: Review whether S128 performance, resource, soak, and regression requirements are complete and measurable
**Created**: 2026-09-04
**Feature**: [spec.md](../spec.md)

## Matrix Completeness

- [x] CHK001 Are all shipped protocol families and both retention modes named explicitly? [Completeness, Spec FR-002]
- [x] CHK002 Is every case required to declare its workload and budgets before measurement? [Clarity, Spec FR-003]
- [x] CHK003 Is case identity protected against omission, duplication, renaming, and source drift? [Coverage, Spec US1]
- [x] CHK004 Are per-case failures prohibited from being hidden by pooled results? [Consistency, Spec FR-005]

## Metric Clarity

- [x] CHK005 Are throughput and latency definitions tied to useful traffic and a same-run baseline? [Clarity, Spec FR-004]
- [x] CHK006 Are CPU, memory, disk, queue, cache, task, and shutdown measurements all required? [Completeness, Spec FR-004]
- [x] CHK007 Are configured ceilings, observed peaks, terminal ownership, and refusals distinguished? [Clarity, Spec FR-006]
- [x] CHK008 Are hard pass thresholds quantified for every required metric? [Measurability, Spec SC-003 through SC-008]
- [x] CHK009 Is shared-runner tolerance bounded without permitting a hard-budget breach? [Consistency, Spec FR-010 and SC-009]

## Pressure and Conservation

- [x] CHK010 Is degradation behavior specified independently from forwarding correctness? [Coverage, Spec FR-007 and FR-008]
- [x] CHK011 Does the specification require an exact conservation equation with no unexplained remainder? [Measurability, Spec SC-006]
- [x] CHK012 Are retention-disabled and retention-saturated outcomes stated separately? [Coverage, Spec US2 and SC-005]
- [x] CHK013 Are concurrency, queue, certificate-name, and cleanup pressure scenarios included? [Completeness, Spec US2]

## Soak and Reproduction

- [x] CHK014 Is the long campaign's minimum wall-clock duration explicit? [Clarity, Spec FR-011 and SC-008]
- [x] CHK015 Are periodic samples and a distinct terminal reconciliation required? [Completeness, Spec FR-012]
- [x] CHK016 Is an interrupted or undersized soak prohibited from claiming success? [Exception Flow, Spec US3]
- [x] CHK017 Are environment provenance and comparability fields defined without machine-unique secrets? [Security, Spec FR-013]
- [x] CHK018 Are non-comparable reports explicitly excluded from regression conclusions? [Consistency, Spec FR-014]
- [x] CHK019 Are repeatability thresholds and case-level pass agreement measurable? [Acceptance Criteria, Spec SC-009]

## Scope and Safety

- [x] CHK020 Are loopback-only synthetic traffic and no trust mutation required? [Security, Spec FR-017 and FR-018]
- [x] CHK021 Is the absence of a shipped performance or fault-control switch explicit? [Scope, Spec FR-016]
- [x] CHK022 Is the distinction from the Windows integration slice documented? [Dependency, Spec Assumptions]
- [x] CHK023 Is Deep Capture completion language reserved for issue #334? [Scope, Spec FR-020]

## Notes

- The checklist is intended for specification and pull-request review before implementation evidence is accepted.
