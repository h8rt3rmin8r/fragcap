# Implementation Plan: S161 Interactive CLI Input Ownership

**Branch**: `codex/s161-interactive-cli-input-ownership` | **Date**: 2026-09-29 | **Spec**: [spec.md](spec.md)

**Input**: Issue #437, S161 specification, and the owner's autopilot authorization.

## Summary

Correct the v0.10.2 self-deadlock caused by retaining the process-global stdin lock through CLI dispatch while Doctor, target registration, and warm restart acquire it again. Keep only a short lived authorization read lock, make Doctor prompt I/O failure an error rather than a false decline, flush the target question, and cover the input ownership and action decisions with bounded controlled regressions. Preserve exact plan authorization and existing command gates.

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.88, pinned toolchain 1.96.0
**Primary Dependencies**: Rust standard library and existing workspace dependencies; no added dependency
**Storage**: No data or artifact schema change
**Testing**: Rust unit and CLI integration tests with bounded input ownership and prompt-result assertions; full `cargo xtask ci`
**Target Platform**: Windows x86-64 production; portable logic tests where possible
**Project Type**: Rust workspace CLI
**Performance Goals**: Prompt responds as soon as a complete line is available; no command-wide stdin lock
**Constraints**: No target, proxy, Capture, trust, routing, or release change; no visible Windows console during automated tests
**Scale/Scope**: One input holder and three interactive prompt paths, plus focused tests and documentation

## Constitution Check

*GATE: Passed before research, rechecked after design.*

- **P-1**: Pass. Input ownership changes no target instrumentation or capture technique.
- **P-2/P-3**: Pass. The correction stays inside the CLI; core, capture, and attribution remain unchanged.
- **P-4/P-5**: Pass. No packet or artifact path changes.
- **P-6/P-7/P-8**: Pass. No new product vocabulary or wrapper logic; Rust and Markdown remain under existing gates.
- **P-9**: Pass. A failed prompt no longer masquerades as an operator decline; no unconfirmed action runs.
- **P-10/P-11**: Pass. Target storage and published release identity remain unchanged. The draft master specification gains only the corrected interactive failure contract.
- **Development workflow**: Spec, clarification, checklist, plan, tasks, analyze, implementation, verification, changelog, and PR all remain required. Issue #437 is the concrete active-release defect; future operator testing is not the issue-completion gate.
- **Pinned artifacts**: No pinned CI workflow, script, toolchain, release configuration, or release documentation is changed.

Post-design check: PASS. The stdin lock is acquired only for an authorization read, each prompt returns a typed decision or error, and controlled tests exercise the deadlock boundary and effect ordering.

## Design Decisions

1. **Acquire stdin only while consuming an authorization response.** Replace the long lived `StdinLock` field with a shared `Stdin` handle and take its lock inside `read_response`. The global buffered stdin remains the single source, so separate plan reads retain buffered bytes and existing exactness checks. Passing one locked reader through every command would expand unrelated command signatures; raw Win32 reads would bypass the established buffer and alter semantics.
2. **Propagate Doctor confirmation failure.** Make `ActionConfirm` return a decision or `CliError`, and make `drive_actions` stop on error. An empty completed line remains No. EOF, read errors, prompt write failures, and flush failures are command failures, never `skipped`. Returning only bool cannot express that distinction.
3. **Flush the target prompt before reading.** The prompt has no newline and currently ignores the write result. Preserve its existing EOF-to-unsure behavior, which authors no socket holder, but propagate write, flush, and read errors.
4. **Do not turn warm-restart input errors into declines.** Preserve complete affirmative and negative responses; report EOF or read errors as failures before restart effects.
5. **Test the actual ownership boundary.** A bounded test must prove constructing the production input holder does not retain the global stdin mutex while another reader needs it. Controlled prompt tests then prove negative, positive, multi-action, and I/O failure effects. Exercise the built CLI through the integrated headless terminal when Doctor offers an action; retain the automated owner and prompt tests as the CI gate.

## Project Structure

### Documentation

```text
specs/161-interactive-cli-input-ownership/
  spec.md
  checklists/requirements.md
  checklists/reliability.md
  plan.md
  research.md
  data-model.md
  contracts/interactive-input.md
  quickstart.md
  tasks.md
```

### Source

```text
crates/fragcap-cli/src/lib.rs
crates/fragcap-cli/src/doctor/fix.rs
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/tests/
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Keep the correction in the existing CLI crate, reuse the existing report and action performer seams, and add no cross-crate interface.

## Complexity Tracking

No constitution violation or new subsystem is required.
