# Implementation Plan: Crash-Safe Lifecycle Authority

**Branch**: `codex/109-lifecycle-authority` | **Date**: 2026-09-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/109-lifecycle-authority/spec.md`

## Summary

Add an immutable target-scoped routing plan and strategy seam, place a bounded fsync-backed resource journal around every current Deep Capture effect, and replace finalization-only proxy and cleanup records with append-only versioned lifecycle streams. Preserve `cleanup.json` as a derived compatibility summary, keep manifest version 2 additive, and correct S108's unbounded localized body-loss map.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library, serde_json, base64, zeroize, windows-sys, and workspace crates; no new package

**Storage**: Protected bundle-local JSON Lines journals and sidecars, per-record `sync_all`, atomic compacted snapshots and final summaries

**Testing**: Unit, model, contract, deterministic failure injection, kill-boundary simulation, CLI integration, Windows platform, schema, and xtask gates

**Target Platform**: Windows production runtime; portable recovery and stream tests require no elevation, driver, game, account, or Internet

**Project Type**: Rust workspace with library-first facade and thin CLI

**Performance Goals**: Effect recording is bounded and occurs only on the control path; proxy and cleanup streams never block forwarding; every identity index has a fixed cap

**Constraints**: No target process handles, hidden global routing, uncertain cleanup, unbounded identity map, competing artifact authority, new package, or feature-complete claim

**Scale/Scope**: Issues #306, #320, and #336 plus the S108 body-loss boundedness correction; #305 remains a separate conformance slice

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Only explicit child-scoped environment routing is implemented. Global proxying, interception, hooks, injection, memory reads, executable changes, and pinning bypass remain unavailable.
- **P-2/P-3**: PASS. Effect-neutral types and lifecycle ownership stay in the facade; proxy protocol mechanics remain in `fragcap-proxy`; packet capture and attribution are unchanged.
- **P-4/P-9**: PASS. Stream loss, journal uncertainty, recovery refusal, and overflow localization are counted and never normalized into success.
- **P-5**: PASS. Packet truth remains ordinary pcapng; lifecycle evidence uses separate sidecars.
- **P-6/P-8**: PASS. New vocabulary receives glossary entries and all files remain under repository mechanical gates.
- **P-10**: PASS. Routing is selected from the prepared target without adding another target storage or resolution path.
- **P-11**: PASS. Master specification and public status keep Deep Capture incomplete and leave #305 open.

Post-design check: PASS. The facade owns one lifecycle truth model; `cleanup.json` is explicitly derived from `cleanup.jsonl`, and manifest v2 names both without changing old readers.

## Architecture and Phases

1. Add typed `RoutingPlan`, `RoutingStrategyKind`, effect declarations, verification outcomes, and `RoutingAdapter`/`RoutingLease` seams. Bind the plan into `SessionPlan` and authorization equality.
2. Add protected `resource-journal.jsonl` with a versioned header, obligation-before-effect transitions, ownership evidence, safe parsing, inspection, recovery planning, bounded compaction, and exact executor seam.
3. Integrate journal transitions around proxy, trust, routing, launch, Capture, artifact, and cleanup operations in the coordinator and native/CLI adapters.
4. Add bounded `proxy.jsonl` and `cleanup.jsonl` leases with headers, append-only records, gaps, accounting, crash-prefix readers, and reconciling trailers.
5. Fan out proxy application events into lifecycle records, add runtime start/stop/drain accounting, and link every application connection to proxy lifecycle truth.
6. Derive `cleanup.json` from the cleanup chronology, declare all three operational artifacts in manifest v2, and wire startup/doctor journal inspection and recovery seams.
7. Bound S108 body-loss localization and preserve exact unlocalized overflow totals.
8. Add failure, restart, ownership reuse, compatibility, documentation, and full repository verification.

## Project Structure

### Documentation (this feature)

```text
specs/109-lifecycle-authority/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── routing.md
│   ├── resource-journal.md
│   └── lifecycle-streams.md
├── checklists/
│   ├── requirements.md
│   └── security-recovery.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/deep_capture/
├── adapters.rs
├── application.rs
├── journal.rs
├── lifecycle.rs
├── manifest.rs
├── model.rs
├── native.rs
├── routing.rs
└── session.rs

crates/fragcap-cli/src/
├── commands/deep_capture.rs
└── doctor/

crates/fragcap-proxy/src/
├── application.rs
├── model.rs
└── runtime.rs

crates/fragcap/tests/
├── deep_capture_journal.rs
├── deep_capture_lifecycle.rs
├── deep_capture_routing.rs
└── deep_capture_session.rs

crates/fragcap-cli/tests/cli_deep_capture.rs
```

**Structure Decision**: Extend the existing facade and proxy seams. Durable policy, recovery, and artifacts live in `fragcap`; the proxy emits typed observations but never learns bundle paths or recovery policy; the CLI supplies filesystem and Windows effect adapters only.

## Complexity Tracking

No constitution violation requires an exception.
