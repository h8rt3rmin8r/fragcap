<!-- spec-impact: 15.7 -->

Technology detection is now data, not code. The engine, anti-cheat, and DRM
signatures that identify a game from its install directory live in a `signature`
table in the shipped catalog database, and one generic matcher evaluates them.
Adding a signature of an implemented kind is honored on the next scan with no code
change and no release: `fragcap targets seed-signatures` refreshes detection
capability through the same catalog-seed path that refreshes the title catalog,
seeded from a bundled set that covers the engines Unity, Unreal, Source, Godot,
CryEngine, and RE Engine; the anti-cheat products Easy Anti-Cheat, BattlEye,
Vanguard, mhyprot, nProtect GameGuard, and Xigncode3; and the DRM products Denuvo,
Steam DRM, Arxan, and VMProtect.

Detection now runs automatically in the scan phase of every discovery source: a
directory whose shape matches an engine signature is a game, the walk stops
descending on the hit, and the candidate carries the detected engine and any
anti-cheat or DRM found alongside it as evidence. A locally detected engine is
stamped `verified`, which outranks the `heuristic-unverified` attribution a remote
catalog carries, because evidence on disk outranks a remote claim. The standalone
`fragcap technologies --path <dir> --catalog-db <catalog.db>` command a researcher
uses to inventory an unknown binary reads the same table, and `fragcap steam
profile` labels a scaffolded profile's technologies from it when a catalog is
given.

A detected anti-cheat or DRM product is neutral evidence, never a gate. Nothing in
any output frames a title as off limits, risky, or discouraged, and a title with
no recorded online mode is still fully capturable.

Filename, directory-shape, and PE-version-string matching are evaluated this
slice; the binary-marker kind is carried in the schema for the deep-protection DRM
products (Denuvo, Arxan, VMProtect) and left inert, counted and surfaced as
not-yet-matchable rather than dropped. The `signature` table is an additive
migration advancing the catalog schema to version 5. The change adds no dependency
and removes the vendored SteamDB ruleset that was compiled into the binary,
authored for depot manifests rather than on-disk installs, and never validated
against a real game.
