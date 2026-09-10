# Security Requirements Checklist: Guided Steam Client Setup

**Purpose**: Review the completeness, clarity, and consistency of S143 authority, mutation, and no-effect requirements before implementation
**Created**: 2026-09-10
**Feature**: [spec.md](../spec.md)

## Authority Completeness

- [x] CHK001 Are eligibility requirements limited to one exact durable Steam target with an absent launch declaration? [Completeness, Spec FR-001 through FR-003]
- [x] CHK002 Is the distinction between Steam launch metadata and operator-authored socket-holder authority explicit? [Clarity, Spec FR-007, FR-012, FR-019]
- [x] CHK003 Are the complete target, candidate, discovery, store, proposed result, and no-effect authorities bound before confirmation? [Completeness, Spec FR-008]
- [x] CHK004 Is every later calibration or session authorization explicitly separated from setup confirmation? [Consistency, Spec FR-014, FR-021]

## Mutation and Race Safety

- [x] CHK005 Are target and Steam evidence revalidation requirements defined after confirmation and before persistence? [Coverage, Spec FR-015 and FR-016]
- [x] CHK006 Is the concurrent-writer boundary specified as an exact conditional update rather than an unchecked overwrite? [Clarity, Spec FR-017]
- [x] CHK007 Are all fields that the setup operation must preserve enumerated? [Completeness, Spec FR-018]
- [x] CHK008 Is the exact permitted mutation limited to one client declaration and authored fidelity? [Scope, Spec FR-019 and FR-020]
- [x] CHK009 Are deletion, replacement, import, promotion, and identical concurrent-update races addressed as changed authority? [Edge Case, Spec Edge Cases]

## Input and Output Integrity

- [x] CHK010 Are human default-decline and structured exact-line requirements independently specified? [Clarity, Spec FR-012 and FR-013]
- [x] CHK011 Are plan write and flush failures required to remain pre-mutation failures? [Exception Flow, Spec FR-011 and SC-004]
- [x] CHK012 Are setup plan and outcome records required to remain distinguishable from registration, guidance, and session events? [Consistency, Spec FR-010]

## Prohibited Capability Boundaries

- [x] CHK013 Are process access, process control, trust, proxy, capture, artifact, compatibility-fact, and workflow effects explicitly excluded from setup? [Coverage, Spec FR-020 and FR-024]
- [x] CHK014 Are launcher inference, publisher topology, and metadata-only ownership claims explicitly excluded? [Scope, Spec FR-003, FR-024, Assumptions]
- [x] CHK015 Are tests required for every authorization, drift, field-preservation, and separate-authority boundary? [Measurability, Spec FR-023 and SC-001 through SC-006]

## Review Result

- The requirements define complete authority, mutation, failure, and no-effect boundaries for this slice.
