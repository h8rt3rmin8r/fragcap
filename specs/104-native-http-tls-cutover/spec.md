# Feature Specification: Native HTTP/TLS Production Cutover

**Feature Branch**: `codex/104-native-http-tls-cutover`

**Created**: 2026-08-30

**Status**: Complete

**Input**: User description: "Kick off S104. Run it with spec-kit end to end."

**Tracker Scope**: Issues #290, #292, and #293

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run Deep Capture without an external proxy (Priority: P1)

An authorized operator starts an ordinary Deep Capture or compatibility calibration session and the application owns the complete local inspection path without requiring Python, mitmdump, or another proxy executable.

**Why this priority**: Removing the external runtime is the remaining foundation gate, but the cutover is acceptable only when the native path can carry the HTTP and HTTPS traffic the shipped mode already handles.

**Independent Test**: On a machine with no Python or mitmdump installation, run the controlled Deep Capture target through the public session API and command line, observe HTTP and HTTPS transactions, then finish with no proxy task or trust residue.

**Acceptance Scenarios**:

1. **Given** an approved Deep Capture plan and a compatible target, **When** the session starts, **Then** the native loopback proxy becomes ready, receives only session-authorized traffic, and the selected target receives its scoped proxy configuration.
2. **Given** Python and mitmdump are absent, **When** an ordinary or calibration session completes, **Then** its lifecycle, application observations, bundle, facts, and cleanup report are produced without an external proxy process.
3. **Given** startup, observation, shutdown, or cleanup failure injection, **When** the session terminates, **Then** the terminal report names the failed stage, preserves partial evidence, and reports every retained obligation.

---

### User Story 2 - Forward complete HTTP/1.1 sessions (Priority: P1)

An operator inspecting a proxy-compatible target can observe and forward ordinary HTTP/1.1 requests, persistent connections, tunnels, and upgrades without changing the target's meaning or silently losing traffic.

**Why this priority**: HTTP/1.1 and CONNECT are the base protocol and the entry point for HTTPS. A native default that cannot carry them would regress the shipped Deep Capture mode.

**Independent Test**: Drive a controlled client and origin through the native listener using standard methods, persistent requests, chunked messages, trailers, informational responses, CONNECT, half-close, and upgrade boundaries; reconcile forwarded outcomes with typed observations and loss accounting.

**Acceptance Scenarios**:

1. **Given** a valid absolute-form HTTP request, **When** the authorized client sends it through the proxy, **Then** the origin receives the equivalent request, the client receives the complete response, and typed request and response evidence is retained.
2. **Given** a valid CONNECT request, **When** the authorized client opens a tunnel, **Then** both directions remain usable until bounded close or cancellation and tunnel lifecycle evidence identifies the result.
3. **Given** malformed, ambiguous, oversized, unauthenticated, or disallowed input, **When** it reaches the listener, **Then** it is refused before prohibited upstream work and the refusal is counted and reported.

---

### User Story 3 - Inspect HTTPS with session-owned trust (Priority: P1)

An operator who explicitly approved the session certificate trust can inspect HTTPS routed through the native proxy while upstream identity validation remains mandatory.

**Why this priority**: Modern game service traffic is predominantly encrypted. Native cutover without client-facing TLS termination and verified upstream TLS would falsely claim parity.

**Independent Test**: Trust one generated session authority in an isolated store, route controlled TLS 1.2 and TLS 1.3 clients through the native proxy to independently validated origins, exercise SNI, protocol negotiation, resumption, alerts, and orderly shutdown, and verify exact trust cleanup.

**Acceptance Scenarios**:

1. **Given** explicit trust approval and a valid upstream identity, **When** a target establishes HTTPS through CONNECT, **Then** the proxy presents a session-issued identity, validates the upstream name and chain, forwards HTTP, and records the two TLS boundaries distinctly.
2. **Given** an invalid upstream chain or hostname, **When** the proxy attempts TLS, **Then** it fails closed and never reports decrypted application success.
3. **Given** an unsupported or certificate-pinned client, **When** the client-facing handshake fails, **Then** the session reports the supported refusal boundary or an honest unknown result without bypassing pinning.

---

### User Story 4 - Preserve a thin command boundary (Priority: P2)

A library consumer can run the same native session lifecycle as the command line, while the command line remains limited to argument mapping, confirmation, presentation, and exit status.

**Why this priority**: The public session API is the product boundary. Keeping proxy policy and lifecycle out of the command module prevents a second, untestable implementation.

**Independent Test**: Run equivalent controlled sessions through the public library and CLI entry points and compare their ordered lifecycle outcomes, with a repository gate proving no external proxy command or CLI-owned proxy engine remains.

**Acceptance Scenarios**:

1. **Given** equivalent approved plans, **When** library and command-line callers execute them, **Then** both use the same native adapter and produce equivalent lifecycle and failure contracts.
2. **Given** repository source and release inputs, **When** the policy gate scans them, **Then** no production Python, mitmdump, external proxy spawn, or external CA discovery path is present.

### Edge Cases

