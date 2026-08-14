The targets hint database (issue #78) gained its foundation: a new
`fragcap-targets` crate holding an embedded SQLite store of known game binaries
and launch patterns, the three-tier seeding model, and a schema-conformant JSON
export (slice S034). The store carries a game's Steam application id, name, and
catalog metrics, its `launcher_mediated` and `token_required` flags, an optional
engine attribution, its launch entries (carried whole, never flattened to a
single process name), and per-title technology findings, plus per-tier seed state
so a later fetch can resume. The three seeding tiers own their columns
independently: the public catalog owns appid and name, the launch metadata owns
the launch array and the launcher flag, and the community engine data owns the
engine attribution. No seeder runs this slice; there is no network fetching, and
the store is populated offline.

The store exports to the `export` variant of the master target schema: a single
envelope of records, one per title, each stamped fidelity `heuristic-unverified`
regardless of engine confidence (P-9), with an unknown engine and an empty launch
array both represented by omission. The exporter validates its own output against
the embedded schema before returning it, so it can never emit a document the
validator rejects. The store cannot hold a row it could not export: SQLite CHECK
constraints and the value types refuse an out-of-set engine source or confidence
and an empty launch executable, and an engine attribution must carry both a source
and a confidence or neither.

A `fragcap targets` command imports a local JSON seed document into a store and
exports a store to schema-conformant JSON, both offline. Import is transactional
and idempotent per application id: a duplicate appid within one seed is rejected
with no partial store, and an appid already present is replaced wholesale rather
than merged into a half-updated row; a malformed seed leaves no store behind. A
committed seed fixture (The Elder Scrolls Online as a launcher-mediated title, a
title carrying an engine, and a catalog-only title) round-trips through the
command to schema-valid JSON.

The crate is exposed through the facade behind an optional `targets` feature, so a
default library build compiles no SQLite engine; the shipped command-line tool
enables it. The one new dependency, `rusqlite` with `default-features = false` and
`bundled`, adds six packages to `Cargo.lock`, is MIT or Apache-2.0 across the
delta (the bundled SQLite amalgamation is public domain), and keeps the minimum
supported toolchain at 1.82, verified by building through it.
