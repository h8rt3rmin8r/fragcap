# Artifact Contract Checklist: Deep Capture Bundle and Artifact Reference

**Purpose**: Test whether the S092 requirements completely and unambiguously define the public bundle, sensitivity, omission, and correlation contract
**Created**: 2026-08-28
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [X] CHK001 Are ordinary Capture outputs and the Deep Capture bundle defined as distinct output families? [Completeness, Spec FR-001]
- [X] CHK002 Are all nine current Deep Capture artifact roles included in the requirements? [Completeness, Spec FR-005]
- [X] CHK003 Does every artifact require path, authority, sensitivity, lifetime, required status, and production or omission conditions? [Completeness, Spec FR-006]
- [X] CHK004 Are all three finalized manifest states defined? [Completeness, Spec FR-004]
- [X] CHK005 Are all four current finalized-manifest omission reason tokens required? [Completeness, Spec FR-012]

## Requirement Clarity

- [X] CHK006 Is packet truth assigned unambiguously to `capture.fcapng` without assigning application objects to it? [Clarity, Spec FR-007]
- [X] CHK007 Is application JSON Lines distinguished from packet JSON Lines by producer, record family, and authority? [Clarity, Spec FR-002, FR-008]
- [X] CHK008 Is HAR defined as a lossy projection of only the HTTP fields currently retained? [Clarity, Spec FR-009]
- [X] CHK009 Is the TLS key log defined as proxy-owned, live-consumable, nonempty-only, and secret-adjacent? [Clarity, Spec FR-010]
- [X] CHK010 Are the sidecar authorities defined separately for proxy lifecycle, process chronology, compatibility evidence, and cleanup results? [Clarity, Spec FR-011]

## Requirement Consistency

- [X] CHK011 Are artifact sensitivity and handling requirements consistent with preserving original observations? [Consistency, Spec FR-006, FR-017]
- [X] CHK012 Are manifest omissions explicitly separated from application reasons and cleanup statuses? [Consistency, Spec FR-013]
- [X] CHK013 Are cross-page links required without allowing duplicated full artifact matrices to drift? [Consistency, Spec FR-018]
- [X] CHK014 Is terminology bound to existing glossary entries rather than introducing synonyms? [Consistency, Spec FR-019]

## Acceptance Criteria Quality

- [X] CHK015 Can artifact coverage be measured against a fixed 9-role inventory? [Measurability, Spec SC-001]
- [X] CHK016 Can state and omission coverage be measured against fixed token sets? [Measurability, Spec SC-002]
- [X] CHK017 Can correlation claims be checked against fields emitted by current artifacts? [Measurability, Spec SC-004]
- [X] CHK018 Can synthetic-example hygiene be checked for prohibited local and secret material? [Measurability, Spec SC-005]

## Scenario and Edge-Case Coverage

- [X] CHK019 Are complete sessions with omitted optional artifacts covered? [Coverage, Spec Edge Cases]
- [X] CHK020 Are partial and failed sessions with and without packet truth covered? [Coverage, Spec Edge Cases]
- [X] CHK021 Are missing correlation anchors defined as unavailable joins rather than negative observations? [Coverage, Spec FR-015]
- [X] CHK022 Is early cleanup evidence without a final manifest covered? [Recovery, Spec Edge Cases]
- [X] CHK023 Are metadata-only, unsupported, and process-trace-unavailable records covered without overstating absence? [Coverage, Spec Edge Cases]

## Security and Privacy Requirements

- [X] CHK024 Are sensitive and secret-adjacent artifacts required to carry explicit handling guidance? [Security, Spec FR-010, FR-017]
- [X] CHK025 Is target TLS key extraction explicitly excluded from the key-log description? [Security, Spec FR-010]
- [X] CHK026 Is the synthetic example prohibited from containing local paths, account data, private endpoints, host identifiers, or usable TLS secrets? [Security, Spec FR-016]
- [X] CHK027 Is later residue cleanup described as confirmation-gated without promising deletion of completed operator-owned evidence? [Security, Spec FR-017]

## Dependencies and Scope

- [X] CHK028 Is current runtime behavior named as the authority when older forward-looking prose is broader? [Assumption]
- [X] CHK029 Are the CLI-tree gate and rendered UX audit explicitly left to their owning issues? [Scope, Spec Assumptions]
- [X] CHK030 Is the slice constrained to documentation with no runtime, dependency, workflow, toolchain, release, or master-specification change? [Scope, Spec FR-021]

## Notes

- The checklist uses reviewer-level rigor and emphasizes artifact authority plus sensitive-data handling.
- All 30 requirement-quality checks pass before planning.
