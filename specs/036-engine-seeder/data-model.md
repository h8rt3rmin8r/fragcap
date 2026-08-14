# Phase 1 Data Model: Tier 3 Engine Seeder (PCGamingWiki)

New value types, one new store operation, and one new seeder function, all in
`fragcap-targets`. No SQLite schema change: the engine columns, the both-or-neither
CHECK, and `SeedTier::Engine` all exist from S033/S034; this slice adds a merge
that writes the engine columns and a seeder that drives it. `SeedSummary` and
`Engine`/`EngineSource`/`EngineConfidence` are reused, not redefined.

## Engine source types

```text
trait EngineFeed {
    /// Yield the next batch starting at `cursor` (None = from the beginning).
    /// Returns the batch's entries, a count of items the source could not parse
    /// in this page (counted as failed, never dropped), and the cursor to resume
    /// after them (None = the source is exhausted).
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<EngineBatch, TargetsError>;
}

EngineBatch { entries: Vec<EngineEntry>, failed: u64, next_cursor: Option<String> }

EngineEntry {
    appid: u32,
    engine: Option<ResolvedEngine>,   // None = no engine, or ambiguous -> excluded
}

ResolvedEngine {
    name: String,                     // non-empty; the resolved single engine name
    confidence: EngineConfidence,     // within-field grade; defaulted when source omits it
}
```

- **`FixtureEngineFeed`** (default build, tested): constructed from a committed
  JSON document listing engine entries; paginates its in-memory list by the cursor
  so resumability is exercised offline. Its parse maps each document entry to an
  `EngineEntry`, resolving the engine field and counting an unparsable entry as
  failed (surfaced on the first page, like `FixtureCatalog`).
- **`HttpEngineFeed`** (behind `net`, operator-run): performs read-only HTTPS GETs
  against PCGamingWiki's Cargo query API and maps the responses into `EngineEntry`
  values; never exercised in CI.

### Fixture document shape

An array of entry objects. Each entry:

```text
{
  "appid": <u32, required>,               // missing / wrong-typed -> failed
  "engine": <string | [string, ...] | absent>,
  "confidence": <string, optional>        // one of the EngineConfidence tokens
}
```

Engine field resolution (offline mirror of the live field's blank/single/multi
cases):

- absent, null, `""`, or `[]`            -> `engine: None` (no engine -> excluded)
- a single string, or a one-element array -> `ResolvedEngine { name, .. }` (written)
- an array with more than one element      -> `engine: None` (ambiguous -> excluded)
- any other JSON type (number, object)     -> entry is malformed -> failed

`confidence`:

- absent / null    -> the documented default token (a resolved engine)
- a valid token    -> that `EngineConfidence`
- wrong-typed or out-of-set -> entry is malformed -> failed (never coerced, P-9)

`confidence` only bears on a resolved (`Some`) engine; on an excluded entry it is
ignored (but a wrong-typed value is still a parse failure).

## Seed summary (reused from S035, P-4/P-9 conservation)

```text
SeedSummary { fetched, written, excluded, duplicates, failed }   // all u64

invariant: fetched == written + excluded + duplicates + failed   // asserted in tests
```

- `written`: distinct appids whose resolved engine was merged into the store.
- `excluded`: entries with no engine or an ambiguous engine (left absent, not
  guessed).
- `duplicates`: a resolved appid already written earlier in this run; merged
  idempotently but counted once as written, so the summary does not overstate.
- `failed`: an entry that could not be parsed (missing/wrong-typed appid, a
  wrong-typed or out-of-set confidence, or a wrong-typed engine field); the run
  continues past it.

## The seeder

```text
fn seed_engine(
    store: &mut Store,
    source: &dyn EngineFeed,
    now: Option<String>,
) -> Result<SeedSummary, TargetsError>
```

- Reads the **engine** tier's `seed_state.resume_cursor` and starts `fetch_batch`
  there (None on a fresh store).
- For each batch: `fetched += batch.failed; failed += batch.failed`; then for each
  entry, `fetched += 1`; if `entry.engine` is `Some`, `merge_engine` and count
  `written` (new appid) or `duplicates` (repeat); else `excluded += 1`.
- After each batch, writes `next_cursor` and the run timestamp into the engine
  tier's `seed_state` (resumability), then continues until `next_cursor` is None.
- Never deletes or prunes; only inserts and updates via `merge_engine`.
- The run timestamp is supplied by the caller (the CLI passes the wall clock),
  keeping `seed_engine` and the store free of ambient time for testability.

Note there is **no gate parameter**: unlike Tier 1 (which scopes a corpus by a
review threshold), Tier 3 enriches whatever titles the source names an engine for;
the keep/exclude decision is entirely "did a single engine resolve".

## New store operation: `merge_engine`

```text
fn merge_engine(&mut self, appid: u32, engine: &Engine) -> Result<(), TargetsError>
```

SQL:

```sql
INSERT INTO games (appid, engine_name, engine_source, engine_confidence)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(appid) DO UPDATE SET
    engine_name       = excluded.engine_name,
    engine_source     = excluded.engine_source,
    engine_confidence = excluded.engine_confidence;
```

Only the engine columns appear. `name`, `review_count`, `owners`, `peak_ccu`,
`launcher_mediated`, `token_required`, and the `launch_entries` and `technologies`
rows are never referenced, so an engine merge over a catalog-seeded, launch-bearing
game leaves Tiers 1 and 2 intact. `engine.source` and `engine.confidence` are bound
together (non-optional on `Engine`), satisfying the both-or-neither CHECK;
`engine.name` binds `NULL` when `None` (though the PCGamingWiki seeder always
supplies a name). An insert for an unseen appid creates an engine-only row (other
columns NULL), which is schema-valid on export.

## Seed state (reused from S034, no schema change)

The `seed_state` row for `tier = 'engine'` carries `resume_cursor` and
`last_run_at`. This slice is the first writer of it. `SeedTier::Engine` and
`Store::seed_state`/`set_seed_state` already exist.

## Export

Unchanged. After an engine seed, `export` projects each game as before; a row with
an engine yields a record carrying `engine.name`, `engine.source: pcgamingwiki`,
`engine.confidence`, and `fidelity: heuristic-unverified`. An engine-only row (no
name) yields a record with `game.app_id`, `game.platform`, and `engine`, name
omitted. The existing `validate_value` self-check still guarantees schema
conformance.
