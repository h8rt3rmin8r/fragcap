# Data Model: Shell wrappers

No persistent storage. The model is the two wrappers' option surface, their exit
contract, and the gate's structure.

## Shared wrapper options

Both wrappers accept the same options and pass unrecognized ones through to
fragcap unchanged (the shared contract, specification 18.4).

| Option (Bash / PowerShell) | Meaning |
| --- | --- |
| `-h`, `--help` / `-Help`, `-h` | Print help and exit 0 without capturing. |
| `-q`, `--quiet` / `-Quiet`, `-q` | Suppress informational output; keep warnings and errors. |
| `--silent` / `-Silent` | Suppress warnings too; errors still emit. |
| `--dry-run` / `-DryRun` | Print the assembled fragcap invocation and exit 0; no elevation, driver check, or capture. |
| `--no-color` / `NO_COLOR` env | Disable colored output. |
| `-o <template>` / `-Out <template>` | Output-path template with date, time, and profile tokens. |
| everything else | Passed through to fragcap unchanged. |

**Invariants**: `-h`/`--help` is help only, never a suppression alias; `-q`/`-s`
never means "silent" (the standards forbid `-s` as a suppression alias);
`--dry-run` never starts a capture.

## Exit contract (both wrappers, section 17.4)

| Code | Meaning |
| --- | --- |
| 0 | The wrapper's work succeeded (capture completed, help/dry-run printed). |
| 1 | A runtime failure (the driver is absent; capture unavailable on this platform). |
| 2 | An environment precondition failure (elevation declined). |

## Output template

An output-path string carrying tokens the wrapper expands before capture:

| Token | Expands to |
| --- | --- |
| `{date}` | The capture date, `YYYY-MM-DD`. |
| `{time}` | The capture time, `HHMMSS`. |
| `{profile}` | The profile name from the invocation. |

**Invariants**: expansion is pure and deterministic given the inputs; the target
directory is prepared before capture; `--dry-run` prints the expanded path
without creating anything.

## `Invoke-FragCap.ps1` shape (ShruggieTech PowerShell standard)

- Comment-based help block before `[CmdletBinding(SupportsShouldProcess=$false,
  ConfirmImpact='None',DefaultParameterSetName='Default')]`.
- `Param(` with `Default` and `HelpText` parameter sets; each parameter a
  single-letter `[Alias]`, typed, validated.
- The four-section layout: `## Declare Functions`, `## Declare Variables and
  Arrays`, `## Execute Operations`, `## End of script`, each divider `#` plus 79
  underscores.
- `Write-Log` for progress; `LiteralPath` for filesystem operations; the 0/1/2
  exit contract; no emojis; UTF-8 no BOM, LF.
- Responsibilities: elevation verification and self-relaunch; driver
  presence/version detection (read-only) with download guidance; interface
  enumeration filtering virtual adapters; output templating and directory prep.

## `fragcap.sh` shape (ShruggieTech Bash standard)

- Line 1 `#!/usr/bin/env bash`; line 2 `# SPDX-License-Identifier: Apache-2.0`;
  the man-page help block (`NAME`, `SYNOPSIS`, `DESCRIPTION`, `OPTIONS`,
  `EXAMPLES` x2+, `EXIT CODES`, `AUTHOR`); `set -euo pipefail` with `IFS`.
- The four-section layout: `# Declare Functions`, `# Declare Variables and
  Arrays`, `# Execute Operations`, `# End of script`, each divider `#` plus 79
  underscores; `# End of script` the last content line.
- The `print_help`/`log_*`/`has_cmd`/`safe_run` fixtures; `-q`/`--silent`/
  `NO_COLOR`/TTY-aware color; the manual argument-parsing `case` loop with the
  `-h`/`--help` gate first; the 0/1/2 contract; no emojis.
- Responsibility: the WSL2 subsystem boundary (interop invocation, bidirectional
  path translation) and the Linux-host-without-a-binary case (exit 1).

## `cargo xtask wrappers` gate

- `xtask/src/wrappers.rs::run(root) -> io::Result<usize>`: the count of failing
  checks; `Ok(0)` clean (exit 0), `Ok(n)` failures (exit 1), `Err` could-not-run
  (exit 2), matching `lint`/`deps`/`license`.
- Runs: the vendored PowerShell checker (POSIX twin) on `Invoke-FragCap.ps1`; the
  authored Bash structural checker on `fragcap.sh`; `bash -n fragcap.sh`; each
  script's `--help` (exit 0) and `--dry-run` (assembled command) when its
  interpreter is present.

## `xtask/src/lint.rs` refinement

The SPDX-first-line check accepts a shebang: when a source file's first line
starts with `#!`, the SPDX identifier is required on line 2 instead of line 1.
