# Research: Launcher TLS Refusal Diagnosis

## Existing evidence and boundaries

Issue #574 describes 19 installed v0.10.14 client-facing TLS failures after authenticated CONNECT: 14 `unexpected-eof`, five `invalid-data`, no recorded TLS alert, and a shared `tls-refusal-unknown` code. Ten `http-header-timeout` terminals followed completed HTTP exchanges. These facts locate the failing boundary but cannot identify a certificate policy, pinning, client-auth requirement, intentional closer, or a causal effect on launch timing. The private bundle is not part of this source change.

The native runtime already records accepted connection IDs and reached/completed stages in `crates/fragcap-proxy/src/runtime.rs`. `accept_client_tls` in `crates/fragcap-proxy/src/tls.rs` converts its handshake I/O into a stable refusal code, optional rustls alert, transport category, and OS code. `ConnectionDiagnostic` is bounded, and `crates/fragcap/src/deep_capture/native.rs` joins proxy connections to packet-flow ownership independently in `application.jsonl`. Its exact owner is not inferable from the proxy peer tuple. The HTTP/1 accounting path already distinguishes header timeout before the first request from timeout after completed work.

The pinned rustls 0.23.46 error contract distinguishes `InvalidMessage` (peer TLS message contents invalid), `PeerIncompatible` (required TLS feature/version unavailable), and `AlertReceived` (peer sent an alert). The [rustls error documentation](https://docs.rs/rustls/0.23.46/rustls/enum.Error.html) and exact pinned crate source are the classification authority. `std::io::ErrorKind::InvalidData` alone is broader than `rustls::Error::InvalidMessage`, so it does not justify the same code.

## Decisions

### D1: Classify a transport-ended handshake without assigning motive

**Choice**: Add one stable `tls-transport-ended` refusal for EOF, reset, aborted connection, and broken pipe during a TLS handshake. Keep the exact transport category and optional OS code. The boundary states client-facing or upstream TLS; the code does not say the client deliberately closed or rejected a certificate.

**Alternatives**: Keep `tls-refusal-unknown` for every bare I/O result, or map EOF to trust/pinning. The first loses observable transport information at the stable-code layer; the second asserts unsupported cause. The selected code is a factual transport outcome.

### D2: Classify only structured malformed TLS as malformed

**Choice**: Add `tls-invalid-message` for a rustls `InvalidMessage` cause and keep `protocol-mismatch` for existing `PeerIncompatible`, no-ALPN, and explicit protocol alerts. Generic invalid-data I/O without a structured rustls cause remains `tls-refusal-unknown` with `invalid-data` transport category. Search the I/O error source chain for the structured rustls error so one wrapper cannot erase it.

**Alternatives**: Treat every invalid-data kind as protocol mismatch or parse the human error message. Both can misclassify unrelated I/O and are unstable across libraries and locales. Parsing TLS flights would introduce raw-content handling unnecessary for this slice.

### D3: Reuse the existing bounded artifact and owner authority

**Choice**: Keep connection-diagnostic schema version 1 and its existing fields. New codes flow through the established `ProtocolError`, runtime trace, facade projection, proxy lifecycle stream, and calibration JSON. Assert the accepted connection ID joins the independently proven exact process generation in application correlation; preserve unavailable and ambiguous results and the existing dropped-detail counter.

**Alternatives**: Add a second owner field to proxy diagnostics or a new data store. The proxy lacks process identity at accept time; duplicating post-session correlation would produce conflicting authorities. No storage migration or dependency is needed.

### D4: Prove controlled compatibility and report the private-title limit

**Choice**: Drive two independent controlled launcher/updater-like clients through authenticated CONNECT, client-facing TLS, verified upstream TLS, and HTTP, then exercise EOF, malformed TLS, alert, and timeout comparators. The compatible fixture proves this native path can work; it cannot establish why the installed title refused. No real game, machine trust mutation, or private bundle is required.

**Alternatives**: Re-run the same child-environment calibration or infer a field cause from 0.10.14 aggregate counts. Neither closes the source-level evidence gap. The remaining direct-socket routing capability is #572.

## No unresolved clarification

The owner selected #574 for S198, the issue and architecture define the supported evidence boundary, and the controlled fixture can give a deterministic acceptance result. No architecture-defining decision lacks a best-supported option.
