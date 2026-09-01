# Feature Specification: Authenticated SOCKS5 TCP Routing

**Feature Branch**: `codex/114-authenticated-socks5-tcp`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "S114: implement authenticated SOCKS5 TCP routing under issue #310."

## Overview

S114 closes issue #310 by adding an authenticated SOCKS5 TCP path to the existing native Deep Capture listener. A selected target can use the session-scoped SOCKS route when HTTP proxy variables are insufficient. The listener admits only the current session capability, accepts CONNECT for IPv4, IPv6, and domain destinations, applies the existing upstream destination policy, and forwards both directions under finite buffers, timeouts, cancellation, and terminal accounting.

The proxy owns domain resolution for domain-form requests and reports that ownership explicitly. Accepted tunnels are classified without consuming or changing their bytes. HTTP and TLS candidates enter the existing native protocol boundaries where supported; otherwise the tunnel remains byte-transparent and metadata-only. Generic TCP payload evidence and non-HTTP TLS semantics remain issue #312, UDP ASSOCIATE remains issue #311, and Deep Capture remains incomplete until issue #334.

## Clarifications

### Session 2026-09-01

- Q: Does SOCKS5 use a separate listener? -> A: No. One loopback listener recognizes SOCKS5 by its first octet and retains one lifecycle and connection authority.
- Q: Which authentication method is accepted? -> A: RFC 1929 username/password only, with the fixed `fragcap` username and current session capability as password.
- Q: Who resolves a domain-form destination? -> A: The proxy resolves it locally, then applies policy independently to every returned address before attempting a connection.
- Q: What does protocol classification authorize? -> A: It routes supported HTTP and TLS candidates into existing native engines and otherwise forwards an opaque metadata-only TCP tunnel without inventing semantics.
- Q: How is the route presented to managed targets? -> A: `HTTP_PROXY` and `HTTPS_PROXY` retain the HTTP URL, while `ALL_PROXY` receives a session-authenticated `socks5h` URL.

## User Scenarios & Testing

### User Story 1 - Route A TCP Connection Through SOCKS5 (Priority: P1)

An authorized operator launches a selected target with the session SOCKS route. The target authenticates, requests one TCP destination, receives an exact SOCKS reply, and exchanges bytes through a bounded tunnel.

**Why this priority**: This is the missing path for targets that honor SOCKS routing but not HTTP proxy variables.

**Independent Test**: A controlled client authenticates against a real loopback listener and completes IPv4, IPv6, and domain CONNECT exchanges with exact echoed bytes and clean half-close behavior.

**Acceptance Scenarios**:

1. **Given** the current session username and capability password, **when** a client negotiates username/password authentication and requests an allowed IPv4, IPv6, or domain destination, **then** the proxy returns success only after the upstream connection exists and forwards both directions.
2. **Given** a domain destination, **when** the proxy resolves it, **then** every candidate address is checked by the existing destination policy and DNS ownership is reported as proxy-owned.
3. **Given** either side half-closes, **when** the other direction still has data, **then** that direction continues until its own EOF, error, cancellation, or timeout.

---

### User Story 2 - Refuse Unauthorized Or Invalid Clients (Priority: P2)

An unrelated local process, malformed peer, or unsupported command receives a finite refusal before any unauthorized upstream or payload work occurs.

**Why this priority**: A loopback bind is not an authorization boundary. The session capability is the boundary.

**Independent Test**: Controlled clients cover no acceptable method, wrong username, wrong password, missing fields, oversized or truncated messages, unsupported commands, policy refusals, and deadlines, proving zero unauthorized upstream connections.

**Acceptance Scenarios**:

1. **Given** no username/password method or an invalid credential, **when** negotiation completes, **then** the proxy refuses before destination resolution or connection.
2. **Given** UDP ASSOCIATE, BIND, an unknown command, address type, or malformed request, **when** it is parsed, **then** the proxy returns the exact supported refusal when a reply is possible and records the terminal reason.
3. **Given** a destination rejected by policy or an upstream failure, **when** CONNECT fails, **then** the reply and evidence distinguish policy, DNS, network, timeout, and cancellation without claiming a tunnel.

