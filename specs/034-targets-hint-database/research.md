# Phase 0 Research: Targets Hint Database (foundation)

All findings below were verified against the actual toolchain and crate registry
on 2026-08-13, not estimated. Where a number appears (version, package count,
MSRV), it was read from a command's output.

## R1. Embedded database dependency: rusqlite, and the feature set

**Decision**: Add `rusqlite` at the workspace level with
`default-features = false, features = ["bundled"]`. The store crate
(`fragcap-targets`) is the only crate that depends on it.

**Rationale**: The store needs an embedded, transactional, indexed SQL database
with no external service. `rusqlite` is the mature Rust binding; `bundled`
compiles the SQLite amalgamation via `cc` so there is no reliance on a
system-installed `libsqlite3` and the build is deterministic on a bare Windows
runner. The bundled SQLite has the JSON functions (JSON1) compiled in, which a
later seeding or query slice may use; this slice's export is built in Rust with
`serde_json`, so JSON1 is available but not yet exercised.

**`default-features = false` is load-bearing.** rusqlite 0.40's default feature
set enables a WebAssembly FFI backend (`ffi-sqlite-wasm-rs`) that drags a large
graph into `Cargo.lock`. Measured with defaults on, the add pulled
`sqlite-wasm-rs`, `rsqlite-vfs`, `js-sys`, `wasm-bindgen` and its three macro
crates, `bumpalo`, `thiserror`/`thiserror-impl`, `foldhash`, `hashbrown`, and
`hashlink`: roughly fourteen extra packages for machinery this project never
runs. With `default-features = false` and only `bundled`, that graph collapses to
the six packages in R2. This mirrors the project's standing discipline of taking
crates with default features off (regex, windows-sys).

**Alternatives considered**:
- A hand-rolled on-disk format: rejected. It re-implements indexing,
  transactions, and (later) JSON querying; the harder half of the work remains,
  and the corpus is large enough that a flat file does not serve incremental,
  queryable access. Consistent with why the glob matcher was hand-rolled but the
  regex engine was not: here the dependency genuinely supplies the hard part.
- Staying on embedded `include_str!` JSON/INI assets (as `assets/steamdb/` and
  the schema do): rejected. Those are small, read-only, and rebuilt wholesale;
  they cannot express partial, per-tier, resumable seeding of thousands of rows.
- A different SQLite crate (`sqlx` with SQLite, `sqlite`): rejected. `sqlx`
  brings an async runtime and a far larger graph for a synchronous, single-file,
  local store; the `sqlite` crate is a thinner binding without rusqlite's
  ergonomics and bundled story. `rusqlite` is the smallest graph that does the
  job.

## R2. Exact Cargo.lock delta (verified by add + diff)

With `rusqlite { default-features = false, features = ["bundled"] }`, the new
`Cargo.lock` packages are exactly six:

| Package | Kind | License | In-graph reason |
| --- | --- | --- | --- |
| `rusqlite` 0.40.2 | runtime | MIT | the binding |
| `libsqlite3-sys` 0.38.2 | runtime (+ build) | MIT | FFI + bundled SQLite via `cc` |
| `fallible-iterator` 0.3.0 | runtime | MIT/Apache-2.0 | rusqlite row iteration |
| `fallible-streaming-iterator` 0.1.9 | runtime | MIT/Apache-2.0 | rusqlite row iteration |
| `smallvec` 1.15.2 | runtime | MIT OR Apache-2.0 | rusqlite internal buffers |
| `vcpkg` 0.2.15 | build | MIT/Apache-2.0 | libsqlite3-sys build-time lib lookup |

`cc`, `bitflags`, `shlex`, `find-msvc-tools`, and `pkg-config` appear in
rusqlite's tree but are **already** in `Cargo.lock` via `pcap`, so they add no
new package. `hashlink`/`hashbrown`/`foldhash` are NOT added, because they come
only from rusqlite's default `hashlink` feature (the prepared-statement cache),
which `default-features = false` drops. The store does not need the statement
cache for this slice.

**Licensing**: every new package is MIT or Apache-2.0, inside the constitution's
allowed set. The SQLite amalgamation that `libsqlite3-sys` (an MIT crate)
compiles is public-domain C source; `cargo-deny` reads the crate's MIT metadata,
so the license gate passes. Public-domain imposes no attribution obligation; the
fact is recorded in the decisions fragment and the crate NOTICE for honesty.

## R3. MSRV 1.82 verification (built, not assumed)

**Decision**: MSRV stays 1.82; no pin beyond `rusqlite = "0.40"` is required.

**Evidence**: none of the six new crates declares a `rust-version` at all, so the
only real test is compilation. `rustup run 1.82 cargo build` of a crate carrying
`rusqlite` (bundled) compiled every new crate and the cc-built SQLite amalgamation
cleanly under Rust 1.82.0 (2024-10-15), finishing green. The MSRV toolchain is
installed (`1.82-x86_64-pc-windows-msvc`), so `cargo xtask msrv` can run this to
completion at verify time rather than exiting 2.

