# Research: Shell wrappers

Phase 0 decisions for slice S18b, weighed against the constitution, the
architecture of record (specification 18, 17.4, 17.5, 17.6), the vendored house
standards, and the CI structure.

## R1. The un-vendored Bash standard (escalated)

The ShruggieTech PowerShell standard is vendored under
`.agents/skills/shruggie-powershell/` with a compliance checker
(`Test-ScriptCompliance.ps1` and a POSIX twin). The Bash standard and a Bash
compliance checker are not on disk; `docs/plans/000-repository-foundation.md`
records this as a known gap that "must be resolved before S18."

**Decision (operator, 2026-08-11)**: proceed. Author `fragcap.sh` to the real
ShruggieTech Bash standard (available this session as the `shruggie-bash` skill)
and author an in-repo Bash compliance checker enforcing its structure; do not
change `skills-lock.json`. The PowerShell wrapper reuses the vendored checker.
Vendoring the `shruggie-bash` skill itself is a separate operator tooling task.

**Alternatives**: pausing S18b until the operator vendors the skill (rejected:
the operator chose to proceed); shipping PowerShell only (rejected: the operator
chose both). Writing an in-repo checker is not "substituting a different standard"
(the foundation doc's concern): it enforces the real standard's structure, the
same scope the vendored PowerShell checker covers.

## R2. Where the compliance checkers run

The house rule (`xtask/src/main.rs`) is that checks live in `xtask` so they need
nothing installed beyond the toolchain. No shell-lint runs today.

**Decision**: a new `cargo xtask wrappers` subcommand, matching `lint`/`deps`/
`license`, runs the vendored PowerShell checker (its POSIX twin, needing only
bash) on `Invoke-FragCap.ps1`, an authored Bash checker on `fragcap.sh`, `bash
-n`, and the scripts' `--help`/`--dry-run`. It is added to the `ci` aggregate and
`ci.yml`. Returns 0/1/2; a missing bash exits 2 (could not run), never a false
pass.

**Alternatives**: a workflow step invoking the checkers directly (rejected:
duplicates logic across the two CI legs and is not runnable with `cargo xtask
ci` locally); a checker script under `scripts/` (rejected: contradicts the
"checks live in xtask" rationale and would itself need to be a compliant
wrapper). The Bash checker is authored in Rust rather than as another `.sh`
because reading a file and checking structure needs no shell, and a Rust checker
is unit-testable against known-bad input the way `lint.rs` is.

## R3. The SPDX-versus-shebang conflict

`CONVENTIONS.md` requires an SPDX identifier as every source file's first line,
and `xtask/lint.rs` enforces it. A Bash script's `#!/usr/bin/env bash` must be
line 1 for the kernel to honor it (a byte before the shebang, or a `\r` on it,
breaks execution).

**Decision**: refine `xtask/lint.rs` so a source file whose first line is a
shebang (`#!`) carries its SPDX on line 2. `fragcap.sh` is then shebang line 1,
`# SPDX-License-Identifier: Apache-2.0` line 2. `xtask` is not a pinned artifact,
so this needs no dated decision beyond the record here.

**Alternatives**: SPDX line 1 and shebang line 2 (rejected: breaks `./fragcap.sh`
execution); no shebang (rejected: the standard and normal invocation require it).
The `.ps1` has no shebang, so its SPDX is line 1 and its help block follows before
`[CmdletBinding`, which the checker accepts.

## R4. Wrapper scope and the event stream (P-7)

Specification 18.1 lists five responsibilities; 17.5 gives the structured event
stream; P-7 forbids capture logic and output parsing.

**Decision**: both wrappers do only privilege, driver detection, interface
enumeration, path translation, and output templating, and consume the `--json`
event stream on standard error (`session.armed`, `stage.matched`, `stage.exited`,
`filter.narrowed`, `session.complete`). Unrecognized options pass through to
fragcap unchanged, so a new binary flag needs no wrapper change.

**Alternatives**: parsing human-readable progress (rejected outright by P-7 and
17.5); re-implementing interface selection in the wrapper (rejected: it lives in
`fragcap-core::interface::select`, and the wrapper only assists, filtering
virtual adapters from a presented list).

## R5. Tier-1 testability through `--dry-run`

The wrappers' function is a live Windows capture, untestable in CI.

**Decision**: each wrapper accepts `--dry-run` (`-DryRun`): it parses arguments,
expands the output template, assembles the fragcap invocation, prints it, and
exits 0 without elevation, driver detection, or capture. This makes templating
(SC-004) and option pass-through (SC-005) checkable at tier 1 with no capture
driver, and is a useful operator preview. The elevation, driver, enumeration, and
interop paths remain tier 2.

**Alternatives**: a shell unit-test harness (bats, Pester) sourcing the scripts'
functions (rejected for this slice: heavier tooling for thin wrappers, and a
`--dry-run` seam covers the pure logic an operator also benefits from). Relying on
the compliance checker alone (rejected: it checks structure, not behavior, and
SC-004/SC-005 ask for behavioral evidence).

## R6. No new dependency

**Decision**: no crate is added. The gate is Rust in `xtask` shelling out to bash
(and pwsh when present); the checkers are the vendored `.sh` twin and authored
Rust; the wrappers are shell. Nothing enters `Cargo.lock`.
