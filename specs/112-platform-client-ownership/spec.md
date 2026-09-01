# Feature Specification: Cold Platform-Client Ownership

**Feature Branch**: `codex/112-platform-client-ownership`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "S112: own cold platform-client launch and proxy propagation under issue #308, using the established spec-kit autopilot approach."

## Overview

S112 closes issue #308 by making one proven-cold platform client part of the selected Deep Capture session before the title is dispatched through it. The operator selects an existing stored target. fragcap prepares the exact local platform executable and title dispatch from that target and current local platform facts, refuses warm or uncertain state before any effect, starts the platform under the session-scoped route, observes its identity, and only then dispatches the title. The resulting process, socket, routing, and propagation evidence remains explicit and bounded.

Steam is the first supported platform. The ownership contract is platform-neutral so another platform can implement the same preparation and dispatch boundary without adding a second target storage or resolution path.

## Clarifications

### Session 2026-09-01

- Q: When may title dispatch occur? -> A: Only after the exact cold platform process has been observed as session-owned.
- Q: How is an already-running same-named platform client classified? -> A: It is a conservative warm refusal before effects because the permitted startup snapshot cannot prove executable path identity.
- Q: What proves environment propagation separately from routing? -> A: A final-client proxy observation under the owned platform ancestry proves propagation; routing and propagation remain separate evidence fields.
- Q: What happens when the platform exits before the terminal client binds? -> A: The session reports a named incomplete-platform outcome and fails within its launch deadline.
- Q: What happens when a matching client appears outside the owned platform ancestry? -> A: It is reported as an escaped descendant and never acquires terminal ownership.

## User Scenarios & Testing

### User Story 1 - Own a cold platform launch (Priority: P1)

An authorized operator starts Deep Capture for a stored Steam title while Steam is not running. fragcap validates the exact installed platform client, starts it with session-scoped routing, observes that exact process as session-owned, and dispatches the title only after ownership is established.

**Why this priority**: Without an owned cold platform process, proxy inheritance through platform dispatch is a compatibility assumption and Deep Capture cannot make a truthful support claim.

**Independent Test**: A controlled adapter timeline with an absent platform startup snapshot prepares one immutable platform plan, records the exact root start, withholds title dispatch until that start is observed, and completes only after a declared client descendant binds.

**Acceptance Scenarios**:

1. **Given** a stored Steam target, a canonical installed Steam executable, current same-case routing evidence, and no running Steam image, **when** the operator authorizes Deep Capture with managed launch, **then** fragcap starts that exact platform executable with the session route and dispatches the selected application only after the platform process is observed.
2. **Given** the owned platform starts and the exact declared client appears within its creation-time ancestry, **when** the client owns the final proxied connection, **then** the client becomes the terminal session owner and capture continues until its terminal lifecycle ends.
3. **Given** platform preparation, startup, observation, or dispatch fails, **when** the launch deadline is reached or the failure is observed, **then** the run ends with a named failure and performs ordinary bounded cleanup.

---

### User Story 2 - Refuse warm and escaped paths (Priority: P2)

An operator receives a precise refusal or incomplete outcome when the platform is already running, cannot be identified exactly, exits before handoff, or produces a matching client outside the owned ancestry.

**Why this priority**: A warm or escaped process cannot inherit the selected child-only route, and labeling it owned would make both attribution and inspection claims false.

**Independent Test**: Offline process snapshots and event timelines cover warm platform presence, platform exit before client binding, same-image unrelated processes, escaped matching clients, ambiguity, and deadline expiry without invoking Steam or a game.

**Acceptance Scenarios**:

1. **Given** any `steam.exe` image exists in the startup snapshot, **when** the operator requests the cold Steam path, **then** fragcap refuses before bundle, proxy, trust, routing, launch, or compatibility mutation.
2. **Given** the owned platform exits before the declared client binds, **when** the session reconciles the event, **then** it reports platform exit and an incomplete-platform outcome without promoting another process.
3. **Given** a matching client image starts outside the owned platform ancestry, **when** the event is observed, **then** fragcap reports an escaped descendant and does not grant terminal ownership.

---

### User Story 3 - Preserve separate compatibility evidence (Priority: P3)

An operator and later tooling can distinguish that traffic reached the selected client from evidence that the platform actually propagated the session route to that client.

**Why this priority**: Routing reachability and environment propagation answer different questions. Combining them would make later calibration and support decisions overstate what was observed.

**Independent Test**: Controlled observations independently vary platform ownership, client ancestry, proxy connection attribution, and traffic reachability, producing separate routing and propagation outcomes with no inferred success from silence.

**Acceptance Scenarios**:

1. **Given** launcher or platform traffic reaches the proxy but no final-client connection does, **when** evidence is finalized, **then** routing does not claim `reached-client` and propagation is not confirmed.
2. **Given** the exact final client connects through the proxy as a descendant of the owned platform, **when** evidence is finalized, **then** routing records client reachability and propagation separately records confirmation.
3. **Given** relevant events are lost, ambiguous, absent, or outside the observation window, **when** evidence is finalized, **then** both dimensions retain truthful inconclusive or not-confirmed outcomes with exact loss accounting.

### Edge Cases

- The platform executable is missing, non-canonical, outside the discovered platform root, or replaced between preparation and execution.
- Steam is absent, its registry root is unreadable, the selected application is not installed, or the stored application identifier is malformed.
- The platform process starts with a different identifier than the created root, starts helpers before the root event is delivered, or exits while dispatch is pending.
- The protocol dispatch command succeeds but no client appears, or reports failure after the platform has started.
- Multiple matching platform or client processes appear, a client reuses the platform image name, or the expected client appears outside the owned ancestry.
- The process watcher reports loss or disconnects while platform ownership or propagation is being established.
- The operator interrupts during platform startup, dispatch, acquisition, capture, or cleanup.
- A non-Steam platform is selected before an adapter exists.

