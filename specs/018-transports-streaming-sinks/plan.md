# Implementation Plan: Transports and streaming sinks

**Branch**: `018-transports-streaming-sinks` | **Date**: 2026-08-10 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`specs/018-transports-streaming-sinks/spec.md` (roadmap slice S15,
specification sections 14.1 to 14.4)

## Summary

Fill in the transport half of the sink model. The format writers (pcapng,
JSON Lines) already exist in `fragcap-sink` and stay orthogonal to transport.
This slice adds, all in `fragcap-sink`:

1. A rotating file transport that closes the current segment at a clean section
   boundary and opens the next numbered segment, by size or by duration.
2. A transport-agnostic streaming sink that serves any number of consumers,
   giving each its own complete, independently valid stream (its own header
   preamble replayed on connect) and its own bounded queue, with per-consumer
   drop and disconnect accounting that never stalls capture or advances the
   pipeline's capture-wide `sink_dropped`.
3. Three connection acceptors feeding that streaming sink: TCP (cross-platform),
   Windows named pipe (`cfg(windows)`), and Unix domain socket (`cfg(unix)`).
4. CLI wiring so every `--sink` scheme (`file:`, `pcapng:`, `jsonl:`, `pipe:`,
   `unix:`, `tcp://`) resolves to a real transport, plus enabling `--mode
   stream`.

The load-bearing structural insight: a single connected consumer's encoder is
itself a `Box<dyn Sink>` writing to that connection. The streaming sink is a
`Sink` that fans each packet out to per-consumer encoders, each constructed
fresh on connect by a format factory that replays the header. This reuses the
existing writers unchanged and keeps format orthogonal to transport by
construction.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (pinned toolchain per
`rust-toolchain.toml`).

**Primary Dependencies**: standard library only for the file, TCP, and Unix
socket transports (`std::fs`, `std::net`, `std::os::unix::net`,
`std::sync::mpsc::sync_channel`, `std::thread`, `std::time`). The Windows named
pipe uses `windows-sys`, which is already a workspace dependency pinned at 0.36
(added by S10 for the IP Helper socket table), taken here under
`[target.'cfg(windows)'.dependencies]` so it adds no package to `Cargo.lock`
and touches no non-Windows build. No new third-party crate is introduced.

**Storage**: capture files on disk (rotated segments); no database.

**Testing**: `cargo test`, tier 1 (offline, no capture driver, no elevation, no
game). Streaming transports are exercised with in-process readers: a loopback
`TcpStream` client, a `cfg(unix)` `UnixStream` client, and (tier 2, Windows dev
machine) a `CreateFileW` client against the named pipe. File rotation is
exercised by running the committed corpus through the rotating sink and walking
the produced segments.

**Target Platform**: Windows first (named pipe is the flagship transport). File
and TCP are cross-platform. Unix domain socket is `cfg(unix)`, present for
parity and future platforms per specification 14.2.

**Project Type**: Rust workspace (library crates plus a CLI), single repository.

**Performance Goals**: a stalled or dead network consumer imposes zero stall on
the capture path or on any other sink (the isolation property, SC-003/SC-004).
The streaming sink's `write` is non-blocking (a bounded-queue `try_send` per
consumer).

**Constraints**: each consumer stream and each rotated segment MUST parse
cleanly in an unmodified pcapng analyzer (P-5). Transports alter nothing they
forward (P-9). Every discard is counted and surfaced (P-4). Core stays
platform-neutral (P-2): all new code is in `fragcap-sink`, not `fragcap-core`.

