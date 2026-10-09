# Access and Recovery Requirements Checklist: S170

**Purpose**: Review identity, permission and historical recovery requirement quality.

**Created**: 2026-10-09

**Feature**: [S170 specification](../spec.md)

## Identity and permission contract

- [x] CHK001 Are intended recipient identity and the elevated producer explicitly distinguished? [Clarity, FR-001, FR-005]
- [x] CHK002 Are same-account, alternate-account and unprovable-recipient outcomes defined? [Coverage, FR-002, FR-003]
- [x] CHK003 Does ordinary access mean verified enumeration and reads in the recipient's unelevated context? [Measurability, FR-006, SC-001]
- [x] CHK004 Does the artifact population include parents, inheritance, atomic creation and lifecycle sidecars? [Completeness, FR-004]
- [x] CHK005 Is access by unrelated principals explicitly excluded without changing evidence content? [Consistency, FR-005, FR-008]

## Historical repair and errors

- [x] CHK006 Are exact inspection and confirmation requirements separate from ordinary read-only Doctor? [Clarity, FR-007, FR-008]
- [x] CHK007 Are provenance, containment, stale preview and reparse-point refusal requirements explicit? [Coverage, FR-009]
- [x] CHK008 Are failed access claims and actual partial repair outcomes objectively testable? [Measurability, FR-010, FR-011, SC-003]
- [x] CHK009 Are idempotent retries and byte preservation required for historical repair? [Completeness, FR-008, FR-011, SC-002]
- [x] CHK010 Are collection, container purge and trust/proxy recovery explicitly separate authorities? [Consistency, FR-011, scope]
- [x] CHK011 Do controlled evidence and documentation cover every issue criterion without owner-artifact mutation? [Acceptance quality, FR-012, FR-013, SC-004]
