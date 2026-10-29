# Feature Specification: S183 v0.10.9 release candidate

**Feature Branch**: `release/0.10.9` | **Created**: 2026-10-29 | **Status**: In progress

**Input**: Prepare the next patch release from merged S182, carry the corrected dependency graph and installed-product fixes into a fully reviewed candidate, push and open its official pull request, resolve at most two Codex review rounds, and hand the green candidate to the owner for final review and merge.

## Clarifications

### Session 2026-10-29

- Q: Does the autopilot request authorize the release tag and publication? A: No. It explicitly ends at owner final review and merge. The automatic push and official PR are authorized.
- Q: What closes issue #519? A: Actual corrected public files and registry archives, after merge and separate release approval. The candidate PR references but does not close it.
- Q: Is a real-title calibration result a candidate acceptance gate? A: No. The project limits that observation to already published bytes. A physical current-user CA trust test is controlled package qualification, with exact outcome recorded.
- Q: How many external review rounds may be requested? A: Automatic first round plus at most one explicit `@Codex` second round. A missing security-bot receipt is reported, not inferred.

## User Scenarios & Testing

### User Story 1 - Review one coherent patch candidate (Priority: P1)

An owner can review one v0.10.9 source candidate containing the already merged dependency correction and S182 recovery work, with version, changelog, notes, generated evidence and documentation agreeing.

**Independent Test**: Every candidate identity names v0.10.9 and the same source tree; current public release markers still identify v0.10.8.

**Acceptance Scenarios**:

1. **Given** merged main and a published v0.10.8 release, **when** the candidate is prepared, **then** all unreleased fragments appear once in chronological release history and short notes state the advisory's Unix API scope without claiming a Windows exploit.
2. **Given** no public v0.10.9 bytes, **when** the candidate is reviewed, **then** current download and registry markers still point to v0.10.8.

### User Story 2 - Trust the candidate's controlled evidence (Priority: P1)

An owner can assess the exact candidate through repository checks, dependency audits, controlled Windows package certification, and an explicit record of what the physical trust interaction did or did not prove.

**Independent Test**: The candidate's product and isolated dependency graphs pass current advisory checks; controlled local and hosted gates report exact source identity and terminal results.

**Acceptance Scenarios**:

1. **Given** the candidate source, **when** product and isolated graph audits run, **then** no current advisory is hidden by a new waiver or weakened policy.
2. **Given** a Windows package or physical trust test, **when** it completes or is blocked, **then** the evidence records exact version, binary identity, outcome, and cleanup state without claiming an unperformed result.

### User Story 3 - Receive a review-ready PR (Priority: P2)

The owner receives an official PR whose required checks are green, actual review threads have been answered and resolved, and whose merge and later publication remain deliberate owner actions.

**Independent Test**: The PR head matches the checked candidate, all applicable checks are terminal success, review comments are addressed, and no more than one explicit second review request was sent.

**Acceptance Scenarios**:

1. **Given** an actionable review comment, **when** the candidate changes, **then** the comment receives a response, the thread is resolved, and checks rerun on the new head.
2. **Given** a green reviewed head, **when** S183 reaches handoff, **then** no merge, tag, GitHub release, registry publication, or current-site marker advance has been performed.

## Edge Cases

- A new advisory or unexpected third-party graph change blocks candidate acceptance until corrected or explicitly scoped.
- A physical Windows trust prompt may require an operator decision; a declined or unavailable action remains unverified and cleanup must still be reconciled.
- A review bot may leave only a reaction or no receipt. Count actual comments and threads, never infer review acceptance.
- New source changes after a review invalidate earlier exact-head check claims.

## Requirements

### Functional Requirements

- **FR-001**: Prepare patch candidate v0.10.9 from merged main `4ffa5f566d51aa51e59c8293160e55f4513e0cd0` or its verified fast-forward successor; exclude deferred IGDB work.
- **FR-002**: Update all candidate-bound versions, generated evidence, graph authorities, specification Applies-To value, changelog and short highlights together; retain v0.10.8 as the current published marker.
- **FR-003**: Include all unreleased fragments exactly once and describe the corrected Tokio graph accurately, including the advisory's Unix-only affected API and lack of demonstrated Windows exploitation.
- **FR-004**: Verify product and isolated locked dependency graphs against current advisory data without a blanket waiver, feature change or new unreviewed third-party resolution.
- **FR-005**: Run the repository's local release checks and hosted exact-head CI, including Windows package certification and same-version documentation checks.
- **FR-006**: Attempt the outstanding controlled physical current-user trust row with exact cleanup and record a precise pass, refusal, or unverified outcome.
- **FR-007**: Push and open the official candidate PR; address every actionable review comment and resolve every review thread. At most one explicit second Codex round is permitted.
- **FR-008**: Stop at owner final review and merge. No tag, protected registry approval, public asset claim, or issue #519 closure occurs within this pre-merge handoff.
- **FR-009**: Do not disclose the private field-test title in any public issue, PR, source file, log or artifact.

## Key Entities

- **Candidate source**: One reviewable commit tree with v0.10.9 version and generated authorities.
- **Published release**: The immutable v0.10.8 public baseline until post-merge publication is independently verified.
- **Qualification receipt**: Exact candidate identity, command or physical test, result and limits.
- **Review receipt**: PR head, CI status, bot response, comment resolution and round count.

## Success Criteria

- **SC-001**: All v0.10.9 candidate identities and release notes agree; no v0.10.9 current-public marker advances.
- **SC-002**: Product and isolated audits plus required release gates pass on the final PR head, with no new advisory waiver.
- **SC-003**: Every actual actionable review thread is resolved, required PR checks are green, and the owner receives an exact merge-ready PR link and remaining physical-evidence limits.

## Assumptions

- This slice prepares a release candidate and stops before owner merge and tag, as explicitly requested in the current turn.
- Issue #519 remains open through actual public delivery and reconciliation after merge.
- Current public documentation stays on v0.10.8 until v0.10.9 assets and registry packages exist.
- Real-title calibration is field evidence only on published product bytes and is outside this candidate's source gate.

## 2026-10-29 owner-directed post-merge continuation

After the owner merged PR #524, the owner directed the agent to perform the release. The v0.10.9 annotated tag, normal protected crates.io approval, public asset and registry verification, installed MSI upgrade, and a separate publication-records PR are now in scope. The pre-merge requirements above remain the candidate acceptance record. The records PR still follows the repository's human review and merge rule. A real-title full calibration and Deep Capture success remain unverified until separately observed on published bytes.
