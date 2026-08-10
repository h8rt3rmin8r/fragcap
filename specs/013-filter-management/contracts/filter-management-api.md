# Contract: Filter Management API

The public surface S13 adds to `fragcap-core`, with the postconditions and
invariants each element guarantees. These are what the tests assert.

## `FilterProgram::narrowed(endpoints: &[Endpoint]) -> FilterProgram`

- **Postcondition**: the expression admits exactly the union of `endpoints`. For
  each endpoint it contains a clause constraining protocol (`tcp`/`udp`), host
  address, and port; clauses are ORed.
- **Determinism**: the expression is a pure function of the endpoint set;
  reordering or duplicating the input does not change the output (endpoints are
  sorted and deduplicated).
- **Family**: an endpoint set mixing IPv4 and IPv6 addresses yields a program
  admitting both families.
- **Empty input**: `narrowed(&[])` is `is_empty()`; the manager never installs it.
- **No side effects**: opens nothing, reads no clock, touches no platform surface.

## `FilterConfig`

- **Invariant**: `FilterConfig::PRODUCTION` is `debounce = 2s`,
  `min_reinstall_interval = 5s` (the section 12.2 constants). `Default` is
  `PRODUCTION`.

## `FilterManager::poll(&mut self, wanted: &[Endpoint], now: Instant) -> Vec<Install>`

- **Debounce**: no `Install` is returned until the wanted set has been unchanged
  for at least `config.debounce` (measured from the last observed change to `now`).
  Repeated changes within the window coalesce; the timer resets on each change.
- **Rate limit**: for any handle, two `Install`s are never returned less than
  `config.min_reinstall_interval` apart; an otherwise-due reinstall is deferred to
  a later `poll` and not dropped.
- **Non-empty only**: an `Install`'s program is never empty. A handle whose wanted
  set is empty stays on bootstrap (if never narrowed) or keeps its prior narrowed
  program (once narrowing has begun); no `Install` is returned for it.
- **Idempotence**: if a handle's installed program already matches the wanted set,
  no `Install` is returned for it.
- **Gap accounting**: after a reinstall on a handle whose previously installed
  program was narrowed, `filter_gaps()` increases by the number of endpoints in
  the new set that the previous set did not contain. A bootstrap-to-first-narrowing
  install adds zero, because bootstrap admitted everything.
- **Purity**: `poll` reads only its arguments and its own state; it performs no
  I/O and installs nothing itself.

## `FilterManager::filter_gaps(&self) -> u64`

- **Postcondition**: the running total of gap occurrences per the accounting above.
  Monotonic non-decreasing across `poll` calls.

## `Pipeline::set_filter_config(&mut self, config: FilterConfig)`

- **Postcondition**: the control thread `run` spawns uses `config` for its manager.
  Absent a call, `FilterConfig::PRODUCTION` is used.
- **Compatibility**: adding this changes neither `Pipeline::new`'s signature nor
  `PipelineConfig`, so every existing caller and test compiles unchanged.

## `Pipeline::run` behavior (internal, observable through statistics and a source double)

- **Narrowing**: when `active_endpoints()` yields a non-empty set that has settled
  past the debounce, the control thread sends a narrowed program to each capture
  thread, which installs it via `set_filter`.
- **Attribution independence** (section 12.3): every packet a capture thread reads
  is still parsed and attributed regardless of the installed filter; the filter
  changes which packets arrive, never how an arrived packet is handled.
- **Gap surfacing**: the report's `CaptureStats::filter_gaps` includes the control
  thread's accumulated total; it is distinct from `kernel_dropped`,
  `buffer_dropped`, and `sink_dropped`.
- **Conservation**: for every sink, `received + buffer_dropped + refusals ==
  packets_captured` continues to hold; `filter_gaps` is not part of this identity
  because it counts no packet fragcap observed and discarded.
- **Maintenance failure**: a `set_filter` error on the maintenance path is
  non-fatal; the interface keeps capturing on its prior program and is not retired.
- **Termination**: the control thread exits when the run stops (after the capture
  threads end, `run` signals stop and joins it); it never blocks shutdown and holds
  no producer, so it cannot keep the buffer open.
