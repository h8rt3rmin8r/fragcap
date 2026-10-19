# Feature Specification: S174 v0.10.7 publication

**Feature Branch**: `release/0.10.7`

**Created**: 2026-10-19

**Status**: v0.10.7 public files/crates verified; final records/site acceptance pending

**Input**: The owner requests a fresh publishing release end to end without further input, replacing the proposed IGDB slice. Publish every merged change since v0.10.6, including S173, then reconcile actual public state.

## Clarifications

### Session 2026-10-19

- Q: Which version and content? A: v0.10.7, all merged work since v0.10.6, including S173; IGDB remains excluded.
- Q: Does the old S173 human merge boundary still apply? A: The latest owner request delegates full release-specific integration, tag, normal approval and publication without additional input; global policy and protection settings remain unchanged.
- Q: Is an unpublished dedicated security report an independent-audit completion claim? A: No. Reconcile actual findings, preserve the two-round cap and disclose the report status without claiming an independent audit.
- Q: Can current published markers move during candidate preparation? A: No, only after actual release files and crates independently reconcile.

## User Scenarios & Testing

### User Story 1 - Install the corrected release (Priority: P1)

An operator obtains one coherent v0.10.7 release containing the merged elevated-output correction, retired historical repair surface and dependency advisory fixes.

**Why this priority**: The current published v0.10.6 lacks the S173 corrections already accepted on main.

**Independent Test**: Candidate and final tagged package certification reconcile exact identity, payload, imports, installer lifecycle and supported controlled smoke outcomes.

**Acceptance Scenarios**:

1. **Given** accepted merged S173 source, **When** the release candidate is prepared, **Then** all first-party identities, generated evidence and user highlights agree on 0.10.7 and include every unreleased fragment exactly once.
2. **Given** a certified tag selecting reviewed merged source, **When** downloads become public, **Then** all six files and their three sidecars match actual certified bytes.

### User Story 2 - Consume published crates and documentation (Priority: P1)

A consumer finds all ten product crates at the same non-yanked version, and current documentation names only a release that actually exists.

**Why this priority**: A GitHub download alone does not complete the established publishing contract.

**Independent Test**: Independently obtain registry records and archives, verify checksums and version identity, then validate current release markers and public site deployment.

**Acceptance Scenarios**:

1. **Given** final GitHub release success, **When** normal protected registry approval is submitted, **Then** all ten crates publish in dependency order without weakening protection.
2. **Given** independently verified publication, **When** the separate records change integrates, **Then** current markers and frozen replay authority advance while historical release facts remain intact.

### User Story 3 - Complete without another operator prompt (Priority: P2)

The owner delegates this release's routine decisions, PR publication and merges, tag, protected approval and verification, receiving a completion report when public delivery is reconciled.

**Why this priority**: The owner explicitly requests unattended end-to-end publication.

**Independent Test**: The record identifies accepted source, exact-head green gates, bounded review ledger, normal approval and immutable public identities without replacing them with assumptions.

**Acceptance Scenarios**:

1. **Given** explicit owner delegation, **When** all required checks and actual review findings are satisfied, **Then** release-specific PRs may merge and the annotated tag may be pushed without a further prompt.
2. **Given** a real failure or partial publication, **When** recovery is attempted, **Then** immutable identities are preserved, only justified affected jobs retry, and uncompleted states remain explicit.

### Edge Cases

- Existing tag/version or moved candidate head; duplicate publication; registry indexing delay; partial publication; expired or invalid SDK TLS; checksum mismatch; stale or contradictory hosted status.
- Advisory or package-certification failure; absent dedicated security report; expected manual/main-only skipped checks; unavailable normal protected approval.
- Publication record changes and generated version evidence must not rewrite old measured source or claim real-game compatibility or independent audit.

## Requirements

### Functional Requirements

- **FR-001**: The release MUST include every merged unreleased change since v0.10.6, including S173, with coherent 0.10.7 first-party identities and once-only chronological changelog assembly.
- **FR-002**: Short user-facing highlights MUST describe actual release effects and link the complete tagged changelog without copying the full changelog.
- **FR-003**: Candidate, public and historical identities MUST remain distinct until publication is independently verified; current baseline markers MUST NOT claim a future release.
- **FR-004**: Required local and exact-final-head hosted gates MUST pass before each release-specific PR merge; every actual review finding MUST be resolved. At most two review rounds per code-bearing candidate PR are permitted; automatic records review is reconciled without an extra candidate round.
- **FR-005**: Annotated v0.10.7 MUST select the exact accepted merged candidate source; no tag rewriting, direct main push or unreviewed bytes.
- **FR-006**: Final tagged Windows bytes MUST pass existing certification before GitHub release creation, retaining normal TLS, pinned SDK checksum, unsigned identity and Npcap non-redistribution.
- **FR-007**: Registry publication MUST follow normal protected approval and preserve existing reviewer, self-review, tag-only and administrator-bypass settings; no policy weakening or bypass is part of the planned path.
- **FR-008**: Completion MUST independently reconcile six public files, their sidecars and certification authority, plus all ten non-yanked product crate records and archive checksums.
- **FR-009**: Actual publication MUST be reconciled through a separately checked records PR, current site deployment, frozen candidate authority and public links; prior release facts and unperformed independent-review status remain truthful.
- **FR-010**: The owner delegation MUST authorize this release's necessary pushes, merges, tag, normal approval and publication without extending that exception to future work. No IGDB or unrelated product work, owner data/account/UAC/trust mutation or live-game execution is included.
- **FR-011**: Tool execution MUST remain hidden/noninteractive; partial outcomes and external blockers MUST be recorded accurately, secrets withheld and retry actions proportionate.

### Key Entities

- **Release candidate**: Version, exact source tree, accepted head, required checks and review findings.
- **Published release**: Annotated tag and source, successful release run, certified file sizes/digests and public timestamp.
- **Registry publication**: Exact ten crate versions, non-yanked state and independently verified archive checksums.
- **Publication record**: Current markers and replay authority reconciled only after actual public delivery.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required candidate identity and acceptance gate agrees on 0.10.7 with zero unresolved actual findings before integration.
- **SC-002**: All six public files and all ten crate archives match certified or registry identities, with zero missing or yanked required versions.
- **SC-003**: Current public documentation and replay authority name actual v0.10.7 without altering historical release facts or claiming unperformed review.
- **SC-004**: The owner receives a final reconciled completion report without another authorization prompt; any genuinely unavailable operation remains explicitly incomplete.

## Assumptions

- Merged S173 main b4cb29a9751ba95b10d0eb411f6603a11c2efbb6 is the starting source; published baseline is v0.10.6.
- 0.10.7 follows the established patch-release line for accepted corrections. No dependency upgrade is planned beyond changes already merged unless a required gate demonstrates a new blocker.
- Latest explicit owner delegation supersedes the normal human-only release-specific merge/tag checkpoint for this release. It does not authorize changing protection settings or global workflow policy.
