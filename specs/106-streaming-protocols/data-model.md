# S106 Data Model

## WebSocketFrame

Carries connection, stream, direction, frame sequence, FIN/RSV/opcode bits, mask presence and key, declared and observed lengths, bounded on-wire payload, and a terminal parse outcome.

## WebSocketMessage

Carries the contributing frame range, text or binary kind, compressed state, observed and retained lengths, bounded derived payload, UTF-8 validity for text, and completion or loss outcome.

## SseField and SseEvent

An `SseField` preserves ordered field name and value bytes, comment status, line number, and UTF-8 outcome. An `SseEvent` retains joined data, event type, last-event-id, retry metadata, contributing line range, and completion outcome.

## GrpcCall and GrpcMessage

`GrpcCall` retains HTTP/2 stream identity, method, content type, encoding metadata, direction activity, and terminal gRPC status. `GrpcMessage` retains direction, ordinal, compressed flag, declared length, bounded opaque bytes, and completion outcome.

## StreamingProtocolAccount

Monotonic counts and byte totals for observed, retained, omitted, malformed, oversized, unsupported, queue-dropped, and cancelled frames, messages, fields, events, and calls. Every accepted unit has one terminal classification.

## Ordering rules

1. Handshake or call-open evidence precedes protocol payload records.
2. Directional frame, event, and message ordinals are monotonic.
3. Derived messages and events reference their authoritative frame or body interval.
4. Protocol terminal evidence appears before the enclosing HTTP stream terminal.
5. Loss never removes or rewrites the forwarded wire stream.
