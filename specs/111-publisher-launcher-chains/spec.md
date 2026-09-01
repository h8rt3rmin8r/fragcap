# Feature Specification: Managed Publisher-Launcher Chains

**Feature Branch**: `codex/111-publisher-launcher-chains`

**Created**: 2026-09-01

**Status**: Draft

**Input**: Implement GitHub issue #307 as S111 by preparing and managing exact publisher-launcher process chains under target-scoped Deep Capture routing.

## User Scenarios & Testing

### User Story 1 - Launch an exact cold publisher chain (Priority: P1)

An authorized operator selects a stored target whose exact publisher launcher, intermediate stages, and socket-owning client are declared. fragcap prepares the same capture and launch inputs used by ordinary Capture, starts observation before the chain, applies routing only to the session-owned root launch, and follows the declared descendants until the socket-owning client is identified.

**Why this priority**: Publisher-launched targets cannot use Deep Capture until fragcap can own the full cold chain without falling back to global routing or guessing which descendant is the client.

**Independent Test**: A controlled three-stage target can be prepared and launched from a cold state, with every stage matched in creation order and the final socket owner shown to inherit the scoped route.

**Acceptance Scenarios**:

1. **Given** an exact stored publisher chain and no matching stage already running, **When** the operator starts Deep Capture, **Then** observation begins before the launcher, the root receives only the declared session routing, and the declared terminal client becomes the correlated socket owner.
2. **Given** an exact stored publisher chain, **When** a library consumer prepares Capture and Deep Capture for it, **Then** both modes receive the same immutable chain identity and managed-launch request before either mode acquires resources.
3. **Given** a chain containing declared intermediate stages, **When** those stages start and exit before the client, **Then** their lifecycle remains part of the session evidence and does not end the session before the terminal client exits.

---

### User Story 2 - Refuse or report uncertain chains truthfully (Priority: P2)

An operator receives a side-effect-free refusal or an explicit inconclusive outcome when fragcap cannot prove that the publisher chain is cold, exact, and contained within the observed ancestry tree.

**Why this priority**: Treating an existing launcher, an escaped process, or an ambiguous descendant as session-owned could route unrelated traffic or produce a confident but false compatibility result.

**Independent Test**: Controlled warm, same-named, escaped-tree, missing-stage, and ambiguous-stage cases each produce their own stable outcome before trust, proxy, launch, or compatibility mutation where preflight evidence permits.

**Acceptance Scenarios**:

1. **Given** the publisher launcher is already running, **When** preparation evaluates the target, **Then** it returns `publisher-launcher-warm` and performs no session effect.
2. **Given** the publisher launcher is running but the game client is absent, **When** preparation evaluates the target, **Then** it returns `publisher-launcher-game-start-clean-warm` rather than treating the game absence as a cold chain.
3. **Given** an observed candidate client is outside the session root's creation-time ancestry, **When** chain reconciliation runs, **Then** the outcome is `escaped-tree` and no client identity or routing success is invented.
4. **Given** multiple descendants satisfy one declared stage or the declared stage sequence cannot be proven, **When** chain reconciliation runs, **Then** the outcome is `ambiguous` and every competing observation remains available for diagnosis.

---

### User Story 3 - Preserve auditable chain evidence and cleanup (Priority: P3)

An operator can determine which stages were prepared, observed, skipped, ambiguous, or escaped, and can see that only session-owned resources were cleaned up.

**Why this priority**: A managed chain is safe only when routing authority, process identity, lifecycle, loss, and cleanup remain visible and reconcilable after the session.

**Independent Test**: A controlled chain session produces a bounded chronological record whose prepared stages reconcile with observations and whose cleanup obligations cover every acquired session resource.

**Acceptance Scenarios**:

1. **Given** a completed controlled chain, **When** its evidence is reconciled, **Then** every declared stage has one exact observed, absent, ambiguous, or escaped disposition.
2. **Given** observation capacity is exhausted, **When** further stage events arrive, **Then** exact overflow totals are retained and surfaced rather than silently dropping evidence.
3. **Given** any failure after resource acquisition, **When** cleanup or recovery runs, **Then** only journaled session-owned resources are released and the final outcome names any residue.

### Edge Cases

