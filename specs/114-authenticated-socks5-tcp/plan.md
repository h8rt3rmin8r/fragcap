# Implementation Plan: Authenticated SOCKS5 TCP Routing

**Branch**: `codex/114-authenticated-socks5-tcp` | **Date**: 2026-09-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/114-authenticated-socks5-tcp/spec.md`

## Summary

Extend the single native loopback listener with bounded SOCKS5 detection, RFC 1929 capability authentication, CONNECT parsing and replies, proxy-owned DNS under the existing destination policy, and cancellation-aware bidirectional TCP forwarding. Emit typed negotiation, tunnel, classification, byte, and terminal evidence through the current application and lifecycle sinks. Keep the HTTP URL for HTTP variables and add a distinct authenticated `socks5h` URL for `ALL_PROXY`.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library, tokio, base64, subtle, zeroize, rustls, and workspace crates; no new package

**Storage**: Existing bounded runtime observation, `application.jsonl`, `proxy.jsonl`, resource journal, and manifest authorities; no new artifact

**Testing**: Unit, wire contract, real loopback protocol lab, security and tenancy, routing, correlation, artifact, CLI controlled-target, and full xtask gates

**Target Platform**: Windows production runtime; portable loopback tests on Windows and Linux without elevation, driver, game, account, or Internet

**Project Type**: Rust workspace with leaf proxy crate, facade orchestration, and thin CLI

**Performance Goals**: One fixed buffer per forwarding direction; parsing and classification bounded by existing protocol deadlines; no forwarding task outlives bounded shutdown

**Constraints**: Single loopback listener, session capability only, proxy-owned DNS, existing destination policy, byte-transparent opaque forwarding, no UDP, no generic TCP payload claim, no new dependency

**Scale/Scope**: Issue #310 only; #311 through #318 and #334 remain open

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. The listener remains explicit, loopback-only, session-authenticated, child-scoped, reversible, and external to the target. No system proxy, process access, injection, hook, driver, key extraction, or pinning bypass is added.
- **P-2/P-3**: PASS. SOCKS wire and forwarding mechanics stay in the leaf `fragcap-proxy`; route policy stays in the facade; CLI only launches the prepared child. Capture and attribution are unchanged.
- **P-4/P-9**: PASS. Every refused, failed, timed-out, cancelled, saturated, dropped, or forced path is counted. Classification peeks without consuming or changing bytes, and unknown remains opaque.
- **P-5**: PASS. Packet output remains ordinary pcapng. SOCKS evidence uses existing sidecars.
- **P-6/P-8**: PASS. SOCKS-specific vocabulary receives glossary entries and all files remain under mechanical gates.
- **P-10**: PASS. The same stored target and child environment route remain authoritative; only the `ALL_PROXY` value gains its correct scheme.
- **P-11**: PASS. The master specification records issue #310 only and retains all later transport and completion gaps.

Post-design check: PASS. One listener and one connection id span admission, tunnel evidence, packet/process correlation, and cleanup. No competing lifecycle or artifact authority is introduced.

## Architecture and Phases

1. Add a bounded SOCKS5 wire module with greeting, RFC 1929 authentication, CONNECT request, reply mapping, classification, and tunnel outcome types.
2. Extend the session capability with a non-allocating constant-time SOCKS credential check and an authenticated `socks5h` URL producer.
3. Detect SOCKS5 on the shared listener, authenticate before upstream work, reuse `DestinationAuthority`, `DestinationPolicy`, and bounded cancellable upstream connection, then forward with configured fixed buffers and shutdown cancellation.
4. Add SOCKS-specific runtime counters and typed application events for negotiation, requested authority, DNS ownership, CONNECT, classification, directional bytes, and terminal reason.
5. Serialize the new events into application and proxy lifecycle streams and retain the existing connection window for packet/process correlation.
6. Extend `ProxyRoute` and child environment resolution so `ALL_PROXY` receives the SOCKS URL while HTTP variables remain unchanged.
7. Add a deterministic loopback matrix for valid address forms, authentication isolation, malformed and unsupported requests, policy and network failures, timeout, pipelining, half-close, cancellation, backpressure, classification, routing, and conservation.
8. Update architecture, glossary, status, changelog, and run every repository gate.

## Project Structure

### Documentation (this feature)

```text
specs/114-authenticated-socks5-tcp/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── socks5-wire.md
│   └── socks5-evidence-routing.md
├── checklists/
│   ├── requirements.md
│   └── security.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-proxy/src/
├── application.rs
├── auth.rs
├── lib.rs
├── model.rs
├── runtime.rs
└── socks5.rs

crates/fragcap-proxy/tests/
└── socks5_proxy.rs

crates/fragcap/src/deep_capture/
├── adapters.rs
├── application.rs
├── lifecycle.rs
├── native.rs
└── routing.rs

crates/fragcap/tests/
└── deep_capture_routing.rs

crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/tests/cli_deep_capture.rs
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/glossary/capture-and-networking.md
docs/plans/README.md
AGENTS.md
```

**Structure Decision**: Put protocol bytes and transport ownership in `fragcap-proxy`, project typed observations and route values through the existing facade seams, and keep CLI behavior limited to the prepared child environment and controlled lab.

## Complexity Tracking

No constitution violation requires an exception.
