# Implementation Plan: Deep Capture Session UX Completion

**Branch**: `codex/s149-deep-capture-session-ux` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: S149 completes issue #332 through existing exact native policy and controlled evidence.

## Summary

Add CLI-owned pure session presentation for consequence summaries, lifecycle progress, bounded observed inspection counters, and terminal evidence/recovery guidance. Preserve complete canonical authorization JSON, all structured event schemas, consent policy, lifecycle, and artifact authority. Wire presentation through the production event adapter and existing emitter verbosity gates. Trace the five issue criteria in an acceptance audit.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88.

**Primary Dependencies**: Existing facade typed API, serde_json, CLI display-cell wrapping, and emitter; no new dependency.

**Storage**: No persistent schema. Fixed-size in-memory inspection counters only.

**Testing**: Pure unit tests, injected adapter tests, controlled command integration tests, full `cargo xtask ci`.

**Target Platform**: Cross-platform controlled tests and Windows native product presentation.

**Project Type**: Multi-crate library and CLI workspace.

**Performance Goals**: Constant memory, no per-observation human line flood; first observation and every 100th observation show cumulative counters.

**Constraints**: No trust, game, installed product, or live capture execution; no schema, policy, artifact, or compatibility changes. Optional diagnostics remain best effort, required plan writes remain checked.

**Scale/Scope**: One session presentation module, production event and plan integration, five acceptance rows, all existing output modes and 40-through-80-column display contracts.

## Constitution Check

GATE: Passed before research and after design.

- P-1: No new effect or authorization path; trust and sensitive-output consent remain separate and exact.
- P-2/P-3: Presentation remains CLI-owned, with no platform leakage or capture/attribution merge.
- P-4/P-9: Observation classes, unavailable ownership, loss, artifact state, and residue stay independent; no inferred inspectability.
- P-5: Structured records, packet files, and analyzer contracts remain unchanged.
- P-6: Existing glossary terms are reused; any new externally visible vocabulary gets a glossary entry.
- P-7/P-8: Existing Rust policy, display wrapping, and gates are reused; no wrapper parser or weakened tests.
- P-10: No target authoring or storage change.
- P-11: Architecture records describe S149 without final completion or live compatibility claims.

## Project Structure

### Documentation

`specs/149-deep-capture-session-ux/` contains spec, checklists, research, data model, presentation contract, quickstart, tasks, analyze, and acceptance evidence. Architecture and chronological planning records plus a changelog fragment record the boundary.

### Source Code

`crates/fragcap-cli/src/session_ux.rs` owns pure summaries and fixed counters; `commands/deep_capture.rs` projects existing typed events and complete plans; `emit.rs` adds verbosity-gated terminal text; `display.rs` supplies cell-aware wrapping. `tests/cli_deep_capture.rs` covers the integrated controlled path.

**Structure Decision**: Avoid duplicating lifecycle policy or replacing canonical JSON with a selective new authority. Pure helpers support synthetic terminal tests without operating host resources.

## Complexity Tracking

No constitutional violation or new dependency requires justification.
