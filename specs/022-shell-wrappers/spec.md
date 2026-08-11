# Feature Specification: Shell wrappers

**Feature Branch**: `022-shell-wrappers`

**Created**: 2026-08-11

**Status**: Draft

**Input**: Roadmap slice S18 sub-slice B (specification section 18, and the
section 17.5 structured event stream it depends on). Deliver the two thin shell
wrappers, `Invoke-FragCap.ps1` (PowerShell) and `fragcap.sh` (Bash), built to the
ShruggieTech house scripting standards, consuming fragcap's structured event
stream rather than parsing human-readable output, with both wrappers covered by
their compliance checkers in continuous integration.

## Clarifications

### Session 2026-08-11

Resolved under autopilot from the constitution, the architecture of record
(specification sections 17.4, 17.5, 17.6, and 18), and the vendored house
standards, with one item escalated to the operator.

- Q: The ShruggieTech PowerShell standard is vendored with a compliance checker,
  but the Bash standard and a Bash compliance checker are not on disk (a known
  gap the foundation doc flagged as "must be resolved before S18"). How is the
  Bash side resolved? -> A (operator, 2026-08-11): proceed. Author `fragcap.sh`
  to the real ShruggieTech Bash standard and author an in-repo Bash compliance
  checker enforcing its structure; do not change `skills-lock.json`. The
  PowerShell wrapper reuses the vendored `Test-ScriptCompliance.ps1`. Vendoring
  the `shruggie-bash` skill itself remains a separate operator tooling task.
- Q: How are the wrappers verified, given they wrap a Windows binary, live
  capture, privilege elevation, and WSL2 interop, none of which run in
  continuous integration? -> A: continuous integration verifies that both
  compliance checkers pass on the two scripts, that each script is syntactically
  valid (`bash -n`, a PowerShell parse), that each script's help runs and exits
  0, and that the pure path-translation and output-templating logic produces the
  specified results. The wrappers' full runtime behavior (elevation self-relaunch,
  real driver detection, interface enumeration, live capture, WSL2 interop against
  a real Windows binary) is tier 2, manually verified on the operator's machine
  and unexecuted in continuous integration, exactly as live capture has been since
  S09.
- Q: Where do the compliance checkers run? -> A: a new `cargo xtask wrappers`
  subcommand runs both checkers and the syntax checks against
  `scripts/Invoke-FragCap.ps1` and `scripts/fragcap.sh`, returns the 0/1/2
  contract, and is added to the `ci` aggregate and to the `ci.yml` workflow. This
  matches the existing `lint`/`deps`/`license` pattern and needs nothing beyond
  the toolchain plus bash and PowerShell, both present on the continuous
  integration legs.
- Q: What does a wrapper consume from fragcap? -> A: the section 17.5 structured
  JSON event stream on standard error under `--json` (`session.armed`,
  `stage.matched`, `stage.exited`, `filter.narrowed`, `session.complete`). A
  wrapper reacts to capture lifecycle from that stream; it never parses
  human-readable output and never touches the capture data on the sinks
  (constitution P-7).
- Q: What is in a wrapper's scope? -> A: the five responsibilities of section
  18.1 only: privilege verification and elevation, capture-driver presence
  detection with actionable guidance, interface enumeration and selection
  assistance, path translation across environment boundaries, and output-path
  templating with directory preparation. A wrapper that grows beyond these
  indicates a missing capability in the binary; the correct response is to add
  the capability to Rust and shrink the wrapper again.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The PowerShell wrapper prepares the environment and captures (Priority: P1)

An operator on Windows runs `Invoke-FragCap.ps1` to capture a game. The wrapper
verifies the session is elevated and relaunches itself elevated when it is not;
it checks that the capture driver is installed and reports where to download it
when it is absent; it enumerates interfaces and filters virtual adapters from the
presented list; it expands the output-path template's date, time, and profile
tokens and prepares the output directory; and it invokes fragcap, passing the
operator's options through and reacting to the capture lifecycle from the
structured event stream. It contains no capture logic of its own.

