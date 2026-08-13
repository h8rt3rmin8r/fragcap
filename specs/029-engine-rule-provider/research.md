# Phase 0 Research: Engine-Rule Provider

All unknowns from the Technical Context are resolved below. No `NEEDS
CLARIFICATION` remains. Each decision is grounded in the S027 code
(`crates/fragcap-profile/src/{resolver,providers,target,schema}.rs`), the
constitution, and the slice scope.

## D1. Where the provider lives

- **Decision**: A new `engine_rule` module inside `fragcap-profile`, with the
  existing `EngineRuleProvider` in `providers.rs` delegating to it.
- **Rationale**: `fragcap-profile` already owns the cascade (`resolver.rs`,
  `providers.rs`, `target.rs`), profile resolution (`resolve.rs`), and matching
  (`matching.rs`). Its only permitted workspace dependency edge is
  `fragcap-profile -> fragcap-core` (`xtask/src/deps.rs` `EXPECTED`). The
  provider is pure logic over already-available inputs, so the crate is the
  right home and `cargo xtask deps` stays green untouched.
- **Alternatives considered**: A new `fragcap-engine` crate. Rejected: it adds a
  workspace edge (`fragcap-profile -> fragcap-engine`) requiring an `EXPECTED`
  edit and a `SIBLINGS` consideration, for logic that needs nothing the profile
  crate lacks. The project's default is to add nothing; this honors it.

## D2. Provider input shape

- **Decision**: Add `install_root: Option<&Path>` to `ResolutionRequest`, with a
  `for_install(install_root, search, bundled)` constructor and an
  `install_root()` accessor. The provider declines when it is `None`. The two
  existing constructors set the field to `None`.
- **Rationale**: An engine rule reasons about a directory tree, not a process. A
  directory input is the most general and is exactly what the S030 platform
  walker will produce (it discovers an install location per title). Modelling the
  input as a directory means the walker composes with this provider without
  changing it (FR-007). The "provider takes only the inputs it needs, and
  declines when they are absent" pattern is already established by the profile
  and observation providers (`resolver.rs:62-74`).
- **Alternatives considered**: (a) A stub executable path. Rejected as the
  primary input: it is one specialization of "a directory to inspect" (its
  parent), and starting from the directory keeps the rule uniform across engines
  whose stub naming differs. A caller holding a stub path passes its parent. (b)
  Overloading the existing `reference` string. Rejected: a reference is a profile
  slug or path for the section 15.3 lookup; conflating an install directory into
  it would blur two distinct inputs and confuse the profile provider.

## D3. The Unreal rule signature

- **Decision**: Match a file whose name ends `-Win64-Shipping.exe`
  (case-insensitive) located in a directory whose trailing path components are
  `Binaries/Win64` (case-insensitive, either separator). Resolve the target to
  that file. Do not read launcher tokens, `.ini`/config, or any AppData/post-run
  artifact.
- **Rationale**: This is Unreal Engine's documented, stable shipping layout. It
  exists on disk before first launch, which a pre-launch resolver requires; the
  AppData artifacts a running game writes do not (FR-005). The suffix plus the
  `Binaries/Win64` anchor is specific enough to avoid matching the root stub or
  unrelated executables.
- **Alternatives considered**: Launcher-token classification (as the Steam
  scaffold uses). Rejected here: tokens are storefront-specific and this rule is
  engine-general and pre-launch. AppData probe. Rejected: does not exist before
  the first run, the exact moment the resolver runs.

## D4. Ambiguity handling (multiple shipping executables)

- **Decision**: When more than one candidate matches a single rule, the provider
  declines (`Ok(None)`) and records an engine-rule ambiguity note; it never picks
  one. The cascade falls through to runtime observation, which disambiguates at
  runtime from the live process set.
- **Rationale**: P-9 forbids presenting an arbitrary pick as the answer, and P-4
  forbids a silent decline. Declining plus a surfaced note satisfies both, and
  runtime observation is exactly the arbiter the cascade already relies on for
  the identical-process case S028 reserved `descends_from` for. A filesystem
  heuristic that cannot tell two shipping binaries apart should not pretend to.
- **Alternatives considered**: (a) Deterministic pick by sorted path. Rejected:
  deterministic but dishonest, it would name a specific binary the evidence does
  not single out. (b) Hard `ProviderError`. Rejected: an ambiguous layout is not
  a broken input the operator must fix before any capture; the cascade can still
  resolve via observation, so aborting it would be over-strong.

## D5. How the answer is carried (`TargetOrigin`)

