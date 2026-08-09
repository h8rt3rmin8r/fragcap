### Profile schema, parsing, and validation (S05)

`fragcap-profile` gains specification section 15 in full: the schema version 1
TOML format, the four-step resolution order, and a validation set that reports
every problem found rather than stopping at the first. `fragcap-core` gains the
duration grammar three later slices need.

**`Profile::parse`** returns either a validated profile or every diagnostic
found, and it is the only way to obtain a `Profile`. There is no public
constructor, no public field, and no `Default`, so section 15.4's requirement
that validation run before every capture cannot be forgotten by a later caller.

**Every problem in one report.** A profile with four mistakes yields four
diagnostics from one call. Each carries a code from a closed enumeration, a
dotted key path such as `stage[1].match.descends_from`, the byte offset the
parser reported, and the one-based line and column derived from it. The set is
sorted so an author reads it in the order they read their file, and so two runs
produce identical output. Two things stop accumulation: a TOML syntax fault
yields one diagnostic, because a document that did not parse has no tables to
check, and an unsupported schema version yields one, because every other fault
is then likely a consequence of reading a later format under this one's rules.

**Validation is the section 15.4 set plus three checks in the same failure
class.** Structural: schema version support, required field presence, type
correctness, and closed key sets for all five tables. Semantic: role name
uniqueness, at most one terminal stage, `descends_from` resolving within the
profile, regular expression compilation, glob well-formedness, duration parsing,
and at least one non-service stage. Added: a terminal stage must be a `session`
stage, the `descends_from` relation must be acyclic, and every role named in
`capture.roles` must be declared.

**The ambiguous image match check is exact.** For every pair of stages whose
`exe` patterns can match a common image name, the profile is refused unless both
stages carry a further predicate. The intersection decision is a reachability
walk over the two patterns rather than an approximation, because a false
negative admits the failure the check exists to prevent: a stage bound to the
wrong process among several sharing an image name produces a capture that exits
zero, is well formed, and contains no gameplay. One focal title runs three
processes under one image name and only the last holds sockets.

**Unknown keys are refused rather than ignored.** An author who writes
`payloads = false` intending `payload = false` is told so, rather than receiving
a capture containing contents they meant to exclude. The schema version is what
makes that safe: a profile written for a later fragcap says so and is told so.

**Resolution takes its search path from the caller.** The resolver implements
section 15.3's order over directories it is given and a bundled set it is given,
and never asks the operating system where a user's configuration lives. A
reference used in steps two through four must be a valid identifier and is
refused before any path is joined to it, so a traversal-shaped reference cannot
reach outside the search directories. A search directory that is absent is
skipped; a candidate file that has won its step and cannot be read is an error
rather than a fall-through, because falling through would silently substitute a
profile the operator did not choose. A successful resolution reports which of
the four steps supplied the profile.

**Duration literals** are one unsigned integer and one required unit from `ms`,
`s`, `m`, `h`. A bare integer, a zero, a fraction, a sign, a compound form such
as `1h30m`, and an overflowing value are all refused. The grammar lives in
`fragcap-core` because the profile schema, the command line, and ring mode all
need the same one.

Nothing here observes a process. A profile describes process topology; this
crate reads the description. Predicate evaluation against real process events
arrives with S12, which uses the same regular expression engine that validated
the patterns.
