# Implementation Plan: Managed Shutdown and Cleanup Truth

**Branch**: `codex/s176-managed-shutdown-cleanup-truth` | **Date**: 2026-10-21 | **Spec**: `specs/176-managed-shutdown-cleanup-truth/spec.md`

**Input**: Owner-approved S176, #498 and #499, after constitution 1.6.0 and master specification 0.1.96-draft were updated before the feature specification.

## Summary

Retain an exact managed-launch ownership receipt through the Deep Capture session, bind observed descendants to it, and use a finite, pre-authorized close/force/verify state machine during route-owner release. A control request does not prove exit. Doctor reads terminal manifest and cleanup facts as historical authority while the resource journal continues to govern present readiness.

## Technical Context

**Language/Version**: Rust 2024, workspace MSRV 1.88.

**Primary Dependencies**: Existing `fragcap`, `fragcap-cli`, `fragcap-core`, and already pinned Windows API bindings; no new package is planned.

**Storage**: Existing versioned `manifest.json`, `cleanup.json`, `cleanup.jsonl`, resource journal, and process trace. No schema migration or old-bundle rewrite.

**Testing**: Controlled Rust tests, Windows process tests, repository `cargo xtask ci`, hosted CI; optional authorized real-title calibration recorded separately.

**Target Platform**: Windows 10 and later for process control; non-Windows builds preserve explicit unsupported/refusal behavior.

**Project Type**: Native Rust CLI and facade library.

**Performance Goals**: Shutdown work is bounded by the existing 120-second route-owner ceiling. Read-only Doctor classification stays within existing scan and four-MiB manifest bounds.

**Constraints**: No target memory rights; no image-name or PID-only control; no new interactive prompt after plan authorization; no shell or focus-stealing console; existing cleanup order and loss accounting remain authoritative.

**Scale/Scope**: One selected target and one exact managed launch chain per Deep Capture session; bounded declared owner set.

## Constitution Check

*GATE: passed before research; rechecked after design.*

- **P-1**: Constitution 1.6.0 explicitly permits pre-authorized Deep Capture control of exact session-created process objects or causally proven descendants. Each Windows call requests literal minimal query, synchronize, or terminate rights; memory, injection, hooks, and broad process enumeration for control remain prohibited. Capture remains passive.
- **P-2/P-3**: Platform operations stay in the facade or CLI's Windows adapter. `fragcap-core` remains platform-neutral and no capture/attribution dependency edge is added.
- **P-4/P-9**: Release claims require verified exit and failures produce exact partial-cleanup evidence. No silently discarded outcome or widened deadline.
- **P-11**: Existing published version and release markers stay unchanged. Specifications, operator text, tests, and shipped behavior advance together.
- **Workflow**: Specify, clarify, checklist, plan, tasks, analyze, implement, and full verification are required. Real-title field observation is reported independently of repository-controlled completion.

## Research Decisions

1. **Ownership transport**: Keep session-specific, non-serializable query/synchronize process-object receipts separate from exported packet/process facts. Query exact declared images as start events arrive, then retain the narrow handle. Do not retain the broad `std::process::Child` handle. The launch PID and time window bind the exact root; a PID alone never does.
2. **Descendant binding**: Join each retained receipt to ETW creation-time ancestry and the declared launch chain, then revalidate live process creation time, canonical path, account, and session before each control attempt. Missing events or a changed identity is an explicit refusal.
3. **Shared Steam**: If an unrelated title is using Steam, do not control that title or the shared root. An unresolved routed root yields partial cleanup. This preserves the selected target scope rather than promoting the Steam image to authority.
4. **Deadlines**: One monotonic 120-second route-owner budget, graceful requests by the start, up to 60 seconds grace, force by 90 seconds, remainder for verified exit. No reset after a failure.
5. **Doctor authority**: Parse bounded terminal manifest and cleanup projection for historical result and failed resource details. Preserve the existing resource-journal decision for present recoverability. A failed past route owner after later exit is history, not a new fix action.

## Project Structure

### Documentation

```text
specs/176-managed-shutdown-cleanup-truth/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/shutdown-and-history.md
  quickstart.md
  tasks.md
  checklists/{requirements,security}.md
```

### Source Code

```text
.specify/memory/constitution.md
docs/fragcap-specification.md
docs/plans/README.md
AGENTS.md
crates/fragcap/src/managed_launch.rs
crates/fragcap/src/managed_shutdown.rs
crates/fragcap/src/deep_capture/process.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/src/commands/deep_capture/shutdown.rs
crates/fragcap-cli/src/doctor/{probe,residue,checks}.rs
crates/fragcap-cli/tests/{cli_calibrate,cli_doctor}.rs
xtask/src/lint.rs
changelog.d/S176-*.md
```

**Structure Decision**: Use existing facade launch and CLI lifecycle ownership. Do not put Windows process control in core, the proxy, or a new crate. Keep Doctor's historical projection near its bounded manifest reader and residue presentation.

## Execution Order

1. Red regression for readable-journal plus failed manifest; implement Doctor historical projection, human/JSON detail, and current-readiness separation.
2. Red controlled ownership tests; retain and validate exact root/descendant receipts with minimal Windows rights.
3. Extend the pre-effect authorization presentation and immutable identity to include shutdown scope and deadlines.
4. Implement bounded graceful/force/exit-verification route release and lifecycle evidence; keep proxy and trust live through the decision.
5. Cover direct, Steam, publisher, ambiguity, same-name, reused PID, no window, force failure, and interruption; run analysis and CI parity before commit.

## Complexity Tracking

No constitution violation is accepted. The 1.6.0 amendment was made before the S176 feature spec, and process control is restricted to its literal allowlist.
