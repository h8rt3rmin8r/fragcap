# Feature Specification: Doctor Residue Guidance

**Feature Branch**: `codex/135-doctor-residue-guidance`

**Created**: 2026-09-09

**Status**: Draft

**Input**: User description: "Spec out and implement S135 under autopilot, closing issue #373 by rendering native Deep Capture residue as clear human guidance instead of an internal-field blob."

## Clarifications

### Session 2026-09-09

- Q: What is the minimum supported human Doctor report width? -> A: 40 display columns.

## User Scenarios & Testing

### User Story 1 - Understand a Blocking Residue Finding (Priority: P1)

An operator sees an earlier Deep Capture session residue finding and can understand what was found, whether an active owner was proven, why Deep Capture is blocked, and what safe action is available without decoding internal field names.

**Why this priority**: S134 deliberately directs pending prior-session recovery to Doctor. That safe boundary is usable only when Doctor explains the finding and consequence in ordinary language.

**Independent Test**: Classify an abandoned session-owner record and read the human report alone. The report identifies the earlier incomplete session, says no active owner was proven, explains that Deep Capture is blocked, and names the exact confirmed Doctor cleanup command.

**Acceptance Scenarios**:

1. **Given** an abandoned session-owner record with exact cleanup eligibility, **when** Doctor renders the human report, **then** the finding states that an earlier session ended without retiring its owner record, no active owner was proven, and Deep Capture remains blocked until confirmed cleanup.
2. **Given** a stale recoverable journal resource, **when** Doctor renders it, **then** the report names the resource category, incomplete cleanup condition, consequence, and `fragcap doctor --fix` as the review-and-confirm action.
3. **Given** an active session-owner record, **when** Doctor renders it, **then** the report says an active owner is proven, offers no cleanup, and does not describe it as abandoned or safe to retire.
4. **Given** an ambiguous or unknown resource, **when** Doctor renders it, **then** the report states what could not be proven and directs inspection without claiming that cleanup is safe.

---

### User Story 2 - Read a Stable Human Layout (Priority: P2)

An operator can scan Doctor findings at the default report width and at supported narrow widths without a long session or resource identifier moving the status and detail columns out of alignment.

**Why this priority**: The current dynamic check name defeats the report's fixed-column layout and separates wrapped continuation text from the finding it describes.

**Independent Test**: Render the same report with short, long, and non-ASCII identities at the default width and each supported narrow boundary. Every ordinary line fits the selected width, status remains visually attached to its finding, and continuation indentation is derived from the selected layout.

**Acceptance Scenarios**:

1. **Given** a long session and resource identity, **when** the default human report renders, **then** no dynamic identifier changes the status column and every ordinary line remains within 80 display columns.
2. **Given** the minimum supported narrow width, **when** the same report renders, **then** the finding uses a readable compact layout, wraps on display-cell boundaries, and retains every value without truncation.
3. **Given** colorized human output, **when** the report renders at either layout, **then** color does not change visible alignment or line-width accounting.
4. **Given** a session or resource identity containing non-ASCII characters, **when** the report wraps it, **then** complete characters are preserved and display width rather than encoded byte length controls alignment.

---

### User Story 3 - Preserve Exact Machine and Recovery Truth (Priority: P3)

An automation consumer continues to receive stable exact residue identity and lifecycle facts while the improved human wording leaves cleanup selection, confirmation, and execution unchanged.

**Why this priority**: Human simplification must not hide observation authority, invent safety, or create a second recovery policy.

**Independent Test**: Render controlled healthy, active, stale, cleanup-failed, and unknown findings in machine-readable mode, then compare offered actions before and after the presentation change. Exact identity, state, health, ownership authority, and actual Doctor cleanup-action eligibility remain available, and the action set is identical.

**Acceptance Scenarios**:

1. **Given** any native residue finding, **when** machine-readable output is selected, **then** its stable session identity, resource identity, resource kind, lifecycle state, health, ownership authority, and actual Doctor cleanup-action eligibility are available as structured non-secret values.
2. **Given** recoverable and non-recoverable findings, **when** Doctor determines actions, **then** only the same exact recoverable findings offer the existing cleanup action.
3. **Given** a partial or failed cleanup record, **when** human wording is produced, **then** retained evidence and the exact retry boundary remain visible without claiming successful cleanup.
4. **Given** the improved presentation, **when** Doctor runs read-only, **then** it creates no filesystem, trust, routing, listener, process-control, or cleanup effect.

### Edge Cases

- A finding has no recorded session identifier and uses its stable bundle-derived identity.
- Session and resource identifiers are empty, extremely long within existing inventory bounds, contain whitespace, or contain non-ASCII characters.
- A single report contains several findings of the same resource kind and health.
- Healthy retained history, a generation-proven active owner, stale recoverable residue, cleanup failure, unknown authority, and unsupported state appear independently.
- A resource is stale or cleanup-failed but has no exact recovery action.
- A recoverable unknown record still requires explicit confirmation and must not be described as proven safe.
- Human output is plain or colorized, default width or minimum supported narrow width.
- A path-like or otherwise unbreakable identity token is wider than the selected line width.
- Machine-readable output contains characters that require escaping and remains one valid record per line.

## Requirements

### Functional Requirements

