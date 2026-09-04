# Native Failure Matrix Contract

## Command

```text
cargo xtask failure-matrix
```

The command exits `0` when the registry, generated matrix, production inventory, and executable evidence agree; `1` for validation findings; and `2` when the check cannot run.

## Registry contract

The canonical registry is `docs/security/deep-capture-failures.v1.json`. Schema version 1 contains closed effect and lifecycle boundary inventories, ten mandatory failure families, seven outcome dimensions, per-side controlled drivers, expected cleanup and recovery, and attributed Rust test references.

The command rejects:

- an unsupported schema or missing review identity;
- duplicate, empty, or malformed boundary identities;
- a missing before or after driver;
- an unknown failure family, outcome value, resource kind, or lifecycle state;
- a missing outcome dimension;
- production resource, lifecycle-state, executable lifecycle-edge, or coordinator-effect inventory drift;
- a missing, ignored, conditional, untracked, or unattributed Rust test;
- a mandatory failure family with no executable generated cell.

## Generated matrix contract

For each boundary, generation produces these cells in stable boundary then side order:

```text
<boundary-id>:before
<boundary-id>:after
```

No third side is valid. Stored duplicate expanded rows are impossible because expanded rows are not hand-authored.

## Execution contract

Portable tests execute controlled drivers through the production Deep Capture coordinator, its exact boundary-controller calls, journal transition rules, artifact and fact status types, event sink, cleanup sequence, and recovery planner. Production installs only the allow controller and exposes no runtime fault-selection input. The coordinator checks every state change against the edge inventory consumed by the gate and retains each traversed edge in terminal truth. No row may satisfy the contract using a separate lifecycle state machine.

Each row reads and asserts its declared terminal, artifact, fact, event, cleanup, journal, and recovery dispositions independently. An independent fact disposition requires at least one evidence-backed successful append. Effect rows prove before or after placement from both the production resource journal and actual adapter calls; transition rows prove their named edge appears in the ordered terminal trace. The test output names the generated scenario identity on failure.