**Why this priority**: This is the wrapper that most operators use, on the only
platform capture runs on. Without it the environment concerns that belong outside
the binary (elevation, driver guidance, path templating) are left to every
operator to handle by hand.

**Independent Test**: Run the checker against the script and confirm it passes;
run the script's help and confirm it prints usage and exits 0 without capturing;
exercise the output-template expansion and confirm the tokens resolve to the
specified path. The elevation self-relaunch, real driver detection, and interface
enumeration are verified manually on a Windows machine (tier 2).

**Acceptance Scenarios**:

1. **Given** `Invoke-FragCap.ps1` in `scripts/`, **When** the PowerShell
   compliance checker runs against it, **Then** it reports the script compliant
   with the ShruggieTech PowerShell standard (the structural skeleton and
   encoding).
2. **Given** the wrapper invoked with `-Help`, **When** it runs, **Then** it
   prints its comment-based help and exits 0 without starting a capture.
3. **Given** an output-path template carrying date, time, and profile tokens,
   **When** the wrapper expands it, **Then** the tokens resolve to the specified
   values and the target directory is prepared.
4. **Given** a session that is not elevated, **When** the wrapper runs, **Then**
   it relaunches itself elevated, preserving the operator's arguments; a declined
   elevation is a precondition failure (exit 2), not a silent non-capture.

---

### User Story 2 - The Bash wrapper bridges the subsystem boundary (Priority: P1)

An operator working in a Linux or WSL2 shell runs `fragcap.sh`. Under WSL2 the
wrapper invokes the native Windows binary through interop and translates paths in
both directions, so a relative output path given in the Linux shell resolves to
the intended location and the resulting file path is reported back in Linux form.
On a Linux host with no reachable Windows binary, the wrapper reports that capture
is unavailable on this platform and exits 1 rather than failing obscurely.

**Why this priority**: The Bash wrapper is the one that makes fragcap usable from
the shell many operators actually work in, and the subsystem boundary is the one
environment concern the binary genuinely cannot handle, because it is about how
the binary is reached, not what it does.

**Independent Test**: Run the Bash compliance checker against the script and
confirm it passes; run `bash -n` and confirm the script is syntactically valid;
run the script's help and confirm it prints usage and exits 0; exercise the
path-translation function with representative inputs and confirm both directions
resolve as specified. The live capture through interop is verified manually under
WSL2 (tier 2).

**Acceptance Scenarios**:

1. **Given** `fragcap.sh` in `scripts/`, **When** the Bash compliance checker
   runs against it, **Then** it reports the script compliant with the ShruggieTech
   Bash standard (the four-section layout, the required idioms, and encoding).
2. **Given** the wrapper invoked with `-h` or `--help`, **When** it runs,
   **Then** it prints its self-parsing help and exits 0 without starting a
   capture.
3. **Given** a relative output path in a WSL2 shell, **When** the wrapper
   translates it, **Then** it resolves to the intended Windows location and the
   reported result path is in Linux form.
4. **Given** a Linux host with no reachable Windows binary, **When** the wrapper
   runs, **Then** it reports capture unavailable on this platform and exits 1.

---

### User Story 3 - Both wrappers are held to their standards in CI (Priority: P1)

A contributor changes a wrapper. Continuous integration runs both compliance
checkers and the syntax checks on every push, so a wrapper that drifts from its
house standard, loses its four-section layout, gains an emoji, or stops parsing
fails the build rather than shipping broken.

**Why this priority**: Specification section 18.4 and the constitution both
require both checkers to run in continuous integration, and that gate is currently
unmet: no shell-lint of any kind runs today. A standard that is not enforced is a
standard that quietly rots.

