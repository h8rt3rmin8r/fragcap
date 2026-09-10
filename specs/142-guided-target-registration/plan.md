# Implementation Plan: Guided Target Discovery and Registration

**Branch**: `codex/s142-guided-target-registration` | **Date**: 2026-09-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/142-guided-target-registration/spec.md`

## Summary

Extend `fragcap calibrate` with a stored-first resolution adapter that falls back to the existing S133 discovery composition only on a clean miss. One exact Steam application identifier or Unicode-aware case-insensitive candidate display name produces a complete domain-separated registration plan that binds the conserved discovery account and warnings. Human input defaults to decline; structured input must return the exact plan identifier. The command repeats discovery after confirmation, passes only the unchanged candidate to `register_candidate`, resolves the resulting row by durable identifier, and enters the unchanged S139-S141 proposal path. Because discovery candidates intentionally store no launch entries, a new row retains the existing missing-declaration limitation rather than promoting a hint into authority. Ambiguity, discovery loss, drift, and the separate later Deep Capture authorization remain explicit.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `clap`, `serde_json`, `blake3`, `subtle`, `fragcap` facade, `fragcap-targets`, and S139-S141 guided calibration stack

**Storage**: Existing per-user SQLite target store version 10; one idempotent target insert through `register_candidate`; no migration, new table, or workflow state

**Testing**: Pure selection and plan unit tests, CLI argument/help/event tests, controlled discovery integration tests, existing S140-S141 calibration suite, and `cargo xtask ci`

**Target Platform**: Windows production discovery and calibration path, with policy and controlled fixtures testable on all CI hosts

**Project Type**: Rust workspace facade plus command-line binary

**Performance Goals**: Stored resolution remains one store query path; fallback performs at most two bounded discovery passes; matching and canonicalization are linear in the bounded candidate set; one registration and one calibration session at most

**Constraints**: Feature-branch pull-request workflow; UTF-8 without BOM; LF; 100-column Rust; no new package or schema; no process handle; no hidden trust; no system proxy; no pinning bypass; no target TLS key extraction; no bulk automatic registration; no fabricated topology; no internal multi-attempt loop or workflow persistence

**Scale/Scope**: One additive target-resolution adapter, two additive events, one registration-plan model, a discovery fixture seam, focused tests, public reference updates, two changelog fragments, and architecture-of-record updates

## Constitution Check

*GATE: Passed before research and passed again after design.*

- **P-1**: PASS. Registration is local metadata. Any later Deep Capture effects remain separately visible, scoped, reversible, auditable, and plan-authorized. No target process access or prohibited technique is added.
- **P-2**: PASS. The orchestration stays in `fragcap-cli`; `fragcap-core` and dependency direction remain unchanged.
- **P-3**: PASS. Capture and attribution are untouched.
- **P-4**: PASS. Discovery accounts and warnings remain visible; ambiguity, drift, and every authorization outcome are explicit.
- **P-5**: PASS. Capture artifacts and analyzer compatibility are unchanged.
- **P-6**: PASS. Existing candidate, target, registration, plan, and calibration vocabulary is reused; no new domain term needs a glossary entry.
- **P-7**: PASS. No wrapper changes are required.
- **P-8**: PASS. TDD, focused tests, full repository verification, encoding checks, and review convergence remain mandatory.
- **P-9**: PASS. Exact discovery observations are never promoted to launch topology or compatibility facts, and incomplete discovery remains visible.
- **P-10**: PASS. Existing discovery sources and `register_candidate` remain the only production and persistence authorities.
- **P-11**: PASS. Architecture documents will describe only confirmation-gated registration and keep parent #380 open.
- **Third-party obligations**: PASS. No dependency, Npcap, license, redistribution, or packaging change.

## Project Structure

### Documentation (this feature)

```text
specs/142-guided-target-registration/
├── checklists/
├── contracts/calibrate-registration-command.md
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
│   ├── commands/deep_capture.rs
│   ├── commands/targets.rs
│   ├── events.rs
│   └── lib.rs
└── tests/
    ├── cli_calibrate.rs
    ├── cli_help.rs
    └── cli_reference.rs

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md
```

**Structure Decision**: Keep stored/discovered orchestration, plan presentation, and sequential input handling in `fragcap-cli`. Expose the already shared discovery composition to its sibling command module, inject a discovery provider only for controlled tests, and continue using facade-exported `register_candidate`. Do not add selection or persistence logic to `fragcap-core`, and do not duplicate candidate-to-entry conversion.

## Phase 0: Research Decisions

### 2026-09-10: Stored resolution has absolute precedence

A resolved or ambiguous stored result is final. Discovery runs only for `Selection::NoMatch`, preserving row semantics and avoiding a discovered candidate shadowing a curated target.

### 2026-09-10: Match only exact observed namespaces

An exact Steam application identifier or Unicode-aware case-insensitive display name is sufficient to offer a choice. Predicted handles, folder names, executable hints, path fragments, and fuzzy matches are excluded because they are not the namespace the operator supplied.

### 2026-09-10: Bind registration like an authorization plan without merging consent

The plan uses deterministic canonical JSON, BLAKE3 domain separation, and exact structured comparison. It binds the complete candidate together with every conserved discovery count and warning, preventing an incomplete scan from disappearing on the successful path. Human confirmation remains yes/no because the complete plan is already displayed. Its identifier and response are independent from the later S134 Deep Capture plan and response.

### 2026-09-10: Repeat discovery before persistence

The first result is a preview. The second result must select exactly one byte-equivalent canonical candidate from the same effective destination. A moved, renamed, reclassified, or re-evidenced candidate is drift and must be reviewed anew.

### 2026-09-10: Reuse the single registration operation

`register_candidate` already owns deterministic anchors, handles, entry conversion, evidence storage, and idempotency. Explicit confirmation permits one selected candidate even when S133 correctly withholds it from automatic persistence.

### 2026-09-10: Continue by durable identity

After registration, the inserted or exact already-present row is resolved from its anchor or install root, and `calibrate` enters the existing function with an `--id`-equivalent target. Discovery hints do not manufacture launch entries, so unsupported path candidates may reach an existing typed topology limitation.

## Phase 1: Design

The command contract is defined in [contracts/calibrate-registration-command.md](contracts/calibrate-registration-command.md), transient state in [data-model.md](data-model.md), and executable validation in [quickstart.md](quickstart.md). No entity requires a storage migration or dependency change.

## Complexity Tracking

No constitution violation or exceptional complexity is required.
