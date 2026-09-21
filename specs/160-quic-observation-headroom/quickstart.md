# Quickstart: Verify S160 QUIC observation headroom

These commands exercise source and controlled test surfaces only. Do not install
or run a released fragcap binary, a real game, live sensitive capture, or real
trust mutation.

## Focused tests

```powershell
cargo test -p fragcap --features deep-capture application::tests::ready_consumer
cargo test --manifest-path performance/native-proxy/Cargo.toml payload_totals
cargo xtask performance
```

The application regression must prove exact default-capacity admission, the
first counted refusal, ordered drain, and zero final ownership. The performance
regression must mutate retained, omitted, queue-dropped, and storage-dropped
inputs independently and reject every inconsistent result.

## Repository gates

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
cargo xtask msrv
cargo xtask neutral
```

## Hosted evidence

After the branch is pushed, inspect the initial Windows native performance
conclusion on the exact pull-request head. Preserve a failed artifact and amend
the branch if any case loses events or bytes. Do not use a rerun as acceptance.
