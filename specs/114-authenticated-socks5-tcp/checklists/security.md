# Security Requirements Checklist: Authenticated SOCKS5 TCP Routing

**Purpose**: Review the authorization, destination, lifecycle, and evidence requirements before implementation
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md)

## Admission And Secret Scope

- [x] CHK001 Is the sole accepted authentication method and exact session credential source specified? [Completeness, Spec FR-002, FR-003]
- [x] CHK002 Is refusal required before every upstream, DNS, classification, and payload effect? [Ordering, Spec FR-004]
- [x] CHK003 Are constant-time comparison, transient secret handling, and non-disclosure requirements explicit? [Security, Spec FR-003, FR-016]

## Destination And DNS Authority

- [x] CHK004 Are all supported address forms and port constraints specified? [Coverage, Spec FR-006]
- [x] CHK005 Is proxy-owned DNS and per-address policy evaluation unambiguous? [Authority, Spec FR-008]
- [x] CHK006 Are listener aliasing, local destinations, and unsupported commands still governed by refusal? [Coverage, Spec FR-007, FR-008]

## Bounded Runtime And Truth

- [x] CHK007 Are parsing, classification, forwarding, half-close, cancellation, and buffer limits defined? [Completeness, Spec FR-005, FR-011, FR-012]
- [x] CHK008 Are all failure and loss classes tied to named evidence or terminal outcomes? [Traceability, Spec FR-014, FR-015]
- [x] CHK009 Does the scope prohibit unauthenticated, global, process-access, DNS-bypass, UDP, and overclaim paths? [Boundary, Spec FR-019]
- [x] CHK010 Are offline security and tenancy tests required for unrelated local clients and policy-refused destinations? [Coverage, Spec FR-018]

## Notes

- All checks pass. Implementation remains blocked if any item regresses.
