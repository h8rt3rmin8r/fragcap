<!-- spec-impact: 2.1, 13.7, 19.6, 25, 28.1 -->
### Added

- Added bounded native WebSocket inspection over verified HTTP/1.1 upgrades and HTTP/2 extended CONNECT, including raw frames, masking, fragmentation, derived messages, UTF-8 validation, and per-message DEFLATE outcomes.
- Added incremental Server-Sent Events fields, comments, event dispatch, last-event identifiers, and retry metadata without buffering an indefinite response.
- Added schema-free gRPC call, opaque message-envelope, compression-flag, and terminal-status observations over HTTP/2.
