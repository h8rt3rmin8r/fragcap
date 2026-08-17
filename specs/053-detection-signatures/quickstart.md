# Quickstart: Data-driven detection signatures

Runnable validation scenarios that prove the feature works end to end. Every
scenario runs from fixture directories with no real game and no launched process
(FR-014). In this environment use the GNU host toolchain for SQLite-backed crates;
CI runs the real MSVC build.

## Prerequisites

- Workspace builds: `cargo +1.96.0-x86_64-pc-windows-gnu build --workspace`
- The `targets` feature is enabled where the store and classifier are exercised.

## Scenario 1: signatures seed and every Appendix B product is present (SC-001)

1. Seed a fresh `catalog.db` from the bundled document:
   `targets seed-signatures --db catalog.db`.
2. Confirm the signature table holds at least one row per Appendix B product (all 16
   engines, anti-cheat, and DRM products).

Expected: 16 products represented; the load reports applied, inert, and skipped
counts summing to the rows loaded, with the three binary-marker DRM rows counted
inert.

## Scenario 2: a directory-shape match classifies a game and stops descent (SC-004)

1. Build a fixture tree: a directory containing `UnityPlayer.dll` and a `Game_Data/`
   subdirectory, with an executable nested beneath.
2. Run a discovery source (or the classifier directly) over it.

Expected: exactly one candidate is emitted for the directory; its detected engine is
Unity at `verified` fidelity; descent stops at the match (the nested executable is
not emitted as a separate candidate). The discovery account stays conserved.

## Scenario 3: a new signature is honored with no code change (SC-002)

1. Against the seeded `catalog.db`, insert one filename or directory-shape signature
   row for a fictional product with a distinctive marker.
2. Create a fixture directory containing that marker and scan it.

Expected: the fictional product is detected, with no rebuild and no code change.

## Scenario 4: technologies inventory is neutral evidence (SC-006, FR-011)

1. Build a fixture directory containing an anti-cheat marker (for example
   `EasyAntiCheat_x64.dll`) and a DRM marker (`steam_api64.dll`).
2. Run `technologies --path <dir> --catalog-db catalog.db`.

Expected: both products are listed as neutral facts grouped by category; no status,
color, or wording frames either as risky, blocked, or discouraged; a title with no
online multiplayer mode is still presented as capturable.

## Scenario 5: local engine detection outranks a remote catalog claim (SC-005)

1. Seed a catalog whose engine tier attributes a title's engine at
   `heuristic-unverified`.
2. Discover that title from an install directory whose shape locally identifies the
   engine.

Expected: the presented engine is the locally detected `verified` value, not the
remote `heuristic-unverified` claim.

## Scenario 6: reduced coverage is surfaced, not silent (FR-013, P-4)

1. Scan a fixture directory with a readable root and an unreadable subtree.
2. Load a signature set that includes an inert binary-marker row and a malformed
   pattern row.

Expected: the unreadable subtree is surfaced as a coverage warning and the scan
still succeeds; the inert and skipped signature counts are reported; nothing is
silently dropped.

## Scenario 7: the embedded ruleset is gone (Clarifications, D7)

1. Grep the tree for `CompiledRuleset`, `RULES_INI`, and `assets/steamdb/`.

Expected: no matches remain; detection resolves only through the table-backed
matcher; on an absent or unseeded `catalog.db`, detection reports nothing matched
rather than falling back.

## Gate

Before proposing the change, run the full gate in the foreground and read it:
`cargo xtask ci` (MSVC on CI). Locally here, at minimum:
`cargo +1.96.0-x86_64-pc-windows-gnu test --workspace` plus
`cargo fmt --all -- --check` and clippy.
