# Implementation Plan: Steam list identity and JSON output

**Branch**: `067-steam-list-identity-json` | **Date**: 2026-08-22 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/067-steam-list-identity-json/spec.md`

## Summary

`fragcap steam list` (`crates/fragcap-cli/src/commands/steam.rs`) prints two
unlabeled tab-separated columns and ignores the global `--json` flag. This
slice joins each installed title against the local store by its exact Steam
anchor (`steam:<app_id>`), adds a header and a three-state identity column
(registered+positioned / registered-only / unregistered) to the human table,
defines the row order as by-name (case-insensitive) then by-app-id, and adds a
`--json` mode that writes one newline-delimited record per title (matching the
`doctor --json` precedent) carrying the same identity fields plus the install
directory the human table has never shown. The listing snapshot table is read
only, never written, by this command.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (pinned toolchain per
`rust-toolchain.toml`)

**Primary Dependencies**: `fragcap-steam` (installation discovery),
`fragcap-targets` (`Store`, `TargetEntry`, `identifier::steam_anchor`),
`fragcap` facade `write_json_string` (already used by `doctor` and the
emitter for hand-rolled JSON)

**Storage**: The existing SQLite local store (`rusqlite`, feature `targets`),
opened read-only in effect by this command (no write call is made against it)

**Testing**: `cargo test --workspace --locked`; new unit/integration coverage
lives in `crates/fragcap-cli/tests/` alongside the existing `steam` and
`targets` CLI tests

**Target Platform**: Windows (workspace-wide; this slice touches no
platform-specific code beyond what `fragcap-steam` and `fragcap-targets`
already carry)

**Project Type**: CLI (single Cargo workspace, `fragcap-cli` binary crate)

**Performance Goals**: N/A, bounded by one store open plus O(installed
titles) indexed lookups, the same order of magnitude as the existing hero
listing join

**Constraints**: Must not write `listing_snapshot` (FR-006); must not change
the "no Steam installation" exit-2 contract (FR-014); enumeration warnings
must stay on stderr via the emitter in both modes (FR-013, already true
today per the emitter's existing `Format` handling)

**Scale/Scope**: One CLI command (`steam list`), one new store read method,
one new render path per output mode; no schema change (schema version stays
at its current value since no table changes shape, only a new query against
existing tables)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Principles checked against `.specify/memory/constitution.md`:

- **P-1 (passive observation, denylisted techniques)**: N/A. This slice reads
  Steam manifests and the local SQLite store; it opens no process handle and
  captures no traffic.
- **P-4 (every discard path is counted)**: The three-state identity model
  (FR-003 through FR-005) is exactly this principle applied to inspection
  output: an unregistered title must never render identically to a registered
  one whose position lookup failed. Held by construction, see the Key
  Entities section of the spec.
- **P-9 (the instrument does not lie)**: The store-unopenable fallback
  (FR-008) must warn rather than silently render every row as unregistered;
  this is the fabricated-certainty failure P-9 forbids applied to a listing
  command. Held: the plan below emits a warning through the emitter and never
  reports "unregistered" as a fact when it is actually "unknown."
- **P-10 (one path to a target)**: N/A, no new target-creation path is
  introduced. This command still only reads; registration stays
  `targets add --steam <app_id>`.
- **fragcap-core takes no platform-specific dependency**: N/A, this slice
  touches only `fragcap-cli`, `fragcap-targets` (a store read addition), and
  reads (never writes) `fragcap-steam` output. No `fragcap-core` change.
- **A new term gets a glossary entry**: No new term is introduced; "handle"
  is reused per the issue's own instruction not to introduce "slug" as a
  second word for the same concept.
- **Wrappers stay thin**: N/A, no wrapper script involved.
- **UTF-8 no BOM, LF, no em/en dashes**: Applies to all new source and docs;
  enforced by `cargo xtask lint` and this plan's own authoring.

No violations requiring the Complexity Tracking table.

## Project Structure

### Documentation (this feature)

```text
specs/067-steam-list-identity-json/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/            # Phase 1 output (CLI output contract)
└── tasks.md              # Phase 2 output (/speckit-tasks, not this command)
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── commands/
│   ├── steam.rs          # list() rewritten: join, sort, header, --json branch
│   └── targets.rs        # default_local_store() reused (already pub(crate))
├── lib.rs                # dispatch() passes `json` into commands::steam::run
├── cli.rs                # SteamArgs already carries no extra flags needed;
│                          # --json is the existing global flag
└── emit.rs                # unchanged; Format/warn already honor --json

crates/fragcap-targets/src/
└── store.rs               # new read method: listing_snapshot_position(stable_id)

crates/fragcap-cli/tests/
└── cli_steam.rs         # existing file, extended with machine-state-agnostic
                          # wiring/exit-code smoke tests only (see the
                          # /speckit-analyze note below)
```

**Analyze-phase correction**: `fragcap::steam::discover()` has no root
override reachable from `steam list` (unlike `targets discover
--steam-root`), so `crates/fragcap-cli/tests/cli_steam.rs`-level tests
cannot inject synthetic installed titles and cannot exercise the three-state
identity join, sort order, or JSON field shape. Those are tested as
in-module `#[cfg(test)]` unit tests inside
`crates/fragcap-cli/src/commands/steam.rs` instead, against
`Store::open_in_memory()` seeded via `insert_target` /
`write_listing_snapshot` and hand-built `InstalledTitle` values (all its
fields are `pub`). See `tasks.md` Phase 3/4 testing notes.

**Structure Decision**: Single-crate-workspace CLI change. No new crate, no
schema migration. The join logic lives in `fragcap-cli::commands::steam`
(consistent with how `targets.rs`'s hero listing already composes discovery
and store reads at the CLI layer rather than pushing presentation logic into
`fragcap-targets` or `fragcap-steam`); the one new capability added below the
CLI layer is a single new `Store` read method for the reverse snapshot lookup,
since `fragcap-targets` already owns the schema and its read/write surface.

## Complexity Tracking

Not applicable, no constitution violation.
