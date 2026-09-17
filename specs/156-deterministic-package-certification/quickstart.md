# S156 Quickstart

## Local static and unit verification

Do not run `scripts/Test-PackageCertification.ps1` locally and do not execute an installed fragcap product.

```powershell
cargo test -p xtask package_certification
$errors = $null
$tokens = $null
[System.Management.Automation.Language.Parser]::ParseFile((Resolve-Path -LiteralPath 'scripts/Test-PackageCertification.ps1'), [ref]$tokens, [ref]$errors) | Out-Null
if ($errors.Count -ne 0) { throw ($errors | Out-String) }
pwsh -NoLogo -NoProfile -NonInteractive -File .agents/skills/shruggie-powershell/scripts/Test-ScriptCompliance.ps1 -Path scripts/Test-PackageCertification.ps1
cargo xtask ci
```

## Hosted package verification

Push the reviewed branch and allow the ordinary pull-request workflows to run. The final head must produce green `Published review candidate` and `Windows package certification` runs without rerun. Inspect their package reports and confirm schema 4 contains separate portable and installed rows, deterministic structured reachability, zero non-loopback observations and reconciled cleanup.

## Negative contract verification

The focused Rust test constructs one schema-4 positive report, accepts a zero-endpoint diagnostic observation, and independently mutates every required authority and cross-field invariant. It also validates retained schema-2 and schema-3 reports under their historical rules and rejects unknown or mixed schemas.
