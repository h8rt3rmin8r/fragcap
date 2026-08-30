# Implementation Plan: Native Deep Capture Proxy Foundation

**Branch**: `codex/102-native-proxy-foundation` | **Date**: 2026-08-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/102-native-proxy-foundation/spec.md`

## Summary

Resolve issues #279, #280, #281, #282, and #291 as one foundation slice. Raise the workspace MSRV to Rust 1.88, add a publishable `fragcap-proxy` crate with an exact minimal-feature Tokio/Hyper/rustls/rcgen graph, implement a loopback-only bounded listener whose lease owns and accounts for every task, adapt it through the existing library-first Deep Capture seam, and publish an honest completion contract while leaving the shipped CLI on external mitmdump until #290.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88, pinned development toolchain 1.96.0

**Primary Dependencies**: Tokio 1.53.1; selected future protocol stack Hyper 1.11.1, hyper-util 0.1.20, http-body-util 0.1.5, rustls 0.23.43 with ring, Tokio Rustls 0.26.4, rcgen 0.14.10, rustls-native-certs 0.8.4

**Storage**: No new persistent storage; typed runtime state is in memory

**Testing**: Rust unit/integration tests, xtask gates, Cargo deny, MSRV build, Windows release build, documentation/site checks

**Target Platform**: Windows 10/11 x86_64 MSVC; loopback-only tests remain portable

**Project Type**: Multi-crate Rust library and CLI workspace with a documentation site

**Performance Goals**: Live connection tasks never exceed configured capacity; observation and shutdown complete within caller budgets; saturation creates no task

**Constraints**: No external proxy/certificate command in the native path, no global routing mutation, no target-process access, no protocol claim in S102, every task joined or reported

**Scale/Scope**: One listener per lease; configurable finite connection capacity; five GitHub issues and their documentation

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 Defensive posture**: Pass. Explicit loopback-only infrastructure with no injection, hooks, memory access, interception driver, key extraction, pinning bypass, or system-wide proxy mutation.
- **P-2 Core neutrality**: Pass. Networking dependencies remain in `fragcap-proxy`; `fragcap-core` is unchanged.
- **P-3 Dependency direction**: Pass. `fragcap` depends on the new leaf; the leaf depends on no fragcap crate or CLI.
- **P-4 Loss accounting**: Pass. Accepted, saturated, completed, failed, forced, and incomplete outcomes are typed and conserved.
- **P-5 Bounded work**: Pass. Tasks, buffers, shutdown, and command waits have finite bounds.
- **P-6 Documentation**: Pass. Public terminology and links change with the architecture.
- **P-7 Controlled tests**: Pass. Ephemeral loopback tests need no driver, elevation, game, trust change, or Internet.
- **P-8 Determinism**: Pass. Stable identity, monotonic counters, exact pins, and lockfile.
- **P-9 Silent wrongness**: Pass. Inspection capabilities are false and application observations remain empty.
- **P-10 Dependencies**: Pass. Exact graph is justified and audited.
- **P-11 Specification lock-step**: Pass. Master spec, outline, decisions, and changelog change together.

## Project Structure

```text
specs/102-native-proxy-foundation/
├── checklists/
├── contracts/native-proxy-api.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md

crates/fragcap-proxy/
├── Cargo.toml
├── README.md
├── src/{lib.rs,model.rs,runtime.rs}
└── tests/lifecycle.rs

crates/fragcap/
├── Cargo.toml
├── src/deep_capture/native.rs
└── tests/native_proxy.rs
```

**Structure Decision**: `fragcap-proxy` is a leaf because it owns network/runtime effects and a feature-specific graph that cannot enter neutral core. The facade converts native values into existing Deep Capture traits. The CLI retains the shipped external adapter until #290.

## Implementation Sequence

1. Enforce the dependency and MSRV policy.
2. Add failing validation, identity, lifecycle, saturation, cancellation, and repeated-cycle tests.
3. Implement typed values and the bounded Tokio runtime.
4. Add failing facade tests and implement the adapter.
5. Update dependency and release/package gates.
6. Publish the full ownership/support contract and correct public status entry points.
7. Run focused, MSRV, audit, full CI, site, diff, and encoding checks, then commit locally.

## Complexity Tracking

No constitutional exceptions. The new crate and MSRV increase are justified by #278 and isolated from `fragcap-core`.
