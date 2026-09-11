# Quickstart: Bounded Guided Calibration Sequence

## Focused Validation

```powershell
cargo test -p fragcap-cli --lib --locked commands::calibrate
cargo test -p fragcap-cli --test cli_calibrate --locked
cargo test -p fragcap-cli --test cli_deep_capture --locked
cargo test -p fragcap-cli --test cli_reference --locked
```

Confirm reachability-to-protocol progression, multiple requested protocols, observed candidate growth, one confirmation per attempt, current fact gating, decline and invalid-input stops, terminal failure, no-progress protection, fourteen-attempt bound, explicit sibling bundle paths, default unique bundles, additive event fields, and parseable terminal commands.

## Repository Gate

```powershell
cargo xtask ci
```

## Safety Inspection

- Every effectful attempt emits and consumes its own complete S134 plan.
- Target, process, fact, and proposal authority are refreshed before every attempt.
- Exit zero and terminal observations never substitute for a current positive fact.
- No exact case executes twice and no invocation can exceed the closed supported bound.
- Completed bundle roots are never reused, deleted, or overwritten.
- Requested candidates, observed candidates, and compatibility facts remain distinct.
- No process control, blanket trust, system proxy, pinning bypass, persistent workflow, or parent completion claim is added.
- Issue #402 closes on merge while parent #380 remains open.
