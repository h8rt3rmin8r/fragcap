# Phase 0 Research: Tier 1 Catalog Seeder

All dependency numbers below were verified against the real toolchain and crate
registry on 2026-08-13 (add + `cargo tree` + `diff Cargo.lock` + build under
`rustup run 1.82`), not estimated.

## R1. The HTTP client: http_req + native-tls, behind an off-by-default `net` feature

**Decision**: Add `http_req` (`default-features = false, features = ["native-tls"]`)
as an optional dependency of `fragcap-targets`, gated by a new `net` feature. The
real network catalog source is the only code behind it; every test uses an offline
source.

**The constraint that decided it.** A 2025-era HTTPS client has to satisfy three
project constraints at once, and most candidates fail one:

- **License** (constitution: MIT/Apache-2.0/BSD/ISC/Unicode-DFS/Zlib only). This
  eliminated the rustls path: `minreq` with its `https` feature is otherwise ideal
  (20 packages, no `url`/ICU4X, well maintained), but it forces `webpki-roots`,
  which is **CDLA-Permissive-2.0**, outside the allowed set. rustls itself is fine;
  its bundled Mozilla root store is not.
- **Graph size / no gratuitous ICU4X.** This eliminated every client built on the
  `url` crate: `ureq` and `attohttpc` pull `url` -> `idna` 1.x -> the full ICU4X
  stack, measured at **42 new packages** for `ureq` + native-tls. That is the same
  ~40-crate ICU4X graph S025 rejected `boon` for; taking it here would be
  inconsistent.
- **MSRV 1.82.** This turned out **not** to bind the client, because the `net`
  feature is off by default and `cargo xtask msrv` builds default features only
  (`cargo build --workspace --locked`, no `--all-features`). Optional-feature deps
  are outside the MSRV floor, exactly as `pcap` (behind `live`) and `windows-sys`
  (behind `socket-table`/`etw`) already are. Verified: with the `net` deps in the
  lock (including `zeroize` 1.9, which declares edition 2024 / rust-version 1.85),
  `rustup run 1.82 cargo build -p fragcap-targets` (net off) builds green, and
  `cargo build -p fragcap-targets --features net` on the 1.96 toolchain builds
  green. The net graph must compile under the `--all-features` clippy gate on the
  pinned toolchain, not under 1.82.

`http_req` + native-tls is the candidate that clears all three: **18 new
packages**, no `url`/ICU4X, no `ring` (native-tls uses the operating system's TLS
and trust store, which is `schannel` on Windows), and every crate in the delta is
MIT or Apache-2.0. Using the OS trust store is also more correct than a bundled
root set that ages out.

**New Cargo.lock packages (18)**: `http_req`, `native-tls`, `schannel`,
`security-framework`, `security-framework-sys`, `core-foundation`,
`core-foundation-sys`, `foreign-types`, `foreign-types-shared`, `openssl`,
`openssl-sys`, `openssl-macros`, `openssl-probe`, `base64`, `unicase`, `log`,
`zeroize`, `zeroize_derive`. On `x86_64-pc-windows-msvc` only the `schannel` path
compiles; the `openssl-*` and `security-framework*` crates are other-platform
targets present in the lock but never built here. Licenses: all MIT or Apache-2.0.

**Maintenance risk and its mitigation.** `http_req` is smaller and less widely used
than `ureq`/`reqwest`. That risk is bounded by two facts: the client sits behind
the `CatalogSource` trait, so replacing it later touches exactly one implementation
module and no seeder logic; and it is a feature-gated, operator-run path that CI
compiles but never executes, so a client regression cannot break the shipped
default build or the tested pipeline. Recorded as an accepted, mitigated risk.

**Rejected alternatives**: `ureq`/`attohttpc` (ICU4X, 42 packages); `minreq`+rustls
(`webpki-roots` CDLA-Permissive-2.0 license); `reqwest` (async runtime + `url` +
ICU4X, far larger); a hand-rolled TLS client (crypto, out of the question). A pin
of `zeroize` to a pre-edition-2024 line was considered and found unnecessary once
the MSRV analysis showed the net graph is not in the 1.82 build.

## R2. CatalogSource is a trait; the offline source drives every test

**Decision**: `trait CatalogSource` yields catalog entries in cursor-paged
batches. Two implementations: `FixtureCatalog` (reads a committed JSON document,
used by every test and by the offline CLI path) and `HttpCatalog` (behind `net`,
performs read-only GETs, run by the operator).

