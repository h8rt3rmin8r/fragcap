# Quickstart / Validation: Tier 1 Catalog Seeder

All steps are offline except the explicitly operator-only live seed. Run from the
repository root on Windows.

## 1. Library seed pipeline (the MVP, offline)

Run the crate's tests, which drive an offline `FixtureCatalog` through
`seed_catalog` into an in-memory store: a mix of in-corpus games, out-of-corpus
entries, and a failing entry; assert the store holds exactly the in-corpus titles,
the summary's four counts reconcile to the fetched total, and the store exports
schema-valid:

```bash
cargo test -p fragcap-targets
```

Expected: green, including the conservation assertion
`fetched == written + excluded + failed` and a schema-valid export after seeding.

## 2. Per-tier merge preserves other tiers

Covered by a test that gives a title an engine (Tier 3) and launch entries (Tier
2), runs the Tier 1 seeder over a catalog carrying that appid with a new name, and
asserts the name updated while the engine and launch entries are unchanged:

```bash
cargo test -p fragcap-targets tiers
```

## 3. Resumability

A test seeds part of a catalog (recording a cursor), then resumes; asserts the
final corpus equals a single uninterrupted seed with no duplicate rows:

```bash
cargo test -p fragcap-targets resume
```

## 4. CLI offline seed

Seed a store from a committed catalog fixture, no network, then export:

```bash
cargo run -p fragcap-cli -- targets seed --from crates/fragcap-targets/tests/fixtures/catalog.json --db %TEMP%\hint.db --min-reviews 100
cargo run -p fragcap-cli -- targets export --db %TEMP%\hint.db
```

Expected: the seed prints a summary (fetched / written / excluded / failed); the
export prints schema-valid JSON containing the in-corpus titles.

## 5. Live seed (operator only, behind `net`)

Not run in CI. A maintainer builds with the `net` feature and seeds from the real
Steam Web API:

```bash
cargo run -p fragcap-cli --features net -- targets seed --steam --db %TEMP%\hint.db --min-reviews 500
```

Expected: the same command shape as the offline seed, against the wire.

## 6. Full gate set (the slice's Done gate)

```bash
cargo xtask ci
cargo xtask msrv
```

Expected: `cargo xtask ci` passes; its `--all-features` clippy step compiles the
`net` feature (`http_req`) on the pinned toolchain, and its tests run entirely
offline. `cargo xtask msrv` builds the default-feature workspace (no `net`, so no
`http_req`) through `rustup run 1.82` and exits 0. A default `cargo build -p fragcap`
compiles no HTTP client.
