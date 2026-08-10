# Contract: The Attribution Surface After S10

**Slice**: S10

**Date**: 2026-08-09

**Phase**: 1

This slice is a library slice, so its contract is the public Rust surface other
crates and later slices compile against. One existing declaration moves, and
`fragcap-attr` grows a module. Signatures are given in the shape they are
intended to land; the narrative reasons are in [plan.md](plan.md) and
[data-model.md](data-model.md).

## 1. The behavioral seam, `fragcap-core::traits`

```rust
pub trait FlowAttributor: Send + Sync {
    fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution>;
    fn refresh(&mut self) -> Result<(), AttrError>;
    fn active_endpoints(&self) -> Vec<Endpoint>;
}
```

Only the bound changes. The method set is untouched, for the reason S09 gave
when it changed `PacketSource`: section 8.5 intends this surface to reach 1.0.0,
and a bound every existing implementor already satisfies is a far smaller
commitment than a method.

**Compatibility**: source-breaking for any implementor that is not `Sync`.
Both implementors in the workspace, `ScriptedAttributor` and the test stubs in
`traits.rs` and `pipeline/mod.rs`, hold plain data and already are.

**Recorded as a deviation.** Section 8.5 declares the trait with neither bound.

## 2. The pipeline, `fragcap-core::pipeline`

```rust
impl Pipeline {
    pub fn new(
        sources: Vec<SourceBinding>,
        attributor: Box<dyn FlowAttributor>,
        config: PipelineConfig,
    ) -> Result<Self, ConfigError>;
}
```

Unchanged. What changes is internal: the pipeline stored
`Arc<Mutex<Box<dyn FlowAttributor>>>` and took the lock once per packet on the
attribution path. It now stores `Arc<dyn FlowAttributor>` and takes nothing.

**Compatibility**: none broken. `Arc<dyn T>` is constructible from
`Box<dyn T>`, so the conversion happens inside `run` and no caller changes.

## 3. `fragcap-attr::table`, the immutable half

```rust
pub struct SocketTableEntry {
    pub proto: Proto,
    pub local: SocketAddr,
    pub remote: Option<SocketAddr>,
    pub pid: u32,
    pub created: Option<Timestamp>,
}

pub struct SocketTable { /* entries, taken_at */ }

impl SocketTable {
    pub fn new(taken_at: Timestamp, entries: Vec<SocketTableEntry>) -> Self;
    pub fn entries(&self) -> &[SocketTableEntry];
    pub fn taken_at(&self) -> Timestamp;
}
```

`remote` is `Option` on the entry rather than implied by `proto`, because a
listening TCP socket has no peer and a derived field would have to invent one.
There is deliberately no constructor that takes a remote for a UDP entry.

## 4. `fragcap-attr::index`, what a lookup reads

```rust
pub struct AttributionIndex { /* table, names, retained */ }

impl AttributionIndex {
    pub fn resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution>;
    pub fn endpoints(&self, at: Timestamp) -> Vec<Endpoint>;
    pub fn carries(&self, endpoint: Endpoint) -> bool;
}

pub struct PublishedIndex(/* private */);

impl PublishedIndex {
    pub fn new(index: AttributionIndex) -> Self;
    pub fn load(&self) -> Arc<AttributionIndex>;
    pub fn publish(&self, index: AttributionIndex);
}
```

`AttributionIndex` is the whole of what a lookup may consult. Nothing on this
type performs I/O, takes a lock, or reads a clock; `at` is the packet's instant,
supplied by the caller.

`PublishedIndex` is `Send + Sync` and shared behind an `Arc`. It is public
because S13's control thread needs it and because SC-006 cannot be tested
without publishing from one thread while others read.

`carries` exists for FR-014: an unresolved lookup must be able to tell whether
the endpoint was absent from the index, which is what distinguishes a trigger
worth recording from a flow that is simply not fragcap's.

## 5. `fragcap-attr::socket`, the attributor

```rust
pub struct AttributorConfig {
    pub interval: Duration,
    pub retention: Duration,
    pub trigger_limit: Duration,
}

impl Default for AttributorConfig { /* 1s, 30s, 200ms */ }

pub struct SocketTableAttributor { /* private */ }

impl SocketTableAttributor {
    pub fn new(
        source: Box<dyn SocketTableSource>,
        namer: Box<dyn ProcessNamer>,
        clock: Arc<dyn Clock>,
        config: AttributorConfig,
    ) -> Self;

    pub fn published(&self) -> Arc<PublishedIndex>;
    pub fn schedule(&self) -> Arc<RefreshSchedule>;
    pub fn config(&self) -> &AttributorConfig;
}

impl FlowAttributor for SocketTableAttributor { /* ... */ }
```

Construction takes every seam explicitly and defaults none of them. That is the
same argument `Attribution::new` makes about fidelity: a default is an
inference, and an attributor that silently defaulted its clock to the system
clock would make a test that meant to control time pass while measuring
nothing.

## 6. `fragcap-attr::seam`, the injectable halves

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

pub trait SocketTableSource: Send {
    fn read(&mut self) -> Result<SocketTable, AttrError>;
}

pub trait ProcessNamer: Send {
    fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>>;
}
```

`ProcessNamer::names` returns a map rather than a `Result`. A name that cannot
be resolved is a missing name and not a failure, and FR-032 requires the
attribution be produced carrying the observed identifier regardless. Making it
fallible would create a path on which an observation is discarded because a
convenience could not be supplied, which is what P-9 forbids.

Test implementations ship in the crate rather than behind `#[cfg(test)]`:
`SystemClock`, `TestClock`, `DeclaredTable`, and `DeclaredNames`. S13 and S14
will need to drive an attributor without a platform, exactly as S04's
`ScriptedAttributor` is a public type for the same reason.

## 7. `fragcap-attr::schedule`

```rust
pub struct RefreshSchedule { /* private, atomics */ }

impl RefreshSchedule {
    pub fn is_due(&self, now: Timestamp, interval: Duration) -> bool;
    pub fn request_triggered(&self, now: Timestamp, limit: Duration) -> bool;
    pub fn request_immediate(&self);
    pub fn take_request(&self) -> bool;
    pub fn mark_refreshed(&self, now: Timestamp);
}
```

`request_triggered` returns whether the request was recorded rather than
silently dropping it, which is what makes the two hundred millisecond rate
limit observable in a test without reading private state.

`Send + Sync`, shared behind an `Arc`, because FR-014's trigger is recorded on
the acquisition thread and read by whoever drives the cadence.

## 8. `fragcap-attr::platform`, behind the `socket-table` feature

```rust
pub struct IpHelperTable { /* private */ }
impl SocketTableSource for IpHelperTable { /* ... */ }

pub struct ToolhelpNamer { /* private */ }
impl ProcessNamer for ToolhelpNamer { /* ... */ }
```

Present only when the feature is enabled and the target is Windows. Absent
otherwise, rather than stubbed into something that compiles and reports
fabricated contents (FR-036).

## What this contract does not add

- No role and no stage on any attribution this slice produces. S12 owns both.
- No profile keys. The cadence configuration is plain values, for the reason in
  the spec's clarifications.
- No control thread. S13 owns it, and `published()` plus `schedule()` are the
  seam it will attach to.
- No process handle, of any kind, with any rights.
