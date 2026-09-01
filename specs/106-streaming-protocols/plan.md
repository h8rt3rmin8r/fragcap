# Implementation Plan: Streaming Application Protocols

**Branch**: `codex/106-streaming-protocols` | **Date**: 2026-08-30 | **Spec**: `specs/106-streaming-protocols/spec.md`

## Summary

Extend the native Deep Capture proxy with wire-preserving WebSocket inspection over HTTP/1.1 and HTTP/2, incremental Server-Sent Events parsing, and bounded gRPC envelope observation. Raw frames and body bytes remain authoritative, semantic records are derived, and observation pressure never delays or changes forwarding.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Tokio 1.53.1, h2 0.4.19, bytes 1.12, ring 0.17.14, flate2 1.1.10

**Storage**: Existing append-only application JSON Lines version 2 artifact

**Testing**: Cargo unit, integration, controlled loopback protocol-lab, malformed-input, soak, and repository xtask gates

**Target Platform**: Windows product runtime; portable loopback tests require no elevation, trust mutation, capture driver, account, or Internet access

**Project Type**: Rust workspace containing libraries, a facade, and a CLI

**Performance Goals**: Forward large and indefinite streams without whole-stream buffering; parse across arbitrary transport fragmentation; maintain bounded protocol state under slow consumers

**Constraints**: Exact finite frame, message, line, event, queue, retained-byte, idle, and shutdown bounds; transparent forwarding; no protobuf inference; no sensitive payloads in diagnostics

**Scale/Scope**: Three Deep Capture issues (#295, #298, #299), one proxy crate, additive facade artifact records, specifications, tests, and public documentation

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1 Safety boundaries**: PASS. Inspection remains inside the explicitly selected and authenticated local proxy. No target-process access or passive-capture transmission is added.
- **P-2 Least privilege**: PASS. No new privileges or persistent trust effects are required.
- **P-3 Layered architecture**: PASS. Wire parsing stays in `fragcap-proxy`; durable serialization stays in `fragcap`; the CLI remains presentation only.
- **P-4 Loss visibility**: PASS. Oversize, malformed, unsupported compression, queue loss, cancellation, and incomplete outcomes remain distinct.
- **P-5 Determinism**: PASS. Directional offsets, fragment order, field order, and stable record sequencing are explicit.
- **P-6 Testability**: PASS. Controlled loopback peers cover every protocol without external services.
- **P-7 Bounded resources**: PASS. All parser buffers, messages, events, tasks, and waits have finite limits.
- **P-8 Dependency discipline**: PASS. `flate2` is promoted from the existing exact lock graph for stateful RFC 7692 DEFLATE; protocol parsing otherwise uses existing dependencies.
- **P-9 Honest capability claims**: PASS. gRPC payloads remain opaque without schemas and unavailable semantic projections are named explicitly.

Post-design check: PASS. Derived protocol records retain links to authoritative wire/body evidence and cannot influence the forwarding path.

## Architecture and Phases

1. Define bounded protocol models, parser outcomes, and additive application record contracts.
2. Implement incremental WebSocket framing, message assembly, RFC 7692 handling, and verified HTTP/1.1 and RFC 8441 activation.
3. Implement incremental SSE line and event parsing from identity response bodies.
4. Implement gRPC detection and five-byte envelope parsing over HTTP/2.
5. Integrate protocol observers into existing HTTP/1.1 and HTTP/2 body relays without coupling observation backpressure to forwarding.
6. Extend application JSON Lines serialization, accounting, documentation, and controlled protocol tests.
7. Run convergence and the complete repository gate, then commit locally.

## Project Structure

```text
specs/106-streaming-protocols/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md

crates/fragcap-proxy/src/
├── application.rs
├── grpc.rs
├── http1.rs
├── http2.rs
├── model.rs
├── sse.rs
└── websocket.rs

crates/fragcap/src/deep_capture/application.rs
crates/fragcap-proxy/tests/
crates/fragcap/tests/
docs/
```

**Structure Decision**: Extend the existing proxy and facade boundaries. No new crate or reverse dependency is introduced.

## Complexity Tracking

No constitution violation requires an exception.
