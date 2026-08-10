# Quickstart: Attribution Session-to-Pipeline Integration

**Slice**: 015 | **Date**: 2026-08-10

Tier-1 validation. No capture driver, no elevation, no game. Every check runs on
any developer machine and in the default CI job.

## Prerequisites

- Rust toolchain (workspace MSRV 1.82) and the repo checked out on
  `feat/attribution-pipeline-integration`.

## The gate (run in the foreground, watch to completion)

```bash
cargo xtask ci
```

Runs format, clippy (`-D warnings`), `cargo test --workspace --locked`, the
conventions lint, dependency-direction, and license checks. This is the primary
proof for the slice.

Then the two checks not in `ci` (each exits 2, not 0, if its target/toolchain is
absent; report which):

```bash
cargo xtask neutral
```

```bash
cargo xtask msrv
```

## What the new tier-1 tests demonstrate

Map from spec success criteria to the test that proves it:

- **SC-001 (refresh driven mid-run)** - `fragcap-core` pipeline test: a test
  attributor that resolves nothing until its `refresh(&self)` flips its published
  answer; run through `Pipeline`, a flow unresolvable at first is resolvable after
  the control thread's refresh. Also asserts `wants_refresh` gates it.
- **SC-002 (narrow to profiled)** - `fragcap` facade test: a `RoleStampingAttributor`
  over an inner returning owned endpoints for a profiled and an unprofiled PID,
  with a binding snapshot naming only the profiled PID; `active_endpoints()`
  returns only the profiled endpoints, across IPv4, IPv6, and a wildcard UDP bind.
- **SC-003 (lock-free resolve across a publication)** - `fragcap-attr` test: the
  several-threads resolve test, adapted so `refresh(&self)` is driven on one
  thread through the shared `Arc<dyn FlowAttributor>` while others resolve; every
  resolve observes a whole index and none blocks.
- **SC-004 (every implementor moved)** - the workspace compiles with
  `refresh(&self)` applied everywhere, and the dyn-compatibility / `Send + Sync`
  compile-time assertions in `traits.rs` still pass.
- **SC-005 (RefreshDriver removed; offline unchanged)** - the corpus goldens
  (`crates/fragcap/tests/goldens.rs`, `corpus_pipeline.rs`) reproduce
  byte-identically; the socket-table wiring change is `cfg`-gated.

## Offline goldens must not move

```bash
cargo test -p fragcap --test goldens
cargo test -p fragcap-cli --test cli_run
```

Both pass with no golden regeneration. `wants_refresh` defaults false on the
scripted path, so the offline pipeline never refreshes and produces identical
bytes.

## The cfg-gated live wiring (compiled only, not run)

The `RefreshDriver` removal and `live_components` rewrite compile only under the
socket-table feature on Windows:

```bash
cargo build -p fragcap-cli --features socket-table
```

This proves the live wiring compiles. It is NOT executed in CI (tier 2 needs
npcap + elevation); report it as compiled-only, never as verified live.

## Expected outcome

`cargo xtask ci` green; `neutral` and `msrv` exit 0 (or 2 = could-not-run,
reported as such); offline goldens byte-identical; the live wiring compiles under
its feature. The two resolutions are promoted to specification sections 11, 12.2,
and 29, and the trait deviation is recorded in a dated changelog decision.