---

### User Story 3 - Correlate And Classify Accepted Tunnels (Priority: P3)

An operator can reconcile each accepted SOCKS connection with application, proxy lifecycle, packet-flow, and process evidence, including an explicit protocol classification and all loss.

**Why this priority**: A working tunnel without auditable ownership or truthful evidence would weaken Deep Capture's core promise.

**Independent Test**: A controlled matrix sends HTTP, TLS, and opaque TCP prefixes through SOCKS and proves stable connection identity, classification, route inheritance, bounded observation, and complete terminal accounting.

**Acceptance Scenarios**:

1. **Given** an accepted tunnel, **when** its first application bytes are available, **then** classification reports HTTP, TLS, or opaque TCP without consuming, altering, or delaying bytes beyond the finite classification budget.
2. **Given** a SOCKS connection and packet/process evidence, **when** artifacts finalize, **then** the shared connection identifier and exact open/close window support the existing correlation outcomes.
3. **Given** queue pressure, timeout, cancellation, malformed input, or forced shutdown, **when** the session ends, **then** every accepted connection has one terminal outcome and every dropped or unavailable observation is counted.

### Edge Cases

- A client offers multiple methods but omits username/password.
- Username or password fields are empty, maximal length, truncated, or contain arbitrary octets.
- A domain contains invalid UTF-8, a trailing dot, an invalid label, or resolves to mixed allowed and refused addresses.
- An IPv4-mapped IPv6 destination aliases the listener or a refused local destination.
- The upstream connects but the success reply cannot be written.
- A client pipelines application bytes immediately after CONNECT.
- HTTP, TLS, and opaque prefixes arrive one byte at a time or remain incomplete until the classification deadline.
- Either side half-closes while the opposite direction is backpressured.
- Session cancellation races negotiation, DNS, connect, reply, classification, or forwarding.
- Connection and event limits are saturated.

## Requirements

### Functional Requirements

- **FR-001**: The native loopback listener MUST recognize SOCKS5 without adding a second untracked listener or changing HTTP behavior.
- **FR-002**: SOCKS5 negotiation MUST support version 5 and MUST select only username/password authentication. No-authentication and every other method MUST be refused.
- **FR-003**: Username/password authentication MUST require the fixed session username and exact current session capability, compare secret material in constant time, and zero temporary secret buffers where practical.
- **FR-004**: Authentication refusal MUST occur before destination resolution, upstream connection, protocol classification, or payload forwarding.
- **FR-005**: The parser MUST be incremental and bounded by the existing header deadline and finite protocol field sizes. Truncated, malformed, unsupported-version, and unsupported-address input MUST terminate explicitly.
- **FR-006**: CONNECT MUST support IPv4, IPv6, and domain address forms and all valid nonzero TCP ports.
- **FR-007**: BIND, UDP ASSOCIATE, and unknown commands MUST remain unsupported and MUST NOT create network effects. UDP ASSOCIATE remains issue #311.
- **FR-008**: Domain-form requests MUST use proxy-owned resolution. Every resolved address MUST pass the existing normalized destination policy independently before a connection attempt.
- **FR-009**: The implementation MUST map success, general failure, policy refusal, network unreachable, host unreachable, connection refused, TTL expiry, command unsupported, and address unsupported to truthful SOCKS5 replies when the protocol permits a reply.
- **FR-010**: A success reply MUST be sent only after the upstream TCP connection exists and MUST contain the actual local bound address of that connection.
- **FR-011**: Forwarding MUST preserve byte order and content in both directions, use fixed finite buffers, propagate half-close, honor read/write and idle budgets, and terminate under session cancellation.
- **FR-012**: Accepted tunnels MUST be classified as HTTP, TLS, or opaque TCP using a bounded non-consuming prefix observation. Incomplete or unknown prefixes MUST remain opaque rather than guessed.
- **FR-013**: Supported HTTP and TLS candidates MAY enter the existing native protocol engines only when doing so preserves their established authentication, destination, TLS, evidence, and boundedness contracts. Otherwise forwarding MUST remain byte-transparent and metadata-only.
- **FR-014**: Runtime and application evidence MUST identify SOCKS5 negotiation, authentication, requested address form, DNS ownership, CONNECT result, classification, byte counts, terminal state, and exact connection window without recording the capability.
- **FR-015**: Every authentication refusal, parse refusal, policy refusal, DNS failure, connect failure, timeout, cancellation, saturation, observation drop, and forced termination MUST advance a named counter or terminal outcome.
- **FR-016**: The session route MUST retain HTTP URLs for `HTTP_PROXY` and `HTTPS_PROXY`, publish an authenticated `socks5h` URL for `ALL_PROXY`, keep `NO_PROXY` empty for the managed child, and expose no secret in debug or human output.
- **FR-017**: Existing route authorization, resource journaling, cleanup, application artifact, lifecycle stream, manifest, and packet/process correlation authorities MUST remain the owners of their respective facts.
- **FR-018**: The controlled protocol lab MUST cover IPv4, IPv6 where available, domain, authentication, CONNECT, refusal, timeout, malformed input, pipelining, half-close, cancellation, backpressure, classification, correlation, and terminal conservation without Internet, elevation, a game, or real target data.
- **FR-019**: S114 MUST add no system-wide proxy mutation, unauthenticated listener path, target process access, DNS bypass, UDP forwarding, generic TCP payload claim, non-HTTP TLS semantic claim, new runtime dependency, or Deep Capture completion claim.
- **FR-020**: The architecture of record, glossary, status plan, and changelog MUST record S114 as closing issue #310 and leave issues #311 through #318 and #334 open.

