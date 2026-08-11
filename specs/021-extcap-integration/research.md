# Research: Extcap analyzer integration

Phase 0 decisions for slice S18 sub-slice A. Each records the alternatives
weighed against the constitution, the architecture of record (specification 14.1
to 14.5), and the existing command, assembly, and sink contracts.

## R1. The extcap contract surface

The extcap protocol an analyzer drives is four invocations (specification 14.5):
`--extcap-interfaces`, `--extcap-dlts`, `--extcap-config`, and
`--capture --fifo <path>`. In practice the analyzer also passes a version query
(`--extcap-version`) and, on the per-interface calls and the capture, an
`--extcap-interface <name>` selector, plus at capture the values for the options
the config declaration named.

**Decision**: implement the four invocations plus the two standard companions
(`--extcap-version`, `--extcap-interface`). Model the option values as the `run`
flag names so the capture invocation is parsed by the same grammar (see R4).

**Alternatives**: implementing only the literal four would reject the analyzer's
version probe and interface selector and never start. Rejected.

## R2. Interface model: one interface or one per adapter

**Decision**: one logical extcap interface named `fragcap`, declared link type
Ethernet (DLT 1). fragcap's capture subject is the profile and role selection,
not a host network adapter; a single interface keyed by those options is what the
config dialog then parameterizes. Loopback and any non-Ethernet per-packet link
type are carried by the pcapng stream's own Interface Description Blocks, which
the analyzer reads per packet, so the top-level DLT is only the default.

**Alternatives**: one extcap interface per host adapter would push adapter
selection into the analyzer and duplicate the interface-selection precedence that
already lives in `fragcap-core::interface::select`, and it would not map to
fragcap's profile-driven capture model. Rejected. A non-Ethernet default DLT
(for a loopback-only capture) was considered and rejected: the stream's IDBs
carry the true per-packet link type regardless, and Ethernet is the common case.

## R3. Where the capture path lives

The `run` command already resolves a profile, overlays capture options
(`assemble::effective_config`), assembles the pipeline (`assemble::components`),
builds sinks (`assemble::build_sinks`), and runs `orchestrator::capture`. `tap`
reaches the same back half through a second config builder,
`assemble::effective_config_for_tap`, which constructs an `EffectiveConfig`
without a `RunArgs`.

**Decision**: add `assemble::effective_config_for_extcap`, the direct analogue of
`effective_config_for_tap`, and reuse `components`, `build_sinks`, and
`orchestrator::capture` unchanged. extcap is a new front half over the existing
back half.

**Alternatives**: synthesizing a `RunArgs` from the extcap options and calling
`effective_config` would couple extcap to the whole `run` grammar shape (every
future `run` field would need a default here). The `_for_tap` precedent shows the
project prefers a dedicated builder. Rejected. A shared `run_capture` refactor of
`run.rs` was considered; it is a larger change than the slice needs and is
deferred (the `_for_extcap` builder is the minimal reuse seam).

## R4. Config option mapping

`--extcap-config` declares options with `call` names that the analyzer passes
back at capture as `--<call> <value>`. The four options are profile, roles,
direction, loopback (specification 14.5).

**Decision**: the call names are the `run` flag names: `--profile` (string),
`--roles` (string, comma-separated, the existing `parse_roles` grammar),
`--direction` (selector over both/in/out), `--loopback` (boolflag). The extcap
capture parses these with the same value grammars `run` uses and overlays them
the same way, so the dialog and the flags select capture identically (FR-006).

**Alternatives**: a profile `selector` populated from discovered profiles is a
nicer dialog than a free `string`, but it requires enumerating profiles into the
declaration and adds a bundled/user discovery dependency in the emitter for no
capability the string lacks; a string with a tooltip is the minimal form and is
recorded as the choice, with the selector left as a later enhancement. A roles
`multiselect` populated from the profile's declared roles was likewise rejected
for this slice: the declaration would have to resolve a profile before one is
selected, which the extcap config call does not carry.

## R5. The FIFO sink

`SinkFactory::build(Box<dyn Write + Send>)` builds a pcapng encoder over any
writer, which is the seam the file, rotating, and streaming sinks already use. A
FIFO is a single write-only stream to one pre-connected reader (the analyzer),
not a server accepting many consumers.

