# Security Requirements Checklist: Native HTTP/TLS Production Cutover

**Purpose**: Review the completeness, clarity, consistency, and measurability of S104 security requirements before implementation
**Created**: 2026-08-30
**Audience**: Pull-request reviewers

## Authorization and Scope

- [x] CHK001 Are client-admission requirements defined before every class of upstream or retained-payload work? [Completeness, Spec §FR-004]
- [x] CHK002 Is capability scope explicit for session generation, selected launch, artifact exclusion, diagnostic redaction, and cleanup? [Clarity, Spec §FR-005]
- [x] CHK003 Are unauthenticated, stale, and cross-session credentials addressed as distinct negative scenarios? [Coverage, Spec §Edge Cases]
- [x] CHK004 Is the absence of system-wide fallback and unrelated-client inspection an explicit boundary? [Consistency, Spec §Assumptions]

## Protocol Safety

- [x] CHK005 Are malformed, ambiguous, oversized, stalled, and partial HTTP inputs covered by finite refusal requirements? [Completeness, Spec §FR-007]
- [x] CHK006 Does the spec require proxy transformations to remain visible in evidence rather than silently replacing observations? [Consistency, Spec §FR-008]
- [x] CHK007 Are destination-policy edge cases defined for self-routing, local addresses, mixed resolutions, and rebinding between attempts? [Coverage, Spec §Edge Cases]
- [x] CHK008 Are all required connection limits named, including headers, bodies, idle time, connections, and tasks? [Clarity, Spec §FR-007]

## TLS and Private Material

- [x] CHK009 Is explicit operator trust approval required before client-facing TLS inspection? [Completeness, Spec §FR-009]
- [x] CHK010 Are client-facing identity and upstream validation defined as separate boundaries? [Clarity, Spec §FR-009]
- [x] CHK011 Is fail-closed hostname and chain validation stated without a permissive escape hatch? [Consistency, Spec §FR-010]
- [x] CHK012 Are pinning, silence, unsupported clients, alerts, timeouts, and unknown failures prevented from becoming fabricated success evidence? [Coverage, Spec §FR-012]
- [x] CHK013 Are private session materials required to be released or zeroized and excluded from durable output unless separately authorized? [Completeness, Spec §FR-005, §FR-014]

## Failure Truth and Verification

- [x] CHK014 Are cleanup requirements idempotent, bounded, residue-aware, and complete across every acquired resource? [Measurability, Spec §FR-014]
- [x] CHK015 Are every refusal, truncation, parse failure, dropped observation, and upstream-attempt boundary required to reconcile? [Completeness, Spec §FR-007, §SC-002, §SC-005]
- [x] CHK016 Are controlled verification requirements isolated from real trust mutation, games, drivers, remote services, and external proxy tooling? [Clarity, Spec §FR-015]
- [x] CHK017 Is the repository regression gate bounded tightly enough to reject production external proxy behavior while retaining historical specifications? [Clarity, Spec §FR-016]
- [x] CHK018 Are incomplete native observations prohibited from being labeled complete across bundle, event, fact, and terminal contracts? [Consistency, Spec §FR-017]
