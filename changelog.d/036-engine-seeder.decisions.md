**2026-08-13** The Tier 3 engine seeder (issue #78, slice S036) landed, and the
decisions behind it were recorded.

First, no new dependency was taken. The live source reuses the `http_req` client
and off-by-default `net` feature S035 introduced for the whole seeder arc, rather
than adding a second HTTP client for PCGamingWiki. The `CatalogSource` / `EngineFeed`
trait seam already isolates the client, and adding another would violate the
smallest-graph practice for no benefit. `Cargo.lock` is unchanged, the license set
is unchanged, and MSRV 1.82 stays non-binding because `net` is off in the default
and toolchain-check builds, exactly as for `pcap` behind `live`. Re-verified by
building net-off under `rustup run 1.82` and net-on under the pinned toolchain.

Second, the seeder gets a new `Store::merge_engine`, the engine analogue of S035's
`merge_catalog`. Neither the S034 whole-game replace (which would erase Tiers 1 and
2) nor `merge_catalog` (which never writes engine) fits. `merge_engine` is an
`ON CONFLICT(appid) DO UPDATE SET` of only the three engine columns, leaving name,
metrics, launcher flags, and the launch and technology rows intact, and inserting an
engine-only row for an unseen application id. It takes a whole `Engine` value so
source and confidence are always bound together, satisfying the store's
both-or-neither CHECK by construction. Proven by a test that seeds an engine over a
catalog-seeded, launch-bearing game and asserts the name and launch entries survive.

Third, the fetch trait is named `EngineFeed`, not `EngineSource`. `fragcap-targets`
already exports an `EngineSource` enum (the schema `engine.source` token), so a
same-named trait would clash at the crate root. The trait names the paged source the
seeder reads from; the enum names provenance and is always `pcgamingwiki` for this
tier. The distinct names keep both reachable without ambiguity.

Fourth, the live-source CLI flag is `--pcgamingwiki`, not `--steam`. The engine tier
is keyed by Steam application id, but the data comes from PCGamingWiki, so naming the
flag `--steam` (as S035's catalog `--steam` does for a Steam-derived catalog) would
misattribute the source, a P-9 naming concern. The subcommand is `seed-engine`, its
own command rather than an extension of `seed`, because no corpus-review threshold
applies to engine enrichment; the offline `--from` path is always available and the
`--pcgamingwiki` path is present only under `net`, the two a mutually exclusive
clap group.

Fifth, the within-field engine confidence is supplied per entry by the feed, and the
live source maps a cleanly resolved single engine to `high` (a documented, tunable
default: a well-attested community field, still unverified against the binary, so
the row stays heuristic-unverified overall). The offline fixture supplies the token
directly so the store path is exercised across all five confidence values; the
default is not a load-bearing correctness value.
