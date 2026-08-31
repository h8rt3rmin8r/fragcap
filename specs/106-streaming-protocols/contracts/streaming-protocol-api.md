# Streaming Protocol API Contract

- Protocol observers accept byte slices incrementally and never own network forwarding.
- `feed` returns typed events without blocking on artifact storage.
- WebSocket activation requires a verified HTTP/1.1 upgrade or accepted RFC 8441 extended CONNECT.
- SSE activation requires a compatible `text/event-stream` response content type.
- gRPC activation requires HTTP/2 and an exact `application/grpc` media type with only a valid suffix or parameter boundary.
- HTTP/2 WebSocket observation begins after the successful extended CONNECT response establishes the negotiated extension set.
- Every parser has finite configured limits and a terminal `finish` operation.
- Invalid or oversized input retires semantic parsing for the affected unit or stream while transparent forwarding continues when transport policy permits.
