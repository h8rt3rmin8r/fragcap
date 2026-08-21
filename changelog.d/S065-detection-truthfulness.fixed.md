<!-- spec-impact: 15.7, 17.1 -->
`fragcap targets` no longer reports "Steam DRM" for titles that carry no DRM. It
did so on 28 of 32 rows, on the basis of `steam_api64.dll`, which is the
Steamworks SDK redistributable and ships with essentially every Steam title
whether or not any wrapper is applied. The label recorded an observation nobody
made, and a signal that fires on nearly every row carries no information anyway.
The two signature rows that produced it are gone, and the real discriminator is
matched instead: the Steam DRM wrapper appends a PE section named `.bind` to the
executable it wraps, so the `binary-marker` signature kind, which has been
carried in the vocabulary and inert since it was introduced, is now matchable in
a `section:` form that reads a bounded prefix of a candidate binary's own bytes.
Verified on the operator's machine: Detroit Become Human, Palworld, and
Enshrouded still report Steam DRM; ARC Raiders, Barotrauma, Shale Hill Secrets,
and Trapped with Ivy and Piper no longer do.

Every command that scans now names what the scan did not cover, rather than
counting it and leaving the operator to wonder. `fragcap technologies` on a
directory with more executables than the scan cap used to print "no technologies
detected" with nothing saying the scan had been truncated; `targets add --exe`
recorded the row as incompletely scanned and printed nothing at all. Both say so
now, as do the discovery paths.

The scan is bounded rather than a sweep of the tree. Only executables near the
install root are candidates, capped at a count, each read as a bounded prefix.
A candidate dropped by that cap is counted and named, and a candidate that could
not be opened is recorded unreadable rather than treated as carrying no marker,
so an incomplete scan is never presented as a clean one. The byte-sequence marker
rows for Denuvo, Arxan, and VMProtect stay explicitly inert and counted, so the
signature load still reconciles applied plus inert plus skipped to the rows
loaded.

Two engines that were installed and invisible are now detected. Ren'Py is
recognized from its package directory, its interpreter library, and its archive
extension, and GameMaker from its runtime data file and its platform extension
library. Ren'Py had been recognized by the launch-resolution rules for some time
while remaining unnameable in the listing, which is the drift a new check now
prevents: every engine those rules can select a client executable for must have
a detection signature naming the same product, and adding one without the other
fails the ordinary gate. On the operator's machine, `trapped_with_ivy_piper` now
reports Ren'Py, `shale_hill_secrets` reports GameMaker, and ARC Raiders reports
Unreal, which it did not before.

The one KNOWN column becomes two. It had been doing four jobs: it comma-joined
engines with protection products so a reader could not tell which was which, and
substituted a sentence about capture readiness when it had neither. `ENGINE` now
names the detected engine and `SENSITIVITIES` names the anti-cheat and DRM
products, partitioned on the category the findings already carried. The two
readiness sentences are gone rather than moved, because each was a relabeling of
a state the CAPTURE column already prints. The same partition and the same
coverage state are carried by `targets export`, so the table and the
machine-readable output cannot disagree, and `targets show` reports both lines
too.

A blank technology column now says which kind of blank it is. A row that was
scanned and matched nothing, a row whose scan could not finish, and a row nobody
has scanned were previously indistinguishable, which asserted the first of the
three for all of them. They render as `-`, `incomplete`, and `not scanned`. A
target registered by an earlier build carries no coverage record and therefore
reports `not scanned` until it is re-registered, which is correct: the tool does
not claim a scan it did not run.
