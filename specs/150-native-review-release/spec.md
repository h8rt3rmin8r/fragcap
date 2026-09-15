# Feature Specification: Native Documentation and Reviewable Release Handoff

**Feature Branch**: `release/0.10.0` (repository-required release preparation branch)

**Created**: 2026-09-15

**Status**: Draft

**Input**: S150 bundles native documentation alignment under #331, independent-review readiness under #333, and preparation of a fresh operator-published release. It does not perform or certify the independent audit, run sensitive installed software, reproduce #372 against a real host, or close #334.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Follow the actual native product contract (Priority: P1)

An operator can move from separate prerequisites through discovery, guided calibration, authorized Deep Capture, retained evidence, and exact recovery using guidance that agrees with the product rather than historical external-backend instructions.

**Why this priority**: Native implementation and session-presentation slices have landed, but documentation needs to describe their combined contract honestly.

**Independent Test**: Parse published command examples without executing effects, validate controlled artifact specimens, and build and check the documentation site.

**Acceptance Scenarios**:

1. **Given** the native implementation, **When** an operator reads architecture, getting started, CLI, compatibility, outputs, Doctor, troubleshooting, security, privacy, packaging, and library guidance, **Then** each surface describes current supported behavior, exact refusals, and evidence limitations.
2. **Given** a command or artifact example, **When** controlled validation evaluates it, **Then** it agrees with the executable's argument contract or actual artifact reader and contains no real operator evidence.
3. **Given** unfinished independent review and final acceptance, **When** guidance is published, **Then** Deep Capture remains explicitly functional but incomplete.

### User Story 2 - Review a reproducible whole-product boundary (Priority: P1)

An independent reviewer receives source and build provenance covering hostile inputs, private material, trust, admission, destinations, evidence protection, cleanup, and packaging, with controlled checks and separately authorized installed-build procedures.

**Why this priority**: Ordinary PR reviews do not establish the independent whole-product acceptance required by #333.

**Independent Test**: Check that every review area has source, executable evidence, reproduction instructions, and an explicit unresolved review state; reject fabricated approval or incomplete identity.

**Acceptance Scenarios**:

1. **Given** the handoff, **When** a reviewer follows its references, **Then** they can identify the exact source revision, candidate version, dependency lock digest, package checksums, test results, and unperformed installed-build checks.
2. **Given** a finding, **When** recorded, **Then** severity, owner, affected identity, reproduction, disposition, remediation, independent retest, and operator acceptance remain separate facts.
3. **Given** no completed independent review, **When** the handoff is checked, **Then** it does not claim zero findings, security approval, or completion of #333.

### User Story 3 - Publish new bytes before operator testing (Priority: P2)

The operator receives fresh-release preparation with version-bound contracts and short user-relevant notes, ready for merge and tag without being asked to test an unreleased local build.

**Why this priority**: User-owned testing is restricted to already published product bytes.

**Independent Test**: Validate release preparation, notes, version-bound evidence, and ordinary CI without tagging, publishing, installing, or starting a sensitive session.

**Acceptance Scenarios**:

1. **Given** accumulated unreleased changes, **When** release preparation is reviewed, **Then** its record and notes describe delivered improvements, migration, unsigned packages, and incomplete security acceptance without claiming publication.
2. **Given** merged preparation, **When** the operator follows the handoff, **Then** tag and publication remain operator-owned and Doctor #372 measurement uses the published release.

### Edge Cases

- Historical spike and changelog records retain history; current usage contains no obsolete external-proxy or external trust-helper instructions.
- A missing test, stale command, malformed specimen, broken link, or premature completion claim fails validation rather than being skipped.
- Version changes invalidate exact calibration applicability; guidance distinguishes preserved historical rows from current evidence.
- Unperformed installed-build review never becomes a pass inferred from portable CI or a bot emoji.
- Credentials, capabilities, private keys, real addresses, and host identities never enter public handoff examples.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Current guidance MUST cover the eleven surfaces in User Story 1 and link their native contracts.
- **FR-002**: Every current Deep Capture command example MUST parse without session effects; each committed application or manifest specimen MUST validate against its actual product reader.
- **FR-003**: Guidance MUST separate traffic family, detection, inspectability, exact compatibility, loss, correlation, terminal outcome, artifact completeness, and cleanup authority.
- **FR-004**: Guidance MUST state passive Capture versus deliberate scoped Deep Capture, separate npcap prerequisites, current-user trust consent, pinning limits, sensitive artifacts, and exact confirmed recovery.
- **FR-005**: Review handoff MUST cover architecture, dependencies, unsafe code, parsers, TLS, certificate issuance, trust, listener isolation, routing, artifact protection, recovery, and packaging with executable evidence references.
- **FR-006**: Review identity MUST bind a recorded immutable source revision, product version, lock digest, package checksums, environment, methods, and results; a branch alone MUST NOT identify reviewed bytes.
- **FR-007**: Finding records MUST separate severity, owner, reproduction, identity, remediation or explicit risk acceptance, independent retest, and public-safe closure; a blank register MUST mean review not performed.
- **FR-008**: Release preparation MUST preserve operator tag, package publication, and registry approval gates, produce validated short notes, and identify remaining acceptance work.
- **FR-009**: S150 MUST NOT run a game, installed product, sensitive live capture, or real host trust mutation. Controlled evidence and CI remain implementation acceptance.
- **FR-010**: #331 and #333 MUST retain unfinished review dependencies; #372, #334, and #278 MUST remain open. S150 MUST have bounded GitHub acceptance without duplicate parent issues.
- **FR-011**: Documentation, release records, and architecture currency MUST agree with the prepared version without calling a candidate already published or feature-complete.

### Key Entities

- **Documentation coverage record**: Product surface, page, executable example or artifact specimen, contract, and validation method.
- **Review handoff**: Scope, immutable source and package identity, methods, unresolved installed-build checks, and finding requirements.
- **Finding record**: Independently observed issue with severity, owner, identity, reproduction, remediation, acceptance, retest, and disclosure state.
- **Release preparation**: Proposed version, accumulated changes, validated summary, prerequisites, migration, and operator publication steps.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All eleven documentation surfaces have current guidance and zero current external-backend setup instructions.
- **SC-002**: All committed S150 command and artifact examples pass controlled validation with zero skipped examples.
- **SC-003**: All twelve review areas have source and verification references plus explicit review-state guidance.
- **SC-004**: Site build, links, search navigation, accessibility, release-note validation, and repository gates pass.
- **SC-005**: Review identity and finding records require neither ambiguous branch-only provenance nor disclosure of sensitive evidence.
- **SC-006**: No S150 record claims audit completion, published candidate bytes, live compatibility, or final completion before its external event.

## Assumptions

- S149 is merged and supplies the current product boundary.
- A new minor release suits accumulated features since v0.9.0; planning resolves exact preparation against release policy.
- Independent installed-build assessment and the elevated Doctor stall require operator or reviewer execution against published bytes.
- Documentation can land before security review finishes; #331 remains open for review-driven reconciliation.

## Clarifications

### Session 2026-09-15

- Q: Must S150 finish the independent installed-build audit? -> A: No. It delivers the handoff and explicitly retains unperformed audit under #333.
- Q: Can ordinary PR approval or portable CI close final acceptance? -> A: No. #334 and #278 retain their actual acceptance criteria.
- Q: Does release preparation authorize tag or publication? -> A: No. Local preparation and PR are in scope; tag and publication approval remain operator-owned.
