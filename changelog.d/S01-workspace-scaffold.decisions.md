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
