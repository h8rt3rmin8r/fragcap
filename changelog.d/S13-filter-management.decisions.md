### Decisions

**2026-08-10. Six decisions taken while implementing S13, recorded for promotion
to specification section 29. Two carry deviation candidates, noted below.**

- **Narrowing reads the attribution map, not a process-tree flow set.** The
  endpoint set comes from `FlowAttributor::active_endpoints`, the seam slice S10
  built for this. Specification section 12.2 names the attribution map as the only
  reliable source; the section 8.6 diagram draws a "flow set" from the process
  tree, and the two denote the same set. **Deviation candidate:** the diagram and
  the prose should be reconciled.
- **`filter_gaps` counts occurrences, not packets.** A packet the kernel filter
  excludes is never delivered to fragcap, so a literal packet count would be
  fabricated, which constitution P-9 forbids. The counter counts endpoints briefly
  excluded by a stale narrowed filter, a set difference computed at each reinstall.
  **Deviation candidate:** section 12.3's prose says "packets," and the unit is
  occurrences.
- **Per-source delivery is a `std::sync::mpsc` channel, not `arc-swap`.** Adding
  `arc-swap` to `fragcap-core` would widen its dependency allowlist from the single
  entry `bytes`, which the dependency check treats as a P-2 guard. The filter slot
  is read between reads, off the per-packet path, so section 11.6's lock-free
  mandate does not extend to it, and a std channel needs no dependency and no lock.
- **The maintenance timings are injectable through a setter.** `FilterConfig`
  carries the section 12.2 constants; `Pipeline::set_filter_config` overrides them
  for tests without changing `Pipeline::new` or `PipelineConfig`, so no existing
  caller or struct-literal construction breaks. The policy takes the current
  instant as a parameter, so it needs no clock abstraction in core.
- **Gap counting is accumulated on the control thread and absorbed by the run.**
  The control thread holds the per-handle installed history the count is computed
  from, so it counts there and returns a `CaptureStats` the run folds in with the
  existing `absorb`, which already sums `filter_gaps`.
- **A maintenance reinstall failure is non-fatal.** A `set_filter` rejection during
  phase three keeps the prior program and continues capturing, because correctness
  never depends on filter freshness and retiring the interface would lose all its
  later traffic to spare a failed optimization. It advances no drop counter. The
  program is generated from a fixed grammar, so this path is defensive; a bootstrap
  rejection at open still retires, which is existing S09 behavior.

**2026-08-10, in review of pull request 17. Four code findings fixed, two recorded
as required follow-up.** An automated review raised six findings against the first
commit.

- **A wildcard bind drops the host constraint.** A UDP socket reported bound to
  `0.0.0.0` or `::` was compiled as `host 0.0.0.0`, which matches no real packet,
  so the first narrowing would silently exclude that socket's whole traffic while
  recording no gap. Such a bind now admits by protocol and port alone.
- **A filter gap is counted when it begins.** Gap accounting ran only at a
  reinstall, so an endpoint excluded during the debounce or rate-limit window that
  then closed, or that was still excluded when capture ended, went uncounted. Gaps
  are now counted the first poll an endpoint is excluded by the installed program,
  once per episode, independent of any later reinstall.
- **A retired handle stops accruing gaps and installs.** With begin-time gap
  counting, a handle whose capture thread ended would otherwise fabricate a gap for
  every new endpoint against its frozen program. `FilterManager::retire`, called
  when the control thread can no longer reach a capture thread, stops both.
- **A control-thread panic propagates.** The control thread's join swallowed a
  panic, which could present a defect as a completed capture. It is now carried to
  the caller after orderly shutdown, the same contract the acquisition threads have.
- **Follow-up, not fixed here: narrowing is not yet restricted to profiled
  processes.** `FlowAttributor::active_endpoints` returns every socket-table
  endpoint, not only those owned by profiled processes, because the pipeline has no
  access to the S11/S12 process-tree stage bindings and `active_endpoints` has
  dropped the owning process identifier. Restricting the narrowing input to
  profiled endpoints is the session-to-pipeline integration that S12 deferred to
  S13 and S14; it is required before the live backend narrows correctly and is
  recorded as a **section 29 open item**. The filter-management machinery and its
  tier-1 verification do not depend on it, because the scripted attributor supplies
  a controlled endpoint set.
- **Follow-up, not fixed here: the attribution snapshot is not refreshed in the
  pipeline.** `FlowAttributor::refresh` takes `&mut self` and cannot be called
  through the `Arc<dyn FlowAttributor>` that section 11.6 requires for lock-free
  `resolve`, so no thread refreshes the socket table during a run. Driving the
  periodic refresh from the control thread (section 8.6) needs a `refresh(&self)`
  trait signature, which is a **section 29 deviation** to be taken with its own
  change rather than rushed here; every `FlowAttributor` implementor changes with
  it. Pre-existing (the pipeline never refreshed), surfaced by S13's control
  thread, which is the natural owner of the driven refresh once the signature
  allows it.
