# Feature Specification: S167 dependency alert reconciliation

**Feature Branch**: `codex/s167-dependency-alert-reconciliation`

**Created**: 2026-10-06

**Status**: Draft

**Input**: Reconcile the new Dependabot findings against the released repository, including the four historical proxy spike lockfiles and the documentation site, in one reviewed work slice.

## User Scenarios & Testing

### User Story 1 - Remove vulnerable versions from tracked dependencies (Priority: P1)

As a maintainer, I want the affected tracked dependency inventories to resolve fixed versions so the known advisory findings no longer apply to current source.

**Why this priority**: Eight open alerts identify vulnerable versions in five tracked lockfiles, including one high severity site finding.

**Independent Test**: Inspect all five locked inventories and confirm that each affected package meets its advisory's fixed version while the site and spike projects still resolve.

**Acceptance Scenarios**:

1. **Given** the four historical proxy spike inventories contain rustls 0.23.43, **when** the slice lands, **then** each contains a fixed version at or above 0.23.45.
2. **Given** the site inventory contains vulnerable source-map-js, KaTeX, and DOMPurify versions, **when** the slice lands, **then** every resolved instance meets the applicable fixed version.
3. **Given** the product workspace already resolves a fixed rustls version, **when** the slice lands, **then** its reviewed dependency graph and release identity stay unchanged.

---

### User Story 2 - Preserve site and research usability (Priority: P2)

As a maintainer, I want the dependency correction to preserve the documentation build and historical proxy spike checks, so fixing alerts does not make the repository unusable.

**Why this priority**: The site is the public documentation surface, and the spike manifests remain reproducible research records.

**Independent Test**: Install and build the site from its locked inventory, and run the applicable locked checks for both proxy spike projects.

**Acceptance Scenarios**:

1. **Given** the corrected site lockfile, **when** the site installs and builds from the lockfile, **then** documentation output is generated without dependency resolution errors.
2. **Given** the corrected spike lockfiles, **when** their supported checks run with locked resolution, **then** they complete without dependency resolution errors.

### Edge Cases

- A patched transitive version may coexist with an older vulnerable instance; the result is incomplete until every resolved instance is fixed.
- An upstream package may constrain a vulnerable major line; the correction must preserve the site build and document any needed parent dependency change.
- An existing Dependabot pull request covers only four of the five lockfiles; its useful changes must be reconciled without leaving a duplicate merge path.
- GitHub may delay alert recalculation after a branch push; source lockfile evidence and hosted checks remain the merge gate until default branch scanning catches up.

## Requirements

### Functional Requirements

- **FR-001**: The four proxy spike lockfiles MUST resolve rustls at version 0.23.45 or newer and retain valid registry checksums.
- **FR-002**: The site lockfile MUST resolve source-map-js at 1.2.2 or newer, KaTeX at 0.18.2 or newer, and DOMPurify at 3.4.16 or newer across every occurrence.
- **FR-003**: The site MUST install reproducibly from its committed lockfile and complete its production build and existing automated checks.
- **FR-004**: Both historical proxy spike projects MUST retain their supported locked checks.
- **FR-005**: The product dependency graph, released package versions, and published release records MUST remain unchanged by this maintenance slice.
- **FR-006**: The official slice pull request MUST identify all eight current alerts, reconcile the overlapping Dependabot PR, and pass the repository's required checks before owner merge review.

### Key Entities

- **Advisory finding**: A GitHub alert naming a package, affected manifest, severity, and first fixed version.
- **Locked inventory**: A committed dependency resolution for one site or spike project.
- **Maintenance pull request**: The reviewed S167 change set that brings all affected inventories together.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All eight findings observed on 2026-10-06 have a fixed version in the corresponding proposed locked inventory, with zero remaining vulnerable resolved instances.
- **SC-002**: The site completes frozen installation, production build, and its existing automated checks from the proposed source.
- **SC-003**: Both proxy spike projects complete their applicable locked checks, and the product workspace passes its existing CI parity gate.
- **SC-004**: One official S167 pull request is ready for owner review with required CI green and no unresolved review findings; GitHub's default branch alert closure follows merge and is not claimed before it occurs.

## Assumptions

- The current eight GitHub Dependabot alerts are the scope baseline; newly discovered alerts during the slice will be evaluated for inclusion when they share the same inventories.
- The prior release and product dependency graph are already reviewed; this slice changes historical research and site dependency inventories only.
- The owner retains the final review and merge decision.
