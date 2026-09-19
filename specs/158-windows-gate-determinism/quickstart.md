# Quickstart: Verify S158 Windows gate determinism

These commands exercise source and controlled test surfaces only. Do not install or run a released fragcap binary, a real game, live sensitive capture or real trust mutation.

## Focused tests

```powershell
cargo test -p fragcap --features deep-capture application::tests::writer_start
cargo test -p fragcap --features deep-capture --test deep_capture_session
cargo test -p fragcap-cli --features deep-capture --test cli_calibrate
```

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

After the branch is pushed, inspect the ordinary Windows native performance and platform conclusions on the exact pull-request head. Do not use a rerun as acceptance. Preserve any first-attempt failure as evidence and correct the mechanism before requesting another review round.
