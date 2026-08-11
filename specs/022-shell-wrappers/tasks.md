# Tasks: Shell wrappers

**Feature**: roadmap slice S18 sub-slice B (specification section 18)
**Branch**: `022-shell-wrappers`
**Input**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

Test-driven where the artifact is testable: the Bash checker and the lint
refinement are unit-tested against known-bad input before the wrappers are
authored to pass them. Verification is `cargo xtask ci` (with the new `wrappers`
gate) plus `cargo xtask neutral`, run in the foreground.

Path conventions: wrappers in `scripts/`, the gate in `xtask/src/wrappers.rs`,
the lint refinement in `xtask/src/lint.rs`, glossary in `docs/glossary.md`.

## Phase 1: Setup

- [ ] T001 Add glossary entries (P-6) to `docs/glossary.md` for `WSL2 interop`,
  `path translation`, and `output template`, following the existing entry
  template and cross-linking to the shell-wrapper and event-stream entries.
- [ ] T002 [P] Add changelog fragments `changelog.d/S18b-wrappers.added.md` (the
  two wrappers and the CI gate, present tense, citing section 18 and roadmap slice
  S18) and `changelog.d/S18b-wrappers.decisions.md` (the pinned-artifact decision
  for `scripts/**` and `ci.yml`, date-led, plus the design decisions D1 to D8 from
  plan.md including the shebang/SPDX lint refinement).

## Phase 2: Foundational (the lint refinement and the gate skeleton)

- [ ] T003 Write a failing unit test in `xtask/src/lint.rs` (`#[cfg(test)]`): a
  source file whose first line is a shebang (`#!/usr/bin/env bash`) and whose
  second line is the SPDX identifier passes the SPDX check; one with a shebang and
  no SPDX on line 2 fails.
- [ ] T004 Refine the SPDX check in `xtask/src/lint.rs` so a first-line shebang
  moves the SPDX requirement to line 2; make T003 pass. Leave the non-shebang
  case (SPDX on line 1) unchanged.
- [ ] T005 Create `xtask/src/wrappers.rs` with `run(root) -> io::Result<usize>`
  and the authored Bash structural checker `check_bash(bytes) -> Vec<&str>` (a
  pure function over file bytes, like `lint::rules`): shebang line 1, SPDX line 2,
  `set -euo pipefail` with `IFS`, the four `#`-plus-79-underscore dividers with the
  four `# ` headings in order, `# End of script` last, a help block present, no
  emoji, and encoding. Register `mod wrappers;` in `xtask/src/main.rs`.
- [ ] T006 [P] Write failing unit tests in `xtask/src/wrappers.rs` for
  `check_bash`: a compliant skeleton passes; a missing divider, a wrong heading
  order, a missing `set -euo pipefail`, an emoji, and a missing SPDX each produce
  the specific finding.
- [ ] T007 Add the `wrappers` dispatch arm to `xtask/src/main.rs` (0/1/2 like
  `lint`) and the `wrappers` line to `USAGE`; add the `wrappers` step to the `ci`
  aggregate after `license`.

**Checkpoint**: `cargo xtask wrappers` runs (reporting the absent scripts), the
lint refinement is green, the Bash checker is unit-tested.

## Phase 3: US1 - The PowerShell wrapper (Priority: P1)

**Goal**: `Invoke-FragCap.ps1` to the ShruggieTech PowerShell standard, passing
the vendored checker, with help and dry-run.
**Independent test**: the vendored checker passes; `-Help` and `-DryRun` behave.

- [ ] T008 [US1] Author `scripts/Invoke-FragCap.ps1` from the vendored template:
  comment-based help before `[CmdletBinding(SupportsShouldProcess=$false,...)]`,
  `Default`/`HelpText` parameter sets with single-letter aliases, the four-section
  layout, `Write-Log`, `LiteralPath`, the 0/1/2 exit contract, no emoji, UTF-8
  no BOM/LF/SPDX line 1.
- [ ] T009 [US1] Implement the wrapper responsibilities: the `-Help` gate first;
  the `-DryRun` seam (expand the output template, assemble and print the fragcap
  invocation with `--json`, pass through unknown options, exit 0); elevation
  verification and self-relaunch (declined elevation exits 2); driver presence and
  version detection read-only with the download location when absent (exit 1);
  interface enumeration filtering virtual adapters; output-template expansion and
  directory preparation.
- [ ] T010 [US1] In `xtask/src/wrappers.rs`, run the vendored PowerShell checker
  (POSIX twin, `bash .agents/skills/shruggie-powershell/scripts/test-script-compliance.sh
  scripts/Invoke-FragCap.ps1`) and count a failure; run `Invoke-FragCap.ps1
  -Help` and `-DryRun` through `pwsh` when present, asserting exit 0 and the
  assembled command. Confirm the vendored checker passes on the authored script.

