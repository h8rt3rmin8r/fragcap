# Feature Specification: S181 Dependabot reconciliation and green-main recovery

**Feature Branch**: `codex/s181-dependabot-main-reconciliation`

**Created**: 2026-10-27

**Status**: Specified

**Input**: Owner-approved consolidated outline and kickoff: "Approved. Kick off S181 under autopilot and carry it through until all Dependabot items are resolved and main is green."

**Authority**: Issue [#511](https://github.com/h8rt3rmin8r/fragcap/issues/511); master specification sections 20.4, 24.2, 24.3, and 25. The current request authorizes S181 publication and integration through final main verification.

## User Scenarios & Testing

### User Story 1 - Remove outstanding dependency findings (Priority: P1)

The maintainer receives a complete reconciliation of open dependency findings, including updates whose automatic resolution is blocked.

**Why this priority**: Five open alerts name vulnerable site dependencies, including a critical DOMPurify advisory.

**Independent Test**: Inspect every affected manifest and resolved graph, perform fresh ecosystem audits, and read back default-branch alert state after integration.

**Acceptance Scenarios**:

1. **Given** Zod alerts #37 and #38 and open bot PR #510, **When** S181 integrates the patched dependency, **Then** both manifest and lockfile resolve an unaffected version and #510 has a traceable final disposition.
2. **Given** DOMPurify alerts #33, #34, and #35 and the blocking override, **When** S181 updates the dependency graph, **Then** no affected resolved copy remains and the bot can evaluate the graph successfully.
3. **Given** an additional finding appears during the slice, **When** the completion inventory runs, **Then** it is resolved within S181 before completion.

### User Story 2 - Validate merges from a clean environment (Priority: P1)

The maintainer can trust that dependency preparation and root content validation work without a previously populated cache, both before and after merge.

**Why this priority**: The docs main-push gate failed after #508 while its PR skipped the failing step.

**Independent Test**: Execute dependency preparation followed by the original documentation validation from an isolated empty cache and inspect PR and main run evidence.

**Acceptance Scenarios**:

1. **Given** an empty dependency cache, **When** documented CI preparation and validation execute, **Then** locked dependencies are available before offline discovery and every original validation executes.
2. **Given** a proposed dependency change, **When** its PR runs, **Then** it executes the same root content contracts required on main.
3. **Given** unavailable advisory data or a failed dependency download, **When** validation executes, **Then** it reports failure instead of treating unavailable evidence as a pass.

### User Story 3 - Finish with verified main (Priority: P1)

The maintainer receives a merged result with terminal green checks, deployed documentation, and no outstanding Dependabot inventory.

**Why this priority**: The owner explicitly requested delivery through green main rather than a proposed patch or PR handoff.

**Independent Test**: Bind all completion evidence to the final main commit, verify Pages deployment, and freshly query open bot PRs and security alerts.

**Acceptance Scenarios**:

1. **Given** the reviewed S181 source passes required checks, **When** integration completes, **Then** all applicable final-main workflows complete successfully and documentation deploys.
2. **Given** patched main has been reevaluated, **When** the final inventory runs, **Then** zero open Dependabot alerts and zero unresolved Dependabot PRs remain, and #511 records completion evidence.

### Edge Cases

- A warm cache conceals missing dependency preparation; isolated cold-cache evidence is required.
- Multiple dependency copies or manifests can retain a finding after a direct version changes; inspect the complete graphs.
- A green Rust audit does not establish the site advisory result; both ecosystems have explicit evidence.
- Service throttling, scanner latency, an unrelated required-check failure, or a newly opened bot PR delays completion and is reported accurately.
- Historical failed workflow attempts remain incident evidence; completion uses terminal results for the final main commit.

## Requirements

### Functional Requirements

- **FR-001**: Inventory all open Dependabot alerts, PRs, and blocked update attempts at kickoff and completion, including all committed dependency manifests and lockfiles.
- **FR-002**: Resolve Zod findings #37/#38 and DOMPurify findings #33/#34/#35 throughout affected resolved graphs without advisory dismissal or suppression.
- **FR-003**: Incorporate #510's useful update in the consolidated delivery and close it with explicit supersession evidence once equivalent changes are on main.
- **FR-004**: Make documentation validation succeed from a clean cache through explicit locked dependency preparation while preserving offline executable discovery and its existing checks.
- **FR-005**: Execute equivalent root content contracts on PRs and main pushes.
- **FR-006**: Add blocking fresh advisory validation of the complete site dependency graph alongside existing Rust checks on PR, main, weekly, and manual audit runs.
- **FR-007**: Preserve dependency integrity, license policy, reviewed graph evidence, current product versions, release identity, and hidden noninteractive Windows child-process launches.
- **FR-008**: Run required repository and affected-project gates, resolve review findings, integrate through a PR, and verify terminal checks and documentation deployment on final main.
- **FR-009**: Absorb additional Dependabot findings discovered before completion, and close #511 only with a zero-outstanding inventory and final-main evidence.

### Key Entities

- **Dependency finding**: Advisory identity, package, affected manifest, vulnerable range, patched version, and observed state.
- **Validation run**: Exact commit, workflow and job identity, execution condition, terminal result, and evidence link.
- **Reconciliation**: Included findings, superseded PR identity, final main commit, scanner reevaluation, and issue disposition.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All five kickoff alerts are fixed and the final open-alert count is zero.
- **SC-002**: The final unresolved Dependabot PR count is zero, with explicit disposition of #510.
- **SC-003**: A clean-environment regression executes the original root content validation successfully; a failed acquisition or advisory lookup remains a failure.
- **SC-004**: All applicable checks on final main complete successfully, documentation deployment succeeds, and each required outcome has recorded evidence.

## Clarifications

### Session 2026-10-27

- Q: Does the standard pre-push pause end this run? A: The owner's explicit approved scope includes publication, integration, and final main verification, so S181 continues through those steps.
- Q: Can findings be deferred to another issue? A: Every Dependabot finding present before final verification belongs to S181; filing a follow-up alone does not satisfy completion.
- Q: Does zero alerts require ignoring a finding? A: No. Completion requires patched source and default-branch scanner readback.

## Assumptions

- This slice targets source integration and documentation deployment. Current release tags and registry publication do not change.
- Existing feature-branch and PR integration practices remain the delivery mechanism; owner delegation is specific to S181.
- GitHub access and public advisory/package registries are available; transient failures remain visible and retryable.
