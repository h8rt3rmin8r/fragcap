# Security Requirements Checklist: Native Deep Capture Threat Model

**Purpose**: Validate the completeness, clarity, and consistency of S125 security requirements before planning.

**Created**: 2026-09-04

**Feature**: `specs/125-native-threat-model/spec.md`

## Coverage Quality

- [x] CHK001 Are all issue-mandated threat categories named explicitly? [Completeness, Spec FR-003]
- [x] CHK002 Are trust boundaries and sensitive assets required across the complete shipped lifecycle? [Coverage, Spec FR-002]
- [x] CHK003 Does each threat require prevention, detection, containment, evidence, and test ownership? [Completeness, Spec FR-004]
- [x] CHK004 Is high-risk executable evidence distinguished from prose and residual-risk acceptance? [Clarity, Spec FR-005 and FR-006]

## Fail-Closed Semantics

- [x] CHK005 Are unauthenticated and unrelated-client outcomes explicitly refused before forwarding? [Security, Spec FR-011]
- [x] CHK006 Are upstream policy, rebinding, normalization, and ambiguity requirements explicit? [Security, Spec FR-011 and FR-012]
- [x] CHK007 Are saturation, loss, and interrupted cleanup required to remain visible? [Observability, Spec FR-013]
- [x] CHK008 Is P-1 reconfirmed across all routing and protocol paths? [Constitution, Spec FR-014]

## Review Currency

- [x] CHK009 Must protocol-family drift force model review? [Traceability, Spec FR-009]
- [x] CHK010 Must direct proxy dependency drift force model review? [Traceability, Spec FR-010]
- [x] CHK011 Are missing, ignored, duplicate, and malformed evidence references rejected? [Testability, Spec FR-007, FR-008, and FR-016]

## Scope and Verification

- [x] CHK012 Is the portable, offline CI contract stated? [Testability, Spec FR-015]
- [x] CHK013 Are later fuzzing, performance, platform, packaging, supply-chain, and completion slices excluded? [Scope, Spec FR-017 and FR-018]
- [x] CHK014 Are measurable outcomes present for coverage, drift, abuse refusal, and dependency stability? [Measurability, Spec SC-001 through SC-005]

## Notes

- Completed during the S125 clarify/checklist pass. No unresolved requirement ambiguity remains.
