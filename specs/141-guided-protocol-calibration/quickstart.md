# Quickstart: Guided Protocol Calibration Attempt

## Focused Validation

```powershell
cargo test -p fragcap --lib --features deep-capture deep_capture::policy --locked
cargo test -p fragcap-cli --test cli_args --locked
cargo test -p fragcap-cli --test cli_help --locked
cargo test -p fragcap-cli --test cli_calibrate --locked
cargo test -p fragcap-cli --test cli_deep_capture --locked
```

Confirm reachability precedence, protocol normalization, one-attempt selection, direct final-client candidate derivation, launcher and unknown exclusion, no-candidate coverage, already-positive suppression, warm preservation, authorization refusal, successful evidence, missing observation, stable structured output, and parseable continuation commands.

## Repository Gate

```powershell
cargo xtask ci
```

## Safety Inspection

- No requested candidate becomes evidence without a direct observation.
- No launcher, intermediate, unknown, unrouted, ambiguous, unavailable, or uncorrelated observation becomes a candidate.
- No invocation starts more than one session.
- Every trust-bearing attempt retains exact plan authorization, current target authority, bounded effects, and cleanup.
- No hidden or system trust, pinning bypass, target process access, target key extraction, workflow persistence, or second capture is added.
- Issue #396 closes on merge while parent #380 remains open.
