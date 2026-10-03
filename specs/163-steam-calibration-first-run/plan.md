# Implementation Plan: S163 Steam Calibration First Run

**Branch**: `codex/s163-steam-calibration-first-run` | **Date**: 2026-10-03 | **Spec**: [spec.md](spec.md)

**Input**: Owner-approved S163 outline and issues #443 through #447.

## Summary

Make the owned cold Steam root bind when ETW reports a basename, guide first-run client declaration from observed or explicitly selected evidence, distinguish stored launch roles from Steam hints, report the actual failed stage and facts, and document the complete first-run path. Preserve exact authorization, immutable target authority, child-only routing, and conservative compatibility results.

## Technical Context

**Language/Version**: Rust 2021, MSRV 1.88, pinned toolchain 1.96.0
**Primary Dependencies**: Existing workspace crates and Windows ETW/IP Helper facilities; no new dependency planned
**Storage**: Existing versioned local target and append-only compatibility store; no migration planned
**Testing**: Controlled Rust unit and CLI integration tests, `cargo xtask ci`
**Target Platform**: Windows x86-64 production; portable policy tests
**Project Type**: Rust workspace CLI and published documentation
**Performance Goals**: Bounded first-run observation and existing 30 second launch, 60 second observation defaults
**Constraints**: No process handle or instrumentation, no false path or traffic inference, no system-wide proxy change, no publication of operator-specific evidence
**Scale/Scope**: One owned Steam root and one selected target per session; no publisher-chain generalization

## Constitution Check

*GATE: Passed before research; rechecked after design.*

- **P-1**: Pass. ETW and passive flow attribution establish process evidence without process handles or target instrumentation. Deep Capture retains explicit exact authorization.
- **P-2/P-3**: Pass. Process tree remains platform neutral. Windows observation stays in existing adapters; Capture and attribution remain separate.
- **P-4/P-5**: Pass. Observation and proxy counters retain loss accounting and ordinary pcapng compatibility. No packet path is silently suppressed.
- **P-6/P-7/P-8**: Pass. Use existing CLI and facade vocabulary and gates, without a second launch state machine.
- **P-9**: Pass. A Steam hint, manual selection, launch binding, final-client acquisition, and proxy reachability are separate facts. Unavailable evidence remains unavailable.
- **P-10/P-11**: Pass. One stored target and CAS authoring path remain. Draft specification and public docs will describe shipped behavior precisely.
- **Development workflow**: Full Spec Kit analyze and CI parity are required. Controlled evidence closes implementation issues; later field outcomes are separate defects.

Post-design check: PASS. The owned-root rule is receipt-gated, client authoring remains a separate reviewed write, and summaries derive only from exact session evidence.

## Design Decisions

1. **Bind the owned root in the existing session tree.** Arm an exact launch authority before snapshots, then attach receipt PID, expected parent, launch interval, and prepared executable. A basename ETW event may satisfy image identity only under that authority. If ETW reports a path, compare its canonical spelling. Reject prelaunch snapshots and stale instances. Retain raw ETW text. A global basename-only profile relaxation would admit foreign processes.
2. **Keep dispatch dependent on the bound receipt.** The existing one-shot gate consumes only a platform binding from the exact owned root. A later client match and proxy accept remain independent; timeout does not become success merely because the platform binds.
3. **Treat client selection as a separate setup action.** Prefer bounded, read-only ETW process observation joined to IP Helper socket ownership during one owned cold Steam launch. A socket observation identifies a candidate; it never proves packet traffic or proxy reachability. Show one candidate with evidence; show ambiguity or absence explicitly. An operator may declare an exact client manually, labeled as a declaration. Reuse the existing canonical setup plan and compare-and-swap target update after review. Do not reuse ordinary Capture's automatic dominant-image promotion. Ask for a separate plain-language confirmation before starting Steam for this setup observation, then exit setup after authoring so a later calibration gets a fresh cold launch.
4. **Report a causal terminal state.** Derive the earliest known failed stage from launch receipt, process trace, Capture outcome, proxy observations, and exact compatibility writes. Print one cleanup summary from the authoritative results. Report unknown values as unavailable.
5. **Keep publisher matching separate.** Publisher-chain generated stages have a related path-predicate assumption, but this slice repairs the owned Steam root only and makes no publisher completion claim.

## Project Structure

### Documentation

```text
specs/163-steam-calibration-first-run/
  spec.md
  checklists/requirements.md
  checklists/ownership.md
  checklists/ux.md
  plan.md
  research.md
  data-model.md
  contracts/first-run-cli.md
  quickstart.md
  tasks.md
```

### Source

```text
crates/fragcap/src/session.rs
crates/fragcap/src/managed_launch.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/commands/calibrate.rs
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/src/session_ux.rs
crates/fragcap-cli/tests/
site/content/docs/getting-started.mdx
site/content/docs/reference/
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Extend the existing session, calibration, target, and presentation modules, using the existing store CAS and event stream. Add no new product crate.

## Complexity Tracking

No constitution violation or new service is planned.
