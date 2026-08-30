# Native Proxy Security Requirements Checklist

**Purpose**: Review whether S103 completely and unambiguously defines selected-session isolation, upstream safety, certificate ownership, loss accounting, and controlled-lab boundaries.

**Created**: 2026-08-30

**Feature**: [spec.md](../spec.md)

## Listener Isolation

- [x] CHK001 Are IPv4, IPv6, wildcard, remote-interface, port-reuse, replay, and local-race boundaries explicitly specified? [Coverage, Spec FR-001 through FR-004]
- [x] CHK002 Is authentication required before upstream work and payload collection for every admitted client? [Ordering, Spec FR-002]
- [x] CHK003 Are capability transport, comparison, secrecy, lifetime, and unsupported-protocol behavior unambiguous? [Clarity, Spec FR-002a, FR-003]
- [x] CHK004 Are refusals required to remain payload-free while still producing exact accounting? [Consistency, Spec FR-004, SC-001, SC-002]

## Upstream Policy

- [x] CHK005 Are authority parsing and identity preservation requirements defined for names, IPv4, IPv6, and malformed inputs? [Completeness, Spec FR-005]
- [x] CHK006 Are DNS, connect, read, write, cancellation, and shutdown budgets independently measurable with an exact scheduler tolerance? [Clarity, Spec FR-006, FR-006a, SC-003]
- [x] CHK007 Are recursion, rebinding, private destinations, controlled-test grants, and mixed resolution results covered without a silent exception? [Coverage, Spec FR-007, FR-007a]
- [x] CHK008 Is secure upstream verification mandatory with typed failure and no downgrade or fabricated application result? [Safety, Spec FR-008, SC-004]

## Certificate and Trust Ownership

- [x] CHK009 Are per-session authority identity, non-reuse, validity, provenance, storage protection, partial creation, and cleanup obligations defined? [Completeness, Spec FR-009 through FR-011]
- [x] CHK010 Are leaf identity, use, validity, concurrency, count, byte, lifetime, rotation, eviction, and malformed-input requirements defined? [Coverage, Spec FR-012, FR-013]
- [x] CHK011 Is trust a separate explicitly authorized exact-thumbprint action limited to current-user Root? [Safety, Spec FR-011, FR-014]
- [x] CHK012 Are duplicates, wrong-store entries, same-subject different-key entries, denial, interruption, partial cleanup, and unrelated-certificate preservation specified? [Edge Cases, Spec FR-015]

## Observation Integrity

- [x] CHK013 Are event families, versioning, correlation, deterministic order, time, provenance, payload ownership, malformed data, and unknown data specified? [Completeness, Spec FR-016, FR-017, FR-019]
- [x] CHK014 Are queue and payload bounds paired with exact dropped, truncated, refused, unparsed, and projection-gap accounting? [Consistency, Spec FR-018 through FR-020]
- [x] CHK015 Is drop-oldest overflow behavior explicit and required to invalidate completeness without rewriting later order? [Clarity, Spec FR-018a]

## Controlled Lab and Scope

- [x] CHK016 Are all required protocol families and positive, refusal, malformed, timeout, cancellation, disconnect, and cleanup scenarios named? [Coverage, Spec FR-021, FR-022]
- [x] CHK017 Are offline, synthetic-data, no-account, no-game, no-driver, no-elevation, and platform-isolation requirements explicit? [Safety, Spec FR-023 through FR-025]
- [x] CHK018 Does the lab distinguish fixture availability from native proxy support and prevent unsupported behavior from becoming a completion claim? [Honesty, Spec FR-024, FR-026]
- [x] CHK019 Are #290, production cutover, full protocol inspection, and every constitutionally prohibited technique clearly outside S103? [Scope, Spec FR-026 through FR-028]

## Notes

- This checklist validates the written requirements. All nineteen items pass after the autopilot clarification session.
