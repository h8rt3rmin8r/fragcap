# Feature Specification: S159 v0.10.2 release

**Feature Branch**: `release/0.10.2`

**Created**: 2026-09-20

**Status**: Specified for implementation and publication

**Input**: The operator requested a new release with minimal interaction and authorized the release publication workflow.

## User Scenarios & Testing

### User Story 1 - Reviewed patch candidate (Priority: P1)

The operator wants every merged change since v0.10.1 packaged into one coherent v0.10.2 candidate before an immutable release is created.

**Why this priority**: A tag cannot safely precede version, changelog, release-note, generated-output and certification agreement.

**Independent Test**: All ten workspace crates, embedded identities, generated goldens, specification applicability, changelog and release highlights agree on 0.10.2 while the published baseline remains v0.10.1.

**Acceptance Scenarios**:

1. **Given** merged S154 through S158 work, **When** release preparation runs, **Then** one reviewed candidate contains all unreleased fragments and no published-state claim.
2. **Given** a prepared candidate, **When** repository gates and hosted pull-request checks run, **Then** every required check passes on the final candidate head before human merge.

### User Story 2 - Verified public release (Priority: P2)

The operator wants the merged candidate published as immutable GitHub assets and non-yanked crates with minimal manual work.

**Why this priority**: Users cannot obtain the recent reliability corrections until the official release and registry publication both complete.

**Independent Test**: The v0.10.2 tag peels to the merged candidate, all release jobs pass, six public files reconcile with checksums, and all ten registry versions are non-yanked.

**Acceptance Scenarios**:

1. **Given** the human-merged candidate and explicit publication authorization, **When** v0.10.2 is tagged, **Then** the tag selects that exact source and the workflow creates certified public artifacts.
2. **Given** GitHub requests environment approval, **When** the owner approves the protected crates.io deployment, **Then** publication continues without changing or bypassing repository policy.
3. **Given** a completed release run, **When** public evidence is checked independently, **Then** every expected asset and crate version is present, exact and non-yanked.

### User Story 3 - Truthful published baseline (Priority: P3)

The operator wants current documentation and machine-readable records to name v0.10.2 only after public verification.

**Why this priority**: Candidate preparation and publication are different facts, and confusing them corrupts later audit and review work.

**Independent Test**: A separate records-only pull request binds exact tag, source, workflow, assets and registry evidence without modifying the published release.

**Acceptance Scenarios**:

1. **Given** public v0.10.2 evidence, **When** records are reconciled, **Then** current baseline markers move from v0.10.1 to v0.10.2 and historical release evidence remains immutable.
2. **Given** incomplete independent review or final Deep Capture acceptance, **When** release records are authored, **Then** those gates remain explicitly open.

### Edge Cases

- The release preparation PR is not yet merged, or merged source differs from the reviewed candidate.
- A tag already exists, points elsewhere, or the workspace version does not match it.
- A hosted release job fails, is cancelled, or requires owner environment approval.
- GitHub assets are missing, duplicated, renamed, or fail their checksum sidecars.
- A registry version is unavailable, yanked, delayed, or reports an unexpected checksum.
- A records change attempts to describe publication before all public evidence exists.

## Requirements

### Functional Requirements

- **FR-001**: S159 MUST prepare patch version 0.10.2 from the exact merged S158 main head and MUST include every unreleased S153 through S158 changelog fragment in chronological order.
- **FR-002**: All ten workspace crate versions, lockfile packages, embedded product identities, conformance metadata, generated goldens, specification Applies-To and candidate release notes MUST agree on 0.10.2.
- **FR-003**: Preparation MUST preserve `docs/published-release.json`, current published-review identity and other actual-publication markers at v0.10.1 until v0.10.2 public verification completes.
- **FR-004**: Release highlights MUST remain bounded, user-facing, truthful about S154 through S158, and MUST NOT claim independent security review, installed QUIC acceptance or final Deep Capture completion.
- **FR-005**: The release preparation MUST pass full local source gates and every required hosted pull-request check without installing or running the sensitive product locally.
- **FR-006**: The official release preparation MUST be pushed through one human-reviewed pull request. The agent MUST NOT merge its own pull request or push directly to main.
- **FR-007**: After human merge, the v0.10.2 tag MUST be created from the exact merged candidate source and pushed under the operator's explicit publication authorization.
- **FR-008**: The release workflow MUST preserve exact tag and workspace identity, current registry protection, certified Windows packaging, checksum validation and dependency-ordered registry publication.
- **FR-009**: If the protected crates.io job waits for approval, the agent MUST request only that owner action and MUST NOT approve, bypass or weaken the environment policy.
- **FR-010**: Publication MUST NOT be declared complete until all four release jobs are green, the GitHub release is public, all six public files reconcile and all ten crates report 0.10.2 as non-yanked.
- **FR-011**: Actual publication evidence MUST be reconciled through a separate records-only human-reviewed pull request after public verification.
- **FR-012**: Historical v0.10.1 records, assets and tags MUST remain unchanged, and open independent review, Doctor field validation and final Deep Capture acceptance MUST remain open.
- **FR-013**: S159 MUST use no more than two review rounds per pull request and MUST respond to every review comment before handoff.
- **FR-014**: Local verification MUST NOT execute an installed fragcap binary, a real game, real trust mutation, production Doctor or sensitive live capture.

### Key Entities

- **Release candidate**: Reviewed 0.10.2 source with aligned package, output, changelog and note identities.
- **Published release**: Immutable tag, successful release run, public assets and registry versions.
- **Publication evidence**: Exact source, tag object, run jobs, file identities, checksums and registry records.
- **Published baseline record**: Current repository metadata that changes only after publication evidence is complete.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Ten of ten product crates and every candidate identity surface report 0.10.2 with zero stale candidate-version findings.
- **SC-002**: Every required local and hosted candidate check is green on the final release preparation head.
- **SC-003**: The v0.10.2 tag peels to the exact human-merged release candidate commit.
- **SC-004**: Four of four release jobs complete successfully without policy weakening or agent deployment approval.
- **SC-005**: Six of six public files match expected names, sizes and SHA-256 identities, including three valid checksum sidecars.
- **SC-006**: Ten of ten registry packages report version 0.10.2 with `yanked=false` and a nonempty checksum.
- **SC-007**: Published baseline records move only after SC-003 through SC-006 pass.
- **SC-008**: Zero installed local product, real-game, real-trust or sensitive live-capture executions occur.

## Assumptions

- Semantic version 0.10.2 is appropriate because the unreleased work corrects reliability and release evidence and completes documentation without a breaking product contract.
- The existing release workflow, environment and publication credentials remain available and unchanged.
- Human merge of each pull request and any protected-environment approval remain owner actions.
- The user's current release request supplies the tag and publication authorization that S152 intentionally withheld.

## Clarifications

### Session 2026-09-20

- Q: Which version and slice should carry this release? A: Use S159 and patch version 0.10.2 because the delta is nonbreaking reliability, certification and documentation work.
- Q: Does the release close independent review or final Deep Capture acceptance? A: No. Preserve those external gates and describe them truthfully.
- Q: Can the agent push the release branch and later tag? A: Yes. The explicit request to publish authorizes both, but repository policy still requires human PR merge and owner environment approval when GitHub requests it.
- Q: May preparation and post-publication reconciliation share one branch? A: No. Candidate preparation must merge before its immutable tag exists; actual publication records therefore require a later records-only branch and pull request.
