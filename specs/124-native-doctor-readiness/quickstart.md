# Quickstart: Native Deep Capture Doctor Readiness

## Controlled verification

```powershell
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
cargo test -p fragcap --features deep-capture
```

The controlled matrix must cover both mode verdicts, all resource health
states, malformed and bounded-out evidence, PID reuse, unrelated listener
occupancy, active-session preservation, and exact recovery outcomes.

## Repository verification

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
git diff --check
```

## Manual review

Run `fragcap doctor` and `fragcap doctor --json` against the same controlled
session root. Confirm that Capture and Deep Capture verdicts agree across
formats, native findings contain no secret values, and a read-only run changes
no filesystem, trust, routing, listener, or process state.