**Watch item**: because these crates declare no MSRV floor, a future minor bump
could raise it silently. The dependency is therefore taken as `rusqlite = "0.40"`
(compatible-range within the verified minor), and `cargo xtask msrv` is the gate
that would catch a regression on any future lock update. This is the same posture
the project takes generally; only `clap` needed an exact pin, and only because it
actively broke the floor.

## R4. Export contract: how store rows map to the schema

**Decision**: Export a `kind: "export"` envelope; put the per-title hint fields
inside each record, never at the envelope top level.

**Rationale**: the published schema (`docs/schema/target-schema.v1.json`) makes
this exact split binding. Its `allOf` forbids `launch`, `launcher_mediated`, and
`engine` at the top level for `kind in {profile, package, export}` (lines 79-91),
and its `record` `$def` is where those three optional fields live (lines 233-239).
The envelope itself requires `schema` (const 1), `kind`, `fidelity`, and (for
export) `provenance` (lines 62-69). Each record requires only `fidelity` and
`provenance`. See data-model.md for the field-by-field mapping.

**Validation path**: build the document as a `serde_json::Value` and validate it
with `fragcap_profile::jsonschema::validate_value(&value) -> SchemaDiagnostics`,
which is the same structural validator the schema publishes and the profile-load
path uses. The exporter returns an error if the diagnostics are non-empty, so it
can never emit a document the validator rejects (validity by construction,
following the `fragcap-steam` scaffold's D4 precedent). Building a `Value` (not a
`Serialize` derive) matches the project's existing JSON-writing pattern in
`scaffold.rs::render` and keeps escaping correct by construction.

**`game` mapping caution**: `game.id` must match `^[a-z0-9_-]+$`, so records omit
`id` and carry `name`, `platform: "steam"`, and `app_id` (a string). `game` and
its subfields are all optional inside a record, so a Tier-1-only row (appid + name)
is valid.

## R5. Crate placement, feature gating, and the dependency graph

**Decision**: new crate `fragcap-targets`, depending only on `fragcap-profile`
(for `jsonschema::validate_value`) and the workspace `serde_json`, plus `rusqlite`.
Exposed through the `fragcap` facade behind an optional `targets` feature.

**Graph edits** (all mechanically checked by `cargo xtask deps`):
- `xtask/src/deps.rs` `EXPECTED` gains `("fragcap", "fragcap-targets")` and
  `("fragcap-targets", "fragcap-profile")`.
- `xtask/src/deps.rs` `SIBLINGS` gains `"fragcap-targets"` (it is a leaf sibling
  like `fragcap-steam`; it depends on `fragcap-profile`, which is not a sibling,
  and no sibling depends on it).
- `fragcap-core`'s allowlist is untouched; core never sees this crate (P-2).

**Feature gating**: the `fragcap` facade declares `targets = ["dep:fragcap-targets"]`
with `fragcap-targets = { workspace = true, optional = true }`, off by default, so
a default build of the library compiles no SQLite and needs no C toolchain for it
(SC-005). The `fragcap-cli` binary enables `fragcap/targets` so the shipped CLI
carries the `targets` subcommand; the CLI is our own binary and legitimately needs
the capability. This keeps "the library default build skips the engine" and "the
shipped tool has the command" both true.

**Rationale for a new crate over extending `fragcap-steam`**: the corpus is
broader than Steam (Tier 3 is PCGamingWiki engine data), and folding the SQLite C
build into `fragcap-steam` would drag it into every consumer of that crate. A
dedicated, feature-gated crate isolates the build and states the scope. `fragcap-core`
is out on P-2 grounds and is enforced mechanically.

## R6. Import semantics and idempotency

**Decision**: `import` reads a local JSON seed document (the same `export` shape,
or a small superset the loader understands) into the store, transactionally. A
duplicate appid within one document is an error that rolls back the whole import
(no partial store). An appid already present in the store is replaced wholesale
(its `games`/`launch_entries`/`technologies` rows are deleted and reinserted), so
re-importing the same seed is idempotent.

**Rationale**: P-4/P-9 forbid silent partial merges; a half-updated row is a
record that lies about its own contents. A transaction gives all-or-nothing, and
delete-then-insert gives a clean, idempotent replace. A malformed record (bad
engine source, launch entry missing its executable) is rejected before any write,
because the store's write path enforces the same enum/required checks the schema
does, so the store can never hold a row it could not export.

**Seed fixture**: a small hand-authored JSON committed under the crate's test
fixtures, carrying a handful of titles including one launcher-mediated title (The
Elder Scrolls Online) and one with an engine attribution, plus a Tier-1-only title
(appid + name, no launch, no engine) to exercise the omission branches. It is not
a catalog dump.
