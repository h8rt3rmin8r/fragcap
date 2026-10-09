# Feature Specification: S172 v0.10.6 release stabilization and preparation

**Feature Branch**: `release/0.10.6`

**Created**: 2026-10-09

**Status**: Candidate owner-merged and v0.10.6 published; separate publication-records reconciliation follows the later owner authorization.

**Input**: Prepare the next patch release from merged S167-S171, reconcile the pending documentation-site dependency patch, align release identities and notes, and drive the authorized local work through the spec-kit autopilot protocol.

## User Scenarios & Testing

### User Story 1 - One accurate patch candidate (Priority: P1)

The operator needs one v0.10.6 candidate containing every merged change since v0.10.5, so diagnostics, packet acquisition and correlation, recipient access, and finite session retention reach users together.

**Why this priority**: These are delivered corrections awaiting a distributable release.

**Independent Test**: The release identity inventory, generated evidence, changelog preview, and release highlights consistently identify v0.10.6 and include all unreleased fragments in chronological order.

**Acceptance Scenarios**:

1. **Given** merged S171 and published v0.10.5, **when** the candidate is prepared, **then** all first-party release identities agree on v0.10.6 and every current fragment appears once in the assembled changelog.
2. **Given** a prepared candidate, **when** an operator reads the public documentation, **then** current published-release markers still identify verified v0.10.5 and v0.10.6 is described as a candidate.

### User Story 2 - Reconciled dependency and release gates (Priority: P1)

The operator needs the pending site patch evaluated against current source and any demonstrated acceptance failure corrected before the release handoff.

**Why this priority**: A known failing acceptance test or unresolved dependency correction prevents a useful release candidate.

**Independent Test**: Frozen site installation, production export and browser checks pass for the selected patch; controlled proxy observation coverage and the repository acceptance gates pass without weakening expectations.

**Acceptance Scenarios**:

1. **Given** dependency PR #470 updates only the site dependency declaration and lockfile, **when** its patch is incorporated, **then** the candidate resolves the same exact patched dependency with preserved site overrides and no unrelated dependency upgrade.
2. **Given** an asynchronous controlled proxy observation, **when** its regression is evaluated, **then** required evidence is awaited under a finite bound rather than inferred from forwarding or an arbitrary delay.
3. **Given** a failing required gate, **when** local preparation finishes, **then** the failure is fixed and rerun or reported as blocking, never reported as passing.

### User Story 3 - Reviewable release handoff (Priority: P2)

The operator needs a committed candidate, short highlights and exact next commands, with candidate verification distinguished from publication.

**Why this priority**: A consistent candidate must remain reviewable before any remote publication effect.

**Independent Test**: A clean feature branch carries the complete analyzed slice and verified changes; its handoff records which local checks ran and which hosted/publication actions remain.

**Acceptance Scenarios**:

1. **Given** local verification is complete, **when** autopilot reaches its push boundary, **then** the exact branch, commit, gate evidence and push command are presented.
2. **Given** no v0.10.6 tag or public artifacts have been verified, **when** the handoff is written, **then** it makes no claim that v0.10.6 is published, certified by hosted packaging, or tested against real games.

### Edge Cases

- The pending dependency PR was based on older main and has an unrelated Ubuntu proxy-test failure.
- A release identity appears inside historical evidence and must retain its original value.
- Release notes exist before the tag and must link to the future tagged complete changelog without asserting publication.
- A required local toolchain or browser is unavailable; its gate remains explicitly unverified.
- Main, candidate source, dependency head, or the target release tag changes before remote handoff.
- Protected registry deployment and release tagging require later explicit authorization and owner actions.

## Requirements

### Functional Requirements

- **FR-001**: Prepare v0.10.6 from merged main `2019ddac844dae891903e1c5c4b5a90a93593947`, retaining all S167-S171 corrections and the unreleased S166 publication records.
- **FR-002**: Align ten product crate versions and task-runner identity, internal version requirements, all first-party lockfile entries, generated examples and golden output, conformance identity, supply-chain snapshot, and master-specification applicability.
- **FR-003**: Include every current changelog fragment once in chronological release assembly and synthesize short user-focused highlights with a link to the complete tagged changelog.
- **FR-004**: Preserve verified v0.10.5 published-state markers and immutable historical release/review evidence until independent v0.10.6 publication reconciliation occurs.
- **FR-005**: Reconcile the exact Next.js 16.3.8 patch from #470, preserving existing overrides and evaluating its frozen install, production export, unit and accessibility gates. The existing PR remains open until an authorized remote handoff establishes supersession or an owner merge.
- **FR-006**: Diagnose and correct any demonstrated release-blocking regression proportionally, with controlled evidence and unchanged security/protocol expectations. Await asynchronous observation by a finite condition-based deadline.
- **FR-007**: Pass blocking spec-kit analysis before candidate implementation and run the repository CI-parity, version, specification, notes, generated-evidence, supply-chain, platform-neutral, minimum-toolchain, and site gates. Preserve failures and unavailable gates accurately.
- **FR-008**: Commit the candidate and provide an exact pre-push handoff under autopilot. Remote push, PR publication, tag creation, registry publication and public-record advancement are separate states requiring their applicable authorization.
- **FR-009**: Do not launch real games, mutate operator session history or trust, or claim universal compatibility or independent whole-product review from controlled candidate checks.

### Key Entities

- **Release candidate**: Version, base source, dependency-patch source, local commits, consistent identity inventory, completed gates, and public notes.
- **Published baseline**: Exact v0.10.5 source and previously verified public artifacts, retained independently of candidate version.
- **Gate evidence**: Command, environment, result, failure or limitation, and source state to which it applies.
- **Dependency disposition**: Exact #470 source and file scope, validation, and pending external supersession decision.

## Success Criteria

- **SC-001**: One clean locally committed v0.10.6 candidate contains all mapped release changes with zero candidate-version mismatches or omitted current fragments.
- **SC-002**: Every required available local gate passes and every unavailable or hosted gate is labeled explicitly; no passing result is inferred from older source.
- **SC-003**: The release highlights satisfy the repository's one-screen notes contract and preserve separate candidate and published identities.
- **SC-004**: The pre-push handoff is reviewable without further implementation work, and no publication claim exceeds its recorded evidence.

## Clarifications

### Session 2026-10-09

- Q: Prepare and publish immediately, or prepare an authorized local candidate? A: This kickoff invokes autopilot without new S172 push/tag authorization. Complete local work, then honor the skill's single pre-push boundary. Earlier push instructions named earlier slices.
- Q: Wait for an owner merge of #470, or incorporate its exact patch? A: Incorporate the reviewed exact two-file patch into S172 after analysis. This avoids a separate dependency merge becoming a prerequisite and leaves the existing PR open for later explicit disposition.
- Q: Consume fragments into tracked CHANGELOG.md now, or use the release exception's assembly tooling? A: Follow the established local release assembly and inspect its exact preview before committing the release candidate; general feature PRs continue using fragments.
- Q: What constitutes completion? A: The authorized local candidate is complete after analyzed preparation, required local gates and a clean commit. Hosted certification, external reviews, owner merge, tagging and verified publication remain separate handoff states.

## Scope Boundaries

This is release preparation and proportionate acceptance stabilization. IGDB #155 and community sync #94 remain deferred. No new protocol, storage schema, capture capability or operating policy is introduced.
