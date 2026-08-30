# Implementation Plan: HTTP/2, Metadata, and Streaming Bodies

**Branch**: `codex/105-http2-metadata-bodies` | **Date**: 2026-08-30 | **Spec**: `specs/105-http2-metadata-bodies/spec.md`

**Input**: Feature specification from `specs/105-http2-metadata-bodies/spec.md`

## Summary

Extend the native Deep Capture proxy with bounded HTTP/2 multiplexing, protocol-faithful metadata, incremental raw and decoded body observations, and a live versioned application JSON Lines artifact. The proxy owns protocol truth and nonblocking event production, the facade owns durable application schema and writing, and the CLI remains presentation and filesystem plumbing only. Forwarding uses bounded flow control independently from observation retention, so artifact pressure cannot silently corrupt or stop otherwise permitted traffic.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Tokio 1.53.1, h2 0.4.19, bytes 1.12, rustls 0.23.43, tokio-rustls 0.26.4, httparse 1.10.1, serde_json 1.x, async-compression 0.4.43

**Storage**: Append-only local JSON Lines application artifact with bounded writer queue and explicit completion trailer

**Testing**: Cargo unit, integration, controlled protocol-lab, golden, malformed-input, and repository xtask gates

**Target Platform**: Windows product runtime; portable loopback tests require no elevation, trust mutation, capture driver, game account, or Internet access

**Project Type**: Rust workspace containing libraries, a facade, and a CLI

**Performance Goals**: Correctly pair at least 32 overlapping HTTP/2 streams; network forwarding remains bounded and independent of artifact I/O; all accepted stream and writer work terminates inside the session budget

**Constraints**: Loopback and session-capability admission; no process injection or reads; exact finite connection, stream, header, queue, retained-byte, decoder, time, and shutdown bounds; raw evidence is authoritative; no sensitive metadata in human logs

**Scale/Scope**: Four Deep Capture issues (#294, #296, #297, #301), one proxy crate, the facade Deep Capture orchestration, CLI artifact plumbing, specification and verification documentation

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 Safety boundaries**: PASS. The design remains an explicitly selected loopback proxy, authenticates admitted traffic, applies destination policy before upstream work, does not inject into or read target processes, and never transmits through passive Capture mode.
- **P-2 Least privilege**: PASS. Tests are portable and unprivileged. Existing current-user trust effects are neither expanded nor hidden.
- **P-3 Layered architecture**: PASS. Protocol truth stays in `fragcap-proxy`; durable application schema and orchestration stay in `fragcap`; the CLI supplies filesystem effects and presentation.
- **P-4 Loss visibility**: PASS. Queue saturation, truncation, scope omission, decode failure, storage failure, reset, refusal, and forced shutdown receive distinct counters and terminal evidence.
- **P-5 Determinism**: PASS. Stable connection, stream, record sequence, and byte-offset rules plus golden fixtures make output repeatable.
- **P-6 Testability**: PASS. The controlled loopback lab covers all new protocol behavior without external services.
- **P-7 Bounded resources**: PASS. Forwarding and observation retention have separate finite bounds; all tasks are owned and end within the session shutdown budget.
- **P-8 Dependency discipline**: PASS. `h2` and `bytes` are promoted from the existing lock graph. Exact-pinned `async-compression` adds only the selected pure-Rust gzip, zlib, and Brotli codecs. No C decoder or broad feature set is enabled.
- **P-9 Honest capability claims**: PASS. HTTP/2 compressed cross-name field order, frame bytes, push content, deferred protocol semantics, and incomplete artifacts are named unavailable rather than reconstructed or implied.

Post-design check: PASS. The data model and contracts keep raw evidence separate from derived projections, preserve unavailable-value provenance, and make completion dependent on a reconciling trailer.

## Architecture and Phases

### Phase 0: Research

Resolve HTTP/2 API ownership, ALPN coordination, metadata fidelity limits, bounded body decoding, live artifact ownership, crash framing, and dependency policy. Decisions are recorded in `research.md`.

### Phase 1: Design

Define protocol and application entities in `data-model.md`; bind crate-facing behavior in `contracts/native-protocol-api.md`; bind the durable JSON Lines format in `contracts/application-jsonl-v2.md`; define controlled verification in `quickstart.md`.

### Phase 2: Foundational runtime

Promote the protocol and codec dependencies, split forwarding bounds from retention bounds, add stable connection and stream identities, introduce binary-safe metadata and body event types, and add a bounded nonblocking application-event sink.

### Phase 3: Protocol implementation

Negotiate a single exact protocol on both TLS legs, dispatch HTTP/1.1 or HTTP/2, proxy multiplexed HTTP/2 streams with finite flow-control ownership, retain complete metadata at each available boundary, and tee bodies incrementally without whole-message buffering.

### Phase 4: Durable application stream

Open the approved artifact before proxy startup, serialize version 2 records through a dedicated bounded writer, flush record framing during the session, retire safely on failure, reconcile with one trailer on orderly completion, and read valid incomplete prefixes honestly.

### Phase 5: Integration and verification

Exercise the 32-stream protocol lab, malformed and adversarial bounds, metadata and body goldens, writer saturation and interruption, HTTP/1.1 regressions, lifecycle cleanup, dependency gates, documentation truth, and issue traceability.

## Project Structure

### Documentation (this feature)

```text
specs/105-http2-metadata-bodies/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── application-jsonl-v2.md
│   └── native-protocol-api.md
├── checklists/
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-proxy/
├── Cargo.toml
├── src/
│   ├── application.rs
│   ├── body.rs
│   ├── http1.rs
│   ├── http2.rs
│   ├── lib.rs
│   ├── metadata.rs
│   ├── model.rs
│   ├── protocol.rs
│   ├── runtime.rs
│   ├── tls.rs
│   └── upstream.rs
└── tests/
    └── protocol_lab.rs

crates/fragcap/
├── Cargo.toml
├── src/deep_capture/
│   ├── application.rs
│   └── mod.rs
└── tests/
    └── deep_capture.rs

crates/fragcap-cli/
└── src/commands/deep_capture.rs

docs/
├── fragcap-specification.md
└── plans/README.md
```

**Structure Decision**: Extend the existing three-layer Deep Capture boundary. `fragcap-proxy` owns protocol execution and typed observation. `fragcap` owns public orchestration and artifact contracts. `fragcap-cli` owns only user-facing launch, filesystem effects, and presentation. No new crate or reverse dependency is introduced.

## Complexity Tracking

No constitution violation requires an exception.
