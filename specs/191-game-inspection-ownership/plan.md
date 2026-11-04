# Implementation Plan: Exact Game Inspection Ownership

**Branch**: `codex/s191-game-ownership-inspection` | **Date**: 2026-11-04 | **Spec**: [spec.md](spec.md)

## Summary

Complete #552's controlled Windows handoff and cleanup coverage, then establish #558's exact live owner decision for the native proxy. Reuse the existing process-instance and socket authorities. Keep the current CLI in explicit launch-chain mode until the relay and operator integration slices.

## Technical Context

**Language/Version**: Rust workspace using `rust-toolchain.toml`.
**Primary Dependencies**: Existing `fragcap`, `fragcap-proxy`, `fragcap-attr`, Tokio, and Windows IP Helper / ETW bindings; no new package planned.
**Storage**: Add bounded decision facts to the native evidence stream; no database migration.
**Testing**: Controlled session, identity, socket, hook-order, and Windows process-chain regressions plus applicable repository checks.
**Target Platform**: Windows product; platform-neutral typed proxy contract and deterministic non-Windows tests.
**Project Type**: Rust workspace with CLI, facade, and native proxy.
**Performance and scope**: One finite owner decision per connection or association; no blocking work on the accept loop and no unbounded retention.

## Constitution Check

- P-1/P-2: Exact selected-target identity reuses existing allowed process query rights and launch receipts; no target memory access or new effect.
- P-3: The proxy consumes a platform-neutral decision interface. Windows socket and process lookup stays in the facade; core stays platform neutral.
- P-4/P-9: Unknown, stale, ambiguous, and dropped owner evidence are named and counted. No traffic is promoted to game inspection on a guess.
- P-7/P-8: No shell wrapper changes are planned. UTF-8, LF, and Rust formatting are checked.
- P-11: The architecture of record, public API inventory, slice ordering, and changelog describe only delivered behavior.
- Completion follows repository-controlled checks. Later release observations create concrete defects rather than standing conditions.

## Design Sequence

1. Reproduce #552 with a controlled process chain and inspect whether the landed S189 handoff needs correction. Include exact cleanup/recovery state after helper exit.
2. Define immutable requested/effective scope and decision/reason vocabulary. Keep routing and destination admission independent.
3. Publish live, bounded process evidence from the orchestrator to the existing observation context; use the client socket inception collector and exact process binding before deciding.
4. Add the proxy-facing decision interface and call it before the first client-facing TLS leaf or transport inspection boundary. Preserve current launch-chain selection by default; later slices make game-only selectable with blind relay.
5. Record bounded decision evidence and verify identity, timing, uncertainty, hook order, and lifecycle countercases.
6. Update the master spec, slice index, public API inventory, changelog, and run the repository checks.

## Decision Record

- **D-1**: Use a proxy-facing trait with a synchronous bounded result. The facade owns asynchronous Windows lookup and publishes a finite result; the proxy cannot import Windows attribution. A direct socket-table read on the accept loop would block unrelated connections.
- **D-2**: Treat a role match as eligible only when paired with exact socket/process-generation and launch-root proof. Generic descendant or image-name matches are insufficient.
- **D-3**: Default the current CLI to explicit launch-chain scope. A game-only scope becomes selectable with #559 and #561 once a blind relay path exists, so this slice does not silently refuse existing middleware traffic.
- **D-4**: Freeze each decision at its deadline. Late evidence may be retained for reporting but cannot alter the transport's initial inspection choice.

## Project Structure

### Documentation

```text
specs/191-game-inspection-ownership/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/inspection-decision.md
  quickstart.md
  tasks.md
  checklists/requirements.md
```

### Source

```text
crates/fragcap/src/session.rs
crates/fragcap/src/deep_capture/native.rs
crates/fragcap/src/deep_capture/process.rs
crates/fragcap/src/deep_capture/api.rs
crates/fragcap-proxy/src/runtime.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/commands/deep_capture.rs
docs/fragcap-specification.md
docs/plans/README.md
```

**Structure decision**: Extend the existing facade bridge and native runtime seam. No new crate or core dependency is needed.
