# Research: Hint database resolution provider (S037)

All questions below were resolvable from the existing codebase and the
constitution; none required external investigation. Each records the decision,
the rationale, and the alternatives weighed.

## R1. Where the concrete provider lives (the forbidden-edge question)

- **Decision**: The concrete provider that reads the store lives in
  `fragcap-targets` as `HintDatabaseProvider`, implementing
  `fragcap_profile::TargetProvider`. The resolver trait, the precedence enum, the
  request, and the target types stay in `fragcap-profile`.
- **Rationale**: `fragcap-profile` may not depend on `fragcap-targets`
  (`cargo xtask deps` allows `fragcap-targets -> fragcap-profile` and no reverse
  edge). `fragcap-targets` already depends on `fragcap-profile` (it uses
  `fragcap_profile::jsonschema::validate_value`), so implementing the trait there
  adds no new edge. This is the exact precedent S030 set: the concrete
  `SteamWalkerProvider` lives in `fragcap-steam`, not `fragcap-profile`, for the
  same direction reason, and the profile-crate stub was removed.
- **Alternatives considered**:
  - Provider in the `fragcap` facade or the CLI: both may depend on both crates,
    but putting reusable resolution logic in the CLI buries it where a library
    consumer cannot reach it, and the facade is a thin re-export layer, not a
    home for logic. The walker precedent puts the logic in the owning capability
    crate; the hint database's owning crate is `fragcap-targets`.
  - Inverting the dependency (profile depends on targets): forbidden by P-2/P-3
    and the deps gate. Not viable.

## R2. How the appid reaches the provider

- **Decision**: Add `steam_app_id: Option<u32>` to `ResolutionRequest`, with a
  `with_steam_app_id(u32)` builder and a `steam_app_id()` accessor, mirroring
  `with_install_root`. The three existing constructors initialize it to `None`.
- **Rationale**: The request is already the channel by which per-provider inputs
  arrive (`install_root` for the engine rule and walker, `identity`+`tree` for
  observation). A provider whose input is absent declines. This keeps the single
  request able to offer an appid to the hint provider and an install root to the
  lower providers without them interfering, which is exactly how the profile
  provider and engine rule already coexist on one request (FR-015).
- **Alternatives considered**:
  - A separate `for_steam(...)` constructor only: rejected because the `run`
    `--steam` path also needs the install root for the lower providers, so the
    builder that composes onto `for_install` is the shape that fits. A dedicated
    constructor can be added later without changing this.
  - Carrying the appid as a `&str`: rejected. The store key is `u32`; parsing at
    the CLI boundary keeps the provider's lookup total and avoids re-parsing per
    query.

## R3. The new target origin and its honest shape

- **Decision**: Add `TargetOrigin::HintDatabase(HintTarget)` to `fragcap-profile`,
  mirroring `PlatformWalker(WalkerTarget)`. `HintTarget` carries the Steam
  `app_id: u32`, the `image_name: String` (the executable file name to match),
  the `identity: MatchPredicates` keyed on that file name, and the carried facts
  `launcher_mediated: Option<bool>` and `engine: Option<String>`.