**Independent Test**: Run `cargo xtask wrappers` and confirm it runs both checkers
and both syntax checks and returns 0 when the wrappers are compliant; introduce a
deliberate violation and confirm it returns non-zero naming the script.

**Acceptance Scenarios**:

1. **Given** the two compliant wrappers, **When** `cargo xtask wrappers` runs,
   **Then** it reports both compliant and both syntactically valid and exits 0.
2. **Given** a wrapper with a structural violation, **When** the gate runs,
   **Then** it exits non-zero and names the offending script and the failing
   check.
3. **Given** the `ci` aggregate and the `ci.yml` workflow, **When** they run,
   **Then** the wrappers gate is among the checks, so both checkers run in
   continuous integration.

---

### Edge Cases

- The PowerShell session is not elevated: the wrapper self-relaunches elevated
  with the operator's arguments preserved; a declined User Account Control prompt
  is a precondition failure (exit 2), reported, not swallowed.
- The capture driver is absent: the wrapper reports the download location and
  exits 1, installing nothing (the Licensing rule).
- A Linux host with no reachable Windows binary: `fragcap.sh` reports capture
  unavailable and exits 1 rather than invoking a binary that is not there.
- An unrecognized option: both wrappers pass it through to fragcap unchanged, so
  a new binary flag works through the wrapper without a wrapper change (the shared
  contract).
- `NO_COLOR` set, or standard error is not a terminal: neither wrapper emits color
  codes.
- An output template naming a directory that does not exist: the wrapper prepares
  the directory before capture rather than failing at first write.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `scripts/Invoke-FragCap.ps1` MUST exist and be built to the
  ShruggieTech PowerShell standard (the fixed four-section layout, the
  comment-based help block, an explicit `CmdletBinding`, parameter sets with
  single-letter aliases, `Write-Log`, `ShouldProcess` gating on any destructive
  action, `LiteralPath` handling, and the 0/1/2 exit-code contract), and MUST pass
  the vendored `Test-ScriptCompliance.ps1` checker.
- **FR-002**: The PowerShell wrapper MUST verify that the session is elevated and
  relaunch itself elevated when it is not, preserving the operator's arguments; a
  declined elevation MUST be reported as a precondition failure (exit 2).
- **FR-003**: The PowerShell wrapper MUST detect the capture driver's installed
  state and version and report the download location when it is absent (exit 1),
  installing, downloading, and modifying nothing (constitution P-1, the Licensing
  rule).
- **FR-004**: The PowerShell wrapper MUST enumerate interfaces and filter virtual
  adapters from the presented list.
- **FR-005**: The PowerShell wrapper MUST expand output-path templates carrying
  date, time, and profile tokens and prepare the output directory before capture.
- **FR-006**: `scripts/fragcap.sh` MUST exist and be built to the ShruggieTech
  Bash standard (the fixed four-section layout, a self-parsing help block, `set
  -euo pipefail` with an explicit `IFS`, the `has_cmd`, `log_*`, and `safe_run`
  fixtures, `-q`/`--silent` verbosity with `NO_COLOR` and terminal detection, and
  the 0/1/2 exit-code contract), and MUST pass the authored Bash compliance
  checker.
- **FR-007**: `fragcap.sh` under WSL2 MUST invoke the native Windows binary
  through interop and translate paths in both directions, so a relative output
  path given in a Linux shell resolves to the intended location and the resulting
  file path is reported back in Linux form.
- **FR-008**: `fragcap.sh` on a Linux host with no reachable Windows binary MUST
  report that capture is unavailable on this platform and exit 1 rather than
  failing obscurely.
- **FR-009**: Both wrappers MUST accept the same options, pass through
  unrecognized options to fragcap unchanged, and consume the section 17.5
  structured event stream rather than parsing human-readable output (constitution
  P-7).
- **FR-010**: Both wrappers MUST stay within the five section-18.1
  responsibilities (privilege, driver detection, interface enumeration, path
  translation, output templating); neither MUST contain capture logic or parse
  capture output.
