# Architecture Documentation Requirements Checklist

**Purpose**: Validate the completeness, precision, and testability of the S091 architecture-page requirements before planning.
**Created**: 2026-08-28
**Feature**: [spec.md](../spec.md)

## Mode Boundaries

- [x] CHK001 Are Capture and Deep Capture required to have separate execution views rather than a blended pipeline? [Completeness, Spec FR-001]
- [x] CHK002 Does the Capture requirement state the passive acquisition, external attribution, retention, loss, output, and analyzer boundaries? [Completeness, Spec FR-002]
- [x] CHK003 Does the Deep Capture requirement identify every acceptance-critical execution stage from selection through cleanup? [Completeness, Spec FR-003]
- [x] CHK004 Is the current real-target compatibility gate stated with exact evidence values and refused launch states? [Clarity, Spec FR-004]
- [x] CHK005 Is ordinary Capture explicitly preserved when Deep Capture traffic cannot be inspected? [Consistency, Spec FR-008]

## Trust and Security

- [x] CHK006 Is CA trust authorization distinguished from a nonexistent second interactive prompt? [Clarity, Spec FR-005]
- [x] CHK007 Are the owner, store scope, lifetime, and cleanup evidence of the trust change required? [Completeness, Spec FR-005]
- [x] CHK008 Are routing, CA acceptance, pinning, and protocol limitations all bounded without implying bypass capability? [Coverage, Spec FR-008]
- [x] CHK009 Are prohibited fallback and covert target techniques excluded explicitly enough to audit? [Clarity, Spec FR-009]
- [x] CHK010 Does Npcap acquisition language require both explicit confirmation and the precise non-bundling boundary? [Consistency, Spec FR-010]

## Evidence and Dependencies

- [x] CHK011 Are packet truth, proxy observations, proxy-owned analyzer material, correlation metadata, and audit evidence required to remain distinct? [Completeness, Spec FR-006]
- [x] CHK012 Is correlation bounded to structured anchors without requiring fabricated matches? [Clarity, Spec FR-007]
- [x] CHK013 Are Npcap, mitmdump, and external analyzers assigned distinct architectural roles? [Completeness, Spec FR-011]
- [x] CHK014 Are required cross-references enumerated while preserving issue #248's ownership of the exhaustive artifact matrix? [Scope, Spec FR-012]

## Presentation and Verification

- [x] CHK015 Are diagram readability requirements measurable rather than subjective? [Measurability, Spec FR-013, SC-005]
- [x] CHK016 Are synthetic-content and documentation-only scope constraints explicit? [Scope, Spec FR-014, FR-015]
- [x] CHK017 Do the success criteria cover mode classification, security-sensitive actions, evidence authority, forbidden claims, diagram size, and repository gates? [Coverage, Spec SC-001 through SC-006]
- [x] CHK018 Are edge cases defined for refused preflight, unsupported launch state, routing bypass, trust rejection, partial sidecars, cleanup failure, and missing dependencies? [Coverage, Edge Cases]

## Notes

- Reviewed against GitHub issue #247, the constitution, the master specification, the merged S090 behavior, and the existing public architecture page.
- All items pass. No clarification marker remains.
