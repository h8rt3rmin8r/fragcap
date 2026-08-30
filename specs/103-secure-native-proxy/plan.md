# Implementation Plan: Secure Native Proxy Foundation

**Branch**: `codex/103-native-proxy-foundation-completion` | **Date**: 2026-08-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/103-secure-native-proxy/spec.md`

## Summary

Resolve issues #283 through #289 as one ordered foundation slice. Extend `fragcap-proxy` with session capability authentication, bounded destination resolution and connection policy, per-session certificate-authority and bounded leaf ownership, a shared exact Windows trust implementation, and a versioned drop-oldest observation stream. Add a deterministic test-only protocol lab, including a real loopback QUIC endpoint, while preserving the external production proxy until #290.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88, pinned development toolchain 1.96.0

**Primary Dependencies**: Existing exact-pinned Tokio/Hyper/rustls/Tokio Rustls/rcgen/rustls-native-certs/ring graph; direct zeroize and ring edges already present in the lock; windows-sys 0.36 on Windows; dev-only x509-parser and Quinn for independent certificate and QUIC lab validation

**Storage**: Session-owned certificate directory with protected private material and explicit cleanup inventory; all connection, cache, event, and lab state bounded in memory

**Testing**: Rust unit and integration tests, deterministic loopback protocol lab, Windows-isolated trust tests, xtask gates, Cargo deny, MSRV build, Windows release build, documentation/site checks

**Target Platform**: Windows 10/11 x86_64 MSVC; effect-free models and portable loopback lab run cross-platform; trust and ACL effects are cfg-isolated

**Project Type**: Multi-crate Rust library and CLI workspace with documentation site

**Performance Goals**: No unauthenticated upstream work; every DNS/connect/read/write operation and shutdown has a finite budget; event and leaf-cache occupancy never exceed configuration; every overflow and refusal is counted

**Constraints**: No global routing, external certificate command, target-process access, pinning bypass, unrelated-client inspection, Internet-dependent test, or production native-proxy cutover; raw observations remain authoritative over projections

**Scale/Scope**: One session capability and CA generation per proxy lease; bounded concurrent connections, upstream attempts, event records, payload bytes, and leaf entries; seven GitHub issues

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. All effects are explicit local proxy, selected-session authentication, purpose-specific CA, current-user trust, and controlled local endpoints. Every denied technique remains absent.
- **P-2 Core neutrality**: Pass. Networking, cryptography, and Windows bindings stay in the leaf `fragcap-proxy` crate or facade/CLI adapters. `fragcap-core` is unchanged.
- **P-3 Capture and attribution separation**: Pass. No proxy work enters packet acquisition or attribution.
- **P-4 No silent loss**: Pass. Observation overflow is bounded drop-oldest with exact counters and an explicit incomplete flag. Refused and unparsed work is counted separately.
- **P-5 Compatibility outranks richness**: Pass. The lab and raw event contract do not change `.fcapng`; unsupported projections remain explicit omissions.
- **P-6 Glossary first**: Pass. New public terms are added with references before documentation uses them.
- **P-7 Wrappers stay thin**: Pass. No shell logic changes. Native trust replaces process orchestration inside Rust.
- **P-8 House standards**: Pass. Source, specs, tests, and fragments remain linted UTF-8/LF artifacts.
- **P-9 Instrument honesty**: Pass. Unknown, malformed, truncated, refused, dropped, and unavailable states remain distinct and prevent full-inspection claims.
- **P-10 One path to a target**: Pass. Target storage and resolution are unchanged.
- **P-11 Specification describes shipped state**: Pass. Production remains external until #290; docs describe S103 as a library foundation.

## Project Structure

```text
specs/103-secure-native-proxy/
├── checklists/
├── contracts/native-proxy-foundation-api.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md

crates/fragcap-proxy/
├── src/
│   ├── auth.rs
│   ├── certificate.rs
│   ├── event.rs
│   ├── lib.rs
│   ├── model.rs
│   ├── runtime.rs
│   ├── upstream.rs
│   └── windows/{acl.rs,mod.rs,trust.rs}
└── tests/
    ├── authentication.rs
    ├── certificates.rs
    ├── events.rs
    ├── protocol_lab.rs
    ├── protocol_lab_support/
    └── upstream.rs

crates/fragcap/
└── src/deep_capture/native.rs

crates/fragcap-cli/
├── src/commands/deep_capture.rs
├── src/doctor/fix.rs
└── src/windows_cert.rs
```

**Structure Decision**: Production proxy effects and portable contracts stay in the existing leaf crate. Windows trust and ACL code is cfg-isolated there and reached through the facade so sessions and doctor share one implementation. The protocol lab remains test-only to avoid turning fixture parsers into supported product APIs.

## Implementation Sequence

1. Activate the already-selected cryptography features and add the audited dev-only certificate/QUIC validation edges.
2. Add failing session-capability and listener isolation tests, then authenticate before spawning connection work.
3. Add failing authority, resolver, policy, budget, TLS-verification, and cancellation tests, then implement the bounded upstream connector.
4. Add failing CA, protected-storage, leaf-cache, rotation, and exact trust tests, then implement native certificate and Windows trust ownership.
5. Add failing event round-trip, ordering, drop-oldest, payload-limit, and conservation tests, then implement the raw observation stream.
6. Add the deterministic protocol lab and all protocol/failure matrix cases, using a real dev-only Quinn loopback endpoint for QUIC.
7. Replace CLI `certutil` mutations with the shared native trust implementation without switching the production proxy backend.
8. Update architecture, glossary, dependency, public status, and changelog records; run the analyze and complete CI-parity gates.

## Complexity Tracking

No constitutional exceptions. The dev-only Quinn edge is a deliberate scope-proportional addition: #289 requires a positive local QUIC client and origin, and a reference UDP datagram would falsely claim that criterion. It is exact-pinned, default features are disabled, and platform verification/root bundles remain excluded.
