# Routing and Evidence Requirements Checklist

**Purpose**: Review the quality of S207's route, scope, authorization, and evidence requirements before implementation.

**Created**: 2026-11-15

**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Does the spec name every effect that must be identical in the authorized and applied route? [FR-001]
- [x] CHK002 Does it define the sole allowed late-bound route value and the refusal point for other drift? [FR-001]
- [x] CHK003 Does it distinguish process observation for ownership from packet and application retention? [FR-002]
- [x] CHK004 Does it define ordinary and advanced launcher network effects separately? [FR-003, FR-004]
- [x] CHK005 Does it state how historical compatibility data is used after calibration removal? [FR-007, FR-008]

## Requirement Clarity And Consistency

- [x] CHK006 Can a reviewer distinguish a launcher HTTP CONNECT abort from a TLS or origin refusal from the written contract? [FR-005]
- [x] CHK007 Does unknown prefix provenance remain distinct from plaintext and decrypted provenance? [FR-006]
- [x] CHK008 Do the scope requirements avoid claiming that an uncertain owner is the selected game? [FR-002, FR-010]
- [x] CHK009 Are first-run behavior and the retained cold-launch limitation stated without conflict? [Scope, Assumptions]

## Acceptance And Edge Coverage

- [x] CHK010 Are zero-, one-, and multi-intermediary chains covered with an unrelated process? [SC-002, Edge Cases]
- [x] CHK011 Are successful late listener binding and deliberate plan drift both defined? [SC-003]
- [x] CHK012 Are delayed, partial, empty, failed, and genuine cleartext prefixes represented? [SC-005]
- [x] CHK013 Is unrelated traffic continuity and observed packet loss accounting still an explicit gate? [FR-010, SC-006]
- [x] CHK014 Is the completion gate limited to controlled evidence rather than a future private game run? [Scope, SC-006]

## Notes

The requirements are ready for technical planning. The plan must decide CLI syntax and versioning while preserving these stated user outcomes.
