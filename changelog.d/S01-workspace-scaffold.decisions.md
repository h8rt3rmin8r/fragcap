**2026-08-06** The build toolchain is pinned at 1.96.0 while the minimum
supported version is declared separately as 1.82 and verified by its own check.
Pinning the build channel at the minimum would hold every later slice's
dependencies back to a 2024 toolchain in exchange for a claim obtainable more
cheaply.

**2026-08-06** The repository conventions linter is a task runner subcommand in
Rust rather than a shell script. The house shell standard is a known missing
gap, the task runner is specified to require nothing beyond the language
toolchain, and a Rust check can be unit tested against known-bad input. A
linter whose matcher never fires is indistinguishable from a clean repository.

**2026-08-06** The facade crate depends on `fragcap-core` directly. The
dependency diagram in specification section 8.3 omits that edge, but a facade
that re-exports core types needs core as a direct dependency. The edge violates
neither stated rule. To be promoted to section 8.3 at the next revision.

**2026-08-06** Platform neutrality is enforced by two checks rather than one.
Building `fragcap-core` for a target with no capture backend proves it
compiles portably, but does not fail when a platform crate is added to it,
because such crates compile to nothing off-platform. The manifest check in
`cargo xtask deps` asserts the stronger property that data model rule V-4
requires.

**2026-08-06** The `docs` and `links` workflows are manual-dispatch only until
slice S18 implements them. Both were written to run automatically and exit
non-zero, on the reasoning that a skeleton must not report success for work it
has not done. That reasoning is right and the implementation was wrong: a job
that fails on every push reports red permanently, which carries no more
information than reporting green and trains readers to ignore the signal. Not
reporting at all is the honest option. The `release` workflow keeps its trigger,
because it is tag-gated and will not fire until a release is deliberately cut.

**2026-08-06** Workflow triggers scoped after the first push produced six runs
for three workflows. An unqualified `on: push` plus `on: pull_request` fires
both for any branch with an open pull request, doubling the minute burn.
`push` is now scoped to the default branch, and every workflow carries a
concurrency group so superseded runs cancel rather than queue.

`platform` and `audit` are manual-dispatch only for the same reason `docs` and
`links` are: neither has anything to verify yet. No crate links the capture
library until S09, and the workspace has no external dependencies to audit
until S02. Both regain real triggers with the slice that gives them a subject.

**2026-08-06** Removed `--features platform-tests` from the platform workflow.
No crate declares that feature, so the flag would have failed resolution. It
was never caught locally because the workflow has never run.
