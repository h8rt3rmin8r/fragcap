# Implementation Plan: Extcap analyzer integration

**Branch**: `021-extcap-integration` | **Date**: 2026-08-11 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `specs/021-extcap-integration/spec.md`
(roadmap slice S18 sub-slice A, specification section 14.5).

## Summary

Deliver the extcap interface so an external analyzer enumerates, configures, and
starts fragcap as a capture source. The capture back half already exists: the
`run` command resolves a profile, overlays capture options, assembles the
pipeline through `assemble::components`, builds sinks through
`assemble::build_sinks`, and runs `orchestrator::capture`. extcap is a new front
half over that same back half plus one new transport. This slice adds:

1. A real `extcap` command in `fragcap-cli` replacing the stub: an `ExtcapArgs`
   grammar, the three declaration invocations printing the extcap control grammar
   to standard output, and a `--capture --fifo <path>` path that streams pcapng
   to the analyzer's FIFO.
2. A FIFO sink: a new `SinkTransport::Fifo` and a `fifo:` sink scheme, built by a
   small `fragcap_sink::open_fifo` that opens the analyzer-supplied path for
   writing and hands it to the existing `SinkFactory`. No new format code; pcapng
   over one writer.
3. A real `doctor` extcap report: the probe reads the analyzer's extcap directory
   read-only and reports whether a fragcap binary is present and the directory
   path.
4. `docs/glossary.md` entries for extcap, DLT and link type, and named pipe and
   FIFO (P-6).

The load-bearing structural insight: extcap capture is `run` with a different
front half and a FIFO sink. The declaration invocations are pure functions over a
fixed interface model, unit-testable with no analyzer; the capture reuses the
existing offline substrate (a replayed source) so the whole path is tier-1
testable against a regular temp file, with the real named-pipe connect being the
only tier-2 piece, exactly as live capture has been since S09.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (pinned `rust-toolchain.toml`).

**Primary Dependencies**: standard library only (`std::fs`, `std::io`). The FIFO
sink reuses the existing `SinkFactory` and pcapng writer in `fragcap-sink`. No new
third-party crate. On Windows the named-pipe client open uses the `std::fs`
open path (`OpenOptions`), not a new binding.

**Storage**: none new. The capture streams to the analyzer's FIFO; nothing is
retained.

**Testing**: `cargo test`, tier 1 (offline, no capture driver, no elevation, no
analyzer). The declaration invocations are asserted against the extcap control
grammar directly. The capture is driven through the existing hidden offline
substrate (`--replay-source`) writing to a regular temp file used as the FIFO
path, read back with the same pcapng parser the writer tests use, and compared to
the committed pcapng golden. The doctor classifier is unit-tested over a
constructed `Inputs`.

**Target Platform**: cross-platform for everything except the production
named-pipe client connect. The declaration invocations, the config mapping, the
FIFO sink over a regular path, and the doctor classifier all run in the neutral
core build environment and on Windows alike. The Windows named-pipe client open
is the one platform-specific branch and its live use is tier 2.

**Project Type**: Rust workspace (library crates plus a CLI), single repository.

**Performance Goals**: the FIFO stream is the existing pcapng encoder over one
writer; no added copy. Backpressure from a slow analyzer flows into the pipeline's
existing bounded drop-oldest buffer (counted), exactly as for a slow file sink.

**Constraints**: the streamed bytes MUST parse cleanly in an unmodified analyzer
(P-5), and MUST be the same bytes the file sink produces (FR-005). No new capture
or attribution technique (P-1, P-3, P-9). The doctor probe installs, downloads,
and copies nothing (P-1, Licensing). Core stays platform-neutral (P-2): all new
code is in `fragcap-sink` and `fragcap-cli`, not `fragcap-core`.