**Scale/Scope**: multiple simultaneous consumers per streaming transport
(default per-consumer queue depth 1024 packets); rotation by size or duration
(opt-in, no default threshold); disconnect after a default 5 s of continuous
queue-full.

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1 design.*

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | PASS. Transports are ordinary output IO (file writes, `TcpListener`, named-pipe/Unix-socket server). No denylisted technique. `windows-sys` is used only for named-pipe server calls (`CreateNamedPipeW`, `ConnectNamedPipe`, `CreateFileW` in tests); `cargo xtask lint` already forbids transmit/injection names and is unaffected. |
| P-2 Core Stays Platform-Neutral | PASS. All new code lands in `fragcap-sink` (a leaf, already permitted platform deps). `fragcap-core` and its `Sink` trait gain nothing. The neutral core build is unchanged; `windows-sys` enters `fragcap-sink` only under `cfg(windows)`. |
| P-3 Capture And Attribution Separate | PASS. Sinks are neither a `PacketSource` nor a `FlowAttributor`; no merge occurs. |
| P-4 No Silent Loss | PASS, and central. Per-consumer drops and disconnects are counted in named per-consumer counters and surfaced. The streaming sink's `write` returns `Ok`, so the pipeline conservation invariant (received + buffer_dropped + refusals = captured) is preserved; per-consumer backpressure is additional accounting the sink reports, never a capture-wide loss. |
| P-5 Compatibility Outranks Richness | PASS, and central. Each consumer receives its own SHB + IDBs before packet data; each rotated segment begins its own section. Both open in an unmodified analyzer. Verified by producing bytes a standard pcapng parser accepts, not by driving an analyzer. |
| P-6 Glossary First | ACTION. New terms (transport, streaming sink, consumer, rotation segment, per-consumer queue) get `docs/glossary.md` entries in this slice's change. |
| P-7 Wrappers Stay Thin | N/A. No wrapper logic added; `doctor` is untouched by this slice. |
| P-8 House Standards Apply | PASS by gate. `cargo fmt`/`clippy`, UTF-8/LF, no em/en dashes. |
| P-9 The Instrument Does Not Lie | PASS. Transports forward observed bytes unaltered; the only omission is the counted, reported per-consumer backpressure drop, which P-4 explicitly permits. |
| Licensing | PASS. No new crate. `windows-sys` (MIT/Apache-2.0) is already vendored and pinned at 0.36; reused, adding no `Cargo.lock` package. |
| Pinned artifacts | No change required. The Windows named-pipe test is `cfg(windows)` and runs under the existing `cargo test` step of `platform.yml`; no workflow, toolchain, or release-config edit is needed. |

No principle is violated; the Complexity Tracking table is empty.

## Project Structure

### Documentation (this feature)

```text
specs/018-transports-streaming-sinks/
├── plan.md              # This file
├── research.md          # Phase 0: decisions, rationale, alternatives
├── data-model.md        # Phase 1: entities and their invariants
├── quickstart.md        # Phase 1: runnable validation scenarios
├── contracts/
│   ├── sink-cli-grammar.md   # --sink scheme + options grammar
│   └── streaming-sink.md     # streaming sink / factory / acceptor contract
├── checklists/
│   ├── requirements.md  # spec quality (from /speckit-specify)
│   └── streaming.md     # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/fragcap-sink/src/
├── lib.rs                     # re-export the new transport types
├── transport/
│   ├── mod.rs                 # SinkFactory (format -> Box<dyn Sink> over a Write),
│   │                          # shared helpers, platform gating of the transport set
│   ├── file.rs                # RotatingFileSink: numbered segments, size/duration
│   ├── stream.rs              # StreamSink: consumer registry, per-consumer bounded
│   │                          # queue, drop + disconnect accounting, fan-out
│   ├── tcp.rs                 # TcpAcceptor (std::net), cross-platform
│   ├── pipe.rs                # NamedPipeAcceptor (windows-sys), #[cfg(windows)]
│   └── unix.rs                # UnixAcceptor (std::os::unix), #[cfg(unix)]
├── pcapng/                    # unchanged
├── json/                      # unchanged
└── annotation.rs, error.rs    # unchanged

crates/fragcap-sink/tests/
├── rotation.rs                # corpus through RotatingFileSink; walk segments
├── streaming_tcp.rs           # loopback consumers, multi-consumer, backpressure
├── streaming_unix.rs          # #[cfg(unix)] consumer
└── streaming_pipe.rs          # #[cfg(windows)] consumer (tier 2, dev machine)

crates/fragcap-cli/src/
├── args.rs                    # SinkSpec gains Unix; per-sink options parsed
└── assemble.rs                # build_sinks constructs the real transports;
                               # reject_unsupported drops the S15 stubs, keeps
                               # platform + config refusals

docs/glossary.md               # new term entries (P-6)
changelog.d/S15-*.md           # added + decisions fragments
```

