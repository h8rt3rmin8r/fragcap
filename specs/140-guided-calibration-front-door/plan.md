# Implementation Plan: Guided Reachability Calibration Front Door

**Branch**: `codex/s140-guided-calibration-front-door` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/140-guided-calibration-front-door/spec.md`

## Summary

Add the first executable child of #380 as a thin top-level `calibrate` command for registered targets. The CLI resolves the existing target authority, obtains one complete query-only process image inventory, loads the target's exact compatibility facts, and passes those values to S139's stable proposal operation with no protocol candidates. A ready proposal emits the durable ordinary Deep Capture command. A warm proposal emits an effect-free operator action unless `--restart-warm` explicitly selects the existing close-and-retry path. One proposed reachability step is translated into the existing low-level `deep-capture --launch --calibrate reachability --calibration-protocol routing` path, preserving the complete S134 authorization and S121 append-only evidence authorities.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap` facade, `fragcap-targets`, and Windows Tool Help bindings already present in the workspace

**Storage**: Existing per-user SQLite target store version 10; read current target facts and append only through the existing session fact adapter; no migration

**Testing**: Focused CLI unit and integration tests, controlled Deep Capture harness, stable event serialization tests, public CLI reference tests, and `cargo xtask ci`

**Target Platform**: Windows production path, with offline argument and no-effect behavior testable on CI hosts

**Project Type**: Rust workspace facade plus command-line binary

**Performance Goals**: One target resolution, one target-scoped fact read, and one process snapshot before any effect; no polling unless `--restart-warm` is explicitly selected

**Constraints**: Feature-branch pull-request workflow; UTF-8 without BOM; LF; 100-column Rust; no new dependency or lockfile package; no process handle; no hidden trust; at most one reachability attempt; no target registration, TLS attempt, multi-attempt loop, or workflow persistence

**Scale/Scope**: One top-level command, one guided event contract, a small refactor exposing existing CLI authorities within the crate, focused offline and controlled tests, two changelog fragments, and architecture-of-record updates

## Constitution Check

*GATE: Passed before research and passed again after design.*

- **P-1**: PASS. Cold detection reads only a Tool Help process snapshot. All effects remain behind explicit Deep Capture selection and the S134 plan identifier.
- **P-2**: PASS. Platform process enumeration remains in `fragcap-cli`; no change enters `fragcap-core`.
- **P-3**: PASS. The command delegates to the existing facade coordinator and changes neither backend.
- **P-4**: PASS. Existing bounded accounting remains unchanged; the wrapper filters no observation.
- **P-5**: PASS. Artifact formats and schemas are unchanged.
- **P-6**: PASS. Existing calibration vocabulary is extended before architecture prose uses the new meaning.
- **P-7**: PASS. No shell wrapper changes; the Rust command remains a thin adapter.
- **P-8**: PASS. All house and repository gates remain mandatory.
- **P-9**: PASS. Unavailable inventory stays unavailable, all limitations remain visible, and readiness requires exact current evidence.
- **P-10**: PASS. Stored-target selection is reused and no registration path is added.
- **P-11**: PASS. The architecture record will describe only this bounded reachability front door and leave #380 open.
- **Third-party obligations**: PASS. No dependency, Npcap artifact, or packaging change.

## Project Structure

### Documentation (this feature)

```text
specs/140-guided-calibration-front-door/
├── checklists/
├── contracts/calibrate-command.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/
├── src/
│   ├── cli.rs
│   ├── commands/calibrate.rs
│   ├── commands/deep_capture.rs
│   ├── commands/mod.rs
│   ├── events.rs
│   └── lib.rs
└── tests/
    ├── cli_args.rs
    ├── cli_calibrate.rs
    ├── cli_deep_capture.rs
    ├── cli_help.rs
    └── cli_reference.rs

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
├── glossary/capture-and-networking.md
└── plans/README.md
```

**Structure Decision**: Keep the new command in `fragcap-cli`, which owns argument mapping, same-process authorization input, presentation, and next-command text. Consume S139 through `fragcap::deep_capture::api`, and delegate every selected effect to `commands::deep_capture::run`. Expose only crate-private target-store, target-resolution, and process-snapshot helpers instead of copying their policy. No public Rust API changes are needed because S139 already established the reusable proposal boundary.

## Phase 0: Research Decisions

### 2026-09-09: Make S140 a one-attempt vertical slice

The full #380 workflow combines registration, multiple trust-bearing attempts, protocol discovery, resumable state, and final coverage. S140 stops after either one reachability attempt or an effect-free readiness decision. This produces a useful front door while ensuring the first command cannot silently cross from routing measurement into certificate trust.

### 2026-09-09: Adapt into the existing low-level command path

The guided command constructs the exact low-level reachability arguments and calls the existing implementation in the same process. A separate coordinator or duplicated adapter set would create two authorization, cleanup, and fact-write paths. Spawning a child process would break the same-process structured authorization contract.

### 2026-09-09: Use durable identifiers in generated commands

The stable identifier is unambiguous, survives listing reorder, and avoids shell quoting. Handles and names remain accepted inputs but are not copied into paste-ready commands.

### 2026-09-09: Keep warm restart explicit

Warm detection produces effect-free guidance and a `--restart-warm` rerun. That flag reuses S113's bounded close-and-retry flow. Automatically entering a wait would weaken deliberate operator selection.

## Phase 1: Design

The command contract is defined in [contracts/calibrate-command.md](contracts/calibrate-command.md), typed decision state in [data-model.md](data-model.md), and executable validation in [quickstart.md](quickstart.md). No entity requires storage and no new dependency or facade export is needed.

## Complexity Tracking

No constitution violation or exceptional complexity is required.