**Scale/Scope**: one logical extcap interface (`fragcap`); one FIFO per capture;
four configurable options (profile, roles, direction, loopback).

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1 design.*

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | PASS. extcap adds a command front half, a FIFO sink (a file/pipe write), and a read-only directory probe. No denylisted technique, no process handle, no capture or transmit call. `cargo xtask lint` is unaffected. |
| P-2 Core Stays Platform-Neutral | PASS. All new code lands in `fragcap-sink` and `fragcap-cli`. `fragcap-core` and its `Sink`/`PacketSource`/`FlowAttributor` traits gain nothing. The neutral core build is unchanged. |
| P-3 Capture And Attribution Separate | PASS. extcap reuses the assembled pipeline; it introduces no source and no attributor and merges neither. |
| P-4 No Silent Loss | PASS. The FIFO sink is an ordinary `Sink`: it returns `Ok` for a written packet, so the pipeline conservation invariant holds; a slow reader backpressures into the existing bounded buffer (buffer_dropped, counted); a closed reader retires the sink (the only sink), ending the run cleanly. No new uncounted discard path. |
| P-5 Compatibility Outranks Richness | PASS, and central. The FIFO stream is the unchanged pcapng writer's bytes over a different transport: a Section Header Block, one Interface Description Block per declared interface, then the packets. It is byte-identical to a file capture of the same input, so an unmodified analyzer reads it. |
| P-6 Glossary First | ACTION. `extcap`, `DLT (link type)`, and `named pipe / FIFO` get `docs/glossary.md` entries in this slice's change. |
| P-7 Wrappers Stay Thin | N/A here. This slice touches no shell wrapper; the wrappers are S18 sub-slice B. `doctor` gains a read-only report, not wrapper logic. |
| P-8 House Standards Apply | PASS by gate. `cargo fmt`/`clippy`, UTF-8/LF, no em/en dashes, SPDX headers on new files. |
| P-9 The Instrument Does Not Lie | PASS. The FIFO stream carries observed bytes unaltered; the extcap options scope the capture (a declared operator choice), they do not mask or rewrite observations. |
| Licensing | PASS. No new crate. The doctor probe detects only; it installs and downloads nothing. |
| Pinned artifacts | No change required. extcap is exercised under the existing `cargo test` step; no workflow, toolchain, or release-config edit is needed. |

No principle is violated; the Complexity Tracking table is empty.

## Project Structure

### Documentation (this feature)

```text
specs/021-extcap-integration/
├── plan.md              # This file
├── research.md          # Phase 0: decisions, rationale, alternatives
├── data-model.md        # Phase 1: entities and their invariants
├── quickstart.md        # Phase 1: runnable validation scenarios
├── contracts/
│   ├── extcap-cli-grammar.md  # the four invocations + declaration output grammar
│   └── fifo-sink.md           # SinkTransport::Fifo, open_fifo, build contract
├── checklists/
│   ├── requirements.md  # spec quality (from /speckit-specify)
│   └── extcap.md        # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/fragcap-sink/src/
├── lib.rs                     # re-export open_fifo
├── transport/
│   ├── mod.rs                 # unchanged (SinkFactory, InterfaceSpec, Format)
│   ├── fifo.rs                # open_fifo: open the analyzer path for writing
│   │                          # (named-pipe client on Windows, write+create else)
│   └── file.rs, stream.rs...  # unchanged
└── pcapng/                    # unchanged (reused through SinkFactory)

crates/fragcap-cli/src/
├── cli.rs                     # Extcap(ExtcapArgs) replaces Extcap(StubArgs);
│                              # ExtcapArgs grammar (declaration + capture +
│                              # config-call flags + flattened OfflineArgs)
├── args.rs                    # SinkTransport::Fifo + `fifo:` scheme in parse_sink
├── assemble.rs                # effective_config_for_extcap (mirrors _for_tap);
│                              # build_fifo_sink in build_one_sink
├── commands/
│   ├── extcap.rs              # NEW: declaration invocations + capture dispatch
│   ├── extcap/grammar.rs      # NEW (or inline): pure extcap-grammar emitters
│   ├── mod.rs                 # register the extcap module
│   └── stub.rs                # drop the Extcap variant
├── doctor/
│   ├── mod.rs                 # Inputs gains extcap_dir: Option<PathBuf>
│   ├── checks.rs              # integration() names the directory in both states
│   └── probe.rs               # detect a fragcap binary in the extcap dir
├── paths.rs                   # extcap_dir(): APPDATA\Wireshark\extcap (+ env override)
└── lib.rs                     # dispatch Extcap to commands::extcap::run

crates/fragcap-cli/tests/
└── cli_extcap.rs              # NEW: the four invocations + doctor report e2e

docs/glossary.md               # extcap / DLT / named pipe entries (P-6)
changelog.d/S18a-extcap.added.md          # user-facing capability
changelog.d/S18a-extcap.decisions.md      # promotable decisions (if any)
```

**Structure Decision**: The capture back half (`components`, `build_sinks`,
`orchestrator::capture`) is reused unchanged. extcap adds a front half in
`commands/extcap.rs`, a second `EffectiveConfig` entry point
(`effective_config_for_extcap`, the direct precedent being the existing
`effective_config_for_tap`), and one new transport (`SinkTransport::Fifo` built by
`open_fifo`). `fragcap-core`, the pipeline, the capture session, the attributor,
and the pcapng writer are all unmodified.

