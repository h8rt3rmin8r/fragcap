# Contracts: Tier 3 Engine Seeder (PCGamingWiki)

Two surfaces: the `fragcap-targets` library additions and the `fragcap targets
seed-engine` CLI. The JSON export contract is unchanged from S034.

## Library contract (`fragcap-targets`)

- `EngineFeed` trait: `fetch_batch(cursor) -> Result<EngineBatch, TargetsError>`.
  Implementations: `FixtureEngineFeed` (always available), `HttpEngineFeed`
  (behind `net`).
- `FixtureEngineFeed::from_json(text) -> Result<FixtureEngineFeed, TargetsError>`:
  parse a committed engine document into an offline source. A structurally invalid
  document (not a JSON array) is a `TargetsError::Seed`; a single unparsable entry
  is tolerated and surfaced as failed (mirrors `FixtureCatalog`).
- `seed_engine(store, source, now) -> Result<SeedSummary, TargetsError>`: the seeder.
  Resumes from the engine `seed_state`, merges resolved engines, counts every entry,
  updates the cursor per batch, never prunes. No gate parameter.
- `Store::merge_engine(appid, &Engine)`: Tier 3 upsert that writes only the engine
  columns and leaves other tiers intact; inserts an engine-only row for an unseen
  appid.
- `SeedSummary { fetched, written, excluded, duplicates, failed }` reused from S035,
  with the conservation invariant `fetched == written + excluded + duplicates +
  failed`.
- Reused value types: `Engine`, `EngineSource` enum (the schema `engine.source`
  token type), `EngineConfidence`. The fetch trait is named `EngineFeed`, not
  `EngineSource`, precisely so it does not clash with this enum at the crate root.
- Behind `net`: `HttpEngineFeed::new()` / `with_base_url(..)` performing read-only
  HTTPS GETs against PCGamingWiki's public query API. No process handle, no capture
  (P-1).

Name disambiguation: `fragcap-targets` already exports an `EngineSource` **enum**
(the schema `engine.source` token: `pcgamingwiki` / `exe_heuristic` /
`depot_filename_rules`). This slice's fetch abstraction is therefore a trait named
`EngineFeed` (module `engine_feed`), so both the enum and the trait are reachable at
the crate root without ambiguity and `cargo build` is clean. The enum names
provenance and is always `Pcgamingwiki` for this tier; the trait names the paged
source the seeder reads from.

### Error contract

`TargetsError` (from S034) is reused. A per-entry failure inside `seed_engine` is
counted in `SeedSummary.failed` and does not surface as an `Err`; an `Err` from
`seed_engine` is a whole-run failure (store I/O, or a source `fetch_batch` error
that is not per-entry, such as a network or whole-page parse failure). No path
prunes or silently drops.

## CLI contract

### `fragcap targets seed-engine --from <FILE> --db <DB>`

- `--from <FILE>`: a local engine document; drives `FixtureEngineFeed`. Offline,
  no network, always available.
- `--db <DB>`: the store to seed (created if absent).
- Behavior: runs `seed_engine`, prints the summary (fetched / written / excluded /
  duplicates / failed) so the operator can see the Tier 3 enrichment is what it says
  it is.
- Exit codes: `0` success (even with some `failed` entries, which are reported); `1`
  operational failure (unreadable file, store I/O); `2` usage error.

### `fragcap targets seed-engine --pcgamingwiki --db <DB>` (behind `net`)

- Same command, `--pcgamingwiki` selecting `HttpEngineFeed` instead of a file.
  Present only in a build with the `net` feature; a default build reports that live
  engine seeding needs the `net` feature rather than offering a flag that cannot
  work.
- The flag names its actual source (PCGamingWiki), not `--steam`: the tier is keyed
  by Steam application id but the data is PCGamingWiki's, so `--steam` would
  misattribute the source (P-9). Run by the operator/maintainer; never exercised in
  CI.

`--from` and `--pcgamingwiki` are mutually exclusive (a clap group), mirroring the
`seed` command's `--from`/`--steam` group.

## What stays offline and tested

Every automated test drives `FixtureEngineFeed`. The conservation invariant, the
resolve rule (resolved vs no-engine vs ambiguous vs malformed), the per-tier merge
non-clobber property (engine over a catalog-seeded, launch-bearing game preserves
the name and launch entries), resumability, the never-prune property, and the
post-seed schema-valid export (including an engine-only row) are all asserted with
no network. `HttpEngineFeed` is compiled under `net` (so the `--all-features`
clippy gate covers it) but never run by a test.
