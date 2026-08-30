# Implementation Plan: Managed Direct-Executable Launch

**Branch**: `codex/101-managed-direct-executable-launch` | **Date**: 2026-08-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/101-managed-direct-executable-launch/spec.md`

## Summary

Add a public `fragcap` managed-launch model that represents Steam protocol and direct-executable launches as immutable variants. Resolve a stored target's direct executable beneath its install root during Capture preparation, deriving that root from a legacy authored absolute executable when necessary, retain that exact value through Deep Capture preflight, and execute it after the existing watcher and packet pipeline are armed. Direct execution uses an explicit program, working directory, ordered arguments, and target-scoped environment additions with no command shell. Deep Capture accepts only cold direct launch, applies its loopback proxy environment to the retained launch, and preserves partial cleanup truth if child creation fails.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.82

**Primary Dependencies**: Existing workspace crates and Rust standard library process APIs

**Storage**: Existing target-owned SQLite row and JSON launch entries, with no schema change

**Testing**: Public facade unit and integration tests, CLI offline tests, controlled local child process, and full repository gates

**Target Platform**: Windows product path, with portable side-effect-free model tests

**Project Type**: Rust workspace library facade plus CLI application

**Performance Goals**: One target and filesystem resolution per preparation; no post-effect store or path lookup

**Constraints**: No shell evaluation, process inspection, target memory access, system proxy mutation, second storage shape, or Steam behavior change

**Scale/Scope**: One stored target and one managed launch per Capture or Deep Capture session

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1 No covert target instrumentation**: Pass. Direct launch creates an operator-selected child with explicit argv and scoped environment, then drops creation ownership. Existing ETW and socket-table observation perform attribution. No target inspection handle, memory read, injection, hook, driver, or system proxy mutation is added.
- **P-2 Core stays platform-neutral**: Pass. Launch preparation and execution live in the `fragcap` facade. `fragcap-core` is unchanged.
- **P-3 Capture and attribution stay separate**: Pass. The existing Capture pipeline, watcher, and flow registry are reused unchanged.
- **P-4 No silent loss**: Pass. Preparation and spawn failures are named, and Deep Capture records partial cleanup truth for effects acquired before spawn.
- **P-5 Compatibility outranks richness**: Pass. Output formats and packet truth are unchanged.
- **P-6 Glossary first**: Pass. Managed direct launch and target-scoped environment inheritance are defined before broader documentation changes.
- **P-7 Wrappers stay thin**: Pass. No wrapper logic is added.
- **P-8 House standards apply**: Pass. Encoding, formatting, lint, test, license, documentation, MSRV, and full gates remain required.
- **P-9 The instrument does not lie**: Pass. Ambiguous, missing, escaping, or changed executables are refused. Arguments remain explicit values, and cleanup is reported rather than inferred.
- **P-10 One path to a target**: Pass. The selected `TargetEntry` remains the only identity and fact owner. Preparation derives from its launch entries and stored or authored-path-derived install root once.
- **P-11 The specification describes what shipped**: Pass. Master specification, outline, glossary, security guidance, CLI reference, and changelog change with implementation.

Post-design re-check: passed. The immutable launch enum removes the current CLI-only Steam type from Capture configuration without introducing another target resolver or storage model. No constitution exception or complexity waiver is required.

## Project Structure

### Documentation (this feature)

```text
specs/101-managed-direct-executable-launch/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── managed-launch.md
├── checklists/
│   ├── launch-safety.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/
├── lib.rs
└── managed_launch.rs              # public immutable launch model and execution

crates/fragcap-cli/src/
├── assemble.rs                    # effective config carries the shared launch enum
├── orchestrator.rs                # executes the prepared launch after arm
└── commands/
    ├── capture.rs                 # retains the selected entry through preparation
    ├── target_resolve.rs          # returns the selected target with its profile
    └── deep_capture.rs            # cold-direct selection and scoped environment adapter
```

**Structure Decision**: The facade is the lowest existing product layer that can depend on both Steam and target storage without coupling sibling crates. Capture and Deep Capture consume one public launch value from there, while the CLI remains responsible for resolving command selectors.

## Complexity Tracking

No constitution violation requires justification.
