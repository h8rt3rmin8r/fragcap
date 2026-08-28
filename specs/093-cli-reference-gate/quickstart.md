# Quickstart: CLI Reference Gate

## Run the focused contract

```powershell
cargo test -p fragcap-cli --test cli_reference --locked
cargo test -p fragcap-cli --test cli_reference --features net --locked
```

Both runs parse the public page, compare it with the active clap command tree, audit sink grammar, and parse examples without dispatch.

## Run the contributor-facing documentation gate

```powershell
cargo xtask docs check
```

This composes the existing documentation linter with both CLI-reference variants. It does not contact the network or require Windows capture facilities.

## Build and run the complete gates

```powershell
cargo xtask docs build
cargo fmt --all -- --check
cargo xtask lint
cargo xtask ci
```

## Review a deliberate drift failure

Temporarily add a visible command or option to the clap definition without editing `cli.mdx`, or add a stale command heading to the page. The focused test must fail with the owning command path and the two compared sets. Revert the specimen after observing the failure.

## Review safety

The test may construct clap commands and parse argument vectors. It must never invoke runtime dispatch, spawn `fragcap`, read or write a managed store, start capture, alter trust, open a proxy, contact an external service, or require elevation.
