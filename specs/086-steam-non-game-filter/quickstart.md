# Quickstart: Steam Non-Game Filter

## Focused Validation

Run the Steam discovery fixture tests:

```powershell
cargo test -p fragcap --features targets --test steam_source
cargo test -p fragcap-targets --test known_roots excluded_steam_common_children_are_not_reintroduced_as_known_root_games
cargo test -p fragcap-cli listing_hides_platform_rows_for_current_non_game_steam_installs_only
```

Expected outcome:

- Non-game app types are not emitted as candidates.
- The not-a-game count increases for every excluded app type.
- `Demo`, `Game`, and unknown app types remain candidates.
- Known-roots does not reintroduce exact Steam non-game install directories.
- Existing platform-created rows for current Steam non-game installs are hidden from the hero listing while user-authored rows remain visible.
- Discovery accounts remain conserved.

## Repository Gate

Run the full repository gate:

```powershell
cargo xtask ci
```

Expected outcome:

- Formatting, clippy, tests, repository lint, dependency direction, licenses, wrappers, skills, docs, and spec checks pass.
