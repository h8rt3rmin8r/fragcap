# Implementation Plan: Tier 1 Catalog Seeder

**Branch**: `035-catalog-seeder` | **Date**: 2026-08-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/035-catalog-seeder/spec.md`

## Summary

Write the first seeder for the targets hint database (#78): Tier 1, the public
catalog. A `CatalogSource` trait yields catalog entries; the seeder applies a
corpus gate (game classification plus a configurable review-count threshold),
merges the admitted titles into the store's Tier 1 columns via a new non-clobbering
`merge_catalog`, records a resume cursor per batch, and returns a summary whose four
counts reconcile to the fetched total (P-4/P-9). Every test drives an offline
`FixtureCatalog`; the real `HttpCatalog` (http_req + native-tls) sits behind an
off-by-default `net` feature, compiled under the all-features gate but run only by
the operator, exactly as live packet capture is.

## Technical Context

**Language/Version**: Rust, edition 2021, workspace MSRV 1.82 (built with 1.96).

**Primary Dependencies**: `http_req` 0.13 (`default-features = false`,
`["native-tls"]`, optional, behind `net`) for the live source; existing
`serde_json`, `rusqlite`, `fragcap-profile`. Exact 18-package Cargo.lock delta,
licenses, and MSRV analysis are in research.md R1.

**Storage**: the existing S034 SQLite store; no schema change, one new merge.

**Testing**: `cargo test` (offline `FixtureCatalog` drives the seeder, the gate, the
merge non-clobber, resumability, and the post-seed export); the gate set is
`cargo xtask ci` + `cargo xtask msrv`.

**Target Platform**: Windows (x86_64-pc-windows-msvc); native-tls resolves to
`schannel`. The seeder logic is platform-neutral.

**Project Type**: Rust library additions plus a CLI subcommand.

**Performance Goals**: not a hot path; a seed run is a maintainer operation over a
large but finite catalog, paced by the source. No latency target.

**Constraints**: offline-testable (no network in CI); MSRV 1.82 held (the `net`
graph is outside the default/msrv build); the HTTP+TLS graph stays inside the
allowed license set; `fragcap-core` untouched.

**Scale/Scope**: new source/seeder/gate/summary types and one store merge in
`fragcap-targets`, a CLI `seed` subcommand, catalog fixtures, tests, and the
dependency + glossary + changelog paperwork.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

- **P-1 Passive Observation Only**: PASS. The seeder performs read-only HTTPS GETs
  against a public Web API. It opens no process handle, touches no packet path, and
  names no denylisted technique. `http_req` cannot capture or intercept traffic.
- **P-2 Core Stays Platform-Neutral**: PASS. `http_req` lands only in
  `fragcap-targets`, behind `net`. `fragcap-core` keeps its `bytes`-only allowlist;
  no new internal edge is added (`cargo xtask deps` unchanged).
- **P-3 Capture And Attribution Stay Separate**: PASS. Not applicable; no
  `PacketSource` or `FlowAttributor` is involved.
- **P-4 No Silent Loss**: PASS, and central. Every fetched entry is counted as
  written, excluded, or failed; the conservation identity is asserted in tests. A
  truncated corpus cannot read as complete.
- **P-5 Compatibility Outranks Richness**: PASS. The store still exports the
  published schema after seeding; nothing about the output format changes.
- **P-6 Glossary First**: ACTION. New terms (catalog seeder, corpus gate, catalog
  source, seed summary) get glossary entries in this change. Tracked as a task.
- **P-7 Wrappers Stay Thin**: PASS. The live fetch is Rust in the library, not a
  shell wrapper parsing output; the CLI is a thin clap surface.
- **P-8 House Standards Apply**: PASS. UTF-8 no BOM, LF, no em/en dashes, SPDX per
  file; enforced by `cargo xtask lint`.
- **P-9 The Instrument Does Not Lie**: PASS. Every seeded row stays
  `heuristic-unverified`; the gate's exclusions are reported, not hidden; the merge
  never fabricates or overwrites another tier's observation; a missing popularity
  signal excludes (honest) rather than being guessed.

**Licensing gate**: the 18 `net` packages are all MIT or Apache-2.0 (research.md
R1); `webpki-roots`' CDLA-Permissive-2.0 is the reason the rustls path was rejected
in favor of native-tls, which pulls no bundled root set. Recorded as a dated
decision. No new publishable crate (the dependency lands in the existing
`fragcap-targets`).

**MSRV note**: the `net` feature is off by default, so `cargo xtask msrv`
(default-feature workspace build) does not compile `http_req`; the floor is
unaffected, consistent with `pcap` behind `live`. Verified by building both net-off
under 1.82 and net-on under 1.96 (research.md R1).

Result: no violations; no Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/035-catalog-seeder/
├── plan.md  research.md  data-model.md  quickstart.md
├── contracts/seeder.md
├── checklists/{requirements,seeding-honesty,network-dependency}.md
└── tasks.md   # /speckit-tasks
```

