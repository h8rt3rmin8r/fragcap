# Tasks: Transports and streaming sinks

**Feature**: roadmap slice S15 (specification 14.1 to 14.4)
**Branch**: `018-transports-streaming-sinks`
**Input**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

Test-driven: within each story a failing test is written before the code that
satisfies it. Verification is `cargo xtask ci`, run in the foreground.

Path conventions: sink code in `crates/fragcap-sink/src/transport/`, sink tests
in `crates/fragcap-sink/tests/`, CLI in `crates/fragcap-cli/src/`.

## Phase 1: Setup

- [x] T001 Create the `transport` module skeleton: `crates/fragcap-sink/src/transport/mod.rs` with empty `file`, `stream`, `tcp` submodules declared, plus `#[cfg(windows)] pub mod pipe;` and `#[cfg(unix)] pub mod unix;`; wire `pub mod transport;` into `crates/fragcap-sink/src/lib.rs`.
- [x] T002 Add glossary entries (P-6) to `docs/glossary.md` for: transport, streaming sink, consumer, rotation segment, per-consumer queue. Follow the existing entry template and cross-link.
- [x] T003 [P] Add changelog fragments `changelog.d/S15-transports-streaming-sinks.added.md` and `changelog.d/S15-transports-streaming-sinks.decisions.md` capturing the slice summary and the architecture-affecting decisions D1 to D8 from plan.md.

## Phase 2: Foundational (blocks all streaming stories US1, US3, US4, US5)

- [x] T004 Define `SinkFactory` in `crates/fragcap-sink/src/transport/mod.rs`: holds format kind, ordered interface declarations, and payload mode; `build(conn: Box<dyn Write + Send>) -> Result<Box<dyn Sink>, WriteError>` constructs a `PcapngWriter` (writing SHB + one IDB per interface) or `JsonLinesWriter` over `conn`.
- [x] T005 [P] Unit test in `crates/fragcap-sink/src/transport/mod.rs` (`#[cfg(test)]`): `SinkFactory::build` over a `Vec<u8>` writes a valid pcapng preamble (SHB magic + IDB count) and a JSON factory writes no preamble; both then accept a packet.
- [x] T006 Define `Consumer`, `ConsumerReport`, and the `ToConsumer` channel message in `crates/fragcap-sink/src/transport/stream.rs` per data-model.md (id, sender, delivered, dropped, full_since, thread; report reasons client-closed/timeout/capture-ended/write-error).
- [x] T007 Implement the consumer writer thread in `crates/fragcap-sink/src/transport/stream.rs`: `recv` loop handling `Packet` (encoder.write; break with write-error on failure), `Finish(stats)` (encoder.finish; break capture-ended), and channel disconnect; produce a `ConsumerReport`.
- [x] T008 Implement `StreamSink` in `crates/fragcap-sink/src/transport/stream.rs` implementing core `Sink`: `write` does per-consumer `try_send` (Ok->delivered/clear full_since; Full->dropped/set full_since; Disconnected->reap), enforces the disconnect timeout by shutting a stuck consumer's connection, and returns `Ok(())` unconditionally; `flush` is a no-op; `finish(stats)` fans `Finish` to all consumers, joins consumer and acceptor threads, closes the transport, and gathers `ConsumerReport`s.
- [x] T009 Define the `Acceptor` seam in `crates/fragcap-sink/src/transport/mod.rs`: a bound transport that, on its own thread, accepts connections and registers each as a `Consumer` (via `SinkFactory::build`) into the shared registry; bind failure surfaces before capture.
- [x] T010 Expose `StreamSink::dropped_total()`, per-consumer reports, and a `Sync`-safe shared consumer registry (`Arc<Mutex<...>>`) so `write` (output thread) and the acceptor thread share it without a lock on the socket write path.

**Checkpoint**: streaming core compiles and its unit tests pass; no transport bound yet.

## Phase 3: US2 - Long capture with file rotation (Priority: P1, independent)

**Goal**: numbered, independently-readable segments by size or duration.
**Independent test**: corpus through a rotating file sink yields valid numbered
segments whose union equals the un-rotated capture.

- [x] T011 [P] [US2] Write failing test `crates/fragcap-sink/tests/rotation.rs`: run the corpus through a `RotatingFileSink` with a tiny `rotate-size`; assert multiple numbered segments exist, each starts with the SHB magic, and a block-walker over all segments reproduces the single-file packet sequence (no loss/dup/reorder).
- [x] T012 [P] [US2] Add a minimal pcapng block-walker test helper (in the test file or `tests/common`) that iterates blocks and extracts EPB payloads, sufficient to assert segment validity and packet equality.
- [x] T013 [US2] Implement `RotatingFileSink` in `crates/fragcap-sink/src/transport/file.rs` implementing core `Sink`: `None` policy is a single segment byte-identical to today; `Size`/`Duration` policy finishes the current segment and opens the next `name-NNNNN.<ext>` before writing the crossing packet; `finish(stats)` finalizes the current segment.
- [x] T014 [US2] Handle the degenerate threshold (smaller than a segment header) so no unreadable segment is emitted; assert in `rotation.rs`.
- [x] T015 [US2] Re-export `RotatingFileSink` and its rotation policy type from `crates/fragcap-sink/src/lib.rs`.

