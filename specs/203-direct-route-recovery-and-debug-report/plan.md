# Implementation Plan: Direct Route Recovery and Debug Report

**Branch**: codex/s203-direct-route-recovery | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Owner-authorized S203 bundle of #593, #594, #595, and #598, with push, official PR, CI, initial bot reviews, and at most one explicit second review round. Owner reserves final review and merge.

## Summary

Establish selected-flow route evidence before changing the bridge. Reproduce and repair the external client handshake failure, then use exact route proof independently of passive Npcap first-packet retention for calibration. Persist bounded selected-flow diagnostics and export a source-preserving debug report. Correct auto inspection help against the live resolver.

## Technical Context

**Language/Version**: Rust MSRV 1.89, pinned development Rust 1.96.

**Primary Dependencies**: Existing signed WinDivert 2.2.2-A, Npcap, smoltcp private TCP stack, native SOCKS proxy, serde_json, and std bounded channels. No new runtime dependency planned.

**Storage**: Existing native bundle and manifest, new protected route.jsonl with a finite crash-readable stream, derived report at a distinct operator-selected output path. No target-store schema change unless exact fact applicability needs additive metadata.

**Testing**: Driver-free route and correlation tests, controlled Windows non-loopback IPv4/IPv6 and TCP/UDP scenarios, signed MSI/ZIP package checks, existing local CI parity and hosted exact-head CI.

**Target Platform**: Windows 11 x64 live route; platform-neutral report parsing and decision fixtures where possible.

**Project Type**: Rust workspace library, native proxy, and CLI.

**Performance Goals**: No synchronous filesystem or JSON work on broad pass-through packets; diagnostics enabled and disabled meet the existing selected-flow and unrelated sentinel latency/loss limits.

**Constraints**: Finite queue, stream bytes, record count, event metadata, and report input/output bounds; exact recipient access; immutable source bundle; truthful route and passive-packet authority.

**Scale/Scope**: One authorized session with bounded selected flows; no system-wide capture requirement and no target process memory access.

## Constitution Check

**Pre-design**: P-1 permits exact authorized signed routing and requires unrelated packet continuity, no unrelated payload retention, exact owner proof, bounded queues, and cleanup. P-4 requires named loss. P-9 forbids invented observations or silent transformation. P-8 binds text and terminal layout. P-10 retains one target path; P-11 requires truthful shipped claims. The installed-product field-test rule applies to any packaged default-path run.

**Post-design**: The route stream observes only selected flows and explicitly accounts for gaps. Direct reachability and passive packet presence are independent, preserving P-9. The report is an explicit downstream copy with transformation metadata. No process instrumentation, new target store, or global proxy effect is introduced. Any new finding that changes this design must re-run the constitution check and analyze gate.

## Project Structure

The slice artifacts live in specs/203-direct-route-recovery-and-debug-report/. Runtime route and WinDivert changes belong in crates/fragcap/src/deep_capture/direct_runtime.rs, direct_bridge.rs, direct_stack.rs, windivert.rs, native.rs, routing.rs, and a focused route stream module. Artifact and manifest integration stays in the existing deep_capture artifact and lifecycle owners. CLI export, calibration verdict, Doctor inventory, terminal summaries, and scope help remain in crates/fragcap-cli. Controlled tests stay next to their owning crate and in existing package qualification tiers. Active documentation, glossary, master specification, plan index, and changelog receive only needed reconciliations.

## Implementation Sequence

1. Add selected-route event schema, bounded producer and writer, crash-prefix reader, and loss tests; instrument selected admission, proxy, reply, client response, transfer, refusal, and terminal boundaries without broad pass-through payload work.
2. Add failing non-loopback controlled TCP tests against the current bridge. Diagnose original/chosen interface and checksum behavior, fix the proven boundary, and assert client ACK, exact two-way bytes, sentinels, and cleanup for IPv4 and IPv6.
3. Add failing original-retained and original-absent direct correlation tests. Bind one-use route proof, process generation, client delivery, proxy transfer, and passive packet status under one decision; make compatibility, calibration, fact, and later admission agree for TCP and supported UDP.
4. Register route.jsonl in manifest, protected retention, Doctor, cleanup, and share export. Add the bounded read-only debug-report command and shared summary projection; cover normal, empty, refused, gap, writer failure, crash-prefix, optional capture correlation, and source immutability.
5. Correct auto inspection help and active docs, add resolver-linked conformance tests, update master specification and dated decisions, and run analyze, full local/Windows/package gates, hosted CI, and review closeout.

## Decisions

- **Diagnosis before repair**: Interface and checksum are candidates. A successful WinDivertSend is not delivery. Select the fix by a failing non-loopback reproduction, then keep the exact regression.
- **Evidence authority**: One-use route receipt and process generation prove selected ownership; observed client exchange and proxy transfer prove reachability; passive Npcap truth remains separate and may be absent.
- **Stream architecture**: Use typed selected-flow events and a bounded writer patterned on existing lifecycle.rs, not synchronous packet-path JSON or a new general logger.
- **Report sharing**: Keep detailed native route metadata protected; export a declared downstream transformation with opaque IDs and no secret receipt, without modifying native source.
- **Completion**: Repository-controlled gates close scoped issues. Later private-title failure becomes a new active-release defect, not a standing S203 gate.
