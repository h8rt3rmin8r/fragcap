# Feature Specification: Native Deep Capture Doctor Readiness

**Feature Branch**: `codex/124-native-doctor-readiness`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Spec out and implement S124 under autopilot, closing the native Deep Capture readiness and residue boundary described by issue #321."

## User Scenarios & Testing

### User Story 1 - Read Separate Capture and Deep Capture Verdicts (Priority: P1)

An operator can run Doctor once and understand independently whether ordinary Capture and native Deep Capture are ready. A problem confined to one mode does not make the other mode's verdict ambiguous.

**Why this priority**: The current single verdict can say a machine is ready while Deep Capture has unresolved residue, or can imply Capture is blocked by a Deep Capture-only concern. Operators need the mode boundary before acting.

**Independent Test**: Classify a matrix containing Capture-only failures, Deep Capture-only failures, both-mode failures, and fully ready inputs. Confirm that the human and machine-readable reports give the same two verdicts and preserve the existing command exit contract.

**Acceptance Scenarios**:

1. **Given** Capture prerequisites pass and a Deep Capture capability is unavailable, **when** Doctor runs, **then** it reports Capture ready and Deep Capture not ready with the exact blocking check.
2. **Given** Deep Capture prerequisites pass but the capture driver is absent, **when** Doctor runs, **then** both modes report not ready because Deep Capture includes packet capture.
3. **Given** all prerequisites pass and no unresolved native residue exists, **when** Doctor runs, **then** both mode verdicts are ready and the command succeeds.
4. **Given** human or JSON output is selected, **when** the same inputs are classified, **then** both forms carry identical check states and mode verdicts.

---

### User Story 2 - Audit Native Readiness and Residue (Priority: P2)

An operator can see every native Deep Capture readiness and residue class Doctor can prove from existing bundle, journal, trust, listener, and session-owner authorities. Active work remains distinct from stale work, and an unreadable or bounded-out inventory is never shown as clean.

**Why this priority**: Legacy aggregate fields still report port and proxy process state as unobservable even though the native lifecycle records now exist. Folding journal failures into a generic stale-manifest warning loses the exact repair boundary.

**Independent Test**: Feed Doctor controlled inventories containing healthy completed bundles, active sessions, crash prefixes, cleanup failures, recovery refusals, unknown journal versions, malformed records, trust mismatches, retained sensitive artifacts, and scan-limit exhaustion. Confirm one stable finding per observed class and no invented ownership.

**Acceptance Scenarios**:

1. **Given** an active session-owner registration and matching journal, **when** Doctor scans it, **then** the listener, route, capture, trust, launch, and artifact obligations are classified as active rather than stale.
2. **Given** an owner is no longer live and its journal contains nonterminal resources, **when** Doctor scans it, **then** every latest resource state and exact recovery eligibility is visible.
3. **Given** a journal is complete and every latest resource is terminal, **when** Doctor scans it, **then** it is healthy history and does not create a cleanup action.
4. **Given** a record is malformed, unknown-versioned, outside the trusted root, or omitted because a finite scan bound was reached, **when** Doctor reports, **then** the affected authority is unknown with a stable reason and never clean.
5. **Given** no external proxy exists in the product path, **when** Doctor reports backend readiness, **then** it names the compiled native backend and emits no mitmdump installation guidance or process placeholder.

---

### User Story 3 - Repair Only Proven Owned Residue (Priority: P3)

An operator can request Doctor fixes and receive one confirmation-gated action only for residue whose exact ownership and recovery operation are already established by the native journal or manifest authority. Active sessions and ambiguous resources are preserved.

**Why this priority**: Doctor is the recovery surface promised by the constitution. Broad deletion is unsafe, while guidance without exact action leaves proven residue unresolved.

**Independent Test**: Drive the action loop with recoverable, active, ambiguous, already-terminal, partially failed, and successfully recovered inventories. Confirm that only report-carried actions are offered, each outcome is recorded honestly, and the re-check reflects the remaining state.

