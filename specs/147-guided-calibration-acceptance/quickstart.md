# Quickstart: Guided Calibration Acceptance

## Scope

This quickstart validates S147 without launching a real game, contacting a game service, mutating the real trust store, or running sensitive live capture.

## Focused acceptance gate

```powershell
cargo xtask guided-calibration-acceptance
```

Expected result: schema version 1, thirteen criteria, and every referenced test accepted.

## Controlled guided-calibration tests

```powershell
cargo test -p fragcap-cli --test cli_calibrate --locked
```

Expected result: all controlled command tests pass with temporary stores, synthetic state, and effect-recording adapters.

## Complete repository gate

```powershell
cargo xtask ci
```

Expected result: formatting, linting, all workspace tests, repository policy gates, registry validation, and existing controlled native authorities pass.

## Explicit non-result

These commands do not demonstrate compatibility with a real game. Any future real-game validation belongs to the operator and must use a published release.
