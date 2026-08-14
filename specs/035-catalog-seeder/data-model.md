# Phase 1 Data Model: Tier 1 Catalog Seeder

New value types and one new store operation, all in `fragcap-targets`. No SQLite
schema change: the columns already exist from S034; this slice adds a merge that
writes a subset of them.

## Catalog source types

```text
trait CatalogSource {
    /// Yield the next batch starting at `cursor` (None = from the beginning).
    /// Returns the batch's entries and the cursor to resume after them
    /// (None = the source is exhausted).
    fn fetch_batch(&self, cursor: Option<&str>) -> Result<CatalogBatch, TargetsError>;
}

CatalogBatch { entries: Vec<CatalogEntry>, next_cursor: Option<String> }

CatalogEntry {
    appid: u32,
    name: Option<String>,          // absent/empty -> stored as NULL (S034 guard)
    classification: Classification, // Game | Other
    review_count: Option<u64>,      // the gate's popularity signal
    owners: Option<u64>,            // secondary metric, when the source has it
    peak_ccu: Option<u64>,          // secondary metric, when the source has it
}

Classification = Game | Other
```

- **`FixtureCatalog`** (default build, tested): constructed from a committed JSON
  document listing catalog entries; paginates its in-memory list by the cursor so
  resumability is exercised offline.
- **`HttpCatalog`** (behind `net`, operator-run): performs read-only HTTPS GETs and
  maps the responses into `CatalogEntry` values; never exercised in CI.

## Corpus gate

```text
CorpusGate { min_reviews: u64 }   // default set in plan, a few hundred

fn admits(&self, entry: &CatalogEntry) -> bool
    // true iff entry.classification == Game
    //      && entry.review_count.is_some_and(|n| n >= self.min_reviews)
```

An entry with `Other` classification, or with `review_count == None`, or below the
threshold, is not admitted (counted as excluded, never as failed).

## Seed summary (P-4/P-9 conservation)

```text
SeedSummary { fetched: u64, written: u64, excluded: u64, failed: u64 }

invariant: fetched == written + excluded + failed   // asserted in tests
```

- `written`: admitted by the gate and merged into the store.
- `excluded`: not admitted by the gate.
- `failed`: an entry that could not be processed (a per-entry error); the run
  continues past it.

## The seeder

```text
fn seed_catalog(
    store: &mut Store,
    source: &dyn CatalogSource,
    gate: &CorpusGate,
) -> Result<SeedSummary, TargetsError>
```

- Reads the catalog tier's `seed_state.resume_cursor` and starts `fetch_batch`
  there (None on a fresh store).
- For each batch: for each entry, `fetched += 1`; if `gate.admits`, `merge_catalog`
  and `written += 1`, else `excluded += 1`; a per-entry error is caught and
  `failed += 1` without aborting.
- After each batch, writes `next_cursor` and a run timestamp into the catalog
  tier's `seed_state` (resumability), then continues until `next_cursor` is None.
- Never deletes or prunes; only inserts and updates via `merge_catalog`.
- The run timestamp is supplied by the caller (the CLI passes the wall clock),
  keeping `seed_catalog` and the store free of ambient time for testability.

## New store operation: `merge_catalog`

```text
fn merge_catalog(
    &mut self,
    appid: u32,
    name: Option<&str>,            // "" or None -> NULL
    review_count: Option<u64>,
    owners: Option<u64>,
    peak_ccu: Option<u64>,
) -> Result<(), TargetsError>
```

SQL:

```sql
INSERT INTO games (appid, name, review_count, owners, peak_ccu)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(appid) DO UPDATE SET
    name         = excluded.name,
    review_count = excluded.review_count,
    owners       = excluded.owners,
    peak_ccu     = excluded.peak_ccu;
```

Only the Tier 1 columns appear. `launcher_mediated`, `token_required`,
`engine_name`, `engine_source`, `engine_confidence`, and the `launch_entries` and
`technologies` rows are never referenced, so a Tier 1 merge over an enriched game
leaves Tiers 2 and 3 intact. An empty or absent `name` binds NULL (the S034
`CHECK (name IS NULL OR length(name) > 0)` still holds and the write path maps `""`
to `None` before binding).

## Seed state (reused from S034, no schema change)

The `seed_state` row for `tier = 'catalog'` carries `resume_cursor` and
`last_run_at`. This slice is the first writer of it. `SeedTier::Catalog` and
`Store::seed_state`/`set_seed_state` already exist.

## Export

Unchanged. After a catalog seed, `export` projects each game as before; a
Tier-1-only row yields a record with `game.app_id`, `game.name` (when present),
`fidelity: heuristic-unverified`, and no `launch` or `engine`. The existing
`validate_value` self-check still guarantees schema conformance.
