# Contract: Platform-Walker Provider and Helpers

The public contract this slice adds. Signatures are the intended shape; exact Rust
is settled in implementation, but these are binding.

## Cascade participation (`fragcap-steam`)

```text
pub struct SteamWalkerProvider;
impl SteamWalkerProvider { pub fn new() -> SteamWalkerProvider; }

impl fragcap_profile::TargetProvider for SteamWalkerProvider {
    fn precedence(&self) -> Precedence { Precedence::PlatformWalker }
    fn provide(&self, request, notes) -> Result<Option<Target>, ProviderError>;
}
```

`provide` contract:

- `request.install_root()` absent: return `Ok(None)`.
- `client_for` returns `NoMatch` (no plausible client): return `Ok(None)`.
- `client_for` returns `Ambiguous { candidates }`: record
  `notes.note_walker_ambiguous(candidates)` and return `Ok(None)`.
- `client_for` returns `Unreadable { path }`: record
  `notes.note_walker_unreadable(path)` and return `Ok(None)`.
- `client_for` returns `Resolved(walker_target)`: return `Ok(Some(target))` where
  - `target.fidelity() == FidelityTier::HeuristicUnverified`
  - `target.provenance().source() == "steam-library"`
  - `target.origin()` is `TargetOrigin::PlatformWalker(WalkerTarget { .. })`
- Never returns `Err`: every non-answer is a decline, so the cascade always
  continues to runtime observation.

## Enumeration helper (`fragcap-steam`)

```text
pub fn install_root_for(app_id: &str) -> Result<Option<PathBuf>, SteamError>;
pub fn install_root_in(root: &Path, app_id: &str) -> Result<Option<PathBuf>, SteamError>;
```

Returns the install directory of the installed title with `app_id`, or `None` if
it is not installed. `install_root_for` locates Steam (registry) then delegates to
`install_root_in`, which is portable and used in tests. The caller enriches a
`ResolutionRequest` with the returned directory via `with_install_root`.

## New profile-crate surface (`fragcap-profile`)

```text
pub struct WalkerTarget { /* platform, image_name, image_path, identity */ }
impl WalkerTarget {
    pub fn new(platform: String, image_name: String, image_path: String,
               identity: MatchPredicates) -> WalkerTarget;
    pub fn platform(&self) -> &str;
    pub fn image_name(&self) -> &str;
    pub fn image_path(&self) -> &str;
    pub fn identity(&self) -> &MatchPredicates;
}

enum TargetOrigin { Profile(..), EngineRule(..), PlatformWalker(WalkerTarget), Observed(..) }

// ResolutionNotes / Unresolved
fn note_walker_ambiguous(&mut self, candidates: usize);
fn note_walker_unreadable(&mut self, path: PathBuf);
fn walker_ambiguous(&self) -> Option<WalkerAmbiguity>;   // on Unresolved
fn walker_unreadable(&self) -> Option<&Path>;            // on Unresolved
```

The `PlatformWalkerProvider` stub is removed from `fragcap-profile`; `run.rs` and
`watch.rs` construct `fragcap::steam::SteamWalkerProvider` in its place.

## `client_for` contract (internal, `fragcap-steam`)

```text
enum ClientResolution { Resolved(WalkerTarget), NoMatch,
                        Ambiguous { candidates: usize }, Unreadable { path: PathBuf } }
fn client_for(install_dir: &Path) -> ClientResolution;
```

- Scans the install directory (Err reading it -> `Unreadable`), drops non-game
  executables (installers, redistributables, helpers, hash-named installers) and
  launcher stubs using the shared scaffold predicates.
- Exactly one plausible client remains -> `Resolved`, with `identity` carrying at
  least `exe = <client file name>`.
- Zero remain -> `NoMatch`. More than one remain -> `Ambiguous { candidates }`.
- Deterministic: the scan is path-sorted, so the outcome does not depend on
  directory iteration order.

## Invariants asserted by tests

- Composition: a Steam library whose title is an Unreal install resolves to the
  shipping exe via the engine rule (which outranks the walker) when the request
  carries the install directory.
- Direct answer: a Steam library whose title is a single-client non-engine install
  resolves via the walker at `heuristic-unverified` with provenance
  `steam-library`.
- Degradation: a not-installed title, an ambiguous install (several clients), and
  an unreadable install each make the walker decline, and the cascade resolves via
  runtime observation when a matching process is present.
- Precedence: an authored profile for the same title outranks both the engine rule
  and the walker.
- Dependency direction: `cargo xtask deps` stays green (walker in `fragcap-steam`;
  `fragcap-profile` gains no dependency on `fragcap-steam`).
- P-1: the walker names no process-handle API; `cargo xtask lint` stays green.
