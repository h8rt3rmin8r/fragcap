The targets hint database is now wired into the live resolution cascade as its
precedence-2 provider (issue #78, slice S037), the final wiring step of the
targeting redesign. A new `HintDatabaseProvider` reads a store row for a Steam
application id carried on the resolution request and, when that row names a single
usable Windows client executable, answers with a heuristic-unverified target keyed
on that executable, carrying the row's launcher-mediated flag and engine name as
facts. It sits below authored and curated profiles and above the engine rule, the
platform walker, and runtime observation, so a title the community has documented
resolves without an operator authoring a profile first, while a live observation
always overrides it.

The provider never guesses. A sparse catalog-only row, an engine-only row with no
launch executable, a request with no application id, a launcher-mediated row (whose
launch executable is the publisher launcher rather than the socket-holding client,
so resolving it would record the launcher as the game and lose the gameplay
traffic), and a row whose Windows launch entries name more than one distinct
executable are all declines, so the cascade falls through to the lower providers
rather than arming a capture against a launcher or a guessed process (P-4); an
ambiguous decline records the application id and candidate count, surfaced by the
`run` error, so a not-resolved outcome can explain itself. Launch entries are first restricted
to those applicable to Windows and reduced to the set of distinct executable file
names, so one executable repeated across arguments, architectures, and beta
branches is one candidate, not an ambiguity. Every answer is stamped
`heuristic-unverified` with provenance `hint-db`, the same name the database's
export projection uses, and it carries no on-disk path because the store knows the
executable name but not where a machine installed the title (P-9).

The database is optional and its absence is never an error. A `fragcap run
--hint-db <path>` option, and a `FRAGCAP_HINT_DB` environment override, supply a
database for resolution; a `--steam` capture then offers the provider the
application id while the install root stays available to the lower providers. When
no database is supplied, or the path does not exist, or the build excludes the
targets feature, precedence 2 is simply empty and resolution is byte-identical to
before this slice. A database that is present but cannot be opened (corrupt or a
wrong schema version) fails loudly at the boundary where the operator named it,
rather than being silently treated as absent. The whole feature is testable
offline: the cascade ordering, every decline, and the graceful degradation are
proven over an in-memory store with no network and no game.

The concrete provider lives in `fragcap-targets`, which already depends on
`fragcap-profile` and implements its provider trait, so no dependency is
introduced from the resolver's home crate onto the targets database; the
no-answer stub the profile crate held at precedence 2 is removed. This mirrors the
S030 platform walker, which lives in `fragcap-steam` for the same reason. No new
dependency is taken.
