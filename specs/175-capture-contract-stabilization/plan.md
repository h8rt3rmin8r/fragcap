# Implementation Plan: Capture Contract Stabilization

**Branch**: `codex/s175-capture-contract-stabilization` | **Date**: 2026-10-21 | **Spec**: [spec.md](spec.md)

**Input**: Owner-approved S175 bundle of #484, #485, #486, #490, #496, and #497.

## Summary

Make Capture's deadline, scope, sink, and help contracts agree with observed output. Separate elapsed deadline time from absolute packet/process evidence; validate roles before side effects; apply direction at the shared write gate with exact accounting; give the pcapng sink a metadata-only mode using a conservative core header boundary; make sink destinations comma-capable; and align help with shipped modes and retained-volume semantics. Keep full-payload output byte-identical.

## Technical Context

**Language/Version**: Rust workspace, edition and MSRV inherited from `Cargo.toml` and `rust-toolchain.toml`.

**Primary Dependencies**: Existing `fragcap-core`, `fragcap`, `fragcap-sink`, `fragcap-profile`, `fragcap-cli`, and clap. No new package is planned.

**Storage**: Existing pcapng, JSON Lines, and session bundle files. No database migration.

**Testing**: Controlled unit, replay integration, and controlled live-source tests; `cargo xtask ci` in the foreground; additional targeted Windows/feature checks if needed.

**Target Platform**: Windows product path, platform-neutral core and offline corpus.

**Project Type**: Rust CLI and library workspace.

**Performance Goals**: One bounded scope decision per packet; no sink-induced capture-thread wait or unbounded header walk.

**Constraints**: P-1 through P-11, especially P-2 platform-neutral core, P-4 counted omissions, P-5 pcapng compatibility, and P-9 faithful evidence; no new runtime dependency; preserve full-payload goldens.

**Scale/Scope**: Six open issues at one Capture contract; no Doctor, target-store, bundle-cleanup, proxy-protocol, or release work.

## Constitution Check

**Before research**: PASS. No process instrumentation or new privilege is proposed (P-1). The core parser remains platform neutral (P-2). The write gate remains outside packet acquisition and attribution (P-3). New omissions receive explicit counters (P-4/P-9). Metadata-only pcapng must remain consumable by unmodified readers (P-5). CLI and documentation wording must describe exactly what ships (P-6/P-11). Existing wrappers remain thin (P-7), and repository formatting applies (P-8). Profile and target resolution remain single-path (P-10).

**After design**: PASS, conditional on tests proving marker absence and pcapng readability. [research.md](research.md) records the alternatives and exact boundary choices. No constitution deviation is proposed.

## Project Structure

### Documentation

```text
specs/175-capture-contract-stabilization/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── contracts/capture.md
├── quickstart.md
└── tasks.md
```

### Source and tests

```text
crates/fragcap-core/src/parse/
crates/fragcap/src/session.rs
crates/fragcap-sink/src/pcapng/
crates/fragcap-sink/src/transport/
crates/fragcap-cli/src/args.rs
crates/fragcap-cli/src/assemble.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-cli/src/events.rs
crates/fragcap-cli/src/output.rs
crates/fragcap-cli/src/live_status/
crates/fragcap-cli/src/cli.rs
crates/fragcap-cli/tests/
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Extend the existing owning modules. The facade owns session scope, core owns packet parsing, sinks own encoding, and the CLI owns grammar, assembly, and presentation. No new cross-crate dependency is introduced.

## Delivery order

1. Run the Spec-Kit analyze gate over the spec, plan, and tasks, and resolve every finding before implementation.
2. Write failing regressions for epoch-stamped duration, role validation, direction retention/accounting, pcapng marker absence, comma destinations, and help/counter examples.
3. Implement the clock separation without changing packet timestamps or the gate's absolute window.
4. Implement role validation and one directional write-gate decision with named counters.
5. Implement conservative core header-boundary reporting and payload-aware pcapng encoders across every sink path.
6. Implement comma destination parsing and help corrections.
7. Complete issue-by-issue checks, run CI parity, commit, push, open the PR with six `Closes` references, and monitor CI and the permitted review rounds. The owner performs final review and merge.

## Complexity Tracking

No constitution violation or new framework is proposed.