- **FR-011**: A Bash compliance checker MUST be authored, enforcing the
  ShruggieTech Bash structural standard; the PowerShell checker is the vendored
  `Test-ScriptCompliance.ps1` reused unchanged.
- **FR-012**: A `cargo xtask wrappers` gate MUST run both compliance checkers and
  a syntax check of each script against the two wrappers, return the 0/1/2
  contract, and be added to the `ci` aggregate and to the `ci.yml` workflow, so
  both checkers run in continuous integration (specification 18.4).
- **FR-013**: Any term this slice introduces (WSL2 interop, path translation,
  output template, and any other) MUST receive a glossary entry in the same change
  (constitution P-6).
- **FR-014**: The changes to `scripts/**` and `.github/workflows/ci.yml` (pinned
  artifacts) MUST be recorded as a dated decision in the changelog.

### Key Entities *(include if data involved)*

- **PowerShell wrapper**: `Invoke-FragCap.ps1`, the thin Windows-side wrapper
  handling elevation, driver guidance, interface filtering, and output templating.
- **Bash wrapper**: `fragcap.sh`, the thin shell-side wrapper handling the WSL2
  subsystem boundary and path translation.
- **Compliance checker**: the vendored PowerShell checker and the authored Bash
  checker, each validating its script against the house standard's structure.
- **Wrappers gate**: `cargo xtask wrappers`, running both checkers and both syntax
  checks, wired into `ci` and `ci.yml`.
- **Output template**: an output-path string carrying date, time, and profile
  tokens the wrapper expands before capture.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo xtask wrappers` reports both wrappers compliant and both
  syntactically valid and exits 0; a deliberate violation makes it exit non-zero
  naming the script and the failing check.
- **SC-002**: `Invoke-FragCap.ps1` passes `Test-ScriptCompliance.ps1`, and
  `fragcap.sh` passes the authored Bash checker.
- **SC-003**: Each wrapper's help (`-Help` for PowerShell, `-h`/`--help` for
  Bash) prints usage and exits 0 without starting a capture.
- **SC-004**: The output-path template expansion and the WSL2 path translation
  produce the specified results for representative inputs, checked without a
  capture driver or a real Windows binary.
- **SC-005**: An unrecognized option is passed through to fragcap unchanged,
  checked through a dry-run that captures the assembled invocation.
- **SC-006**: `cargo xtask ci` passes with the wrappers gate included, and the two
  wrappers carry an SPDX identifier and the required encoding (UTF-8 without BOM,
  LF, no trailing whitespace, single trailing newline, no em or en dashes) per the
  conventions linter.
- **SC-007**: Neither wrapper contains capture or capture-output-parsing logic
  (constitution P-7), and neither installs, downloads, or modifies the capture
  driver (P-1, Licensing).

## Assumptions

- The wrappers' full runtime behavior (elevation self-relaunch, real driver
  detection, interface enumeration, live capture, WSL2 interop against a real
  Windows binary) is tier 2, manually verified on the operator's machine and
  unexecuted in continuous integration, as live capture has been since S09.
- PowerShell 7 and bash are available on both continuous integration legs (ubuntu
  and windows), so the wrappers gate runs there. The GitHub-hosted ubuntu runner
  ships both; the windows runner ships PowerShell and Git Bash.
- The Bash standard applied is the real ShruggieTech Bash standard; a repo-vendored
  `shruggie-bash` skill is out of scope for this slice (operator decision,
  2026-08-11) and remains a separate tooling task. The authored checker enforces
  the standard's structure so the section 18.4 continuous-integration gate is met.
- fragcap's `--json` event stream (section 17.5) is the wrapper input contract; it
  exists, is emitted on standard error, and is stable across the wrapper's life.
- The wrappers are packaged into the release archive by the existing `release.yml`
  step, which already globs `scripts/*.ps1` and `scripts/*.sh`.
