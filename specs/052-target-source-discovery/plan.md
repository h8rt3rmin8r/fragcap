# Implementation Plan: TargetSource discovery seam and discovery tiers

**Branch**: `052-target-source-discovery` | **Date**: 2026-08-17 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/052-target-source-discovery/spec.md`

## Summary

Every origin of a capture target becomes one implementor of a single seam,
`TargetSource`, that produces `CandidateTarget` values and a truthful per-run
account. Steam's existing library and app-metadata walk is refactored onto the
seam as `SteamSource` with no observable change; a `KnownRootsSource` walks a
fixed list of game-only directories across every eligible fixed volume so a
machine without Steam still lists games; `DirectorySource` and `InteractiveSource`
back the user-pointed `targets scan`/`targets add` entry points. Tiers 2 and 3
classify a directory by its shape through a `DirectoryClassifier` seam and stop
descending on a hit (the signature matcher itself is S053; this slice ships the
seam and the stop-on-hit contract). A persistent, allowlist-shaped volume
eligibility table in `local.db` keeps the cross-volume walk safe; on first run it
is seeded with the fixed volumes then present, and a later-appearing or
misreporting volume needs explicit opt-in.

Two operator clarifications shape the approach (spec Clarifications, 2026-08-17):
the eligibility table is permissive-seeded at first run then behaves as an
allowlist (FR-016a); discovery surfaces candidates live and a candidate is
persisted to `local.db` only when the user acts on it (FR-020/FR-021), so this
slice's only durable write is the eligibility table.

The load-bearing architecture decision: the seam, the candidate type, the pure
tiers, the `DirectoryClassifier` seam, and the eligibility store live in
`fragcap-targets`, which stays portable and free of both the Windows API and
`fragcap-steam`; the two platform-touching adapters (`SteamSource`, wrapping
`fragcap-steam`'s walk, and the real Win32 volume inventory) live in the `fragcap`
facade, the one crate that legitimately depends on both leaf crates. This adds no
new inter-crate edge to `xtask/src/deps.rs` and keeps `fragcap-targets` building
under the GNU host toolchain.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (built/released on the pinned
toolchain; see `rust-toolchain.toml`).

**Primary Dependencies**: no new crate is required. Steam walking reuses
`fragcap-steam` (`library`, `appinfo`); the catalog join reuses
`fragcap-targets::catalog`/`Store`; the eligibility table reuses `rusqlite`
(bundled SQLite, already present behind the `targets` feature). The real volume
inventory uses `windows-sys` (already workspace-pinned at 0.36, so no `Cargo.lock`
delta) behind `cfg(windows)` in the facade; `GetLogicalDrives`, `GetDriveTypeW`,
and `GetVolumeInformationW`/volume-GUID enumeration supply the fixed-volume list
and its stable identity.

**Storage**: SQLite via `rusqlite`. One new `volume_eligibility` table added by an
additive migration bumping the shared schema from version 3 (S051) to version 4.
Conceptually `local.db` only; the shared store type carries the table on both
files (catalog leaves it empty), consistent with the S050/S051 decision that later
slices add their own tables to `local.db`.

**Testing**: `cargo test --workspace --locked`. Every source is exercised through
injected seams (`VolumeInventory`, a directory lister, a scripted confirmation, a
fixture `DirectoryClassifier`, and `FixtureSource`) so the whole discovery model
runs with no filesystem, no live platform, and no Steam install. In this
environment SQLite-backed crates build under the GNU host toolchain
(`cargo +1.96.0-x86_64-pc-windows-gnu test --workspace`); CI runs the real MSVC
build. The Win32 volume adapter is `cfg(windows)` and exercised only where a real
machine runs it; its consumers are tested against the fixture inventory.

**Target Platform**: Windows (capture host). The discovery model in
`fragcap-targets` is pure/portable computation; the Windows-specific volume
enumeration and the Steam walk are confined to `cfg(windows)`/facade adapters, so
`fragcap-core` gains nothing (P-2) and `fragcap-targets` stays platform-neutral.

**Project Type**: Single Rust workspace (CLI + libraries).

**Performance Goals**: The known-roots walk is the one cost that matters, and the
stop-on-hit descent contract (FR-015) is what bounds it: each directory is tested
for a signature and descent stops on a hit, so the walk never enumerates the
thousands of executables an exhaustive scan would (FR-009). No per-packet cost is
introduced; discovery runs at listing/authoring time, not during capture.

**Constraints**: Add no dependency to `fragcap-core` and no new inter-crate edge
(the facade already depends on both `fragcap-steam` and `fragcap-targets`);
`fragcap-targets` stays free of the Windows API and of `fragcap-steam`; every
considered candidate lands in exactly one counted outcome (P-4); an unknown
candidate is classified unknown, never dropped or guessed (P-9); Steam's
observable candidate set does not change (FR-006).

**Scale/Scope**: A known root holds at most a few hundred game directories; a
machine has a handful of fixed volumes. Linear walks are acceptable; the
eligibility table is keyed on a stable volume identity and is small.

## Constitution Check

*GATE: evaluated before Phase 0 and re-checked after Phase 1 design.*

- **P-1 Passive Observation Only (NON-NEGOTIABLE)**: PASS. Discovery reads the
  filesystem, the Steam metadata files, the catalog, and the volume list, and
  writes only the eligibility table. It opens no process handle, reads no process
  memory, intercepts nothing, and launches nothing (`InteractiveSource`'s
  confirmation is a console prompt, not a process start). `cargo xtask lint` still
  asserts the absence of `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`.
- **P-2 Core Stays Platform-Neutral**: PASS. Nothing lands in `fragcap-core`. The
  pure discovery model lands in `fragcap-targets` (which keeps zero platform
  deps); the Win32 volume adapter and the Steam-walk adapter land in the facade
  behind `cfg(windows)`.
- **P-3 Capture And Attribution Stay Separate**: PASS. Discovery is neither
  capture nor attribution; it touches neither `fragcap-capture` nor `fragcap-attr`
  and adds no edge between them. `cargo xtask deps` sees no new inter-crate edge at
  all (facade already reaches both leaf crates).
- **P-4 No Silent Loss**: PASS by design. Every source returns a discovery account
  whose named outcome counts reconcile to the number of candidates considered
  (FR-002), mirroring `SeedSummary::is_conserved`. A discard path with no counter
  is a defect the account's conservation test catches.
- **P-6 Glossary First**: ACTION (a task). New terms enter the glossary in the
  same change: `TargetSource`, `CandidateTarget`, discovery tier, known-roots
  source, directory source, interactive source, discovery account, volume
  eligibility table, and the descent stop-on-hit contract.
- **P-7 Wrappers Stay Thin**: PASS. The CLI `targets scan`/`targets add` wiring is
  a thin call into the facade's discovery; no output is parsed by a wrapper.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS. An appid absent from
  the catalog, or a directory whose classification is unknown, is surfaced as
  unknown, never dropped and never guessed (FR-005, edge cases).
- **P-10 One Path To A Target**: PASS. This slice operationalizes P-10: every
  source, whatever its batch size, yields the same `CandidateTarget`, and every
  candidate becomes a durable target through the one S051 entry model.
- **P-11 The Specification Describes What Shipped**: ACTION (a task). Master
  specification section 7 is reconciled with what ships, and the changelog fragment
  carries the `<!-- spec-impact: 7 -->` header the S049 gate requires.

No violations to justify; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/052-target-source-discovery/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── target-source.md
│   ├── discovery-account.md
│   └── volume-eligibility.md
├── checklists/
│   └── requirements.md  # Spec quality checklist (from /speckit-specify)
└── tasks.md             # /speckit-tasks output (not created here)
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/
├── source.rs            # NEW: TargetSource trait, CandidateTarget, DiscoveryAccount
├── classifier.rs        # NEW: DirectoryClassifier seam + trivial/fixture impl (matcher is S053)
├── sources/
│   ├── mod.rs           # NEW: source family re-exports
│   ├── known_roots.rs   # NEW: KnownRootsSource (pure; injected VolumeInventory + lister)
│   ├── directory.rs     # NEW: DirectorySource
│   └── interactive.rs   # NEW: InteractiveSource (injected confirmation)
├── volume.rs            # NEW: Volume, VolumeInventory seam, eligibility seed/query logic
├── schema.rs            # EDIT: SCHEMA_VERSION 3->4, volume_eligibility DDL, MIGRATE_3_TO_4
├── store.rs             # EDIT: eligibility upsert/query; sequential migration step
└── lib.rs               # EDIT: re-export the new surface

crates/fragcap/src/
├── discovery.rs         # NEW (facade): SteamSource adapter + Win32 VolumeInventory (cfg(windows))
└── lib.rs               # EDIT: expose discovery composition to the CLI

crates/fragcap-cli/src/commands/targets.rs   # EDIT: wire `targets scan <dir>` and `targets add <exe>`

crates/fragcap-targets/tests/               # NEW: source_seam.rs, known_roots.rs, volume_eligibility.rs
crates/fragcap/tests/                        # NEW/EDIT: steam_source.rs (refactor parity), discovery.rs
docs/glossary/                               # EDIT: new term entries (P-6)
docs/fragcap-specification.md                # EDIT: section 7 reconciliation (P-11)
changelog.d/S052-target-source-discovery.added.md   # NEW: spec-impact header
```

**Structure Decision**: Single Rust workspace. The pure discovery model (seam,
candidate, tiers, classifier seam, eligibility store) lands in `fragcap-targets`
behind the existing default-off `targets` feature; the two platform adapters
(`SteamSource`, real Win32 `VolumeInventory`) land in the `fragcap` facade, the
only crate that already depends on both `fragcap-steam` and `fragcap-targets`. No
new inter-crate edge is introduced.

## Complexity Tracking

No constitution violations; no entries.
