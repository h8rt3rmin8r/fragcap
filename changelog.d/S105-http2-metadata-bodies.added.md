<!-- spec-impact: 2.1, 13.7, 19.6, 25, 28.1 -->
### Added

- Native Deep Capture now proxies multiplexed HTTP/2 over the separately verified TLS boundaries with distinct connection and stream identity, bounded flow control, and stream-local terminal evidence.
- HTTP observations now retain protocol-faithful metadata. HTTP/1.1 keeps wire order and casing; HTTP/2 keeps typed pseudo-fields, binary-safe values, and the duplicate ordering exposed after decompression while naming unavailable HPACK representation explicitly.
- Request and response bodies now produce bounded incremental raw evidence independently from forwarding capacity, with separate bounded gzip, zlib-deflate, and Brotli transformation records.
- `application.jsonl` version 2 is appended and flushed during the session, remains prefix-readable after interruption, and uses exactly one reconciling trailer to mark orderly writer completion.

### Changed

- The HTTP body byte limit now controls retained evidence rather than refusing otherwise valid large or indefinite forwarding.
- The application artifact schema is versioned independently from the bundle manifest and records explicit non-export reasons for deferred protocol families.

### Fixed

- HTTP/2 connection-driver tasks abort with their owning connection rather than detaching during forced shutdown.
- Origin TLS verification still completes before CONNECT success while the client-facing handshake advertises only the exact application protocol selected by the verified origin.
