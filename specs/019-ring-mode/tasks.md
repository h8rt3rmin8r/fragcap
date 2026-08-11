# Tasks: Ring mode and triggers

**Feature**: roadmap slice S16 (specification section 7.2, FR-8)
**Branch**: `019-ring-mode`
**Input**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

Test-driven: within each story a failing test is written before the code that
satisfies it. Verification is `cargo xtask ci`, run in the foreground, plus
`cargo xtask neutral` for the platform-neutral core build.

Path conventions: sink code in `crates/fragcap-sink/src/transport/`, sink tests
in `crates/fragcap-sink/tests/`, CLI in `crates/fragcap-cli/src/`, facade test in
`crates/fragcap/tests/`.

## Phase 1: Setup

- [x] T001 Add the `ring` submodule to the transport module: declare `pub mod ring;` in `crates/fragcap-sink/src/transport/mod.rs` with an empty `crates/fragcap-sink/src/transport/ring.rs`.
- [x] T002 Add glossary entries (P-6) to `docs/glossary.md` for "ring mode" and "ring window", each explicitly distinguishing the FR-8 rolling retained window from the internal bounded ring buffer of specification section 12.4; follow the existing entry template and cross-link to the 12.4 buffer entry if one exists.
- [x] T003 [P] Add changelog fragments `changelog.d/S16-ring-mode.added.md` (the slice summary) and `changelog.d/S16-ring-mode.decisions.md` (the architecture-affecting decisions D1 to D9 from plan.md, including the terminology split from the 12.4 ring buffer).

## Phase 2: Foundational (blocks US1 and US2)

- [x] T004 Define `RingWindow` in `crates/fragcap-sink/src/transport/ring.rs` per data-model.md: an enum `Duration(std::time::Duration) | Size(u64)`, `Clone, Copy, Debug, PartialEq, Eq`, with a doc comment noting a size window is measured by captured length (matching `--max-bytes`).
- [x] T005 Define the `RingSink` struct in `crates/fragcap-sink/src/transport/ring.rs` (fields per data-model.md: `path`, `window`, `factory: SinkFactory`, `retained: VecDeque<CapturedPacket>`, `retained_bytes: u64`, `evicted: u64`) and a `RingSink::create(path, window, factory)` constructor that opens no file yet.
- [x] T006 Re-export `RingSink` and `RingWindow` from `crates/fragcap-sink/src/lib.rs`.

**Checkpoint**: the sink crate compiles with an empty `RingSink` and the type is re-exported.

## Phase 3: US1 - Keep only the tail, dumped on trigger (Priority: P1)

**Goal**: a rolling window bounded by size or duration, dumped as a valid pcapng
on capture end.
**Independent test**: replay the corpus through a `RingSink` at several windows
and read the dump back; assert the retained tail, whole-input equivalence, and
the local conservation identity.

- [x] T007 [P] [US1] Write a failing unit test in `crates/fragcap-sink/src/transport/ring.rs` (`#[cfg(test)]`): a size window smaller than a sequence of fixed-size packets retains exactly the newest packets fitting the window (evict-from-front), always keeps at least the newest packet even when the window is smaller than one packet, and `evicted + retained == accepted`.
- [x] T008 [P] [US1] Write a failing unit test in `crates/fragcap-sink/src/transport/ring.rs`: a duration window retains exactly the packets whose instant is within the window measured back from the newest retained packet's instant, evicting an out-of-order older packet like any other.
- [x] T009 [US1] Implement `Sink::write` for `RingSink` in `crates/fragcap-sink/src/transport/ring.rs`: push the cloned packet, add its `captured_len()` to `retained_bytes`, evict from the front per the window (never below one packet), count evictions, and return `Ok(())` unconditionally; make T007 and T008 pass.
- [x] T010 [US1] Implement `Sink::flush` (no-op returning `Ok`) and `Sink::finish` for `RingSink` in `crates/fragcap-sink/src/transport/ring.rs`: create the `--out` file, build a pcapng encoder via `self.factory.build(...)`, write each retained packet in front-to-back order, and `encoder.finish(stats)`; return IO failures as `SinkError` naming the path.
- [x] T011 [P] [US1] Write a failing integration test `crates/fragcap-sink/tests/ring.rs`: run the committed corpus through a `RingSink` with a small size window; read the dump back with the existing pcapng block-walker helper and assert it is valid pcapng (SHB + IDBs + EPBs) containing exactly the retained tail in capture order.
- [x] T012 [P] [US1] Extend `crates/fragcap-sink/tests/ring.rs`: a window larger than the whole corpus retains every packet, and the dumped packet record sequence equals a plain single-segment `RotatingFileSink` capture of the same corpus (byte-comparable for the single-interface fixture) with none lost, reordered, or duplicated (FR-012).
- [x] T013 [US1] Make the integration tests pass and assert the sink-local conservation identity (`evicted + retained == accepted`) in `crates/fragcap-sink/tests/ring.rs`.