- A loopback client without the session credential, with a stale credential, or with a credential from another concurrent session is refused before DNS, connection, certificate, or retained-payload work.
- A destination that resolves to the listener itself, a disallowed local address, multiple mixed-policy addresses, or a changed address between attempts never widens the destination policy.
- HTTP messages with conflicting framing, oversized headers, slow partial headers, early EOF, invalid upgrade boundaries, or trailers after cancellation terminate finitely and remain accounted.
- A client or upstream peer may half-close, omit TLS close notification, send an alert, cancel a persistent request, or stall during shutdown; every owned task still has a finite terminal outcome.
- A client-facing certificate request for an invalid host, IP literal, expired cache entry, or rotated session authority cannot reuse an unrelated leaf.
- Failure after listener acquisition, authority generation, trust insertion, launch, or partial observation preserves accurate cleanup ownership and never labels the bundle complete.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Ordinary Deep Capture and both compatibility calibration phases MUST select the native backend by default and MUST NOT require an external proxy program.
- **FR-002**: The production and release source set MUST contain no Python, mitmdump discovery, external proxy spawning, readiness polling, event parsing, log ownership, shutdown, or external certificate-authority discovery path.
- **FR-003**: The public library MUST own native proxy construction, session authorization material, lifecycle integration, trust identity sharing, observation conversion, and cleanup; the command layer MUST remain limited to arguments, confirmation, presentation, and exit mapping.
- **FR-004**: Every accepted client connection MUST prove possession of a fresh, session-specific capability through standard proxy authentication before upstream, certificate, or retained application work is allocated.
- **FR-005**: Session proxy credentials MUST be scoped to the selected launch, excluded from plans and artifacts, redacted from diagnostics, compared without timing leakage, and released with the session.
- **FR-006**: The native listener MUST support HTTP/1.1 forward-proxy requests and CONNECT, including standard methods, all request-target forms where valid, persistent connections, chunked framing, trailers, informational responses, upgrades, cancellation, and half-close.
- **FR-007**: The native path MUST reject malformed or ambiguous HTTP framing, enforce finite header, body, idle, connection, and task limits, and count every refusal, truncation, parse failure, and dropped observation.
- **FR-008**: Forwarding MUST preserve the semantic request and response observed at each boundary; any required proxy-to-origin transformation MUST be represented in evidence rather than silently changing the recorded observation.
- **FR-009**: For explicitly approved HTTPS inspection, the native path MUST terminate client TLS with a leaf issued by the exact session authority and establish a separately verified upstream TLS connection for the requested identity.
- **FR-010**: Upstream TLS chain and hostname verification MUST fail closed and MUST NOT expose a permissive verification option.
- **FR-011**: Client and upstream TLS MUST support TLS 1.2 and 1.3, SNI, protocol negotiation, session resumption where peers permit it, alerts, bounded handshake, and orderly or explicitly incomplete shutdown.
- **FR-012**: TLS failures MUST distinguish the observable boundary, including client handshake, certificate issuance, upstream connection, upstream validation, negotiation, alert, timeout, and unknown client refusal; pinning MUST never be bypassed or claimed solely from silence.
- **FR-013**: Typed native observations MUST identify session, connection, direction, lifecycle stage, HTTP method/target/status where observed, TLS boundary and negotiated facts, refusal or failure reason, and exact loss-accounting state without claiming later milestone metadata or body completeness.
- **FR-014**: Native proxy stop and cleanup MUST be idempotent and bounded, release the listener and all connection tasks, zeroize or release private session material, and report any residue.
- **FR-015**: Lifecycle and failure-injection verification MUST exercise the production native adapter through both the public library boundary and the controlled command-line path without a driver, game account, Internet service, real user trust mutation, Python, or mitmdump.
- **FR-016**: A repository gate MUST reject reintroduction of production external proxy commands, Python proxy assets, mitmdump names, or CLI-owned proxy business logic while permitting clearly isolated historical specification text.
- **FR-017**: Existing bundle, event, fact, and terminal-report contracts MUST remain compatible or receive an explicit version change; incomplete native observations MUST never be promoted to complete HTTP/TLS evidence.

### Key Entities

- **Session proxy capability**: An opaque credential bound to one listener generation and selected launch, never a durable artifact.
- **Native proxy lease**: The single owner of listener, connection tasks, protocol engines, observations, session authority reference, and bounded shutdown.
- **HTTP exchange**: A connection-scoped request and response outcome with the minimum evidence needed for current application and compatibility contracts.
- **TLS boundary observation**: Facts and failure state for either the client-facing or verified upstream handshake, kept distinct.
- **Session identity set**: The one authority and bounded leaf identities owned by a session and shared exactly with its trust lease.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All controlled ordinary Deep Capture and calibration scenarios complete on a host where Python and mitmdump are absent.
- **SC-002**: The HTTP/1.1 matrix covers every scenario named in FR-006, with 100% reconciliation between terminal connection outcomes and typed accounting.
- **SC-003**: Controlled TLS 1.2 and TLS 1.3 HTTPS sessions succeed for valid identities and fail for every invalid-chain and invalid-hostname case without a permissive result.
- **SC-004**: Ten consecutive start, traffic, stop, and cleanup cycles leave zero bound listener ports, live proxy tasks, trusted test identities, or unreported cleanup obligations.
- **SC-005**: Every unauthenticated, cross-session, malformed, ambiguous, oversized, disallowed-destination, timeout, and injected-failure case has one stable refusal or failure outcome and no unaccounted upstream attempt.
- **SC-006**: The full repository merge gate passes with no production Python or mitmdump dependency and with the external-proxy regression gate enabled.

## Assumptions

- This slice deliberately combines #290 with #292 and #293. Cutting over under #290 alone would regress the shipped HTTP and HTTPS behavior and violate the instrument-truth requirement.
- HTTP/2, WebSocket frames, full HTTP metadata, streaming body retention and decoding, SSE, gRPC, native key logs, versioned application JSONL, complete HAR, cross-artifact correlation, client certificates, and milestone-wide conformance remain assigned to #294 through #305, #335, and #336.
- Generic TCP, SOCKS, UDP, QUIC, launcher expansion, routing expansion, and final completion gates remain later native Deep Capture milestones.
- The controlled protocol lab and secure foundation from S103 are the test authority for local behavior. Real launcher compatibility remains observed target data rather than a universal promise.
- No system-wide proxy fallback, certificate-pinning bypass, target-process access, target key extraction, or Internet-dependent automated test is permitted.