**Acceptance Scenarios**:

1. **Given** stale resources with exact journal recovery actions, **when** the operator confirms cleanup, **then** Doctor invokes the shared recovery implementation and reports each success or failure.
2. **Given** an active owner, an unknown journal version, or insufficient ownership evidence, **when** fixes are offered, **then** Doctor does not mutate that resource and explains the refusal.
3. **Given** cleanup succeeds for some resources and fails for others, **when** Doctor re-checks, **then** successful resources no longer appear stale and failures remain explicit.
4. **Given** noninteractive or machine-readable operation without the existing explicit confirmation contract, **when** cleanup is requested, **then** no mutation occurs.

### Edge Cases

- A session-owner registration names a PID that has been reused by an unrelated live process.
- A registration file is truncated, oversized, duplicated, or names a non-absolute or unreadable bundle; an exact registered custom bundle remains supported.
- A journal is a valid crash prefix, a completed stream, an unknown version, an invalid transition sequence, or over its byte or record bound.
- A completed manifest disagrees with its resource journal or declared artifact paths.
- An exact listener endpoint is journaled but currently occupied by an unrelated process.
- Current-user trust exists without readable matching bundle authority, or bundle authority exists without trust.
- CA, leaf, key-log, route, process-trace, application, cleanup, or manifest artifacts are missing independently.
- The inventory reaches its directory depth, entry, session, or finding bound.
- Recovery is already in progress under another live owner.
- A cleanup attempt is interrupted after some journal transitions have synchronized.
- Deep Capture is unavailable on a non-Windows build while ordinary offline commands remain usable.

## Requirements

### Functional Requirements

- **FR-001**: Doctor MUST produce independent Capture and Deep Capture readiness verdicts from the same ordered check set.
- **FR-002**: Deep Capture readiness MUST include every prerequisite shared with Capture plus native proxy, loopback, trust, storage, and unresolved-resource requirements.
- **FR-003**: The command exit MUST remain unsuccessful when either mode has a blocking failure and successful when neither mode has one; warnings MUST remain non-blocking.
- **FR-004**: Human and machine-readable reports MUST expose the same check identities, states, details, remediations, and separate mode verdicts in stable order.
- **FR-005**: The production Doctor surface MUST identify the compiled native backend directly and MUST contain no external proxy discovery, mitmdump installation guidance, or legacy proxy-process placeholder.
- **FR-006**: One bounded read-only inventory MUST classify session-owner registrations, resource journals, bundle manifests, listener endpoints, routing obligations, launch and capture obligations, CA and leaf material, trust entries, key logs, process traces, application evidence, cleanup streams, and incomplete bundles when those authorities exist.
- **FR-007**: Every inventory item MUST retain its session, bundle, resource kind, latest lifecycle state, ownership authority, observed health, and exact recovery eligibility without exposing secret material.
- **FR-008**: Inventory states MUST distinguish absent, healthy, active, stale, cleanup-failed, unknown, and unsupported conditions where applicable.
- **FR-009**: An active session MUST be established by a bounded session-owner record and process-generation evidence sufficient to prevent PID reuse from transferring ownership.
- **FR-010**: A listening port without exact session ownership MUST remain an unrelated occupied endpoint and MUST NOT be called an orphaned proxy or cleanup target.
- **FR-011**: Complete terminal journals MUST be healthy history; crash prefixes with nonterminal resources MUST be stale or cleanup-failed according to their latest transitions; unknown or invalid journals MUST be unknown.
- **FR-012**: Scan errors, invalid paths, unsupported versions, malformed records, and every finite-bound overflow MUST produce explicit unknown findings and MUST NOT be normalized to absence or health.
- **FR-013**: Doctor MUST derive cleanup actions only from the existing journal recovery plan and manifest trust or artifact authority; it MUST NOT create a second recovery policy.
- **FR-014**: Every cleanup action MUST remain visible before execution, require the existing confirmation contract, reuse the shared native recovery implementation, and preserve active or ambiguous resources.
- **FR-015**: Partial cleanup MUST report each performed, refused, skipped, and failed outcome and MUST retain the evidence needed for a later exact retry.
- **FR-016**: Deep Capture findings MUST distinguish runtime readiness from historical residue. Healthy retained output MUST NOT block a new session solely because it exists.
- **FR-017**: The read-only Doctor path MUST create no directory, lock, registry entry, listener, trust mutation, route mutation, or process handle.
- **FR-018**: The implementation MUST remain within query-only process enumeration and MUST NOT add memory rights, injection, hooks, system proxy changes, executable modification, target key extraction, or cleanup of unrelated resources.
- **FR-019**: The inventory and report MUST be finite in time, directory depth, entries, sessions, and findings, with every truncation visible.
- **FR-020**: Tests MUST cover every readiness combination and residue state without a game, account, Internet service, elevation, capture driver, real trust mutation, or unrelated process cleanup.
- **FR-021**: Documentation MUST describe the native Doctor authority and the runtime-versus-packaging boundary without declaring Deep Capture complete before issue #334.
- **FR-022**: S124 MUST NOT claim installer, archive, offline release smoke, artifact-size, upgrade, repair, uninstall, supply-chain, Windows integration, or final completion work owned by issues #323 through #334.
- **FR-023**: A legacy owner record without a generation lease MUST remain unproven during startup. Terminal journal evidence MAY retire it automatically; otherwise only an explicit confirmed Doctor repair MAY replay its exact journal plan and retire its exact owner record.

