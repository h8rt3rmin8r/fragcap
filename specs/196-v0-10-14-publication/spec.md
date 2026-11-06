# Feature Specification: v0.10.14 Publication

**Feature Branch**: `codex/s196-v01014-publication`

**Created**: 2026-11-06

**Status**: In progress

**Input**: Publish the already reviewed and merged v0.10.14 candidate completely, verify every public artifact and registry archive, exercise the installed public product at its documented defaults, reconcile current-public records through a reviewed and merged pull request, and continue without another owner halt until everything is live and public.

## Clarifications

### Session 2026-11-06

- Q: Does this authorization include the annotated tag, protected release approval, public publication, records pull request, and records merge? -> A: Yes. Complete all of them without another owner halt.
- Q: Are publication and current-record reconciliation separate work slices? -> A: No. S196 owns the complete publication journey, with distinct evidence phases inside one slice.
- Q: When may installed-product and private-title checks run? -> A: Only after public publication. Concrete failures become version-specific GitHub issues and do not become invented pre-release conditions.

## User Scenarios & Testing

### User Story 1 - Obtain the complete public release (Priority: P1)

Users can obtain v0.10.14 from one exact reviewed source through the public GitHub release and crates.io, with independently verifiable package and archive identities.

**Why this priority**: The source candidate has already merged and all merge-commit checks passed. The remaining user value is the public release itself.

**Independent Test**: The annotated tag peels to the accepted merge, the tagged workflow succeeds, six GitHub files and their sidecars verify, and all ten exact crate versions are public and non-yanked.

**Acceptance Scenarios**:

1. **Given** accepted merge `5efb38ee93daff140e1ba79f57eea0a93d5b572f`, **when** v0.10.14 is tagged and the release workflow completes, **then** the tag, GitHub release, certified packages, and registry archives all name that source and version.
2. **Given** normal crates.io environment protection, **when** publication reaches protected approval, **then** the authenticated owner approval is recorded without weakening or bypassing the environment configuration.

---

### User Story 2 - Exercise the released product (Priority: P2)

The owner receives an installed-product observation from the public v0.10.14 package using Fragcap's documented default storage, retention, and cleanup behavior.

**Why this priority**: Product behavior is assessed after release. The public package, not historical or locally rebuilt bytes, is the authority for this observation.

**Independent Test**: A public v0.10.14 installer is acquired from the verified release, installed normally, its effective canonical target and session locations are observed, controlled installed commands run, and supported cleanup leaves an exact accounting of retained native state.

**Acceptance Scenarios**:

1. **Given** the verified public installer, **when** the installed journey runs, **then** no custom target store, session directory, bundle directory, or application-data override is supplied.
2. **Given** an observed failure in the active release, **when** evidence is sufficient and privacy-scrubbed, **then** a concrete v0.10.14 GitHub issue records it while publication truth remains unchanged.

---

### User Story 3 - See v0.10.14 as current everywhere (Priority: P3)

Users and contributors see verified v0.10.14 public identity consistently in documentation, release records, the site, and frozen review authority.

**Why this priority**: Current markers must follow verified public bytes, never precede them.

**Independent Test**: A records pull request derived from public readback passes repository checks and hosted review, merges, deploys Pages successfully, and leaves issue #567 and the v0.10.14 milestone complete.

**Acceptance Scenarios**:

1. **Given** independently verified public files and registry archives, **when** current records advance, **then** every changed marker names v0.10.14, its exact public source, and the observed artifact identities.
2. **Given** the records pull request is green and reviewed, **when** it is merged under this authorization, **then** main and the public site converge without another owner merge halt.

### Edge Cases

- An existing v0.10.14 tag or release must be identical to the accepted source and intended publication; a divergent object is not overwritten.
- A failed tagged job remains failed evidence and is corrected through the normal workflow rather than represented as published success.
- Protected crates.io approval uses the configured environment review and never changes the protection settings.
- Registry propagation may lag the workflow. Public verification retries bounded reads without republishing an already accepted crate.
- A public asset may exist before all registry archives propagate. Records remain unchanged until both authorities are complete.
- Installed-product commands may require elevation or unavailable private-title conditions. Only actually observed behavior is claimed, and absence of a concrete failure is not universal compatibility proof.
- Publication records must not reuse candidate or merge-check artifacts as proof of public bytes.

