# Truth and Privacy Checklist: Deep Capture Compatibility Documentation

**Purpose**: Test whether the requirements fully define truthful compatibility
claims and safe public documentation

**Created**: 2026-08-26

**Feature**: [spec.md](../spec.md)

**Note**: This checklist evaluates the requirements, not the implementation.

## Requirement Completeness

- [x] CHK001 Are Capture and Deep Capture requirements separately defined
  before protocol-specific outcomes are stated? [Completeness, Spec FR-001]
- [x] CHK002 Are all seven required traffic families named individually rather
  than grouped under a broader TLS or HTTP claim? [Completeness, Spec FR-002]
- [x] CHK003 Does every traffic-family requirement cover visibility,
  inspectability, prerequisites, blockers, and outputs? [Completeness, Spec
  FR-003]
- [x] CHK004 Are the target-specific matrix and the public traffic reference
  defined as different deliverables with different data sources? [Completeness,
  Spec FR-006]
- [x] CHK005 Are refresh and contribution requirements included alongside the
  read-only display requirements? [Completeness, Spec FR-013 through FR-015]

## Requirement Clarity

- [x] CHK006 Is `full` inspectability bounded so it cannot be read as complete
  payload, header, body, or frame retention? [Clarity, Spec FR-005]
- [x] CHK007 Is unknown defined as absence of evidence rather than a negative or
  positive compatibility verdict? [Clarity, Spec FR-010 and Key Entities]
- [x] CHK008 Is stale evidence defined through both the stale marker and the
  explicit stale-observation source? [Clarity, Spec FR-012]
- [x] CHK009 Is the no-inference rule explicit about platform, engine,
  executable, and title metadata? [Clarity, Spec FR-010]
- [x] CHK010 Is deterministic ordering required without implying that order
  selects a winning fact? [Clarity, Spec FR-008 and FR-009]

## Requirement Consistency

- [x] CHK011 Do the matrix requirements preserve historical evidence while the
  refresh requirements avoid silent replacement? [Consistency, Spec FR-009 and
  FR-014]
- [x] CHK012 Does the side-effect-free viewing requirement align with the stated
  explicit measurement path for refresh? [Consistency, Spec FR-013 and FR-014]
- [x] CHK013 Do the protocol requirements align with the prohibition on pinning
  bypass, QUIC decryption, custom dissection, and target key extraction?
  [Consistency, Spec FR-004]

## Acceptance Criteria Quality

- [x] CHK014 Can coverage of every traffic family be measured without relying
  on subjective words such as comprehensive or robust? [Measurability, Spec
  SC-001]
- [x] CHK015 Can evidence-source and freshness coverage be measured from
  placeholder-only records? [Measurability, Spec SC-002]
- [x] CHK016 Is deterministic handling of conflicting evidence objectively
  defined across repeated runs? [Measurability, Spec SC-003]

## Scenario And Edge Coverage

- [x] CHK017 Are current, stale, unknown, repeated, conflicting, and partially
  populated fact scenarios all covered? [Coverage, User Story 2 and Edge Cases]
- [x] CHK018 Are HTTP handshakes with unsupported WebSocket frames distinguished
  from complete WebSocket inspection? [Coverage, Edge Cases]
- [x] CHK019 Is packet visibility for application-unsupported traffic preserved
  as a separate outcome? [Coverage, Edge Cases]
- [x] CHK020 Are missing optional provenance fields prevented from silently
  changing freshness or certainty? [Coverage, Edge Cases]

## Privacy And Publication Boundaries

- [x] CHK021 Are all prohibited public data classes named, including real local
  titles, accounts, paths, endpoints, tokens, and host identifiers? [Security,
  Spec FR-015]
- [x] CHK022 Is placeholder-only committed test and fixture data required, not
  merely recommended? [Security, Spec FR-015 and SC-005]
- [x] CHK023 Is the absence of a checked-in title matrix explicit so local facts
  cannot leak through documentation? [Security, Clarifications]

## Dependencies And Scope

- [x] CHK024 Are the existing fact store and shipped MVP named as sources of
  truth? [Dependency, Assumptions]
- [x] CHK025 Are new protocol support, active probing, fact editing, community
  synchronization, and native proxy work explicitly excluded? [Scope,
  Assumptions]

## Notes

- All requirements-quality checks passed before planning.
- The checklist treats privacy and truthfulness as release gates because a
  technically valid matrix can still be harmful if it publishes local evidence
  or turns incomplete facts into a compatibility verdict.