### Key Entities

- **Mode Readiness Verdict**: The ready or not-ready result for Capture or Deep Capture, derived from the checks applicable to that mode.
- **Native Residue Inventory**: A finite read-only collection of session and resource observations plus scan limitations.
- **Session Owner Record**: Bounded evidence connecting a session bundle to the process generation that currently owns it.
- **Resource Finding**: The latest observed state, health, ownership authority, and recovery eligibility for one journaled or manifested resource.
- **Inventory Limitation**: A stable reason and count for unreadable, malformed, unsupported, unsafe, or unretained inventory evidence.
- **Recovery Offer**: A confirmation-gated action copied from an existing exact recovery plan, never inferred from an endpoint or filename alone.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of readiness-matrix cases produce independent Capture and Deep Capture verdicts that agree between human and machine-readable output.
- **SC-002**: One hundred percent of controlled journal resource states map to exactly one healthy, active, stale, cleanup-failed, unknown, or unsupported finding with no false clean result.
- **SC-003**: Zero generation-proven active, out-of-root, or unrelated resources are offered for cleanup across the controlled recovery matrix. Legacy unproven ownership requires an explicit confirmation before exact recovery.
- **SC-004**: One hundred percent of exact recoverable resource actions originate from the shared journal or session-owner authority and retain their performed, refused, skipped, or failed outcome.
- **SC-005**: Every injected scan error and finite-bound overflow produces a stable visible limitation, and none is reported as absence.
- **SC-006**: Human output remains within the existing 80-column contract and JSON remains one valid record per line with stable identities.
- **SC-007**: A read-only Doctor run causes zero filesystem, trust-store, routing, listener, or process-control mutations.
- **SC-008**: The complete repository verification gate passes with no prohibited capability and no new dependency package.

## Assumptions

- S104, S107, S109, S119, and S123 already own the native backend, sensitive artifacts, resource journal and recovery, address-family readiness, and process evidence used by this slice.
- Issue #321 supplies runtime and installed-artifact diagnostic contracts for later packaging work. Issue #329 remains responsible for validating produced installer and archive artifacts, offline smoke behavior, upgrade, repair, uninstall, and artifact contents.
- Historical specifications may retain accurate references to the former mitmdump architecture. The live Doctor surface and current product guidance may not present it as a supported prerequisite.
- Npcap remains a separate Capture prerequisite and is not residue owned by Deep Capture.
- No new third-party dependency is expected. Any proposed dependency requires separate license, MSRV, and capability review.
