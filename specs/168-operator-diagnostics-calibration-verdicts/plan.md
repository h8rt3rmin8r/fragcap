# Implementation Plan: S168 operator diagnostics and calibration verdicts

**Branch**: `codex/s168-operator-diagnostics-calibration-verdicts` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Resolve #459, #460, #461, #462, #463, #465, #466, and #467 in one branch using shared terminal projection, selected-target preparation, and typed calibration evidence. Every original issue criterion remains binding in [issue-acceptance.md](issue-acceptance.md).

## Technical Context

**Language/Version**: Existing Rust workspace, MSRV 1.88.
**Primary Dependencies**: Existing Steam metadata, facade, native proxy, and target compatibility APIs plus exact-pinned `unicode-width` 0.2.2 in the CLI (default features disabled, MIT OR Apache-2.0, declared MSRV 1.66). One lock package supplies whole-string terminal-cell measurement; the decision supersedes the initial no-new-package expectation after the handwritten ranges failed non-Latin combining and emoji coverage.
**Storage**: Existing append-only target/calibration facts, workflow checkpoints, and session artifacts. No ACL or retention migration.
**Testing**: Existing CLI/unit/controlled native tests plus focused contracts for display positions, no-effect ordered preflight, and typed terminal evidence.
**Target Platform**: Windows product, offline fixture tests on supported CI hosts.
**Project Type**: Rust library/facade/CLI workspace.
**Performance Goals**: Controlled refusal within one second with ready injected observation and no downstream calls; constant healthy-summary row count within finite inventory.
**Constraints**: Four-space columns everywhere, finite evidence bounds, honest unavailable/ambiguous outcomes, no added eligibility checks, preserved consent, structured compatibility, no real game/trust mutation.
**Scale/Scope**: Eight issue inventories; all CLI report families audited.

## Constitution Check

P-1 preserves external observation, target-scoped proxy, exact consent and recovery. P-2/P-3 keep native acquisition and attribution below the facade and platform-neutral core unchanged. P-4/P-9 retain conservation, typed independent evidence and truthful omissions, including missing correlation. P-5 preserves analyzer artifacts. P-6/P-8 require glossary/docs, encoding, conventions and controlled gates. P-7 leaves wrappers thin. P-10 reuses one target authority. P-11 updates the master/public contract without changing release identity. No exception or future field gate is needed.

## Project Structure

- `specs/168-operator-diagnostics-calibration-verdicts/`: spec, research, design, contracts, quickstart, tasks, acceptance evidence.
- `crates/fragcap-cli/src/display.rs`, Doctor, output, live_status and other human families: shared layout/history projection.
- `crates/fragcap-cli/src/commands/targets.rs`, calibrate front door, target_resolve, Steam/source APIs: selected discovery and ordered preflight.
- `crates/fragcap/src/deep_capture/`, native proxy and CLI adapter: additive typed bounded diagnostic/phase authority.
- `crates/fragcap-cli/src/session_ux.rs`, calibration result renderer: coherent chapters and final readiness guidance.
- `crates/fragcap-cli/tests/` and owning crate unit tests: production-path controlled regressions.
- `docs/`, `site/` where applicable, `AGENTS.md`, `CONVENTIONS.md`, `CONTRIBUTING.md`, `changelog.d/`: aligned public/governed contract.

## Design Decisions

Use ColumnLayout::new(indent, rows), anchor, render_row, render_wrapped_row and render_fields over complete actual emitted rows. Styling is excluded from width; exact tokens and repeated spaces are preserved, with a newline replacing one separator when prose wraps. Exact identity/path scalar fields may extend beyond the terminal width while preserving all computed anchors. Doctor --history-details is a human detail request; JSON retains exact records. Source diagnostics carry provenance without replacing existing public message projections. Exact selected Steam refresh replaces global learning where target authority is already known. TerminalSnapshot receives additive bounded proxy diagnostics and phase windows through a default optional lease method. Current stored readiness reuses compatibility applicability. No omitted artifact path claims, no duplicated fact progress, and no cleanup-prose parsing remain.

## Implementation Sequence

1. Specify, clarify, checklist, design research, tasks, and blocking consistency analysis.
2. Establish failing focused regressions before owning implementations.
3. Parallel agent work is explicitly directed by Spec-Kit plan research and the independent [P] task groups: layout/Doctor, selected discovery/preflight, and facade/native evidence. Root owns session_ux and final human integration. Shared signatures are agreed first; no overlapping edits to the same functions.
4. Integrate exact final calibration readiness, phase chapters and evidence/cleanup/artifact results. Preserve complete canonical authorization review.
5. Audit every issue criterion and renderer; update specification, directives, help and public examples.
6. Run focused and full repository gates in watched headless execution, inspect final diff, commit with changelog fragments.
7. Automatically push and publish one official PR under explicit authorization. Resolve every review thread and CI failure, optionally request one second Codex round, never a third. Verify green checks and satisfied reviews on the final exact head before owner handoff.

## Post-Design Constitution Recheck

All projections preserve original evidence; findings and limitations coexist, partial artifacts remain partial, and later cleanup traffic cannot authorize earlier calibration. Deferred ACL, retention and acquisition issues remain accurately named. No violation or complexity exception remains.
