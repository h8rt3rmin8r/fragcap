**2026-08-13** The Tier 1 catalog seeder (issue #78, slice S035) landed, adding the
project's first HTTP client, and the decisions behind it were recorded.

First, the HTTP client is `http_req` with `default-features = false, features =
["native-tls"]`, behind an off-by-default `net` feature. Two constraints forced it.
The license allowlist eliminated the rustls path: `minreq`+`https` is otherwise the
minimal choice but forces `webpki-roots`, whose bundled Mozilla root store is
CDLA-Permissive-2.0, outside the constitution's allowed set; native-tls uses the
operating system trust store (schannel on Windows) and bundles no roots. The
graph-size rule eliminated every `url`-based client: `ureq` and `attohttpc` pull
`idna` 1.x and the whole ICU4X stack, measured at 42 packages, the graph S025
rejected `boon` for. `http_req` does its own URL parsing and, with native-tls, adds
18 packages (no ICU4X, no `ring`), all MIT or Apache-2.0. The delta was measured by
adding the dependency and diffing `Cargo.lock`, not estimated.

Second, MSRV 1.82 does not bind the client. The `net` feature is off by default and
`cargo xtask msrv` builds the default-feature workspace, so `http_req` (and a
transitive `zeroize` 1.9 that declares edition 2024 / rust-version 1.85) is never
compiled under 1.82, exactly as `pcap` behind `live` is not. Verified by building
net-off under `rustup run 1.82` and net-on under the pinned toolchain; the net graph
must only compile under the `--all-features` clippy gate on 1.96.

Third, the client's maintenance risk is accepted and mitigated. `http_req` is
smaller and less widely used than `ureq`; it sits behind the `CatalogSource` trait,
so replacing it later is a one-module change, and it is compiled but never run in
continuous integration (the offline `FixtureCatalog` drives every test), so a client
regression cannot break the default build or the tested pipeline.

Fourth, the seeder gets a new `Store::merge_catalog` rather than reusing the S034
whole-game replace. The replace path deletes the game row (and, by cascade, its
launch and technology rows) and reinserts; a Tier 1 seeder that knows only appid and
name would thereby erase Tier 2 and Tier 3 data a prior seeder wrote. `merge_catalog`
is an `ON CONFLICT(appid) DO UPDATE SET` of only the catalog columns, leaving the
other tiers intact, which is what the three-tier model requires and is proven by a
test that seeds Tier 1 over a game already carrying an engine.

Fifth, the live source targets SteamSpy's paginated `all` listing (bulk name and
review tallies, paged by page number as the cursor). The precise endpoint is an
operator-facing detail, not a tested contract: any source producing `CatalogEntry`
values feeds the same gate and merge, and the live path is exercised by the
maintainer, not by continuous integration.
