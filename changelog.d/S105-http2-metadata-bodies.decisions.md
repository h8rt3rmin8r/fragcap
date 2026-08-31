<!-- spec-impact: 13.7, 19.6, 25, 28.1 -->
### Decisions

- S105 combines issues #294, #296, #297, and #301. Streaming bodies require a durable bounded application authority, so leaving the live artifact to a later slice would force either unbounded memory or a temporary output contract.
- HTTP/2 uses the exact-pinned `h2` API directly because stream identity, reset, flow-control, and SETTINGS behavior are evidence rather than transport details. Hyper remains in the graph but does not own the multiplexed bridge.
- HTTP/2 metadata states the boundary the protocol engine actually exposes. Original compressed HPACK bytes and complete cross-name wire order are unavailable and are never reconstructed as if observed.
- Forwarding bounds and observation-retention bounds are separate. Network flow control limits in-flight memory while message, session, queue, storage, decoder output, ratio, concurrency, and time limits bound retained and derived evidence.
- `async-compression` is exact-pinned with defaults disabled and only Tokio gzip, zlib, and Brotli enabled. HTTP `deflate` means the zlib-wrapped form; raw-deflate fallback is deliberately unsupported because guessing would make malformed input ambiguous.
- The proxy emits typed events without awaiting disk. The facade owns schema version 2 and a dedicated bounded writer, and the CLI no longer overwrites a live native artifact during final bundle assembly.
- Review changed CONNECT negotiation order: after destination policy and TCP admission, the proxy establishes the client TLS boundary first, learns the client's selected ALPN, and then offers exactly that protocol to the verified origin. This avoids rejecting HTTP/1.1-only clients when an origin also supports HTTP/2. Origin TLS failure is consequently reported inside the already-established CONNECT tunnel.
- Session body retention and decoder concurrency belong to the proxy runtime, not an HTTP connection. One runtime-owned counter and semaphore are shared by HTTP/1.1 and HTTP/2 across every connection so configured session bounds remain real.
