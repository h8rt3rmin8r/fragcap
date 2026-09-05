# Windows Integration Requirements Checklist

**Purpose**: Review the completeness, clarity, consistency, and measurability of S129 Windows integration requirements before implementation

**Created**: 2026-09-04

**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are all Windows-only completion domains named in the closed registry requirement? [Completeness, Spec FR-001]
- [x] CHK002 Are row identity, setup, outcome, evidence, effect, cleanup, and publication fields all required? [Completeness, Spec FR-002]
- [x] CHK003 Are both hosted-runner and physical-host evidence responsibilities defined? [Completeness, Spec FR-018, FR-019]
- [x] CHK004 Are non-admin, consent-denied, elevation-denied, Npcap-present, and Npcap-absent scenarios covered? [Coverage, Spec FR-001]
- [x] CHK005 Are IPv4, IPv6, trust, recovery, key-log, analyzer, process, firewall, staged-layout, and residue scenarios covered? [Coverage, Spec FR-001]

## Requirement Clarity

- [x] CHK006 Is a required row distinguished clearly from a reviewed expected-unavailable or expected-refusal outcome? [Clarity, Spec FR-004]
- [x] CHK007 Is capability drift defined as an incomplete run rather than a skip or pass? [Clarity, Spec FR-005]
- [x] CHK008 Is current-user trust scope separated explicitly from prohibited machine-wide mutation? [Clarity, Spec FR-007, FR-009]
- [x] CHK009 Are build-time Npcap SDK presence and installed Npcap runtime presence distinguished? [Clarity, Spec FR-010]
- [x] CHK010 Is staged installed-layout evidence separated explicitly from final MSI/archive certification? [Clarity, Spec FR-011]

## Requirement Consistency

- [x] CHK011 Do effect-bearing requirements consistently preserve explicit authorization, exact ownership, and reversibility? [Consistency, Spec FR-006 to FR-009]
- [x] CHK012 Do public evidence requirements preserve exact failures while excluding secret-bearing material? [Consistency, Spec FR-016, FR-017]
- [x] CHK013 Does the physical-evidence rule agree with the prohibition on simulated success and skipped required rows? [Consistency, Spec FR-004, FR-019]
- [x] CHK014 Does the scope boundary agree across assumptions, staged-layout requirements, and the no-completion-claim rule? [Consistency, Spec FR-011, FR-020]

## Acceptance Criteria Quality

- [x] CHK015 Can matrix completeness be measured as exactly one terminal result for every required identity? [Measurability, Spec SC-001]
- [x] CHK016 Can residue be measured across every declared owned-effect category? [Measurability, Spec SC-003]
- [x] CHK017 Can authority-denial behavior be measured through zero mutation and zero unexpected prompts? [Measurability, Spec SC-004]
- [x] CHK018 Can report hygiene be measured against seeded prohibited value classes? [Measurability, Spec SC-009]
- [x] CHK019 Are hosted runtime and physical-evidence currency bounds explicit? [Measurability, Spec SC-010]

## Scenario and Edge-Case Coverage

- [x] CHK020 Are host capability loss and preflight-to-execution drift addressed? [Coverage, Edge Cases]
- [x] CHK021 Are child timeout, missing report, and visible-console hazards addressed? [Coverage, Edge Cases]
- [x] CHK022 Are exact pre-existing trust and mismatched-certificate cases addressed? [Coverage, Edge Cases]
- [x] CHK023 Are interrupted cleanup and unrelated-state preservation addressed? [Coverage, Spec FR-013, Edge Cases]
- [x] CHK024 Are analyzer partial success and binary identity mismatch addressed? [Coverage, Edge Cases]

## Dependencies and Boundaries

- [x] CHK025 Is reuse of existing production authorities required instead of test-only reimplementation? [Dependency, Assumptions]
- [x] CHK026 Is Npcap redistribution prohibited for committed and uploaded artifacts? [Boundary, Spec FR-010]
- [x] CHK027 Are external traffic, system proxy, firewall mutation, and denylisted instrumentation excluded? [Boundary, Spec FR-007]
- [x] CHK028 Are recurring schedules, background monitors, packaging completion, supply-chain completion, and feature-completion claims excluded? [Boundary, Spec FR-020]

## Notes

- All 28 requirements-quality checks pass.
- This is a formal release-gate checklist for specification authors and pull-request reviewers.
