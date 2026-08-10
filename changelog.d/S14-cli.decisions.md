### Decisions

**2026-08-10. Decisions taken while implementing S14, recorded for promotion to
specification section 29.**

- **clap and ctrlc land on `fragcap-cli` alone.** The argument grammar of
  section 17.2 is a fixed set of flags, defaults, subcommands, and help text, and
  clap derive produces exactly that from typed structs rather than a hand-rolled
  parser that would drift from the help the specification prints. `ctrlc`
  supplies the portable console-interrupt hook the standard library lacks, so an
  operator interrupt becomes `StopReason::Interrupt` and an exit-0 success. Both
  sit at the top of the dependency graph where nothing reaches them, which `cargo
  xtask deps` enforces, so a large graph on the binary crate never touches core.
- **clap is pinned exactly to 4.5.32 for the 1.82 minimum.** clap 4.6 declares
  edition 2024 and rust-version 1.85, and later 4.5 patches (4.5.61) pull
  `clap_lex` 1.0, which declares the same, both above the workspace's 1.82
  minimum. Either resolves under a caret or tilde range and breaks `cargo xtask
  msrv`, a check most contributors cannot run locally, exactly as `libloading`
  0.9 did in S09. Because the incompatibility is in a transitive patch a version
  range cannot exclude, the pin is exact; 4.5.32 is edition 2021, rust-version
  1.74, on `clap_lex` 0.7. Raise it only alongside a workspace MSRV bump. `ctrlc`
  and the dev-only `tempfile` build at 1.82 unpinned.
- **The size grammar lives in `fragcap-core::size`, base 1024.** It mirrors the
  duration grammar (integer plus a required unit, zero rejected) so the two
  literal grammars are consistent, and living in core lets the ring slice (S16)
  reuse it beside `duration` rather than reimplement a size parser in the CLI.
  Binary units match how buffer and file sizes are reasoned about.
- **The role and stage bridge is a `FlowAttributor` decorator in the facade
  `session` module.** `Attribution` already carries `role` and `stage` with
  builder methods, so `RoleStampingAttributor` populates existing fields rather
  than changing a type, and a decorator is still just a `FlowAttributor` with no
  packet acquisition, so P-3 holds. The facade `session` module is its home
  because that is the one place already above both `fragcap-capture` and
  `fragcap-attr`; `arc-swap` is not pulled into the facade for it, because the
  binding snapshot is published on a rare write (a process start or exit) and a
  short-held lock around an `Arc` swap suffices off the per-packet path.
- **The session and the pipeline run side by side, joined by a tee.** The
  pipeline owns the packet threads and never surfaces individual packets; a
  session driver owns the `CaptureSession` and connects through a `StopHandle` and
  the published binding snapshot. A `TeeCountingSink` prepended to the sink list
  forwards each retained packet's length and instant to the driver, so the session
  stays the single authority for the volume bound and its retained counters while
  it never sees the packet path, and the tee's receipts stay inside the pipeline's
  conservation identity.
- **Events are hand-rolled NDJSON over the sink escaper; every diagnostic stream
  is standard error.** `serde_json` stays test-only, so the small fixed event set
  is serialized by hand over the one escaper the sinks already use, keeping serde
  out of the runtime graph. Command results (`doctor`, `profile`) go to standard
  output and a capture's progress, summary, and events to standard error, so a
  sink writing capture data to standard output is never contaminated. Timestamps
  are RFC3339 `Z` formatted by hand with a civil-date conversion, no date crate.
- **`doctor` is a pure `Inputs` to `Report` classifier over a thin probe.** Every
  classification and the exit decision are testable with hand-built inputs and
  goldens on any target, which is the only way to cover the section 26.3 matrix
  without the environment. The thin `cfg(windows)` probe reads the machine
  read-only and installs nothing. The two npcap options are separate checks, each
  naming its own remediation when absent. A missing process-event session is a
  blocking fail only when the session is elevated and cannot open, and a
  non-blocking skip when the tracing capability is not built in.
