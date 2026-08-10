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
