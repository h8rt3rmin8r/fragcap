# Quickstart: Steam Non-Game Filter

## Focused Validation

Run the Steam discovery fixture tests:

```powershell
cargo test -p fragcap --features targets --test steam_source
```

Expected outcome:

- Non-game app types are not emitted as candidates.
- The not-a-game count increases for every excluded app type.
- `Demo`, `Game`, and unknown app types remain candidates.
- Discovery accounts remain conserved.

## Repository Gate

Run the full repository gate:

```powershell
cargo xtask ci
```

Expected outcome:

- Formatting, clippy, tests, repository lint, dependency direction, licenses, wrappers, skills, docs, and spec checks pass.
