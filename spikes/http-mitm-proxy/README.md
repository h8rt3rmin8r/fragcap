# S100 `http-mitm-proxy` spike

This nested Cargo workspace is a non-shipping, loopback-only research harness for issue #274. It is not a fragcap product crate and must not enter the released workspace graph.

The harness generates a private CA in memory, trusts its public certificate only in controlled clients, and never changes the system proxy or trust store. It contacts only local origin services. Raw private material and machine-specific run output are temporary; only sanitized results belong in `specs/100-http-mitm-proxy-spike/evidence.md`.

Run `cargo test --all --locked` and `cargo run --locked -- candidate`. The `audit` workspace contains only the exact candidate feature set used for dependency, license, advisory, toolchain, and build-cost measurements.
