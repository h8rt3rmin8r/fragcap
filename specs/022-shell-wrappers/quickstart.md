# Quickstart: Shell wrappers

Runnable validation for slice S18b. The tier-1 scenarios need no capture driver,
no elevation, and no analyzer; the tier-2 scenarios are manual, on a real machine.

## 1. The compliance gate (tier 1)

```bash
cargo xtask wrappers
```

Expect: both wrappers reported compliant and syntactically valid, help and
dry-run passing, exit 0 (SC-001). Introduce a violation (drop a divider, add an
emoji) and it exits non-zero naming the script and the failing check.

## 2. Help paths (tier 1)

```bash
bash scripts/fragcap.sh --help
pwsh -NoProfile -File scripts/Invoke-FragCap.ps1 -Help
```

Expect: each prints its usage and exits 0 without starting a capture (SC-003).

## 3. Templating and pass-through (tier 1, dry-run)

```bash
bash scripts/fragcap.sh --dry-run --profile eso -o "caps/{profile}-{date}.fcapng" --loopback
```

Expect: the assembled `fragcap run ...` line with `{profile}` and `{date}`
expanded, `--loopback` passed through, and `--json` added (SC-004, SC-005). The
PowerShell wrapper's `-DryRun` does the same.

## 4. The full gate (tier 1)

```bash
cargo xtask ci
```

Expect: green, with the wrappers gate among the checks (SC-006); the wrappers
carry SPDX and the required encoding per the conventions linter.

## 5. Real capture through the wrappers (tier 2, manual)

On Windows, elevated is handled by the wrapper:

```powershell
scripts\Invoke-FragCap.ps1 -Profile eso -Out "caps\{profile}-{date}.fcapng"
```

Expect: the wrapper verifies elevation (relaunching if needed), confirms the
capture driver, filters virtual adapters, expands the template, and captures.
Under WSL2:

```bash
scripts/fragcap.sh --profile eso -o "caps/{profile}-{date}.fcapng"
```

Expect: the wrapper invokes the native Windows binary through interop and reports
the resulting file path in Linux form. On a Linux host with no reachable Windows
binary, it reports capture unavailable and exits 1 (FR-008). These paths are not
exercised in continuous integration, exactly as live capture has not been since
S09.
