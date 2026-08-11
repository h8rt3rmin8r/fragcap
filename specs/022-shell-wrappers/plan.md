# Implementation Plan: Shell wrappers

**Branch**: `022-shell-wrappers` | **Date**: 2026-08-11 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `specs/022-shell-wrappers/spec.md` (roadmap
slice S18 sub-slice B, specification section 18 and the section 17.5 event
stream).

## Summary

Deliver the two thin shell wrappers to the ShruggieTech house standards, and the
continuous-integration gate that holds them to those standards. This slice adds:

1. `scripts/Invoke-FragCap.ps1`, the Windows-side PowerShell wrapper: elevation
   self-relaunch, capture-driver detection with download guidance, interface
   enumeration filtering virtual adapters, and output-path templating. Built to
   the vendored ShruggieTech PowerShell standard; passes `Test-ScriptCompliance.ps1`.
2. `scripts/fragcap.sh`, the shell-side Bash wrapper: the WSL2 subsystem boundary,
   invoking the native Windows binary through interop and translating paths in
   both directions, and reporting capture unavailable on a Linux host with no
   reachable binary. Built to the ShruggieTech Bash standard.
3. An authored Bash compliance checker and a `cargo xtask wrappers` gate that runs
   both checkers and the syntax checks against the two wrappers, wired into the
   `ci` aggregate and `ci.yml` so both checkers run in continuous integration.
4. `docs/glossary.md` entries for the new terms (P-6), and the pinned-artifact
   decision fragment for `scripts/**` and `ci.yml`.

The load-bearing constraint is P-7: the wrappers handle environment concerns that
belong outside the binary (elevation, driver guidance, path translation, output
templating) and contain no capture logic and no parsing of capture output. They
consume the section 17.5 `--json` event stream, never human-readable text. The
honest verification boundary mirrors live capture since S09: continuous
integration verifies the compliance checkers, the syntax validity, the help
paths, and the pure templating and translation logic (through a `--dry-run` seam);
the wrappers' full runtime behavior is tier 2, manually verified.

## Technical Context

**Language/Version**: PowerShell 7 (`Invoke-FragCap.ps1`), Bash 5.x via
`#!/usr/bin/env bash` (`fragcap.sh`), Rust for the `cargo xtask wrappers` gate
(workspace MSRV 1.82).

**Primary Dependencies**: none new in Cargo. The gate shells out to `bash` (for
the vendored PowerShell checker's POSIX twin, and `bash -n`) and, when present,
`pwsh`. The vendored `Test-ScriptCompliance.ps1` / `test-script-compliance.sh`
under `.agents/skills/shruggie-powershell/scripts/` is reused unchanged.

**Storage**: none. The wrappers prepare output directories at capture time.

**Testing**: `cargo xtask wrappers` runs the two compliance checkers, `bash -n`,
and the scripts' `--help` and `--dry-run` paths (the latter prints the assembled
fragcap invocation and exits 0 without capturing, which makes templating and
option pass-through checkable with no capture driver). The wrappers' full runtime
behavior (elevation, real driver detection, interface enumeration, live capture,
WSL2 interop against a real Windows binary) is tier 2, manually verified.

**Target Platform**: `Invoke-FragCap.ps1` is Windows/PowerShell 7. `fragcap.sh`
is Linux/WSL2 Bash. The gate runs anywhere bash is present (both continuous
integration legs, and locally through Git Bash).

**Project Type**: Rust workspace plus committed shell wrappers under `scripts/`.

**Performance Goals**: not applicable; the wrappers are launch-time glue.

**Constraints**: P-7 (thin wrappers, no capture logic, no output parsing); P-1
and the Licensing rule (driver detection installs and downloads nothing); P-8
(house standards, UTF-8/LF, no em or en dashes, SPDX); the 0/1/2 exit contract
(section 17.4).

