# Security and Authorization Requirements Checklist: Guided Protocol Calibration Attempt

**Purpose**: Validate that the specification states the required authorization, evidence, and prohibited-capability boundaries
**Created**: 2026-09-10
**Feature**: [spec.md](../spec.md)

## Deliberate Selection and Scope

- [x] The protocol attempt is explicitly selected and limited to one authorized session per invocation
- [x] The complete exact plan identifier remains the authorization boundary
- [x] Target, launch case, route, address family, protocol, backend, versions, artifacts, deadlines, trust, and cleanup remain plan-bound
- [x] Changed target or launch authority requires refusal or reassessment before effects

## Evidence Integrity

- [x] Candidate input is defined as a request and never as evidence
- [x] Automatic candidates require concrete final-client observations
- [x] Unknown, unrouted, unrelated, ambiguous, unavailable, and uncorrelated observations are excluded
- [x] Positive facts remain append-only direct observations under existing applicability rules
- [x] Missing observation and loss remain visible and cannot become success

## Prohibited Capabilities

- [x] No automatic registration or internal multi-attempt loop is introduced
- [x] No persisted workflow or resume state is introduced
- [x] No hidden or system-wide trust is introduced
- [x] No certificate-pinning bypass or target key extraction is introduced
- [x] No target process handle, memory read, injection, signal, message, or forced termination is introduced
- [x] No weaker fallback path is permitted after trust, routing, launch, protocol, or cleanup failure

## Notes

- Every checklist item maps to a normative requirement or acceptance scenario in the feature specification.