- **Decision**: Add `TargetOrigin::EngineRule(EngineRuleTarget)`.
  `EngineRuleTarget` carries the recognized `Engine`, the resolved client's image
  file name and full path, and a `MatchPredicates` built for it. `profile()` and
  `into_profile()` return `None` for the new variant.
- **Rationale**: An engine-rule answer is neither a profile nor a live-process
  observation, so reusing `Observed` (which carries a `pid`) would be a lie:
  there is no process yet. A distinct variant keeps every origin honest about
  what it is. Carrying `MatchPredicates` mirrors `ObservedTarget` (`target.rs:34`)
  and gives the pipeline the rules to bind the process once it appears (watch
  mode, S028).
- **Alternatives considered**: Reuse `TargetOrigin::Observed` with a sentinel
  pid. Rejected: dishonest and would break the observed-origin invariants. Add a
  free-form origin. Rejected: loses the type distinction the enum exists for.

## D6. Building `MatchPredicates` in code

- **Decision**: Construct `MatchPredicates::default()` then set `exe` (an
  `ImagePattern::new(<file name>)`) and `path_contains` (the `Binaries\Win64`
  anchor for Unreal; the analogous anchor for Unity/Ren'Py). All in-crate.
- **Rationale**: `MatchPredicates` derives `Default` and exposes `pub(crate)`
  setters `set_exe`/`set_path_contains` (`schema.rs:257-262`), and
  `ImagePattern::new` returns a `Result` (`schema.rs:819` uses it in-crate). The
  `engine_rule` module is in the same crate, so it constructs predicates the same
  way the parser does, with no new public API on `schema.rs`.
- **Alternatives considered**: Serialize a JSON profile string and re-parse it
  (as the tests do). Rejected for production code: indirection through a string
  format for values known at the call site.

## D7. Bounding the scan

- **Decision**: The scan is bounded to the directories the rules name (the
  install root, and `Binaries/Win64` beneath it for Unreal; the sibling `*_Data`
  and `renpy` directories for Unity/Ren'Py). No unbounded recursive descent.
- **Rationale**: The layouts are documented and shallow, so a targeted probe is
  both correct and cheap, and avoids surprising cost on a large install tree.
- **Alternatives considered**: Full recursive walk collecting all executables.
  Rejected: slower, and it invites false matches from bundled tools deeper in the
  tree.

## D8. Case sensitivity and separators

- **Decision**: Match path components and filename suffixes case-insensitively,
  and treat `/` and `\` as equivalent separators, using `std::path` components
  rather than raw string search.
- **Rationale**: The capture platform is Windows (case-insensitive paths), and
  tests run on the CI/dev filesystem. Component-based matching avoids substring
  false positives (a directory literally named `Binaries\Win64Extra` must not
  match) and is robust to separator style.
- **Alternatives considered**: Case-sensitive raw substring match. Rejected:
  wrong on Windows and brittle to separator and casing variation.

## D9. Fixtures

- **Decision**: Build temporary directory trees at test time with a small
  test-only helper in the `engine_rule` test module, in the spirit of
  `fragcap-steam`'s `TempTree` (create under `std::env::temp_dir()`, write
  placeholder files, remove on drop). No fixtures are committed to `fixtures/`.
- **Rationale**: The provider's behavior is a function of directory shape;
  building the shape in a temp dir is the direct test. Committing binary trees to
  `fixtures/` would add nothing the generator does not and would newly involve
  the corpus drift check for no reason. `fragcap-profile` has no `TempTree`
  today, so a minimal local helper is the least-surprise choice.
- **Alternatives considered**: Reuse `fragcap-steam`'s `TempTree`. Rejected:
  it lives in another crate and importing a sibling's test support would cross a
  crate boundary the deps direction discourages.

## D10. Unity and Ren'Py in this slice

- **Decision**: Implement all three rules (Unreal, Unity, Ren'Py) in this slice,
  behind one rule-evaluation path with a total, iteration-order-independent
  order. Unreal is the hard acceptance gate; Unity and Ren'Py split to a
  follow-up only if unforeseen complexity forces it.
- **Rationale**: The three share the provider, the origin variant, the fidelity
  and provenance stamping, and the fixture helper. Their marginal cost is a
  recognizer function and a fixture each. Landing them together is the plan's
  default and gives the cascade real breadth immediately.
- **Alternatives considered**: Unreal only, defer Unity/Ren'Py. Held in reserve
  as the fallback if Unity/Ren'Py detection proves subtler than documented; the
  spec (FR-008) and acceptance (SC-001 targets Unreal) permit it.
