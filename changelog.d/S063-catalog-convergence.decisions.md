<!-- spec-impact: none -->
**2026-08-20** Changed `.github/workflows/release.yml`, a pinned artifact. Its
catalog build step invoked `catalog seed-signatures`, a verb this slice removed;
it now invokes `catalog seed --tier signature`. `--db` stays explicit there
because the step writes the build's staging copy, not the per-user default the
flagless form resolves. Verified by executing the new invocation rather than by
reading the diff, on the standing rule that release infrastructure names CLI
subcommands and `cargo xtask ci` does not cover it: 23 detection signatures
seeded, matching the previous grammar.

**2026-08-20** Corrected the recorded offline path for refreshing the catalog.
The operator's decision was to drop `catalog update` and document the offline
route, following issue #175's option 2, whose route was "download `catalog.db`
from the releases page and run `catalog import`". Measurement shows that route
is hollow: `assets/hint-seed.json` carries zero records and is what the release
imports, so the published `catalog.db` is the same title-less store the user
already has and downloading it changes nothing. The decision's substance is kept
(no network code in the shipped binary, no dead end) and its mechanism is
corrected to creating the store locally, which the first-run bootstrap and the
bundled signature document already do offline. `doctor` therefore offers an
action it can perform rather than one that always degrades.

**2026-08-20** Declined issue #175's request that the release build enable the
network feature or that slice S056's npcap installer fetch be deleted. The
operator's decision forbids network code in the shipped binary, which settles
the first. The second rests on reading what #175 objects to: a binary that
*promises* a remediation it cannot perform. The npcap action makes no such
promise. Its degraded form offers the official download page and states plainly
that this build cannot fetch, which is a real step a user can take, and it was
the catalog action alone that told users to rebuild fragcap from source. The
fetch code stays exercised in maintainer builds and under `--all-features`.

**2026-08-20** Recorded, not closed: the released `catalog.db` has no titles.
The shipped store carries detection signatures and zero catalog records, and
with no network in the shipped build there is no compiled-in way to gain any.
This is not fatal, since discovery resolves titles from Steam and the signature
table (33 on the developer machine against a zero-record catalog), but it means
the title tier is a maintainer-populated enrichment that is empty in every
release. Filling it is a data-publishing decision rather than a command-surface
change, so this slice reports it instead of absorbing it.