**Checkpoint**: file rotation fully testable offline and green.

## Phase 4: US3 - Remote consumer over TCP (Priority: P2)

**Goal**: a TCP client receives its own independently valid stream.
**Independent test**: a loopback client reads a well-formed stream; a second
mid-feed client gets its own preamble and only later packets.

- [x] T016 [P] [US3] Write failing test `crates/fragcap-sink/tests/streaming_tcp.rs`: bind a `StreamSink` over TCP to `127.0.0.1:0`, connect a `TcpStream` client, feed the corpus, and assert the client reads a valid pcapng preamble + packets via the block-walker helper.
- [x] T017 [US3] Implement `TcpAcceptor` in `crates/fragcap-sink/src/transport/tcp.rs`: bind a `TcpListener`, accept loop on its own thread, wrap each `TcpStream` as `Box<dyn Write + Send>`, register a consumer; bind failure returns before capture.
- [x] T018 [US3] Add the mid-capture-join assertion to `streaming_tcp.rs`: a second client connecting mid-feed receives its own SHB + IDBs and only packets sent after connect (FR-008).
- [x] T019 [US3] Add the multi-consumer assertion: two concurrent clients each receive a complete, independently valid stream declaring the same interfaces (FR-007).

**Checkpoint**: TCP streaming green; streaming core proven end to end.

## Phase 5: US4 - A slow or dead consumer never stalls capture (Priority: P2)

**Goal**: per-consumer isolation, counted drops, timeout disconnect, file sink
unaffected. Builds on the TCP transport from US3.
**Independent test**: one stalled client; the reader and the file sink still get
everything; the stalled client's drops are counted; it is disconnected on
timeout.

- [x] T020 [P] [US4] Write failing test in `crates/fragcap-sink/tests/streaming_tcp.rs`: attach a `StreamSink` (small `queue`) with a fast and a non-reading client plus a `RotatingFileSink`; feed enough packets to fill the slow client's queue; assert the fast client and the file sink receive every packet, capture does not block, and the slow client's `dropped` is non-zero and reported.
- [x] T021 [US4] Verify/adjust `StreamSink::write` so a full per-consumer queue never advances `CaptureStats.sink_dropped` and never retires the sink (assert the pipeline conservation identity holds and `sink_dropped` is unchanged by per-consumer drops).
- [x] T022 [US4] Implement and test disconnect-on-timeout: hold the slow client past `timeout`; assert it is disconnected and a `ConsumerReport`/event with reason `timeout` is produced.
- [x] T023 [US4] Test abrupt disconnect: a client that closes mid-stream is reaped with reason `client-closed`/`write-error`, and remaining consumers keep receiving without loss.

**Checkpoint**: no-silent-loss and isolation guarantees proven.

## Phase 6: US1 - Live analysis over a named pipe (Priority: P1, Windows)

**Goal**: a Windows named pipe serves consumers; Wireshark can open it.
**Independent test** (tier 2, Windows dev machine): a `CreateFileW` client reads
a valid stream.

- [x] T024 [P] [US1] Write failing test `crates/fragcap-sink/tests/streaming_pipe.rs` (`#[cfg(windows)]`): bind a `StreamSink` over `\\.\pipe\fragcap-test-<n>`, connect a client via `CreateFileW`, feed the corpus, assert a valid stream via the block-walker.
- [x] T025 [US1] Implement `NamedPipeAcceptor` in `crates/fragcap-sink/src/transport/pipe.rs` (`#[cfg(windows)]`) over `windows-sys`: `CreateNamedPipeW` (byte pipe, outbound, `PIPE_UNLIMITED_INSTANCES`), `ConnectNamedPipe`, hand the connected instance to a consumer as `Box<dyn Write + Send>`, loop to the next instance.
- [x] T026 [US1] Add `windows-sys` to `crates/fragcap-sink/Cargo.toml` under `[target.'cfg(windows)'.dependencies]` pinned to the workspace 0.36 with exactly the feature modules the named-pipe calls need; confirm `Cargo.lock` gains no new package.
- [x] T027 [US1] Implement a `Write` wrapper over the pipe instance handle (`WriteFile`), with `Drop` closing the handle (`CloseHandle`); ensure no transmit/injection call names (keep `cargo xtask lint` green).
- [x] T028 [US1] Multi-instance assertion in `streaming_pipe.rs`: two clients connect and each receives a complete, independently valid stream.

**Checkpoint**: named pipe green on the Windows dev machine; result reported.

