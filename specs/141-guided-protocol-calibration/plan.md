# Implementation Plan: Guided Protocol Calibration Attempt

**Branch**: `codex/s141-guided-protocol-calibration` | **Date**: 2026-09-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/141-guided-protocol-calibration/spec.md`

## Summary

Extend S140's registered-target calibration front door with repeatable concrete protocol candidates and one plan-bound protocol attempt per invocation. The CLI normalizes requested candidates, supplies them to S139, preserves reachability-first ordering, and adapts only the first proposed step into the existing low-level executor. The executor returns a crate-private terminal observation projection so the guided layer can derive a bounded final-client candidate set through a facade-owned policy helper, re-read append-only facts, and emit exact coverage plus a parseable continuation command. No workflow state, schema, dependency, or second session is added.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap` facade, `fragcap-targets`, and native Deep Capture stack already present in the workspace

**Storage**: Existing per-user SQLite target store version 10; read current target facts and append only through the existing session fact adapter; no migration or workflow record

**Testing**: Facade policy unit tests, focused CLI unit and integration tests, controlled Deep Capture harness, stable event serialization and help tests, and `cargo xtask ci`

**Target Platform**: Windows production path, with policy, argument, no-effect, and controlled protocol behavior testable on CI hosts

**Project Type**: Rust workspace facade plus command-line binary

**Performance Goals**: One target resolution and proposal evaluation before effects, one authorized session at most, one target-scoped fact re-read after execution, and bounded linear observation classification

**Constraints**: Feature-branch pull-request workflow; UTF-8 without BOM; LF; 100-column Rust; no new dependency or lockfile package; no process handle; no hidden trust; no system proxy; no pinning bypass; no target TLS key extraction; no registration, internal multi-attempt loop, workflow persistence, or completion claim

**Scale/Scope**: One additive top-level option, one expanded guided event, one facade policy helper, one crate-private executor outcome, focused controlled tests, two changelog fragments, and architecture-of-record updates

## Constitution Check

*GATE: Passed before research and passed again after design.*

- **P-1**: PASS. All effects remain target-scoped, visible, reversible, auditable, and behind the exact S134 plan identifier. Candidate input cannot authorize trust or become evidence.
- **P-2**: PASS. Classification and evidence policy stay in the facade while process inventory and CLI mapping stay in `fragcap-cli`; `fragcap-core` remains platform-independent.
- **P-3**: PASS. The guided layer delegates to the existing coordinator and changes neither proxy, capture, nor attribution backend boundaries.
- **P-4**: PASS. Existing bounded observation and loss accounting remain authoritative. Candidate derivation consumes the terminal retained set and never hides loss.
- **P-5**: PASS. Artifact and storage schemas are unchanged.
- **P-6**: PASS. Existing protocol, classification, calibration, and evidence vocabulary is reused consistently.
- **P-7**: PASS. No shell wrapper is added; the Rust command remains a thin policy adapter.
- **P-8**: PASS. TDD, focused tests, the repository gate, and review convergence remain mandatory.
- **P-9**: PASS. Requests, observations, facts, unavailable coverage, and missing observations remain distinct.
- **P-10**: PASS. Stored-target selection is reused and no registration path is added.
- **P-11**: PASS. The architecture record will describe only one protocol attempt and keep #380 open.
- **Third-party obligations**: PASS. No dependency, Npcap artifact, packaging, or license change.

## Project Structure

### Documentation (this feature)

```text
specs/141-guided-protocol-calibration/
├── checklists/
├── contracts/calibrate-protocol-command.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/
└── src/deep_capture/
    ├── api.rs
    └── policy.rs

crates/fragcap-cli/
├── src/
│   ├── cli.rs
│   ├── commands/calibrate.rs
│   ├── commands/deep_capture.rs
│   └── events.rs
└── tests/
    ├── cli_args.rs
    ├── cli_calibrate.rs
    ├── cli_deep_capture.rs
    ├── cli_help.rs
    └── cli_reference.rs

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md
```

**Structure Decision**: Keep candidate mapping and final-client eligibility in the `fragcap` facade beside compatibility-fact derivation so every consumer shares one evidence boundary. Keep candidate parsing, S139 request construction, one-step selection, coverage projection, and next-command text in `fragcap-cli`. Refactor the existing low-level executor only enough to return a crate-private terminal observation projection to the guided caller while retaining the current public `run` result and all low-level behavior.

## Phase 0: Research Decisions

### 2026-09-10: Treat candidate input as an attempt request

The candidate list narrows which exact protocol facts S139 may propose. It never proves protocol use, inspectability, or trust. Existing fact derivation remains the only durable evidence path.

### 2026-09-10: Derive automatic candidates in facade policy

The mapping from S120 traffic families to compatibility protocol dimensions already belongs beside fact eligibility and final-client correlation. A public pure helper can return sorted deduplicated eligible candidates without exposing CLI policy or duplicating the mapping.

### 2026-09-10: Return terminal observations without parsing emitted JSON

The low-level executor already owns the authoritative `TerminalReport`. A crate-private outcome preserves that typed result for the guided caller while the current `run` wrapper retains its existing signature. Parsing presentation events would turn a stable internal handoff into a lossy string protocol.

### 2026-09-10: Keep one session per process invocation

S139 may propose several protocol steps, but the guided command selects only the first. Remaining candidates are carried in a parseable continuation. This keeps every trust-bearing attempt separately visible and authorized.

### 2026-09-10: Separate routing readiness from protocol coverage

A target with current routing evidence and no eligible candidate remains ready for ordinary Deep Capture, but its guided protocol coverage is unknown. The command reports both truths and starts no speculative session.

### 2026-09-10: Reassess facts after every session

The terminal observations identify eligible next candidates, but completion comes only from a fresh target-scoped fact read and S139 proposal. This prevents an event or request from bypassing S121 applicability.

## Phase 1: Design

The command contract is defined in [contracts/calibrate-protocol-command.md](contracts/calibrate-protocol-command.md), transient state in [data-model.md](data-model.md), and executable validation in [quickstart.md](quickstart.md). No entity requires a storage migration and no dependency changes.

## Complexity Tracking

No constitution violation or exceptional complexity is required.
