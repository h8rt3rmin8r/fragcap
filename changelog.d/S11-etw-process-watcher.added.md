### Added

- **Process observation.** `fragcap-attr` gains a `ProcessWatcher` backed by an
  ETW kernel session, behind an `etw` feature that is off by default.
  Specification section 10.1. The feature being off is what keeps
  `cargo xtask ci` passing on a machine with no elevation and no Windows.
- **The process tree.** `fragcap-core::process::tree` carries the whole of
  specification section 10.2 as a fold over process events: synthetic
  session-local identifiers that are never reused, resolution by the pair of
  operating system identifier and timestamp, exited nodes retained for the
  session, and ancestry answerable after the entire parent chain has gone. It
  opens nothing and names no platform type, so all of section 10.2 is tested at
  tier 1 on any machine.
- **The chains reconnaissance observed, as tests.** Both focal titles' launcher
  chains from Appendix D replay through a scripted watcher.
  `crates/fragcap-attr/tests/chains.rs` asserts the ESO chain's five levels and,
  for The Division 2, that the three processes sharing the image name
  `TheDivision2.exe` are three distinct nodes told apart by ancestry. This is
  the case specification section 15.4 makes a validation error and section
  10.3's `descends_from` exists for, and it now has a test rather than a
  paragraph.
- **A scripted process watcher.** `fragcap-attr::proc_script` publishes a
  declared sequence of process events, mirroring the scripted attributor S04
  built for the same reason. Not behind any feature, so it works everywhere.
  Both watchers feed one `ProcessTree::apply`, so a test that passes against a
  script states something the ETW watcher must also satisfy.
- **Ancestry provenance.** Every node records whether its parent was observed at
  creation, read from the startup snapshot, or unresolved. Specification section
  5.3 says the first is unambiguous and the second may name an unrelated
  process; carrying the difference is what stops a consumer treating a guess as
  a measurement.
- **A command line is either observed or declared unavailable.** Never an empty
  string standing in for either. A process the startup snapshot finds cannot
  yield one without a memory-read right constitution P-1 forbids, so its absence
  is recorded as an absence.
- **Loss that a packet counter cannot express.** A `WatcherReport` carries the
  events and buffers the kernel itself reported dropping, separately from
  `CaptureStats`, and a tree built while anything was lost reports itself
  incomplete. A lost start event removes a node and orphans everything beneath
  it, which a packet's loss never does.
- **The P-1 claim is mechanical.** `cargo xtask lint` now fails if any fragcap
  source names `PROCESS_VM_READ`, `PROCESS_VM_WRITE`, `PROCESS_VM_OPERATION`, or
  `PROCESS_ALL_ACCESS`, alongside the transmit-call check S09 added. The one
  handle this slice opens asks for `PROCESS_QUERY_LIMITED_INFORMATION` and names
  it literally at the call site.
- **`cargo xtask neutral` builds `fragcap-attr`.** It already built
  `fragcap-core` and, since S09, `fragcap-capture`. The claim that
  `fragcap-attr` builds for a target with no process telemetry backend was
  equally unchecked until now.