## Phase 4: US2 - The Bash wrapper (Priority: P1)

**Goal**: `fragcap.sh` to the ShruggieTech Bash standard, passing the authored
checker, with help, dry-run, and the WSL2 boundary.
**Independent test**: the authored checker and `bash -n` pass; `--help` and
`--dry-run` behave; the Linux-no-binary case exits 1.

- [ ] T011 [US2] Author `scripts/fragcap.sh` from the vendored Bash template:
  shebang line 1, SPDX line 2, the man-page help block, `set -euo pipefail` with
  `IFS`, the four-section layout, the `print_help`/`log_*`/`has_cmd`/`safe_run`
  fixtures, `-q`/`--silent`/`NO_COLOR`/TTY color, the manual `case` argument loop
  with the `-h`/`--help` gate first, the 0/1/2 contract, no emoji.
- [ ] T012 [US2] Implement the responsibilities: the `--dry-run` seam (expand the
  template, assemble and print the fragcap invocation with `--json`, pass through
  unknown options, exit 0); WSL2 detection and interop invocation of the native
  Windows binary with bidirectional path translation; the Linux-host-without-a-
  reachable-binary case reporting unavailable and exiting 1.
- [ ] T013 [US2] In `xtask/src/wrappers.rs`, run `check_bash` on
  `scripts/fragcap.sh`, `bash -n scripts/fragcap.sh`, and `fragcap.sh --help`
  and `--dry-run` through bash, asserting exit 0 and the assembled command.
  Confirm `check_bash` and `shellcheck` (best-effort) pass on the authored script.

## Phase 5: US3 - Both checkers run in CI (Priority: P1)

**Goal**: the gate runs both checkers and both syntax checks and is wired into
continuous integration.
**Independent test**: `cargo xtask wrappers` exits 0 on the compliant wrappers
and non-zero on a deliberate violation; the `ci.yml` step is present.

- [ ] T014 [US3] Finalize `cargo xtask wrappers` to run, in order: the vendored
  PowerShell checker, the authored Bash checker, `bash -n`, and the help/dry-run
  behavioral checks, returning 0/1/2 (exit 2 when bash is absent, never a false
  pass). Print one OK or FAIL line per check.
- [ ] T015 [US3] Add the `wrappers` step to `.github/workflows/ci.yml` (a pinned
  artifact): `cargo run --package xtask -- wrappers`, after the `license` step, on
  both the ubuntu and windows legs. Record it in the decisions fragment (T002).

## Phase 6: Polish & Cross-Cutting

- [ ] T016 Run `cargo xtask wrappers` and confirm exit 0; temporarily introduce a
  violation in each wrapper and confirm it exits non-zero naming the script, then
  revert (SC-001).
- [ ] T017 Run `cargo xtask ci` in the foreground and watch it to completion (the
  `wrappers` gate included); then run `cargo xtask neutral`. Fix any failure
  before proceeding (SC-006).
- [ ] T018 Final review pass: confirm no em/en dashes, SPDX present (line 1 for
  `.ps1`, line 2 for `.sh`), UTF-8/LF/single-trailing-newline, no emoji, that
  neither wrapper contains capture or output-parsing logic (P-7) and neither
  installs the driver (P-1), and that every new term has its glossary entry (P-6,
  P-8). Run `shellcheck scripts/fragcap.sh` if available and resolve findings.

## Dependencies

- Phase 1 (Setup) has no code dependencies.
- Phase 2 (Foundational): T004 depends on T003; T005/T006 are the checker; T007
  depends on T005. The lint refinement (T004) unblocks a shebang-first `fragcap.sh`.
- Phase 3 (US1) depends on Phase 2 (the gate to check against).
- Phase 4 (US2) depends on Phase 2 (the checker and the lint refinement).
- Phase 5 (US3) depends on Phases 3 and 4 (the wrappers to check) and Phase 2.
- Phase 6 depends on all prior phases.

## Parallel opportunities

- T002 runs parallel to T001.
- US1 (Phase 3, PowerShell) and US2 (Phase 4, Bash) are independent scripts and
  can be authored in parallel once Phase 2 lands.

## Implementation strategy

Foundational first: the lint refinement (so a shebang-first script is legal) and
the authored Bash checker (unit-tested against known-bad input). Then the two
wrappers, each to its standard and to its checker. Then the gate wiring into
`ci` and `ci.yml`. The wrappers' full runtime behavior is tier 2; tier 1 proves
the checkers, the syntax, and the help and dry-run seams.
