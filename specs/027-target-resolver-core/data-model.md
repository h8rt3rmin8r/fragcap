# Phase 1 Data Model: Target Resolution Cascade -- Resolver Core

All types live in `fragcap-profile`. Signatures are indicative; the implement
phase settles exact derives and visibility. Nothing here adds a dependency or a
core edge.

## Shared metadata types (in `schema.rs`)

### FidelityTier

The targeting trust tier. Separate from the attribution `Fidelity`
(`fragcap-core::attribution`). Declared in ascending trust order so `Ord` makes
"more trusted" the greater value (D-2).

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FidelityTier { Observed, HeuristicUnverified, Verified, Authored }

impl FidelityTier {
    pub const ACCEPTED: &'static [&'static str] =
        &["authored", "verified", "heuristic-unverified", "observed"];
    pub fn parse(s: &str) -> Option<FidelityTier>; // maps the four schema names
    pub fn as_str(&self) -> &'static str;          // the schema spelling
}
```

Invariant: `Authored > Verified > HeuristicUnverified > Observed`.

### Provenance

Matches the schema `$defs/provenance`.

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance { source: String, seeded_at: Option<String> }

impl Provenance {
    pub fn source(&self) -> &str;
    pub fn seeded_at(&self) -> Option<&str>;
}
```

### Kind

The artifact discriminator. `Profile::parse` loads `Profile` and `Package`;
`Hint` and `Export` are refused as capture profiles (unchanged behavior).

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { Profile, Package, Hint, Export }
```

### Profile (extended)

`Profile` gains the metadata it previously discarded. Constructor stays
`pub(crate)`; `Profile::parse` remains the only way in.

```rust
pub struct Profile {
    game: Game,
    capture: CaptureDefaults,
    stages: Vec<Stage>,
    kind: Kind,                        // NEW
    fidelity: FidelityTier,            // NEW
    provenance: Option<Provenance>,    // NEW
    notes: Option<String>,             // NEW
}

impl Profile {
    // existing: schema(), game(), capture(), stages(), terminal_stage(), stage()
    pub fn kind(&self) -> Kind;                     // NEW
    pub fn fidelity(&self) -> FidelityTier;         // NEW
    pub fn provenance(&self) -> Option<&Provenance>;// NEW
    pub fn notes(&self) -> Option<&str>;            // NEW
}
```

`Profile::ACCEPTED` extends to include `"kind"`, `"fidelity"`, `"provenance"`,
and `"notes"` (the schema already requires/permits them; this keeps the
unknown-key diagnostic list honest).

## The Target (in `target.rs`)

### Target

The resolved answer handed onward.

```rust
pub struct Target {
    fidelity: FidelityTier,
    provenance: Provenance,
    origin: TargetOrigin,
}

impl Target {
    pub fn fidelity(&self) -> FidelityTier;
    pub fn provenance(&self) -> &Provenance;
    pub fn origin(&self) -> &TargetOrigin;
    pub fn profile(&self) -> Option<&Profile>; // Some for a profile-backed target
}
```

### TargetOrigin

```rust
pub enum TargetOrigin {
    /// Backed by a validated profile (from the ProfileProvider).
    Profile(Profile),
    /// Derived from a live process that matched an identity (ObservationProvider).
    Observed(ObservedTarget),
}
```

### ObservedTarget

The minimal record of a matched live process. Uses only what the process
snapshot already holds (P-1): no handle, no memory read.

```rust
pub struct ObservedTarget {
    pid: u32,
    image_name: String,   // file name (from ProcessNode::image_name)
    image_path: String,   // full path  (from ProcessNode::image)
}
```

## The resolver (in `resolver.rs`)

### Precedence

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    // Declared highest-first; the resolver sorts providers by this.
    Profile,
    HintDatabase,
    EngineRule,
    PlatformWalker,
    RuntimeObservation,
}
```

### ResolutionRequest

What providers may read. Each provider takes what it needs; absent inputs simply
mean the providers that need them return no answer.

```rust
pub struct ResolutionRequest<'a> {
    reference: Option<&'a str>,        // CLI --profile reference (ProfileProvider)
    search: &'a SearchPath,
    bundled: &'a BundledSet,
    identity: Option<&'a MatchPredicates>, // identity to observe (ObservationProvider)
    tree: Option<&'a ProcessTree>,         // the observed process tree
}
```

### TargetProvider

```rust
pub trait TargetProvider {
    fn precedence(&self) -> Precedence;
    fn provide(&self, req: &ResolutionRequest) -> Result<Option<Target>, ProviderError>;
}
```

### TargetResolver

```rust
pub struct TargetResolver { providers: Vec<Box<dyn TargetProvider>> }

impl TargetResolver {
    pub fn new(providers: Vec<Box<dyn TargetProvider>>) -> Self; // sorts by precedence
    pub fn resolve(&self, req: &ResolutionRequest) -> Result<Target, Unresolved>;
}
```

`resolve` queries providers highest precedence first: the first `Ok(Some)` wins;
an `Err` aborts; if all return `Ok(None)`, it returns `Unresolved`.

### Unresolved and ProviderError

```rust
pub enum ProviderError {
    /// A candidate was found and could not be used (broken profile), or the
    /// reference was unusable. Wraps the underlying ResolveError.
    Profile(ResolveError),
}

pub struct Unresolved {
    /// The profile provider's NotFound, when the profile path was attempted and
    /// nothing matched, so the CLI can print the same "searched ..." message.
    profile_not_found: Option<ResolveError>,
}
```

## The providers (in `providers.rs`)

- **ProfileProvider** (`Precedence::Profile`): wraps `resolve(reference, search,
  bundled)`. Maps found -> `Ok(Some(Target { origin: Profile, fidelity:
  profile.fidelity(), provenance: profile provenance or synthesized from
  ProfileSource }))`; `NotFound` -> `Ok(None)`; `Load`/`InvalidReference` ->
  `Err(ProviderError::Profile(..))`.
- **ObservationProvider** (`Precedence::RuntimeObservation`): needs
  `req.identity` and `req.tree`; returns `Ok(Some(Target { origin: Observed,
  fidelity: Observed, provenance: source "runtime-observation" }))` for the first
  live node where the identity predicates hold (via `matching::first_live_match`),
  else `Ok(None)`.
- **HintProvider, EngineRuleProvider, PlatformWalkerProvider**: registered at
  their precedence positions; `provide` returns `Ok(None)` in this slice. Their
  data arrives in #78, S029, S030 without touching the resolver.

## Matching reuse (in `matching.rs`)

```rust
/// The first live node whose predicates all hold, in creation order.
pub fn first_live_match(preds: &MatchPredicates, tree: &ProcessTree) -> Option<NodeId>;
```

Refactors the existing private `predicates_hold` into a callable form so the P-9
command-line-unavailable rule stays in one place.

## Relationships and invariants

- A `Target` has exactly one `FidelityTier` and one `Provenance` (FR-002).
- `Authored > Verified > HeuristicUnverified > Observed` (FR-003).
- `ObservationProvider` is the unique `RuntimeObservation` precedence and the
  unique producer of `Observed` (D-4).
- `TargetResolver::resolve` is a pure function of the request and the sorted
  provider set; provider registration order does not affect it (FR-004).
- Nothing here references `fragcap-core::attribution::Fidelity` (FR-010).
