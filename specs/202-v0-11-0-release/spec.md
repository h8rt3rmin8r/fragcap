# Feature Specification: S202 v0.11.0 Direct Route Release

**Feature Branch**: `release/0.11.0`

**Created**: 2026-11-10

**Status**: In progress

**Input**: Prepare merged S197 through S201 as one public release of the completed direct-route feature and the other accepted changes since v0.10.14 under release issue #589. The owner authorizes automatic push, an official candidate pull request, CI, and at most one explicit second Codex review request, then reserves final candidate review and merge. Publication follows only the accepted merge and the owner's separate tag and registry approval actions.

## Clarifications

### Session 2026-11-10

- Q: Which version identifies this new public capability? -> A: v0.11.0. It is the next minor version after v0.10.14 and distinguishes direct final-client routing from the existing 0.10 patch series. The choice is reversible until the owner publishes the tag.
- Q: Does this kickoff authorize candidate push, merge, tag, and registry publication? -> A: Push and official PR are explicitly authorized. The owner retains final merge, version-tag push, and registry approval. The same S202 record will continue after those owner actions instead of treating the candidate PR as the public release.
- Q: Must a broken or absent security-review bot, private-title run, or the independent #580 research block publication? -> A: No. Act on every review finding actually received and retain the exact review record. Local and controlled repository checks establish release readiness. #580 and later installed-product observations are separate.

## User Scenarios & Testing

### User Story 1 - Review a coherent release candidate (Priority: P1)

The owner receives one reviewable v0.11.0 candidate containing every accepted change since v0.10.14, with matching product version, package identity, changelog, release highlights, and first-use instructions. A reader can tell that the candidate is not yet the public download.

**Why this priority**: The merged feature is not in the public v0.10.14 package. Publishing an inconsistent version or describing candidate behavior as already shipped would mislead users.

**Independent Test**: The candidate source and generated outputs agree on v0.11.0, every pending release fragment is incorporated once, current-public records still identify v0.10.14, and the repository's complete candidate checks pass.

**Acceptance Scenarios**:

1. **Given** the accepted S197 through S201 source, **when** the owner reviews the v0.11.0 candidate, **then** the installer, portable package, crate identities, release highlights, changelog, and supported route documentation agree on the same candidate behavior.
2. **Given** a reader of the documentation before publication, **when** they follow a direct-route instruction, **then** the page clearly distinguishes the v0.11.0 source candidate from the v0.10.14 public download.

---

### User Story 2 - Obtain the exact accepted public release (Priority: P1)

After the owner merges the candidate and pushes its version tag, users can download certified Windows packages and see the same version published in the Rust registry. The released files and source are independently matched to the accepted merge.

**Why this priority**: A green candidate PR is not a public release. Public availability requires a tagged build, package certification, download verification, and registry publication.

**Independent Test**: The tag selects the accepted merge, the tagged workflow completes, every public asset matches its checksum and certification record, and every published crate reports the same non-yanked version.

**Acceptance Scenarios**:

1. **Given** the owner-approved merged candidate, **when** the owner publishes the version tag and approves the normal registry deployment, **then** the GitHub release and all ten crates expose v0.11.0 from the accepted source.
2. **Given** a failed or incomplete publication job, **when** release status is assessed, **then** no current-public record is advanced to v0.11.0 until the missing public evidence is repaired and verified.

---

### User Story 3 - Find current public guidance (Priority: P2)

Once v0.11.0 is public, a new user finds the correct installer, Npcap, bundled WinDivert, calibration, and Deep Capture instructions without seeing the feature mislabeled as only a source candidate.

**Why this priority**: The public documentation is the first-run path, and its current-version claims must follow actual publication.

**Independent Test**: The verified public release, current-version records, site pages, release notes, and documentation links all identify v0.11.0 and preserve the distinct limits of routing and readable application evidence.

**Acceptance Scenarios**:

1. **Given** verified public v0.11.0 files, **when** a user opens the getting-started and compatibility pages, **then** the pages identify the published version and give the correct MSI and portable instructions.
2. **Given** the separate #580 research issue, **when** a user reads route and encryption guidance, **then** the release does not promise application-supported key export or universal decryption.

### Edge Cases

- If the accepted candidate changes after review, its checks and release identity must be revalidated on the final source before tagging.
- If an external review bot produces no receipt, record that fact without inventing findings or making the absent bot a new publication prerequisite.
- If the protected registry job waits for owner approval, do not call a GitHub release alone complete registry publication.
- If a public download, checksum, package report, or registry archive disagrees, keep current-public records on the prior verified release and repair the concrete mismatch.
- If a later installed title behaves differently, file a version-specific defect with observed evidence rather than keeping this release indefinitely open.

## Requirements

### Functional Requirements

- **FR-001**: S202 MUST select one v0.11.0 candidate from the accepted main source and include every merged change since v0.10.14 exactly once in its release history.
- **FR-002**: Candidate-bound product, package, dependency, generated-output, specification, and release-note identities MUST agree on v0.11.0, while current-public records remain on verified v0.10.14 until publication.
- **FR-003**: Candidate documentation MUST explain the direct-route operator path and mark it as a candidate until the exact public version exists; published documentation MUST then remove obsolete candidate wording.
- **FR-004**: The candidate MUST pass the repository's local release, documentation, package, dependency, security, formatting, and test checks and the official pull-request checks applicable to its final source.
- **FR-005**: S202 MUST push one official candidate branch and PR, answer every review finding actually received, resolve its threads, and make no more than one explicit second Codex review request.
- **FR-006**: The owner MUST retain final candidate merge, version-tag push, and normal registry approval. No agent action may claim or perform these steps from candidate authorization alone.
- **FR-007**: After the owner merges the candidate, the publication stage MUST use an immutable version tag selecting that accepted merge and must retain the tagged package-certification and registry-job results.
- **FR-008**: S202 MUST independently verify the public Windows assets, checksum sidecars, certification report, and all ten registry archives before advancing current-public records.
- **FR-009**: Current-version records and site guidance MUST advance to v0.11.0 only after verified public availability and must reconcile through a reviewed records change and successful site deployment.
- **FR-010**: S202 MUST keep #580, private-title observations, and an absent security-review bot outside the release completion gate; concrete observed failures remain actionable defects.

### Key Entities

- **Candidate source**: The reviewable versioned tree containing accepted S197 through S201 behavior and the release-preparation changes.
- **Public release**: The exact tagged source, certified Windows downloads, registry crates, and publication status observed after owner actions.
- **Current-version record**: The documentation and machine-readable pointers that identify the latest verified public release.
- **Verification receipt**: A source or artifact identity, command or run, observed result, and the limits of that observation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All ten product crates and both Windows package names report one candidate version, and every unreleased changelog fragment is consumed exactly once.
- **SC-002**: The candidate passes every applicable local and PR check with zero unresolved review findings actually received and no more than two Codex code-review rounds.
- **SC-003**: The public release contains six expected Windows files, all three checksum sidecars match their downloads, and all ten registry archives match non-yanked v0.11.0 records.
- **SC-004**: Every current-public pointer and first-use page identifies verified v0.11.0 after publication, with no remaining claim that direct routing exists only in source.
- **SC-005**: A failed or absent public artifact leaves the previous verified release as the current public record.

## Assumptions

- v0.11.0 is the next public release because S199 through S201 add a new direct-routing capability beyond the v0.10 patch line.
- S197 through S201 are accepted and merged; #580 remains independent research.
- Candidate preparation, PR publication, and review handling are authorized now. The owner will separately perform final merge and authorize tag and registry publication when ready.