## Phase 7: US5 - Unix domain socket parity (Priority: P3, cfg(unix))

**Goal**: parity transport for future platforms.
**Independent test** (Unix only): a `UnixStream` client reads a valid stream.

- [x] T029 [P] [US5] Write test `crates/fragcap-sink/tests/streaming_unix.rs` (`#[cfg(unix)]`): bind a `StreamSink` over a Unix socket path, connect a `UnixStream`, feed the corpus, assert a valid stream and per-consumer semantics.
- [x] T030 [US5] Implement `UnixAcceptor` in `crates/fragcap-sink/src/transport/unix.rs` (`#[cfg(unix)]`) over `std::os::unix::net::UnixListener`, same shape as `TcpAcceptor`; remove the socket file on close.

**Checkpoint**: Unix path compiles and tests on Unix (unexercised on Windows gate, per plan honesty note).

## Phase 8: CLI wiring (cross-cutting; needs the transports above)

- [x] T031 Extend `SinkSpec` in `crates/fragcap-cli/src/args.rs`: add `Unix(PathBuf)` and the `unix:` scheme; parse trailing comma-separated options (`format`, `payload`, `rotate-size`, `rotate-duration`, `queue`, `timeout`) into a `SinkOptions` carried on each spec; update `parse_sink` doc and unit tests.
- [x] T032 Implement format resolution and per-sink option validation in `crates/fragcap-cli/src/assemble.rs`: infer format from extension else require `format=`; reject a rotation option on a non-file sink, a streaming option on a file sink, a `pipe:` on non-Windows, a `unix:` on non-Unix, each as a configuration error naming the cause (FR-014).
- [x] T033 Rewrite `reject_unsupported` in `crates/fragcap-cli/src/assemble.rs`: remove the `Pipe`/`Tcp` "deferred to S15" refusals and the `--mode stream` refusal; keep the platform-availability and configuration refusals and the S16 ring / S17 launch refusals.
- [x] T034 Extend `build_sinks` in `crates/fragcap-cli/src/assemble.rs` to construct `RotatingFileSink`, and `StreamSink` with the correct acceptor (`TcpAcceptor`, `NamedPipeAcceptor`, `UnixAcceptor`) for each scheme, wiring options and the interface declarations.
- [x] T035 Enable `--mode stream` end to end and allow a streaming-only run (no file sink) in `crates/fragcap-cli/src/assemble.rs`/orchestrator; add a CLI test that `--mode stream --sink tcp://127.0.0.1:0` assembles and a no-scheme/unresolved-format sink is rejected pre-capture (SC-005, FR-017).
- [x] T036 Surface per-consumer `ConsumerReport`s through the CLI `--json` NDJSON event stream and the end-of-run summary in `crates/fragcap-cli/src/` (events/output), kept separate from `CaptureStats`.

## Phase 9: Polish and cross-cutting

- [x] T037 [P] Confirm a single-consumer/single-segment capture is byte-identical to the pre-slice goldens (no golden regeneration needed); if any golden legitimately changes, regenerate and record why.
- [x] T038 [P] Rustdoc on every new public type (`SinkFactory`, `RotatingFileSink`, `StreamSink`, acceptors, `SinkOptions`) explaining the P-4/P-5 guarantees and the platform gating.
- [x] T039 Run `cargo xtask ci` in the foreground and fix to green; run `cargo xtask neutral` to confirm the platform-neutral core still builds (FR-016, SC-006); run the `#[cfg(windows)]` named-pipe test locally and record its result.
- [x] T040 Update `crates/fragcap-sink/src/lib.rs` module doc: replace "transports and streaming sinks arrive in S15" with a description of what landed, matching the house doc style.

## Dependencies and order

- Phase 1 (Setup) -> Phase 2 (Foundational) blocks all streaming stories.
- US2 (Phase 3) is independent of the streaming core and can proceed in parallel
  with Phase 2 once Setup is done; it is the earliest green increment.
- US3 (Phase 4) requires Phase 2. US4 (Phase 5) requires US3. US1 (Phase 6) and
  US5 (Phase 7) require Phase 2. CLI wiring (Phase 8) requires the transports it
  wires (US1/US2/US3/US5). Polish (Phase 9) is last.

## Parallel opportunities

- T003 (changelog) parallel with T001/T002.
- Within a story, the `[P]` test tasks (T011/T012, T016, T020, T024, T029) are
  written before their implementation tasks.
- US2 (file rotation) and the streaming-core foundational work touch disjoint
  files and can progress concurrently after Setup.

## MVP scope

US2 (file rotation) plus the Phase 2 streaming core with US3 (TCP) is the
minimum that demonstrates both halves of the slice: independently-readable
rotated segments and a live multi-consumer stream. US1 (named pipe) is the
headline capability and lands next; US4 proves the isolation guarantees; US5 is
parity.
