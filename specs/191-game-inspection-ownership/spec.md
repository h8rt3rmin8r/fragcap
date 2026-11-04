# Feature Specification: Exact Game Inspection Ownership

**Slice**: S191
**Branch**: `codex/s191-game-ownership-inspection`
**Created**: 2026-11-04
**Status**: In progress
**Issues**: #552 and #558
**Architecture of record**: `docs/fragcap-specification.md` sections 10, 11, 20, 27, and 29

## User Scenarios

### User Story 1: Continue an owned game handoff (P1)

An operator starts a cold managed launch whose first client-image process starts an owned launcher and exits. Fragcap continues observing through that provisional handoff, binds a later exact client successor, and preserves the full authorized observation interval. If no successor appears within the acquisition budget, the result identifies the incomplete handoff and retains cleanup and recovery facts separately.

**Independent check**: A controlled Windows process chain reproduces the initial short-lived client, launcher, bootstrap, and final same-image client, with countercases for wrong path, escaped ancestry, stale PID generation, ambiguity, and no successor.

### User Story 2: Decide game inspection before protocol handling (P2)

An authorized future game-only session can determine whether a proxy connection or association belongs to the exact selected game before any client-facing TLS termination, key logging, or content capture. Known middleware and unresolved ownership receive an explicit uninspected decision with a reason. The decision remains fixed for that connection even if later evidence changes.

**Independent check**: Controlled identities for a game and middleware using the same destination yield different decisions. Late, missing, conflicting, and stale evidence never produces a game decision, and the native hook runs before TLS leaf creation.

## Edge Cases

- A first client-image instance that has no live owned descendant is a true terminal exit, not an indefinite handoff.
- Packets observed during a provisional handoff remain retained with their actual role and time; they do not prove final-client observation or calibration success.
- A later same-image process under an escaped, wrong-path, wrong-account, wrong-session, or reused-PID chain does not inherit game ownership.
- A socket created after connection acceptance cannot own that connection.
- Socket-table delay, missing ETW events, queue saturation, and process watcher loss produce bounded uncertainty.
- A pending decision cannot hold the accept loop or retain unbounded client bytes.
- Middleware remains in process observation and route-owner cleanup even when excluded from inspection.

## Functional Requirements

- **FR-001 (#552)**: A provisional client exit with live causally owned descendants MUST preserve bounded acquisition and MUST NOT authorize end-of-observation shutdown. Only a verified successor may resume terminal ownership and begin the full observation interval.
- **FR-002 (#552)**: The session MUST report the actual first match, provisional exit, successor or incomplete decision, packet window, proxy window, and cleanup/recovery outcome without promoting pre-client traffic into a positive game fact.
- **FR-003 (#552)**: Controlled production-path Windows coverage MUST exercise the multi-process handoff and wrong-path, escaped, stale-generation, ambiguous, and no-successor cases, including the absence of cleanup requests while the owned launch continues.
- **FR-004 (#558)**: A typed requested and resolved inspection scope MUST distinguish exact-game and explicit launch-chain modes. An `auto` request MUST resolve before session effects. The current public CLI behavior remains launch-chain until the later operator integration issue.
- **FR-005 (#558)**: A native proxy decision interface MUST consume a bounded live owner decision without importing CLI or platform-specific attribution implementation into the proxy or `fragcap-core`.
- **FR-006 (#558)**: An exact-game decision MUST require the client-side socket, socket creation no later than connection acceptance, process generation, canonical executable identity, account/session, launch receipt, causal ancestry, and a selected-game role or proven successor. Destination, image basename, session capability, and descendant status alone are insufficient.
- **FR-007 (#558)**: A scope decision MUST precede client-facing TLS leaf creation, TLS termination, application-content retention, and key-log callbacks on exposed transport paths. Connection and association decisions MUST be immutable after the bounded decision deadline.
- **FR-008 (#558)**: Known middleware and unresolved owners MUST receive typed uninspected outcomes with stable reasons. Lookup latency, pending work, byte retention, and evidence history MUST be finite; overflow and unavailable telemetry MUST be counted.
- **FR-009 (#558)**: Decision evidence MUST record requested and effective scope, owner proof or uncertainty, timing, policy identity, and loss without credentials, key material, payloads, or private target identity in public diagnostics.
- **FR-010**: Process observation, route-owner inventory, exact shutdown authority, existing session authentication, and destination policy MUST remain intact. The blind relay engine, calibration migration, and full CLI operator journey remain in #559 through #561.

## Key Entities

- **Inspection scope**: Immutable session request and effective mode, separate from routing strategy and destination policy.
- **Owner proof**: A connection or association's client-side socket and exact process-instance lineage, or a named uncertainty.
- **Inspection decision**: Immutable per-connection or association result with action, reason, timing, and policy identity.

## Success Criteria

- **SC-001**: The controlled multi-stage launch retains observation until a proven final client or finite incomplete outcome, and no provisional exit causes managed shutdown.
- **SC-002**: Every countercase refuses unproven game ownership with a named reason and leaves the route-owner inventory intact.
- **SC-003**: The game and middleware on one destination select different actions using process identity, while missing or late evidence remains uninspected.
- **SC-004**: Hook-order checks demonstrate that no game-only connection reaches leaf creation, TLS accept, key logging, or content retention before a verified game decision.
- **SC-005**: All applicable repository checks and CI complete successfully. Installed-product behavior remains unverified until after release, and later observed failures become new issues.

## Clarifications (2026-11-04)

- The current CLI's explicit Deep Capture behavior remains launch-chain. Game-only scope is a library contract in this slice, with public selection and blind relay delivery assigned to #559 and #561.
- A pending owner lookup has one finite deadline. After that deadline it freezes as uninspected uncertainty; a late result may improve later reporting but cannot authorize inspection of already forwarded traffic.
- Selected-game proof follows the terminal role and its verified successor history. Arbitrary descendants, launcher helpers, and bootstrap services remain middleware unless a future reviewed target contract explicitly declares a game-owned helper role.
- Controlled repository regressions are performed during implementation. Private installed-product observation is performed after release and is not an issue-completion condition.

## Assumptions and Scope

- Existing process watcher, socket table, managed launch identity, and session plan are the authority to reuse. No new target process rights are introduced.
- #559 supplies actual blind byte forwarding. This slice supplies the typed decision boundary and native pre-inspection hook without claiming the full game-only operator journey.
- #556 storage placement and #542 launcher TLS causality are separate issues.
