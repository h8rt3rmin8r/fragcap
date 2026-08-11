### Decisions

**2026-08-11: extcap analyzer integration (slice S18 sub-slice A), decisions
worth recording for promotion to specification section 29.**

- **One logical extcap interface named `fragcap`, not one per host adapter.**
  fragcap's capture subject is the profile and role selection, not a network
  adapter, so it presents a single interface the configurable options
  parameterize. Its declared link type is Ethernet (DLT 1); heterogeneous
  per-packet link types (a loopback conversation) are carried by the stream's own
  interface blocks, which the analyzer reads, so the top-level DLT is a default
  rather than a constraint. One interface per adapter was rejected: it would push
  adapter selection into the analyzer and duplicate the section 12.1 selection
  precedence.
- **The extcap capture reuses the `run` back half through a second config
  builder.** `effective_config_for_extcap` mirrors the existing
  `effective_config_for_tap`: it overlays the extcap options on the profile
  exactly as `run` does and carries the FIFO as its single sink, then the same
  `components`, `build_sinks`, and `orchestrator::capture` run unchanged.
  Synthesizing a `RunArgs` was rejected as coupling extcap to the whole `run`
  grammar shape; the `_for_tap` precedent is the project's pattern for a second
  entry point.
- **The FIFO is a new transport built through the existing sink machinery, not a
  streaming sink.** A `SinkTransport::Fifo` and a `fifo:` scheme are opened by a
  small `fragcap_sink::open_fifo` and a pcapng encoder is built over the writer,
  reusing `build_sinks`. The S15 `StreamSink` (a multi-consumer server with
  per-consumer queues and a backpressure timeout) was rejected as the wrong shape:
  the analyzer hands fragcap one already-open FIFO, and the pipeline's own bounded
  drop-oldest buffer already absorbs a slow reader and counts the drops (P-4).
- **`open_fifo` is platform-correct and tier-1 testable.** A Windows `\\.\pipe\`
  path is opened as a named-pipe client (write, no create, a bounded retry on a
  busy pipe); any other path is opened for writing, created and truncated. That
  keeps production correct (connect to the analyzer's pipe on Windows, open the
  analyzer's FIFO on Unix) and lets a tier-1 test point `--fifo` at a regular temp
  file on any platform. The live named-pipe connect is tier 2, the same boundary
  live capture has had since S09.
- **doctor detection is read-only.** A new `paths::extcap_dir()` computes the
  analyzer's personal extcap directory (`%APPDATA%\Wireshark\extcap` on Windows,
  an XDG or HOME location elsewhere, with a `FRAGCAP_EXTCAP_DIR` override for
  tests). The probe reports the directory and whether a fragcap binary is present;
  it installs, downloads, and copies nothing, which is the Licensing rule and P-1
  made mechanical.
- **No new dependency.** The declaration emitters are string formatting, the FIFO
  open is `std::fs`, the capture reuses the existing pipeline, and the doctor
  probe is `std::fs`, so the slice adds nothing to `Cargo.lock`.

### Fixed

- **`fragcap run --roles` no longer panics.** `RunArgs.roles` was declared with a
  `value_parser` returning `Vec<String>` over an `Option<Vec<String>>` field;
  clap derives the element type from the `Vec` and panicked at access time on the
  type mismatch, so any `run --roles` invocation aborted. No test exercised it, so
  it had gone unnoticed. This slice's extcap-versus-run parity test surfaced it.
  Both `run` and `extcap` now split the comma-separated roles with clap's
  `value_delimiter`, and the parity test covers `--roles`.
