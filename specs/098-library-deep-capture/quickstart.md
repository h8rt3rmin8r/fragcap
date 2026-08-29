# Quickstart: Library-First Deep Capture Sessions

## Controlled library use

1. Build a `SessionConfig` for one stored target and launch case.
2. Supply controlled implementations for proxy, trust, launch, ordinary Capture, facts, artifacts, clock, identifiers, and event delivery.
3. Call side-effect-free preflight and display or otherwise review `PreparedSession::plan()`.
4. Approve that exact `plan_id` and create the coordinator.
5. Drive the granular lifecycle or use the end-to-end convenience runner.
6. Treat `TerminalReport` as authoritative, including partial evidence and cleanup failures.

Conceptual Rust usage:

```rust
use fragcap::deep_capture::{Authorization, DeepCapture, SessionConfig};

let prepared = DeepCapture::preflight(config, &adapters)?;
present(prepared.plan());

let authorization = Authorization::approved(prepared.plan().id());
let report = prepared.into_session(adapters).run_to_completion(authorization);

if !report.is_complete() {
    inspect_failures_and_cleanup(&report);
}
```

The concrete API names may be refined during implementation, but the prepared-plan authorization boundary and returned terminal report are contractual.

## Verification targets

Run focused library and CLI tests first:

```bash
cargo test -p fragcap --features deep-capture --test deep_capture_session
cargo test -p fragcap-cli --test cli_deep_capture
```

Then run the repository gates:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
```

## Safety expectations

- Controlled tests perform no trust mutation, packet-driver access, remote network access, or game launch.
- Production sessions remain loopback-only, target-scoped, explicit, bounded, reversible, and audited.
- The library never reads stdin or chooses an exit code.
- The CLI never classifies evidence, selects facts, orders cleanup, or defines bundle truth.
