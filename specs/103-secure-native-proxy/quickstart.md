# S103 Validation Quickstart

S103 completes the native foundation used by future protocol handlers. The shipped CLI still uses the external proxy until #290.

## Focused checks

```powershell
cargo test -p fragcap-proxy --all-features --locked
cargo test -p fragcap --features deep-capture --locked
cargo test -p fragcap-cli --all-features --locked
```

Expected: capability isolation, upstream policy, certificate/leaf bounds, exact trust seams, observation conservation, and every protocol-lab scenario pass without Internet, a game, Npcap, elevation, or mutation of the operator certificate store.

## Repository parity

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask lint
cargo xtask deps
cargo xtask spec
cargo xtask msrv
cargo deny check
```

Expected: all commands exit zero. The Windows feature/package jobs must also compile the native trust and ACL modules.

## Manual review

Confirm that source and production tests contain no `certutil` process invocation after this slice, unauthenticated clients cannot allocate upstream work, private destinations require exact controlled-test grants, and public documentation still says production Deep Capture is external and incomplete.
