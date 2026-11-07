# Implementation Plan: Causal Launch-Chain Coverage

**Branch**: `codex/s197-causal-chain-coverage` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/197-causal-chain-coverage/spec.md`

## Summary

Separate Deep Capture packet membership from its application inspection choice. Use exact session-created process generations to admit causal-chain packets from launch inception through route-owner release, including undeclared descendants. Keep the packet pipeline active through release, carry typed admission reasons to loss and correlation, and extend bounded shutdown to every proven owned descendant. Change `auto` to full-chain inspection, invalidate old policy facts, and refuse resume policy drift before effects.

## Technical Context

**Language/Version**: Rust 1.88 workspace, PowerShell 7 and Bash repository tooling, MDX documentation site

**Primary Dependencies**: Existing workspace dependencies only, including clap, arc-swap, Windows process and socket APIs, native proxy and facade crates

**Storage**: Existing SQLite compatibility fields and versioned session artifacts; policy identifiers advance without a new SQLite column or target store

**Testing**: Rust unit and controlled integration tests, CLI/parser and output tests, replay packet conservation tests, Windows managed-process tests, hosted package certification, full `cargo xtask ci`, exact-head CI

**Target Platform**: Windows 11 product; Linux and Windows CI for platform-neutral contracts

**Project Type**: Multi-crate Rust library and CLI with Windows packaging and static documentation

**Performance Goals**: Existing finite capture and process-event budgets remain enforced; delayed proof uses the existing single bounded packet buffer plus at most one writer-held head packet, with a managed-session byte ceiling, preserved observed order, and counted saturation; no blocking process query on the packet writer

**Constraints**: No new dependency, target instrumentation, packet interception driver, ordinary Capture process-control authority, second packet buffer, unbounded packet spool, game-only fact promotion, or claim of #572 direct routing or #574 TLS compatibility

**Scale/Scope**: Three issues (#570, #571, #573), one internal Deep Capture packet scope, one typed gate decision contract, one capture-tail lifecycle, one policy version change, existing 64 managed-receipt ceiling

## Constitution Check

*GATE: Passed before research and rechecked after design.*

- **P-1**: Existing Npcap observation, ETW/process/socket evidence, and the authorized exact descendant shutdown exception are sufficient. Control refuses incomplete process-generation proof; no target instrumentation or system-wide route is added.
- **P-2/P-3**: Core owns only platform-neutral gate and omission types. Windows proof remains in facade/CLI adapters; packet acquisition remains separate from attribution.
- **P-4/P-9**: Every withheld or staged packet has an actual counted outcome. Bounded staging preserves output order and refuses complete coverage after saturation. A direct socket never becomes a positive application-inspection fact.
- **P-5**: Existing pcapng remains readable by unmodified analyzers; artifact changes are additive with explicit schema/version handling.
- **P-6**: Public packet-scope and rejection vocabulary receives glossary coverage.
- **P-7/P-8**: No wrapper owns behavior. UTF-8, LF, Markdown, Rust formatting, and hidden Windows child launches remain required.
- **P-10/P-11**: Compatibility uses the existing target store; the master specification and chronology describe the S197 candidate separately from published v0.10.14.
- **Workflow**: Full Spec-Kit analysis, controlled tests, local gate, exact-head hosted checks, initial reviews and at most one requested second round precede owner final review. The current user instruction explicitly authorizes push and official PR creation.

Post-design recheck: passed. The new packet scope is internal to managed Deep Capture, policy identity is exact and versioned, process control retains P-1 proof, and each finite failure remains visible.

## Project Structure

### Documentation (this feature)

```text
specs/197-causal-chain-coverage/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/evidence-contract.md
├── checklists/requirements.md
├── checklists/causal-coverage.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/{traits.rs,flow.rs,pipeline/mod.rs}
crates/fragcap/src/{session.rs,managed_launch.rs,deep_capture/process.rs,deep_capture/inspection.rs,deep_capture/proposal.rs,deep_capture/native.rs}
crates/fragcap-cli/src/{cli.rs,orchestrator.rs,commands/calibrate.rs,commands/deep_capture.rs,commands/deep_capture/shutdown.rs}
docs/{fragcap-specification.md,plans/README.md,glossary/}
site/content/docs/
```

**Structure Decision**: Extend the existing gate, process evidence, compatibility case, and shutdown paths in place. One internal causal-generation authority is shared by packet admission, role stamping, endpoint narrowing, and release watch; CLI presentation consumes it.

## Implementation Sequence

1. Establish red controlled regressions for the three defects and cross-cutting packet, process, and resource conservation.
2. Introduce a typed gate result and cause-preserving flow omission without changing ordinary Capture.
3. Add generation-aware causal membership and a timed pending decision for the existing writer-held head packet while the one existing bounded buffer remains the sole fan-in; bind loopback orientation and endpoint narrowing to the same authority.
4. Separate client observation completion from packet pipeline finalization so managed Deep Capture records during authorized shutdown and route release, then finalizes once with explicit coverage bounds.
5. Expand receipt acquisition and shutdown selection to proven causal descendants, retaining exact identity recheck, minimal rights, finite deadlines, and incomplete-state recovery.
6. Resolve `auto` to full-chain inspection throughout CLI/facade, version policy facts, reject stale resume, and preserve explicit game-only application treatment with full-chain packets.
7. Reconcile artifacts, calibration, Doctor, terminal output, documentation, package certification, and changelog; verify and review at exact head.

## Complexity Tracking

No constitution violation requires justification.
