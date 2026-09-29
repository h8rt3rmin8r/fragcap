# Feature Specification: S162 v0.10.3 patch release

**Feature Branch**: `release/0.10.3`

**Created**: 2026-09-29

**Status**: Specified for implementation and publication

**Input**: The operator approved the recommended S162 patch release, explicitly requested publication, and will approve crates.io when GitHub requests it. Tracking issue: #439.

## User Scenarios & Testing

### User Story 1 - Reviewed patch candidate (Priority: P1)

The operator wants a single reviewed v0.10.3 candidate that includes every merged, unreleased change since v0.10.2, especially S160 queue headroom and S161 interactive CLI input.

**Independent Test**: Ten workspace crates, the lockfile, embedded versions, generated outputs, conformance identities, specification applicability, changelog, and release highlights agree on 0.10.3 while published-state records still name v0.10.2.

**Acceptance Scenarios**:

1. **Given** exact clean merged main after S161, **when** release preparation runs, **then** one candidate contains all unreleased fragments in chronological order and makes no premature published-state claim.
2. **Given** the candidate, **when** local and hosted checks run, **then** every required gate passes on the final PR head before operator merge.

### User Story 2 - Verified public release (Priority: P2)

The operator wants the reviewed candidate published as an exact tag, certified Windows downloads, and ten crates.io packages.

**Independent Test**: The v0.10.3 tag peels to the operator-merged candidate, four release jobs pass, six public files reconcile, and ten crates.io version records are non-yanked with checksums.

**Acceptance Scenarios**:

1. **Given** the operator-merged candidate, **when** v0.10.3 is tagged, **then** the tag selects that exact source and the existing workflow creates certified public artifacts.
2. **Given** a protected crates.io deployment wait, **when** the operator approves that deployment, **then** publication continues without environment-policy changes or agent approval.
3. **Given** public release completion, **when** its independent evidence is checked, **then** the release, assets, certification report, and registry records all reconcile to the tag.

### User Story 3 - Accurate published baseline (Priority: P3)

The operator wants current published-state documentation to move to v0.10.3 only after public verification.

**Independent Test**: A separate records-only PR binds the exact tag, source, workflow, assets, and registry evidence without changing published bytes or historical records.

**Acceptance Scenarios**:

1. **Given** complete public evidence, **when** records are reconciled, **then** current markers move from v0.10.2 to v0.10.3 and historical evidence remains immutable.
2. **Given** only source or candidate evidence, **when** records are inspected, **then** they do not assert that 0.10.3 was published.

### Edge Cases

- Main advances after candidate review, the PR merges different content, or a v0.10.3 tag already points elsewhere.
- A release job fails, is cancelled, or waits for protected-environment approval.
- Assets or sidecars are absent, duplicated, inconsistent, or not bound to the tag source.
- A registry version is unavailable, delayed, yanked, or missing its checksum.
- The Print Screen hotkey observation remains unproven and must not be described as corrected.

## Requirements

### Functional Requirements

- **FR-001**: S162 MUST start from exact clean main after S161 and prepare patch version 0.10.3 with all unreleased S159 through S161 fragments assembled in chronological order.
- **FR-002**: All ten product crate versions and the xtask version, lockfile identities, embedded output, generated goldens, native conformance data, specification applicability, and release highlights MUST agree on 0.10.3.
- **FR-003**: Candidate preparation MUST preserve `docs/published-release.json` and other actual-publication markers at v0.10.2 until public verification.
- **FR-004**: Highlights MUST describe S160 and S161 accurately and MUST NOT claim a Print Screen fix, installed-host test, independent product security acceptance, or universal game compatibility.
- **FR-005**: The final candidate MUST pass full local source gates and required hosted PR checks without locally installing or executing the product, a game, production Doctor, real trust mutation, or sensitive live capture.
- **FR-006**: The candidate MUST use an operator-reviewed PR; the agent MUST NOT push to main or merge its own PR.
- **FR-007**: After operator merge, the agent MUST verify exact main and absence of a conflicting tag before creating and pushing annotated v0.10.3 under the operator's explicit publication authorization.
- **FR-008**: The existing tag release workflow, package certification, dependency-ordered registry publication, and protected crates.io environment MUST remain in force.
- **FR-009**: If GitHub waits for deployment approval, only the operator may approve it; the agent MUST provide the exact run and environment and then continue after approval.
- **FR-010**: Publication MUST NOT be declared complete until four release jobs succeed, the public release is non-draft and non-prerelease, six files and three sidecars reconcile, certification binds the tag, and all ten crates report 0.10.3 as non-yanked with checksums.
- **FR-011**: Actual publication records MUST be reconciled in a separate records-only operator-reviewed PR after public verification.
- **FR-012**: Historical v0.10.2 evidence and unsigned policy MUST remain unchanged; review and field observations outside the release boundary MUST be reported honestly.
- **FR-013**: At most two Codex review rounds per PR may be requested, and every actionable review comment MUST be resolved before operator handoff.

### Key Entities

- **Release candidate**: Reviewed source with internally consistent version, generated outputs, changelog, notes, and certification gates.
- **Published release**: Immutable tag, successful workflow, verified public assets, and non-yanked registry packages.
- **Publication evidence**: Exact source, tag object, run jobs, protected-environment decision, files, checksums, certification, and registry records.
- **Published baseline record**: Current repository metadata changed only after public evidence is complete.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Ten of ten product crates and every candidate identity surface report 0.10.3 with no stale candidate-version finding.
- **SC-002**: Local gates and every required hosted PR check pass on the final candidate head.
- **SC-003**: The v0.10.3 tag peels to the exact operator-merged candidate.
- **SC-004**: Four release jobs pass under unchanged registry protection.
- **SC-005**: Six public files match expected names, sizes, and SHA-256 identities; the certification summary binds the tagged source.
- **SC-006**: All ten registry packages report exact 0.10.3, `yanked=false`, and a nonempty checksum.
- **SC-007**: Published markers move only after SC-003 through SC-006 are observed.
- **SC-008**: No sensitive local product, real-game, or real-trust execution is performed by the agent.

## Assumptions

- Patch version 0.10.3 is appropriate for the merged nonbreaking reliability corrections.
- Existing release workflows, approval protection, and publication credentials remain available.
- The operator will merge the reviewed candidate and later records PR; the operator will approve crates.io if GitHub pauses it.
- This request authorizes candidate push, PR creation, and later tag publication, but does not authorize the agent to merge its own PR or approve the protected environment.

## Clarifications

### Session 2026-09-29

- Q: Which slice and version carry the post-v0.10.2 corrections? A: S162 and v0.10.3, covering S160 and S161 plus other unreleased fragments without expanding product scope.
- Q: May the agent publish after merge? A: Yes, the operator explicitly requested publication; required operator merge and crates.io approval remain separate controls.
- Q: Can current publication markers move in the candidate? A: No, they move only in a later records PR after exact public verification.