### Source Code (repository root)

```text
crates/fragcap-targets/
├── Cargo.toml                   # EDIT: `net` feature + optional http_req
├── src/
│   ├── lib.rs                   # EDIT: re-exports; net-gated HttpCatalog
│   ├── catalog.rs               # NEW: CatalogSource trait, CatalogEntry, CatalogBatch,
│   │                            #      Classification, FixtureCatalog
│   ├── gate.rs                  # NEW: CorpusGate
│   ├── seed.rs                  # NEW: seed_catalog + SeedSummary
│   ├── http_catalog.rs          # NEW (cfg(feature="net")): HttpCatalog over http_req
│   └── store.rs                 # EDIT: add merge_catalog
└── tests/
    ├── catalog_seed.rs          # NEW: US1 - seed, conservation, export valid
    ├── catalog_resume.rs        # NEW: US2 - resumability
    ├── catalog_tiers.rs         # NEW: US3 - merge preserves other tiers
    └── fixtures/catalog.json    # NEW: committed catalog fixture

crates/fragcap/Cargo.toml        # EDIT: `net = ["fragcap-targets/net"]` passthrough
crates/fragcap/src/lib.rs        # EDIT: re-export CatalogSource/seed types; net-gated HttpCatalog
crates/fragcap-cli/Cargo.toml    # EDIT: `net` passthrough feature (off by default)
crates/fragcap-cli/src/cli.rs    # EDIT: `targets seed` subcommand
crates/fragcap-cli/src/commands/targets.rs   # EDIT: seed handler (offline; net-gated live)
Cargo.toml                       # EDIT: [workspace.dependencies] http_req (justified)
AGENTS.md                        # EDIT: dependency inventory row
docs/glossary/process-and-attribution.md     # EDIT: new terms (+ regenerate index)
changelog.d/035-catalog-seeder.{added,decisions}.md   # NEW
```

**Structure Decision**: the seeder, source trait, gate, summary, and merge live in
`fragcap-targets` (the crate that owns the store). The live `HttpCatalog` is a
single `cfg(feature = "net")` module so the client is isolated behind one seam; the
trait means replacing the client later is a one-module change. The CLI command lives
in `fragcap-cli` alongside the existing `targets` command.

## Key Design Decisions

1. **http_req + native-tls behind `net`, off by default.** The only 2025-era HTTPS
   client clearing all three constraints (allowed licenses, no ICU4X, small graph);
   MSRV is a non-issue because `net` is outside the default/msrv build. research.md
   R1.
2. **CatalogSource is a trait; the offline fixture drives every test.** The
   `live`-capture pattern: the pipeline is fully tested offline, the wire adapter is
   compiled-not-run. research.md R2.
3. **A new `merge_catalog`, not the foundation's whole-game replace.** Tier 1 must
   update only its columns; the replace path would clobber Tiers 2/3. research.md R3.
4. **Truthful summary with a conservation invariant.** `fetched == written +
   excluded + failed`, asserted; a bad entry is counted, not fatal; the seeder never
   prunes. research.md R4.
5. **Caller supplies the run timestamp.** `seed_catalog` takes `now` rather than
   reading a clock, keeping the store and seeder free of ambient time and fully
   deterministic in tests.
6. **The gate excludes on a missing signal.** An entry whose popularity is unknown
   is excluded (honest) rather than admitted on a guess (P-9).
7. **Default review threshold** set to a documented few-hundred figure, tunable via
   `--min-reviews`; not a load-bearing correctness value.

## Complexity Tracking

No constitution violations; no entries.
