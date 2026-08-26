# Quickstart: Targets Warning Stream Contract

## Focused Checks

```powershell
cargo test -p fragcap-cli --test cli_targets targets_warnings
cargo test -p fragcap-cli --test cli_targets targets_warning
```

Expected outcome:

- Warning-producing targets tests pass.
- Standard output contains command results only.
- Standard error carries warnings through the emitter.

## Full Gate

```powershell
cargo xtask ci
```

Expected outcome:

- Formatting, clippy, workspace tests, repository lint, dependency direction, and license checks pass.
