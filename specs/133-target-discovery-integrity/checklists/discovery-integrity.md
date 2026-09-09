# Requirements Quality Checklist: Discovery Integrity

**Purpose**: Test whether the S133 requirements are complete, clear, consistent, and measurable before planning
**Created**: 2026-09-09
**Audience**: Pull-request reviewers and security reviewers

## Requirement Completeness

- [x] CHK001 Are authoritative platform identity and positive local title evidence both defined as independent automatic-registration grants? [Completeness, Spec FR-001/FR-004/FR-005]
- [x] CHK002 Are location-only signals explicitly insufficient for automatic registration? [Completeness, Spec FR-002]
- [x] CHK003 Are default registration, explicit broad discovery, and reconciliation specified as separate operations with separate mutation authority? [Completeness, Spec FR-006/FR-009/FR-010]
- [x] CHK004 Are every existing discovery loss and truncation outcome required to remain visible? [Completeness, Spec FR-007/FR-008]
- [x] CHK005 Are historical residue, ambiguous rows, and user-authored rows all covered by reconciliation requirements? [Completeness, Spec FR-010/FR-011]

## Requirement Clarity

- [x] CHK006 Is the exact Steam-root exclusion boundary stated independently from directory names or detected file contents? [Clarity, Spec FR-003]
- [x] CHK007 Is positive local title evidence distinguished from anti-cheat, DRM, UI, browser, and location evidence? [Clarity, Spec FR-005]
- [x] CHK008 Is automatic-registration refusal observable per candidate and in aggregate? [Clarity, Spec FR-007/FR-009]
- [x] CHK009 Are exact ownership evidence and ambiguity defined without relying on folder-name inference? [Clarity, Spec FR-010/FR-011]
- [x] CHK010 Is explicit confirmation required after preview and before any deletion? [Clarity, Spec FR-012/FR-013]

## Requirement Consistency

- [x] CHK011 Does the platform-manifest grant remain consistent with P-9 when engine and catalog classification are unavailable? [Consistency, Spec FR-004 and Assumptions]
- [x] CHK012 Does broad discovery remain useful while consistently retaining its non-persistent boundary? [Consistency, Spec FR-006/FR-009]
- [x] CHK013 Do default hero and doctor actions share one policy and one accounting vocabulary? [Consistency, Spec FR-008]
- [x] CHK014 Do reconciliation mutation rules agree with preservation of user-owned and ambiguous data? [Consistency, Spec FR-010 through FR-014]

## Acceptance Criteria Quality

- [x] CHK015 Can zero infrastructure candidates and complete unsupported-engine Steam registration be measured in one synthetic tree? [Measurability, Spec SC-001/SC-002]
- [x] CHK016 Can every candidate be reconciled across produced, accepted, refused, registered, and already-present counts? [Measurability, Spec SC-003/SC-004]
- [x] CHK017 Can atomic cleanup and zero unintended deletion be objectively demonstrated? [Measurability, Spec SC-005/SC-006]
- [x] CHK018 Is privacy-preserving real-machine evidence measurable as counts with zero private identifiers? [Measurability, Spec SC-008]

## Scenario and Edge-Case Coverage

- [x] CHK019 Are nested and direct Steam-root layouts both covered? [Coverage, Spec User Story 1 and Edge Cases]
- [x] CHK020 Are unsupported engines, absent catalog entries, and missing installs addressed without false classification? [Coverage, Spec Edge Cases]
- [x] CHK021 Are false-positive markers inside client infrastructure unable to bypass the exact root exclusion? [Coverage, Spec User Story 1 and Edge Cases]
- [x] CHK022 Are duplicate authoritative/path rows and multi-engine aggregates included in repair coverage? [Coverage, Spec User Story 3 and Edge Cases]
- [x] CHK023 Are declined, empty, stale, and failed reconciliation paths all required to leave zero partial mutation? [Coverage, Spec User Story 3 and Edge Cases]

## Safety, Dependencies, and Scope

- [x] CHK024 Are P-4, P-9, and P-10 named as preserved authorities? [Traceability, Spec FR-019]
- [x] CHK025 Is exhaustive machine scanning explicitly excluded? [Scope, Spec FR-020]
- [x] CHK026 Are automatic deletion and folder-name ownership inference explicitly excluded? [Safety, Spec FR-010/FR-020]
- [x] CHK027 Are #374 and #376 prevented from leaking into this slice? [Scope, Spec FR-020]
- [x] CHK028 Is the deliberate S133 roadmap deviation recorded with its authority and downstream consequence? [Traceability, Spec Clarifications and Assumptions]
