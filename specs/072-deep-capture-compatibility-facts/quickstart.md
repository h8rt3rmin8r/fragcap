# Quickstart: Deep Capture compatibility facts

This slice has no user-facing command. Validate it through the store tests and repository gates.

## Automated validation

1. Run `cargo test -p fragcap-targets compatibility`.
2. Run `cargo test -p fragcap-targets compatibility_facts`.
3. Run `cargo test --workspace --quiet`.
4. Run `cargo xtask lint`.
5. Run `cargo xtask deps`.
6. Run `cargo xtask spec`.
7. Run `cargo xtask changelog --check`.

## Privacy check

Scan the new slice files and touched code for local title names, account material, local paths, endpoints, screenshots, or other fact-finding PII before pushing.
