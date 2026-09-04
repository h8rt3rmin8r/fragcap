# Implementation Plan: Native Deep Capture Failure Injection

**Branch**: `codex/127-native-failure-injection` | **Date**: 2026-09-04 | **Spec**: `specs/127-native-failure-injection/spec.md`

**Input**: Feature specification from `specs/127-native-failure-injection/spec.md`

## Summary

Publish a closed failure-boundary registry, generate both sides of every owned effect and lifecycle boundary, execute deterministic failures through the production coordinator and journal authorities, and make ordinary CI reject missing scenarios, stale tests, incomplete outcome vectors, or production inventory drift.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88, pinned product toolchain 1.96.0

**Primary Dependencies**: Existing workspace graph only, including serde_json already used by the facade and task runner

**Storage**: Versioned JSON failure registry plus existing resource journal and lifecycle JSON Lines authorities

**Testing**: Parameterized facade integration tests, journal recovery tests, task-runner validator tests, full `cargo xtask ci`

**Target Platform**: Portable deterministic tests representing Windows effects through existing narrow adapters; shipped product remains Windows 11 x86-64

**Project Type**: Rust workspace and CLI

**Performance Goals**: Generate and validate the complete matrix in under one second; execute the portable matrix within the ordinary test budget

**Constraints**: No destructive host mutation, external network, real trust change, game, account, capture driver, new dependency, hidden cleanup omission, or inferred success

**Scale/Scope**: Seven journaled effects, eight checked lifecycle transitions including failure-shortened paths, two injection sides per boundary, ten mandatory failure families, and seven independent outcome authorities

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Controlled adapters simulate host failures without target access, real trust mutation, system proxy changes, interception, or external traffic.
- **P-2/P-3**: PASS. The coordinator and journal remain in the facade; packet acquisition and attribution are unchanged.
- **P-4/P-9**: PASS. Each failure row carries explicit loss, artifact, fact, event, cleanup, journal, and recovery expectations. Uncertainty never normalizes to success.
- **P-5**: PASS. Capture and artifact format compatibility are unchanged.
- **P-6/P-8**: PASS. New matrix vocabulary is defined with the implementation, and all process artifacts follow repository mechanics.
- **P-10/P-11**: PASS. Target resolution is unchanged, while the specification and roadmap record only S127 and preserve the #334 completion gate.

Post-design check: PASS. The validator derives production resource and lifecycle inventories from their owning Rust sources, the executable matrix uses the real public coordinator and recovery planner, and no new effect path or dependency is introduced.

## Architecture and Phases

1. Define a versioned registry of journaled effects, lifecycle transitions, failure families, expected outcome dimensions, and executable evidence.
2. Add a task-runner validator that generates both injection sides, cross-checks production enums and coordinator resource calls, and validates attributed test references.
3. Extend the controlled facade harness with missing route, post-effect, timeout, cancellation, event, artifact, fact, and cleanup failure cases.
4. Execute the generated cases against production coordinator reports and resource-journal recovery decisions.
5. Wire the matrix gate into ordinary CI, publish the failure-injection review guide, and update architectural records.
6. Run focused tests, analyze implementation convergence, then run the complete repository gate.

## Project Structure

```text
specs/127-native-failure-injection/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
docs/security/
├── deep-capture-failures.v1.json
└── deep-capture-failure-injection.md
crates/fragcap/tests/
└── deep_capture_session.rs  # Existing controlled adapters plus generated matrix
xtask/src/
├── failure_matrix.rs
└── main.rs
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
AGENTS.md
changelog.d/
```

**Structure Decision**: Keep injected effect behavior and the generated matrix beside the existing facade controlled-adapter harness, keep durable recovery assertions on the resource journal, and keep exhaustive inventory generation in the repository task runner. Co-location avoids a duplicate lifecycle scaffold. No product-only fault switch or parallel lifecycle implementation is added.

## Complexity Tracking

No constitution violation requires an exception.
