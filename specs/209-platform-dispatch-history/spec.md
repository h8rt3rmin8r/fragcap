# Feature Specification: Platform Dispatch and Advisory Capture History

**Feature Branch**: `codex/s209-platform-history`
**Created**: 2026-11-16
**Status**: Ready for Review
**Input**: Owner-approved S209 autopilot for #610 and #616 under #607. An ordinary game capture follows its associated platform without a user cold-start ritual, and each attempt retains truthful, bounded advisory route and launch history.

## Scope

**In**: Associated-platform dispatch for cold and warm Steam, direct dispatch for an unassociated target, exact final-client generation proof, wrong or missing association outcomes, bounded authorized fallback without duplicate launch, and chronological local capture history with route, process, transfer, inspection, failure, loss, and cleanup provenance.

**Out**: New platform discovery or an invented platform association, live memory instrumentation, control of warm or unrelated processes, advanced launcher TLS repair (#621), generic TLS lifetime (#617), direct-helper latency (#619), packet-correlation and buffer-loss repair (#594, #596, #618, #620), final public surface audit (#622), private-game qualification, and release publication.

**Done when**: #610 and #616 controlled scenarios pass, authorized and applied launch choices agree, each capture attempt has an honest retained history outcome, migration preserves older rows, the full repository gate passes, and the PR is ready for owner merge.

## Clarifications

### Session 2026-11-16

- **Warm platform ownership**: A warm platform is never a session-created route owner and is never stopped by Fragcap. Its game also lacks session-created shutdown authority. The plan discloses that the game remains running after capture and that existing routed connections may close during finite route teardown. Exact final-game process generation and route admission still require current evidence. A warm dispatch that cannot establish that proof reports a specific refusal.
- **Association authority**: A target's stored platform association chooses the first dispatch. Missing association selects direct launch. An association failure can select a different declared candidate only when the same pre-effect authorization names that candidate and no game is running or already dispatched.
- **History authority**: Each row describes an observed attempt, not a reusable authorization or a compatibility guarantee. Comparable rows may surface drift, but no history row blocks or silently changes a new capture.
- **Fallback bound**: At most one alternate dispatch is attempted per authorization. A failed dispatch with uncertain game state never retries automatically.
- **Unavailable association**: If the associated platform cannot be resolved into an exact executable and one final client during preflight, the attempt refuses before authorization. A direct alternate is available only after a valid platform plan was authorized and its cold root fails before title dispatch. Other missing platform adapters remain explicit unsupported associations.

## User Scenarios & Testing

### User Story 1 - Start the game through its association (Priority: P1)

An operator selects a stored game and approves one Deep Capture plan. The product starts the game through its associated platform whether that platform is already open or not, then captures only the proven final client. If no platform association exists, it starts the declared direct client.

**Independent Test**: Controlled cold and warm platform cases and an unassociated direct case each produce one dispatch, exact final-client generation evidence, and no unrelated payload retention.

**Acceptance Scenarios**:

1. Given a Steam-associated target and cold Steam, when the authorized capture begins, then Steam receives one title dispatch and the exact final game is acquired.
2. Given a Steam-associated target and warm Steam, when the authorized capture begins, then the existing Steam process remains untouched, one title dispatch occurs, and only a newly proven final-game generation becomes route eligible.
3. Given no platform association, when the authorized capture begins, then the declared direct executable is used without a platform guess.
4. Given an already-running game, unrelated same-name process, ambiguous generation, or unproved publisher handoff, when the attempt starts, then it refuses or reports the exact incomplete outcome without duplicating a launch or broadening capture.

### User Story 2 - Recover from a wrong association within the approved plan (Priority: P2)

A target association can be stale. The operator's plan identifies any allowed alternate before effects. A failed first choice is retained, and the alternate runs only when the first attempt cannot have started a game.

**Independent Test**: Controlled missing or wrong platform association attempts show either one authorized alternate or one refusal, never two game launches.

**Acceptance Scenarios**:

1. Given an exact preauthorized direct alternate, when the cold platform root fails to start before title dispatch and no game is running, then exactly one direct attempt may follow.
2. Given uncertain dispatch delivery or an observed game generation, when the first choice fails, then no alternate starts and the result states why.
3. Given no alternate in the plan, when dispatch fails, then the run fails with the recorded first-choice outcome.

### User Story 3 - Read chronological advisory history (Priority: P2)

Every capture attempt contributes a bounded local record of what was selected and what happened. An operator can see comparable successes, failures, and route changes without running calibration or treating earlier results as permission for a new effect.

**Independent Test**: First, repeat, failed, updated-target, changed-scope, changed-route, and rollback cases append distinct ordered records; only comparable records produce drift.

**Acceptance Scenarios**:

1. Given no prior row, when an attempt ends, then its selected and observed launch, route, scope, versions, proof, transfer, inspection, loss, failure, and cleanup states are retained with explicit unknowns.
2. Given a later comparable attempt, when the route outcome changes, then history reports the change while preserving both rows and the current plan's independent authorization.
3. Given a target version, scope, or route change, when a new attempt ends, then the rows remain ordered but no false like-for-like drift is claimed.
4. Given bounded retention, when the limit is reached, then only the oldest eligible history rows are pruned and the deletion is accounted for.

### Edge Cases

- The platform exists but is inaccessible, exits during dispatch, or has more than one plausible process generation.
- The associated platform executable or game identity is unavailable at preflight, so no valid alternate can be authorized.
- A publisher handoff produces an exact client, an escaped client, or two plausible clients.
- The first dispatch returns success but no client appears before the deadline.
- An alternate is authorized but a game appears after the first dispatch.
- A target registration changes between plan review and execution.
- The store append or pruning step fails after artifacts are finalized.
- A prior history row has an older schema or lacks one comparison dimension.

## Requirements

### Functional Requirements

- **FR-001 (#610)**: Ordinary Deep Capture MUST select the stored associated platform when known and a declared direct client when no platform is associated. Warm or cold platform state MUST be internal, with no routine close-and-retry demand.
- **FR-002 (#610)**: The pre-effect plan MUST identify the chosen launch method, any single alternate, exact scope and route effects, and the distinct ownership and cleanup obligations of cold and warm processes, including the warm game's continued execution and possible routed-connection teardown at capture end.
- **FR-003 (#610)**: A dispatch MUST bind only an exact final-game process generation before selected routing or retained game evidence. A query-verified warm platform root may establish observation-only packet membership for post-snapshot descendants, while shutdown authority continues to require a session-created root. Warm, escaped, ambiguous, reused-ID, unrelated, and already-running processes MUST NOT be controlled as session-owned.
- **FR-004 (#610)**: A failed first choice MAY trigger at most one preauthorized alternate only when the prior choice is proved not to have started a game. Uncertain dispatch MUST refuse automatic fallback.
- **FR-005 (#616)**: Each capture attempt MUST append a local chronological record of selected and observed launch and route, target/build clues, scope, versions, process proof, transfer, inspection, failure, loss, and cleanup, with unknown and unavailable values kept distinct. Inspection history MUST use the reconciled classification summary and explicitly mark incomplete classification coverage.
- **FR-006 (#616)**: History MUST be bounded, ordered, target-scoped, private, and migratable without rewriting or inventing older facts. Pruning MUST be explicit and must not erase the current attempt.
- **FR-007 (#616)**: Drift MUST compare only rows with matching target identity, build clue when known, scope, route identity, and comparable evidence dimensions. It MUST remain advisory and MUST NOT gate admission or authorize changed effects.
- **FR-008**: Human and structured results, native artifacts, history, and the applied launch plan MUST agree about success, refusal, partial capture, and cleanup. A game process lifetime MUST NOT imply gameplay.
- **FR-009**: Capture remains passive. Deep Capture retains one explicit authorization, exact game process and destination proof, finite loss accounting, and no target memory access, game-file edits, warm-process control, or unrelated payload retention.

### Key Entities

- **Launch association**: The stored target's known platform and exact declared game or publisher stages.
- **Dispatch attempt**: The authorized method, effect boundary, delivery result, and observed final-game generation or refusal.
- **Capture history row**: A target-scoped, time-ordered summary of one attempt, its versions and comparison dimensions, and independently observed outcome fields.
- **Drift assessment**: An advisory comparison of two eligible rows with named dimensions and changed observations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All required cold Steam, warm Steam, unassociated direct, publisher handoff, already-running, and unrelated-process controlled cases produce their expected single dispatch or explicit refusal with zero duplicate launches.
- **SC-002**: Every selected direct route in the controlled matrix has exact current game-generation and destination proof; zero unrelated or launcher payload records are retained by default.
- **SC-003**: All controlled fallback cases take no more than one alternate and no fallback after uncertain delivery or a possible running game.
- **SC-004**: First, repeat, failure, update, scope change, route change, and rollback cases retain ordered truthful history, with zero stale-row authorization and zero false drift across incomparable cases.
- **SC-005**: All migration, privacy, bounded-retention, cleanup, and repository gates pass without relying on a later private game run.

## Assumptions

- Steam is the currently implemented concrete platform adapter. Other platforms are used only when a stored association and a concrete reviewed adapter exist; no platform is guessed from an executable name.
- Existing exact-game direct routing is the active route for ordinary Deep Capture. Platform and publisher network traffic remains native in the ordinary scope.
- Warm platform processes and any game they create are not session-created process-control objects; routing and artifact scope still require exact current game proof.
- Historical compatibility facts remain readable. The new ordered history is a separate outcome record and never becomes an admission gate.
