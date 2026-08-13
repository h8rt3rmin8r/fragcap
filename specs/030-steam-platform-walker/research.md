# Phase 0 Research: Steam Platform-Walker Refactor

All unknowns resolved. Each decision is grounded in the S027-S029 code, the
`fragcap-steam` code (`library.rs`, `scaffold.rs`, `launch.rs`), the constitution,
the deps gate, and the library research recorded in project memory.

## D1. Where the walker provider lives

- **Decision**: The `SteamWalkerProvider` lives in `fragcap-steam`, implementing
  `fragcap_profile::TargetProvider`. The no-op `PlatformWalkerProvider` stub in
  `fragcap-profile` is retired.
- **Rationale**: The provider needs Steam knowledge (enumeration, the scaffold
  classifier). `xtask/src/deps.rs` allows `fragcap-steam -> fragcap-profile` and
  forbids the reverse (it is absent from `EXPECTED`, so it flags as an unexpected
  edge). So the provider cannot live in `fragcap-profile`. `fragcap-steam` already
  depends on `fragcap-profile`, and `TargetProvider`, `Target`, `TargetOrigin`,
  `Provenance`, and `MatchPredicates` are all public and externally constructible.
- **Alternatives considered**: Keep the provider in `fragcap-profile` and pass
  Steam data in. Rejected: `fragcap-profile` cannot depend on `fragcap-steam`, and
  the classifier lives in `fragcap-steam`; inverting the edge to reach it is
  exactly what the deps gate exists to prevent.

## D2. How a walker answer is carried

- **Decision**: Add `TargetOrigin::PlatformWalker(WalkerTarget)` to
  `fragcap-profile`. `WalkerTarget` carries the platform (`"steam"`), the resolved
  client's image name and full path, and the `MatchPredicates` the pipeline binds
  it by.
- **Rationale**: A walker answer is neither a profile, an engine-rule answer, nor
  a live observation; reusing any of those origins would misrepresent it. The
  `TargetOrigin` doc already anticipates further origins. Carrying
  `MatchPredicates` mirrors `ObservedTarget` and `EngineRuleTarget` so watch mode
  can bind the process once it appears.
- **Alternatives considered**: Reuse `TargetOrigin::EngineRule`. Rejected: a
  storefront library walk is not an engine layout rule, and the provenance and the
  origin should agree about what produced the answer.

## D3. The walker declines rather than guess a client

- **Decision**: `client_for(install_dir)` resolves only when, after dropping
  non-game executables and launcher stubs (the existing scaffold predicates),
  exactly one plausible client executable remains. Zero remaining is `None`
  (only launchers, or nothing statically); more than one remaining is `Ambiguous`.
  It does not pick the largest among several.
- **Rationale**: The scaffold picks the largest non-launcher as the client, which
  is correct for a human-reviewed skeleton the author then corrects. For automatic
  capture the walker must not present a guess as the answer (P-9). The library
  research (project memory) found size-based client selection to be coincidental
  rather than a real signal, and runtime observation resolves the ambiguous and
  launcher-mediated cases (ESO, Division 2) correctly from the live socket-holding
  process. So the walker declines and degrades to observation rather than guess.
- **Alternatives considered**: (a) Largest non-launcher wins, as the scaffold
  does. Rejected: dishonest for auto-capture and unreliable per the research. (b)
  Name-similarity to the install-directory name as a tiebreak. Held in reserve as
  a follow-up refinement; it adds a fuzzy heuristic (spaces, version suffixes,
  `64`) whose failure modes need their own study, and declining is the safe,
  honest floor for this slice.

## D4. Honest provenance

- **Decision**: A walker answer's provenance source is `steam-library`. It is not
  `steam-appinfo`.
- **Rationale**: The walker resolves from the library manifests and by classifying
  install-directory files. It does not read Steam's application info (the
  `config.launch` array). Stamping `steam-appinfo` would claim a source it did not
  consult, which P-9 forbids. `provenance.source` is a free-form string, so an
  honest label is available; `steam-appinfo` remains reserved for a future slice
  that actually reads appinfo.
- **Alternatives considered**: Use `steam-appinfo` per the slice plan's original
  wording. Rejected on P-9 grounds; recorded as a deliberate deviation from the
  plan's string.

## D5. Appinfo/PICS reading is deferred

- **Decision**: This slice does not read Steam application info (networked PICS or
  the local binary `appinfo.vdf` cache). It wires the existing library walk and
  install-directory classification into the cascade.
