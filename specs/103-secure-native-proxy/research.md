# S103 Research: Secure Native Proxy Foundation

## Decision 1: Complete seven dependent foundation issues together

S103 resolves #283 through #289 in their dependency order. Authentication, upstream policy, CA/trust, and raw events all consume the bounded runtime delivered by S102; the protocol lab consumes all of those contracts. Splitting them would repeat CI and review while leaving intermediate foundation states that #290 cannot use.

The production native cutover remains #290. S103 does not replace mitmdump or claim completed protocol handlers.

## Decision 2: Use protocol-native capability fields and constant-time comparison

One opaque 32-byte capability is generated per listener generation. HTTP proxy requests use the standard proxy authorization field; SOCKS uses username/password authentication. A future protocol without an equally scoped standard or adapter remains unsupported. The runtime authenticates before allocating upstream state or retaining application payload.

The capability is generated from the selected ring system random source and compared with a fixed-length XOR accumulation. It is never formatted through `Debug`, logged, persisted in plaintext, or reused after cleanup. A new listener generation receives a new capability even when it reuses the same port.

## Decision 3: Separate authority parsing, resolution, policy, and transport effects

`DestinationAuthority` validates one exact host and non-zero port without user information, wildcard DNS, zone identifiers, or ambiguous delimiters. An injected resolver returns ordered address candidates. `DestinationPolicy` evaluates every candidate, refuses the listener, loopback, private/link-local/unspecified/multicast addresses by default, and permits only exact controlled-lab grants. The connector rechecks each resolved address immediately before connect so rebinding cannot bypass the policy decision.

DNS, connect, read, write, and shutdown budgets remain separate. Typed errors name parse, DNS, policy, TCP, TLS, timeout, or cancellation stages. Secure upstream configuration loads operating-system roots, reports every rejected root, fails closed on an empty usable store, uses the explicit ring provider, and retains hostname verification with no permissive verifier.

## Decision 4: Generate one zeroized CA owner per session

Rcgen 0.14.10 already belongs to the selected graph. Enable its `zeroize` feature and hold `KeyPair` inside `Zeroizing`. Generate a P-256 ECDSA/SHA-256 CA with a test-distinct per-session subject, constrained CA basic constraints, signing key usage, bounded validity, and no PEM copies. SHA-1 over exact DER remains the Windows store locator only; SHA-256 is the provenance fingerprint. Ring supplies both without another hash package.

Every session receives a new CA key. Trust is a separate explicit action tied to the exact DER and thumbprint. Plaintext PKCS#8 buffers under fragcap control are zeroized. Ring signing objects have no documented complete erasure contract, so the implementation reports buffer erasure and object drop accurately rather than claiming perfect memory erasure.

## Decision 5: Bound and invalidate leaf ownership

Each validated DNS name or IP address receives a fresh P-256 key and SAN-only server certificate whose validity cannot exceed the CA or cache policy. Wildcards, uncanonicalized internationalized names, malformed SNI, and zoned IP literals are refused. The cache is bounded by entry count, total owned DER bytes, and lifetime, evicts least-recently-used entries, and invalidates on CA or policy generation changes.

Cached rustls signing keys own no extra plaintext PKCS#8 copy. Independent dev tests use x509-parser to assert SAN, CA, key-use, extended-key-use, and validity semantics. Production does not add a general X.509 parser.

## Decision 6: Share one native current-user trust implementation

Windows trust lives behind an injected `CertificateStore` seam in `fragcap-proxy`. The native implementation opens `CurrentUser/Root`, adds only a new exact certificate, queries by SHA-1 and exact DER, and deletes only the matching owned context. Missing cleanup is idempotent; duplicate, same-thumbprint/different-DER, wrong-store, access-denied, and partial readback states are typed. `LocalMachine/Root` is diagnostic only and never a widened mutation target.

The existing windows-sys 0.36 line exposes the required CryptoAPI calls. Private-material protection uses a dedicated directory, current-user data protection for persisted PKCS#8 bytes, and a protected current-user ACL through the authorization APIs. Tests use injected stores and ACL effects; ordinary CI never mutates an operator trust store.

## Decision 7: Make raw events bounded and authoritative

`ObservationStream` is one versioned contract with stable session, connection, order, timestamp, provenance, payload state, and typed event family fields. Its queue uses the established drop-oldest rule. Payloads are truncated only at the declared bound, with original length and a named counter retained. Refused, unparsed, dropped, truncated, and projection-gap counts are distinct. Any non-zero gap makes completeness false.

Raw events do not depend on HAR, key-log, or bundle projections. Unknown families and malformed source bytes remain representable.

## Decision 8: Keep the lab test-only and use real QUIC

The lab is an integration-test support tree inside `fragcap-proxy`. HTTP/1.1, HTTPS, HTTP/2, streaming HTTP, raw TCP, non-HTTP TLS, UDP, and QUIC use real loopback transports. WebSocket, gRPC, and SOCKS use valid framed exchanges on real sockets without promoting their test parsers into production. Every family declares negotiated-wire, framed-wire, or reference-vector fidelity and independent output expectations.

The selected production graph covers every family except negotiated QUIC. Quinn 0.11.11 is therefore dev-only with `runtime-tokio` and `rustls-ring`, default features disabled. This avoids its platform-verifier default and provides the positive local QUIC endpoint #289 requires. Exact packet bytes are not goldenized; semantic transcripts and resource conservation are.

## Decision 9: Preserve deterministic truth without fabricating packet capture

Every scenario uses port zero, symbolic endpoint identifiers, barriers or channels instead of sleeps, synthetic `.invalid` names and payloads, and a truth ledger for reads, writes, datagrams, tasks, and terminal state. The portable lab records wire truth at client/origin wrappers. It marks `.fcapng` unavailable until an actual Capture adapter supplies packet truth and never synthesizes a plausible capture from application writes.

## Dependency Impact

- Production adds no registry package: ring and zeroize are already locked; windows-sys 0.36 is already direct elsewhere.
- Rcgen enables `zeroize`; direct zeroize uses `alloc` without derive.
- Dev-only x509-parser validates generated DER independently.
- Dev-only Quinn is exact-pinned with defaults off; its graph must pass MSRV, advisory, license, and duplicate review before commit.
- Rejected: OpenSSL, native-tls, AWS-LC, webpki roots, platform verifier, permissive TLS verification, production x509-parser, secrecy, tonic, tungstenite, and a reference datagram mislabeled as QUIC negotiation.
