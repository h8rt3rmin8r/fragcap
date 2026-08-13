# Phase 1 Data Model: Steam Platform-Walker Refactor

The entities this slice adds or changes, split by crate.

## New types in `fragcap-profile`

### `WalkerTarget` (struct, `target.rs`)

What a platform-walker answer carries. Analogous to `EngineRuleTarget`, but naming
the storefront and a client resolved from a library install directory.

- `platform: String` -- the storefront, `"steam"` in this slice.
- `image_name: String` -- the resolved client's file name.
- `image_path: String` -- the resolved client's full path on disk.
- `identity: MatchPredicates` -- the match rules the pipeline binds by (at least
  the `exe`).
- Derives: `Clone, Debug, PartialEq, Eq`.
- Accessors: `platform()`, `image_name()`, `image_path()`, `identity()`.
- Constructor: `WalkerTarget::new(platform, image_name, image_path, identity)`
  (public, so `fragcap-steam` builds it).

### `TargetOrigin::PlatformWalker(WalkerTarget)` (enum variant, `target.rs`)

- Added alongside `Profile`, `EngineRule`, `Observed`.
- `Target::profile()` and `Target::into_profile()` return `None` for it.
- Any other exhaustive `match` on `TargetOrigin` (in the CLI) gains the arm.

### Resolution notes (`resolver.rs`)

- `ResolutionNotes` gains `walker_ambiguous: Option<WalkerAmbiguity>` and
  `walker_unreadable: Option<PathBuf>`, with recorders
  `note_walker_ambiguous(candidates)` and `note_walker_unreadable(path)` (first
  path wins), mirroring the engine-rule notes.
- `WalkerAmbiguity` is a small record `{ candidates: usize }` (the count of
  plausible clients that could not be reduced to one).
- `Unresolved` carries both through, exposed via `walker_ambiguous()` and
  `walker_unreadable()`, alongside the existing profile and engine-rule notes; the
  `Unresolved` display mentions the unreadable path when nothing else resolved.

## New types in `fragcap-steam`

### `ClientResolution` (module-internal enum, `walker.rs`)

The result of classifying an install directory. Internal; the provider maps it to
the cascade contract.

- `Resolved(WalkerTarget)` -- exactly one plausible client remained.
- `NoMatch` -- no plausible client (only launchers, or nothing).
- `Ambiguous { candidates: usize }` -- more than one plausible client remained.
- `Unreadable { path: PathBuf }` -- the install directory could not be read.

### `SteamWalkerProvider` (struct, `walker.rs`)

- `impl fragcap_profile::TargetProvider`:
  - `precedence()` returns `Precedence::PlatformWalker`.
  - `provide()` reads `request.install_root()`; declines `Ok(None)` when absent;
    calls `client_for`; maps `Resolved` to `Ok(Some(Target::new(HeuristicUnverified,
    Provenance::new("steam-library", None), TargetOrigin::PlatformWalker(t))))`,
    `NoMatch` to `Ok(None)`, `Ambiguous` to recording the note then `Ok(None)`,
    `Unreadable` to recording the note then `Ok(None)`. Never returns `Err`.

### `install_root_for` (function, `library.rs` or `lib.rs`)

- `install_root_for(app_id: &str) -> Result<Option<PathBuf>, SteamError>`:
  discovers the Steam installation and returns the matching title's install
  directory, or `None` if the title is not installed. The CLI/future capture path
  uses it to enrich the request with `install_root`. Portable variant
  `install_root_in(root, app_id)` over a given Steam root for tests.

## Shared within `fragcap-steam`

- `scan(dir) -> Result<Vec<ExecutableImage>, SteamError>`, `is_non_game(name)`,
  and `is_launcher(image)` in `scaffold.rs` become reachable from `walker.rs`
  (made `pub(crate)`), so the walker and the scaffold classify with the same
  predicates. `client_for` is the walker's own decision over the shared scan.

## `client_for` decision (walker.rs)

```text
client_for(install_dir):
  images = scan(install_dir)            // Err -> Unreadable{install_dir}
  candidates = images.filter(!is_non_game)   // fall back to images if that empties
  clients = candidates.filter(!is_launcher)
  match clients.len():
    0 -> NoMatch                        // only launchers / nothing on disk
    1 -> Resolved(WalkerTarget{ "steam", name, path, identity(exe=name) })
    _ -> Ambiguous{ candidates: clients.len() }   // several plausible clients
```

Determinism: `scan` already sorts by path, so the single-client and count
decisions do not depend on directory iteration order.

## Fidelity and provenance (fixed values)

- Fidelity: every walker answer is `FidelityTier::HeuristicUnverified`, never
  higher.
- Provenance: `Provenance::new("steam-library".to_string(), None)`.

## Relationships

```text
CLI (run/watch) assembles TargetResolver with:
  [ProfileProvider, HintProvider, EngineRuleProvider,
   SteamWalkerProvider (from fragcap-steam), ObservationProvider]

Steam enumeration (install_root_for) --> request.with_install_root(dir)
  EngineRule (higher) reads install_root -> resolves engine layout titles
  SteamWalkerProvider (lower) reads install_root -> client_for:
     Resolved  -> Target{HeuristicUnverified, "steam-library", PlatformWalker(..)}
     NoMatch   -> Ok(None)  (degrade to observation)
     Ambiguous -> note; Ok(None)  (degrade to observation)
     Unreadable-> note; Ok(None)  (degrade to observation)
```
