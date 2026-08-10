# Quickstart: validating transports and streaming sinks

All scenarios are tier 1 (offline: no capture driver, no elevation, no game)
except the named-pipe scenario, which is tier 2 and runs on a Windows machine.
They drive the sinks from the replay source and scripted attributor the corpus
tests already use.

## Prerequisites

- `cargo` with the pinned toolchain.
- The committed fixture corpus under `fixtures/` (already present).

## Run the gate

```sh
cargo xtask ci
```

This is the authoritative check. The scenarios below are what the new tests
inside it assert.

## Scenario 1: file rotation produces independent segments (SC-002)

- Run the corpus through a `RotatingFileSink` with `rotate-size` small enough to
  force several segments.
- Expect: numbered segments `name-00000.fcapng`, `name-00001.fcapng`, ...; each
  begins with a Section Header Block and its Interface Description Blocks; each
  parses on its own.
- Expect: walking the segments in order yields exactly the packet sequence of a
  single-file capture of the same input, none lost, duplicated, or reordered.
- Test: `crates/fragcap-sink/tests/rotation.rs`.

## Scenario 2: TCP consumer receives a valid stream (SC-001)

- Bind a `StreamSink` over TCP to `127.0.0.1:0` (ephemeral port).
- Connect a `std::net::TcpStream` client, then feed the corpus.
- Expect: the client reads a well-formed pcapng stream (SHB, one IDB per
  interface, then packet blocks) accepted by a pcapng block walker.
- Connect a second client mid-feed: it receives its own SHB + IDBs and only
  packets sent after it connected, never an earlier one (FR-008).
- Test: `crates/fragcap-sink/tests/streaming_tcp.rs`.

## Scenario 3: a stalled consumer degrades only itself (SC-003, SC-004)

- Attach a `StreamSink` over TCP with two consumers and a `RotatingFileSink` to
  the same run. Set a small `queue`.
- Have one client stop reading; keep feeding the corpus.
- Expect: the reading client receives every packet; the file sink writes every
  packet; the stalled client's `dropped` count is non-zero and reported;
  capture never blocks.
- Expect: with the stalled client held past `timeout`, it is disconnected and a
  disconnect event with its id and reason `timeout` is emitted.
- Test: `crates/fragcap-sink/tests/streaming_tcp.rs`.

## Scenario 4: conservation and no-silent-loss (P-4)

- For every run above, assert the pipeline conservation identity still holds
  (received + buffer_dropped + refusals = captured) and that `sink_dropped` is
  unaffected by per-consumer streaming drops.
- Assert per-consumer `delivered + dropped` equals packets offered while
  connected.

## Scenario 5: Unix domain socket parity (cfg(unix))

- On a Unix target, bind a `StreamSink` over a Unix domain socket, connect a
  `UnixStream` client, feed the corpus, assert a valid stream with the same
  per-consumer semantics.
- Test: `crates/fragcap-sink/tests/streaming_unix.rs` (compiled and run only on
  Unix; unexercised on the Windows gate, per the plan's honesty note).

## Scenario 6: Windows named pipe (tier 2, Windows dev machine)

- Bind a `StreamSink` over a named pipe `\\.\pipe\fragcap-test-<n>`.
- Connect a client with `CreateFileW`, feed the corpus, assert a valid stream.
- Test: `crates/fragcap-sink/tests/streaming_pipe.rs` (`cfg(windows)`); run
  locally and report the result.

## Scenario 7: CLI wiring (SC-005)

- `fragcap run --replay-source <fixture> --attr-script <script> --mode stream
  --sink tcp://127.0.0.1:0` starts and serves.
- Every accepted scheme resolves to a working transport; an unresolvable
  format, an unavailable-platform scheme, or an unbindable destination is
  rejected before capture with a message naming the cause.
