# Phase 0 Research: Target Resolution Cascade -- Resolver Core

Decisions taken under autopilot from the constitution, issue #77, and the
S025/S026 code on main. Each records the alternatives weighed and why the chosen
option wins.

## D-1: Where the resolver, Target, and shared metadata live

**Decision**: All in `fragcap-profile`. `Target`, the `TargetProvider` trait, and
`TargetResolver` in new files (`target.rs`, `resolver.rs`, `providers.rs`); the
shared metadata types `FidelityTier`, `Provenance`, and `Kind` in `schema.rs`
because `Profile` carries them too.

**Alternatives**: (a) a new `fragcap-target` crate; (b) in `fragcap-core`.
Rejected: (a) is premature and adds a workspace edge the deps check would have to
learn, for logic that reads the profile schema and the process tree, both of which
`fragcap-profile` already depends on; (b) violates P-2 (core allowlist is
`["bytes"]`) and would drag the profile schema into core. `fragcap-profile` is the
one crate that already reads both the stages and the tree (see `matching.rs`), so
it is where the cascade belongs.

## D-2: FidelityTier naming and ordering (higher trust is the greater value)

**Decision**: A new enum `FidelityTier` in `fragcap-profile::schema`, distinct from
the attribution `Fidelity` (`Live`/`Retained`/`None`) in
`fragcap-core::attribution`. Variants are declared in ascending trust order so
`#[derive(Ord)]` makes the more trusted tier the greater value:

```rust
pub enum FidelityTier { Observed, HeuristicUnverified, Verified, Authored }
```

`FidelityTier::Authored > FidelityTier::Observed` then reads naturally as "more
trusted". Parsing from the schema string maps the four names to variants
regardless of declaration order, and a `rank()`/`Ord` gives the resolver a total
comparison.

**Alternatives**: (a) reuse the attribution `Fidelity`; (b) declare in the
schema's descending order and invert comparisons; (c) a bare enum with a separate
`rank()` returning an integer. Rejected: (a) is the exact conflation the spec
forbids, on a different axis in a different crate; (b) makes every comparison read
backwards ("authored is less than observed"), an invitation to a sign bug in the
precedence assertion; (c) is redundant once `Ord` already encodes the order. The
ascending declaration plus derived `Ord` is the least error-prone.

## D-3: Precedence is a provider ordering, imposed by the resolver

**Decision**: Each provider reports a `Precedence` (a five-position enum:
`Profile`, `HintDatabase`, `EngineRule`, `PlatformWalker`, `RuntimeObservation`).
The `TargetResolver` sorts its providers by `Precedence` before querying, so the
result never depends on the order providers were registered or iterated in. The
permutation test registers the providers in every order and asserts one answer.

**Why five positions, not six**: The issue's chain names "authored package" and
"verified profile" as the top two. In the code, both are profiles, and `resolve()`
already returns exactly one profile by its own four-step file precedence (explicit
path > command-line dir > user dir > bundled). So the `ProfileProvider` is one
provider at the top band, and the authored-versus-verified distinction is realized
by (a) the profile's declared `FidelityTier` stamp and (b) `resolve()`'s existing
file precedence, not by two separate providers. Collapsing them avoids inventing a
second profile lookup that does not exist.

**Alternatives**: (a) providers self-order by returning a precedence at query time
that the resolver trusts in iteration order; (b) the registration order is the
precedence. Rejected: both make the order incidental rather than imposed, which is
exactly the failure the permutation test exists to catch (mirrors the
`fragcap-attr/src/index.rs` MatchRank discipline).

## D-4: The precedence-never-inverts-fidelity invariant, stated testably

**Decision**: The invariant is that a higher-precedence provider's produced answer
never carries a strictly lower fidelity than a lower-precedence provider's produced
answer. For this slice's two live providers it reduces to a clean, testable fact:
the `ObservationProvider` is the unique lowest precedence and the unique producer
of `observed`, and a `ProfileProvider` answer (authored, verified, or
heuristic-unverified) always ranks strictly above `observed`. Equal-fidelity,
different-precedence pairs are allowed (a locally scaffolded heuristic-unverified
profile outranking a future shipped hint of the same tier is correct: precedence
breaks the tie). The design keeps each provider's `Precedence` consistent with the
ceiling of the fidelity it can stamp, and a unit test asserts the observed-is-
lowest fact directly.

