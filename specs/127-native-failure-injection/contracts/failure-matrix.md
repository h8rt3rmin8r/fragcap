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
- production resource, state, or coordinator-effect inventory drift;
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

Portable tests execute controlled drivers through the production Deep Capture coordinator, journal transition rules, artifact and fact status types, event sink, cleanup sequence, and recovery planner. No row may satisfy the contract using a separate lifecycle state machine.

Each applicable row asserts terminal, artifact, fact, event, cleanup, journal, and recovery dispositions independently. The test output names the generated scenario identity on failure.
