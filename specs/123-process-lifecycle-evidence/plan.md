# Implementation Plan: Complete Process Lifecycle Evidence

**Branch**: `codex/123-process-lifecycle-evidence` | **Date**: 2026-09-03 | **Spec**: `specs/123-process-lifecycle-evidence/spec.md`

## Summary

Retain the process events already consumed by the shared capture orchestrator in one bounded evidence report, preserve managed launch and ETW watcher authority, reconcile process instances and socket-owner transitions against the existing flow registry after capture, and replace the placeholder sidecar with a versioned JSON Lines chronology whose trailer drives compatibility and manifest truth.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library, serde_json, and workspace crates; no new package

**Storage**: Bounded in-memory raw-event retention during capture and a sensitive versioned `process-trace.jsonl` sidecar with a readable prefix and reconciling trailer

**Testing**: Unit, permutation, PID-reuse, loss-injection, offline orchestrator, CLI bundle integration, manifest reconciliation, and xtask gates

**Target Platform**: Windows product runtime; portable controlled tests need no elevation, driver, game, account, or Internet

**Project Type**: Rust workspace with platform attribution, core evidence vocabulary, facade reconciliation, and CLI orchestration

**Performance Goals**: Process-event retention is O(1) per event within a fixed cap; no packet acquisition or proxy forwarding path waits on process-side serialization

**Constraints**: No target process handles, memory rights, second attribution pass, fabricated ancestry, silent loss, unbounded event history, new package, or feature-completion claim

**Scale/Scope**: Issue #319 only; direct, owned platform, and publisher managed launches plus truthful limitations for other cases

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Evidence uses existing managed launch receipts, ETW, query-only snapshots, and packet attribution. No target process handle or denylisted technique is added.
- **P-2/P-3**: PASS. Generic process and flow evidence remain in core, platform collection remains in attribution, reconciliation remains in the facade, and CLI owns presentation and session composition.
- **P-4/P-9**: PASS. Every watcher, parser, retention, writer, and join gap is counted or typed; no missing identity is inferred.
- **P-5**: PASS. Packet truth remains ordinary pcapng; process evidence remains a sidecar.
- **P-6/P-8**: PASS. New vocabulary, spec, documentation, and text gates are included.
- **P-10/P-11**: PASS. Target storage is unchanged and completion language remains bounded to issue #319.

Post-design check: PASS. The bounded report does not block capture, flow ownership reuses the existing registry, and incomplete process evidence can only weaken bundle completeness.

## Architecture and Phases

1. Extend the flow registry with a deterministic all-flow snapshot and define the process evidence report returned by the capture orchestrator.
2. Retain startup snapshot authority, raw streamed events, managed launch receipt, role and stage transitions, watcher loss, and terminal outcome under one finite cap.
3. Reconcile process instances by event time, PID generation, and ancestry; select only launch, stage, flow-owner, and ancestor-relevant records for the sensitive artifact.
4. Derive socket-owner transitions from existing flow summaries and bind them to the same `flow_id` used by packet and application records.
5. Serialize versioned JSON Lines with header, deterministic records, typed limitations, and one trailer; parse the result to drive manifest and compatibility state.
6. Integrate controlled and offline tests, documentation, changelog, and repository gates.

## Project Structure

```text
specs/123-process-lifecycle-evidence/
crates/fragcap-core/src/flow.rs
crates/fragcap/src/deep_capture/process.rs
crates/fragcap/src/deep_capture/mod.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/commands/capture.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/tests/cli_deep_capture.rs
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/glossary/capture-and-networking.md
docs/plans/README.md
```

**Structure Decision**: Reuse the existing crate direction. Core exposes an immutable flow snapshot, the facade owns lifecycle reconciliation and sidecar semantics, and the CLI transports observations between existing authorities without acquiring new ones.

## Complexity Tracking

No constitution violation requires an exception.
