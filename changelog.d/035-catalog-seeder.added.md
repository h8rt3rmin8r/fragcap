The targets hint database gained its first seeder (issue #78, slice S035): the
Tier 1 catalog seeder that fills a store's public-catalog columns (application id,
name, and popularity metrics) from a catalog source. A `CatalogSource` trait fixes
the shape the seeder reads, so its fetch-parse-gate-merge pipeline is driven in
every test by an offline `FixtureCatalog` over committed data, with no network; the
live `HttpCatalog` (behind a new `net` feature) is a thin read-only HTTPS adapter
that continuous integration compiles but never runs, the same posture as live
packet capture.

A corpus gate scopes the written rows to titles that are games and clear a
configurable review-count threshold, so the store holds the corpus that matters
rather than the whole ~150k app-list universe. Every fetched title is accounted for
in a seed summary as written, excluded, a within-run duplicate, or failed, and the
counts reconcile (fetched equals written plus excluded plus duplicates plus failed),
so a corpus that could not handle something, or a repeated appid that would
otherwise overstate the total, can never read as complete (P-4, P-9). A title whose
popularity is unknown is excluded rather than admitted on a guess; an entry with a
present but wrong-typed field is counted as failed rather than coerced to an absent
value; and a single unparsable entry is counted as failed without aborting the run.
The offline `targets seed` command requires exactly one catalog source, so `--from`
together with the live `--steam`, or neither, is a usage error rather than a silent
choice.

The seed is idempotent and resumable: it merges each title by application id
through a new `merge_catalog` that writes only the Tier 1 columns, leaving any
launch entries (Tier 2) and engine attribution (Tier 3) a later seeder wrote intact,
and it records a resume cursor after each page so an interrupted seed continues
rather than restarting. It never prunes: a stored title absent from a run is left
as it is. After a seed the store still exports schema-valid JSON, every record
`heuristic-unverified`.

A `fragcap targets seed --from <catalog> --db <store>` command drives the offline
seed and prints the summary; a maintainer builds with `--features net` to seed from
the live catalog with `--steam`. The one new dependency, `http_req` with
`native-tls`, is optional behind `net`, adds 18 MIT/Apache packages to `Cargo.lock`,
and does not touch the minimum supported toolchain, which stays 1.82.