**Rationale**: this is the `live`-capture pattern (specification section 25.1's
"the whole pipeline runs with no capture driver"): the seeder's fetch-parse-map-gate
logic is exercised entirely offline against `FixtureCatalog`, and `HttpCatalog` is
a thin adapter compiled under `net` but never run in CI. The trait fixes the shape
the seeder consumes (an entry carries an application id, an optional name, a
game/other classification, and a popularity count), so the seeder cannot come to
depend on a live-only detail a fixture cannot express.

**Batching and the cursor.** `fetch_batch(cursor: Option<&str>) -> CatalogBatch`,
where `CatalogBatch { entries, next_cursor }`. The cursor is an opaque string the
source defines; for the real Steam app-list it is the last application id seen
(the Web API's `last_appid` pagination). The seeder writes `next_cursor` into the
catalog tier's `seed_state.resume_cursor` after each batch, so a resumed run passes
the stored cursor back to `fetch_batch` and continues. `FixtureCatalog` implements
the same cursor contract over its in-memory list, so resumability is tested offline.

**The real source's endpoint** is an operator-facing detail, not a CI-tested one.
`HttpCatalog` fetches the Steam Web API app list (`ISteamApps/GetAppList`, which
paginates by `last_appid` and yields application id + name) for the universe; the
classification and popularity signal the gate needs come from a companion lookup
the operator configures. The precise companion endpoint is left to the operator
and documented, because it does not affect the tested seeder contract: `HttpCatalog`
just has to produce `CatalogEntry` values, and the gate and merge are identical
whether the entry came from a fixture or the wire.

## R3. The per-tier merge, and why the foundation's write path is wrong here

**Decision**: add `Store::merge_catalog(appid, name, metrics)` performing
`INSERT INTO games (...) VALUES (...) ON CONFLICT(appid) DO UPDATE SET` of **only**
the Tier 1 columns (`name`, `review_count`, `owners`, `peak_ccu`).

**Rationale**: the foundation's `upsert_game`/`write_game` delete the game row (and,
by cascade, its launch entries and technologies) and reinsert. That is correct for
importing a whole hand-authored game, but a Tier 1 seeder that only knows appid +
name would erase any Tier 2 launch data and Tier 3 engine data a prior seeder wrote.
`merge_catalog` updates its own columns and leaves `launcher_mediated`,
`token_required`, `engine_*`, `launch_entries`, and `technologies` untouched. This
realizes FR-007 and is proven by seeding Tier 1 over a game that already carries an
engine and asserting the engine survives. The empty-name guard from S034 applies:
`merge_catalog` stores `NULL` for an empty or absent name, never `""`.

## R4. The corpus gate and the seed summary (P-4/P-9)

**Decision**: `CorpusGate { min_reviews: u64 }` (default recorded in the plan, a
few hundred). An entry is in corpus iff it is classified a game and its review
count is present and `>= min_reviews`. Everything else is excluded.

The seeder returns `SeedSummary { fetched, written, excluded, failed }`, and the
conservation identity `fetched == written + excluded + failed` is asserted in
tests. A malformed or unfetchable single entry increments `failed` and the run
continues (FR-006); a below-threshold or non-game entry increments `excluded`; a
kept entry increments `written`. No entry leaves the run uncounted, so a truncated
corpus can never read as complete (P-4/P-9). The seeder never prunes: an appid in
the store but absent from a run is left as it is.

## R5. Feature wiring across the crates

- `fragcap-targets`: new `net` feature => `dep:http_req` + the `HttpCatalog` impl.
  Off by default. The trait, `FixtureCatalog`, the seeder, the gate, the summary,
  and `merge_catalog` are all in the default build and fully tested offline.
- `fragcap` facade: `net = ["fragcap-targets/net"]` passthrough, off by default,
  re-exporting `HttpCatalog` under `#[cfg(feature = "net")]`.
- `fragcap-cli`: the `targets seed` command drives `FixtureCatalog` from a local
  file with no feature; the real `HttpCatalog` path is behind the CLI's own `net`
  passthrough, so a default CLI build seeds offline and a maintainer builds with
  `--features net` to seed from the wire. The hint database is maintainer-seeded and
  shipped as data (#58), so end users never seed and never pay for the client.
- `xtask/src/deps.rs`: no new internal edge (http_req is external); `fragcap-core`
  stays `bytes`-only. The existing `fragcap-targets -> fragcap-profile` and
  `fragcap -> fragcap-targets` edges are unchanged.
