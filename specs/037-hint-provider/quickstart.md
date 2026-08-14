# Quickstart / validation: Hint database resolution provider (S037)

The whole feature validates offline, with no network and no game. The primary
evidence is the workspace test suite; the manual CLI walk-through is secondary.

## Prerequisites

- The standard toolchain the repository pins (`rust-toolchain.toml`).
- No npcap, no elevation, no game: this slice touches only the resolution
  cascade and an embedded database.

## Automated validation (authoritative)

Run the full CI-parity gate and watch it to completion:

```bash
cargo xtask ci
```

This runs `fmt`, `clippy --all-targets --all-features`, `test --workspace
--locked`, `xtask lint`, `xtask deps`, and `xtask license`. The
dependency-direction check (`xtask deps`) is the gate that proves
`fragcap-profile` gained no edge onto `fragcap-targets`.

Then the two checks not in `ci`:

```bash
cargo xtask msrv
```

The unit and integration tests that must be green (see `tasks.md` for the full
list):

- `fragcap-profile`: the request carries an appid; the `HintDatabase` origin
  reports no profile and yields its identity; the removed stub no longer exists;
  the `HintAmbiguity` note threads through `Unresolved`.
- `fragcap-targets`: `HintDatabaseProvider` over an in-memory store
  (`Store::open_in_memory`) resolves a one-executable row at heuristic-unverified
  with `hint-db` provenance; declines on an absent row, a Tier-1-only row, and an
  engine-only row; declines with a note on a multi-executable row; ignores a
  macOS-only launch entry; treats one executable repeated across configs as one
  candidate.
- `fragcap` facade (or CLI integration): a `TargetResolver` assembled with the
  concrete provider resolves an appid request from a seeded store; a profile
  outranks the hint answer; the hint answer outranks the engine rule; with no
  store registered the outcome is identical to the no-provider cascade.

## Manual CLI walk-through (illustrative)

Build a store with one seeded row that carries a launch executable (the offline
seed fixture used by the `targets` tests), then resolve a title by appid:

```bash
# Seed a scratch store from an offline fixture.
cargo run -p fragcap-cli -- targets import --seed <seed.json> --db ./scratch.db

# Offline resolution dry-run against that store (no capture driver needed).
FRAGCAP_HINT_DB=./scratch.db cargo run -p fragcap-cli -- run --steam <app_id> --offline ...
```

Expected: the resolved target is reported at the `heuristic-unverified` tier with
`hint-db` provenance and the executable the row named. With `FRAGCAP_HINT_DB`
unset (or pointing at a non-existent file), the same command behaves exactly as it
did before this slice.

## What "done" looks like

- `cargo xtask ci` green, including `xtask deps`.
- `cargo xtask msrv` green.
- A glossary entry for the hint provider / hint answer exists.
- A `changelog.d/` fragment records the slice and any dated decision.
