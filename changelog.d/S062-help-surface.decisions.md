<!-- spec-impact: none -->
**2026-08-20** Added clap's `wrap_help` feature, keeping the exact `=4.5.32`
pin. The `Cargo.lock` delta was measured, not estimated, and is exactly one
package: `terminal_size 0.4.4`, MIT OR Apache-2.0, declaring `rust-version
1.71`. It resolves against the `windows-sys 0.61.2` that anstream already brings
through clap's default `color` feature, so the `windows-sys 0.36` pin shared by
`pcap` and the socket-table backend is untouched and no second tree appears.
clap is non-optional in `fragcap-cli`, so unlike `pcap` behind `live` this is
compiled under the 1.82 floor rather than skipped, and `terminal_size` clears it
(it declares `rust-version 1.71`). That is a claim about this package only, not
about the gate as a whole; see the last entry in this fragment for the state of
`cargo xtask msrv` itself.
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

**2026-08-20** Recorded as an environment observation, not a defect. On the
Windows developer machine this slice was written on, `cargo xtask msrv` fails
parsing `constant_time_eq 0.4.2`, which declares `edition = "2024"` and
`rust-version = "1.85.0"` and is reached through `blake3` from S051. That was
first read as a pre-existing break in the repository, and that reading was
wrong: the `minimum supported toolchain` job runs the same
`cargo build --workspace --locked` under the same 1.82 toolchain and compiles
that exact package successfully, on `main` and on this branch, confirmed by
reading the job result rather than the workflow conclusion. This slice's own
addition clears the floor independently (`terminal_size` declares
`rust-version 1.71`) and the lock delta is exactly that one package. The local
divergence is unexplained and no issue was filed for it, because the evidence
does not support asserting a defect; it is written down here so the next person
who hits it locally starts from the comparison rather than from the panic.