- **FR-001**: Human native residue findings MUST lead with a short stable label and a plain-language diagnosis rather than a dynamic session/resource name or flattened internal key-value sequence.
- **FR-002**: Every human residue diagnosis MUST explain what was observed, its consequence for Deep Capture readiness, and whether active ownership was proven.
- **FR-003**: Stable session and resource identity MUST remain available as secondary human context without controlling the report's status-column position.
- **FR-004**: Abandoned session-owner guidance MUST state that the earlier session ended without retiring its owner record, that no active owner was proven, and that Deep Capture is blocked pending review and confirmed cleanup of all eligible inactive Deep Capture records.
- **FR-005**: A generation-proven active owner MUST be described as active, MUST remain non-blocking when current policy says it is healthy, and MUST NOT receive a cleanup action.
- **FR-006**: Healthy retained history MUST be described as completed history and MUST NOT be presented as residue requiring cleanup.
- **FR-007**: Stale and cleanup-failed resources MUST distinguish incomplete cleanup from a failed cleanup attempt and MUST state exact recovery availability truthfully.
- **FR-008**: Unknown, unsupported, ambiguous, and non-recoverable findings MUST explain their limitation without claiming absence, safety, or cleanup eligibility.
- **FR-009**: Human remediation for a finding with an available Doctor cleanup action MUST tell the operator to run `fragcap doctor --fix` and disclose that one review and confirmation covers all eligible inactive Deep Capture records, without requiring internal terms such as recovery authority.
- **FR-010**: The human report MUST preserve its existing section ordering, status vocabulary, separate Capture and Deep Capture verdicts, and ordinary non-residue check meaning.
- **FR-011**: The default human report MUST keep ordinary lines within 80 display columns except an indivisible token that cannot fit without altering its value.
- **FR-012**: The human renderer MUST support widths of 40 display columns and greater and MUST use a compact readable layout at and below the width where aligned columns cannot retain useful detail space.
- **FR-013**: Wrapped detail, secondary context, and remediation continuations MUST align to the actual selected layout rather than a constant belonging to a different layout.
- **FR-014**: Visible alignment and wrapping MUST count terminal display cells, preserve complete Unicode scalar values, and ignore non-printing color sequences.
- **FR-015**: Human output MUST retain every observed non-secret identity value without silent truncation or normalization.
- **FR-016**: Machine-readable output for every native residue finding MUST expose stable session identity, resource identity, resource kind, lifecycle state, health, ownership authority, and actual Doctor cleanup-action eligibility as structured fields.
- **FR-017**: Machine-readable output MUST remain one valid record per line, retain the existing common check fields and verdict records, and expose no private key, capability, payload, or newly added local path.
- **FR-018**: Cleanup action selection, confirmation, execution, shared recovery authority, active-resource preservation, partial-failure behavior, and exit semantics MUST remain unchanged.
- **FR-019**: Read-only Doctor execution MUST remain free of filesystem, trust-store, routing, listener, process-control, and cleanup effects.
- **FR-020**: Tests MUST cover every residue health class, recoverable and non-recoverable branches, long and non-ASCII identities, default and narrow layouts, plain and color output, structured escaping, and the abandoned session-owner regression without requiring a game, capture driver, elevation, real trust mutation, or external cleanup.
- **FR-021**: Documentation MUST explain the improved human and machine residue contract without declaring Deep Capture complete before issue #334.
- **FR-022**: S135 MUST NOT broaden cleanup ownership, add a new recovery policy, change inventory classification, implement broader first-run UX from issue #332, or claim final completion.

### Key Entities

- **Residue Diagnosis**: The operator-facing explanation of one observed native resource, its health, readiness consequence, ownership proof, and safe next action.
- **Residue Identity Context**: The stable session and resource identity retained as secondary human context and exact machine-readable fields.
- **Human Report Layout**: The selected aligned or compact arrangement and its width-specific continuation indentation.
- **Machine Residue Record**: One structured check record carrying common Doctor fields plus exact non-secret native residue facts.
- **Recovery Eligibility**: The existing truth that an exact shared recovery action is or is not available for a finding.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of controlled residue health and recovery combinations produce a human diagnosis that states the observed condition, active-ownership truth, readiness consequence, and available next action without flattened key-value prose.
- **SC-002**: The abandoned session-owner regression report contains all four required meanings: earlier incomplete session, unretired owner record, no proven active owner, and review and confirmation of all eligible inactive Deep Capture records.
- **SC-003**: One hundred percent of ordinary human report lines fit the selected default or supported narrow display width, except indivisible tokens whose exact preservation is explicitly permitted.
- **SC-004**: Plain and colorized renderings have identical visible text, alignment, and wrapping for every controlled layout case.
- **SC-005**: One hundred percent of controlled native residue records expose all seven required structured machine facts and parse as one valid record per line.
- **SC-006**: The before-and-after set of offered cleanup actions is identical for every controlled health and recoverability combination.
- **SC-007**: Zero internal key-value blobs or the phrase `recovery authority` appear in human native residue findings or their remediations.
- **SC-008**: The focused Doctor suite and complete repository verification gate pass with no new dependency package and no prohibited capability.

## Assumptions

- The S124 native residue inventory remains the sole authority for health, ownership, and recorded recovery plans; existing Doctor action selection remains the authority for whether cleanup is offered.
- The S109 and S124 shared recovery planner remains the sole source of cleanup actions.
- The existing 80-column default is retained and 40 display columns is the minimum supported human-report width.
- Machine-readable output may add fields to native residue check records while retaining all existing common fields and verdict records.
- Local bundle paths are not required in the new structured residue object because stable session and resource identities already provide machine correlation without newly exposing profile paths.
- No new third-party dependency is expected because terminal-width and Unicode display-width support already exist in the workspace.
