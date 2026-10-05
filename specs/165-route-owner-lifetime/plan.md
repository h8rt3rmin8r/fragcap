# Implementation Plan: S165 Route Owner Lifetime

**Branch**: `codex/s165-steam-route-lifetime` | **Date**: 2026-10-05 | **Spec**: [spec.md](spec.md)

**Input**: Active-release defect #452 and the approved one-slice repair request.

## Summary

Add a finite route-owner release phase between Capture return and proxy shutdown. The shipped adapter observes only process image inventories, prompts for normal application exit, and records success or explicit unresolved recovery. Apply it to Steam, direct, and publisher child-environment launches, including failed and interrupted capture. Keep proxy stop and trust cleanup under their existing finite budgets.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88.
**Primary Dependencies**: Existing facade, CLI, Windows ToolHelp query-only snapshot, standard library timing.
**Storage**: Existing session journal and bundle; no database migration.
**Testing**: Controlled facade lifecycle and CLI process-inventory tests, `cargo xtask ci`.
**Target Platform**: Windows for managed launch; neutral facade contracts remain platform independent.
**Project Type**: Rust workspace library and CLI.
**Performance Goals**: Snapshot polling at bounded intervals without spawning a new console process.
**Constraints**: No process control or memory handles, no system proxy mutation, finite plan-bound wait, truthful partial cleanup.
**Scale/Scope**: One new session deadline and release result across facade, shipped adapter, tests, and operator guidance.

## Constitution Check

- **P-1**: Pass. Uses the existing query-only ToolHelp snapshot; no process handle with memory rights or target control.
- **P-2/P-3**: Pass. The facade owns lifecycle ordering; CLI supplies platform inventory.
- **P-9**: Pass. Absence is claimed only from a complete inventory; timeout or read failure stays unresolved.
- **P-11**: Pass after master specification and operator documentation state the shipped behavior and its abrupt-termination limit.
- **Plan authority**: Pass after the new finite deadline is included in canonical authorization and bundle fields.

## Project Structure

```text
specs/165-route-owner-lifetime/
  spec.md
  checklists/requirements.md
  research.md
  data-model.md
  contracts/route-owner-release.md
  quickstart.md
  plan.md
  tasks.md
crates/fragcap/src/deep_capture/{adapters,model,session}.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap/tests/deep_capture_session.rs
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

## Phases

1. Extend the plan deadline and facade release barrier. Prove the call order under controlled success and failure.
2. Derive the exact declared route-owner image set and add a bounded query-only polling adapter with timely operator guidance.
3. Update plan/bundle serialization and operator recovery documentation.
4. Run focused controlled tests and the repository CI gate, repair failures, commit, push, open the PR, and stop at owner merge.

## Complexity Tracking

The separate release deadline is necessary because charging normal application shutdown to the existing ten-second proxy stop budget would make the operator instruction ineffective. The owner set is derived from an already authorized launch and is not a new target-discovery or process-control subsystem.
