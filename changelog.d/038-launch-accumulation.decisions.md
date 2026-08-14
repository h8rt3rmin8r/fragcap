**2026-08-14** The last piece of issue #78's launch tier was re-scoped, and two
architecture-affecting choices were recorded.

First, the maintainer-run Tier 2 seeder was cancelled and replaced by per-user
local accumulation. A seeder that read the maintainer's own appinfo cache and
baked launch executables into the shipped database would disclose which games the
maintainer owns. Instead the launch tier is filled on each end user's own machine
from that user's own appinfo cache, and the shipped database carries only the
public catalog and engine tiers. Two alternative sources were rejected: a
SteamKit-class network client (a large asynchronous, protobuf, and cryptographic
graph that fails the no-gratuitous-graph test that rejected `ureq` and `boon`, and
a fresh license-audit surface), and a `steamcmd` subprocess (parsing a subprocess's
stdout violates the thin-wrapper non-negotiable, P-7). The chosen source is a
hand-rolled parser of the local binary `appinfo.vdf`: zero new dependency, no `net`
feature, and P-1 clean by construction, because it only reads a file Steam already
wrote. Pooling accumulated launch data across users is deferred to issue #94.

Second, this is the embedded store's first schema-version migration (version 1 to
version 2). Deciding staleness by the appinfo change-number, rather than a plain
have-it-or-not check, requires recording that number per application, and the store
had no column for it. The migration adds one nullable `appinfo_change_number`
column with a single additive `ALTER TABLE`, applied in one transaction alongside
the version stamp; an existing version-1 store keeps every row and reads the new
column as NULL, so it refreshes on the first walk. The column is store-internal
bookkeeping, never exported and never surfaced on the game model, so the export
projection and the published schema are unchanged. This revises the earlier
expectation that the launch tier would need no store migration; the migration is
the cost of honoring change-number staleness rather than mere presence. The
`token_required` column is left unpopulated: it has no reliable appinfo source and
is not exportable.

Third, the appinfo parser is a header-first streaming reader rather than a
parse-everything pass. The reader reads the file header once and then yields one
section header at a time, skipping each section's key-values body by its size
field; the orchestrator decides staleness from the cheap section header and
decodes a body only for an installed application that is missing or stale. An
unchanged application is therefore never decoded, so a repeat run over a
multi-megabyte cache does near-zero work, as the performance goal requires. Two
integrity rules follow the same discipline: a key-values object truncated before
its end marker is a parse failure rather than a silently-partial launch entry
(end of input is permitted only at a section's root, not inside a nested object),
and a file-level fault (an unrecognized magic, a malformed string table, a
truncated tail, or a size that runs past the end) makes the whole cache
untrustworthy, so accumulation records nothing and surfaces the fault rather than
writing the valid-looking prefix (P-4, P-9).
