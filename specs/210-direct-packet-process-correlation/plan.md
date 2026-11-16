# Implementation Plan: Direct Packet and Process Correlation

**Branch**: `codex/s210-direct-packet-process-correlation` | **Date**: 2026-11-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification in specs/210-direct-packet-process-correlation/spec.md

## Summary

Close #594 and #618 at the shared packet, flow, process, and artifact evidence boundary. Carry packet flags and exact socket creation from their actual authorities, bind finite post-exit tails only with unique generation and continuity proof, and reconcile selected direct routes independently from passive original-packet availability.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.89
**Primary Dependencies**: Existing parser, IP Helper socket-table attributor, ETW process watcher, WinDivert direct route, Npcap passive capture, native proxy, and BLAKE3 already present in the workspace lock graph; no new lockfile package planned
**Storage**: Existing finite flow registry and versioned native session bundle; no database migration
**Testing**: Parser-to-flow-to-process controlled matrix, direct TCP/UDP route and artifact matrix, cargo xtask ci, hosted Windows package certification
**Target Platform**: Windows native; pure evidence decision tests run on every CI platform
**Performance Goals**: One optional flag, creation instant, and fixed-size datagram digest per retained observation; no additional capture-thread lock or global packet payload retention. Keep streaming packets inline to avoid per-consumer allocation.
**Constraints**: No process lifetime widening, no synthetic original packet, no target memory access, no stale compatibility admission
**Scale/Scope**: #594 and #618 under #607, including core packet/flow attribution, facade correlation, artifacts, specification, and controlled gates

## Constitution Check

- **P-1**: Capture remains passive; Deep Capture uses the existing authorized exact-game direct route. No target process handle or memory access is added.
- **P-4 and P-9**: Route, packet, and process facts remain independent. Missing or lost evidence is reported as missing; the exit timestamp is not moved.
- **P-3**: The socket attributor owns creation-time evidence while core carries it as optional metadata; process matching stays in the facade.
- **P-8 and verified cleanup**: Controlled fixtures own exact outputs, use UTF-8/LF, and verify cleanup; native evidence is preserved when ownership is uncertain.
- **Installed-product boundary**: Hosted package proof and installed private-game outcomes remain separate claims.

**Post-design re-check**: Exact creation time and flags are observed values, not inference. A tail is a separate basis with a finite bound and complete-history gate. No constitutional exception is required.

## Project Structure

```text
specs/210-direct-packet-process-correlation/
  spec.md
  checklists/
  research.md
  data-model.md
  contracts/correlation-evidence.md
  quickstart.md
  plan.md
  tasks.md
```

## Source Boundary

`fragcap-core` adds optional TCP flags, socket creation, and a digest of the complete network datagram to retained observations without changing packet payload bytes. `fragcap-attr` attaches the row's creation instant to the resolved attribution. The facade process trace applies the strict tail proof and emits versioned records. The existing direct-route bridge reconciles route receipt and transfer proof with the finite passive flow history, retaining both outcomes in application, compatibility, and manifest projections. The curated Rust API advances to major version 2 because a public observation struct gains an optional field.

## Decision Record

The explicit tail basis wins over extending process lifetime because the latter would make `process.exited` false. Independent route and packet facts win over requiring the consumed original packet because driver order is not an observation guarantee. A prior exact same-flow in-lifetime observation is required for ACK/FIN because flags alone cannot distinguish reuse.

## Complexity Tracking

No new subsystem or dependency is planned. The existing flow registry is the bounded packet evidence authority; the existing direct-route receipt is the route authority.
