# Quickstart: Proxy Bypass and Local-Destination Policy

## Focused parser and route tests

```powershell
cargo test -p fragcap --test deep_capture_routing
```

Expected: exact DNS, suffix, IP, CIDR, port, IPv6, canonical ordering, listener infrastructure, and environment ownership tests pass.

## Proxy destination safety tests

```powershell
cargo test -p fragcap-proxy --test upstream
```

Expected: listener aliases, local/private destinations, mixed DNS answers, rebinding attempts, and exact controlled grants remain deterministic with no direct fallback.

## CLI and evidence tests

```powershell
cargo test -p fragcap-cli --test cli_deep_capture
```

Expected: valid repeated bypass inputs appear canonically in plan and bundle output; malformed inputs refuse before effects; inherited proxy variables do not survive controlled launch.

## Full repository gate

```powershell
cargo xtask ci
```

Expected: all required checks pass with no dependency or lockfile change.
