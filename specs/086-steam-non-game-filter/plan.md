# Implementation Plan: Steam Non-Game Filter

**Branch**: `codex/086-steam-non-game-filter` | **Date**: 2026-08-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/086-steam-non-game-filter/spec.md`

## Summary

Fix issue #212 by widening Steam discovery's non-capturable app-type filter beyond `Music` to the Steam app types that cannot be capture targets: `Tool`, `Application`, `Config`, and `Video`, while preserving `Demo`, `Game`, and unknown app types. The implementation stays in the Steam discovery adapter, reuses the existing `considered_not_a_game` account bucket, adds fixture coverage, and updates the master specification.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: Existing workspace crates only (`fragcap`, `fragcap-steam`, `fragcap-targets`)

**Storage**: No storage changes

**Testing**: Focused `cargo test -p fragcap --features targets --test steam_source`, then repository gate

**Target Platform**: Windows-first product behavior, with fixture-backed tests that need no live Steam installation

**Project Type**: Rust workspace CLI and libraries

**Performance Goals**: No additional filesystem, appinfo, catalog, detection, capture, or network work

**Constraints**: Preserve discovery-account conservation, no new dependency, no new storage field, no CLI flag, no name-based exclusion as the primary app filter

**Scale/Scope**: One Steam discovery adapter, its fixture tests, one master-spec revision, one changelog fragment

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice changes only local metadata filtering in discovery and adds no capture, proxy, process, ETW, socket table, trust, or network behavior.
- **P-2 Core Stays Platform-Neutral**: Pass. No `fragcap-core` changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution changes.
- **P-4 No Silent Loss**: Pass. Excluded app types are counted in the existing `considered_not_a_game` bucket and the conservation invariant remains tested.
- **P-5 Compatibility Outranks Richness**: Pass. No capture output format changes.
- **P-6 Glossary First**: Pass. No new domain term is introduced; Steam app type and discovery vocabulary already exist in the specification.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. All generated and edited text must satisfy repository lint, including no em or en dashes.
- **P-9 The Instrument Does Not Lie**: Pass. The filter relies on observed Steam app type and keeps unknown types eligible rather than guessing from names.
- **P-10 One Path To A Target**: Pass. The shared discovery source still yields the same candidate shape and registration path.
- **P-11 The Specification Describes What Shipped**: Pass. The master specification will be updated with the widened filter.

## Project Structure

### Documentation (this feature)

```text
specs/086-steam-non-game-filter/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── steam-non-game-filter.md
├── checklists/
│   ├── app-types.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap/src/discovery.rs
crates/fragcap/tests/steam_source.rs
docs/fragcap-specification.md
changelog.d/212-steam-non-game-filter.fixed.md
```

**Structure Decision**: Keep the behavior in `fragcap`'s `SteamSource` adapter because the decision belongs to target discovery, not to the lower-level Steam library walk. The Steam crate still reports installed titles and app types; the discovery adapter decides which installed apps are capture candidates.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/steam-non-game-filter.md](contracts/steam-non-game-filter.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The design still passes all constitution checks. It changes only target discovery filtering, keeps unknown app types eligible under P-9, and keeps every excluded installed app counted under P-4.
