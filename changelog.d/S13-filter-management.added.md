### Added

- **Kernel filter narrowing.** `fragcap-core::filter` gains
  `FilterProgram::narrowed`, which compiles a set of endpoints into a libpcap
  expression admitting only those endpoints, across IPv4 and IPv6. Specification
  section 12.2, phase two. It is a pure function over core types, so the whole
  strategy is tested at tier 1 with no capture driver.
- **The maintenance policy.** `FilterManager` runs specification section 12.2's
  phase three: it debounces recompilation by two seconds and rate limits
  reinstallation to one per five seconds per handle, coalescing the endpoint churn
  of connection establishment. It is a pure decision over a wanted endpoint set
  and a supplied instant, tested against synthetic instants.
- **The control thread.** `Pipeline::run` now spawns the section 8.6 control
  thread's filter manager: it reads the attribution map's `active_endpoints`
  (slice S10), narrows the filter, and hands each capture thread its current
  program over a private channel, which the capture thread installs on its own
  handle. `Pipeline::set_filter_config` overrides the section 12.2 timings for
  tests without changing any existing caller.
- **Filter gaps are counted and surfaced.** `CaptureStats::filter_gaps` is
  populated per specification section 12.3: an endpoint briefly excluded by a
  stale narrowed filter is counted as a gap occurrence, distinct from the three
  drop counters and outside the pipeline conservation identity, because it counts
  no packet fragcap observed and discarded.
- **Filter-lifecycle glossary entries.** `docs/glossary.md` gains `Narrowing`,
  `Maintenance`, `Filter program`, `Filter manager`, and `Filter gap`, the last
  resolving a dangling reference the `Bootstrap filter` entry already carried
  (constitution P-6).