**Scale/Scope**: two wrapper scripts, one authored checker, one xtask gate, one
workflow edit.

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1 design.*

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | PASS. The wrappers detect the driver read-only and install nothing; they launch and template, they do not touch traffic or a target. `cargo xtask lint` is unaffected. |
| P-2 Core Stays Platform-Neutral | PASS. No crate changes except `xtask` (the gate) and `xtask/lint.rs` (the shebang refinement); `fragcap-core` is untouched. |
| P-3 Capture And Attribution Separate | N/A. The wrappers add no source and no attributor. |
| P-4 No Silent Loss | N/A here. The wrappers count nothing; they react to the event stream that already carries the counters. |
| P-5 Compatibility Outranks Richness | N/A. No output format changes. |
| P-6 Glossary First | ACTION. `WSL2 interop`, `path translation`, and `output template` get `docs/glossary.md` entries in this change. |
| P-7 Wrappers Stay Thin | PASS, and central. Scope is the five section-18.1 responsibilities only; the wrappers consume the section 17.5 event stream and never parse capture output. A `--dry-run` seam prints the assembled invocation and does no capture. |
| P-8 House Standards Apply | PASS, and central. Both scripts are built to their ShruggieTech standards and held there by the checkers in CI; UTF-8/LF, no em or en dashes, SPDX. |
| P-9 The Instrument Does Not Lie | N/A. The wrappers alter no observation. |
| Licensing | PASS. Driver detection reports the download location and installs nothing. |
| Pinned artifacts | ACTION. `scripts/**` (the two wrappers) and `.github/workflows/ci.yml` (the gate step) change, recorded as a dated decision. `xtask/**` is not pinned. |

No principle is violated; the Complexity Tracking table is empty.

## Project Structure

### Documentation (this feature)

```text
specs/022-shell-wrappers/
├── plan.md              # This file
├── research.md          # Phase 0: decisions, rationale, alternatives
├── data-model.md        # Phase 1: the wrapper option/exit model
├── quickstart.md        # Phase 1: runnable validation scenarios
├── contracts/
│   └── wrapper-contract.md   # shared option/exit/dry-run contract + the gate
├── checklists/
│   ├── requirements.md  # spec quality (from /speckit-specify)
│   └── wrappers.md      # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
scripts/
├── Invoke-FragCap.ps1        # NEW: PowerShell wrapper (ShruggieTech PS standard)
└── fragcap.sh                # NEW: Bash wrapper (ShruggieTech Bash standard)

xtask/src/
├── main.rs                   # add the `wrappers` dispatch arm + USAGE + ci step
└── wrappers.rs               # NEW: the gate (PS checker + authored Bash checker
                              # + syntax + help/dry-run), 0/1/2 like lint/deps
xtask/src/lint.rs             # SPDX check accepts a shebang line 1 then SPDX line 2

.github/workflows/ci.yml      # NEW step: cargo run -p xtask -- wrappers (pinned)

docs/glossary.md              # WSL2 interop / path translation / output template
changelog.d/S18b-wrappers.added.md       # user-facing capability
changelog.d/S18b-wrappers.decisions.md   # pinned-artifact + design decisions
```

