**2026-08-12** The engine-rule provider (issue #77, slice S029) landed, and five
decisions were recorded rather than left implicit.

First, a resolved engine-rule answer gets its own `TargetOrigin::EngineRule`
variant carrying an `EngineRuleTarget`, distinct from the profile and observed
origins. Reusing the observed origin would have been a lie: it carries a process
identifier, and an engine rule resolves a file on disk with no process running
yet. The new origin carries the recognized engine, the resolved client's image
name and full path, and the `MatchPredicates` the pipeline binds it by, mirroring
how `ObservedTarget` carries its identity so watch mode (S028) can bind the
process once it appears.

Second, the resolution request gained an optional `install_root` input and a
`for_install` constructor, plus a `with_install_root` builder so a request that
also carries a profile reference can offer the engine-rule provider a directory.
An engine rule reasons about a directory tree, not a process, and modelling the
input as a directory is exactly what the S030 platform walker will produce, so the
walker composes with this provider without changing it. The builder is what lets
the precedence relationship be tested honestly: a single request carrying both a
profile reference and an install root resolves to the profile when one matches and
to the engine rule when none does, regardless of provider registration order.

Third, an ambiguous layout (more than one candidate client under one rule)
declines rather than picking one, and records an `EngineRuleAmbiguity` note
surfaced through `Unresolved`. A filesystem heuristic that cannot tell two
shipping binaries apart must not present an arbitrary pick as the answer (P-9),
and a silent decline would violate P-4; declining plus a surfaced note satisfies
both, and runtime observation is the arbiter the cascade already relies on for the
identical-process case. Ren'Py's common dual-launcher shipping (a 64-bit exe and a
32-bit sibling) is resolved this same way: the rule declines as ambiguous and lets
observation choose, rather than encoding a fragile name-based tie-break.

Fourth, the rules key on install-layout convention only, never on launcher tokens
or AppData artifacts. AppData is written on first run, which is after the moment a
pre-launch resolver runs, so it is useless here; launcher tokens are
storefront-specific while an engine rule is engine-general. Layout matching is
component-based and case-insensitive (a directory named `Win64Extra` does not
satisfy the `Win64` component) and separator-agnostic, correct on the
case-insensitive Windows filesystem the rules target. The directory scan is
depth-bounded rather than an unbounded walk, so it stays cheap on a large install
and does not match tools buried deep in the tree.

Fifth, the provider stays in `fragcap-profile` as a new `engine_rule` module and
adds no dependency: `std::fs` and `std::path` are the whole toolkit, `MatchPredicates`
is built in-crate via `Default` and the existing setters, and nothing is added to
`fragcap-core` (allowlist stays `["bytes"]`) or to `Cargo.lock`. The targeting
fidelity this provider stamps (`heuristic-unverified`) remains separate from the
attribution fidelity (`Live`/`Retained`/`None`) in `fragcap-core::attribution`,
which this slice does not touch. MSRV stays 1.82.
