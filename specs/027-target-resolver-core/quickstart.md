# Quickstart: Exercising the Target Resolver

This walks the two live providers and the precedence engine, all offline, with no
capture driver, no elevation, and no game (the section 25.1 property this slice
preserves).

## 1. A profile-backed resolution

```rust
use fragcap_profile::{
    TargetResolver, ProfileProvider, ObservationProvider, HintProvider,
    EngineRuleProvider, PlatformWalkerProvider, ResolutionRequest,
    SearchPath, BundledSet, FidelityTier,
};

let resolver = TargetResolver::new(vec![
    Box::new(ProfileProvider::new()),
    Box::new(HintProvider::new()),
    Box::new(EngineRuleProvider::new()),
    Box::new(PlatformWalkerProvider::new()),
    Box::new(ObservationProvider::new()),
]);

let search = SearchPath { command_line: vec!["profiles".into()], user: None };
let bundled = BundledSet::empty();
let req = ResolutionRequest::for_reference("eso", &search, &bundled);

let target = resolver.resolve(&req).expect("resolves");
assert!(target.profile().is_some());
assert_eq!(target.fidelity(), FidelityTier::Verified); // whatever the file declared
```

## 2. Fall back to runtime observation

Build a process tree from a scripted event stream (as `matching.rs` tests do),
supply an identity, and confirm the observation provider answers when no
higher-precedence provider can.

```rust
use fragcap_profile::MatchPredicates; // built via a one-stage profile parse in tests
// tree: a ProcessTree with a live "eso64.exe" node
let identity = /* MatchPredicates with exe = "eso64.exe" */;
let req = ResolutionRequest::for_observation(&identity, &tree); // no profile reference
let target = resolver.resolve(&req).expect("observes");
assert_eq!(target.fidelity(), FidelityTier::Observed);
assert_eq!(target.provenance().source(), "runtime-observation");
```

## 3. Precedence is total (the permutation test)

```rust
// Two providers that both answer for the same input. For every permutation of the
// providers vec, resolve returns the higher-precedence answer.
for perm in permutations_of(providers) {
    let r = TargetResolver::new(perm);
    assert_eq!(r.resolve(&req).unwrap().fidelity(), expected_high);
}
```

## 4. Not resolved is named, not silent

```rust
let req = ResolutionRequest::for_reference("no-such-game", &empty_search, &empty_bundled);
match resolver.resolve(&req) {
    Err(Unresolved { .. }) => { /* the CLI renders the same "searched ..." message */ }
    Ok(_) => panic!("must not resolve"),
}
```

## 5. Verify

```bash
cargo xtask ci
```

Then the two that need a target/toolchain the runner may lack:

```bash
cargo xtask msrv
cargo xtask neutral
```

Both exit 2 (not 0) when their toolchain/target is absent; that is a skip to
report, not a pass.
