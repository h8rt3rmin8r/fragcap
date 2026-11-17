# Implementation Plan: Direct Route Reliability

**Branch**: `codex/s211-direct-route-reliability` | **Date**: 2026-11-16 | **Spec**: [spec.md](spec.md)

## Summary

Close #619 by moving slow selected TCP SOCKS admission off the packet worker and recording exact diagnostic outcomes. Finish #594's portable packaged physical WinDivert/Npcap original-packet ordering and artifact matrix.

## Technical Context

**Language**: Rust 2021, workspace MSRV 1.89.
**Dependencies**: Existing WinDivert, Npcap, native proxy, route receipt and journal. No new package planned.
**Storage**: Existing versioned native bundle and protected route stream.
**Verification**: Controlled delayed-origin and refusal fixtures, worker sentinel matrix, full foreground repository gate, portable package physical matrix, hosted Windows package certification.

## Constitution Check

- P-1: Only the authorized exact-game route is selected. No target memory or unrelated payload access is added.
- P-4/P-9: Every held packet has a terminal disposition. Helper error and passive packet truth remain exact and independent.
- P-8: Controlled outputs have exact ownership, fallible cleanup, and no focus-stealing console children.
- Installed-product boundary: Portable package driver proof is reported as such; no installed default path claim follows.

**Post-design re-check**: Finite helper work and explicit pending packet ownership preserve pass-through and accounting without expanding interception scope.

## Source Boundary

`direct_bridge.rs` owns staged SOCKS errors, finite handshake deadline, and nonblocking established helper polling. `direct_runtime.rs` owns bounded off-worker admission, pending-flow identity, selected packet dispositions, watchdog progress, and cleanup. `route_events.rs` owns protected diagnostic serialization. The core pipeline owns a separate finite output decision lane; the managed session gate admits only exact selected-packet proof before ETW publication. Native correlation binds retained capture digests to one selected flow and records ambiguity. The ETW watcher stops the owned producer before consumer drain. The controlled package harness owns physical ordering, terminal-to-bundle reconciliation, and cleanup checks.

## Project Structure

```text
specs/211-direct-route-reliability/
  spec.md
  checklists/
  research.md
  data-model.md
  contracts/helper-and-packet-evidence.md
  quickstart.md
  plan.md
  tasks.md
```

## Decision Record

Use a bounded helper task with the existing worker as the only packet disposition authority. A long synchronous read on that worker cannot meet the watchdog. Preserve stage and `io::ErrorKind` rather than reduce all failures to proxy refusal. The owner-scheduled physical run is required for #594, not replaced by source fixtures.
