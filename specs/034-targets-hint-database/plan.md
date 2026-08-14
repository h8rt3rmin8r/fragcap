# Implementation Plan: Targets Hint Database (foundation)

**Branch**: `034-targets-hint-database` | **Date**: 2026-08-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/034-targets-hint-database/spec.md`

## Summary

Build the foundation of the targets hint database (issue #78): a new
`fragcap-targets` crate holding an embedded SQLite store (rusqlite, bundled), the
three-tier seeding model (which of the public-catalog / launch-metadata /
community-engine tiers owns which columns), and a `kind: "export"` JSON projection
that validates against the S033 hint-record subschema. Demonstrable offline via a
`fragcap targets import`/`export` CLI over a committed seed fixture. No network
fetching, no cascade wiring; those are later slices. The store is engineered so
each tier can be filled independently and resumably.

## Technical Context

**Language/Version**: Rust, edition 2021, workspace MSRV 1.82 (built with 1.96).

**Primary Dependencies**: `rusqlite` 0.40 (`default-features = false`, `["bundled"]`)
for the embedded store; `fragcap-profile` for `jsonschema::validate_value`;
workspace `serde_json` for Value construction and seed parsing; `clap` (via the
existing CLI) for the subcommand. Exact resolution and the six-package Cargo.lock
delta are in research.md R2.

**Storage**: a single embedded SQLite file (schema version via `PRAGMA
user_version`); see data-model.md.

**Testing**: `cargo test` (unit + integration in `fragcap-targets`, a schema
conformance corpus, and a CLI round-trip test using `tempfile`); the gate set is
`cargo xtask ci` plus `cargo xtask msrv`.

**Target Platform**: Windows (x86_64-pc-windows-msvc) primarily; the crate is
portable Rust and the store logic is platform-neutral (no `cfg(windows)` needed).

**Project Type**: Rust library crate plus a CLI subcommand in the existing binary.

**Performance Goals**: not a hot path. The store holds thousands of rows and is
read in bulk for export; ordinary indexed SQLite access is ample. No latency
target.

**Constraints**: offline (no network this slice); MSRV 1.82 held (verified,
research.md R3); default build must not compile the database engine (feature-gated);
`fragcap-core` must not gain the dependency (P-2).

**Scale/Scope**: one new crate (~6 source modules), one CLI command module, one
seed fixture, schema-conformance and round-trip tests, plus workspace/xtask/AGENTS
wiring and two changelog fragments.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

- **P-1 Passive Observation Only**: PASS. The slice adds a data store and a JSON
  exporter. It opens no process handle, touches no packet path, and names no
  denylisted technique. rusqlite cannot transmit or intercept traffic. No capture
  or attribution logic is added.
- **P-2 Core Stays Platform-Neutral**: PASS. `fragcap-targets` depends only on
  `fragcap-profile` and does not touch `fragcap-core`. The `cargo xtask deps`
  allowlist is updated to admit exactly the two new edges and to list the crate as
  a sibling; `fragcap-core`'s `bytes`-only allowlist is unchanged. Dependencies
  flow concrete toward abstract.
- **P-3 Capture And Attribution Stay Separate**: PASS. Not applicable; the slice
  adds neither a `PacketSource` nor a `FlowAttributor`.
- **P-4 No Silent Loss**: PASS. Import is transactional and all-or-nothing; a
  malformed record fails the whole import with a diagnostic rather than being
  dropped. Nothing is discarded uncounted.
- **P-5 Compatibility Outranks Richness**: PASS. The export is the published
  `kind: "export"` shape an unmodified schema validator accepts; no bespoke format.
- **P-6 Glossary First**: ACTION. Any new term introduced in prose or code (for
  example "hint database", "seeding tier", "seed state") gets a glossary entry in
  the same change per section 4.3. Tracked as a task.
- **P-7 Wrappers Stay Thin**: PASS. The capability lives in Rust; the CLI is a
  thin `clap` surface with no parsing of output. No shell wrapper is added.
- **P-8 House Standards Apply**: PASS. UTF-8 no BOM, LF, no em/en dashes, SPDX on
  every source file; enforced by `cargo xtask lint` in the gate. Any Markdown
  authored follows the house standard.
- **P-9 The Instrument Does Not Lie**: PASS, and central. Every exported record is
  fidelity `heuristic-unverified`; engine confidence grades one field and is never
  a fidelity tier. The launch array is persisted and exported whole, never
  flattened. Out-of-set values are refused at write time, not coerced. The store
  cannot hold a row it could not export, and the exporter self-validates.

**Licensing gate**: the six new packages are MIT/Apache-2.0 (research.md R2),
inside the constitution's allowed set; the bundled SQLite amalgamation is
public-domain C compiled by an MIT crate. Recorded as a dated decision. The new
publishable crate carries `LICENSE`, `NOTICE`, `README.md` (byte-checked by
`cargo xtask license`).

**Pinned-artifact note**: `xtask/src/deps.rs` is source, not a pinned process
artifact, but the dependency addition itself is recorded as a dated decision in a
changelog fragment per the workflow rule. No `.github/workflows/**`,
`rust-toolchain.toml`, `release.toml`, or `scripts/**` change is needed.

Result: no violations; no Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/034-targets-hint-database/
├── plan.md              # This file
├── research.md          # Phase 0: rusqlite resolution, MSRV proof, export contract
├── data-model.md        # Phase 1: SQLite schema, Rust types, column-to-JSON mapping
├── quickstart.md        # Phase 1: offline validation guide
├── contracts/
│   └── cli-and-library.md
├── checklists/
│   ├── requirements.md
│   ├── schema-and-honesty.md
│   └── dependency-and-msrv.md
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/fragcap-targets/          # NEW crate
├── Cargo.toml                   # rusqlite (no-default-features + bundled), fragcap-profile, serde_json
├── LICENSE  NOTICE  README.md   # byte-checked by cargo xtask license
├── src/
│   ├── lib.rs                   # crate surface, error type, re-exports
│   ├── model.rs                 # Game, LaunchEntry, Engine, Technology, enums, SeedState
│   ├── schema.rs                # embedded DDL + migration (user_version = 1)
│   ├── store.rs                 # Store: open/upsert/query/seed_state
│   ├── export.rs                # rows -> serde_json::Value -> validate_value -> String
│   └── import.rs                # seed JSON -> transactional load (dup/replace rules)
└── tests/
    ├── round_trip.rs            # insert across tiers, export, assert valid
    ├── conformance.rs           # fixtures: valid passes, malformed rejected with code
    └── fixtures/
        ├── seed.json            # committed hand-authored seed (ESO + engine + Tier-1-only)
        └── ...                  # malformed fixtures

crates/fragcap-cli/src/commands/
└── targets.rs                   # NEW: `targets import|export` clap subcommand (+ registration)

crates/fragcap/Cargo.toml        # EDIT: optional fragcap-targets dep + `targets` feature + re-export
crates/fragcap-cli/Cargo.toml    # EDIT: enable fragcap/targets
Cargo.toml                       # EDIT: [workspace.dependencies] rusqlite + fragcap-targets
xtask/src/deps.rs                # EDIT: EXPECTED += two edges; SIBLINGS += fragcap-targets
AGENTS.md                        # EDIT: dependency inventory row for rusqlite
changelog.d/034-targets-hint-database.added.md      # NEW
changelog.d/034-targets-hint-database.decisions.md  # NEW (dated: dependency + license)
```

**Structure Decision**: a new leaf crate `fragcap-targets` (sibling to
`fragcap-steam`), depending only on `fragcap-profile`, exposed through the facade
behind an optional `targets` feature. This isolates the SQLite C build from every
other consumer and states the store's scope (broader than Steam). The CLI command
lives in `fragcap-cli` (the only crate that legitimately depends on the facade),
matching where `steam` and `run`/`watch`/`tap` commands live.

## Key Design Decisions

1. **rusqlite with `default-features = false, ["bundled"]`.** Defaults drag a
   wasm-bindgen stack (~14 packages); disabling them yields a six-package delta and
   still bundles SQLite (JSON1 included). research.md R1/R2.
2. **MSRV verified by building under 1.82, not assumed.** No new crate declares a
   `rust-version`; the bundled SQLite and all six crates compile green under
   1.82.0. Taken as `rusqlite = "0.40"`; `cargo xtask msrv` is the standing gate.
   research.md R3.
3. **Hint fields live inside records, never on the export envelope.** The schema
   `allOf` forbids `launch`/`launcher_mediated`/`engine` at top level for
   `kind: export`; they belong to each `record`. Envelope carries
   `schema/kind/fidelity/provenance/records`. data-model.md.
4. **Validity by construction via `validate_value`.** The exporter builds a
   `serde_json::Value` and validates it against the embedded schema before
   returning; it cannot emit a rejected document. Same discipline as the Steam
   scaffold (D4).
5. **The store cannot hold an unexportable row.** Enum sets, the engine
   both-or-neither invariant, and the non-empty executable are enforced by SQLite
   CHECKs and the write path, so P-9 holds at the storage layer, not just the
   exporter.
6. **Idempotent import: transaction + wholesale replace.** A duplicate appid in
   one seed is a rolled-back error; an existing appid is replaced via
   delete-then-insert (ON DELETE CASCADE). No partial merge (P-4/P-9). research.md
   R6.
7. **Feature gate placement.** `targets` is off by default at the facade (default
   build skips SQLite, SC-005) and enabled by the CLI binary (the shipped tool
   carries the command). research.md R5.
8. **Technologies are stored but not projected per-record this slice**, because the
   schema `record` `$def` has no `technologies` member. Recorded so the omission is
   not later read as a defect. data-model.md.

## Complexity Tracking

No constitution violations; no entries.
