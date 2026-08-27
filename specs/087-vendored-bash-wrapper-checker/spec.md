# Feature Specification: Vendored Bash Wrapper Checker

**Feature Branch**: `codex/087-vendored-bash-wrapper-checker`

**Created**: 2026-08-27

**Status**: Draft

**Input**: User description: "Spec out S087 as defined and run it end-to-end"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One Bash Standard Authority (Priority: P1)

A maintainer running `cargo xtask wrappers` sees every Bash script under the wrapper gate checked by the vendored ShruggieTech Bash compliance checker rather than by a stale Rust reimplementation.

**Why this priority**: P-8 requires the repository to enforce the house standards. Keeping a second authored Bash checker in `xtask` lets the gate drift from the vendored standard that agents and maintainers are told to follow.

**Independent Test**: Run `cargo xtask wrappers` and verify `fragcap.sh`, `lint-docs.sh`, and `cut-release.sh` each report a vendored Bash checker result before their syntax and seam checks.

**Acceptance Scenarios**:

1. **Given** the vendored Bash checker and `scripts/fragcap.sh` are present, **When** `cargo xtask wrappers` runs, **Then** the checker is invoked for `fragcap.sh` and a checker failure increments the wrapper failure count.
2. **Given** the vendored Bash checker and `scripts/lint-docs.sh` are present, **When** `cargo xtask wrappers` runs, **Then** the checker is invoked for `lint-docs.sh` and a checker failure increments the wrapper failure count.
3. **Given** the vendored Bash checker and `scripts/cut-release.sh` are present, **When** `cargo xtask wrappers` runs, **Then** the checker is invoked for `cut-release.sh` and a checker failure increments the wrapper failure count.
4. **Given** any gated Bash script or the vendored Bash checker is missing, **When** `cargo xtask wrappers` runs, **Then** the missing file is reported as a failed check rather than ignored.

---

### User Story 2 - Static Analysis Cannot Be Skipped (Priority: P2)

A maintainer whose shell environment cannot run ShellCheck from Bash receives an environment failure instead of a passing wrapper gate with static analysis skipped.

**Why this priority**: The vendored Bash checker warns and continues when ShellCheck is absent. That is useful for direct ad hoc use, but CI needs a hard distinction between "scripts comply" and "the required analyzer did not run."

**Independent Test**: Run the wrapper gate in an environment where Bash cannot resolve `shellcheck`; `cargo xtask wrappers` exits 2 with an actionable missing-ShellCheck message.

**Acceptance Scenarios**:

1. **Given** Bash is installed but ShellCheck is not runnable from Bash, **When** `cargo xtask wrappers` runs, **Then** it exits as unable to run before reporting compliance success.
2. **Given** ShellCheck is visible to PowerShell but cannot be run from Bash, **When** `cargo xtask wrappers` runs, **Then** Bash visibility is authoritative because the vendored Bash checker runs inside Bash.
3. **Given** Bash, Bash-visible ShellCheck, and PowerShell 7 are available, **When** `cargo xtask wrappers` runs, **Then** all existing syntax, help, dry-run, and PowerShell checks still run.

### Edge Cases

- The PowerShell checker path and PowerShell parser checks must remain unchanged.
- Missing Bash itself remains an environment failure.
- Missing PowerShell 7 remains an environment failure.
- The direct vendored checker may still warn when invoked by hand without ShellCheck; only `cargo xtask wrappers` strengthens that condition for CI.
- Relative repository paths remain the invocation contract so Git Bash, WSL Bash, and native Bash can resolve targets consistently from the repository root.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `cargo xtask wrappers` MUST invoke `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh` for `scripts/fragcap.sh`.
- **FR-002**: `cargo xtask wrappers` MUST invoke `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh` for `scripts/lint-docs.sh`.
- **FR-003**: `cargo xtask wrappers` MUST invoke `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh` for `scripts/cut-release.sh`.
- **FR-004**: A non-zero vendored Bash checker result MUST increment the wrapper failure count and produce exit code 1 from `cargo xtask wrappers`.
- **FR-005**: A missing gated Bash script or missing vendored Bash checker MUST increment the wrapper failure count rather than produce a clean pass.
- **FR-006**: `cargo xtask wrappers` MUST return an unable-to-run result when ShellCheck is not runnable from Bash.
- **FR-007**: Bash-visible ShellCheck absence MUST map through the existing xtask exit-code contract as exit code 2.
- **FR-008**: Existing `bash -n`, `--help`, `fragcap.sh --dry-run`, PowerShell vendored-checker, PowerShell parser, `-Help`, and `Invoke-FragCap.ps1 -DryRun` checks MUST remain covered.
- **FR-009**: The Rust-authored Bash structural checker MUST be removed or made unreachable so there is only one Bash compliance authority.
- **FR-010**: The master specification MUST describe the vendored Bash checker delegation and the hard ShellCheck environment precondition.
- **FR-011**: The change MUST NOT alter any shipped wrapper script behavior, capture behavior, CLI command contract, dependency graph, workflow file, or vendored checker bytes.

### Key Entities

- **Vendored Bash Checker**: `.agents/skills/shruggie-bash/scripts/test-script-compliance.sh`, the authoritative checker for ShruggieTech Bash script structure and ShellCheck integration.
- **Bash-runnable ShellCheck**: A ShellCheck executable resolvable by the same Bash environment that runs the vendored checker. On Windows this may be exposed as `shellcheck.exe`.
- **Wrapper Gate**: `cargo xtask wrappers`, the repository gate that composes vendored compliance checks, syntax checks, and wrapper seam checks under the 0/1/2 exit-code contract.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo xtask wrappers` reports vendored Bash checker results for all three gated Bash scripts.
- **SC-002**: The old Rust Bash structural checker and its rule-unit tests are absent from `xtask/src/wrappers.rs`.
- **SC-003**: A Bash environment that cannot run ShellCheck returns exit 2 from `cargo xtask wrappers`.
- **SC-004**: `scripts/fragcap.sh`, `scripts/lint-docs.sh`, and `scripts/cut-release.sh` pass the vendored Bash checker when Bash can run ShellCheck.
- **SC-005**: Focused xtask tests pass.
- **SC-006**: `cargo xtask ci` passes after implementation.

## Assumptions

- CI runners that execute `cargo xtask wrappers` can install or expose ShellCheck to Bash.
- The vendored Bash checker is the standard authority; this slice does not modify its behavior for direct standalone invocation.
- The existing wrapper seam tests remain sufficient for runtime wrapper behavior because this slice changes only compliance-gate authority.
