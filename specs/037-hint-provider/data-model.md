# Data Model: Hint database resolution provider (S037)

This slice adds one origin type and one request input to `fragcap-profile`, one
diagnostic note, one store method, and one provider. It introduces no new stored
schema: it reads the S034 store as it stands.

## New type: `HintTarget` (fragcap-profile, `target.rs`)

A capture target derived from a hint-database row. Parallel to `WalkerTarget` and
`EngineRuleTarget`, but it carries no `image_path` because the store does not know
where the title is installed on this machine (P-9: never name an unread fact).

| Field | Type | Meaning |
| --- | --- | --- |
| `app_id` | `u32` | The Steam application id the row was keyed by. |
| `image_name` | `String` | The executable file name to match (the file-name component of the selected launch executable). |
| `identity` | `MatchPredicates` | The match rules the pipeline binds by, with `exe` set to `image_name`. |
| `launcher_mediated` | `Option<bool>` | Carried from the row when present; the fact that the title launches through a mediator. |
| `engine` | `Option<String>` | Carried from the row's engine name when present. A plain string, not the `fragcap-targets` `Engine` type, which `fragcap-profile` cannot import. |

Accessors: `app_id()`, `image_name()`, `identity()`, `launcher_mediated()`,
`engine()`. Derives `Clone, Debug, PartialEq, Eq` like its siblings.

## Extended enum: `TargetOrigin` (fragcap-profile, `target.rs`)

Add one variant:

```
HintDatabase(HintTarget)
```

Match-arm updates in the same file:

- `Target::profile()` and `Target::into_profile()`: `HintDatabase(_) => None`
  (a hint is not a profile).
- `Target::identity()`: `HintDatabase(t) => Some(t.identity())`.

## Extended struct: `ResolutionRequest` (fragcap-profile, `resolver.rs`)

Add one field and its accessor/builder:

| Field | Type | Meaning |
| --- | --- | --- |
| `steam_app_id` | `Option<u32>` | The Steam application id offered to the hint provider. |

- Constructors `for_reference`, `for_observation`, `for_install` initialize
  `steam_app_id: None`.
- Builder `with_steam_app_id(self, app_id: u32) -> ResolutionRequest` sets it,
  mirroring `with_install_root`.
- Accessor `steam_app_id(&self) -> Option<u32>`.

## New diagnostic: `HintAmbiguity` (fragcap-profile, `resolver.rs`)

Mirrors `WalkerAmbiguity`.

| Field | Type | Meaning |
| --- | --- | --- |
| `app_id` | `u32` | The row's application id. |
| `candidates` | `usize` | How many distinct candidate executables remained. |

- `ResolutionNotes` gains `hint_ambiguous: Option<HintAmbiguity>` and
  `note_hint_ambiguous(app_id, candidates)`.
- `Unresolved` gains `hint_ambiguous: Option<HintAmbiguity>` and accessor
  `hint_ambiguous()`, threaded through `resolve()`.
- `Display` for `HintAmbiguity`: names the appid, the count, and that runtime
  observation will disambiguate, matching the walker's wording.

## Removed: `HintProvider` stub (fragcap-profile, `providers.rs`)

The no-answer stub struct and its `TargetProvider` impl are deleted, along with
the stub-specific assertions in `the_stub_providers_decline_at_their_precedence`.
Precedence 2 is now served only by the concrete provider, and only when
registered.

## New store method: `Store::game` (fragcap-targets, `store.rs`)

```
pub fn game(&self, appid: u32) -> Result<Option<Game>, TargetsError>
```

A single-row lookup by application id, returning the fully-hydrated `Game`
(including its `launch` and `technologies` rows) or `None` when the row is absent.
Reuses the existing `load_launch` and `load_technologies` helpers; the games query
gains a `WHERE appid = ?` variant. Takes `&self` (read-only).

## New provider: `HintDatabaseProvider` (fragcap-targets, new `hint_provider.rs`)

| Aspect | Value |
| --- | --- |
| State | Owns an opened `Store`. |
| Constructor | `new(store: Store) -> HintDatabaseProvider`. |
| Precedence | `Precedence::HintDatabase`. |
| Fidelity of answers | `FidelityTier::HeuristicUnverified`, always. |
| Provenance | `Provenance::new("hint-db".into(), None)`. |

Selection logic (pure, unit-testable):

1. `request.steam_app_id()` absent -> `Ok(None)`.
2. `store.game(appid)` -> `None` -> `Ok(None)` (absent row); `Err(e)` -> map to a
   hard `ProviderError` so a post-open read fault is never silent (P-4). [See the
   contract for the exact error surface.]
3. From `game.launch`, keep entries whose `os` is `None` or equals `windows`
   (case-insensitive); collect distinct executable file-name components
   (case-insensitive).
   - Empty set -> `Ok(None)` (no usable executable; FR-007).
   - One name -> build `HintTarget` (identity `exe = name`, carry
     `launcher_mediated` and `engine`), return the stamped `Target`.
   - Two or more -> `notes.note_hint_ambiguous(appid, n)`, `Ok(None)` (FR-008).

## Facade re-exports (fragcap, `lib.rs`)

- `pub mod profile`: add `HintTarget` (and `HintAmbiguity` if a consumer needs to
  read the note; the CLI does not, so this is optional).
- `#[cfg(feature = "targets")] pub mod targets`: add `HintDatabaseProvider`.

## CLI wiring (fragcap-cli)

- `RunArgs`: add `--hint-db <PathBuf>` (optional).
- `paths.rs`: add `hint_db_path(flag: Option<&Path>) -> Option<PathBuf>` reading
  the flag, then `FRAGCAP_HINT_DB`.
- A shared resolver-assembly helper builds the provider vector and, under
  `#[cfg(feature = "targets")]`, opens the store (error -> `CliError`, FR-014) and
  pushes `HintDatabaseProvider` when a path resolves to a present file.
- `run.rs` `--steam` path: parse the appid to `u32` and attach it with
  `with_steam_app_id`. `attach.rs`: drop the removed `HintProvider` import; its
  observation-only request cannot use the hint provider, so its resolver omits
  precedence 2 (behavior identical to the old no-answer stub).
