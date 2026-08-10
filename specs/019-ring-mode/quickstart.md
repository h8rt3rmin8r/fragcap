# Quickstart: Ring mode and triggers

Runnable validation for slice S16. Everything here is tier 1: no capture driver,
no elevation, no game. The offline substrate (a replay source plus the hidden
`--replay-source` / `--fire-interrupt` flags) drives the whole path.

## Prerequisites

- The workspace builds: `cargo build --workspace`.
- The committed fixture corpus under `fixtures/` (used by the existing pipeline
  and sink tests).

## Gate (run in the foreground, watched to completion)

```bash
cargo xtask ci
```

This is the full repository gate (fmt, clippy, `cargo test --workspace --locked`,
lint, deps, license, and the fixture drift check). Ring mode's unit and
integration tests run inside the `cargo test` step. Additionally confirm the
platform-neutral core still builds (not part of `ci`):

```bash
cargo xtask neutral
```

## Scenario 1: the recent tail is retained and dumped (SC-001)

Replay a fixture through ring mode with a size window smaller than the fixture,
fire the end-of-capture interrupt, and read the dump back.

- **Setup**: a `RingSink` (or the CLI `--mode ring --ring <small> --out <path>`
  over `--replay-source <fixture> --fire-interrupt`).
- **Expected**: the dump opens as valid pcapng (SHB + IDBs + packet blocks); the
  packet records are exactly the newest packets whose total captured length is
  within the window, in capture order, with older packets absent.

Covered by `crates/fragcap-sink/tests/ring.rs` (direct `RingSink` drive) and a
ring case in the facade pipeline test.

## Scenario 2: whole-input window equals a plain file capture (SC-002)

Replay the same fixture with a window larger than the whole fixture.

- **Expected**: no eviction; the dumped packet record sequence equals a plain
  `--out` file capture of the same fixture, none lost, reordered, or duplicated.
  For a single-interface fixture the dump is byte-comparable to the file golden.

## Scenario 3: every stop condition dumps (SC-003)

Drive ring mode to end by an interrupt, by a duration bound, and by source
exhaustion, each with the same input and window.

- **Expected**: each run produces the same well-formed dump of the retained
  window; the dump path does not depend on which stop condition fired.

## Scenario 4: misconfiguration is refused before capture (SC-004)

Invoke the CLI four ways: `--mode ring` with no `--out`; with no `--ring`; with
`--max-packets`; and `--ring` with no `--mode ring`.

- **Expected**: each exits 2 with a message naming the cause; no file is written
  and no capture starts.

Covered by `crates/fragcap-cli` assemble tests.

## Scenario 5: conservation holds (SC-005)

For a ring capture with evictions, assert the sink-local identity
`evicted + retained == accepted` and that the pipeline's conservation check
(received + buffer_dropped + refusals = captured) still passes, with no
capture-loss counter advanced by an eviction.

## Reading a dump back

The tests read the dumped pcapng with the same parser the existing pcapng writer
tests use, asserting the block sequence and packet count rather than driving an
external analyzer, consistent with the project's offline-testability discipline.
