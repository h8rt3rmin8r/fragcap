# Quickstart: TargetSource discovery seam and discovery tiers

Validation scenarios that prove the slice end-to-end. All are fixture-driven and
run on any machine with no filesystem, no Steam install, and no elevation
(FR-019). Type/field detail is in [data-model.md](data-model.md); guarantees are
in [contracts/](contracts/).

## Prerequisites

- Rust workspace toolchain. In this environment, SQLite-backed crates build under
  the GNU host toolchain; CI runs the MSVC build.
- No external service, no network, no capture driver.

## Build & test

```sh
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets
cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap --test steam_source --test discovery
```

(The canonical gate is `cargo xtask ci` under the pinned MSVC toolchain in CI.)

## Scenario 1 - The seam accepts a new source with no driver change (SC-006)

Add `FixtureSource` (a canned candidate list) to a discovery run in a test and
assert the driver returns its candidates and a conserved account, with no change
to the driver, the tiers, or the entry model. Proves P-10's forward-looking
property.

## Scenario 2 - Steam refactor parity (US1, FR-006)

Drive `SteamSource` against committed Steam metadata fixtures (library folders +
appinfo). Assert: one candidate per installed title, each at heuristic-unverified
fidelity, each carrying its appid; appids present in the catalog fixture carry the
catalog classification, an absent appid is `unknown` and still produced; a
deliberately corrupt appinfo section is counted `parse_failed` and the rest
survive. Assert the candidate set equals the pre-refactor walk's output for the
same fixture (no regression).

## Scenario 3 - No Steam, games still listed (US2, SC-001)

Give `KnownRootsSource` a fixture `VolumeInventory` (two fixed volumes) and a
fixture directory tree with an Epic Games folder holding two game directories on
volume A and one on volume B. Assert three candidates, including the one on the
second volume (cross-volume enumeration, FR-008). A known root absent on a volume
contributes nothing and no error (FR-010).

## Scenario 4 - Descent stops on a hit (US2, FR-015)

Give the walk a fixture `DirectoryClassifier` that reports `Hit` at a game
directory's top level. Assert the walk emits one candidate for that directory and
does not descend into its subtree (a nested decoy directory below the hit is never
classified). A sibling directory the classifier reports `Miss` is counted
`considered_not_a_game`.

## Scenario 5 - User points at a place (US3)

`DirectorySource` on a fixture path yields exactly one candidate. `InteractiveSource`
with a scripted "yes" stamps the candidate `authored`; with a scripted "no"
produces no candidate and counts `declined_by_user` (never lost). In a
non-interactive context the interactive source produces no candidate and says why.

## Scenario 6 - Excluded volume is never walked (US4, SC-003)

Seed the eligibility table, then set volume B `eligible = false, reason =
user-excluded`. Run `KnownRootsSource` across an inventory containing A and B.
Assert B is enumerated zero times, its directories never appear as candidates, and
the account counts B's items `volume_skipped` (visible, not silent). Re-include B
and assert it is enumerated again.

## Scenario 7 - First-run seeding (US4, FR-016a)

Against an empty eligibility table, run discovery with a fixture inventory of two
present fixed volumes. Assert both are recorded `eligible = true, reason =
seeded-first-run` and both are walked. Then present a third fixed volume appearing
*after* seeding and assert it is unseen, hence not walked, until an explicit
`user-added` opt-in.

## Scenario 8 - Conservation holds everywhere (P-4, SC-004)

For every scenario above, assert `DiscoveryAccount::is_conserved()` (the named
outcome counts sum to items considered). This is the standing guard: a new
discard path with no counter fails here.

## CLI smoke (US3, FR-013)

```sh
fragcap targets discover --catalog-db <c> --local-db <l>   # tier 1 + tier 2 listing
fragcap targets scan <dir>                                 # DirectorySource listing
fragcap targets add <name> --exe <exe>                     # author (persist on use)
```

Assert `discover` lists Steam and known-roots candidates with the conserved
account, `scan` lists the pointed-at directory as one candidate, and `add` authors
a target entry (the persist-on-first-use step, FR-021). The interactive one-step
scan-confirm-author flow that wires `InteractiveSource` into the command line is
deferred to the S055 targets hero command.
