# Release and Security Requirements Checklist: S152

**Purpose**: Requirements-quality review of approval authority and release-state truth before planning.

**Created**: 2026-09-15

**Feature**: [spec.md](../spec.md)

## Authority and Coverage

- [x] CHK001 Is the sole operator reviewer identity explicit? [Clarity, Spec FR-001]
- [x] CHK002 Is the self-review tradeoff explained without inventing another reviewer? [Consistency, Spec FR-001, Assumptions]
- [x] CHK003 Is administrator bypass explicitly retained at the operator's direction and distinguished from normal review? [Completeness, Spec FR-001]
- [x] CHK004 Are tag-only allowances distinguished from branch policies and exact version matching? [Clarity, Spec FR-002]
- [x] CHK005 Are environment identity, secrets and unrelated settings preserved? [Coverage, Spec FR-003]
- [x] CHK006 Are malformed, missing, partial, oversized and unsupported protection records addressed? [Coverage, Spec FR-004, Edge Cases]
- [x] CHK007 Is verification distinguished from actual deployment approval? [Consistency, Spec FR-004, FR-005]
- [x] CHK008 Are fresh checks required at both release and registry boundaries? [Completeness, Spec FR-004]

## Release Truth and Completion

- [x] CHK009 Are all ten candidate crates and embedded outputs included? [Completeness, Spec FR-006]
- [x] CHK010 Is prepared applicability separated from actual publication identity? [Clarity, Spec FR-006, FR-007]
- [x] CHK011 Are historical v0.10.0 tags, assets, evidence and unsigned policy preserved? [Coverage, Spec FR-007]
- [x] CHK012 Are agent-installed product, real games and trust mutation excluded? [Coverage, Spec FR-008]
- [x] CHK013 Are every review comment and current-head checks included in handoff criteria? [Measurability, Spec FR-009, SC-003]
- [x] CHK014 Is the two-round review bound explicit? [Clarity, Spec FR-009]
- [x] CHK015 Are human merge, tagging, publication and approval separate actions? [Consistency, Spec FR-010, SC-004]
- [x] CHK016 Are external acceptance and deferred backlog explicitly kept open? [Completeness, Spec FR-010]

## Notes

Created a new 16-item reviewer checklist with every item traceable. All requirements-quality items pass. The installed prerequisite helper requires plan.md; the governing constitution instead requires checklist before plan, so this pre-plan requirements review proceeds without fabricating a plan. Recheck the helper after the plan exists. Configuration recovery stops on unexpected unrelated drift rather than silently overwriting it.