**Structure Decision**: The wrappers live in `scripts/` (already release-staged
and inside the conventions linter's walk). The gate is a new `xtask` subcommand
matching the `lint`/`deps`/`license` pattern, so it needs nothing installed beyond
the toolchain plus bash. The vendored PowerShell checker is reused; the Bash
checker is authored in `xtask` (reading the file), because there is no vendored
Bash checker and the operator chose to author rather than vendor one.

## Key design decisions (recorded per autopilot decision policy)

Decided from the constitution, the architecture of record (specification 18,
17.5, 17.6, 17.4), the vendored house standards, and the one operator decision;
reasoning and alternatives are in [research.md](research.md).

- **D1. `Invoke-FragCap.ps1` is a non-destructive PowerShell script.** It
  captures, prepares output directories, and self-relaunches elevated: creation
  and launch, not destruction, so `SupportsShouldProcess=$false`,
  `ConfirmImpact='None'`. Comment-based help before `[CmdletBinding`, `Default`
  and `HelpText` parameter sets with single-letter aliases, `Write-Log`,
  `LiteralPath`, and the 0/1/2 exit contract. It passes the vendored
  `Test-ScriptCompliance.ps1`.
- **D2. `fragcap.sh` carries the shebang on line 1 and the SPDX on line 2.** A
  shell script's `#!/usr/bin/env bash` must be line 1 for the kernel to honor it,
  which conflicts with the repository's SPDX-first-line rule. `xtask/lint.rs` is
  refined so a source file whose first line is a shebang carries its SPDX on line
  2. `xtask` is not a pinned artifact. The script is otherwise the standard shape:
  man-page help, `set -euo pipefail` with `IFS`, the four-section layout, the
  `print_help`/`log_*`/`has_cmd`/`safe_run` fixtures, `-q`/`--silent`/`NO_COLOR`,
  and the 0/1/2 contract.
- **D3. The Bash wrapper's distinguishing job is the WSL2 boundary.** It detects
  WSL2, invokes the native Windows binary through interop, and translates paths in
  both directions; on a Linux host with no reachable Windows binary it reports
  unavailable and exits 1. Path translation is a small pure function checkable
  through `--dry-run`.
- **D4. Both wrappers consume the section 17.5 `--json` event stream, not
  human-readable output (P-7).** They react to lifecycle (`session.armed`,
  `stage.matched`, `session.complete`) and never read the capture data on the
  sinks. Unrecognized options are passed through to fragcap unchanged.
- **D5. A `--dry-run` seam makes the thin logic tier-1 testable.** Each wrapper,
  given `--dry-run` (`-DryRun`), expands the output template and assembles the
  fragcap invocation, prints it, and exits 0 without elevation, driver detection,
  or capture. This exercises templating (SC-004) and option pass-through (SC-005)
  with no capture driver and is useful to an operator as a preview.
- **D6. The Bash compliance checker is authored in `xtask` (Rust).** It reads
  `fragcap.sh` and checks the ShruggieTech Bash structure: the shebang on line 1,
  the SPDX on line 2, `set -euo pipefail` with `IFS`, the four `#`-plus-79-
  underscore dividers with the four `# ` headings in order, the help block, no
  emoji, and the encoding. This matches the vendored PowerShell checker's
  structural scope. The full semantic conventions (fixtures present, quoting) are
  authored by the standard and, where a tool is available, checked by `shellcheck`
  best-effort.
- **D7. `cargo xtask wrappers` is the gate.** It runs the vendored PowerShell
  checker (its POSIX twin, needing only bash) on `Invoke-FragCap.ps1`, the
  authored Bash checker on `fragcap.sh`, `bash -n` on the Bash script, and each
  script's `--help` and `--dry-run` when its interpreter is present. It returns
  the 0/1/2 contract and is added to the `ci` aggregate and to `ci.yml`. If the
  required interpreter (bash) is absent it exits 2 (could not run), never a false
  pass, matching `neutral`/`msrv`.
- **D8. Driver detection installs nothing.** The PowerShell wrapper checks the
  driver's presence and version from the filesystem read-only and reports the
  download location when absent (exit 1), which is P-1 and the Licensing rule
  made mechanical.

## Open honesty note (surfaced at the pre-push halt)

The wrappers' core function is a live capture on Windows, through elevation, a
capture driver, real interface enumeration, and WSL2 interop against a native
binary, none of which run in continuous integration, exactly as live capture has
not run since S09. What this slice proves at tier 1 is: both scripts pass their
compliance checkers, both parse, both print help and exit 0, and the pure
templating and path-translation logic produces the specified results through the
`--dry-run` seam. The elevation self-relaunch, the real driver and interface
detection, and the interop capture are verified by hand on a Windows and WSL2
machine, and the changelog says so rather than implying a green runtime.
