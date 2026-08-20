<!-- spec-impact: none -->
**2026-08-20** Added clap's `wrap_help` feature, keeping the exact `=4.5.32`
pin. The `Cargo.lock` delta was measured, not estimated, and is exactly one
package: `terminal_size 0.4.4`, MIT OR Apache-2.0, declaring `rust-version
1.71`. It resolves against the `windows-sys 0.61.2` that anstream already brings
through clap's default `color` feature, so the `windows-sys 0.36` pin shared by
`pcap` and the socket-table backend is untouched and no second tree appears.
clap is non-optional in `fragcap-cli`, so unlike `pcap` behind `live` this is
compiled by `cargo xtask msrv` under the 1.82 floor; that was run and is green.
The alternative was hand-wrapping the doc comments, which needs no dependency
and was rejected because the available width depends on the longest item name in
whichever list is being rendered, computed at render time, so a string wrapped
for one indent is wrong at another and adding one long flag silently invalidates
every hand-wrap in that list.

**2026-08-20** The help guard enumerates pages from clap's command tree, not
from a hand-written list, and matches leaks by pattern over the whole normalized
page, not by token list over single lines. Both halves are corrections to the
guard that issue #67 shipped and issue #178 caught failing. That guard checked
three pages out of twenty-nine, so nine leaking pages were never looked at, and
it matched the literals `S15`, `S16`, `S17`, and `slice S`, so `S051` through
`S055` slipped past all but one and the bare `(S051)` form slipped past all of
them on a page that was covered. `fragcap-cli` gained one public function,
`command()`, returning the clap tree, so a new subcommand inherits every
assertion the day it is declared. The whole-page normalization is equally
load-bearing and was found by this slice's own analyze gate: with wrapping
turned on, `fragcap extcap --help` renders `specification section` at the end of
one line and `14.5` at the start of the next, so a line-based scan reports that
page clean while it still leaks. A guard defeated by the wrapping shipped in the
same slice would have repeated issue #178 one slice after fixing it.

**2026-08-20** The Cargo-feature clause of the leak rule matches the phrasing
that names a build feature to a user (`` `X` feature ``, ``feature `X` ``), never
the set of feature names declared in the workspace. Four of the five declared
features (`live`, `net`, `targets`, `etw`) are ordinary English words: matching
`net` bare fires on "network" and `targets` fires on most of the `targets`
pages. A rule that cries wolf earns an exception list, and an exception list is
what decayed into the hardcoded token set this slice replaced.

**2026-08-20** Recorded, and deliberately not fixed here: `cargo xtask msrv`
fails on `main` and continues to fail on this branch, at `constant_time_eq
0.4.2`, which declares `edition = "2024"` and cannot be parsed by Cargo 1.82. It
reaches a default build through `blake3 1.8.6` -> `fragcap-targets` (S051) ->
`fragcap` -> `fragcap-cli`, which carries the `targets` feature unconditionally.
Verified pre-existing by stashing this branch and running the gate on `main`,
where it fails identically. This slice's own addition clears the floor
(`terminal_size` declares `rust-version 1.71`) and the lock delta is exactly
that one package. The fix is a choice between pinning `constant_time_eq`,
pinning `blake3`, raising the declared minimum, and dropping the dependency,
which is a decision about the S051 dependency argument rather than about help
text; burying it in a help-surface diff would put an architecture call somewhere
nobody would review for it.
