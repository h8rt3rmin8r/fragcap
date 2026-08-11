### Decisions

**2026-08-11: shell wrappers (slice S18 sub-slice B). Pinned-artifact and design
decisions.**

- **Pinned artifacts changed, recorded here.** This slice adds
  `scripts/Invoke-FragCap.ps1` and `scripts/fragcap.sh` (both under the pinned
  `scripts/**`) and a `wrappers` step to `.github/workflows/ci.yml` (a pinned
  workflow). The step runs `cargo run --package xtask -- wrappers` on both the
  ubuntu and windows legs, after the licensing step.
- **The un-vendored Bash standard was resolved by authoring, not vendoring.** The
  ShruggieTech PowerShell standard is vendored with a compliance checker; the Bash
  standard and a Bash checker are not on disk, a gap the foundation doc flagged as
  "must be resolved before S18." The operator chose to proceed: `fragcap.sh` is
  authored to the real ShruggieTech Bash standard, and a Bash structural checker
  is authored in `xtask` (a pure function over the file bytes, unit-tested against
  known-bad input like `lint.rs`). `skills-lock.json` is unchanged; vendoring the
  `shruggie-bash` skill itself remains a separate operator tooling task. The
  PowerShell wrapper reuses the vendored `Test-ScriptCompliance.ps1` (its POSIX
  twin, so only bash is needed to run it).
- **A shell script's shebang forced a lint refinement.** `#!/usr/bin/env bash`
  must be a Bash script's first line, which conflicts with the SPDX-first-line
  rule. `xtask/lint.rs` now accepts a first-line shebang and requires the SPDX
  identifier on the second line instead. `xtask` is not a pinned artifact.
- **The gate runs bash with relative paths from the repository root.** An absolute
  drive-letter path (`A:\...`) is not one WSL bash can resolve; a relative path
  under `current_dir(root)` resolves under native bash, Git Bash, and WSL bash
  alike. The PowerShell runtime checks are best-effort: they run when `pwsh` is
  present and are skipped (not failed) when it is not, since the vendored checker's
  POSIX twin already validates the PowerShell script's structure with bash alone.
- **The wrappers' runtime behavior is tier 2.** The elevation self-relaunch, real
  driver and interface detection, live capture, and WSL2 interop against a native
  binary do not run in continuous integration, exactly as live capture has not
  since S09. Continuous integration verifies the compliance checkers, the syntax
  validity, the help paths, and the templating and pass-through through
  `--dry-run`.
