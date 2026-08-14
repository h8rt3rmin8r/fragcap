# Quickstart / Validation: Tier 3 Engine Seeder (PCGamingWiki)

All steps are offline except the explicitly operator-only live seed. Run from the
repository root on Windows.

## 1. Library seed pipeline (the MVP, offline)

Run the crate's tests, which drive an offline `FixtureEngineFeed` through
`seed_engine` into an in-memory store: a mix of titles with clear engines, a title
with no engine, a title with an ambiguous engine, and a malformed entry; assert the
store holds an engine only for the clear titles, the summary's five counts
reconcile to the fetched total, and the store exports schema-valid:

```bash
cargo test -p fragcap-targets
```

Expected: green, including the conservation assertion
`fetched == written + excluded + duplicates + failed` and a schema-valid export
after seeding (including an engine-only row for an application id not previously in
the store).

## 2. Per-tier merge preserves other tiers

Covered by a test that gives a title a catalog name (Tier 1) and launch entries
(Tier 2), runs the engine seeder over a source that names that appid's engine, and
asserts the engine columns filled while the name and launch entries are unchanged:

```bash
cargo test -p fragcap-targets engine_tiers
```

## 3. Resumability

A test seeds part of an engine universe (recording a cursor under the engine tier),
then resumes; asserts the final result equals a single uninterrupted seed with no
duplicate rows:

```bash
cargo test -p fragcap-targets engine_resume
```

## 4. CLI offline seed

Seed a store's engine columns from a committed engine fixture, no network, then
export:

```bash
cargo run -p fragcap-cli -- targets seed-engine --from crates/fragcap-targets/tests/fixtures/engine.json --db %TEMP%\hint.db
cargo run -p fragcap-cli -- targets export --db %TEMP%\hint.db
```

Expected: the seed prints a summary (fetched / written / excluded / duplicates /
failed); the export prints schema-valid JSON in which the engine-bearing titles
carry `engine.source: pcgamingwiki` and a confidence, and every record's fidelity
is heuristic-unverified.

## 5. Live seed (operator only, behind `net`)

Not run in CI. A maintainer builds with the `net` feature and seeds engines from
PCGamingWiki's query API:

```bash
cargo run -p fragcap-cli --features net -- targets seed-engine --pcgamingwiki --db %TEMP%\hint.db
```

Expected: the same command shape as the offline seed, against the wire.

## 6. Full gate set (the slice's Done gate)

```bash
cargo xtask ci
cargo xtask msrv
```

Expected: `cargo xtask ci` passes; its `--all-features` clippy step compiles the
`net` feature (`http_req` + the `HttpEngineFeed` module) on the pinned toolchain,
and its tests run entirely offline. `cargo xtask msrv` builds the default-feature
workspace (no `net`, so no `http_req`) through `rustup run 1.82` and exits 0. A
default `cargo build -p fragcap` compiles no HTTP client and no new dependency.
