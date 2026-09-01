# Tasks: Streaming Application Protocols

**Input**: Design documents from `/specs/106-streaming-protocols/`

## Phase 1: Foundations

- [x] T001 Add exact direct DEFLATE dependency declarations
- [x] T002 Define bounded protocol event models and parser limits
- [x] T003 Extend application JSON Lines v2 protocol records and loss accounting

## Phase 2: WebSocket

- [x] T004 [US1] Add fragmented, masked, compressed, malformed, oversized, UTF-8, and control-frame tests
- [x] T005 [US1] Implement incremental RFC 6455 frame and message observation
- [x] T006 [US1] Implement RFC 7692 negotiation and bounded per-message decompression
- [x] T007 [US1] Verify HTTP/1.1 handshakes and replace generic upgrade copying with observed relay
- [x] T008 [US1] Enable and observe RFC 8441 HTTP/2 extended CONNECT

## Phase 3: Server-Sent Events

- [x] T009 [US2] Add arbitrary-chunk, field, comment, reconnect, malformed UTF-8, limit, cancellation, and soak tests
- [x] T010 [US2] Implement bounded incremental WHATWG SSE parsing
- [x] T011 [US2] Attach SSE observers to identity event-stream response bodies

## Phase 4: gRPC

- [x] T012 [US3] Add unary, client, server, bidirectional, compressed, malformed, oversized, cancelled, and status tests
- [x] T013 [US3] Implement bounded gRPC envelope parsing without protobuf inference
- [x] T014 [US3] Detect and observe gRPC request and response streams over HTTP/2

## Phase 5: Integration and Gates

- [x] T015 Add controlled end-to-end protocol and application artifact coverage
- [x] T016 Update architecture, support-boundary, issue-traceability, and public documentation
- [x] T017 Add S106 changelog and architecture decision fragments
- [x] T018 Run focused verification and resolve every failure
- [x] T019 Run spec-kit convergence analysis and append any missing requirement work
- [x] T020 Run `cargo xtask ci` and resolve every failure
- [x] T021 Mark S106 complete and commit the verified slice locally

## Dependencies and Execution Order

```text
Foundations -> WebSocket -> SSE -> gRPC -> Integration -> Convergence -> Full CI -> Commit
```

The three protocol parsers share typed event and accounting foundations. Each parser is independently testable. HTTP integration follows parser correctness, and durable serialization follows final event shapes.
