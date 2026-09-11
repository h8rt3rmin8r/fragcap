# Research: Guided Calibration Acceptance

## Decision 1: Close the parent with a checked acceptance registry

**Decision**: Add one versioned JSON registry whose thirteen stable criterion identifiers map to exact Rust test functions, then validate it through `cargo xtask guided-calibration-acceptance` and the full CI command.

**Rationale**: S140 through S146 spread the parent behavior across proposal, front-door, registration, topology, sequencing, resume, and explicit-choice boundaries. A checked registry makes that accumulated proof inspectable and rejects evidence drift after #380 closes.

**Alternatives considered**:

- A Markdown completion checklist was rejected because it cannot prove that named tests exist or execute.
- One giant duplicate end-to-end test was rejected because it would repeat the mature controlled harness and obscure which authority proves each criterion.
- Treating historical slice specifications as evidence was rejected because specifications do not execute.

## Decision 2: Reuse exact tests and add only missing propositions

**Decision**: Map each criterion to the smallest existing tests that directly prove it, and add focused controlled tests only where the audit finds a proposition without executable coverage.

**Rationale**: Existing tests already drive the production CLI, facade, stores, plans, and effect-recording authorization adapters. Reusing them avoids a second orchestration path and keeps changes proportional.

**Alternatives considered**:

- Snapshotting only command output was rejected because output alone cannot prove store, effect, history, or cleanup behavior.
- Adding a second synthetic calibration executable was rejected because the existing controlled adapters already expose the necessary seams.

## Decision 3: Validate evidence references structurally

**Decision**: The validator accepts only confined, Git-tracked Rust paths and exact functions carrying an unconditional `#[test]` or `#[tokio::test]` attribute. It rejects ignored, conditionally disabled, missing, duplicate, or malformed references.

**Rationale**: A function name in JSON is useful only if it remains an ordinary executable test. The repository's threat-model gate already establishes this parser and validation pattern.

**Alternatives considered**:

- Executing each reference separately was rejected because `cargo test --workspace --locked` already executes the complete set and per-reference process startup would substantially slow CI.
- Accepting ignored Windows physical tests was rejected because S147's implementation gate must complete without sensitive or host-specific effects.

## Decision 4: Correct the real-game validation timing

**Decision**: Update the master testing strategy so real-game validation is operator-owned and may occur only against a published release. S147 records explicitly that no real-game compatibility was demonstrated.

**Rationale**: The previous statement that Tier 3 ran before release conflicts with the operator's explicit safety and release requirement. Controlled tests remain the reproducible implementation gate. A real game run is compatibility evidence for published bytes, not permission for an agent to run sensitive software.

**Alternatives considered**:

- Pretending approval or leaving the pre-release wording in place was rejected as false evidence.
- Blocking all development until a release was rejected because it would make an unreleased implementation depend on a test the operator will only perform after publication.

## Decision 5: Keep general Deep Capture completion separate

**Decision**: S147 closes only #380. It does not close or weaken the native feature-completion gate in #334 and adds no runtime, dependency, network, trust, or artifact-schema change. The audit-authorized behavior corrections are limited to one additive pause-vocabulary migration and one confirmation-gated stored-client ambiguity path.

**Rationale**: Guided calibration is one operator workflow. General native Deep Capture completion has broader release, security, documentation, and verification authorities.

## Audit findings

- The durable pause vocabulary covers login, EULA, gameplay, shutdown, and interruption, but issue #380 also names update and anti-cheat action. S147 must add those two no-effect reasons and test their persistence.
- The combined direct, Steam, and publisher CLI test supplies `--launch-case`, so it proves override propagation but not automatic topology defaults. S147 must exercise those topologies without the override and assert the inferred dimensions.
- Existing guided tests count authorization plans but do not directly assert the complete visible target, route, deadline, artifact, trust, and cleanup fields. S147 must add focused assertions against the emitted plan.
- Issue #380 requires ambiguity resolution for a stored non-Steam target, while S146 covers only target discovery and Steam metadata. S147 must allow an exact content-derived executable choice only for client-only ambiguous launch declarations, bind the complete rewrite to a separate confirmation plan, revalidate it after confirmation, and leave Steam and publisher chains untouched.
- Existing `#[cfg(windows)]`, non-ignored controlled CLI tests provide the Windows integration half of the parent criterion. The retained S129 physical matrix must not be rewritten because doing so would invalidate its evidence digest without an authorized physical rerun.