## Requirements

### Functional Requirements

- **FR-001**: fragcap MUST prepare one immutable platform launch plan before any Deep Capture effect, containing the selected platform kind, exact canonical platform executable, working directory, argument vector, selected application dispatch, declared process stages, and effective launch deadline.
- **FR-002**: Platform preparation MUST derive from the existing stored target, existing target resolution path, and current local platform installation facts; it MUST NOT add platform-specific target storage or precedence.
- **FR-003**: The cold platform path MUST refuse before effects when any same-named platform image is already running, platform identity is uncertain, the selected application is unavailable, the exact executable cannot be validated, or no supported platform adapter exists.
- **FR-004**: fragcap MUST start only the exact prepared platform executable with the session-scoped environment and MUST NOT use a shell, system-wide proxy setting, process injection, hook, target process handle, or executable modification.
- **FR-005**: Title dispatch MUST occur only after the exact created platform process has been observed and bound to the session's platform role.
- **FR-006**: The platform adapter contract MUST separate side-effect-free preparation, cold root start, and title dispatch so future platforms can implement the same lifecycle without changing target resolution or the Deep Capture coordinator.
- **FR-007**: Process reconciliation MUST use exact executable identity, creation-time ancestry, and one-owner stage binding for the platform, declared launch intermediates, and terminal client.
- **FR-008**: A matching process outside the owned platform ancestry MUST be recorded as escaped and MUST NOT acquire a declared role or terminal ownership.
- **FR-009**: Platform exit before terminal client acquisition, competing stage matches, missing client, dispatch failure, watcher loss, and launch deadline expiry MUST remain distinct named outcomes.
- **FR-010**: Platform startup, title dispatch, terminal acquisition, proxy shutdown, and cleanup MUST remain finite under displayed deadlines, including when the platform starts but the title never does.
- **FR-011**: Routing reachability and platform-to-client proxy propagation MUST remain separate observations and compatibility facts. Neither may be inferred from the other or from silence.
- **FR-012**: Propagation MAY be confirmed only when a proxy connection attributed to the exact terminal client is reconciled beneath the owned platform ancestry during the authorized session.
- **FR-013**: Platform, launcher, helper, and terminal-client process and socket ownership observations MUST retain their actual roles without folding platform traffic into game-client evidence.
- **FR-014**: Every dropped or unlocalizable platform lifecycle, routing, or propagation observation MUST advance a named bounded loss count surfaced in the session evidence.
- **FR-015**: Platform launch and compatibility evidence MUST be written through the existing session artifacts and local compatibility store without exposing platform credentials, account identifiers, user library ownership, or real game data in committed fixtures.
- **FR-016**: Capture mode's existing Steam protocol launch behavior and ordinary non-platform profiles MUST remain backward compatible.
- **FR-017**: Steam MUST be the first supported platform adapter; unsupported platforms MUST produce a stable pre-effect refusal.
- **FR-018**: The master specification, outline, slice ordering, agent guide, and changelog fragments MUST record S112 without claiming warm restart, generic transport support, or Deep Capture completion.

### Key Entities

- **Platform Launch Plan**: Immutable pre-effect authority for one exact cold platform executable, selected application dispatch, environment scope, declared roles, and deadline.
- **Platform Adapter**: Typed boundary that prepares platform facts, starts the exact cold root, and dispatches a selected title after ownership is observed.
- **Platform Launch Receipt**: Created root process identity and dispatch state used to reconcile observed ownership without opening a target process handle.
- **Platform Ownership State**: Ordered lifecycle from prepared, root started, root observed, title dispatched, terminal acquired, to a named terminal failure or completion.
- **Platform Evidence**: Separate routing, propagation, process ownership, socket ownership, loss, and omission facts retained in existing artifacts.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every controlled cold-platform scenario dispatches the title zero times before platform ownership and exactly once after ownership.
- **SC-002**: Every warm, uncertain, missing, unsupported, or invalid platform scenario refuses before the first Deep Capture effect.
- **SC-003**: One hundred percent of escaped or competing controlled process timelines end without granting false terminal ownership.
- **SC-004**: Every platform-started but client-missing scenario reaches a named terminal outcome within the displayed launch deadline.
- **SC-005**: Controlled evidence permutations independently produce all specified routing and propagation combinations without treating silence as success.
- **SC-006**: The complete repository CI parity suite passes with no new runtime dependency and no regression to existing Capture or direct and publisher Deep Capture paths.
- **SC-007**: Committed tests and fixtures contain zero real platform credentials, account identifiers, owned-library inventories, or game capture data.

## Assumptions

- S109 supplies immutable child-environment routing and lifecycle cleanup authority.
- S108 supplies packet-flow and proxy-connection correlation, and S111 supplies exact multi-stage ownership semantics.
- Steam's installed root can be discovered through the existing registry-backed Steam integration, and its exact client executable is `steam.exe` beneath that root.
- The permitted startup snapshot identifies warm platform state by image name only; uncertainty therefore refuses rather than attempting a target process path query.
- Warm-to-cold shutdown and restart UX remains issue #309 and is outside S112.
- Generic SOCKS, TCP, UDP, QUIC, and IPv6 transport expansion remains issues #310 through #315.
- General compatibility calibration persistence remains issue #317; S112 records only the evidence necessary for this owned platform path.
