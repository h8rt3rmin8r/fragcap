# Implementation Plan: Guided Calibration Acceptance

**Branch**: `codex/s147-guided-calibration-acceptance` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/147-guided-calibration-acceptance/spec.md`

## Summary

Close the guided-calibration parent contract with a versioned registry that maps all thirteen issue #380 criteria to current executable controlled tests. Add an xtask validator to reject evidence drift, wire it into CI, fill only genuine controlled-test gaps, and correct the testing strategy so real-game compatibility is operator-owned and published-release-only.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `serde_json` in `xtask`; no product dependency change

**Storage**: One repository JSON acceptance registry plus additive target-store schema version 13 for the expanded no-effect pause vocabulary

**Testing**: Validator unit tests, existing and focused controlled CLI/store tests, and `cargo xtask ci`

**Target Platform**: Cross-platform controlled tests plus ordinary Windows CI; no live game or physical trust effects

**Project Type**: Multi-crate Rust CLI and libraries with repository task runner

**Performance Goals**: Registry validation completes in one source scan and adds negligible CI time

**Constraints**: No real game, game account, remote game service, capture driver, real trust-store mutation, process control, sensitive live capture, new dependency, or false compatibility claim

**Scale/Scope**: Thirteen parent criteria, one registry, one validator command, CI wiring, two missing pause values, one confirmation-gated non-Steam stored-client ambiguity path, focused default-inference and plan-visibility assertions, and testing-policy documentation

## Constitution Check

*GATE: Passed before Phase 0 research and rechecked after Phase 1 design.*

- **P-1 and authorized use**: The slice adds only controlled evidence and repository validation. It acquires no process, trust, network, or capture authority.
- **P-2 and P-3**: Product crate boundaries remain unchanged. Acceptance validation belongs to `xtask` and references public behavior through existing tests.
- **P-4 and P-9**: Every incomplete, partial, ambiguous, stale, negative, refused, and interrupted state remains explicit. Missing live evidence is stated, not inferred away.
- **P-5**: Product artifact schemas are unchanged; the new repository registry has its own version, and target-store schema version 13 additively widens only the closed no-effect pause vocabulary.
- **P-6**: Implementation acceptance, compatibility evidence, authorization, effects, and general Deep Capture completion remain distinct terms.
- **P-7**: One registry and one validator replace ad hoc completion prose. No duplicate product orchestration path is added.
- **P-8**: Validator mutation tests lead implementation, and the complete repository gate remains mandatory.
- **P-10**: No alternate capture or proxy implementation appears.
- **P-11**: Architecture, testing strategy, and changelog state both the proved boundary and the unproved live-game boundary.

## Project Structure

### Documentation

```text
specs/147-guided-calibration-acceptance/
├── checklists/
│   ├── requirements.md
│   └── security.md
├── contracts/
│   └── guided-calibration-acceptance.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md

integration/guided-calibration-acceptance-v1.json
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
```

### Source Code

```text
xtask/src/guided_calibration_acceptance.rs
xtask/src/main.rs
crates/fragcap-targets/src/schema.rs
crates/fragcap-targets/src/store.rs
crates/fragcap-targets/src/workflow.rs
crates/fragcap-cli/src/commands/calibrate.rs
crates/fragcap-cli/src/events.rs
crates/fragcap-cli/tests/cli_calibrate.rs
AGENTS.md
changelog.d/
```

**Structure Decision**: Acceptance metadata belongs beside other versioned integration authorities. `xtask` owns validation, existing product tests own behavior, and documentation owns the release-only manual boundary.

## Complexity Tracking

No constitutional violation requires justification.
