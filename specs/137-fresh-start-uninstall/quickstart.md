# Quickstart: Verify Explicit Fresh-Start Uninstall

## Preserve by default

```powershell
cargo test -p fragcap-cli --test cli_fresh_start preserve
```

Confirm ordinary command parsing and installer conditions do not authorize cleanup without both explicit values.

## Preview an isolated current-user inventory

```powershell
cargo test -p fragcap-cli --test cli_fresh_start preview
```

Confirm the preview lists both exact roots and every category without mutation.

## Verify containment and recovery truth

```powershell
cargo test -p fragcap-cli --test cli_fresh_start containment
cargo test -p fragcap-cli --test cli_fresh_start recovery
```

Confirm custom paths and redirections remain untouched, changed inventories refuse, and failed Deep Capture recovery retains evidence with partial status.

## Run full repository parity

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

The Windows release job additionally runs the effectful package certification lifecycle over certification-owned roots. Never point a development test at real profile data.
