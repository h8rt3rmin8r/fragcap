# Implementation Plan: Warm-To-Cold Restart

**Branch**: `codex/113-warm-to-cold-restart` | **Date**: 2026-09-01 | **Spec**: [spec.md](spec.md)

## Summary

S113 closes #309 with an explicit preflight workflow ahead of the existing Deep Capture coordinator. A pure facade model describes the observed warm class, declared images, effective two-minute-or-shorter deadline, and no-process-control policy. The CLI presents and confirms that plan, polls the existing no-handle image snapshot while the operator closes the application normally, then resolves the target and launch state again. Only an exact corresponding cold case proceeds to ordinary preflight. A second confirmation binds authorization to the newly prepared plan before effects.

This deliberately declines automated graceful shutdown. The snapshot cannot prove which same-named process belongs to the selected target, so sending even a polite close request could affect unrelated software. Normal application exit remains operator-owned.

## Technical Context

**Language/Version**: Rust 1.96.0 pinned, Rust 1.88 MSRV

**Dependencies**: Existing standard library, target resolver, no-handle Toolhelp snapshot, Deep Capture launch cases, managed launch, and CLI event emitter

**Storage**: Existing target store and session bundle; no schema migration

**Testing**: Pure state-transition unit tests, CLI argument and event tests, controlled integration tests, and full repository gates

**Target Platform**: Windows 10 and later for live inventory; platform-neutral policy tests

**Constraints**: No process handle, stop, signal, message, kill, relaunch, shell, hook, injection, global proxy, new dependency, or effect before second authorization

**Scale/Scope**: One selected target, one bounded declared image set, one warm-to-cold transition, and one resulting session

## Constitution Check

### Pre-design gate

- **P-1**: PASS. The product performs no target process control and uses only the existing read-only process snapshot.
- **P-2**: PASS. No platform code enters core.
- **P-3**: PASS. Capture and attribution remain unchanged.
- **P-4**: PASS. Every wait and inventory failure is explicit.
- **P-5**: PASS. Existing artifact formats remain compatible.
- **P-6**: PASS. Warm, cold, managed launch, and authorization vocabulary is reused.
- **P-7**: PASS. Wrappers are unchanged.
- **P-8**: PASS. House format, lint, encoding, and test gates apply.
- **P-9**: PASS. Image presence is described as uncertain observation, never ownership.
- **P-10**: PASS. Target resolution is rerun through the sole existing path.
- **P-11**: PASS. Architecture documents advance with the implementation.

### Post-design gate

All checks remain PASS. Research rejects automatic close messages, force termination, path queries through process handles, stale warm-plan reuse, infinite waiting, and calibration expansion.

## Project Structure

```text
specs/113-warm-to-cold-restart/
├── checklists/{requirements,security}.md
├── contracts/warm-restart-api.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md

crates/fragcap/src/deep_capture/
├── mod.rs
└── restart.rs

crates/fragcap-cli/src/
├── cli.rs
├── events.rs
└── commands/deep_capture.rs

crates/fragcap-cli/tests/cli_deep_capture.rs
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
changelog.d/S113-warm-to-cold-restart.{added,decisions}.md
```

**Structure Decision**: Pure restart policy belongs in the facade Deep Capture module. The CLI owns presentation, consent, real snapshots, and bounded waiting. Existing session code owns every later effect and cleanup.

## Complexity Tracking

No constitution exception or new dependency is required.
