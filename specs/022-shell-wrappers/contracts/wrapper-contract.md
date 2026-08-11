# Contract: wrapper option/exit contract and the compliance gate

The shared behavior both wrappers honor, and the `cargo xtask wrappers` gate that
holds them to their standards.

## Shared option contract (specification 18.4)

- Both wrappers accept the same options and **pass unrecognized options through
  to fragcap unchanged**, so a new binary flag needs no wrapper change.
- `-h`/`--help` (`-Help`) prints help and exits 0 **before any work**.
- `-q`/`--quiet` suppresses informational output; `--silent` (`-Silent`)
  suppresses warnings too; errors always emit. `-s` is never a suppression alias.
- `--no-color` and the `NO_COLOR` environment variable disable color; a non-TTY
  standard error disables color.
- `--dry-run` (`-DryRun`) prints the assembled fragcap invocation and exits 0,
  with no elevation, no driver detection, and no capture.
- The wrappers consume the section 17.5 `--json` event stream on standard error;
  they never parse human-readable output (P-7).

## Exit contract (section 17.4)

| Code | Condition |
| --- | --- |
| 0 | Capture completed, or help/dry-run printed. |
| 1 | Runtime failure: the capture driver is absent (PowerShell), or capture is unavailable on this platform (Bash on a Linux host with no reachable binary). |
| 2 | Precondition failure: elevation declined (PowerShell). |

## Output-template contract

`-o`/`-Out` accepts a template expanded before capture:

- `{date}` -> `YYYY-MM-DD`, `{time}` -> `HHMMSS`, `{profile}` -> the profile name.
- The target directory is prepared before capture.
- `--dry-run` prints the expanded path and creates nothing.

Example (Bash, dry-run):

```text
$ fragcap.sh --dry-run --profile eso -o "caps/{profile}-{date}.fcapng" --loopback
fragcap run --profile eso --out caps/eso-2026-08-11.fcapng --loopback --json
```

The assembled line shows the expanded template and the passed-through `--loopback`
and confirms `--json` is added so the wrapper consumes the event stream.

## PowerShell compliance (vendored checker)

`Invoke-FragCap.ps1` passes `Test-ScriptCompliance.ps1` (and its POSIX twin),
which checks: UTF-8 no BOM, LF, no trailing whitespace, single trailing newline,
no emoji, the four `#`-plus-79-underscore dividers with the four `## ` headings in
order, and a `<# ... #>` help block before the first `[CmdletBinding`.

## Bash compliance (authored checker)

`fragcap.sh` passes the authored Bash checker, which checks: line 1 is
`#!/usr/bin/env bash`; line 2 is the SPDX identifier; `set -euo pipefail` with an
explicit `IFS` is present; the four `#`-plus-79-underscore dividers with the four
`# ` headings (`Declare Functions`, `Declare Variables and Arrays`, `Execute
Operations`, `End of script`) in order; `# End of script` is the last content
line; a help block is present; no emoji; UTF-8 no BOM, LF, no trailing
whitespace, single trailing newline. `bash -n` confirms syntax; `shellcheck`
runs best-effort when installed.

## The gate

`cargo xtask wrappers`:

| Result | Meaning |
| --- | --- |
| exit 0 | Both wrappers compliant, both syntactically valid, help and dry-run pass. |
| exit 1 | A check failed; the output names the script and the failing check. |
| exit 2 | The gate could not run (bash absent). Never a false pass. |

It is added to the `cargo xtask ci` aggregate and to the `ci.yml` workflow, so
both checkers run in continuous integration (specification 18.4).
