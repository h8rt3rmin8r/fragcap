# Safety Requirements Checklist: Guided Reachability Calibration Front Door

**Purpose**: Review requirement quality at the PR boundary for process, authorization, trust, evidence, and compatibility risks
**Created**: 2026-09-09
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are all permitted and prohibited process interactions defined for cold detection and warm restart? [Completeness, Spec FR-003, FR-010, FR-011]
- [x] CHK002 Are all effect boundaries that must remain after authorization named explicitly? [Completeness, Spec FR-007, FR-012, FR-015]
- [x] CHK003 Are trust-bearing and sensitive artifact effects excluded from reachability with no implicit exception? [Completeness, Spec FR-008, FR-019]
- [x] CHK004 Are target resolution and registration responsibilities separated without creating a second target path? [Completeness, Spec FR-001, FR-002]
- [x] CHK005 Are fact selection, exact context, append-only persistence, and non-positive evidence reasons covered? [Completeness, Spec FR-004, FR-006, FR-007]

## Requirement Clarity

- [x] CHK006 Is the maximum number and kind of session attempt unambiguous? [Clarity, Spec FR-006]
- [x] CHK007 Is the distinction between ready, operator action, selected reachability, and refusal objective? [Clarity, Spec FR-009, FR-010, FR-012, FR-013]
- [x] CHK008 Is the same-process structured authorization requirement explicit? [Clarity, Spec FR-015]
- [x] CHK009 Is a complete process snapshot distinguished from failure or partial knowledge? [Clarity, Spec FR-003]
- [x] CHK010 Are generated command identity and quoting assumptions explicit and testable? [Clarity, Spec FR-009, FR-016]

## Requirement Consistency

- [x] CHK011 Do the guided command requirements consume S139 policy instead of duplicating launch-case decisions? [Consistency, Spec FR-005]
- [x] CHK012 Does execution remain consistent with S134 authorization and the existing low-level path? [Consistency, Spec FR-007, FR-015]
- [x] CHK013 Do no-effect outcomes remain consistent with P-9 truthfulness and the exact S121 fact model? [Consistency, Spec FR-006, FR-009, FR-013]
- [x] CHK014 Do the scope exclusions consistently leave later #380 orchestration work open? [Consistency, Spec FR-019]

## Scenario and Edge-Case Coverage

- [x] CHK015 Are direct, Steam, and publisher cold and warm paths all specified? [Coverage, Spec User Stories 1 and 2]
- [x] CHK016 Are missing, stale, legacy, mismatched, negative, conflicting, and positive routing facts all addressed? [Coverage, Spec FR-006, FR-009]
- [x] CHK017 Are no-match, ambiguity, malformed topology, and inventory failure defined before effects? [Coverage, Spec FR-012, FR-018]
- [x] CHK018 Are authorization decline, identifier mismatch, interruption, timeout, target drift, and non-cold retry outcomes covered? [Coverage, Spec Edge Cases, FR-018]
- [x] CHK019 Are human and structured no-effect and execution paths both specified? [Coverage, Spec FR-014, FR-015]

## Acceptance Criteria Quality

- [x] CHK020 Can every user story be validated without a live game, elevation, or trust mutation? [Measurability, Spec Independent Tests]
- [x] CHK021 Can absence of pre-authorization and no-effect adapter calls be measured directly? [Measurability, Spec SC-002]
- [x] CHK022 Can every generated next command be parsed and identity-checked? [Measurability, Spec SC-003]
- [x] CHK023 Does the repository gate define an objective no-dependency, no-migration, and no-prohibited-capability completion result? [Measurability, Spec SC-005]
