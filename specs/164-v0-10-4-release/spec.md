# Feature Specification: S164 v0.10.4 patch release

**Feature Branch**: `release/0.10.4`
**Created**: 2026-10-03
**Status**: Specified for candidate preparation; publication follows the operator merge
**Input**: The operator authorized S164 autopilot, branch push, an official pull request, two review rounds at most, and a green-CI handoff for their final review and merge. The recommended scope is a v0.10.4 release containing merged S163 first-run calibration corrections.

## User Scenarios & Testing

### User Story 1 - Reviewed patch candidate (Priority: P1)

The operator wants one coherent v0.10.4 source candidate containing all merged unreleased changes since v0.10.3, with the S163 calibration and first-run guidance corrections explained accurately.

**Independent Test**: The candidate source, lockfiles, generated evidence, specification applicability, changelog, and highlights agree on 0.10.4; required local and hosted gates pass on the final PR head; current published-state records still identify v0.10.3.

**Acceptance Scenarios**:

1. **Given** clean main after S163, **when** the candidate is prepared, **then** all unreleased fragments are assembled in chronological order and the patch identity is internally consistent.
2. **Given** the candidate PR, **when** required checks and external reviews complete, **then** every actionable comment is answered and resolved, no more than two Codex review rounds have been triggered, and all required final-head checks are green before the operator is asked to merge.
3. **Given** release highlights, **when** a reader uses them, **then** they describe the observed S163 fixes without naming any real title, private path, or universal compatibility claim.

### User Story 2 - Accurate publication handoff (Priority: P2)

The operator wants the reviewed candidate ready for their merge ritual, with exact subsequent tag, publication, and record steps documented without treating an open PR as a public release.

**Independent Test**: The handoff names the candidate PR and final checked commit, the owner merge is still pending, and the public v0.10.3 baseline is unchanged. Post-merge publication gates are explicit and cannot be marked complete by candidate evidence.

**Acceptance Scenarios**:

1. **Given** a green reviewed PR, **when** the agent reports readiness, **then** the operator receives the PR, review results, final-head CI status, and exact remaining merge and release steps.
2. **Given** no v0.10.4 public tag, **when** published-state records are checked, **then** they still identify verified v0.10.3.
3. **Given** the operator later merges the candidate and authorizes publication, **when** release work resumes, **then** exact merged-source, tag, workflow, public-asset, certification, and registry evidence precede any current published-state update.

### Edge Cases

- Main changes after candidate preparation, the PR head changes after checks, or a conflicting v0.10.4 tag appears.
- Review bots publish inline comments, a top-level review, a PR body reaction only, or no review; no review state is invented.
- Required CI is pending or fails, including isolated fuzz or performance lockfile drift.
- A release workflow waits for protected crates.io approval or a public asset or registry version has not propagated.
- First-run changes resolve demonstrated defects but do not prove every title, protocol, or environment works.

## Requirements

### Functional Requirements

- **FR-001**: S164 MUST prepare v0.10.4 from exact clean main after S163 and include every unreleased fragment since v0.10.3 in chronological order.
- **FR-002**: Ten product crates and xtask, workspace and isolated lockfiles, embedded output, generated goldens, native conformance identity, supply-chain snapshot, and specification applicability MUST agree on 0.10.4 without changing third-party dependency policy solely for the release.
- **FR-003**: Candidate highlights MUST accurately describe S163 first-run setup, owned cold Steam root, and causal failure reporting without naming a real title or claiming a completed owner field trial or universal compatibility.
- **FR-004**: Current published-state records MUST remain bound to verified v0.10.3 until independent public v0.10.4 verification after merge.
- **FR-005**: The candidate MUST pass repository source, documentation, specification, notes, MSRV, neutral, version, generated-output, and changed-text hygiene gates; required hosted PR checks MUST pass on the final head.
- **FR-006**: The agent MUST push the release branch and open an official PR, inspect every external review and comment, answer and resolve actionable threads, request no more than one additional Codex review round, and stop before operator merge.
- **FR-007**: The handoff MUST distinguish candidate readiness from release publication and identify the exact post-merge tag, protected deployment, public asset, certification, registry, and records-only PR gates.
- **FR-008**: The candidate MUST NOT require installing or running a real title, production Doctor, real trust mutation, or sensitive live capture on the operator machine; an owner field trial may follow published bytes and is not a release acceptance gate.
- **FR-009**: A conflicting tag, changed merged source, failed release job, missing asset, failed certification, or missing/yanked registry version MUST prevent a publication-complete claim.

### Key Entities

- **Candidate**: Source branch and PR with version-aligned source, generated evidence, documentation, and green final-head checks.
- **Published release**: A later annotated tag on exact owner-merged source, green release workflow, independently reconciled public files, certification, and registry packages.
- **Published-state record**: A separate later records-only PR that moves current markers only after public verification.

## Success Criteria

- **SC-001**: One official v0.10.4 candidate PR is open with zero unresolved actionable review threads, at most two Codex review rounds, and all required checks green on its final head before owner handoff.
- **SC-002**: No candidate artifact or current published-state marker claims that v0.10.4 has been published before its verified public release exists.
- **SC-003**: Release notes and public PR text contain no title-specific install, executable, application ID, account, or sensitive capture evidence.
- **SC-004**: The operator can follow the documented post-merge publication gates without reinterpreting candidate checks as public-release evidence.
