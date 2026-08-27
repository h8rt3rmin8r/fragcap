# Data Model: Vendored Bash Wrapper Checker

## Wrapper Script

A repository script covered by `cargo xtask wrappers`.

Fields:

- `label`: Human-facing script name for gate output.
- `relative_path`: Repository-root-relative path passed to Bash or PowerShell.
- `language`: Bash or PowerShell.
- `seam_checks`: Runtime checks owned by fragcap rather than the vendored compliance checker.

Validation and behavior:

- Bash scripts are `scripts/fragcap.sh`, `scripts/lint-docs.sh`, and `scripts/cut-release.sh`.
- PowerShell scripts remain `scripts/Invoke-FragCap.ps1` and `scripts/New-Release.ps1`.
- Missing scripts fail the gate.
- Relative paths are used for checker invocations from the repository root.

## Compliance Checker

A vendored standard checker under `.agents/skills`.

Fields:

- `relative_path`: Repository-root-relative checker path.
- `language_standard`: Bash or PowerShell.
- `runner`: Bash for both vendored POSIX checker scripts.

Validation and behavior:

- The Bash checker path is `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh`.
- The PowerShell checker path remains `.agents/skills/shruggie-powershell/scripts/test-script-compliance.sh`.
- Missing checker bytes fail the gate.

## Environment Precondition

A required executable for the wrapper gate.

Fields:

- `name`: Bash, ShellCheck, or PowerShell 7.
- `visibility`: Host process or Bash subprocess, depending on where the executable is used.
- `failure_exit`: Exit code 2 through `cargo xtask wrappers`.

Validation and behavior:

- Bash must be available to run vendored checkers and Bash syntax checks.
- ShellCheck must be runnable from Bash.
- PowerShell 7 must be available to parse and run PowerShell wrappers.
