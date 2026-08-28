# Journey Requirements Checklist: Verified First Capture and Deep Capture Journeys

**Purpose**: Test whether the S090 requirements completely and unambiguously define the first-run journeys, product truth, safety boundary, and validation evidence
**Created**: 2026-08-28
**Feature**: [spec.md](../spec.md)

## Journey Completeness

- [x] CHK001 Are the starting conditions and successful outcome defined for the first Capture journey? [Completeness, Spec §User Story 1]
- [x] CHK002 Are target discovery, selection, bounded capture, and analyzer use all included in the Capture sequence? [Coverage, Spec §FR-001, Spec §FR-005]
- [x] CHK003 Are the additional prerequisites and successful outcome defined for the known-compatible Deep Capture continuation? [Completeness, Spec §User Story 2]
- [x] CHK004 Are unknown, stale, unsupported, partial, failed, and incomplete-cleanup paths addressed? [Coverage, Spec §User Story 3, Spec §Edge Cases]

## Product-Truth Clarity

- [x] CHK005 Are packet observations, process attribution, application observations, and analyzer aids defined as distinct facts? [Clarity, Spec §FR-005, Spec §FR-009]
- [x] CHK006 Is payload retention qualified by explicit operator scope rather than generalized as always present or always encrypted? [Consistency, Spec §FR-005]
- [x] CHK007 Are current target-listing and doctor specimen obligations measurable enough to reject retired output? [Measurability, Spec §FR-003, Spec §SC-003]
- [x] CHK008 Are each of the current Deep Capture traffic-family limits and inspection conditions named explicitly? [Completeness, Spec §FR-011]
- [x] CHK009 Is the meaning of complete, partial, and failed bundles defined without implying equivalent authority across artifacts? [Clarity, Spec §FR-008, Spec §FR-009]

## Safety and Privacy Boundaries

- [x] CHK010 Are managed launch, current launch-specific evidence, proxy availability, and trust authorization through `--trust-ca` all required before the documented Deep Capture run? [Coverage, Spec §FR-006]
- [x] CHK011 Are warm Steam, direct-executable, unknown-evidence, and certificate-pinning cases explicitly bounded? [Completeness, Spec §FR-007, Spec §FR-012, Spec §FR-015]
- [x] CHK012 Are forbidden silent trust, system-wide proxy fallback, target instrumentation, and target key extraction claims excluded consistently? [Consistency, Spec §FR-015]
- [x] CHK013 Are sensitive artifact handling requirements defined directly for application observations and proxy-owned TLS key logs without relying on the stale output-format page? [Coverage, Spec §FR-010, Spec §FR-014]
- [x] CHK014 Is the synthetic-example requirement broad enough to exclude titles, accounts, local paths, endpoints, host identifiers, and payloads? [Privacy, Spec §FR-013, Spec §SC-005]

## Scope and Evidence Quality

- [x] CHK015 Is S090 separated clearly from the CLI gate, architecture rewrite, bundle reference, and rendered-site audit owned by later issues? [Boundary, Spec §Assumptions]
- [x] CHK016 Are executable examples and illustrative outputs distinguishable and objectively verifiable? [Clarity, Spec §FR-002, Spec §SC-002]
- [x] CHK017 Are documentation, link, encoding, punctuation, and production-export gates included as completion evidence? [Completeness, Spec §FR-017, Spec §SC-006]
- [x] CHK018 Does the specification avoid promising automatic compatibility calibration that the current release does not provide? [Conflict, Spec §FR-012]

## Notes

- All 18 requirements-quality checks pass before technical planning.