## Key design decisions (recorded per autopilot decision policy)

Decided from the constitution, the architecture of record (specification 14.1 to
14.5), and the existing command, assembly, and sink contracts; reasoning and
alternatives are in [research.md](research.md).

- **D1. One logical extcap interface named `fragcap`, DLT Ethernet.** fragcap is a
  single process-attributed capture source whose subject is the profile and role
  options, not a host adapter, so it presents one extcap interface rather than one
  per adapter. Its declared link type is Ethernet (DLT 1); heterogeneous per-packet
  link types (loopback) are carried by the stream's own Interface Description
  Blocks, which the analyzer reads, so the top-level DLT is a default, not a
  constraint.
- **D2. The declaration invocations are pure emitters to standard output.** Each
  of `--extcap-interfaces`, `--extcap-dlts`, `--extcap-config` produces a fixed
  block of extcap control grammar from the interface model, with no capture and no
  IO beyond the write. They are unit-tested against the grammar, so the contract
  holds with no analyzer installed.
- **D3. Capture reuses the run back half via `effective_config_for_extcap`.** A
  new builder mirrors the existing `effective_config_for_tap`: it overlays the
  extcap options (profile, roles, direction, loopback) onto the resolved profile's
  `[capture]` defaults exactly as `run` does (FR-006), sets `mode = File`, no ring,
  no launch, no volume bounds, and carries the FIFO as its single sink. Then
  `components` and `build_sinks` and `orchestrator::capture` run unchanged.
- **D4. The FIFO is a new transport built through the existing sink machinery.**
  `SinkTransport::Fifo(PathBuf)` and a `fifo:` scheme are added to the sink
  grammar; `build_one_sink` handles it by opening the path with
  `fragcap_sink::open_fifo` and building a pcapng `SinkFactory` encoder over the
  returned writer. It is a direct single-writer sink, not a `StreamSink` (which
  serves many consumers through an acceptor); the analyzer is one pre-connected
  reader, so the acceptor model does not apply.
- **D5. `open_fifo` is platform-correct and tier-1 testable.** On Windows a path
  under `\\.\pipe\` is opened as a named-pipe client (write, no create, a short
  retry on a busy pipe); any other path is opened for writing, creating and
  truncating it. That keeps the production case correct (connect to the analyzer's
  pipe on Windows, open the analyzer's FIFO on Unix) and lets a tier-1 test point
  `--fifo` at a regular temp file on any platform, reading the bytes back. The live
  named-pipe connect is tier 2, like live capture.
- **D6. The config options map to the `run` flag names.** `--extcap-config`
  declares `--profile` (string), `--roles` (string), `--direction` (selector
  both/in/out), `--loopback` (boolflag). The call names are the `run` flag names,
  so the capture invocation the analyzer builds is parsed by the same grammar and
  resolved by the same overlay, which is what makes the dialog and the flags select
  capture identically (FR-006).
- **D7. extcap tolerates the standard protocol surface and refuses misuse.** It
  accepts `--extcap-version [value]` and `--extcap-interface <name>` (the version
  query and the interface selector), errors (exit 2) on an unknown interface, and
  errors on `--capture` without `--fifo` and on a declaration invocation missing
  its required selector, before any capture starts (FR-007, FR-008).
- **D8. doctor reads the extcap directory read-only.** `paths::extcap_dir()`
  computes the analyzer's personal extcap directory (`%APPDATA%\Wireshark\extcap`
  on Windows, an XDG/HOME location on Unix, with a `FRAGCAP_EXTCAP_DIR` override
  for tests). The probe reports whether a fragcap binary is present and sets a new
  `Inputs.extcap_dir`; the classifier names the directory in both the installed and
  not-installed details. It installs and copies nothing (P-1, Licensing).
- **D9. Only pcapng streams over extcap.** Analyzers consume pcapng, so the FIFO
  sink is pcapng; JSON Lines and rotation are not extcap transports. The `fifo:`
  scheme, like the network transports, requires pcapng.

## Open honesty note (surfaced at the pre-push halt)

The production extcap path is a live capture the analyzer starts over a real
named pipe; that connect (the Windows `\\.\pipe\` client open and a live capture
driver) is tier 2 and unexecuted in continuous integration, exactly as live
capture has been since S09. What this slice proves at tier 1 is the whole extcap
contract and capture assembly: the three declarations against the grammar, and a
`--capture --fifo` run driven by the offline substrate reproducing the committed
pcapng golden through the FIFO sink over a regular temp file. The bytes on the
FIFO are the file sink's bytes (FR-005), which the existing goldens already pin.
