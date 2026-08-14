# Quickstart / Verification Guide: FileVersion stamp and extcap scope flags

## Gate suite (foreground, CI parity)

```bash
cargo xtask ci        # fmt, clippy, tests, lint, deps, license
cargo xtask msrv      # THE CRUX: builds the workspace under 1.82, compiling winresource
```

`cargo xtask ci` includes the extcap and doctor tests. `cargo deny`/`cargo xtask
license` must accept `winresource` + `version_check` (both MIT/Apache-2.0). Install
the 1.82 toolchain first so msrv actually runs rather than exit-2 skipping:

```bash
rustup toolchain install 1.82
cargo xtask msrv
```

If `cargo xtask msrv` fails to compile winresource, apply the research.md fallback
(pin a winresource patch that builds on 1.82; else hand-roll the `.rc`).

## Scope-flag tests

```bash
cargo test -p fragcap-cli --test cli_extcap
```

Expected: `--system` registers into the machine-wide dir (via
`FRAGCAP_SYSTEM_EXTCAP_DIR`), `--user`/default into the per-user dir, a double scope
(`--user --system`, or `--dir` with a scope) exits non-zero as a clap conflict, and
the existing idempotency and end-to-end doctor tests stay green.

## FileVersion proof (Windows, direct evidence for the #104 aside)

```bash
cargo build --release --locked -p fragcap-cli --features live,socket-table,etw
```
```powershell
(Get-Item target\release\fragcap.exe).VersionInfo.FileVersion   # expect 0.3.0.0
(Get-Command target\release\fragcap.exe).Version                # expect 0.3.0.0, not 0.0.0.0
target\release\fragcap.exe --version                            # expect fragcap 0.3.0 (unchanged)
```

## Scope-flag manual smoke (optional, Windows)

```powershell
$env:FRAGCAP_SYSTEM_EXTCAP_DIR = "$env:TEMP\sys-extcap"
target\release\fragcap.exe extcap install --system
# expect the binary in $env:TEMP\sys-extcap ; doctor then reports machine-wide ok
target\release\fragcap.exe extcap uninstall --system
```

## Done signal

`cargo xtask ci` green, `cargo xtask msrv` green under 1.82, the scope tests pass,
and a Windows release exe reports the crate version rather than `0.0.0.0`.
