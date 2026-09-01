# Security Requirements Checklist: Cold Platform-Client Ownership

**Purpose**: Review the completeness, clarity, and consistency of S112 authority, scope, and evidence requirements
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md)

## Authority And Scope

- [x] CHK001 Are pre-effect authority and refusal requirements defined for every platform preparation outcome? [Completeness, Spec FR-001, FR-003]
- [x] CHK002 Is the permitted launch scope limited to one exact stored target and one exact local platform executable? [Clarity, Spec FR-002, FR-004]
- [x] CHK003 Are shell, global proxy, process-handle, injection, hook, and image-modification boundaries explicit? [Coverage, Spec FR-004]
- [x] CHK004 Is the observe-before-dispatch authority transition objectively specified? [Measurability, Spec FR-005]
- [x] CHK005 Are unsupported and uncertain platform identities required to refuse before effects? [Consistency, Spec FR-003, FR-017]

## Process Ownership

- [x] CHK006 Are exact executable, creation-time ancestry, and unique stage ownership all required together? [Completeness, Spec FR-007]
- [x] CHK007 Does the specification define how same-named warm processes are treated under limited startup evidence? [Clarity, Spec FR-003]
- [x] CHK008 Are escaped descendants prevented from acquiring both stage and terminal identity? [Coverage, Spec FR-008]
- [x] CHK009 Are platform exit, ambiguity, watcher loss, dispatch failure, missing client, and timeout distinct outcomes? [Completeness, Spec FR-009]
- [x] CHK010 Are every startup and acquisition path bounded by displayed deadlines? [Measurability, Spec FR-010]

## Evidence Integrity

- [x] CHK011 Are routing and environment propagation defined as separate non-substitutable observations? [Consistency, Spec FR-011]
- [x] CHK012 Is propagation confirmation tied to exact terminal-client proxy evidence beneath owned ancestry? [Clarity, Spec FR-012]
- [x] CHK013 Are platform and helper observations prohibited from being folded into client evidence? [Coverage, Spec FR-013]
- [x] CHK014 Are dropped and unlocalizable observations required to carry named exact loss accounting? [Completeness, Spec FR-014]
- [x] CHK015 Does the specification prohibit committed fixtures from exposing credentials, account identity, library ownership, and real game data? [Security, Spec FR-015, SC-007]

## Compatibility And Boundaries

- [x] CHK016 Is backward compatibility required for ordinary Capture and non-platform launch paths? [Coverage, Spec FR-016]
- [x] CHK017 Is the reusable platform adapter boundary specified without introducing a second target path? [Consistency, Spec FR-002, FR-006]
- [x] CHK018 Are warm restart, generic transports, general calibration, and final completion explicitly outside scope? [Completeness, Spec Assumptions]

## Notes

- All requirements-quality checks pass. Implementation behavior is validated separately by the task plan and test suite.
