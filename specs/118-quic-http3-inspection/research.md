# Research: Scoped QUIC And HTTP/3 Inspection

## Decision 1: Promote Quinn And Add Hyperium HTTP/3

**Decision**: Promote exact-pinned Quinn 0.11.11 from development-only to runtime in `fragcap-proxy`, and add exact-pinned `h3` 0.0.8 plus `h3-quinn` 0.0.10 with default features disabled. Retain ring as the sole cryptographic provider.

**Rationale**: Quinn is already lock-resolved, exercised by the controlled protocol lab, Tokio-native, and built on the same rustls and ring stack as existing proxy TLS. Hyperium `h3` supplies client and server HTTP/3 semantics over Quinn and declares Rust 1.70, below the workspace Rust 1.88 floor. The adapter version explicitly supports Quinn 0.11. The dependencies are MIT licensed and compatible with Apache-2.0 distribution.

**Alternatives considered**: Hand-written QUIC and QPACK were rejected as security-critical protocol reimplementation. Quiche and MsQuic were rejected because they add native/C cryptographic and packaging surfaces outside the selected pure-Rust stack. Treating HTTP/3 as generic UDP was rejected because issue #314 explicitly requires native semantic observations.

**Primary references**: [Quinn documentation](https://docs.rs/quinn/0.11.11/quinn/), [h3 repository](https://github.com/hyperium/h3), [h3-quinn 0.0.10](https://docs.rs/h3-quinn/0.0.10/h3_quinn/)

## Decision 2: Intercept Only Through The Existing Authenticated UDP Route

**Decision**: A QUIC gateway is created only after an authenticated S115 association admits an exact destination through existing DNS and policy checks. The gateway owns a client-facing QUIC endpoint and a separately verified upstream endpoint for that immutable destination.

**Rationale**: Reusing the existing association preserves target and session tenancy, exact outer client identity, destination policy, finite peer ownership, lifecycle cancellation, and S117 packet-independent datagram truth. A new wildcard UDP listener would be a second unauthenticated route and violate P-1.

**Alternatives considered**: A system-wide UDP redirection layer was rejected as prohibited interception and global mutation. A free-standing public QUIC listener was rejected because it cannot prove target ownership. Passive QUIC parsing was rejected because encrypted application semantics would remain unavailable.

## Decision 3: Refuse 0-RTT And Disable Active Migration

**Decision**: Set no early-data allowance on either rustls QUIC configuration, never invoke Quinn 0-RTT connection APIs, disable server migration, and terminate when outer association identity or selected destination changes. Connection identifier rotation without path change remains ordinary transport behavior.

**Rationale**: QUIC 0-RTT application data is replayable, while the proxy cannot safely replay or independently authorize it across two TLS connections. Active migration can move traffic outside the route established for the selected target. Exact refusal is safer and meets issue #314 without a false coverage claim.

**Alternatives considered**: Buffering and replaying 0-RTT was rejected because it changes semantics and creates replay authority. Allowing migration based only on QUIC connection identifiers was rejected because an identifier proves transport continuity, not target or route authorization.

**Primary references**: [RFC 9000 sections 7.3 and 9](https://www.rfc-editor.org/rfc/rfc9000), [RFC 9114 section 10.9](https://www.rfc-editor.org/rfc/rfc9114)

## Decision 4: One Logical Pair, Separate Transport Identities

**Decision**: Allocate one proxy logical pair identity and retain distinct client-facing and upstream stable connection identities. Record endpoint and negotiated protocol facts separately. Do not present either transport connection identifier as if it were preserved end to end.

**Rationale**: A terminating proxy necessarily creates two QUIC connections with unrelated cryptographic and connection identifier spaces. The pair is the correlation authority; each half remains an observed transport fact.

**Alternatives considered**: Reusing client connection identifiers upstream was rejected because it misrepresents two security contexts and conflicts with endpoint-owned identifier generation.

## Decision 5: HTTP/3 Is ALPN-Selected, Everything Else Is Refused

**Decision**: Select HTTP/3 only for negotiated `h3` application protocol values and proxy requests through the `h3` client/server APIs. Refuse absent or unknown QUIC application protocols without transparent forwarding.

**Rationale**: ALPN is the TLS-authoritative protocol choice. Byte sniffing inside a stream cannot distinguish proprietary protocols reliably and would violate P-9. A terminating proxy also cannot pair arbitrary application stream semantics safely without protocol knowledge, so refusal is the only honest current boundary.

**Alternatives considered**: Payload heuristics were rejected as false classification. A generic stream bridge was rejected because it cannot distinguish application-owned streams from protocol control streams or preserve unknown application semantics.

## Decision 6: Use Existing Evidence And Lifecycle Authorities

**Decision**: Add typed QUIC connection, stream, datagram, refusal, and HTTP/3 events to application JSON Lines version 2, then extend existing trailer reconciliation, HAR projection, manifest declarations, correlation, lifecycle, and cleanup summaries.

**Rationale**: The event envelope is already additive and crash-readable. Reusing it preserves one evidence authority, bounded queue and storage loss accounting, and existing recovery behavior.

**Alternatives considered**: A separate QUIC artifact was rejected as a competing authority. Inferring packet-level QUIC facts into application records was rejected because packet capture remains the packet authority.
