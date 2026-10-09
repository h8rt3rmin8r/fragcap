# Retention Requirements Checklist: S171

**Purpose**: Author and PR reviewer requirements quality gate for destructive lifecycle operations.

**Created**: 2026-10-09

## Completeness and consistency

- [x] CHK001 Are finite age/count/byte defaults and collection ordering explicit? [Spec FR-001]
- [x] CHK002 Are historical, saved and custom retention promises reconciled? [Spec FR-002]
- [x] CHK003 Are automatic boundaries distinct from read-only commands? [Spec FR-003]
- [x] CHK004 Is whole-session eligibility independent from one terminal row? [Spec FR-004]
- [x] CHK005 Are inventory refusal reasons and byte accounting requirements defined? [Spec FR-005, FR-009]
- [x] CHK006 Is stale-preview refusal required before effects? [Spec FR-006]
- [x] CHK007 Are empty-container preservation and separately authorized purge consistent? [Spec FR-007, FR-010]
- [x] CHK008 Are path replacement, reparse and outside-alias boundaries specified? [Spec FR-008]
- [x] CHK009 Are partial retirement and idempotent retries defined? [Spec FR-009]
- [x] CHK010 Are bounded scanning and actionable backlog capacity measurable? [Spec FR-011, SC-002]
- [x] CHK011 Do tests cover every lifecycle class without real owner artifacts? [Spec FR-012]
- [x] CHK012 Are public contracts and limits required in the same slice? [Spec FR-013]