- A launcher exits before its intermediate or client descendants start.
- A launcher spawns multiple same-named descendants, only one of which owns target sockets.
- An intermediate process reparents, delegates through a service, or starts a client outside the observed root tree.
- A process identifier is reused after a declared stage exits.
- A stored path or executable identity changes between preparation and launch.
- A chain starts correctly but no declared terminal client appears before the launch deadline.
- The terminal client appears but owns no correlated socket before the observation deadline.
- A launch fails after routing is prepared but before the root process starts.
- A stage event or lifecycle writer reaches its configured capacity.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST prepare an immutable publisher-chain plan from the exact stored target entry before any proxy, trust, capture, routing, or launch effect begins.
- **FR-002**: The prepared plan MUST identify one root publisher launcher, zero or more ordered intermediate stages, and one terminal client using stored executable and ancestry constraints.
- **FR-003**: Preparation MUST reject missing, duplicate, unreachable, circular, or ambiguous stage declarations with all discovered diagnostics.
- **FR-004**: Capture and Deep Capture MUST consume the same prepared chain identity and managed-launch request.
- **FR-005**: The public managed-launch interface MUST accept the prepared chain without requiring CLI-owned reconstruction.
- **FR-006**: A supported launch MUST start the declared root through an argument-safe process API without a shell and apply only the authorized target-scoped routing inherited by its descendants.
- **FR-007**: Process observation MUST be active before the root launch and MUST use creation-time ancestry rather than later parent identifier lookup.
- **FR-008**: Process identity MUST be evaluated from observed image identity, creation instant, parent relationship, and declared stage constraints without target process memory access.
- **FR-009**: The system MUST distinguish clean cold, launcher-warm, game-start-clean-warm, escaped-tree, ambiguous, missing-stage, launch-failed, and deadline-expired outcomes.
- **FR-010**: Only a proven clean cold chain MAY proceed as a supported publisher launch in S111.
- **FR-011**: Launcher-warm and game-start-clean-warm cases MUST refuse before session effects and remain inputs to the later warm-to-cold workflow owned by issue #309.
- **FR-012**: An escaped or ambiguous candidate MUST NOT be promoted to terminal client, routing success, socket ownership, or compatibility success.
- **FR-013**: Every competing observation in an ambiguous outcome MUST remain available within declared bounds.
- **FR-014**: Intermediate stage exit MUST NOT end the session while a declared terminal stage remains viable.
- **FR-015**: The terminal client's exit MUST drive the existing managed-session stop semantics after it has been bound exactly.
- **FR-016**: Stage and socket evidence MUST preserve process identifier reuse safety by pairing identifiers with creation instants.
- **FR-017**: Every observed stage MUST reconcile to exactly one declared stage or to an explicit unmatched, ambiguous, or escaped disposition.
- **FR-018**: Stage observation storage MUST be finite, and every discarded or unlocalized observation MUST advance a named surfaced counter.
- **FR-019**: Every external effect MUST retain the existing journal-before-effect obligation and exact recovery decision contract.
- **FR-020**: Cleanup MUST act only on resources proven to be owned by the current session and MUST report unresolved residue.
- **FR-021**: Controlled multi-stage fixtures MUST cover every modeled chain outcome without real publisher credentials, game accounts, or operator-identifying data.
- **FR-022**: Security tests MUST prove that same-named unrelated processes, escaped descendants, and inherited operator proxy variables cannot widen the selected target scope.
- **FR-023**: The implementation MUST NOT use a shell, code injection, function hooks, target memory access, executable modification, target TLS key extraction, Winsock catalog changes, or hidden system-wide proxy configuration.
- **FR-024**: User-visible status, artifacts, compatibility facts, and documentation MUST describe observed publisher-chain outcomes without claiming support for warm, escaped, ambiguous, or unobserved paths.
- **FR-025**: S111 MUST close issue #307 only and MUST leave platform-client ownership, warm-to-cold control, transport expansion, calibration expansion, and final Deep Capture completion to their existing issues.

### Key Entities

- **Publisher Chain Plan**: The immutable prepared identity of the root launcher, ordered intermediate stages, terminal client, routing authority, deadlines, and target anchor.
- **Declared Chain Stage**: One expected process role with exact executable identity, ancestry constraint, sequence position, and terminal status.
- **Observed Chain Process**: One creation-time process observation paired with its parent observation, stage candidates, lifecycle state, and socket evidence.
- **Chain Reconciliation**: The bounded mapping between declared stages and observed processes, including competing candidates and unmatched or escaped observations.
- **Publisher Chain Outcome**: The stable result that states whether the chain was clean cold, warm, supported, escaped, ambiguous, incomplete, failed, or timed out.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every controlled clean-cold chain reaches exactly one declared terminal socket owner with all declared stages reconciled and zero scope widening.
- **SC-002**: Every warm, escaped, ambiguous, missing-stage, failed-launch, and deadline case produces its required distinct outcome in 100 percent of controlled repetitions.
- **SC-003**: Capture and Deep Capture preparation produce byte-for-byte equivalent publisher-chain identity for the same stored target.
- **SC-004**: All chain event, overflow, and cleanup counts reconcile exactly in controlled success, failure, and capacity-exhaustion runs.
- **SC-005**: No controlled case opens a target process with memory rights, invokes a shell, mutates an executable, changes a system-wide proxy, or accepts an unrelated same-named process.
- **SC-006**: A controlled three-stage cold chain starts observation before the root and binds the terminal client within the configured launch deadline in every successful run.
- **SC-007**: The full repository verification suite passes without a new runtime dependency or lockfile package unless planning proves an unavoidable, policy-compliant need.
- **SC-008**: Issue #307 closes with all other Native Deep Capture milestone 3 issues remaining open.

## Assumptions

- S109's immutable routing plan, journal-before-effect lifecycle, cleanup recovery, and target-scoped child environment are the foundation for this slice.
- Existing stored target and profile stage identities can represent the required launcher, intermediate, and terminal roles without a second target storage shape.
- Existing process start and exit observation supplies creation-time ancestry and image identity without opening target process handles.
- The safe S111 support boundary is a fully cold, session-owned publisher chain. Restarting a warm launcher remains issue #309.
- Controlled synthetic launchers are sufficient to prove the state machine and security boundaries without a live game or publisher account.
- Generic platform-client ownership remains issue #308, and new proxy transports remain issues #310 through #318.