**Decision**: add `SinkTransport::Fifo(PathBuf)` and a `fifo:` scheme, and a
`build_fifo_sink` that opens the path with a new `fragcap_sink::open_fifo` and
builds a pcapng `SinkFactory` encoder over it. It is a direct single-writer sink,
reusing `build_sinks` and the pcapng writer; no new format code and no acceptor.

**Alternatives**: reusing `StreamSink` (the S15 multi-consumer streaming sink with
per-consumer bounded queues and a backpressure timeout) was considered. It targets
the wrong shape: it accepts connections through an `Acceptor` server, while extcap
hands fragcap one already-open FIFO. Its backpressure machinery is unnecessary
here because the pipeline's own bounded drop-oldest buffer already absorbs a slow
reader and counts the drops (P-4), and a closed reader retires the single sink and
ends the run (the intended stop). Rejected as over-engineering. Reusing
`RotatingFileSink` directly was rejected because its `File::create` open is wrong
for a Windows named-pipe client connect.

## R6. Opening the FIFO across platforms and for tests

The analyzer supplies the path. On Windows it is a named pipe the analyzer created
(fragcap connects as a client); on Unix it is a FIFO the analyzer created (fragcap
opens it for writing). A tier-1 test must exercise the extcap capture without a
named-pipe server or a blocking FIFO.

**Decision**: `open_fifo(path)` opens a Windows path under `\\.\pipe\` as a
named-pipe client (write, no create, a short retry on a busy pipe) and any other
path for writing with create and truncate. Production is correct (client connect
to the analyzer's pipe on Windows, open the analyzer's FIFO on Unix, both
pre-existing), and a tier-1 test points `--fifo` at a regular temp file on any
platform and reads the bytes back. The live named-pipe connect is tier 2, the same
boundary live capture has had since S09.

**Alternatives**: always opening with create was rejected because a Windows named
pipe is not created by the client. A real OS FIFO or named-pipe server in the test
was rejected as platform-specific test scaffolding for no additional coverage: the
FIFO stream's bytes are the file sink's bytes (FR-005), already pinned by the
goldens, so a regular-file target exercises the whole assembly deterministically.

## R7. Backpressure and disconnect

**Decision**: the FIFO sink is an ordinary `Sink`. A slow analyzer backpressures
the output loop, which fills the pipeline's existing bounded buffer, which drops
oldest and counts it (`buffer_dropped`); conservation holds (P-4). A closed reader
breaks the write, retiring the sink; since it is the only sink, the run ends
cleanly, which is the intended stop when the analyst quits the tool. No
extcap-specific backpressure code is added.

**Alternatives**: a per-write timeout on the FIFO (the `PollingWriter` S15 uses on
sockets) was considered and deferred: the production consumer (the analyzer) reads
promptly, and a genuinely stuck reader is bounded by the pipeline buffer rather
than stalling forever in the common case. Recorded as a known limitation in the
plan's honesty note.

## R8. doctor install detection

**Decision**: `paths::extcap_dir()` computes the analyzer's personal extcap
directory (`%APPDATA%\Wireshark\extcap` on Windows; an XDG or HOME location on
Unix; a `FRAGCAP_EXTCAP_DIR` override for tests, mirroring the existing
`FRAGCAP_PROFILE_DIR`). The probe reports whether a fragcap binary is present
there and records the directory in a new `Inputs.extcap_dir`. The classifier
`integration()` names the directory in both the installed and not-installed
details. The probe reads the filesystem read-only and installs, downloads, and
copies nothing (P-1, Licensing).

**Alternatives**: probing every possible analyzer install (global program-files
extcap dirs, multiple analyzers) was rejected for this slice as scope beyond the
personal directory the specification's "whether this has been done and where"
implies; the personal directory is the documented install target and is recorded
as the reported one.

## R9. No new dependency

**Decision**: no crate is added. The declaration emitters are string formatting,
the FIFO open is `std::fs`/`std::io`, the capture reuses the existing pipeline, and
the doctor probe is `std::fs`. This keeps the slice in the project's established
pattern of adding nothing where arithmetic over standard-library IO suffices.
