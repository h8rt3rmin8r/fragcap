# Quickstart: Guided Target Discovery and Registration

## Focused Validation

```powershell
cargo test -p fragcap-cli --test cli_calibrate --locked
cargo test -p fragcap-cli --test cli_help --locked
cargo test -p fragcap-cli --test cli_reference --locked
cargo test -p fragcap-cli --lib events --locked
```

Confirm stored precedence, exact Steam-id and display-name matching, explicit ambiguity, discovery warnings, no-match behavior, complete plan rendering, human decline, exact structured input, identifier sensitivity, revalidation drift, idempotent registration, durable-id handoff, unresolved-topology no-effect honesty, separate later session authorization, and unchanged S140-S141 behavior.

## Repository Gate

```powershell
cargo xtask ci
```

## Safety Inspection

- No target row is inserted before confirmation.
- Only one selected unchanged candidate reaches `register_candidate`.
- Discovery evidence never becomes launch topology or compatibility evidence.
- Registration authorization never authorizes trust, proxy, launch, capture, artifact, or fact effects.
- Stored ambiguity, discovery ambiguity, incomplete discovery, and drift remain distinct.
- No process handle, process control, hidden trust, system proxy, pinning bypass, target key extraction, workflow storage, or second calibration session is added.
- Issue #398 closes on merge while parent #380 remains open.
