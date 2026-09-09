# Implementation Plan: Guided Calibration Case Proposals

**Branch**: `codex/s139-calibration-case-proposals` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/139-calibration-case-proposals/spec.md`

## Summary

Implement the first bounded child of #380 as a pure facade policy that converts an already resolved target, a caller-supplied process snapshot, exact S121 context, observed S120 protocol candidates, and retained facts into one deterministic calibration proposal. The proposal owns structural topology discovery, warm-state guidance, conservative route and family defaults, exact evidence assessment, reachability-first ordering, and stable reasons. It is exported through the S132 stable facade for the later guided CLI, but this slice adds no command or effect adapter.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing workspace crates only (`fragcap-targets`, `serde_json` transitively through stored targets); no new dependency or lockfile package

**Storage**: Read-only consumption of existing target entries and compatibility facts; no schema or write path

**Testing**: Rust unit and stable-API inventory tests, plus full `cargo xtask ci`

**Target Platform**: Platform-neutral proposal policy over Windows launch declarations and query-only process-image facts; no platform API in the new module

**Project Type**: Rust workspace facade library with a future CLI consumer

**Performance Goals**: Linear work in one target's bounded launch declarations, observed image list, protocol candidates, and compatibility history; no packet-path work

**Constraints**: Pure and deterministic, complete-snapshot cold proof, exact S121 applicability, no target resolution, registration, process enumeration, process control, listener, trust, artifact, fact, store, or CLI effect

**Scale/Scope**: Three shipped topology classes, three cold launch cases, three warm launch cases, one implemented routing strategy, two loopback families, thirteen concrete S120 protocol candidates, and one target's local fact history

## Constitution Check

- **P-1, no covert target instrumentation**: PASS. The new policy accepts caller values and has no adapter or platform API. Warm state produces guidance only, and no process, route, trust, or capture effect is possible.
- **P-2 and P-3, architecture boundaries**: PASS. The policy lives in the facade above target vocabulary and does not touch packet acquisition or attribution. `fragcap-core` stays unchanged.
- **P-4 and P-9, loss and truth**: PASS. Missing authority, ambiguity, stale or conflicting evidence, and deferred protocols remain explicit. No observation is changed or suppressed into a positive claim.
- **P-5, compatibility**: PASS. Capture and artifact formats do not change.
- **P-6, glossary**: PASS. The existing compatibility-calibration entry will define the proposal boundary in the same change.
- **P-7 and P-8, thin wrappers and standards**: PASS. No wrapper changes; repository formatting and lint gates apply.
- **P-10, one target path**: PASS. The policy consumes the existing `TargetEntry` and `CompatibilityFact` forms and creates no parallel store or selector.
- **P-11, specification truth**: PASS. The master specification, outline, ordering record, and agent guide will record only the shipped proposal authority and leave #380 open.

Post-design recheck: PASS. The stable facade contract remains pure, uses existing authorities, and introduces no constitution exception.

## Project Structure

### Documentation (this feature)

```text
specs/139-calibration-case-proposals/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- checklists/
|   |-- requirements.md
|   `-- safety.md
|-- contracts/
|   `-- proposal-api.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/deep_capture/
|-- api.rs
|-- mod.rs
`-- proposal.rs

crates/fragcap/src/
`-- lib.rs

docs/
|-- fragcap-spec-outline.md
|-- fragcap-specification.md
|-- glossary/
|   `-- capture-and-networking.md
`-- plans/
    `-- README.md

AGENTS.md
changelog.d/
`-- S139-calibration-case-proposals.changed.md
```

**Structure Decision**: The facade owns proposal policy because it is the existing layer above target topology, compatibility facts, launch ownership, and session calibration. `fragcap-targets` remains the vocabulary and persistence authority, while the future CLI remains a presentation and adapter layer. The new contract is added to `deep_capture::api` so the later #380 slice and non-CLI consumers share one policy.

## Implementation Phases

1. Add failing unit tests for strict topology discovery, complete-snapshot cold proof, warm-state classification, conservative defaults, and invalid inputs.
2. Implement the typed request, topology, readiness, limitation, reason, step, deferred-protocol, and proposal models plus the pure builder.
3. Add failing evidence-history permutation tests, then implement exact routing and inspectability assessment with conflict detection and reachability-first sequencing.
4. Export the contract through the stable facade, extend its reviewed inventory, and add compile-time consumer coverage.
5. Update the architecture records and glossary, run analyze and converge, complete all tasks, then run the repository CI-parity gate.

## Decision Log

### 2026-09-09: Split #380 at the pure proposal boundary

S139 implements only case discovery and proposal policy. The public guided command, target registration, workflow state machine, execution, persistence, resume, summaries, and handoff remain in #380. Implementing the XL issue in one slice was rejected because it would couple read-only policy review to trust-bearing and storage effects and would make failures harder to isolate.

### 2026-09-09: Consume a caller-supplied process snapshot

The policy accepts a complete list of present process image names or an explicit unavailable state. It does not enumerate the host. This makes cold-state truth testable and reusable while ensuring absence is authoritative only when the caller declares the snapshot complete. Moving Toolhelp enumeration into the policy was rejected because it would mix platform I/O with the decision and make unavailable state harder to represent.

### 2026-09-09: Strictly validate declared topology without filesystem preparation

Steam requires a numeric `steam:` anchor and one exact client declaration. Direct requires one client declaration. Publisher chains retain declared order and require at least launcher plus client, a non-empty unique role per stage, `launcher` first, `client` last, and no image with conflicting roles. Filesystem existence and canonical path validation remain the managed-launch preparation authority immediately before effects. Running that preparation during proposal was rejected because proposal must be read-only and useful before a potentially unavailable install root is mounted.

### 2026-09-09: Treat conflicting current facts as retest-worthy

One current exact positive value suppresses work only when every current exact row for that fact class agrees. Different values remain a conflict even when a durable row order identifies a latest value. S121's latest-row rule continues to govern ordinary eligibility; the guided proposal is intentionally more conservative because its purpose is to recommend useful verification, not authorize a session.

### 2026-09-09: Propose observed protocol candidates only after routing

Metadata never predicts game traffic. Callers may supply concrete S120 protocol candidates derived from retained classifications or an advanced exact selection. Until current unconflicted final-client routing exists, those candidates are preserved as deferred and only one routing reachability attempt is runnable. Automatically proposing HTTPS for every game was rejected as an invented traffic claim.

### 2026-09-09: Add the proposal to the stable facade now

The later guided CLI and library callers need the same decision authority. Exporting the pure typed contract through `deep_capture::api` now lets inventory tests review the compatibility surface and avoids a CLI-private policy that must later be promoted or duplicated.

## Deviations

This slice deliberately deviates from treating issue #380 as one implementation unit. Issue #380 is labeled `effort: xl` and explicitly spans target registration, interactive orchestration, effects, persistence, interruption, resume, and final handoff. S139 is its first atomic child (#392) and does not close the parent. The split preserves the recommended read-only case-discovery boundary and reduces security and review coupling.
