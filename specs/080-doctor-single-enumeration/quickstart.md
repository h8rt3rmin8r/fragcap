# Quickstart: Doctor Single Enumeration

## Baseline

```powershell
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
```

## Implementation Checks

```powershell
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
```

## Full Gate

```powershell
cargo xtask ci
```

## Manual Timing Check

On a Windows machine with the live backend and npcap available:

```powershell
cargo run -p fragcap-cli --features live -- doctor --timings
```

Use the output only as local evidence. If the local binary or machine cannot exercise the live backend, record that limitation rather than claiming a measured speedup.
