# Implementation Plan: Tier 3 Engine Seeder (PCGamingWiki)

**Branch**: `036-engine-seeder` | **Date**: 2026-08-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/036-engine-seeder/spec.md`

## Summary

Write the Tier 3 seeder for the targets hint database (#78): engine attribution
from PCGamingWiki. An `EngineFeed` trait yields engine entries keyed by Steam
application id; the seeder writes an engine (name, `source = "pcgamingwiki"`,
confidence) for each title the source resolves a single unambiguous engine name
for, via a new non-clobbering `Store::merge_engine`, records a resume cursor per
batch under the engine tier, and returns the same conservation-checked
`SeedSummary` S035 defined (P-4/P-9). A title with no engine or an ambiguous
engine is left absent (not guessed) and counted excluded. Every test drives an
offline `FixtureEngineFeed`; the real `HttpEngineFeed` (the existing
`http_req` client behind the existing off-by-default `net` feature) is compiled
under the all-features gate but run only by the operator, exactly as the S035
`HttpCatalog` and live packet capture are. No new dependency is taken.

## Technical Context

**Language/Version**: Rust, edition 2021, workspace MSRV 1.82 (built with 1.96).

**Primary Dependencies**: none new. Reuses `http_req` 0.13 (optional, behind the
existing `net` feature) for the live source, plus existing `serde_json`,
`rusqlite`, `fragcap-profile`. The dependency was chosen and justified once in
S035 (research.md R1 there) for the whole seeder arc; this slice consumes it.

**Storage**: the existing S034 SQLite store; no schema change (the engine columns,
the both-or-neither CHECK, and `SeedTier::Engine` all exist from S034/S033), one
new merge.

**Testing**: `cargo test` (offline `FixtureEngineFeed` drives the seeder, the
merge non-clobber, resumability, and the post-seed export); the gate set is
`cargo xtask ci` + `cargo xtask msrv`.

**Target Platform**: Windows (x86_64-pc-windows-msvc). The seeder logic is
platform-neutral.

**Project Type**: Rust library additions plus a CLI subcommand.

**Performance Goals**: not a hot path; a seed run is a maintainer operation paced
by the source. No latency target.

**Constraints**: offline-testable (no network in CI); MSRV 1.82 held (the `net`
graph is outside the default/msrv build, unchanged from S035); no new dependency;
`fragcap-core` untouched (P-2).

**Scale/Scope**: new source trait, offline fixture source, seeder function, and one
store merge in `fragcap-targets`; a live source module behind `net`; a CLI
`seed-engine` subcommand; an engine fixture; tests; and the glossary + changelog
paperwork. No new dependency row.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

- **P-1 Passive Observation Only**: PASS. The seeder performs read-only HTTPS GETs
  against PCGamingWiki's public query API. It opens no process handle, touches no
  packet path, and names no denylisted technique. `http_req` cannot capture or
  intercept traffic (asserted by `cargo xtask lint`, unchanged from S035).
- **P-2 Core Stays Platform-Neutral**: PASS. All new code lands in
  `fragcap-targets`; the live source is behind `net`. `fragcap-core` keeps its
  `bytes`-only allowlist; no new internal edge (`cargo xtask deps` unchanged). No
  new external dependency at all.
- **P-3 Capture And Attribution Stay Separate**: PASS. Not applicable; no
  `PacketSource` or `FlowAttributor` is involved.
- **P-4 No Silent Loss**: PASS, and central. Every fetched entry is counted as
  written, excluded, a within-run duplicate, or failed; the conservation identity
  is asserted in tests. A truncated engine seed cannot read as complete.
- **P-5 Compatibility Outranks Richness**: PASS. The store still exports the
  published schema after seeding; nothing about the output format changes. An
  engine-only row (appid + engine, no name) is a valid export record.
- **P-6 Glossary First**: ACTION. New terms (engine seeder, engine source, engine
  attribution as a Tier 3 write, engine confidence as a within-field grade) get
  glossary entries in this change. Tracked as a task. (Some of these terms were
  introduced by S033's schema revision; this slice confirms/extends rather than
  duplicates.)
- **P-7 Wrappers Stay Thin**: PASS. The live fetch is Rust in the library, not a
  shell wrapper parsing output; the CLI is a thin clap surface.
- **P-8 House Standards Apply**: PASS. UTF-8 no BOM, LF, no em/en dashes, SPDX per
  file; enforced by `cargo xtask lint`.
- **P-9 The Instrument Does Not Lie**: PASS, and the slice's sharpest edge. Every
  seeded row stays `heuristic-unverified`; the engine confidence is a within-field
  grade of one heuristic field, never a fifth fidelity tier and never the record's
  overall trust. A missing or ambiguous engine is left absent (honest) rather than
  guessed. A wrong-typed or out-of-set field is counted failed, never coerced to a
  default and then reported as excluded, so the summary never misattributes why an
  engine is or is not present.

**Licensing gate**: no new package; the license set is unchanged from S035. No new
publishable crate (code lands in the existing `fragcap-targets`).

**MSRV note**: unchanged from S035. The `net` feature is off by default, so
`cargo xtask msrv` (default-feature workspace build) does not compile `http_req`;
the floor is unaffected. Verified by building net-off under 1.82 and net-on under
the pinned toolchain in this slice.

Result: no violations; no Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/036-engine-seeder/
├── plan.md  research.md  data-model.md  quickstart.md
├── contracts/seeder.md
├── checklists/{requirements,seeding-honesty,engine-attribution}.md
└── tasks.md   # /speckit-tasks
```

