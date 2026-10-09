# Feature Specification: S171 session retention and garbage collection

**Feature Branch**: `codex/s171-session-retention-garbage-collection`

**Created**: 2026-10-09

**Status**: Implemented; local gates passed, hosted review pending

**Input**: Owner-authorized S171 autopilot implements issue #458, including finite routine retention, backlog reclamation, preserved empty containers and separately explicit purge.

## User Scenarios & Testing

### User Story 1 - Finite ordinary session history (Priority: P1)

An operator repeatedly captures or calibrates without accumulating eligible operational contents and obsolete owner records indefinitely. Routine maintenance retains empty session containers.

**Why this priority**: This addresses the storage lifecycle defect rather than changing presentation alone.

**Independent Test**: A controlled backlog reaches the declared age, count and byte limits after maintenance while active, saved and recovery-required data remain intact.

**Acceptance Scenarios**:

1. **Given** eligible completed default sessions, **When** maintenance runs at a documented session boundary, **Then** expired or excess contents are reclaimed with exact results and empty containers remain.
2. **Given** an active lease or unresolved recovery obligation, **When** maintenance runs, **Then** the session is preserved with its exact refusal reason.
3. **Given** an explicitly saved bundle or custom destination, **When** routine maintenance runs, **Then** its declared retained-evidence policy preserves it.

### User Story 2 - Review and reclaim an existing backlog (Priority: P1)

An operator inventories completed session contents, sees eligible bytes and refusal reasons, then explicitly collects the reviewed population, including historical retained evidence when selected deliberately.

**Why this priority**: Existing retained output cannot silently acquire a new expiration promise.

**Independent Test**: A preview authorizes an unchanged exact population; actual deletion accounts only bytes successfully removed, and interruption can be retried.

**Acceptance Scenarios**:

1. **Given** legacy retained bundles, **When** default collection is previewed, **Then** they remain retained; explicitly including retained evidence changes the proposed scope visibly.
2. **Given** a changed population, **When** its old proposal is applied, **Then** it refuses before deletion.
3. **Given** interruption during collection, **When** collection resumes, **Then** durable retirement state supports idempotent completion without losing recovery authority or claiming unremoved bytes.

### User Story 3 - Explicit empty-container purge and truthful diagnosis (Priority: P2)

An operator can separately request removal of eligible exact empty containers. Read-only Doctor recognizes retained empty containers and reserves actionable finding capacity for actual residue.

**Why this priority**: The owner explicitly distinguishes content collection from directory removal.

**Independent Test**: Routine collection preserves containers, explicit purge removes only proven eligible empty containers, and a large empty backlog does not hide a current recovery obligation.

**Acceptance Scenarios**:

1. **Given** collected empty containers, **When** ordinary Doctor runs, **Then** no false malformed-session finding appears and no files change.
2. **Given** an explicit reviewed purge request, **When** applied, **Then** only eligible exact empty containers are removed; newly nonempty or unrelated directories are refused.

### Edge Cases

- Malformed, unsupported, unreadable or unknown files, custom roots, legacy ownership without generation proof, active generations, partial manifests, incomplete journals, pending sensitive actions and recovery obligations remain explicit unresolved results.
- Junctions, symbolic links, alternate paths, hard-link aliases and concurrent replacement cannot turn tool-owned collection into unrelated deletion.
- Failed attempts are collectible only with whole-session terminal authority; a single terminal resource row is insufficient.
- Partial deletion preserves durable retirement intent outside the collected population; success and reclaimed-byte totals describe actual effects.
- Enumeration bounds, retained populations exceeding configured limits, inaccessible entries and partial inventory remain visible rather than silently evicted.

## Requirements

### Functional Requirements

