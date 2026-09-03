# Calibration Safety Checklist: Complete Native Calibration Matrix

**Purpose**: Test whether S121 requirements completely specify scope, evidence truth, migration, and authorization safety before implementation
**Created**: 2026-09-03
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are all dimensions that can change calibration meaning explicitly listed? [Completeness, Spec FR-001]
- [x] CHK002 Are supported and unavailable launch, routing, address-family, and protocol combinations required to have deterministic outcomes? [Coverage, Spec FR-002]
- [x] CHK003 Are preauthorization plan contents and effect boundaries fully enumerated? [Completeness, Spec FR-003]
- [x] CHK004 Are positive, negative, partial, lost, and unpersisted evidence classes all specified? [Completeness, Spec FR-005, FR-006, FR-015]
- [x] CHK005 Are migration and retest requirements defined for existing and newly appended rows? [Completeness, Spec FR-007, FR-009]

## Requirement Clarity

- [x] CHK006 Is exact case identity defined without relying on an aggregate compatibility verdict? [Clarity, Spec FR-001, FR-012]
- [x] CHK007 Is protocol applicability distinguished clearly between routing and protocol-specific facts? [Clarity, Spec FR-013]
- [x] CHK008 Is current evidence defined by explicit state and applicable dimensions rather than elapsed time? [Clarity, Spec FR-010]
- [x] CHK009 Is latest-row precedence limited to current rows applicable to the same prerequisite? [Clarity, Spec FR-011]
- [x] CHK010 Is target-version evidence explicitly optional only when trustworthy version evidence is unavailable? [Clarity, Spec FR-001, Assumptions]

## Requirement Consistency

- [x] CHK011 Do append-only conflict preservation and latest-applicable eligibility coexist without rewriting history? [Consistency, Spec FR-007, FR-011]
- [x] CHK012 Do calibration facts and S120 classification eligibility use one protocol authority? [Consistency, Spec FR-005, Assumptions]
- [x] CHK013 Do human, structured, artifact, manifest, terminal, and target-detail requirements name the same case identity? [Consistency, Spec FR-014]
- [x] CHK014 Do legacy visibility and legacy ineligibility preserve both historical truth and current safety? [Consistency, Spec FR-009]

## Scenario Coverage

- [x] CHK015 Are cold direct, cold Steam, and cold publisher-chain launch requirements covered without reopening warm paths? [Coverage, Spec Assumptions]
- [x] CHK016 Are IPv4 and IPv6 disagreement and single-dimension mismatch scenarios specified? [Coverage, Spec Edge Cases, SC-002]
- [x] CHK017 Are protocol mismatch and unrelated retained observations prevented from producing selected-case facts? [Coverage, Spec US1 Scenario 3]
- [x] CHK018 Are interruption, timeout, observation loss, and persistence failure kept distinct from compatibility outcomes? [Coverage, Spec Edge Cases, FR-006, FR-015]
- [x] CHK019 Is the controlled matrix required to avoid real accounts, remote services, capture drivers, and trust mutation? [Coverage, Spec FR-016]

## Security and Scope Boundaries

- [x] CHK020 Are explicit selection, plan visibility, confirmation, finite ownership, cleanup, and auditability all mandatory? [Security, Spec FR-004]
- [x] CHK021 Are system proxy fallback, silent trust, pinning bypass, process access, and target key extraction explicitly prohibited? [Security, Spec FR-004]
- [x] CHK022 Is promotion across every applicable dimension prohibited without direct proof? [Security, Spec FR-012]
- [x] CHK023 Is bypass-policy implementation explicitly excluded for issue #318? [Scope, Spec FR-018]
- [x] CHK024 Is any Deep Capture completion claim prohibited until issue #334 closes? [Scope, Spec FR-018]

## Acceptance Criteria Quality

- [x] CHK025 Can matrix closure be measured as complete coverage with zero skipped cases? [Measurability, Spec SC-001]
- [x] CHK026 Can dimension isolation be measured through single-variable permutations? [Measurability, Spec SC-002]
- [x] CHK027 Can migration fidelity and evidence conservation be measured without subjective judgment? [Measurability, Spec SC-003, SC-004]
- [x] CHK028 Can cross-surface agreement and dependency neutrality be objectively verified? [Measurability, Spec SC-005, SC-006]

## Notes

- All 28 requirement-quality checks pass. This checklist is a formal PR-review gate for the calibration and security boundaries.