## Requirements

### Functional Requirements

- **FR-001**: S196 MUST create annotated tag `v0.10.14` only when it peels to accepted merge `5efb38ee93daff140e1ba79f57eea0a93d5b572f` and no divergent tag exists.
- **FR-002**: S196 MUST run the repository's tagged release workflow to successful completion, including identity validation, final Windows package certification, GitHub release creation, and protected crates.io publication.
- **FR-003**: S196 MUST satisfy the existing crates.io environment approval through its normal authenticated review path without changing, bypassing, or weakening protection.
- **FR-004**: S196 MUST independently download and verify all six GitHub release files, all three checksum sidecars, the certification report authority, package identity, source revision, lifecycle completeness, and zero unreported findings.
- **FR-005**: S196 MUST independently verify all ten v0.10.14 crates.io records are public and non-yanked and that each downloaded archive matches its registry checksum.
- **FR-006**: S196 MUST exercise the verified public installed product only after publication, using documented default target, session, retention, and cleanup behavior without silent custom paths.
- **FR-007**: S196 MUST file each concrete privacy-scrubbed active-release defect discovered by installed-product observation as a new v0.10.14 GitHub issue rather than converting it into a retroactive publication condition.
- **FR-008**: S196 MUST create a publication handoff and advance current-public records, documentation, site content, and frozen review authority only from verified public bytes and registry archives.
- **FR-009**: S196 MUST deliver the records change through an official pull request, resolve every review comment, use no more than two hosted Codex review rounds, require terminal-successful applicable checks, and merge it under the owner's explicit end-to-end authorization.
- **FR-010**: S196 MUST verify the merged records commit and public Pages deployment, close #567, complete its project and milestone state, and leave the repository on clean synchronized main.
- **FR-011**: S196 MUST preserve failed or unavailable observations literally and MUST NOT claim independent whole-product security acceptance, universal private-title compatibility, or unperformed installed behavior.
- **FR-012**: S196 MUST retain publication, public readback, installed-product observation, records PR, records merge, and Pages deployment as distinct evidence phases even though they belong to one work slice.

### Key Entities

- **Published Release**: The public GitHub v0.10.14 release, its tag, source revision, publication timestamp, and six downloadable files.
- **Registry Set**: The ten exact v0.10.14 crate records, yank states, registry checksums, and independently downloaded archives.
- **Installed Observation**: The public package identity, effective canonical product locations, commands actually exercised, native artifacts retained, cleanup performed, and explicit limits.
- **Publication Record**: The repository-controlled current markers, handoff document, asset and crate receipts, and frozen review candidate derived from public evidence.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One annotated v0.10.14 tag peels to the accepted merge, and all four tagged release jobs complete successfully.
- **SC-002**: Exactly six public GitHub release files are independently downloaded; all three sidecars validate and every API digest, size, and computed SHA-256 agrees.
- **SC-003**: Exactly ten v0.10.14 crates are public and non-yanked, and all ten independently downloaded archives match registry checksums.
- **SC-004**: The installed public package observation names its exact package identity, effective default locations, retained artifacts, cleanup, and every unverified limitation without using a custom storage path.
- **SC-005**: The publication-record pull request reaches zero unresolved review threads and terminal success for every applicable check, then merges and produces a successful public Pages deployment.
- **SC-006**: Issue #567 and the v0.10.14 milestone are complete, and the repository finishes on clean synchronized main with v0.10.14 shown as the current public release.

## Assumptions

- The merged S195 candidate and merge-commit workflow results remain the accepted source authority.
- Existing repository secrets and normal protected-environment approval are available to the tagged workflow.
- The current Windows host can install the public MSI through the established hidden noninteractive tooling; unavailable elevation or private-title prerequisites remain explicit observation limits.
- No unrelated source implementation change is required for publication. Any discovered release defect is filed separately unless it prevents the repository from truthfully publishing the already accepted bytes.
