# Phase 0 Research: Non-Profile Production Capture Path

Each item resolves an open question from the Technical Context or the spec's
plan-time assumptions.

## R1. The command-line surface for a non-profile capture

- **Decision**: Three inputs on `run` in one clap `ArgGroup` (`required = true`,
  `multiple = false`): `--profile <ref>` (existing, now `Option<String>`),
  `--install-dir <path>`, and `--steam <app_id>`. Exactly one is required; more
  than one, or none, is a usage error (exit 2) reported by clap before
  resolution.
- **Rationale**: The operation is a capture, and `run` already owns the
  effective-config overlay, the sinks, the orchestrator, and the offline harness.
  A clap arg group expresses mutual exclusion and the "exactly one" rule
  declaratively, with the standard usage-error exit, so no hand-rolled validation
  is written. Existing `run --profile X` invocations are unchanged.
- **Alternatives considered**: A separate subcommand (rejected: it would
  duplicate the whole `run` option surface and the orchestrator call for no
  benefit). A single positional that is "a profile ref or a path or an app id"
  (rejected: ambiguous parsing, and it hides which resolution path is taken from
  the operator).

## R2. Extracting the identity from a resolved non-profile target

- **Decision**: Add a pure accessor `Target::identity(&self) -> Option<&MatchPredicates>`
  in `fragcap-profile` that returns the resolved identity for a non-profile
  origin (`Observed`, `EngineRule`, `PlatformWalker`, each of which already has an
  `identity()`), and `None` for a `Profile` origin (whose identity lives in its
  stages). `run` reads that identity and synthesizes the one-stage profile from
  it.
- **Rationale**: The three non-profile origins already expose
  `identity() -> &MatchPredicates`, but `TargetOrigin` has no uniform accessor, so
  without this the command would match all three origin variants inline, coupling
  the command to the origin enum. A single small pure accessor keeps the
  extraction in the crate that owns the type and is unit-testable there, mirroring
  the small accessors S029/S030 added (`install_root`, `with_install_root`). It
  adds no dependency.
- **Alternatives considered**: Match `target.origin()` in `run.rs` per variant
  (rejected: couples the command to the origin enum and spreads the "which origins
  carry an identity" rule across crates). Consume the target and rebuild from
  `image_name()` (rejected: loses the path anchors the engine rule and walker
  set, which are exactly what makes the identity precise).

## R3. Overlaying capture options on the non-profile path

- **Decision**: Reuse `assemble::effective_config(args, &synthesized_profile)`
  unchanged. A synthesized profile declares no `capture` block, so
  `profile.capture()` yields empty defaults and every option comes from the
  command line.
- **Rationale**: `effective_config` already reads capture defaults as
  declared-or-absent and overlays CLI options, so a defaults-less profile is the
  clean "all options from the command line" case. Reusing it keeps the full `run`
  option surface (mode, roles, direction, interfaces, sinks, bounds) on the
  non-profile path for free and avoids a parallel `effective_config_for_*` that
  would drift.
- **Alternatives considered**: A dedicated `effective_config_for_nonprofile`
  (rejected: `watch` needed one only because `WatchArgs` is a smaller arg set;
  `RunArgs` is the full set, so there is nothing to specialize). Threading options
  through a new struct (rejected: unnecessary).

## R4. Surfacing the decline reason

- **Decision**: In the non-profile branch, match `ResolutionError::Unresolved`
  explicitly and render its `ResolutionNotes` (engine-rule ambiguity, walker
  ambiguity, unreadable path, profile-not-found) into the surfaced failure
  message (exit 1). The profile branch keeps the existing
  `From<ResolutionError>` mapping unchanged.
- **Rationale**: The generic `From<ResolutionError>` reduces an `Unresolved` to
  "no target could be resolved", which is correct for a profile reference but
  loses the install-layout decline detail an operator needs (FR-007 requires the
  reason be named). The notes already carry the ambiguity and unreadable detail
  the providers recorded, so rendering them is faithful, not re-derived.
- **Alternatives considered**: Change the global `From` impl to always render
  notes (rejected: it would change the profile path's messages and risk the
  byte-identical invariant of its behavior). A new error type (rejected: the notes
  already exist; only their rendering at the command boundary is missing).

## R5. Offline testability

- **Decision**: Drive the non-profile capture through the existing offline
  harness (`OfflineArgs` plus a process script) that `run`/`watch`/`tap` tests
  already use, with a fixture install directory carrying a recognized engine
  layout (an Unreal twin-exe tree) for the resolution half. Assert the capture
  reproduces the attribution an equivalent authored one-stage identity produces.
  The real-machine `install_root_for` is covered by the Steam crate's own tests;
  the CLI test delegates from a resolved install directory.
- **Rationale**: The whole point of the cascade and the offline harness is that
  resolution and capture are testable with no game, driver, or Steam install. The
  engine-rule fixtures (S029) and the offline capture harness (S028) compose
  directly.
- **Alternatives considered**: A live capture test (rejected: needs a driver and
  a game, and the project's Tier-2 tests do not run in CI). Mocking the resolver
  (rejected: the real resolver over a fixture directory is both available and more
  honest).

## R6. The synthesized profile's game identity

- **Decision**: A generic, honest identity: a fixed placeholder game id (for
  example `run`) and a generic name, mirroring `watch`'s `id: "watch"` /
  `name: "ad hoc watch"`. For `--steam <app_id>` the app id is carried on the
  game's `app_id` field (a fact); the display name stays generic unless the
  library lookup already returned a title, in which case that title is used.
- **Rationale**: The synthesized identity's job is to bind the socket holder, not
  to assert a verified title. Fabricating a name would be the kind of tidy-looking
  lie P-9 forbids; a generic placeholder plus the app id as a fact is honest.
- **Alternatives considered**: Deriving a name from the install directory or the
  executable (rejected: it would read like a real title and invite trusting a
  guess). Requiring the operator to name it (rejected: the non-profile path exists
  precisely so they do not have to author anything).