### Key Entities

- **SOCKS Greeting**: Bounded version and advertised authentication method set received before all other work.
- **SOCKS Authentication**: RFC 1929 username/password exchange bound to one session capability.
- **SOCKS CONNECT Request**: Command, address form, destination authority, and port requested after authentication.
- **SOCKS Reply**: Exact protocol result and bound address returned after request evaluation.
- **SOCKS Tunnel**: One accepted client connection, one allowed upstream TCP stream, two bounded forwarding directions, byte accounting, and one terminal outcome.
- **Tunnel Classification**: HTTP, TLS, or opaque TCP conclusion derived from a bounded non-consuming prefix.
- **SOCKS Route**: Secret-bearing `socks5h` child environment value derived from the current proxy endpoint and capability.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of controlled IPv4, IPv6-available, and domain CONNECT cases either exchange exact bytes or return the expected protocol refusal within their finite deadline.
- **SC-002**: One hundred percent of unauthenticated, incorrectly authenticated, malformed, and unsupported-command cases create zero upstream connections.
- **SC-003**: One hundred percent of domain candidates are recorded as proxy-resolved and policy-evaluated before connection.
- **SC-004**: Every accepted tunnel reaches exactly one complete, refused, timed-out, cancelled, protocol-error, transport-error, or forced terminal outcome, with accepted connection conservation intact.
- **SC-005**: Controlled half-close and backpressure cases preserve all bytes in the still-open direction while memory remains within configured connection and buffer bounds.
- **SC-006**: HTTP, TLS, and opaque test prefixes are classified deterministically without any byte change or loss.
- **SC-007**: Managed child routing exposes the correct distinct HTTP and SOCKS URLs, and no structured event, artifact, debug representation, or human message contains the capability.
- **SC-008**: The full repository verification suite passes without a new dependency or a regression in native HTTP, HTTPS, HTTP/2, WebSocket, SSE, gRPC, cleanup, or correlation behavior.

## Assumptions

- S103 supplies the session capability and upstream policy, S104 supplies the shared native listener and HTTP/TLS path, S105 through S108 supply bounded application evidence and correlation, and S109 supplies route and lifecycle authority.
- The RFC 1929 username/password method is used as a session capability carrier, not as a reusable user credential system.
- `socks5h` is the truthful route scheme because domain-form names must reach the proxy for proxy-owned DNS.
- The existing listener endpoint is shared by HTTP and SOCKS5, so no additional endpoint allocation or cleanup obligation is needed.
- Issue #312 owns generic TCP payload evidence and non-HTTP TLS semantics. S114 retains opaque TCP as metadata-only.
