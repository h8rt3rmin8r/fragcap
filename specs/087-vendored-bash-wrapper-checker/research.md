# Research: Vendored Bash Wrapper Checker

## Decision 1: Delegate Bash compliance to the vendored checker

**Decision**: `cargo xtask wrappers` invokes `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh` for each gated Bash script.

**Rationale**: S071 vendored the Bash standard checker after `xtask` already had a Rust structural checker. Keeping both means two authorities can drift. PowerShell already uses its vendored checker; Bash should follow the same model.

**Alternatives considered**:

- Keep `check_bash` and add more rules: preserves the drift risk issue #199 exists to remove.
- Run both checkers: catches more in the short term, but still leaves two authorities and makes future standard changes ambiguous.
- Modify the vendored checker: out of scope because this slice changes repository gate orchestration, not vendored skill bytes.

## Decision 2: Require ShellCheck from Bash before running the gate

**Decision**: `cargo xtask wrappers` preflights ShellCheck from inside the same plain Bash mode that invokes the vendored checker, and returns an unable-to-run result when that Bash mode cannot run it. On Windows, Bash-visible `shellcheck.exe` is accepted and bridged to the bare command name for the checker process.

**Rationale**: The vendored checker warns and continues when ShellCheck is missing. Direct skill use can tolerate that, but CI cannot report a clean gate when static analysis did not run. Checking inside the checker-equivalent Bash mode is necessary because Windows host shells, Git Bash, WSL, and login-shell initialization can see different executable paths and command-name behavior.

**Alternatives considered**:

- Trust the vendored checker warning: lets CI pass without static analysis.
- Check ShellCheck from Rust using the host `PATH`: can pass on Windows while Bash still cannot run the analyzer.
- Parse checker output for the warning: weaker than an explicit preflight and entangles `xtask` with the checker's text output.

## Decision 3: Preserve seam checks outside the compliance checker

**Decision**: Keep `bash -n`, `--help`, `fragcap.sh --dry-run`, PowerShell parser, PowerShell `-Help`, and PowerShell dry-run checks in `xtask`.

**Rationale**: The vendored compliance checkers validate house-standard script shape and static analysis. The runtime seams prove fragcap-specific wrapper behavior, so they remain local to the repository gate.

**Alternatives considered**:

- Drop the seam checks: would reduce coverage unrelated to the checker authority swap.
- Move seam checks into the vendored checker: would make a generic house-standard checker depend on fragcap-specific behavior.
