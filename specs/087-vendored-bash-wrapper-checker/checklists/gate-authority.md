# Gate Authority Checklist: Vendored Bash Wrapper Checker

**Purpose**: Pin the behavioral contract for replacing a local checker with a vendored standard checker
**Created**: 2026-08-27
**Feature**: [spec.md](../spec.md)

- [X] The Bash checker source of truth is the vendored `shruggie-bash` checker.
- [X] Every gated Bash script is named explicitly.
- [X] Missing scripts and missing checker bytes are failures, not skips.
- [X] Missing Bash-visible ShellCheck is an environment failure, not a compliance pass.
- [X] PowerShell checker behavior remains in scope and unchanged.
- [X] Existing wrapper seam checks remain covered.
