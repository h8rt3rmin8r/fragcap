# Quickstart: Validate Readable Steam Title Listing

## Prerequisites

- Rust toolchain from `rust-toolchain.toml`
- Repository checkout on the S136 branch
- No Steam installation, target store, elevation, or network access is required for controlled renderer tests

## Focused validation

Run the Steam command unit and integration tests:

```powershell
cargo test -p fragcap-cli commands::steam
cargo test -p fragcap-cli --test cli_steam
```

Expected outcomes:

- Short varied rows use aligned columns and contain no tabs.
- Long, localized, and 40-column cases use labeled vertical records without truncation.
- Positioned, unpositioned, and unregistered values remain distinct.
- Embedded layout controls appear as visible escapes.
- Ordering, empty state, store fallback, JSON, diagnostics, and read-only checks continue to pass.

## Full repository validation

```powershell
cargo xtask ci
```

The gate must finish successfully in the foreground. Review the final diff to confirm that `Cargo.lock` and storage schemas are unchanged and that no human `steam list` separator contains a tab.