- **`run` and `tap` are driven offline through hidden flags.** A recorded capture
  replayed as the source, a scripted attributor, and a scripted process timeline
  are selected by hidden flags on `run` and `tap`, so the whole capture path is
  exercised from `run()` in a tier-1 test with no capture driver, no elevation,
  and no game. The flags are hidden rather than removed because the same assembly
  seam is where the feature-gated live path attaches. In the offline shape
  acquisition is resolved before the pipeline starts, so the published bindings
  are visible when every packet is attributed, which is what makes the stamped
  output a stable golden rather than a race between the publish and the resolve.
- **Live, socket-table, and ETW assembly is now wired behind their features.**
  When no offline replay source is given, `assemble::components` assembles the
  real backends: interface enumeration and the section 12.1 selection precedence
  behind `live`, one `LiveSource` binding per selected interface, the
  `SocketTableAttributor` (IP Helper table plus toolhelp namer) behind
  `socket-table`, and the `EtwWatcher` process event stream behind `etw`. A live
  build without `socket-table` falls back to an empty scripted attributor so
  packets are retained unattributed rather than having an owner fabricated (P-4
  permits the first, P-9 forbids the second); a live build without `etw` fails
  naming the missing feature, because with no live process event source no target
  could ever be acquired. The offline path is unchanged: the same replay source,
  scripted attributor, and scripted watcher, and the same usable-backend-absent
  failure when neither offline nor live is present.
- **The live driver is a streaming merged channel, distinct from the offline
  two-phase path.** The offline path folds a pre-collected timeline in two phases
  and stays byte-identical to its committed goldens. A live capture has no
  pre-collected timeline and its packets and process events arrive on separate
  channels, so the live driver merges the counting tee's packets and the ETW
  watcher's events into one totally ordered channel and folds them in arrival
  order. That merge is what lets the run stop on a terminal-stage exit even while
  no further packets arrive: the exit reaches the driver as a merged-channel
  message independent of the packet path, the session leaves its active state,
  and the pipeline is stopped. The pipeline build is factored into one helper both
  drivers call, so the two construct the output path identically.
- **The live path is compiled but has never executed, consistent with the
  project's standing position.** It is compiled under the `--all-features` clippy
  gate and covered only by `#[ignore]`d tier-2 tests, because it needs npcap and
  an elevated ETW session, which continuous integration has neither. Live capture
  has still never run in CI. Recorded for promotion to specification section 29.
- **The socket-table refresh loop is now built through the read/write split the
  S10 design anticipated.** `FlowAttributor::refresh` takes `&mut self`, so an
  attributor shared across the capture threads cannot be refreshed through the
  pointer they hold. The split resolves that: a new platform-neutral
  `PublishedResolver` in `fragcap-attr` is the read side of section 11.6, holding
  the shared published index, the shared refresh schedule, and the clock, and
  answering `resolve` and `active_endpoints` with the exact atomic-load and
  rate-limited-request semantics of `SocketTableAttributor`'s own read path.
  `SocketTableAttributor::resolver()` clones one from an attributor. The CLI's
  live `socket-table` branch builds the mutable attributor, hands the pipeline
  `Arc::new(attributor.resolver())` as the inner attributor, and moves the mutable
  attributor onto a `RefreshDriver` control thread that does one initial refresh
  and then refreshes on the section 11.2 cadence (`wants_refresh`-driven,
  honoring the resolver's triggered requests), so a refresh on the control thread
  is visible to every resolving thread and an unseen-endpoint lookup records a
  request the control thread acts on. The driver is stopped and joined at
  teardown, after the pipeline ends and before the watcher is dropped; it reads
  only the socket table and touches neither the pipeline nor the forwarders, so it
  deadlocks against nothing. This is still compiled-only: it needs npcap and the
  IP Helper socket table on a real machine, so it is exercised solely by
  `#[ignore]`d tier-2 tests and has never run in CI. The one tier-1 addition that
  does run is the `fragcap-attr` unit test proving a resolver answers from the
  index its attributor publishes and that the resolver's own `refresh` is a
  harmless no-op.
