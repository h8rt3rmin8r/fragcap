# Contracts: Tier 1 Catalog Seeder

Two surfaces: the `fragcap-targets` library additions and the `fragcap targets
seed` CLI. The JSON export contract is unchanged from S034.

## Library contract (`fragcap-targets`)

- `CatalogSource` trait: `fetch_batch(cursor) -> Result<CatalogBatch,
  TargetsError>`. Implementations: `FixtureCatalog` (always available),
  `HttpCatalog` (behind `net`).
- `FixtureCatalog::from_json(text) -> Result<FixtureCatalog, TargetsError>`: parse a
  committed catalog document into an offline source. A malformed document is a
  `TargetsError::Seed`.
- `CorpusGate { min_reviews }` with `admits(&entry) -> bool`.
- `seed_catalog(store, source, gate, now) -> Result<SeedSummary, TargetsError>`: the
  seeder. Resumes from the catalog `seed_state`, merges admitted entries, counts
  every entry, updates the cursor per batch, never prunes.
- `Store::merge_catalog(appid, name, review_count, owners, peak_ccu)`: Tier 1 upsert
  that leaves other tiers intact.
- `SeedSummary { fetched, written, excluded, failed }` with the conservation
  invariant.
- Behind `net`: `HttpCatalog::new(config) -> HttpCatalog` performing read-only HTTPS
  GETs against the public Steam Web API. No process handle, no capture (P-1).

### Error contract

`TargetsError` (from S034) is reused. A per-entry failure inside `seed_catalog` is
counted in `SeedSummary.failed` and does not surface as an `Err`; an `Err` from
`seed_catalog` is a whole-run failure (store I/O, or a source `fetch_batch` error
that is not per-entry). No path prunes or silently drops.

## CLI contract

### `fragcap targets seed --from <FILE> --db <DB> [--min-reviews <N>]`

- `--from <FILE>`: a local catalog document; drives `FixtureCatalog`. Offline, no
  network, always available.
- `--db <DB>`: the store to seed (created if absent).
- `--min-reviews <N>`: the corpus threshold; defaults to the documented value.
- Behavior: runs `seed_catalog`, prints the summary (fetched / written / excluded /
  failed) so the operator can see the corpus is what it says it is.
- Exit codes: `0` success (even with some `failed` entries, which are reported); `1`
  operational failure (unreadable file, store I/O); `2` usage error.

### `fragcap targets seed --steam --db <DB> [--min-reviews <N>]` (behind `net`)

- Same command, `--steam` selecting `HttpCatalog` instead of a file. Present only in
  a build with the `net` feature; a default build reports that live seeding needs
  the `net` feature rather than offering a flag that cannot work.
- Run by the operator/maintainer; never exercised in CI.

The two sub-forms are mutually exclusive (a clap group), mirroring `run`'s
`--profile`/`--install-dir`/`--steam` group.

## What stays offline and tested

Every automated test drives `FixtureCatalog`. The conservation invariant, the gate,
the per-tier merge non-clobber property, resumability, and the post-seed schema-valid
export are all asserted with no network. `HttpCatalog` is compiled under `net`
(so the `--all-features` clippy gate covers it) but never run by a test.
