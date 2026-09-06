# API Stability Checklist: Stable Public Rust API

**Purpose**: Validate that the S132 requirements completely define the public compatibility boundary before implementation
**Created**: 2026-09-05
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are all shipped Deep Capture capability groups required to be reachable through the stable facade surface? [Completeness, Spec FR-002]
- [x] CHK002 Are production entry points distinguished from backend implementation details that remain outside the stable inventory? [Completeness, Spec FR-003]
- [x] CHK003 Are configuration, plans, authorization, adapters, events, observations, leases, reports, errors, routing, artifacts, and recovery addressed? [Coverage, Spec FR-002]
- [x] CHK004 Are legacy compatibility exports and their relationship to the curated stable surface defined? [Completeness, Assumption]

## Requirement Clarity

- [x] CHK005 Is the meaning of stable explicit rather than inferred from Rust visibility alone? [Clarity, Spec FR-001 and FR-007]
- [x] CHK006 Are necessary production backend types distinguished from unnecessary implementation leakage? [Clarity, Spec FR-003]
- [x] CHK007 Are compatible enum and input evolution rules stated in terms an external consumer can follow? [Clarity, Spec FR-005 and FR-006]
- [x] CHK008 Are correctness and security exceptions to compatibility bounded and documented? [Clarity, Spec FR-007]

## Concurrency and Ownership

- [x] CHK009 Are `Send`, `Sync`, and thread-confinement guarantees required for every relevant public category? [Coverage, Spec FR-008]
- [x] CHK010 Is cooperative cancellation defined at both lifecycle boundaries and inside bounded adapter calls? [Completeness, Spec FR-009]
- [x] CHK011 Are ownership transfer, lease cleanup, terminal state, and drop behavior part of the required contract? [Coverage, Spec FR-008]
- [x] CHK012 Does the spec avoid promising unsafe preemption of arbitrary blocking Rust trait calls? [Consistency, Spec FR-009]

## Acceptance Criteria Quality

- [x] CHK013 Can exact CLI coverage through the stable facade be measured? [Measurability, Spec SC-001 and SC-002]
- [x] CHK014 Can accidental backend leakage be measured against an inventory? [Measurability, Spec SC-003]
- [x] CHK015 Can the native no-CLI example be run under explicit effect and residue bounds? [Measurability, Spec SC-004]
- [x] CHK016 Can cancellation and non-exhaustive evolution guarantees be tested without relying on review attention? [Measurability, Spec SC-005 and SC-006]

## Safety and Release Boundaries

- [x] CHK017 Are the constitution P-1 denylist and explicit local-only controlled-lab boundary preserved? [Consistency, Spec FR-011 and FR-014]
- [x] CHK018 Are protocol behavior, artifact schemas, packaging, release publication, and the final completion claim excluded from change? [Scope, Spec FR-013 through FR-017]
- [x] CHK019 Is no scheduled or soak work authorized by this slice? [Scope, Spec FR-017]
- [x] CHK020 Is the S131 completion-status correction included without reopening its implementation scope? [Completeness, Spec FR-016]

## Notes

- Standard depth for pull-request reviewers. The highest-risk areas are accidental compatibility promises, hidden CLI-only policy, and concurrency claims that Rust types do not actually guarantee.
