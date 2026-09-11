# Implementation Plan: Bounded Guided Calibration Sequence

**Branch**: `codex/s144-guided-calibration-sequence` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/144-guided-calibration-sequence/spec.md`

## Summary

Refactor S140-S143's single-attempt tail into a finite in-process sequence that repeatedly re-resolves one durable target, rebuilds the S139 proposal, selects the first useful unattempted case, and delegates it through the unchanged S134 executor. Each attempt retains a fresh complete plan, a separate confirmation, a distinct evidence bundle, and a fresh post-session fact check. The sequence advances only after exact current positive evidence, accumulates only eligible final-client protocol candidates, and stops with truthful coverage and a durable continuation on every incomplete boundary.

This deliberately supersedes S141's implementation choice to end the process after one session. The safety rationale for that boundary is preserved through separate per-attempt authorization and fresh authority checks, while S144 removes only the unnecessary process-restart boundary requested by parent issue #380.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap` facade, `fragcap-targets`, and native Deep Capture stack already present in the workspace

**Storage**: Existing per-user SQLite target store version 10 and append-only compatibility facts; no workflow record or migration

**Testing**: Focused CLI unit and integration tests, controlled Deep Capture harness, stable event serialization, documentation-reference tests, and `cargo xtask ci`

**Target Platform**: Windows production path, with orchestration, path, authorization, and controlled session behavior testable on CI hosts

**Project Type**: Rust workspace facade plus command-line binary

**Performance Goals**: At most one target-scoped store open, resolution, process snapshot, proposal, and bounded session per sequence step; no more than fourteen effectful attempts in the current protocol matrix

**Constraints**: Feature-branch pull-request workflow; UTF-8 without BOM; LF; 100-column Rust; no new dependency or lockfile package; no process handle; no blanket authorization; no hidden trust; no system proxy; no pinning bypass; no workflow persistence, non-Steam topology authoring, or parent completion claim

**Scale/Scope**: One guided command refactor, additive attempt progress fields, safe bundle-path derivation, focused controlled tests, public reference updates, two changelog fragments, and architecture-of-record updates

## Constitution Check

*GATE: Passed before research and passed again after design.*

- **P-1**: PASS. Every session remains explicit, target-scoped, separately plan-authorized, reversible, and auditable. No prior response authorizes a later session.
- **P-2**: PASS. S139 remains the facade policy authority. The CLI owns only in-process orchestration, presentation, authorization input routing, and bundle argument adaptation.
- **P-3**: PASS. The existing coordinator, proxy, capture, artifact, compatibility, and cleanup boundaries remain unchanged.
- **P-4**: PASS. Progression consumes current append-only facts and existing loss-accounted terminal observations. Missing or partial evidence stops rather than disappearing.
- **P-5**: PASS. Storage, bundle artifact, and manifest schemas are unchanged. Progress is transient and event-additive.
- **P-6**: PASS. Existing calibration phase, protocol, proposal, authorization-plan, observation, and fact vocabularies are reused.
- **P-7**: PASS. No shell wrapper or second process is added; the Rust command remains the orchestration boundary.
- **P-8**: PASS. TDD, controlled integration coverage, the foreground repository gate, and review convergence remain mandatory.
- **P-9**: PASS. Requested candidates, observed candidates, current facts, attempted cases, incomplete cases, and completed cases remain distinct.
- **P-10**: PASS. The command continues through the established target resolution and S139 proposal path rather than adding another entry point.
- **P-11**: PASS. Documentation will state the finite in-process boundary and keep persistence, resume, non-Steam authoring, and parent completion open.
- **Third-party obligations**: PASS. No dependency, Npcap artifact, package, license, or distribution change.

## Project Structure

### Documentation (this feature)

```text
specs/144-guided-calibration-sequence/
├── checklists/
├── contracts/calibrate-sequence-command.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/
├── src/
│   ├── commands/calibrate.rs
│   ├── events.rs
│   └── lib.rs
└── tests/
    ├── cli_calibrate.rs
    ├── cli_deep_capture.rs
    └── cli_reference.rs

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md

site/content/docs/reference/cli.mdx
```

**Structure Decision**: Keep the closed orchestration state and safe bundle derivation in `fragcap-cli::commands::calibrate`, directly above the existing low-level executor. Preserve S139 policy and current fact applicability in the facade. Extend the existing calibration guidance event rather than creating a persistent workflow schema or parsing presentation output.

## Phase 0: Research Decisions

### 2026-09-11: Preserve one authorization plan per attempt

One command may contain several sessions, but each session remains a complete S134 decision. A response is never cached or widened into sequence authority.

### 2026-09-11: Advance only from current positive facts

Successful process exit and terminal observations are insufficient. After every session, the command reopens the store and rebuilds S139; only disappearance of the attempted useful step under the exact current case proves progression.

### 2026-09-11: Bound by the closed case vocabulary and attempted identity

The loop tracks exact cases and refuses repetition. One reachability case plus thirteen concrete protocols yields the current maximum of fourteen sessions without a separate arbitrary iteration limit.

### 2026-09-11: Preserve explicit bundle compatibility

The first attempt keeps the operator's exact `--bundle` value. Later attempts derive deterministic sibling roots from fixed phase and protocol vocabulary. Existing empty-directory validation remains the overwrite authority.

### 2026-09-11: Keep cross-process continuation stateless

The invocation retains progress only in memory. Every safe stop emits current coverage and a durable command; persistent workflow identity, state migration, and resume remain later #380 work.

## Phase 1: Design

The command contract is defined in [contracts/calibrate-sequence-command.md](contracts/calibrate-sequence-command.md), transient values and transitions in [data-model.md](data-model.md), and executable validation in [quickstart.md](quickstart.md). No entity requires a storage or artifact migration and no dependency changes.

## Complexity Tracking

No constitution violation or exceptional complexity is required.
