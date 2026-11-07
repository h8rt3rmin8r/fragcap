# Feature Specification: Launcher TLS Refusal Diagnosis

**Feature Branch**: `codex/s198-launcher-tls-refusals`

**Created**: 2026-11-06

**Status**: Implemented, local CI green; awaiting exact-head hosted CI and reviews

**Input**: Owner-approved S198 implementation of GitHub issue #574 after merged S197, under the repository autopilot protocol through a reviewed pull request.

## Scope

**Scope in**: Client-facing CONNECT TLS refusal classification from evidence the native proxy actually observes; controlled launcher and updater style connections under explicit launch-chain inspection; preservation of accepted connection ID, exact process-instance correlation, tunnel and TLS stage, terminal transport category, optional peer alert, and HTTP timeout phase through existing bounded artifacts and reports.

**Scope out**: Direct-socket routing (#572), target instrumentation, certificate-pinning bypass, automatic blind-relay fallback after an inspect decision, trust-store changes, private-title compatibility claims, a new release, or claiming that a generic EOF proves why a client stopped.

**Done when**: The controlled fixture demonstrates a compatible explicit launch-chain launcher and updater path plus distinct EOF, malformed TLS, trust alert, pinning evidence, client-auth, and protocol incompatibility classifications where the input proves them; every terminal retains its exact accepted connection ID, while the independent correlation authority reports a proven owner or named gap when packet and process evidence exists; post-exchange HTTP idle timeout remains distinct; no failed inspection becomes a successful or opaque relay; Spec-Kit analysis, local gates, exact-head CI, and external review closeout are complete; and the official PR awaits owner final review and merge.

## User Scenarios & Testing

### User Story 1 - Diagnose a refused launcher TLS tunnel (Priority: P1)

An operator selects explicit launch-chain inspection for a cold managed title. When a publisher or updater reaches the authenticated native proxy but its client-facing TLS handshake fails, the bundle reports the exact accepted connection, tunnel response, TLS boundary, terminal class, observed transport outcome, and any peer alert. The result remains a refusal and does not claim inspected content or switch to blind relay.

**Why this priority**: Installed v0.10.14 recorded 19 client-facing failures after CONNECT success, but the common terminal code was `tls-refusal-unknown` despite separate EOF and invalid-data transport evidence.

**Independent Test**: Drive authenticated CONNECT through the real loopback proxy with a controlled client that closes before completing TLS and another that sends malformed TLS, then inspect terminal diagnostics and application records by accepted connection ID.

**Acceptance Scenarios**:

1. **Given** an authenticated CONNECT with a returned 200 response, **when** the client stream ends during TLS without a rustls alert, **then** the terminal identifies a client-facing TLS transport end, retains the observed EOF or reset category, leaves the alert absent, and records no full inspection.
2. **Given** an authenticated CONNECT, **when** the client sends a syntactically invalid TLS message, **then** the terminal identifies invalid TLS message evidence and its transport category without calling it pinning, trust rejection, or successful inspection.
3. **Given** a structured TLS alert or incompatibility, **when** the handshake fails, **then** the terminal names only the class supported by that alert or rustls incompatibility; bare I/O remains separate.
4. **Given** an established `Inspect` decision, **when** client TLS fails, **then** the connection stays failed and cannot be reclassified as a blind relay or a positive compatibility fact.

### User Story 2 - Prove a compatible launcher and updater path (Priority: P1)

A controlled launch-chain fixture creates separate launcher-like and updater-like proxy connections that authenticate, CONNECT, complete client-facing and verified upstream TLS, and exchange HTTP. The independent correlation regression uses their connection IDs with distinct non-client owners and an unavailable comparator; the proxy result itself does not prove process-instance identity.

**Why this priority**: A successful proxy exchange is only a compatibility claim for the exact routed connection. The installed 0.10.14 launcher failures and direct final-client sockets do not prove that the final client is inspectable.

**Independent Test**: Run two controlled loopback TLS connections and separately exercise synthetic packet-flow ownership through the existing correlation authority using the same accepted connection IDs; inspect existing artifact integration coverage for exact and unavailable owner outcomes.

**Acceptance Scenarios**:

1. **Given** two controlled launcher-like connections that use the session capability, **when** each completes a CONNECT and HTTP exchange through client-facing and separately verified upstream TLS, **then** each connection has complete TLS and HTTP evidence with a distinct accepted connection ID, while independent packet and process proof decides any owner join.
2. **Given** a connection with no exact packet-flow owner, **when** correlation closes, **then** its owner remains unavailable or ambiguous and no launcher or final-client success is inferred.
3. **Given** launcher-only successful connections, **when** calibration considers final-client reachability, **then** they cannot qualify a final-client positive routing or inspection fact.

### User Story 3 - Keep idle HTTP timeouts separate (Priority: P2)

An operator reviewing a failed connection can tell whether a header timeout happened before the first request or after a completed exchange. This distinction survives the bounded diagnostic projection and is not counted as a TLS handshake refusal.

**Why this priority**: The same installed session had ten after-exchange HTTP header timeouts that are a separate timing class from the 19 TLS failures.

**Independent Test**: Complete one HTTP exchange on a controlled keep-alive tunnel, wait for the next request deadline, and compare the connection diagnostic with a first-request timeout and a client-TLS refusal.

**Acceptance Scenarios**:

1. **Given** a completed HTTP exchange, **when** the next header does not arrive before its deadline, **then** the terminal reports `http-header-timeout`, `after-completed-exchange`, and a positive completed-exchange count.
2. **Given** no completed exchange, **when** the first header times out, **then** the terminal reports `before-first-request` and zero completed exchanges.
3. **Given** a TLS handshake refusal, **when** its diagnostic is projected, **then** it carries no HTTP header-timeout phase and no invented completed exchange.

## Requirements

- **FR-001**: The native TLS classifier MUST distinguish a transport-ended handshake, a rustls-invalid client message, a structured trust or certificate alert, a supported pinning signal, a client-certificate signal, a protocol incompatibility, and an unclassified failure without parsing private error prose.
- **FR-002**: EOF, reset, and aborted transport outcomes MUST remain explicit as transport categories; they MUST NOT by themselves establish pinning, certificate trust failure, or which actor intentionally closed the socket.
- **FR-003**: Client-facing and upstream TLS boundaries MUST remain separate, with the accepted connection ID and reached/completed stages preserved through failure and completion.
- **FR-004**: Existing bounded diagnostic and correlation records MUST expose exact process-instance ownership only when independently proven, and MUST preserve unavailable or ambiguous outcomes.
- **FR-005**: The controlled fixture MUST demonstrate explicit launch-chain inspection for at least a launcher-like and an updater-like connection, including full HTTP evidence after both TLS legs complete, without implying final-client coverage.
- **FR-006**: A TLS refusal after an `Inspect` decision MUST remain a refusal; the proxy MUST NOT silently retry the connection as uninspected relay.
- **FR-007**: HTTP header timeouts MUST retain before-first-request versus after-completed-exchange phase and completed-exchange count independently from TLS failure classification.
- **FR-008**: Diagnostic output MUST remain bounded and omit raw authority, packet/body content, proxy credentials, certificate private material, and raw client TLS flights.
- **FR-009**: Public documentation and the master specification MUST state the observable classifications, the generic-EOF limit, and the separate #572 direct-socket boundary.
- **FR-010**: When bounded diagnostic retention cannot keep a connection detail, the existing dropped-detail counter and unavailable correlation outcome MUST remain authoritative; the system MUST NOT fabricate a replacement owner or terminal.
- **FR-011**: Existing session capability authentication, destination admission, finite connection and handshake deadlines, independently verified upstream TLS, and operator-visible trust effects MUST remain enforced on both successful and failed paths.

## Key Entities

- **TLS refusal class**: Stable code derived from a structured rustls error or a transport kind at one named TLS boundary.
- **Connection diagnostic**: Existing bounded accepted-connection record with stages, terminal, cause, boundary, alert, transport category, OS code, and completed HTTP work.
- **Correlation result**: Separate post-session join from a connection ID and endpoint/time evidence to an exact process generation, or an explicit gap.
- **Controlled launch-chain connection**: Synthetic authenticated proxy client with a declared launcher or updater role, a local TLS origin, and no real game or publisher data.

## Assumptions and Clarifications

- The field bundle described in #574 remains private and is not available to this source slice. A controlled success proves the native path can work, while a future installed-product run is required to identify any title-specific refusal.
- A rustls `InvalidMessage` is evidence of malformed TLS input, while `PeerIncompatible` is evidence of incompatible TLS parameters. A bare `InvalidData` I/O category without a structured rustls cause remains unclassified.
- Exact owner identity comes from the existing packet-flow correlation contract, never from the proxy peer tuple or an executable name.
- The current connection-diagnostic schema and application correlation contract can carry the new stable refusal codes without a storage migration.

## Success Criteria

- **SC-001**: Controlled EOF, malformed TLS, trust alert, pinning signal, client-auth signal, protocol incompatibility, and unknown cases yield the exact expected stable code and diagnostic category, with no unsupported causal inference.
- **SC-002**: Two independent controlled launcher/updater connections complete authenticated CONNECT, client and upstream TLS, and HTTP, with distinct connection IDs and complete evidence.
- **SC-003**: Existing exact owner, unavailable, and ambiguous artifact joins remain distinct; the paired controlled IDs exercise separate non-client and unavailable correlation without inventing a final-client owner.
- **SC-004**: Controlled before-first-request and after-completed-exchange header timeouts remain separate from TLS refusals, with exact completed-exchange counts.
- **SC-005**: Local CI parity and all exact-head hosted checks pass, all actionable reviews are resolved, and the PR remains unmerged for owner final review.