- **Rationale**: A hint knows the executable *name* the community documented, but
  not its full path on *this* machine (the store has no per-install path). So,
  unlike `WalkerTarget` and `EngineRuleTarget`, `HintTarget` carries no
  `image_path`: claiming one would name a fact the provider did not read (P-9).
  The identity is keyed on the file name, which is exactly how an authored
  profile's `exe` predicate matches, so the existing capture path consumes it
  unchanged. `engine` is carried as a plain `String` (the engine's name), not the
  `fragcap-targets` `Engine` type, because `fragcap-profile` cannot import that
  type; a string keeps the fact without the edge.
- **Alternatives considered**:
  - Reusing `WalkerTarget` with an empty path: rejected. An empty `image_path`
    reads as "no path" ambiguously; a distinct type with no path field states the
    honest shape at the type level.
  - Carrying the whole `Game` row: rejected. The origin should carry only what a
    capture acts on plus the facts worth surfacing, not the entire record.

## R4. The executable selection rule

- **Decision**: From the row's `launch` array, keep entries whose `os` filter is
  unset or names Windows; reduce them to the set of distinct executable
  file-name components, compared case-insensitively. One distinct name resolves;
  zero declines (no usable executable, FR-007); two or more is an ambiguity
  decline with a recorded note (FR-008).
- **Rationale**: fragcap captures on Windows, so a macOS/Linux launch entry is
  not a candidate. Reducing to distinct file names collapses the common case of
  one executable repeated across arguments, beta branches, and osarch values into
  a single candidate, so those are not spuriously ambiguous. Refusing to pick
  among several distinct executables is the same no-guessing discipline the
  engine rule and the walker already enforce; picking by size or order is the
  coincidental heuristic the S030 research found unreliable.
- **Alternatives considered**:
  - Taking the first launch entry: rejected. Order in the store is not a
    reliability signal; it would silently pick and could arm the wrong process.
  - Ignoring the `os` filter: rejected. A Windows capture keyed on a macOS
    executable name would never match, wasting the hint and possibly masking a
    lower provider that could have answered.

## R5. Where the ambiguity and error notes live

- **Decision**: Add `HintAmbiguity { app_id, candidates }` plus a
  `note_hint_ambiguous` recorder on `ResolutionNotes` and a `hint_ambiguous()`
  accessor on `Unresolved`, mirroring `WalkerAmbiguity`. An ambiguous row is a
  decline that records the note, so a fully-unresolved outcome can explain why the
  hint database did not answer (P-4).
- **Rationale**: This is the established pattern for "a provider recognized the
  input but declined to guess"; reusing it keeps the not-resolved diagnostics
  uniform across providers.
- **Alternatives considered**:
  - No note (silent ambiguous decline): rejected under P-4 -- a decline that
    cannot explain itself is the silent form of loss.

## R6. Open-time error versus mid-resolution behavior; missing DB

- **Decision**: The database path is opened once, at resolver-assembly time in the
  CLI. A present-but-unopenable database (corrupt or wrong schema version) fails
  `Store::open`, which the CLI surfaces as a command error (FR-014). A missing
  database file, or the `targets` feature being off, means the provider is simply
  not registered and precedence 2 is empty (FR-012, FR-013); this is not an error.
  A row that is absent, sparse, or ambiguous is an ordinary decline inside
  `provide` (with the ambiguity note where applicable).
- **Rationale**: Opening once surfaces a broken database loudly and immediately,
  at the boundary where the operator supplied the path, rather than as a
  per-request surprise. It also matches the store's own contract: `Store::open`
  validates the schema version. The empty-slot fallback makes the no-database case
  byte-identical to today's no-answer stub. Holding an opened `Store` in the
  provider is sound because CLI resolution is single-threaded and the store's
  read methods take `&self`; the trait imposes no `Send`/`Sync` bound.
- **Alternatives considered**:
  - Opening the database inside `provide` on each request: rejected. It would move
    the open error from the boundary into the cascade, and re-open per request.
  - Treating an unopenable database as a silent decline: rejected under P-4 and
    FR-014 -- a corrupt database the operator explicitly pointed at is a fault
    they must see, not silence.

## R7. How the operator supplies the database path

- **Decision**: A `--hint-db <path>` option on `run`, plus a `FRAGCAP_HINT_DB`
  environment override (read when the flag is absent), mirroring
  `FRAGCAP_PROFILE_DIR`. No automatic discovery location is introduced this slice.
- **Rationale**: The existing `targets` subcommands already take an explicit
  `--db <path>`; resolution follows the same explicit-path spirit. An environment
  override lets the offline tests point at a scratch store with no developer
  machine dependency, exactly as the profile-directory tests do. A default
  discovery convention is a separate decision with its own packaging questions and
  is out of this wiring slice.
- **Alternatives considered**:
  - A fixed default path (for example `%APPDATA%\fragcap\targets.db`): deferred.
    It commits to a packaging and shipping convention the project has not settled,
    and this slice is wiring, not distribution.

## R8. Dependencies and MSRV

- **Decision**: No new crate is added. `rusqlite` is already a `fragcap-targets`
  dependency (S034), `serde_json` is already present, and `fragcap-profile` gains
  no new dependency. `cargo xtask msrv` builds default features, under which the
  facade's `targets` feature is off, so the new store-reading code is not compiled
  for MSRV -- the same posture as today. The CLI already enables `targets` (and
  thus `rusqlite`) as of S034, so MSRV coverage of the CLI is unchanged.
- **Rationale**: The slice is wiring over existing machinery; the "add nothing"
  posture holds, as it did for the pipeline slices.
