The targets hint database gained its Tier 3 seeder (issue #78, slice S036): the
engine seeder that fills a store's engine-attribution columns (engine name, source
`pcgamingwiki`, and a confidence grade) from PCGamingWiki, keyed by Steam
application id. An `EngineFeed` trait fixes the shape the seeder reads, so its
fetch-parse-resolve-merge pipeline is driven in every test by an offline
`FixtureEngineFeed` over committed data, with no network; the live `HttpEngineFeed`
(behind the existing `net` feature) is a thin read-only HTTPS adapter over
PCGamingWiki's MediaWiki Cargo query API that continuous integration compiles but
never runs, the same posture as the S035 catalog source and live packet capture. No
new dependency is taken: the seeder reuses the `http_req` client S035 chose for the
whole seeder arc.

The seeder writes an engine only for a title that resolves to a single unambiguous
engine name; a title with no engine, or an ambiguous one (the feed names more than
one), is left absent and counted excluded, never guessed (P-9). Every fetched title
is accounted for in the reused seed summary as written, excluded, a within-run
duplicate, or failed, and the counts reconcile (fetched equals written plus excluded
plus duplicates plus failed), so a partial enrichment can never read as complete
(P-4, P-9). A present but wrong-typed field, or an out-of-set confidence token, is
counted as failed rather than coerced to a default and reported as excluded, so the
summary never misattributes why an engine is or is not present; a single unparsable
entry does not abort the run. The engine confidence is a within-field grade of one
heuristic field, never a fifth fidelity tier: a seeded engine leaves the record
`heuristic-unverified` however confident the field grade.

The seed is idempotent and resumable: it merges each engine by application id
through a new `merge_engine` that writes only the engine columns (source and
confidence bound together to satisfy the store's both-or-neither invariant),
leaving any catalog data (Tier 1) and launch data (Tier 2) a prior seeder wrote
intact, and inserting an engine-only row for an application id the store has not
seen. It records a resume cursor under the engine tier after each page so an
interrupted seed continues rather than restarting, and it never prunes: a stored
title absent from a run is left as it is. After a seed the store still exports
schema-valid JSON.

A `fragcap targets seed-engine --from <engine-doc> --db <store>` command drives the
offline seed and prints the summary; a maintainer builds with `--features net` to
seed from PCGamingWiki with `--pcgamingwiki`. The live flag names its actual source
rather than `--steam`: the tier is keyed by Steam application id but the data is
PCGamingWiki's.