### Source Code (repository root)

```text
crates/fragcap-targets/
├── src/
│   ├── lib.rs                   # EDIT: re-exports; net-gated HttpEngineFeed
│   ├── engine_feed.rs           # NEW: EngineFeed trait, EngineEntry, EngineBatch,
│   │                            #      ResolvedEngine, FixtureEngineFeed
│   ├── seed.rs                  # EDIT: add seed_engine (SeedSummary reused)
│   ├── http_engine.rs           # NEW (cfg(feature="net")): HttpEngineFeed over http_req
│   └── store.rs                 # EDIT: add merge_engine
└── tests/
    ├── engine_seed.rs           # NEW: US1 - seed, conservation, export valid, engine-only row
    ├── engine_tiers.rs          # NEW: US2 - merge preserves catalog + launch; never prune
    ├── engine_resume.rs         # NEW: US3 - resumability
    └── fixtures/engine.json      # NEW: committed engine fixture

crates/fragcap/src/lib.rs        # EDIT: re-export EngineFeed/seed_engine types; net-gated HttpEngineFeed
crates/fragcap-cli/src/cli.rs    # EDIT: `targets seed-engine` subcommand
crates/fragcap-cli/src/commands/targets.rs   # EDIT: seed-engine handler (offline; net-gated live)
docs/glossary/process-and-attribution.md     # EDIT: new/confirmed terms (+ regenerate index)
changelog.d/036-engine-seeder.{added,decisions}.md   # NEW
```

**Structure Decision**: the source trait, offline fixture source, seeder function,
and merge live in `fragcap-targets` (the crate that owns the store), mirroring
S035's placement one-for-one. The live `HttpEngineFeed` is a single
`cfg(feature = "net")` module so the client stays isolated behind one seam; the
trait means the live source is a one-module change. The CLI command lives in
`fragcap-cli` alongside the existing `targets` command. No `Cargo.toml` edits are
needed for dependencies (the `net` feature and `http_req` already exist across the
three crates from S035); the only manifest touch, if any, is none.

## Key Design Decisions

1. **No new dependency; reuse S035's `http_req` + `net`.** The HTTP client was
   chosen and justified once in S035 for the whole seeder arc. This slice's live
   source is another `CatalogSource`-shaped adapter behind the same feature, so the
   dependency graph, licenses, and MSRV posture are unchanged. research.md R1.
2. **`EngineFeed` is the fetch trait; the offline fixture drives every test.** The
   exact analogue of S035's `CatalogSource`, named `EngineFeed` (not `EngineSource`)
   to avoid clashing with the existing `EngineSource` schema-token enum: the pipeline
   is fully tested offline against `FixtureEngineFeed`, and `HttpEngineFeed` is
   compiled-not-run. research.md R2.
3. **A new `merge_engine`, mirroring `merge_catalog`.** Tier 3 must update only the
   engine columns; the foundation's whole-game replace would clobber Tiers 1/2, and
   even `merge_catalog` would not write engine. `merge_engine` upserts
   `(engine_name, engine_source, engine_confidence)` and inserts an engine-only row
   for an application id the store has not seen. Source and confidence are always
   written together (the store's both-or-neither invariant, satisfied by taking a
   whole `Engine` value whose source and confidence are non-optional). research.md R3.
4. **Written iff a single unambiguous engine name resolves; else excluded.** The
   source yields, per title, an application id and an optional `ResolvedEngine`
   (name + confidence). `Some` -> merge + written (idempotent per appid; a repeat is
   a within-run duplicate). `None` (no engine, or an ambiguous multi-engine field)
   -> excluded, engine columns left unset. Never a guessed engine (P-9). research.md R4.
5. **Confidence is a within-field grade, defaulted and documented, never a fidelity
   tier.** The offline fixture supplies a confidence token per entry so the store
   path is exercised across all five values; the live source maps a cleanly resolved
   single engine to a fixed token (`high`, documented, tunable, not load-bearing). A
   wrong-typed or out-of-set confidence is a parse failure counted as failed, never
   coerced to the default. research.md R4/R5.
6. **Caller supplies the run timestamp.** `seed_engine` takes `now` rather than
   reading a clock, keeping the seeder and store free of ambient time and
   deterministic in tests, exactly as `seed_catalog` does.
7. **`SeedSummary` is reused, not re-defined.** The engine seeder returns the same
   `{fetched, written, excluded, duplicates, failed}` with the same conservation
   invariant; the honesty property is identical, so the type is shared. research.md R4.
8. **CLI: a new `seed-engine` subcommand, and the live flag is `--pcgamingwiki`.**
   A separate subcommand (not an extension of `seed`) because the source shape and
   arguments differ: no `--min-reviews` corpus gate applies to engine enrichment.
   The live-source flag names its actual source, `--pcgamingwiki`, rather than
   S035's `--steam`: the engine tier is keyed by Steam application id but the data
   is PCGamingWiki's, so `--steam` would misattribute the source (a P-9 naming
   concern). The offline `--from <file>` path is always available; `--pcgamingwiki`
   is present only under `net`. Recorded and surfaced at the pre-push halt for
   operator veto, since the kickoff sketched the flag as `--steam`. research.md R5.

## Complexity Tracking

No constitution violations; no entries.
