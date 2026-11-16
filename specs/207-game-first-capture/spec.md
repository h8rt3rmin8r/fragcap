# Feature Specification: Game-first Deep Capture

**Feature Branch**: `codex/s207-game-first-capture`

**Created**: 2026-11-15

**Status**: Implemented; hosted CI and owner review pending

**Input**: Owner-approved S207 autopilot for #611, #608, #612, #614, and #609 under epic #607. One ordinary authorized capture should target the final game, leave intermediary updater traffic on its native route, measure compatibility during that capture, and state what actually happened.

## Scope

**In**: One authoritative pre-effect route and applied-effects contract; final-game default across route, packet, application, and bundle scope; explicit advanced full-chain routing including a SOCKS-only launcher option; truthful delayed or unknown SOCKS provenance; removal of the dedicated calibration command, guided/resume flow, embedded calibration entry, and prior-positive-fact admission gate; historical fact readback; directly affected help, specification, and controlled tests.

**Out**: Warm-platform dispatch and fallback (#610), full dimensional outcome presentation (#615), chronological route-history UI (#616), direct-game TLS under every advanced inspection policy (#613), generic TLS relay lifetime (#617), packet correlation and buffer-loss repairs (#594, #596, #618 through #620), the final public-documentation inventory (#622), installed-game qualification, and a release.

**Done when**: Each included issue's controlled acceptance contract passes, normal and structured output and native artifacts agree with applied scope and route, the repository's full required gate and applicable controlled Windows route matrix pass, and the PR is ready for owner merge. No private title run is a completion prerequisite.

## Clarifications

### Session 2026-11-15

- **Scope means actual effects**: The owner's final-game default covers launcher environment, packet admission, application retention, and bundle description. Causal process observation remains available for exact ownership and cleanup without admitting launcher payload.
- **One capture measures compatibility**: Historical compatibility rows remain readable and diagnostically useful, but no row authorizes a changed route or blocks a new otherwise valid run. Explicit current-session authorization still names all effects.
- **Cold launch is a bounded first slice**: Warm-platform adoption and fallback belong to #610. This slice does not describe the cold requirement as the intended final user experience.
- **Advanced launcher routing is deliberate**: SOCKS-only exposure is a separately selected full-chain effect, never a silent retry after an ordinary route failure.

## User Scenarios & Testing

### User Story 1 - Capture the selected game on its first attempt (Priority: P1)

An operator selects Deep Capture for a stored game with no prior compatibility measurement. The operator sees one complete plan, authorizes it once, launches through the supported associated platform chain, and receives the final game's traffic and an honest result from that same run. Publisher and platform updater traffic takes its ordinary route and is absent from game-only retained evidence.

**Independent Test**: A controlled cold platform-to-publisher-to-game chain starts with no compatibility rows. Its updater communicates normally, the exact game exchanges inspected traffic, and retained packets and application records contain no intermediary payload.

**Acceptance Scenarios**:

1. Given no prior facts and a supported cold managed game, when the operator authorizes Deep Capture, then the game attempt proceeds without a separate calibration and records the observed route and protocol result.
2. Given zero, one, or several launch intermediates, when the final game is proven, then the default admits only that exact game to the selected route and retained packet and application evidence.
3. Given a failed, stale, or mismatched historical compatibility row, when a new capture is otherwise ready, then the row does not block the run or authorize a different effect.
4. Given an ambiguous, escaped, or unavailable game owner, when scope cannot be proven, then the attempt reports the exact limitation without admitting launcher or unrelated payload as game traffic.

### User Story 2 - Choose advanced launcher inspection deliberately (Priority: P2)

An operator deliberately selects full launch-chain capture and reviews the additional launcher routing effects before authorization. A SOCKS-only launcher route can carry compatible updater traffic without exposing an HTTP proxy URL; a launcher that rejects an HTTP CONNECT receives an exact pre-TLS diagnostic.

**Independent Test**: A controlled publisher-like updater performs network checks through the selected SOCKS route while an exact final game uses the direct route. A separate HTTP CONNECT client abort is classified before TLS, not as a certificate or origin failure.

**Acceptance Scenarios**:

1. Given explicit advanced scope and SOCKS launcher routing, when a compatible updater communicates, then the authorized environment, observed proxy route, byte transfer, and artifact scope agree.
2. Given a client that closes after a successful HTTP CONNECT response, when no TLS bytes follow, then the result names the client-side tunnel handoff and does not claim a TLS trust or origin failure.
3. Given a default game-only plan, when a launcher sends updater traffic, then it uses no Fragcap HTTP or SOCKS proxy route and the game-only bundle retains no updater content.

### User Story 3 - Trust the plan and the recorded protocol facts (Priority: P2)

An operator can inspect the complete route before effects and trust that the applied environment and direct route match it. The resulting compatibility record distinguishes unknown, plaintext, encrypted TLS, and decrypted traffic using actual evidence, even when the first bytes arrive late.

**Independent Test**: A deliberately mismatched route fails before any session effect. Delayed TLS, partial prefixes, genuine cleartext, empty streams, and read failures each produce the expected route and protocol classification.

**Acceptance Scenarios**:

1. Given a route choice that changes between authorization and preparation, when the session would start, then it refuses before launching or changing proxy, trust, or driver state.
2. Given a delayed or incomplete application prefix, when classification runs, then the recorded state remains unknown until enough evidence supports a more specific claim.
3. Given an ordinary capture that cannot inspect a connection, when its facts are saved, then route success and protocol inspectability remain separate and no plaintext or decrypted claim is invented.

### Edge Cases

- A route is authorized before its loopback listener receives a concrete port.
- A launcher inherits both upper- and lower-case proxy variables, including an old value from its parent environment.
- The final game already has a direct route while a publisher helper uses an environment route; a connection must not be intercepted twice.
- An unrelated process shares the game image name, destination, or process identifier at a different generation.
- The first TLS bytes arrive after the initial classification wait, are partial, or never arrive.
- A historical fact belongs to a prior target version, scope, route, or product version.
- A session ends with partial packet or process correlation but successful cleanup.

## Requirements

### Functional Requirements

- **FR-001 (#611)**: The exact route kind, authorized effect set, target identity, scope, and cleanup obligations MUST be fixed before authorization and MUST match the effects prepared for the session. Only a declared listener port may be bound afterward; any other drift MUST refuse before effects.
- **FR-002 (#608)**: Ordinary Deep Capture MUST default to the proven final game's route, packet retention, and application evidence regardless of the number of launch intermediates. Process ancestry MAY be observed for proof and cleanup without retaining intermediary payload.
- **FR-003 (#608, #612)**: The ordinary managed launcher ancestry MUST NOT receive Fragcap proxy URLs when exact final-game direct routing is selected. Intermediary updater traffic MUST remain on its native route, while exact eligible final-game sockets retain direct-route capability.
- **FR-004 (#608, #612)**: Full-chain capture and SOCKS-only launcher routing MUST be explicit pre-effect choices with separately named scope, environment effects, and evidence. They MUST NOT be selected silently after a failed ordinary attempt.
- **FR-005 (#612)**: The product MUST distinguish a successful HTTP CONNECT followed by client close before TLS from a proxy refusal, certificate failure, origin response failure, or idle timeout when the observed stages support that distinction.
- **FR-006 (#614)**: An absent, delayed, partial, or failed SOCKS application prefix MUST be classified as unknown until specific evidence proves plaintext, encrypted TLS, or decrypted content. Later evidence MUST preserve its provenance without rewriting an earlier observation as though inspection occurred.
- **FR-007 (#609)**: An ordinary capture MUST proceed without a prior positive compatibility row and MUST record current-session route, TLS, protocol, inspectability, loss, and cleanup facts when observed. Historical rows MUST remain readable but MUST NOT gate admission or authorize a changed effect.
- **FR-008 (#609)**: The separate calibration command, guided/resume workflow, and embedded Deep Capture calibration entry MUST be retired from current user-facing commands and guidance. Existing historical stores and bundles MUST remain readable.
- **FR-009**: The authorization, human and structured terminal result, route and proxy journals, packet and application evidence, compatibility facts, and manifest MUST agree on the selected scope and observed outcome without presenting partial capture as complete.
- **FR-010**: Capture MUST remain passive. Deep Capture MUST retain explicit authorization, exact game-generation proof, finite routing and loss accounting, unrelated-traffic continuity, and bounded cleanup; no target memory access or game-file change is introduced.

### Key Entities

- **Authorized route selection**: Target, scope, launcher exposure, direct-game route, exact effects, and cleanup obligations fixed before session effects.
- **Applied route**: The concrete listener endpoint and environment/direct-route effects actually installed for one authorized session.
- **Game ownership proof**: Exact final-client process generation and causal ancestry used for route and retained-evidence admission.
- **Protocol observation**: Observed prefix, time, classification confidence, forwarding decision, and retained provenance.
- **Compatibility observation**: Current-session facts with case identity and evidence lineage, retained separately from authorization.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required controlled first-run case with no prior compatibility facts reaches the selected session attempt through one authorization and records its observed terminal result.
- **SC-002**: In each controlled zero-, one-, and multi-intermediary chain, 100% of retained application and payload-bearing packet records belong to the proven final game by default, with zero intermediary or unrelated payload retained.
- **SC-003**: Every deliberate plan/effect mismatch refuses before the first proxy, trust, driver, or launch effect, while valid late port binding preserves the authorized effect set.
- **SC-004**: Controlled publisher repository checks in ordinary game-only mode create zero connections to Fragcap's launcher proxy route; explicit SOCKS advanced checks show authenticated, nonzero two-way transfer when the fixture succeeds.
- **SC-005**: Every delayed TLS, partial prefix, genuine cleartext, zero-byte, EOF, and read-failure fixture produces the expected provenance and compatible terminal/structured fact with zero false plaintext or decryption claims.
- **SC-006**: All required repository and controlled Windows route checks pass with exact unrelated-sentinel continuity and no unaccounted observed packet loss; no physical private-game outcome is inferred from those gates.

## Assumptions

- The existing exact-game direct route and native proxy remain the only approved active routing mechanisms. A launch chain may be observed to prove ownership even when its network content is excluded.
- The currently supported managed cold launch remains the first-slice launch case. Warm platform dispatch, wrong-association fallback, and already-running game adoption belong to #610.
- Existing stored compatibility facts and historic bundle schemas remain readable; their prior-positive meaning is retired for new admission.
- An explicit advanced full-chain operator may choose a SOCKS-only launcher environment. HTTP proxy behavior remains diagnosable but is not an ordinary default.
- A release or installed private-title exercise is separate from this repository-controlled implementation gate.