**Structure Decision**: All transport code is a new `transport` module inside
`fragcap-sink`. The format writers (`pcapng`, `json`) are untouched and reused
through a `SinkFactory`. The CLI is the only other crate that changes, to wire
schemes to transports. `fragcap-core` is not modified: the existing `Sink`
trait already models everything the pipeline needs, and the streaming sink is
just another `Sink`.

## Key design decisions (recorded per autopilot decision policy)

These were decided from the constitution, the architecture of record, and the
existing pipeline contract; the reasoning and alternatives are in
[research.md](research.md). The architecture-affecting ones are also promoted to
a changelog decisions fragment.

- **D1. A consumer's encoder is a `Box<dyn Sink>` over its connection.** The
  streaming sink constructs one per connection via a `SinkFactory` that replays
  the header (SHB + IDBs for pcapng), then feeds it packets. Format stays
  orthogonal to transport with no new abstraction and the writers unchanged.
- **D2. Per-consumer isolation is a bounded `sync_channel` plus a writer
  thread.** `write` does a non-blocking `try_send` per consumer; `Full` counts a
  per-consumer drop, `Disconnected` reaps the consumer. Capture never blocks.
  No concurrency crate: `std::sync::mpsc::sync_channel` supplies the bounded,
  non-blocking-producer semantics needed, the same reasoning S08 used to reject
  `crossbeam`.
- **D3. The streaming sink's `write` returns `Ok` unconditionally** (even with
  zero consumers), so the pipeline conservation invariant holds and the sink is
  never retired for downstream slowness. Per-consumer drops are the sink's own
  reported accounting, distinct from capture-wide `sink_dropped`.
- **D4. Disconnect-on-timeout uses a runtime clock.** A consumer continuously
  queue-full beyond the timeout has its connection shut down, unblocking and
  reaping its writer thread. Reading `Instant::now()` is legitimate here (this
  is runtime control, not the deterministic golden path the writers guard).
- **D5. Rotation is opt-in and boundary-clean.** With no rotation option the
  file sink is byte-identical to today. With `rotate-size` or
  `rotate-duration`, the sink finishes the current segment (its trailer) and
  opens the next numbered file before writing the packet that would cross the
  threshold, so every segment is independently valid. Numbering is a
  zero-padded ordinal inserted before the extension: `name-00000.fcapng`,
  `name-00001.fcapng`.
- **D6. The Windows named pipe uses the classic multi-instance server.** A
  listener thread creates a pipe instance, `ConnectNamedPipe` (blocking), hands
  the connected instance to a consumer, and loops to create the next instance
  (`PIPE_UNLIMITED_INSTANCES`). This is the standard multi-client pattern and
  needs no overlapped IO.
- **D7. Platform gating is by `cfg` at the transport, refusal at the CLI.** The
  named pipe is `cfg(windows)`; the Unix socket is `cfg(unix)`. A scheme whose
  transport is absent on the current platform is refused at configuration time
  with a message naming the limitation, before capture starts.
- **D8. Defaults: per-consumer queue depth 1024 packets; disconnect timeout
  5 s; rotation off unless requested.** All overridable through per-sink
  options. Values chosen to bound per-consumer memory while letting a
  keeping-up analyzer never drop.

## Open honesty note (surfaced at the pre-push halt)

The Unix domain socket transport is `cfg(unix)` and therefore is not compiled
or tested by `cargo xtask ci` on the Windows development machine or the
Windows `platform` workflow. It is implemented for parity and future platforms
per specification 14.2, and its tests run only on a Unix target. This is
recorded rather than hidden: on the primary platform it is unexercised, in the
same sense S09 live capture is scaffolded but unexecuted. The named-pipe tier-2
test does run on the Windows dev machine and its result is reported.
