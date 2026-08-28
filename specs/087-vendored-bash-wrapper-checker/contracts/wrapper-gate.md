# Contract: Wrapper Gate

## Command

```powershell
cargo xtask wrappers
```

## Environment Contract

- Returns `2` if Bash is unavailable.
- Returns `2` if ShellCheck is not runnable from the same plain Bash mode that invokes the vendored checker.
- Returns `2` if PowerShell 7 is unavailable.

## Compliance Contract

- Invokes `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh scripts/fragcap.sh`.
- Invokes `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh scripts/lint-docs.sh`.
- Invokes `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh scripts/cut-release.sh`.
- Invokes the existing vendored PowerShell checker for `scripts/Invoke-FragCap.ps1`.
- Invokes the existing vendored PowerShell checker for `scripts/New-Release.ps1`.

## Failure Contract

- Returns `0` when every check runs and passes.
- Returns `1` when any script, checker, syntax, help, dry-run, or compliance check fails.
- Missing target scripts and missing checker files are check failures, not environment failures.
- Returns `2` only when the gate cannot run because a required executable is absent.

## Preserved Seam Checks

- `bash -n` for all gated Bash scripts.
- `--help` for all gated Bash scripts.
- `fragcap.sh --dry-run` assembly check.
- PowerShell parser checks for all gated PowerShell scripts.
- `-Help` for all gated PowerShell scripts.
- `Invoke-FragCap.ps1 -DryRun` assembly check.
