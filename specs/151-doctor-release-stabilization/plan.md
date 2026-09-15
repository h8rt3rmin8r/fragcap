# Implementation Plan: Doctor and Published Release Stabilization

**Branch**: `codex/s151-doctor-release-stabilization` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: S151 spec, delivery #414, parents #372/#331; master sections 1, 17, 24, 25, 26.3 and 27.3.

## Summary

Run eligible interactive Doctor gather/classify/render in one scoped worker with a caller-thread diagnostic coordinator. Typed bounded begin/completion messages and a two-level active phase state allow monotonic waiting updates during existing blocking calls. Instrument existing readiness leaves, expose timings and automatically time slow completions. Preserve synchronous suppressed/fix paths and all report/fact contracts. Correct current published v0.10.0 applicability and enforce a reviewed actual publication identity independently from candidate version, without rewriting history.

## Technical Context

**Language/Version**: Rust, declared MSRV 1.88; Markdown/MDX.

**Primary Dependencies**: Standard library scoped threads/channels/time, existing clap and serde_json. No added dependency/package.

**Storage**: No product store/schema change. `docs/published-release.json` records reviewed publication.

**Testing**: Controlled blocked/release Rust tests, fixed-instant state tests, parser-only CLI/reference checks, pure publication negatives, existing report goldens, whole CI/MSRV and production site/browser checks.

**Target Platform**: Shipped Windows Doctor, platform-neutral offline test seams.

**Project Type**: Existing workspace CLI and documentation site.

**Performance Goals**: One worker, sixteen fixed-size event slots, two active phase slots, one-second pending cadence. No packet/proxy path changes.

**Constraints**: Borrowed Emitter/output stay on caller thread. Receiver drops before scoped joining on coordinator failure; disconnected event sends are best effort. Explicit join propagates work panic. No detached work, deadline, verdict fabrication, new host effect, altered probe order or consent. Existing aggregate ETW session availability and teardown semantics remain unchanged.

**Scale/Scope**: Doctor seams and thirteen current applicability surfaces, currency gate and Doctor guidance; immutable history excluded.

## Constitution Check

Pre-research and post-design PASS. P-1: no new effect or target instrumentation, no installed sensitive-product or game execution. P-2/P-3: no core or dependency-direction change. P-4/P-9: bounded ordered diagnostic events, no capture observation change, pending never becomes unavailable. P-5/P-7/P-10: formats/wrappers/target authority unchanged. P-6: reuse existing probe/progress/elapsed vocabulary and check glossary for any new technical term. P-8: house Markdown and dated release-documentation decision fragment. P-11: actual publication identity is distinct from candidate Applies-To; S151 diagnostics are explicitly unreleased. Full spec-kit, blocking analysis and TDD remain mandatory. No exception is required.

## Project Structure

### Documentation (this feature)

`specs/151-doctor-release-stabilization/` contains spec, plan, research, data-model, quickstart, contracts/doctor-release, checklists and tasks Markdown artifacts.

### Source Code (repository root)

- `crates/fragcap-cli/src/doctor/probe.rs`: observer transport/runner and nested readiness boundaries.
- `crates/fragcap-cli/src/doctor/progress.rs`: fixed labels and monotonic bounded state/rendering.
- `crates/fragcap-cli/src/commands/doctor.rs`: injectable gather/render and eligible coordinator integration.
- `crates/fragcap-cli/src/cli.rs`, `src/emit.rs` and CLI tests: visible timing option and suppression policy.
- `xtask/src/spec.rs`, `docs/published-release.json`: publication validation and pure applicability regression.
- README, CONTRIBUTING, master spec, current site pages and security/release handoffs: published reconciliation and unreleased diagnostics.
- `docs/plans/README.md`, specification outline and `changelog.d/S151.*.md`: chronological records and dated decision.

**Structure Decision**: Existing CLI and currency modules suffice. No general async probe service, new report schema or live-network CI publication inference. Read-only research agents were used because the installed planning skill explicitly requests them; implementation remains local.

## Phase 0: Research

[research.md](research.md) resolves progress ownership, boundary attribution, failures, ETW limits and publication separation. No unresolved clarification remains.

## Phase 1: Design

[data-model.md](data-model.md), [contracts/doctor-release.md](contracts/doctor-release.md) and [quickstart.md](quickstart.md) define finite state, external contracts and controlled validation. Constitution was rechecked after design and passes.

## Complexity Tracking

No violations. One scoped worker is necessary because synchronous completion callbacks cannot update blocked work. A bounded channel and two-level state are proportionate; no async runtime or cancellation machinery is introduced.