## D-5: Target shape and how it reuses matching

**Decision**: `Target` carries `fidelity: FidelityTier`, `provenance: Provenance`,
and `origin: TargetOrigin`, where `TargetOrigin` is either `Profile(Profile)` (a
profile-backed answer) or `Observed(ObservedTarget)` (a live process that matched
an identity). The identity a provider matches on is the existing match predicates
(`MatchPredicates`): an exe image-name glob plus optional path anchors. The
`ObservationProvider` reuses matching by a new public function
`matching::first_live_match(preds: &MatchPredicates, tree: &ProcessTree) ->
Option<NodeId>`, which scans live nodes and returns the first where the predicates
hold, refactoring the existing private `predicates_hold` so the P-9
command-line-unavailable rule stays in one place.

**Alternatives**: (a) a bespoke identity vocabulary for observation; (b) build a
synthetic one-stage `Profile` for the observation match. Rejected: (a) duplicates
the match predicates the whole project already speaks; (b) is a heavier, less
direct reuse than exposing the predicate evaluator. Exposing `first_live_match`
keeps one matcher and one honesty rule.

## D-6: The not-resolved outcome and CLI behavior preservation

**Decision**: `TargetResolver::resolve(&ResolutionRequest) -> Result<Target,
Unresolved>`. A provider's query returns `Result<Option<Target>, ProviderError>`:
`Ok(Some)` wins, `Ok(None)` continues the cascade, `Err` aborts with a hard error.
The `ProfileProvider` maps `resolve()`'s outcomes so behavior is preserved:

- found and loaded -> `Ok(Some(profile-backed Target))`
- `ResolveError::NotFound` -> `Ok(None)` (nothing matched; cascade continues),
  and the `NotFound` (with its searched paths) is attached to the eventual
  `Unresolved` so the CLI prints the same message it prints today
- `ResolveError::Load` or `ResolveError::InvalidReference` -> `Err(ProviderError)`
  (a present-but-broken profile, or an unusable reference, is a hard failure, not
  a silent skip -- this preserves today's exit-code behavior)

The `run` command calls the resolver, and maps `Target` -> proceed with the
profile, `Unresolved{ carrying the profile NotFound }` -> the existing
"not found, searched ..." `CliError`, and `ProviderError` -> the existing Load /
InvalidReference `CliError`. The existing CLI tests and the corpus goldens hold
the output byte-identical (SC-008).

**Alternatives**: leave `run` calling `resolve()` and only exercise the resolver in
tests. Rejected: the spec's assumption is that the run profile path flows through
the resolver; a fudge that keeps two front doors would leave the integration
untested and the two paths free to drift.

## D-7: Provenance, and what a profile-backed answer's provenance is

**Decision**: `Provenance { source: String, seeded_at: Option<String> }`, matching
the schema's `$defs/provenance`. A `Profile` retains its declared provenance as
`Option<Provenance>` (only hint/export require provenance in the schema, so a
profile may omit it). A profile-backed `Target`'s provenance is the profile's
declared provenance when present, else a synthesized one whose source names the
`ProfileSource` (for example `user-profile`, `bundled`). An observation answer's
provenance source is `runtime-observation`.

## D-8: Kind, and which kinds load as a capture profile

**Decision**: `Kind { Profile, Package, Hint, Export }` in `schema.rs`, parsed from
the top-level `kind`. `Profile::parse` continues to accept `profile` and (newly
explicit) `package` -- an authored target package is the highest-precedence
authored artifact and is loadable as a capture profile -- and continues to refuse
`hint` and `export` (unchanged behavior; parse.rs already refuses those two). The
in-memory `Profile` exposes `kind()`.

**Alternatives**: accept only `profile`. Rejected: the issue names the user-authored
package as the top precedence position, and a package is structurally a profile
with `fidelity: authored`; refusing it would leave the top of the cascade
unreachable. parse.rs already falls through for `package` today, so this only
makes the acceptance explicit and readable via `kind()`.

## Constitution re-check after design

Re-running the P-1..P-9 gate against the concrete types above: no new dependency,
no core edge, no process handle, every not-resolved path named, every answer
stamped, the two fidelity axes distinct. No violations introduced by the design.