- **FR-001**: New default-root operational sessions MUST declare finite retention in authorization, terminal presentation and bundle metadata. Default limits are 30 days, 20 completed managed sessions and 2 GiB of eligible managed contents; oldest eligible sessions are collected first when any limit is exceeded.
- **FR-002**: Explicitly retained output and custom destinations MUST retain evidence until explicit cleanup. Historical bundles keep their existing retention promise unless the operator explicitly includes retained evidence in the reviewed cleanup scope.
- **FR-003**: Maintenance MUST run at documented effectful session boundaries and through an explicit preview/apply command. Plan-only and read-only Doctor MUST perform no collection.
- **FR-004**: Whole-session eligibility MUST reconcile exact ownership, inactive generation leases, complete terminal journal/manifest state and every unresolved routing, trust, artifact or sensitive-action obligation before any deletion.
- **FR-005**: Inventory MUST distinguish eligible, retained, active, recovery-required, empty and unresolved sessions, reporting paths, recoverable bytes, policies, refusal reasons and visible scan limitations.
- **FR-006**: Preview authorization MUST bind the exact population and policy. Changed state MUST refuse before effects; destructive cleanup MUST require an explicit apply instruction.
- **FR-007**: Collection MUST preserve every session container directory. Removing eligible empty containers MUST require a separate explicit purge scope.
- **FR-008**: Path ownership MUST remain exact throughout mutation. Collection MUST refuse reparse traversal, outside aliases, unknown contents and substituted objects; a registered custom root alone MUST NOT grant disposable ownership.
- **FR-009**: Durable collection state MUST permit interrupted deletion and owner-registry retirement to reconcile idempotently. Results MUST report removed, failed and not-attempted objects and count only successfully reclaimed bytes.
- **FR-010**: Collected empty containers MUST remain recognizable without their former files and MUST NOT consume actionable residue capacity or become false incomplete sessions. Purge MUST recheck emptiness and authority.
- **FR-011**: Bounded maintenance and diagnosis MUST expose their limits and support progression through a large backlog without old completed history indefinitely exhausting actionable inventory capacity.
- **FR-012**: Repository-controlled regression coverage MUST include completed, failed/empty, saved, custom-root, active, recovery-required, legacy, malformed, interrupted, replaced-path and large-backlog cases without real games or real trust-store changes.
- **FR-013**: Master specification, public CLI/storage documentation and changelog MUST describe actual retention, cleanup, purge, refusals and limitations. Sensitive-only cleanup and whole-user fresh-start remain distinct operations.

### Key Entities

- **Retention policy**: Managed finite lifetime or explicitly retained evidence, with declared origin and limits.
- **Session collection proposal**: Exact ownership-qualified population, eligibility, policy, bytes and authorization identity.
- **Retirement record**: Durable collection progress and terminal empty-container ownership after bundle contents are removed.
- **Collection report**: Actual object effects, reclaimed bytes, preserved containers, unresolved results and limits.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every scoped lifecycle class produces its expected eligibility and collection result with zero unrelated, active or recovery-required deletion.
- **SC-002**: At least 1,000 synthetic completed or empty sessions cannot prevent diagnosis of a current actionable session; every reached inventory bound is reported.
- **SC-003**: Routine collection preserves 100% of empty containers; explicit purge removes only the exact reviewed eligible empty population.
- **SC-004**: Interrupted and failed collection retries conserve surviving bytes, recovery authority and actual reclaimed-byte accounting.
- **SC-005**: All issue #458 acceptance criteria map to implementation and controlled verification evidence before final handoff.

## Assumptions

- S168 diagnostic presentation, S169 correlation and S170 exact artifact access are merged foundations, not new compatibility or incident-root-cause claims.
- Age/count/byte defaults are routine engineering decisions authorized by #458; saved evidence remains explicitly retainable and historical retention is preserved.
- Dependency PR #470, releases, IGDB #155 and community sync #94 are separate work.
- The user authorizes push, official PR publication and at most two external review rounds; the owner performs final merge.

## Clarifications

### Session 2026-10-09

- Q: Does ordinary GC remove empty session directories? A: No; only separately explicit purge removes eligible exact empty containers (owner clarification in #458).
- Q: Can historical retained evidence automatically expire? A: No; preserve the old promise and require explicit inclusion in backlog collection.
- Q: What finite defaults apply to new operational contents? A: 30 days, 20 completed managed sessions and 2 GiB, with oldest eligible contents collected first; retained and recovery-required data are reported independently.
