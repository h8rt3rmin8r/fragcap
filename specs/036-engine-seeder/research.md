# Phase 0 Research: Tier 3 Engine Seeder (PCGamingWiki)

This slice takes no new dependency, so there is no crate-graph verification to
redo: the HTTP client and its 18-package `net` graph were verified against the
real toolchain in S035 (see `specs/035-catalog-seeder/research.md` R1) and are
consumed here unchanged. The research below records the design reuse and the two
decisions specific to engine attribution.

## R1. No new dependency; the S035 `http_req` + `net` graph is reused

**Decision**: Add no dependency. The live `HttpEngineFeed` uses the existing
optional `http_req` client behind the existing off-by-default `net` feature, which
S035 introduced for the whole seeder arc.

**Rationale**: S035 chose `http_req` (`default-features = false, features =
["native-tls"]`) precisely so every later seeder could reuse it: it clears the
license set (all 18 packages MIT/Apache-2.0), avoids the `url`/ICU4X graph, uses
the OS trust store (`schannel` on Windows), and sits outside the MSRV floor because
`net` is off in the default and `cargo xtask msrv` builds. Adding a second client
for PCGamingWiki would violate the smallest-graph practice for no benefit; the
`CatalogSource`/`EngineFeed` trait seam already isolates the client so it is a
one-module swap if ever needed. The `net` feature already flows
`fragcap-targets -> fragcap -> fragcap-cli`, so no manifest wiring changes.

**MSRV**: unchanged. `net` off => `http_req` not compiled in the default/msrv
build. This slice re-verifies by building net-off under 1.82 and net-on under the
pinned toolchain, the same check S035 ran.

## R2. EngineFeed is the fetch trait; the offline source drives every test

**Decision**: `trait EngineFeed` yields engine entries in cursor-paged batches.
Two implementations: `FixtureEngineFeed` (reads a committed JSON document, used by
every test and by the offline CLI path) and `HttpEngineFeed` (behind `net`,
performs read-only GETs against PCGamingWiki, run by the operator). The trait is
named `EngineFeed`, not `EngineSource`, because `fragcap-targets` already exports an
`EngineSource` enum (the schema `engine.source` token); the trait is the fetch
abstraction, the enum names provenance, and the distinct names keep both reachable
at the crate root without ambiguity.

**Rationale**: the exact analogue of S035 R2 and the `live`-capture pattern
(specification section 25.1): the seeder's fetch-parse-resolve logic is exercised
entirely offline against `FixtureEngineFeed`, and `HttpEngineFeed` is a thin
adapter compiled under `net` but never run in CI. The trait fixes the shape the
seeder consumes (an entry carries an application id and an optional resolved engine
of name + confidence), so the seeder cannot come to depend on a live-only detail a
fixture cannot express.

**Batching and the cursor.** `fetch_batch(cursor: Option<&str>) ->
Result<EngineBatch, TargetsError>`, where `EngineBatch { entries, failed,
next_cursor }`. The cursor is an opaque string the source defines. The seeder
writes `next_cursor` into the **engine** tier's `seed_state.resume_cursor` after
each batch (S033/S034 already provide `SeedTier::Engine`), so a resumed run passes
the stored cursor back to `fetch_batch`. `FixtureEngineFeed` implements the same
cursor contract over its in-memory list, so resumability is tested offline.

**The real source's endpoint** is an operator-facing detail, not a CI-tested one.
`HttpEngineFeed` queries PCGamingWiki's MediaWiki Cargo query API over the
`Infobox_game` table for the Steam application id, page name, and engine field,
paging by an offset cursor. The precise query is left to the operator and
documented, because it does not affect the tested seeder contract: `HttpEngineFeed`
just has to produce `EngineEntry` values, and the resolve, merge, and summary are
identical whether the entry came from a fixture or the wire. The response's engine
field may be blank (no engine) or list more than one engine (ambiguous); the source
resolves a single value or yields `None`, and the offline fixture models the same
three cases.

## R3. The per-tier merge, and why merge_catalog is not enough

**Decision**: add `Store::merge_engine(appid, engine: &Engine)` performing
`INSERT INTO games (appid, engine_name, engine_source, engine_confidence) VALUES
(...) ON CONFLICT(appid) DO UPDATE SET` of **only** the three engine columns.

