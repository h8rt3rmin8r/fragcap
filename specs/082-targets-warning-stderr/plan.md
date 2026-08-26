# Implementation Plan: Targets Warning Stream Contract

**Branch**: `codex/082-targets-warning-stderr` | **Date**: 2026-08-26 | **Spec**: `specs/082-targets-warning-stderr/spec.md`

**Input**: Feature specification from `specs/082-targets-warning-stderr/spec.md`

## Summary

Fix issue #205 by threading the existing CLI `Emitter` into the targets command surface and routing every targets warning through `Emitter::warn`. Keep command results on the existing `out` stream, including listings, discovery accounts, registration counts, technology findings, detail views, import/export documents, and ambiguity lists.

## Technical Context

**Language/Version**: Rust 1.82 minimum, current pinned workspace toolchain for full checks

**Primary Dependencies**: Existing workspace crates only; no new dependency planned

**Storage**: Existing `fragcap-targets` local and catalog SQLite stores, unchanged

**Testing**: Focused `cargo test -p fragcap-cli --test cli_targets` coverage, followed by `cargo xtask ci`

**Target Platform**: Cross-platform CLI behavior, with Windows-only Steam enumeration behavior preserved by existing adapters

**Project Type**: Rust CLI and library workspace

**Performance Goals**: No additional discovery, store, or detection work; stream routing only

**Constraints**: Preserve targets command result bytes except for removing warning diagnostics from stdout; preserve exit codes; preserve `doctor --fix` discovery behavior; use the existing emitter verbosity and JSON diagnostic rules

**Scale/Scope**: One CLI dispatch path, the targets command module, focused integration tests, changelog fragment

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- P-1 No covert target instrumentation: PASS. The slice only moves warning diagnostics between CLI streams and adds no observation technique.
- P-2 Core stays platform-neutral: PASS. No `fragcap-core` dependency or platform code changes.
- P-3 Capture and attribution stay separate: PASS. No packet acquisition or attribution behavior changes.
- P-4 No silent loss: PASS. Warnings are preserved as diagnostics except under the already documented `--silent` behavior.
- P-5 Compatibility outranks richness: PASS. No capture output format changes.
- P-6 Glossary first: PASS. No new glossary term is introduced.
- P-7 Wrappers stay thin: PASS. The fix uses the Rust CLI emitter, making shell parsing unnecessary.
- P-8 House standards apply: PASS. Slice artifacts and source changes must pass repository lint, formatting, and text hygiene checks.
- P-9 The instrument does not lie: PASS. Warnings remain warnings and are not converted into command-result rows or discarded outside `--silent`.
- P-10 One path to a target: PASS. Target storage and discovery composition remain unchanged.
- P-11 Specification describes what shipped: PASS. Existing specification sections 17.5 and 17.6 already define the stream contract; no master-spec edit is planned.

## Project Structure

### Documentation (this feature)

```text
specs/082-targets-warning-stderr/
|-- checklists/
|   `-- requirements.md
|-- contracts/
|   `-- targets-warning-streams.md
|-- data-model.md
|-- plan.md
|-- quickstart.md
|-- research.md
|-- spec.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/lib.rs
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/tests/cli_targets.rs
changelog.d/205-targets-warning-stderr.fixed.md
```

**Structure Decision**: Thread `Emitter` into `commands::targets` from the existing top-level dispatch path. Do not add a targets-local diagnostic writer, and do not move command-result rendering out of the targets module.

## Phase 0 Research

### Decision: Route warnings through `Emitter::warn`

`Emitter::warn` already owns the stderr destination, quiet/silent policy, and JSON diagnostic record shape.

**Rationale**: Reusing it satisfies the stream contract in `crates/fragcap-cli/src/lib.rs` and prevents targets from having a second diagnostic implementation.

**Rejected Alternative**: Pass a second raw writer into `targets` and hand-write warning lines. That would fix normal human mode but duplicate quiet, silent, and JSON behavior.

### Decision: Keep command-result facts on `out`

Registration counts, discovery account lines, technology findings, target tables, target detail views, and import/export payloads are command results rather than diagnostics.

**Rationale**: Moving those would break existing command result contracts and make a listing incomplete.

**Rejected Alternative**: Route all discovery text through the emitter. That would hide actual result data under `--quiet` or `--silent` and violate P-9.

### Decision: Preserve `doctor --fix` discovery output while giving it the emitter

The doctor action should still print its action result through the doctor report stream, but warnings produced while running target discovery should use the same diagnostics stream as other warnings.

**Rationale**: This keeps the action result visible while satisfying the global warning contract.

**Rejected Alternative**: Leave `run_discovery_default` without an emitter because it is reached from doctor. That would preserve one known stdout warning leak.

## Phase 1 Design

The targets command surface will accept both streams by taking `&mut Emitter` beside `out`.

- `commands::targets::run(args, out, emitter)` dispatches every targets subcommand.
- `list_default(out, footer, emitter)` keeps the bare `fragcap` path consistent with explicit `targets`.
- Warning-producing helpers accept `&mut Emitter` only where needed.
- Helpers that print command-result facts continue to accept only `out`, unless they also emit warnings.
- Tests drive `fragcap_cli::run_with`, because it captures stdout and stderr separately and exercises the real dispatch and emitter wiring.

The externally observable contract is:

```text
stdout: targets command result bytes only
stderr: warnings/errors in the existing emitter shape
```

In JSON mode, stderr warning records use the existing emitter fields:

```json
{"ts":"<time>","event":"warning","message":"<warning text>"}
```

## Complexity Tracking

No constitution violations.
