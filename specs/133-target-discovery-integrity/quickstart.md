# Quickstart: Validate Target Discovery Integrity

## Prerequisites

- Rust toolchain pinned by `rust-toolchain.toml`.
- No live capture driver, elevated privilege, network access, or game account is required for the synthetic checks.

## Focused Library Checks

```powershell
cargo test -p fragcap-targets --test known_roots
cargo test -p fragcap-targets --test detection_walk
cargo test -p fragcap-targets automatic_registration
cargo test -p fragcap-targets reconcile
cargo test -p fragcap-targets exact_batch_delete
```

Expected: the nested Steam client subtree produces zero known-roots candidates; authoritative and verified-local candidates are eligible; all policy and reconciliation accounts conserve.

## Focused CLI Checks

```powershell
cargo test -p fragcap-cli --test cli_targets
```

Expected: detailed discovery is non-persistent and annotated, summary output contains counts only, reconciliation previews without mutation, confirmed cleanup is exact and atomic, and ambiguous or user-authored rows remain.

## Count-Only Real-Machine Validation

Run only after focused checks pass:

```powershell
cargo run -p fragcap-cli -- targets discover --summary
```

Copy only aggregate counts into `real-machine-validation.md`. Do not copy detailed discovery output, local paths, title names, application ids, account names, hostnames, or volume identifiers.

## Full Gate

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo run --package xtask -- ci
```

Expected: all locally supported gates pass. The pull request supplies the Ubuntu-only wrapper, docs, analyzer, hosted-integration, and security automation required by repository policy.
