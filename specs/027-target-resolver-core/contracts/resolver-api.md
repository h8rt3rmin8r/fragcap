# Contract: Target Resolver Public API

The public surface `fragcap-profile` re-exports from `lib.rs` after this slice.
Each item lists the behavioral contract the tests bind.

## Re-exports (lib.rs)

```rust
pub use schema::{FidelityTier, Provenance, Kind};      // metadata types
pub use target::{Target, TargetOrigin, ObservedTarget};
pub use resolver::{
    TargetResolver, TargetProvider, Precedence, ResolutionRequest,
    Unresolved, ProviderError,
};
pub use providers::{
    ProfileProvider, ObservationProvider,
    HintProvider, EngineRuleProvider, PlatformWalkerProvider,
};
pub use matching::first_live_match;
```

## Contracts

### FidelityTier
- `parse("authored")` -> `Some(Authored)`, and likewise for the other three names;
  `parse("Authored")` and any non-name -> `None`.
- `as_str()` round-trips: `parse(t.as_str()) == Some(t)` for every variant.
- Ordering: `Authored > Verified > HeuristicUnverified > Observed`.

### Profile (extended)
- After `Profile::parse` on a valid profile, `kind()`, `fidelity()`,
  `provenance()`, and `notes()` return what the document declared.
- A profile that omits `provenance` yields `provenance() == None` (it is not
  required for the `profile` kind).
- `kind: "package"` loads and `kind()` returns `Package`; `kind: "hint"` and
  `kind: "export"` are still refused by `parse` (a diagnostic, not a `Profile`).
- Every previously passing profile parse still passes with identical `game()`,
  `capture()`, and `stages()`.

### TargetProvider / TargetResolver
- `TargetResolver::new(providers)` sorts by `precedence()`; `resolve` queries
  highest precedence first.
- First `Ok(Some(target))` in precedence order is returned; later providers are
  not consulted.
- A provider returning `Err(ProviderError)` aborts resolution with that error;
  lower-precedence providers are not consulted.
- If every provider returns `Ok(None)`, `resolve` returns `Err(Unresolved)`.
- The result is identical for every permutation of the `providers` vec passed to
  `new` (the permutation test).

### ProfileProvider
- `provide` with a request whose `reference` resolves -> `Ok(Some(target))` where
  `target.profile()` is `Some`, `target.fidelity() == profile.fidelity()`, and
  `target.provenance()` is the profile's or a synthesized `ProfileSource`-named
  one.
- `reference` that matches nothing -> `Ok(None)`, and the resolver's `Unresolved`
  carries the `NotFound` with its searched paths.
- `reference` naming a present but invalid profile -> `Err(ProviderError::Profile(
  ResolveError::Load { .. }))`.
- `reference` that is neither a file nor a valid slug -> `Err(ProviderError::
  Profile(ResolveError::InvalidReference { .. }))`.

### ObservationProvider
- With `identity` and a `tree` containing a live node whose predicates hold ->
  `Ok(Some(target))` where `target.fidelity() == Observed`,
  `target.origin()` is `Observed(ObservedTarget)` with the node's pid, image name,
  and image path, and `target.provenance().source() == "runtime-observation"`.
- With no matching live node, or with `identity`/`tree` absent -> `Ok(None)`.
- Never opens a process handle; reads only `image_name()`/`image()` from the tree.

### Stub providers (Hint / EngineRule / PlatformWalker)
- `precedence()` returns `HintDatabase` / `EngineRule` / `PlatformWalker`
  respectively.
- `provide` returns `Ok(None)` for every request in this slice.

### first_live_match
- Returns the first live node (creation order) where every predicate in `preds`
  holds; `None` otherwise.
- An `Unavailable` command line never satisfies `cmdline_contains` (P-9),
  identical to `bind_stages`.

## Cross-cutting assertions (xtask gates)
- `cargo xtask deps`: fragcap-profile gains no new edge; fragcap-core unchanged.
- `cargo xtask lint`: no `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`
  token appears in the new code.
- `cargo xtask license`: no new crate; per-crate license text unchanged.
- `scripts/lint-docs.sh`: the four new glossary entries resolve and the index
  reproduces.
