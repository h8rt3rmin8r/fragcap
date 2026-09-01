# Evidence Integrity Checklist: Correlated Native Evidence

**Purpose**: Verify the S108 requirements define honest provenance, bounded derived output, and durable bundle authority before implementation
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md)

## Correlation Provenance

- [x] Every observation requires an explicit correlation state and reason
- [x] Connection, stream, message, flow, process, role, fidelity, time, and loss anchors are covered
- [x] Endpoint reuse, retained attribution, process exit, late publication, and race cases are covered
- [x] IPv4, IPv6, direction, multiplexing, and transport identity are covered
- [x] Unsupported transport handlers cannot be presented as implemented
- [x] Determinism and count reconciliation have measurable outcomes

## Projection Fidelity

- [x] HAR is defined as a projection whose fields trace to authoritative native evidence
- [x] Placeholder status, size, timing, body, and completion claims are prohibited
- [x] Partial, failed, interrupted, binary, compressed, and bounded content cases are covered
- [x] Evidence retention is bounded independently from forwarding
- [x] Independent HAR reader conformance is required

## Manifest Authority

- [x] Schema version and product version are distinct
- [x] Every artifact and omission has exactly one authority owner
- [x] Completeness, loss, sensitivity, content type, and correlation capability are required
- [x] Complete, partial, failed, and crash-prefix states are distinguishable
- [x] Version 1 reading and no-rewrite compatibility are required
- [x] Path containment, uniqueness, schema publication, examples, and round trips are covered

## Security and Constitution

- [x] No target handle, memory access, key extraction, interception driver, or pinning bypass is introduced
- [x] Raw packet and application observations remain authoritative over derived projections
- [x] Missing or uncertain evidence is represented without invented values
- [x] Tests require no account, Internet access, elevation, game, or capture driver
- [x] Deep Capture feature-completion language remains prohibited

## Notes

- Passed before planning. The specification resolves scope using existing product authority: S108 adds forward-compatible correlation representation for later transports but implements joins only for observations emitted by current native handlers.
