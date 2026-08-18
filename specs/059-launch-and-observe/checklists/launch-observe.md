# Checklist: Launch-and-observe promotion requirements quality (S059)

**Purpose**: Unit-test the requirements for S059 (observe-mode capture of unresolved targets + capture-time promotion) before implementation.
**Created**: 2026-08-18
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Is the precondition for observe-mode resolution stated exactly (an unresolved launch chain that still names an observed executable), and the still-refused case (empty/steamless) named? [Completeness, Spec §FR-001, Edge Cases]
- [ ] CHK002 - Is the observe-mode profile's validity constraint stated (must pass profile validation, must not be an empty-predicate wildcard)? [Completeness, Spec §FR-002, Clarifications]
- [ ] CHK003 - Is the child-holder requirement stated (the profile binds a socket holder whether it is the observed executable or a descendant)? [Completeness, Spec §FR-003, Clarifications]
- [ ] CHK004 - Is the aggregation defined (per socket-holding image, attributed-packet count, deterministic across runs)? [Completeness, Spec §FR-004]
- [ ] CHK005 - Is the additive constraint on aggregation stated (no existing counter, completion summary, or golden output changes)? [Completeness, Spec §FR-005, SC-004]
- [ ] CHK006 - Is the promote-on-observation / leave-on-no-observation pair fully specified? [Completeness, Spec §FR-006, FR-007]

## Requirement Clarity

- [ ] CHK007 - Is the dominant-image selection precise enough to test (max attributed packets, deterministic tiebreak)? [Clarity, Spec §FR-004, Clarifications]
- [ ] CHK008 - Is "promotion" defined concretely (launch chain rewritten to a resolved client naming the observed image; fidelity raised to verified)? [Clarity, Spec §FR-006, Clarifications]
- [ ] CHK009 - Is the extcap exclusion stated precisely (same resolution, no store write-back)? [Clarity, Spec §FR-008, US3]

## Requirement Consistency

- [ ] CHK010 - Is the shared-seam requirement consistent with the extcap-no-promote requirement (one resolution, one write-back owner)? [Consistency, Spec §FR-008, US3]
- [ ] CHK011 - Is the no-fabrication rule (leave unchanged on no observation) consistent with the P-9 framing throughout? [Consistency, Spec §FR-007, SC-003]
- [ ] CHK012 - Is the Steam-anchored unresolved case consistently routed to the existing cascade rather than the new observe branch? [Consistency, Spec Edge Cases, §FR-009]

## Acceptance Criteria Quality

- [ ] CHK013 - Is each success criterion objectively verifiable (capture succeeds, second capture reads resolved, store unchanged, goldens reproduced, offline test)? [Measurability, Spec §SC-001..SC-005]
- [ ] CHK014 - Is "no new dependency / no Cargo.lock delta" a checkable constraint? [Measurability, Spec §FR-013]

## Scenario & Edge Coverage

- [ ] CHK015 - Are both promotion branches (dominant observed / nothing observed) covered as acceptance scenarios? [Coverage, Spec §US1]
- [ ] CHK016 - Is the zero-attributed-packet run covered as an edge case that does not promote? [Edge Case, Spec Edge Cases, §FR-007]
- [ ] CHK017 - Is the output-parity requirement covered by a golden comparison scenario? [Coverage, Spec §US2, SC-004]

## Dependencies, Assumptions, Governance

- [ ] CHK018 - Is the hard boundary (no new direct-exe launcher; live launch stays Steam-only) recorded and its Tier 2 residue labeled? [Governance, Spec §FR-009, FR-010, Edge Cases]
- [ ] CHK019 - Are the new glossary terms enumerated as a same-change requirement (P-6)? [Governance, Spec §FR-011]
- [ ] CHK020 - Is the spec-reconciliation requirement (17.2/17.7 + `cargo xtask spec`) recorded? [Governance, Spec §FR-012]