**Rationale**: S035's `merge_catalog` writes only the Tier 1 columns and never
touches engine, so it cannot write Tier 3; the foundation's `upsert_game` replaces
the whole game and would erase Tiers 1/2. `merge_engine` updates only
`engine_name`, `engine_source`, `engine_confidence` and leaves `name`,
`review_count`, `owners`, `peak_ccu`, `launcher_mediated`, `token_required`, and
the `launch_entries` and `technologies` rows untouched. This realizes FR-007 and is
proven by seeding engine over a game that already carries a catalog name and launch
entries and asserting the name and launch entries survive (SC-003).

Taking a whole `Engine` value (whose `source` and `confidence` are non-optional and
whose `name` is optional) satisfies the store's both-or-neither CHECK
(`(engine_source IS NULL) = (engine_confidence IS NULL)`) by construction: source
and confidence are always bound together, and `engine_name` may bind NULL. An
`ON CONFLICT` update sets exactly the three engine columns; an insert for an unseen
application id creates an engine-only row (all other columns NULL), which is
schema-valid on export (game.app_id + platform + engine; name omitted).

## R4. The resolve rule and the reused seed summary (P-4/P-9)

**Decision**: The source yields `EngineEntry { appid, engine: Option<ResolvedEngine>
}` where `ResolvedEngine { name: String, confidence: EngineConfidence }`. The seeder
writes iff `engine` is `Some`; otherwise the title is excluded.

The seeder returns the S035 `SeedSummary { fetched, written, excluded, duplicates,
failed }` and asserts `fetched == written + excluded + duplicates + failed`. Per
entry: `fetched += 1`; `Some` engine and a new appid -> `merge_engine` + `written
+= 1`; `Some` engine and an appid already written this run -> `merge_engine`
(idempotent) + `duplicates += 1`; `None` engine -> `excluded += 1`. A source page's
`failed` count (entries the source could not parse) adds to both `fetched` and
`failed`. No entry leaves the run uncounted (P-4), and a missing/ambiguous engine is
excluded honestly rather than guessed (P-9). The seeder never prunes.

**Why excluded rather than failed for a no-engine title.** A title PCGamingWiki
lists but has no engine for is not an error; it is a title with no Tier 3 data. It
is counted excluded, the same category S035 uses for a gate rejection, distinct from
`failed` (an entry that could not be parsed). Conflating the two would make the
summary say the source malfunctioned when it simply had nothing to attribute.

## R5. Confidence mapping and feature wiring

**Confidence is a within-field grade (P-9), supplied per entry.** The store's
`EngineConfidence` enum (`confirmed`/`high`/`medium`/`low`/`unknown`) already
exists (S033/S034). The offline fixture carries an optional `confidence` token per
entry so the store path is exercised across all five values; when the fixture omits
it, the source defaults to a documented token. The live `HttpEngineFeed` maps a
cleanly resolved single engine to `high` (a well-attested community field, still
unverified against the binary, so the row stays heuristic-unverified overall). The
default and mapping are documented and tunable; they are not load-bearing
correctness values. A `confidence` field present but wrong-typed (a number) or a
string outside the enum is a parse failure counted as `failed`, never coerced to the
default (FR-013).

**Feature wiring (all pre-existing from S035, no manifest change):**

- `fragcap-targets`: `net` => `dep:http_req` + the `HttpEngineFeed` impl (added to
  the module the `net` cfg already guards). The trait, `FixtureEngineFeed`, the
  seeder, and `merge_engine` are all in the default build and fully tested offline.
- `fragcap` facade: `net = ["fragcap-targets/net"]` already present; re-exports
  `HttpEngineFeed` under `#[cfg(feature = "net")]`.
- `fragcap-cli`: the `targets seed-engine` command drives `FixtureEngineFeed` from
  a local file with no feature; the real `HttpEngineFeed` path is behind the CLI's
  own `net` passthrough (already present), so a default CLI build seeds offline and a
  maintainer builds `--features net` to seed from the wire.
- `xtask/src/deps.rs`: no new internal or external edge; `fragcap-core` stays
  `bytes`-only. Existing edges unchanged.
