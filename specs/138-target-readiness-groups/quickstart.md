# Quickstart: Target Readiness Groups

## Prerequisites

- Rust 1.88 through the repository toolchain.
- No Npcap installation, elevation, live game, or network access.

## Focused Validation

```powershell
cargo test -p fragcap-cli --test cli_targets --locked
```

Expected results:

- Empty output contains only the existing population guidance and no readiness heading or next-command footer.
- All-ready output contains only `Ready to capture:` and numbers rows from one in handle order.
- All-setup output contains only `Needs setup:` and numbers rows from one in handle order.
- Mixed output places ready rows first, setup-needed rows second, retains continuous numbering, and measures each table independently.
- Every numeric selector resolves through the saved snapshot to the row displayed at that number.
- The next-command footer recommends a ready row whenever one exists.

## Complete Validation

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
```

Expected result: every command exits successfully, the dependency graph is unchanged, and repository text checks report no encoding, punctuation, or mojibake violation.
