# Quickstart: Validating The Session Gates Sink Writes

**Feature**: `017-session-gate-writes` | **Date**: 2026-08-10

Every check below runs at tier 1: no capture driver, no elevation, no game. The live
run-from-arm wiring is tier 2 (compiled and linked, not executed here).

## The full gate set

```sh
cargo xtask ci
```

Runs `fmt --check`, `clippy --all-targets --all-features -D warnings`, `test
--workspace --locked`, and the `lint`, `deps`, and `license` xtasks. This must pass.

## The two checks that gate the constitution but need a target or toolchain

```sh
cargo xtask neutral
cargo xtask msrv
```

`neutral` proves `fragcap-core` still builds for a target with no capture backend
(the `WriteGate` trait adds no platform dependency). `msrv` builds through the pinned
1.82 toolchain. Both exit 2 rather than 0 when they cannot run.

## The live path type-checks

```sh
cargo check -p fragcap-cli --features live,socket-table,etw
```

The run-from-arm change to `capture_live` is compiled only in CI; this is how it is
verified without a capture driver.

## The properties this slice adds

### A packet bound produces an exactly-bounded file (SC-001, SC-002)

```sh
cargo test -p fragcap-cli --test cli_run a_packet_bound_produces_an_exactly_bounded_file
```

Asserts the produced pcapng and JSON Lines each contain exactly N packet records for
`--max-packets N`, the summary reports N retained and zero out of window, and the stop
reason is `volume-reached`.

### A byte bound produces an exactly-bounded file (FR-006)

```sh
cargo test -p fragcap-cli --test cli_run a_byte_bound_produces_an_exactly_bounded_file
```

### A watch-time discard is counted (SC-003)

```sh
cargo test -p fragcap the_gate_counts_a_watch_time_discard
```

Drives the `SessionGate` directly with its window `Watching`; asserts nothing is
admitted and the watch count advances.

### The conservation identity holds with gate_dropped (SC-004)

```sh
cargo test -p fragcap-core pipeline
```

Asserts, for every sink, `received + buffer_dropped + gate_dropped + refusals ==
packets_captured`, and that a no-gate run leaves `gate_dropped` zero.

## The goldens do not move (SC-005)

```sh
cargo test -p fragcap-cli --test cli_run a_run_produces_the_capture_goldens_with_stamped_role_and_stage
cargo test -p fragcap --test corpus_pipeline
```

Both reproduce the committed goldens byte for byte. If they fail, the gate is not a
pass-through for the unbounded offline run and the change is wrong.

## What to read if a check fails

- A moved golden: the gate is discarding or reordering on the unbounded path. Confirm
  the offline driver sets the window `Capturing` before spawning the pipeline and sets
  no bound, so `admit` always returns `true`.
- A broken conservation identity: a discard path with no counter. The gate must count
  every rejected packet in `gate_dropped`, and the test helper must include the term.
- A bound test off by one: the gate's comparison must match
  `CaptureSession::check_volume_bounds` (`>=`, retained-inclusive for bytes), so the
  file and the stop reason agree.
