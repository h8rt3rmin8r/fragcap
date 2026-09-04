# Implementation Plan: Native Deep Capture Threat Model

**Branch**: `codex/125-native-threat-model` | **Date**: 2026-09-04 | **Spec**: `specs/125-native-threat-model/spec.md`

## Summary

Create one versioned native Deep Capture threat registry, validate its complete
control and executable-evidence ownership in the repository task runner, bind
review currency to shipped protocol families and direct proxy dependencies,
and close focused negative-test gaps without changing production behavior.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library and serde_json in xtask; no new package

**Storage**: Versioned JSON threat registry and Markdown security guidance

**Testing**: Validator unit tests, existing proxy/facade/CLI negative tests, focused abuse cases, and full xtask CI

**Target Platform**: Portable validation over the Windows native Deep Capture product model

**Project Type**: Rust workspace with a repository-owned security gate

**Performance Goals**: One bounded read of the registry, tracked Rust sources, protocol contract, and proxy manifest

**Constraints**: Offline, deterministic, no runtime effects, no new dependency, no inferred risk acceptance, no completion claim

**Scale/Scope**: Issue #323 only; review of the native attack surface shipped through S124

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Every routing and protocol family is represented by threats
  that prohibit target memory access, injection, hooks, system proxy mutation,
  target key extraction, and target process handles.
- **P-2/P-3**: PASS. The product architecture remains unchanged. The new gate
  belongs to xtask and consumes public source and manifest authorities.
- **P-4/P-9**: PASS. Refusals, loss, saturation, ambiguity, and cleanup
  interruption have named evidence and executable negative ownership.
- **P-5**: PASS. Packet truth and pcapng are unchanged.
- **P-6/P-8**: PASS. The model, glossary, specifications, and CI gate evolve
  together as one reviewed contract.
- **P-10/P-11**: PASS. Validation is offline and side-effect free, and S125 does
  not absorb later milestone work or claim final completion.

Post-design check: PASS. The registry is data, validation is deterministic, and
protocol/dependency review drift fails closed without changing runtime paths.

## Architecture and Phases

1. Publish the canonical threat registry and human-readable model with stable
   boundary, asset, threat, control, evidence, and review inventories.
2. Add an xtask validator with pure validation functions and seeded rejection
   tests for incomplete rows, evidence drift, and review-inventory drift.
3. Bind executable references to tracked, non-ignored Rust test functions and
   bind protocol/dependency inventories to their source authorities.
4. Add any focused negative test needed for a materially distinct shipped
   abuse path not already demonstrated.
5. Wire the gate into ordinary CI and update architecture, glossary, planning,
   and changelog records.

## Project Structure

```text
specs/125-native-threat-model/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
docs/security/
├── deep-capture-threat-model.md
└── deep-capture-threats.v1.json
xtask/src/
├── main.rs
└── threat_model.rs
crates/fragcap-proxy/tests/
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/glossary/capture-and-networking.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Keep the registry under `docs/security` because it is
both review material and the task runner's canonical input. Keep validation in
xtask so the product dependency graph and runtime stay unchanged.

## Complexity Tracking

No constitution violation requires an exception.
