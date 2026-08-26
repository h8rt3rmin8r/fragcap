# Quickstart: Doctor ETW Session Probe

## Baseline

```powershell
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
```

## Focused Implementation Checks

```powershell
cargo test -p fragcap-cli tracing_availability
cargo test -p fragcap-attr etw
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
```

## Full Gate

```powershell
cargo xtask ci
```

## Local Timing And Leak Checks

On an elevated Windows shell with the `etw` feature available:

```powershell
cargo run -p fragcap-cli --features etw -- doctor --timings
logman query -ets | Select-String fragcap-doctor-probe
```

If elevation or ETW is unavailable, record the exact command and result instead of claiming a measured speedup.
