# Discovery Correctness Checklist: Known-Roots Discovery Corrections

**Purpose**: Verify that S077's requirements preserve truthful discovery, bounded traversal, path identity, and fixture privacy
**Created**: 2026-08-26
**Feature**: [spec.md](../spec.md)

## Classification

- [x] CHK001 The specification defines title, container, and miss as distinct outcomes.
- [x] CHK002 The container signal counts distinct engine products rather than raw findings.
- [x] CHK003 Anti-cheat, DRM, and repeated same-engine findings cannot independently create a container verdict.
- [x] CHK004 Title hits retain stop-on-hit behavior.

## Accounting

- [x] CHK005 Descended containers and depth-limited containers have separate named outcomes.
- [x] CHK006 Both new outcomes participate in the conservation invariant.
- [x] CHK007 A depth-limited container names the affected directory and reduced coverage.
- [x] CHK008 Existing classification coverage warnings remain visible.

## Paths And Privacy

- [x] CHK009 Real filesystem root composition uses the host-native path convention.
- [x] CHK010 Candidate identity and install root share the corrected path value.
- [x] CHK011 The separator-neutral root list remains shared with fixtures.
- [x] CHK012 Regression fixtures prohibit real local titles and operator-identifying paths.

## Scope

- [x] CHK013 The shallow descent limit remains unchanged.
- [x] CHK014 Existing stored rows are not silently rewritten.
- [x] CHK015 Deep scanning and broader folder-name heuristics remain out of scope.

## Notes

- All checklist obligations are represented by functional requirements and measurable outcomes before planning.
