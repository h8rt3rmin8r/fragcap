# Native Boundary Requirements Checklist

**Purpose**: Review whether the S102 requirements adequately define the native product boundary and runtime safety obligations.

**Created**: 2026-08-30

**Feature**: [spec.md](../spec.md)

## Architecture Contract

- [x] CHK001 Does the specification require ownership for every current Deep Capture artifact, event, compatibility fact, cleanup obligation, CLI behavior, protocol, and launch path? [Completeness, Spec FR-002]
- [x] CHK002 Does it distinguish current external-backed behavior, S102 native foundation behavior, future native implementation, and permanent refusals? [Clarity, Spec FR-001, FR-004, FR-005]
- [x] CHK003 Are milestone exit criteria and the prohibition on early completion claims explicit? [Measurability, Spec FR-003, SC-001, SC-007]
- [x] CHK004 Is the facade-to-proxy dependency direction required without permitting a proxy-to-CLI edge? [Architecture, Spec FR-007, FR-008, FR-023]

## Runtime Ownership

- [x] CHK005 Are listener, connection, task, buffer, and shutdown bounds all required? [Completeness, Spec FR-012]
- [x] CHK006 Is every task required to be joined or surfaced as an incomplete cleanup failure? [Safety, Spec FR-013, FR-016]
- [x] CHK007 Are saturation, bind races, shutdown races, panics, cancellation, repeated cleanup, and forced timeout specified? [Coverage, Edge Cases]
- [x] CHK008 Does the specification prevent the foundation listener from claiming forwarding, decryption, or observation it does not implement? [Honesty, Spec FR-017]
- [x] CHK009 Are loopback-only binding and the rejection of ambient routing explicit? [Safety, Spec FR-011, FR-022]

## Dependency and Release Policy

- [x] CHK010 Must the exact selected graph pass the claimed MSRV rather than excluding the native feature from the MSRV claim? [Consistency, Spec FR-018, FR-019]
- [x] CHK011 Are advisory, license, dependency-edge, native-library, package, and Windows release checks required? [Coverage, Spec FR-018, FR-020, SC-006]
- [x] CHK012 Is the relationship between compiled native foundation code and unchanged shipped CLI selection unambiguous? [Clarity, Spec FR-008, Assumptions]

## Constitutional Safety

- [x] CHK013 Are all permanent P-1 refusals preserved despite the native completion requirement? [Safety, Spec FR-022]
- [x] CHK014 Does the specification prohibit external process and command-line certificate dependencies for the native path? [Safety, Spec FR-022]
- [x] CHK015 Are trust mutation, certificate issuance, and protocol inspection clearly deferred rather than accidentally omitted? [Scope, Assumptions]

## Notes

- This checklist reviews the requirements, not the implementation.
