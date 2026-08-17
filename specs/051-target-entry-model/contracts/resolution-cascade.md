# Contract: Fidelity-ordered resolution and the preserved declines

This slice makes the store read fidelity-ordered and preserves the four declines
as fidelity-aware query conditions. The engine-layout and platform-walker
providers remain at their precedence slots (operator decision); their removal and
the literal three-position collapse are S052.

## Fidelity ordering

`authored > verified > heuristic-unverified > observed`.

- A `local.db` target row resolves at its own stored `fidelity`.
- A `catalog.db` row always resolves at `heuristic-unverified`.
- Among competing rows the highest fidelity wins.
- A live runtime observation may promote a match to `verified`.

Contract tests:
- Title present in `local.db` as `authored` and `catalog.db` as
  `heuristic-unverified` -> the `authored` entry wins.
- Competing `local.db` rows at different fidelities -> highest wins, in order.

## The four preserved declines (P-9)

The store read declines, and the cascade continues, for each of:

1. **Sparse row** - a catalog-only row with no usable launch data.
2. **Engine-only row** - engine facts but no launch executable.
3. **Launcher-mediated row** - the launch executable is the publisher launcher,
   not the socket-holding client; resolving it would name the launcher as the
   game and lose the gameplay traffic.
4. **Multi-executable row** - more than one distinct Windows executable the row
   cannot reduce to one; the ambiguity is noted and runtime observation
   disambiguates.

These are re-expressed as fidelity-aware query conditions on the read, not
discarded. The existing `hint_provider.rs` tests (the mediation decline, the
multi-exe ambiguity note, the layered-store mediation merge) MUST keep passing;
new tests assert each decline still holds when the row carries a `local.db`
fidelity.

Each decline is recorded via `ResolutionNotes` so a not-resolved outcome explains
itself (P-4): a decline is never a silent drop.

## What is unchanged this slice

- `Precedence` still lists `EngineRule` and `PlatformWalker`; their providers
  answer as before.
- `RuntimeObservation` remains the bottom arbiter.
- The `Profile` *file* provider and file search are retired (profiles are no
  longer files); `FidelityTier`, `Target`, and the resolver machinery stay.
