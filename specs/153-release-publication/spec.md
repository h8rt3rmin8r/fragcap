# Feature Specification: S153 v0.10.1 publication

**Feature Branch**: `codex/s153-release-publication`\
**Created**: 2026-09-15.\
**Status**: Specification and clarification validated, awaiting design and analysis.\
**Input**: "kick off S153 and get the new version fully released with green CI checks. Do this with as little interaction from me as possible."

## User Scenarios & Testing

### User Story 1 - Obtain the new release (Priority: P1)

The operator obtains v0.10.1 containing merged S151 Doctor diagnostics and S152 publication protection, without agent execution of installed sensitive software.

**Why this priority**: A published build is required before optional operator field validation; candidate preparation alone does not deliver the patch.

**Independent Test**: Inspect the official release, its six assets, certified provenance and all ten registry versions without launching the application.

**Acceptance Scenarios**:

1. **Given** the reviewed, merged 0.10.1 source, **when** every applicable source check passes, **then** an immutable release tag selects that exact source and the established release pipeline publishes certified packages.
2. **Given** the owner-controlled registry gate, **when** publication waits for owner action, **then** administrator bypass remains enabled and any required owner interaction is requested once; an agent does not approve or bypass deployment.
3. **Given** published packages, **when** release verification completes, **then** every asset checksum, package certification and registry version reconciles to the selected source and every required release job is green.

### User Story 2 - Trust the published baseline (Priority: P2)

Readers see one verified release identity across the current specification, contributor guidance and documentation instead of confusing the prepared version with publication.

**Why this priority**: Publication becomes durable evidence only when the management and documentation surfaces reflect the actual outcome.

**Independent Test**: Existing publication-identity checks accept the updated record and all fourteen baseline surfaces, with historical v0.10.0 evidence unchanged.

**Acceptance Scenarios**:

1. **Given** complete verified publication, **when** current publication records are reconciled through a PR, **then** they identify actual v0.10.1 bytes, date and immutable source while historical release records remain intact.
2. **Given** incomplete independent acceptance, **when** the release is reported live, **then** #372, #333/#413 and #331/#334/#278 retain their unresolved external evidence rather than being silently closed.

### Edge Cases

- Existing tag or concurrent release: inspect exact identity; never overwrite or retarget a tag.
- Failed source CI, failed certification, unavailable policy or checksum mismatch: stop the affected release boundary, preserve evidence and do not label the release fully green.
- Partial registry publication: inventory exact completed versions, preserve immutable published bytes and use the existing dependency-ordered recovery rather than upload changed bytes under the same version.
- Environment approval wait: preserve owner authority and distinguish reviewer approval from deliberate owner bypass.
- Local documentation branch differs from release source: always tag the reviewed merged source, not the unpublished reconciliation branch.

## Clarifications

### Session 2026-09-15

- Q: Which source receives the release tag? A: The already human-merged S152 commit a7d24962999d38d7ff130722859d473543864862, never the later documentation branch. This avoids another prep-only merge before delivering the build.
- Q: Does minimal interaction authorize deployment approval or agent product execution? A: No. Preserve owner environment decisions and the prohibition on installed sensitive software; request only genuinely required owner action.
- Q: Must final independent acceptance block this patch? A: No. Publish the controlled-tested patch while retaining independent review, field measurements and final feature-completion tracking.

All ten clarification taxonomy categories are Clear after these recorded autopilot decisions. No questions were sent to the operator; requirements checklist remains 16/16.

## Requirements

### Functional Requirements

- **FR-001**: Publication MUST select the exact reviewed, merged v0.10.1 source only after all applicable source CI checks pass; existing tags and v0.10.0 assets/history MUST remain unchanged.
- **FR-002**: The release MUST consume the established hosted-certified six-file Windows bundle without local product installation or reconstruction.
- **FR-003**: Existing owner-review, retained administrator bypass, tag-only ref policy and fresh fail-closed publication checks MUST remain unchanged; agents MUST NOT approve or bypass deployments or access registry secrets.
- **FR-004**: Completion MUST require all mandatory release jobs green, six complete uploaded assets with matching checksums/certification and all ten non-yanked registry versions at 0.10.1.
- **FR-005**: Actual publication identity and all fourteen current baseline markers MUST update only after FR-004 is verified, through a human-merge PR; release source and reconciliation source MUST remain independently identified.
- **FR-006**: Failure or partial publication MUST preserve evidence, identity and existing bounds; recovery MUST NOT retarget tags, weaken gates or misreport retries as original success.
- **FR-007**: Optional operator measurements, independent installed security review/retest and final feature acceptance MUST remain separate from release completion; deferred #155/#94 MUST remain out of scope.
- **FR-008**: The slice MUST produce the full ordered specification/design/tasks/analysis artifacts, tested publication records and a verified final handoff, without agent execution of installed sensitive software, production Doctor, live capture or real games.

### Key Entities

- Release identity: version, immutable source revision, publication date and official release URL.
- Certified bundle: exact six asset names, sizes and checksums plus source-bound hosted certification evidence.
- Registry inventory: ten product crate names, version, publication status and non-yanked state.
- Gate evidence: source and release run identities, conclusions and actual owner deployment decision.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The official v0.10.1 release is public with six verified assets, all ten non-yanked crate versions and zero failed or pending mandatory release jobs.
- **SC-002**: All fourteen current baseline markers agree with the verified publication record, and the reconciliation PR passes its applicable checks before final handoff.
- **SC-003**: Zero existing release identities are overwritten, zero agent deployment approvals/bypasses occur, and zero installed sensitive-product runs are used as evidence.
- **SC-004**: Every unresolved field or independent acceptance item remains explicitly tracked, with no unsupported feature-completion claim.

## Assumptions

- The user explicitly authorizes release tagging/pushing and publication, plus a scoped publication-record PR, without another pre-push pause. Human merge and owner environment decisions remain separate.
- The release uses merged S152 source a7d24962999d38d7ff130722859d473543864862; S153 is release execution and evidence reconciliation, not another version bump or product feature change.
- The ten product crates and existing unsigned package policy remain unchanged. Checksums do not imply code signing.
- Architecture trace: master specification sections 24.4, 24.5 and 27.3; constitution P-9/P-11; S152 release handoff.
