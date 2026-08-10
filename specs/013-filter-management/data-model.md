# Data Model: Filter Management

The public and internal types this slice adds or grows, all in `fragcap-core`.
Signatures are the contract; doc comments show intent.

## `fragcap-core::filter` (grown)

```rust
/// A capture filter to be installed on a `PacketSource`. (Existing type, grown.)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FilterProgram { /* expression: String */ }

impl FilterProgram {
    // Existing: new, expression, is_empty, Default.

    /// Compile a narrowed program admitting exactly `endpoints`, as the OR of one
    /// clause per endpoint (protocol, host, port), spanning IPv4 and IPv6. The
    /// endpoints are sorted so the expression is deterministic; duplicates
    /// collapse. An empty slice yields an empty program (`is_empty()`), which the
    /// filter manager never installs (it keeps bootstrap or the prior program).
    pub fn narrowed(endpoints: &[Endpoint]) -> Self;
}
```

The bootstrap expression (`ip or ip6`) is not produced here; it stays the single
literal `BOOTSTRAP_FILTER` in `fragcap-capture`, installed by S09 at open. Core
only ever produces narrowed programs.

## `fragcap-core::filter::FilterConfig`

```rust
/// The maintenance timings of specification section 12.2. Plain values, not
/// operator knobs (S14 owns any command line); carried so tests can override the
/// production constants, mirroring `fragcap-attr`'s `AttributorConfig`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterConfig {
    /// Recompilation debounce: the wanted set must be stable this long before a
    /// reinstall. Section 12.2.
    pub debounce: Duration,
    /// Rate limit: at most one reinstall per handle per this interval. Section
    /// 12.2.
    pub min_reinstall_interval: Duration,
}

impl FilterConfig {
    /// The section 12.2 constants: two-second debounce, one reinstall per five
    /// seconds per handle.
    pub const PRODUCTION: FilterConfig = FilterConfig {
        debounce: Duration::from_secs(2),
        min_reinstall_interval: Duration::from_secs(5),
    };
}

impl Default for FilterConfig { /* PRODUCTION */ }
```

## `fragcap-core::filter::FilterManager`

```rust
/// The phase-two and phase-three policy of specification section 12.2, as a pure
/// decision over a wanted endpoint set and a supplied instant. Opens nothing,
/// installs nothing, reads no clock of its own: the control thread drives it and
/// performs the installs it returns. Counts filter gaps (section 12.3).
pub struct FilterManager { /* config, per-handle state, debounce state, gaps */ }

/// One program to install on one handle, returned by `poll`.
pub struct Install {
    pub handle: usize,
    pub program: FilterProgram,
}

impl FilterManager {
    /// A manager for `handle_count` handles (one per capture thread), all
    /// starting in bootstrap.
    pub fn new(handle_count: usize, config: FilterConfig) -> Self;

    /// Feed the current wanted endpoint set and the current instant. Returns the
    /// programs to install now: none until the wanted set has been stable for
    /// `debounce`, at most one per handle per `min_reinstall_interval`, and never
    /// an empty program (a handle stays on bootstrap or its prior program when the
    /// wanted set is empty). Accumulates a filter gap for each endpoint newly
    /// admitted by a reinstall that a previously narrowed program excluded;
    /// bootstrap-to-first-narrowing records none.
    pub fn poll(&mut self, wanted: &[Endpoint], now: Instant) -> Vec<Install>;

    /// Total filter gaps observed so far, for the capture statistics.
    pub fn filter_gaps(&self) -> u64;
}
```

Internal per-handle state (not public):

```rust
enum Installed { Bootstrap, Narrowed(BTreeSet<Endpoint>) }
struct HandleState { installed: Installed, last_install: Option<Instant> }
```

`Endpoint` (`fragcap-core::flow`) gains `Ord`/`PartialOrd` derives if absent, so it
can key a `BTreeSet` for deterministic compilation and set-difference gap counting.
`Proto` likewise. These are plain value types; the derive adds no behavior.

## `fragcap-core::pipeline` (grown)

```rust
impl Pipeline {
    /// Override the filter maintenance timings (default `FilterConfig::PRODUCTION`).
    /// A setter rather than a `new` parameter or a `PipelineConfig` field so no
    /// existing caller or struct-literal construction changes.
    pub fn set_filter_config(&mut self, config: FilterConfig);
}
```

`Pipeline::run` internally, unchanged in signature:

- Creates one `std::sync::mpsc::channel::<FilterProgram>()` per source; each
  capture thread receives its `Receiver`, the control thread holds the `Sender`s.
- Spawns a control thread holding an `Arc<dyn FlowAttributor>` clone, the
  `Sender`s, a `FilterManager::new(sources.len(), filter_config)`, and a
  `StopHandle`. It loops while not stopped: reads `active_endpoints()`, calls
  `poll(&wanted, Instant::now())`, sends each `Install`'s program to its handle's
  channel, and sleeps a short tick. On exit it returns
  `CaptureStats { filter_gaps: manager.filter_gaps(), ..Default::default() }`.
- After joining the capture threads, signals stop, joins the control thread, and
  `absorb`s its `CaptureStats` into the merged report (folding `filter_gaps`).

`acquire` (internal free function) gains a `filter_rx: Receiver<FilterProgram>`
parameter. At the top of each loop iteration it drains the receiver to the latest
program and, if one arrived, calls `source.set_filter(&program)`. A `set_filter`
error during this maintenance path is non-fatal: the loop keeps the prior program
and continues (it does not retire the interface, unlike a `next_packet` failure).

## `fragcap-core::stats` (refined)

The existing `CaptureStats::filter_gaps` field keeps its type and its summation in
`absorb`; its doc comment is refined from "Packets that passed while a filter was
being narrowed" to the section 12.3 definition: the count of gap occurrences,
endpoints briefly excluded by a stale narrowed filter, distinct from the three
drop counters and never a fabricated kernel-excluded packet count (P-9).
