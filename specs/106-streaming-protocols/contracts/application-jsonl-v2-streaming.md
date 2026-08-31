# Application JSON Lines V2 Streaming Additions

Schema version 2 gains additive record families: `websocket.frame`, `websocket.message`, `websocket.terminal`, `sse.field`, `sse.event`, `sse.terminal`, `grpc.call`, `grpc.message`, and `grpc.terminal`.

Every record carries existing correlation anchors when available. Binary values use the existing binary JSON representation. Records distinguish observed length from retained length and name malformed, limit, unsupported, queue-loss, cancellation, and incomplete outcomes. Gap and trailer records reconcile dropped records and protocol payload bytes without weakening body reconciliation.
