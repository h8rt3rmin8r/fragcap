# Contract: Hint database resolution provider

This is the behavioral contract for the S037 wiring. It is expressed as the
observable behavior of the resolution cascade and the CLI, since fragcap is a
library-plus-CLI, not a network service.

## C1. Trait implementation

`fragcap_targets::HintDatabaseProvider` implements
`fragcap_profile::TargetProvider`:

- `precedence(&self) -> Precedence` returns `Precedence::HintDatabase` (position 2,
  between `Profile` and `EngineRule`).
- `provide(&self, request, notes) -> Result<Option<Target>, ProviderError>`
  follows C2-C6.

## C2. Resolve

Given a request whose `steam_app_id()` is `Some(a)`, and a store row for `a` whose
Windows-applicable launch entries reduce to exactly one distinct executable
file-name `n`:

- Returns `Ok(Some(target))` where:
  - `target.fidelity() == FidelityTier::HeuristicUnverified`.
  - `target.provenance().source() == "hint-db"`.
  - `target.origin()` is `TargetOrigin::HintDatabase(t)` with
    `t.app_id() == a`, `t.image_name() == n`, and `t.identity().exe()` matching
    `n`.
  - `t.launcher_mediated()` and `t.engine()` equal the row's values when the row
    recorded them.
- `target.profile()` and `target.into_profile()` are `None`.

## C3. Decline (no answer, `Ok(None)`)

The provider returns `Ok(None)` and records no note in these cases:

- `request.steam_app_id()` is `None` (nothing to look up).
- The store has no row for the appid.
- The row is launcher-mediated (`launcher_mediated == Some(true)`): its launch
  executable is the publisher launcher, not the socket-holding client, so the
  database cannot name the client and the provider defers to the lower providers.
- The row's Windows-applicable launch entries reduce to an empty set of distinct
  executables (a Tier-1-only or engine-only row).

## C4. Ambiguous decline (`Ok(None)` with a note)

When the row's Windows-applicable launch entries reduce to two or more distinct
executable file-names:

- Returns `Ok(None)`.
- Calls `notes.note_hint_ambiguous(app_id, candidates)`.
- If nothing lower in the cascade resolves, the resulting
  `ResolutionError::Unresolved` exposes the note via `hint_ambiguous()`.

## C5. Hard error (`Err(ProviderError)`)

A store read that fails *after* a successful open (a disk fault, a truncated file
opened as valid) maps to a new `ProviderError::Hint(String)` variant carrying the
underlying message. This aborts the cascade rather than declining silently (P-4).
`ProviderError::Hint` holds a `String`, so `fragcap-profile` names no
`fragcap-targets` type.

A database that cannot be *opened* never reaches `provide`: `Store::open` fails at
CLI assembly time (C7), surfaced as a `CliError`.

## C6. Ordering guarantees (exercised through `TargetResolver`)

- A request that both a profile provider and the hint provider can answer resolves
  to the profile answer (precedence 1 outranks 2), regardless of registration
  order.
- A request that both the hint provider and the engine rule can answer resolves to
  the hint answer (precedence 2 outranks 3), regardless of registration order.
- A request the hint provider declines is answered by the next lower provider that
  can, down to runtime observation.

## C7. CLI wiring and graceful degradation

- `run --hint-db <path>` (or `FRAGCAP_HINT_DB=<path>` when the flag is absent),
  with the `targets` feature built in, and a present file at `<path>`: the CLI
  opens the store and registers `HintDatabaseProvider` at precedence 2. A
  `Store::open` failure is a `CliError` (FR-014).
- No path supplied, or the path is confirmed absent, or the `targets` feature is
  not built in: precedence 2 is left empty; no error is raised for a missing
  database; resolution and capture behavior are identical to a build without this
  feature (FR-012, FR-013). Existence is checked with `try_exists`, so a path whose
  existence cannot be determined (for example a denying ACL) is surfaced as a
  `CliError` rather than silently treated as absent, alongside the unopenable case
  (FR-014).
- `run --steam <app_id>`: the CLI parses `<app_id>` to `u32` and attaches it to the
  resolution request with `with_steam_app_id`, so the hint provider is offered the
  appid while the install root remains available to the lower providers.

## C8. Invariants preserved

- No process handle is opened, no process memory is read, no traffic is
  transmitted; the provider reads only the embedded database (P-1).
- `fragcap-profile` acquires no dependency on `fragcap-targets`; the concrete
  provider is in `fragcap-targets`, which already depends on `fragcap-profile`
  (P-2, P-3; `cargo xtask deps`).
- Every hint answer is heuristic-unverified with `hint-db` provenance; it is never
  authored, verified, or observed, and it names no source it did not read (P-9).
