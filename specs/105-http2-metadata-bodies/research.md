# S105 Research: HTTP/2, Metadata, Streaming Bodies, and Application JSONL

## R-1: Drive HTTP/2 directly through `h2`

**Decision**: Promote exact-pinned `h2` 0.4.19 and `bytes` to runtime dependencies of `fragcap-proxy`. Use `h2::server::Builder` for the admitted client leg and `h2::client::Builder` for the verified origin leg.

**Rationale**: Direct `h2` exposes stream identity, reset, flow-control, SETTINGS, and bounded stream configuration needed for accurate evidence. Hyper services abstract away protocol details this slice must report. Version 0.4.19 is already lock-resolved, declares Rust 1.63, and includes the fix for RUSTSEC-2026-0258.

**Rejected**: Hyper-only handling, because it hides required stream lifecycle details. A hand-written HTTP/2 or HPACK stack, because plausible parsing errors violate P-9. A forked `h2`, because the additional maintenance and audit surface is disproportionate to unavailable compressed header order.

## R-2: Coordinate ALPN before protocol dispatch

**Decision**: Advertise `h2` and `http/1.1` on both TLS legs. Resolve and policy-check the origin and open its TCP connection before the client-facing CONNECT success response. Complete client TLS to learn its selected ALPN, then complete origin TLS while offering exactly that protocol. Refuse a mismatch explicitly.

**Rationale**: Negotiating both legs independently can select different protocols and create a silent downgrade or a connection the proxy cannot faithfully bridge.

**Rejected**: Always selecting HTTP/1.1 upstream, because it violates the no-silent-downgrade requirement. Advertising a broader origin set after client negotiation, because the result is nondeterministic when the origin has a different preference.

## R-3: Keep HTTP/1.1 custom and dispatch by negotiated protocol

**Decision**: Retain the S104 HTTP/1.1 engine. Add a protocol dispatcher for TLS ALPN and the cleartext HTTP/2 prior-knowledge preface. An h2c connection is bound to one authenticated authority; an authority change is refused.

**Rationale**: The custom HTTP/1.1 implementation already preserves informational responses, expectation handling, upgrades, trailers, persistent connections, half-close, and strict framing refusals. Replacing it creates unrelated regression risk. A single h2c authority keeps destination policy and connection ownership explicit without adding an unbounded origin pool.

## R-4: State the real HTTP/2 metadata boundary

**Decision**: Preserve HTTP/1.1 names, casing, values, duplicates, and field order exactly as parsed from the wire head. For HTTP/2 preserve typed pseudo-fields, binary-safe regular values, and duplicate value order exposed by `h2`, with provenance stating that original HPACK bytes and compressed cross-name order are unavailable.

**Rationale**: `h2` exposes decoded requests and responses through typed fields and a header map. It does not expose original HPACK block bytes or complete cross-name order. Inventing those would create false evidence.

**Rejected**: Re-encoding fields to simulate an original block, because it is not the observed representation. Vendoring a patched protocol stack solely for order, because it expands the security ownership of this slice beyond its product value.

## R-5: Separate forwarding bounds from retention bounds

**Decision**: Remove S104's total-body forwarding cap. Forward through bounded chunks, protocol windows, queues, idle time, and shutdown time. Apply separate finite per-message, per-session, disk, event-queue, decoded-expansion, and active-decoder retention limits.

**Rationale**: A total body cap makes large or indefinite but valid streams fail. Memory safety requires bounded in-flight state, not a fixed total transfer size. Observation policy may truncate evidence while permitted forwarding continues.

**Explicit deviation**: S104 used `max_body_bytes` as both forwarding and observation policy. S105 splits those meanings because the combined limit produces a protocol regression and incorrectly couples artifact capacity to network behavior.

## R-6: Preserve raw bodies and model transformations separately

**Decision**: Emit bounded ordered raw body segments during forwarding. HTTP/1.1 raw observations retain transfer-framed bytes where available; HTTP/2 raw observations retain DATA payload because `h2` does not expose frame encoding. Transfer-decoded and gzip, zlib-wrapped deflate, or Brotli-decoded bytes are derived streams linked to the raw authority with explicit outcomes.

**Rationale**: Analysts need the observed evidence and the usable representation. Derived decoding must never overwrite or relabel raw bytes.

## R-7: Use narrowly featured `async-compression`

**Decision**: Add exact `async-compression` 0.4.43 with default features disabled and only `tokio`, `gzip`, `zlib`, and `brotli`. Enforce raw input, output, expansion ratio, elapsed time, and concurrent-decoder limits.

**Rationale**: The selected implementation is pure Rust, MIT or Apache-2.0, and declares Rust 1.83. It provides uniform bounded asynchronous decoding. HTTP Content-Encoding `deflate` uses the zlib wrapper; no raw-deflate fallback is attempted.

**Rejected**: C codec backends, broad algorithm features, and raw-deflate fallback, because each adds build, attack, or ambiguity surface. Direct `flate2` plus `brotli` would duplicate asynchronous state and limit enforcement.

## R-8: Make the application stream live and nonblocking

**Decision**: `fragcap-proxy` produces typed application events through a bounded nonblocking sink. `fragcap` owns schema version 2, serialization, reading, sequence assignment, and an artifact lease. A dedicated writer thread opens the approved file before proxy start, writes and flushes a header first, appends complete newline-framed records, and emits one reconciling trailer on orderly completion.

**Rationale**: The current CLI finalizes `application.jsonl` from an in-memory terminal snapshot, so it cannot safely retain streaming bodies or survive interruption. The proxy must never await disk I/O. A live append-only stream supplies durable bounded authority for later projections.

## R-9: Retire writer failure without hiding it

**Decision**: Queue saturation advances loss accounting. Serialization or I/O failure retires the writer atomically; later attempts are counted without repeated I/O. Reserve control capacity for gap and terminal records when the writer remains healthy. A file without exactly one valid trailer is incomplete.

**Rationale**: Continuing forwarding is correct when evidence storage fails, but claiming a complete artifact is not. Already flushed records remain valuable and must not be overwritten during CLI finalization.

## R-10: Version schema independently and preserve compatibility

**Decision**: Every version 2 line carries `schema_version: 2`, `session_id`, monotonic `sequence`, `event_time_ns`, and a record `type`. Readers accept the legacy header with `manifest_version: 1` as application schema 1, accept schema 2, and explicitly reject unknown versions. `manifest.json` remains version 1 and HAR remains deferred.

**Rationale**: Artifact schema and product version are different contracts. Legacy detection allows existing bundles to remain readable without pretending they have version 2 completeness.

## R-11: Reserve, do not fabricate, deferred protocols

**Decision**: The header declares exported families and explicit non-export reasons for WebSocket, Server-Sent Events, gRPC, generic TCP, UDP, and QUIC. S105 emits no semantic records for those families.

**Rationale**: Stable namespaces aid future extension. Reserved shapes must not become accidental feature-completion claims.

## R-12: Verify entirely in a controlled local lab

**Decision**: Extend the existing loopback lab with 32-stream overlap, resets, trailers, GOAWAY, malformed heads and frames, binary metadata, body framing and codecs, queue pressure, writer interruption, repeated shutdown, and HTTP/1.1 regression cases.

**Rationale**: Protocol correctness and lifecycle safety must be reproducible without Internet access, elevation, trust-store mutation, or a real game.
