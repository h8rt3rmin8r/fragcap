**2026-08-13** The technology-detection surface (slice S031) landed, and six
decisions were recorded rather than left implicit.

First, the whole `rules.ini` is vendored verbatim and pinned to a specific
upstream commit, while only the direct category sections are applied. The lock
hash is meaningful only over the complete, unmodified file, and the MIT notice
covers the whole work, so vendoring the whole and applying a subset are
independent decisions. The ruleset's two-pass `Evidence` deduction (secondary
hint patterns plus engine inference) is not executed; its section is recognized so
its lines are not miscounted, but it is deferred as a larger, separable concern.
Deferring it does not weaken the first-pass engine, anti-cheat, SDK, emulator,
container, or launcher findings, which are all direct matches.

Second, a ruleset pattern the RE2-family `regex` crate cannot compile is skipped,
counted, and recorded with its technology, and the vendored bytes are never edited
to make a pattern compile. Rewriting a possessive quantifier or an atomic group to
coerce compilation would silently change the ruleset's meaning and break the
faithful-copy and lock-hash properties; a counted skip is honest where a rewrite
is not (P-4, P-9). Of the 376 applied-section patterns in the pinned commit, 373
compile and 3 (atomic-group patterns) are skipped; `compiled + skipped == total`
is asserted over the real asset.

Third, the asset is integrity-locked with a hand-rolled SHA-256 rather than a
crypto crate. The slice constraint is no new dependency and no new `Cargo.lock`
crate; `sha2` would add one even as a dev-dependency. SHA-256 is a fixed,
fully-specified algorithm, verified here against the published NIST vectors, in
the same idiom as the project's hand-rolled glob matcher, pcapng writer, and
schema validator. The check runs inside `cargo test`, already in the gate, so no
workflow or toolchain (pinned-artifact) change is needed. The hash is computed
over the committed bytes as stored (LF, UTF-8, no BOM), documented in the lock's
`note`, so an external `sha256sum` on the committed file reproduces it.

Fourth, the SteamDB attribution is a distinctly named nested file
(`assets/steamdb/THIRD_PARTY_NOTICES.md`), not a bare `NOTICE`. The `cargo xtask
license` check mirrors each publishable crate's root `LICENSE`/`NOTICE` byte for
byte against the repository-root originals (the Apache-2.0 texts); a second file
named `NOTICE` would collide with that intent. MIT only requires the notice to
travel with the copy, which a file beside the vendored asset (published inside the
crate package) satisfies, and MIT is on the constitution's permitted license list,
so the crate's own `license` field stays `Apache-2.0`.

Fifth, the master target schema gains an optional top-level `technologies` array
and a `technology` definition (a category enum, a name, an optional marker path,
and a fidelity reference), applied identically to both the embedded copy and the
published `docs/schema` copy so the drift check stays green. This is an additive,
backward-compatible extension of schema version 1: prior artifacts still validate,
and an artifact carrying `technologies` now validates where the closed property
set would previously have rejected it. No schema-version bump. The category
vocabulary is the superset the Steam-Catalog-Research design named (`engine`,
`anti_cheat`, `sdk`, `framework`, `emulator`, `container`, `runtime`) plus
`launcher` for the ruleset's `Launcher` section; `framework` and `runtime` are
defined for future sources and unpopulated by this ruleset. The hand-rolled
variant validator learned the new property, and a new `invalid-category`
diagnostic code names an out-of-enum category.

A follow-up from PR review sharpened the scaffold's empty case: the scaffold
always runs detection, so it always emits the `technologies` array, empty
included. An empty array says detection ran and found nothing, which a downstream
consumer must be able to tell apart from an older artifact that predates the
field and never ran detection; omitting the key would conflate the two (P-9). The
schema keeps the field optional so those older artifacts still validate. The same
review made the directory walk surface a directory whose enumeration fails
partway (a `read_dir` iterator error on an individual entry) as an unreadable
path rather than skipping it silently, so a partial scan is never reported as a
complete empty one (P-4).

Sixth, detection surfaces on demand and at scaffold time, and never inside the
live capture loop. A `fragcap technologies` command prints the report, and the
Steam scaffold carries the findings into the target artifact it already
materializes. Running detection during `run` would put a filesystem walk on the
capture path and tempt writing technologies into the packet stream, changing files
unmodified analyzers read for no benefit the on-demand-plus-scaffold surface does
not already deliver (P-5). "Output metadata" for this slice is therefore the
target artifact, not the pcapng or JSON Lines packet stream, both of which are
unchanged.