**Checkpoint**: `RingSink` retention and dump are fully testable offline and green.

## Phase 4: US2 - Ring mode is configured unambiguously or refused (Priority: P1)

**Goal**: the CLI resolves ring mode, builds a `RingSink` for `--out`, and
refuses every misconfiguration before capture starts.
**Independent test**: assemble tests cover the four refusals and the successful
ring build.

- [x] T014 [P] [US2] Write failing assemble tests in `crates/fragcap-cli/src/assemble.rs` (`#[cfg(test)]`) for the four refusals (ring mode without `--out`; without `--ring`; with `--max-packets`/`--max-bytes`; `--ring` without ring mode), each asserting a usage error (exit 2) naming the cause, per contracts/ring-cli-grammar.md.
- [x] T015 [US2] Carry the ring window and effective mode onto `EffectiveConfig` in `crates/fragcap-cli/src/assemble.rs` (add a `ring: Option<RingWindow>` and the resolved `CaptureMode`, mapping the CLI `args::RingWindow` onto the sink `RingWindow`), so `build_sinks` can see them.
- [x] T016 [US2] Replace the two "not yet supported (slice S16)" refusals in `reject_unsupported` in `crates/fragcap-cli/src/assemble.rs` with the ring validation: in ring mode require `--out` and `--ring`; in ring mode refuse `--max-bytes`/`--max-packets`; refuse `--ring` outside ring mode; make T014 pass.
- [x] T017 [US2] In `build_sinks` in `crates/fragcap-cli/src/assemble.rs`, when the effective mode is ring, build a `RingSink` over the `--out` file (a pcapng `SinkFactory` from the declared interfaces) instead of the continuous `RotatingFileSink`; leave file and stream modes unchanged.
- [x] T018 [P] [US2] Add an assemble test in `crates/fragcap-cli/src/assemble.rs` that a valid ring configuration builds a sink set containing the ring dump; that `--duration` combined with ring mode is accepted, not refused (FR-010); and that a profile `mode = "ring"` with no `--mode` override is validated identically (FR-008).

**Checkpoint**: the CLI wires ring mode end to end and refuses every misconfiguration.

## Phase 5: Polish & Cross-Cutting

- [x] T019 [US1] End-to-end ring runs. Placed in the CLI integration harness `crates/fragcap-cli/tests/cli_run.rs` rather than a separate facade test, because that harness already drives the whole offline pipeline through the real command entrypoint (profile resolution, write gate, sinks) and so subsumes a facade-level test (decision recorded in plan.md and the changelog). Drives a ring run with `--fire-interrupt` asserting a valid dump of the recent tail (SC-001, the "dumped on interrupt" headline), and a ring run ending by a terminal-stage-exit (non-interrupt) stop asserting the whole-input window dump equals a plain file capture's packet count (FR-003, SC-002, SC-003). Conservation (SC-005) is covered by the sink-local identity test (T013) plus the ring sink's unconditional `Ok` write, which preserves the pipeline identity the existing summary conservation test already checks.
- [x] T020 Run `cargo xtask ci` in the foreground and watch it to completion; then run `cargo xtask neutral` to confirm the platform-neutral core still builds (SC-006). Fix any failure before proceeding.
- [x] T021 Final review pass: confirm no em/en dashes, SPDX headers on any new file, UTF-8/LF, and that every new term used in code or docs has its glossary entry (P-6, P-8).

## Dependencies

- Phase 1 (Setup) has no dependencies.
- Phase 2 (Foundational) depends on T001.
- Phase 3 (US1) depends on Phase 2 (the `RingSink` type and re-export).
- Phase 4 (US2) depends on Phase 3 (it constructs the `RingSink` US1 delivers).
- Phase 5 (Polish) depends on Phase 3 and Phase 4.

## Parallel opportunities

- T003 runs parallel to T001/T002 (different files).
- T007 and T008 are parallel (same file, independent test fns; author together).
- T011 and T012 are parallel (both new in `tests/ring.rs`).
- T014 and T018 are parallel to each other only after T015/T016/T017 land; write
  the failing T014 first (TDD), then implement, then T018.

## Implementation strategy

MVP is US1 (Phase 3): the `RingSink` with retention and dump, testable in the
sink crate with no CLI. US2 (Phase 4) makes it reachable from `fragcap run`.
Ship both; the slice's value is the operator-facing `--mode ring`, which needs
both. Phase 5 proves the end-to-end offline path and runs the gate.
