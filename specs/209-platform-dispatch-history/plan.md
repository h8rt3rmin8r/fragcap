# Implementation Plan: Platform Dispatch and Advisory Capture History

**Branch**: `codex/s209-platform-history` | **Date**: 2026-11-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification in specs/209-platform-dispatch-history/spec.md

## Summary

Complete #610 and #616 as one capture-time launch and history boundary. Retain the S207 game-only direct route, dispatch through a known associated platform in cold or warm state, and use the declared direct client when no association exists. Give warm platform processes query-only identity and no shutdown rights. Permit at most one preauthorized fallback only after proof that no game started. Append a bounded, versioned per-attempt history row to the local store, then compare eligible rows for advisory drift without feeding history into authorization.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.89; Markdown and SQLite schema

**Primary Dependencies**: Existing Windows Toolhelp, ETW, IP Helper, rusqlite, serde_json, and native Deep Capture crates; no new third-party package planned

**Storage**: Existing per-user local target store, additive schema version 17 history table; native session bundle remains the detailed evidence authority

**Testing**: Controlled process generation and launch matrix, store migration and retention tests, CLI and artifact readback, full cargo xtask ci, hosted Windows checks

**Target Platform**: Windows primary, portable store and pure decision tests on Linux CI

**Project Type**: Rust workspace with CLI, facade, process/session, target store, and native proxy

**Performance Goals**: No capture-path database read; bounded one-row append and at most 100 rows per target; warm-platform identity check occurs before dispatch

**Constraints**: No warm or unrelated process control, no target memory rights, no duplicate game dispatch, no silent scope widening, no new calibration gate, no raw traffic in history

**Scale/Scope**: #610 and #616 under #607, including affected CLI, session, store, artifacts, specification, and controlled gates

## Constitution Check

*Gate: Pass before research and after design.*

- **P-1**: Selected game routing still requires exact generation and destination proof. A warm platform is inspected only with literal query rights, never memory or control rights. The plan states that the warm game continues after route teardown and any routed connection may close. Unrelated payload is not retained. The pre-effect plan states each possible launch and cleanup effect.
- **P-4 and P-9**: Ambiguous roots, failed dispatch, absent transfer, history write failure, and partial cleanup remain separate named outcomes. No negative or stale history becomes a positive fact.
- **P-10**: Platform and direct launch resolve through the same stored target identity and one capture entry point.
- **P-8 and verified cleanup**: New source and docs follow UTF-8/LF, SPDX, formatting, exact fixture ownership, and the S208 aggregate test gate.
- **Installed-product boundary**: Controlled and hosted checks qualify source behavior; no private-game success is inferred.

**Post-design re-check**: The warm identity is query-only and cannot enter shutdown authority. The history table is advisory and has no path into admission or plan effects. The fallback is fixed in the authorization before effects. No constitutional exception is required.

## Project Structure

### Documentation

```text
specs/209-platform-dispatch-history/
  spec.md
  checklists/
  research.md
  data-model.md
  contracts/dispatch-and-history.md
  quickstart.md
  plan.md
  tasks.md
```

### Source

```text
crates/fragcap/src/managed_launch.rs
crates/fragcap/src/session.rs
crates/fragcap/src/deep_capture/policy.rs
crates/fragcap/src/deep_capture/direct_route.rs
crates/fragcap/src/deep_capture/native.rs
crates/fragcap/src/deep_capture/process.rs
crates/fragcap-cli/src/commands/capture.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/src/commands/deep_capture/shutdown.rs
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/events.rs
crates/fragcap-cli/src/workflow_help.rs
crates/fragcap-targets/src/schema.rs
crates/fragcap-targets/src/store.rs
crates/fragcap-targets/src/history.rs
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/s209-platform-dispatch-history.*.md
```

**Structure Decision**: Keep launch selection in the existing shared managed-launch boundary and process proof in the existing session. Keep history in the local store rather than introducing a second database or deriving it from retained bundles.

**Resolved preflight limit**: A stored Steam association must still resolve to one exact Steam executable and final client before authorization. If that identity is unavailable, no alternate is silently substituted. The one authorized direct alternate applies after a valid cold platform plan encounters a root creation failure before title dispatch. This leaves broader missing-adapter recovery outside the implemented fallback boundary.

## Complexity Tracking

No exception is planned. Query-only warm identity and the additive store table are necessary to satisfy exact process proof and per-attempt history without changing the approved route technique.