- **Rationale**: The launch executable lives only in PICS appinfo (networked) or
  the binary `appinfo.vdf` cache. `fragcap-steam` reads only text VDF today and has
  no binary-VDF parser, no `steamcmd`, and no network. Reading appinfo means either
  a heavy networked Steam-client dependency (a large transitive graph and network
  I/O in a passive local tool, P-1-adjacent) or a versioned binary-format parser
  (real work, and the local cache only covers apps the client has already seen).
  The S029 engine rule plus the install-directory classifier already name the
  client for the common cases; the hard launcher-mediated titles degrade to
  runtime observation regardless. The full launch-array model and the
  launcher-mediated flag belong with the hint-database (#78) revision.
- **Alternatives considered**: Add a networked Steam-client dependency; write a
  binary `appinfo.vdf` parser. Both rejected for this slice on dependency-weight
  and effort grounds, consistent with the project's `boon`/`crossbeam` rejections;
  recorded as a follow-up direction.

## D6. How the install directory reaches the cascade

- **Decision**: The install directory is placed on the request as `install_root`
  before the cascade runs (the CLI/assembler enriches the request via
  `ResolutionRequest::with_install_root`), not from inside the walker's
  `provide()`. The walker provider reads `install_root` like the engine rule does.
  A `fragcap-steam` helper, `install_root_for(app_id)` (discover then find),
  produces the directory.
- **Rationale**: A provider receives an immutable `&ResolutionRequest` and cannot
  add `install_root` for a provider consulted before it. Since the engine rule
  (higher precedence) must see `install_root`, the enrichment has to happen at
  request construction. This is exactly the seam S029 built `install_root` and
  `with_install_root` for.
- **Alternatives considered**: The walker enumerates Steam inside `provide()`.
  Rejected: it runs after the engine rule, so the engine rule would never get the
  install directory, breaking the composition the cascade is designed around.

## D7. Reuse the scaffold classifier predicates

- **Decision**: `walker.rs` reuses `scan`, `is_non_game`, and `is_launcher` from
  `scaffold.rs` (sharing them within the crate) rather than duplicating the token
  lists or the scan.
- **Rationale**: The walker and the human-reviewed scaffold must agree on what an
  installer or a launcher is; two copies would drift. Only the final decision
  differs (the scaffold picks a client; the walker declines when it cannot single
  one out), so only that decision is new code.
- **Alternatives considered**: Duplicate the predicates in `walker.rs`. Rejected:
  drift risk for no benefit.

## D8. A filesystem error is not a no-match

- **Decision**: `client_for` returns an `Unreadable` outcome carrying the path
  when the install directory cannot be read, and the provider records a
  walker-unreadable note surfaced through `Unresolved`, declining rather than
  reporting a clean no-match. This mirrors the engine-rule handling S029 landed.
- **Rationale**: The same reasoning the S029 review established: an unreadable
  directory is not an absent layout, and an incomplete scan must not masquerade as
  a confident no-match (P-4). Consistency across the two install-directory
  providers is worth the small addition.
- **Alternatives considered**: Swallow read errors into `None`. Rejected for the
  reasons the S029 review already settled.

## D9. Production activation is deferred (and surfaced)

- **Decision**: The real walker replaces the stub in the production resolver vec
  (`run.rs`, `watch.rs`), and the enumeration-to-`install_root` helper is built and
  tested, but this slice does not add the non-profile capture path that would let
  the walker fire a capture. That path is surfaced at the pre-push halt.
- **Rationale**: `run` errors today on a resolved target with no profile
  ("run cannot capture yet"), and its module doc names the non-profile capture
  path as "a later slice." A profile outranks the walker, so the walker only
  matters for a no-profile capture, which needs that path. Building it is a
  cross-cutting integration (target-identity to capture config, degradation to
  watch semantics) that S027 through S029 all deferred; folding it into this slice
  would balloon it and overlap S028. The walker, its composition, and its
  degradation are proven end to end through the resolver in tests, which is the
  slice's stated done-condition.
- **Alternatives considered**: Build the non-profile capture path here. Held as
  the immediate follow-up; offered to the operator at the halt rather than assumed.

## D10. Fixtures

- **Decision**: Tests build a fake Steam library (a `libraryfolders.vdf`, an
  `appmanifest_*.acf`, and an install directory) composed with the engine-rule
  install-layout fixtures, using temporary directory trees, in the spirit of the
  existing `fragcap-steam` `TempTree` and the S029 `UnrealTree`.
- **Rationale**: The walker's behavior is a function of on-disk library and
  install-directory shape; building the shape in a temp tree is the direct test,
  and composing the Steam library with an Unreal install dir proves the walker plus
  engine-rule composition without a real Steam installation.
- **Alternatives considered**: A real Steam install. Rejected: not reproducible in
  CI and unnecessary.
