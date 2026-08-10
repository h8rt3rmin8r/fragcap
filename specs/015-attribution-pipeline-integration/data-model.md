# Data Model: Attribution Session-to-Pipeline Integration

**Slice**: 015 | **Date**: 2026-08-10

This slice changes one trait (the architecture-of-record `FlowAttributor` seam)
and adds one small value type. No stored data, no schema.

## `OwnedEndpoint` (new, `fragcap-core::flow`)

An active endpoint paired with the process identifier that owns it, if known.

```text
pub struct OwnedEndpoint {
    pub endpoint: Endpoint,     // existing type: addr + proto
    pub owner: Option<u32>,     // owning PID, None when the source cannot supply it
}
```

- `owner` is `None` for sources that do not track ownership (the scripted
  attributor, stubs) and `Some(pid)` for the socket-table attributor.
- Derives: `Clone, Copy, Debug, PartialEq, Eq`. Constructor helper
  `OwnedEndpoint::unowned(endpoint)` for the default trait path.
- Rationale: `active_endpoints() -> Vec<Endpoint>` has already dropped the PID
  the socket table carried, so the profiled-process join needs a carrier that
  keeps it. Kept minimal (no name, no role) because narrowing needs only the
  endpoint and its owner.

## `FlowAttributor` trait contract (changed, `fragcap-core::traits`)

The seam is architecture of record; the change is a specification section 29
deviation recorded with a dated decision fragment. It stays dyn-compatible and
`Send + Sync` (the compile-time assertions in `traits.rs` continue to pass).

| Method | Before | After | Notes |
|--------|--------|-------|-------|
| `resolve(&self, key, at)` | unchanged | unchanged | wait-free read of the published snapshot |
| `refresh` | `&mut self -> Result<(), AttrError>` | `&self -> Result<(), AttrError>` | callable through `Arc<dyn FlowAttributor>` |
| `wants_refresh` | (none) | `&self -> bool` default `false` | control thread gates refresh; keeps schedule type out of core |
| `active_endpoints` | `&self -> Vec<Endpoint>` | unchanged | pipeline reads this (now profiled-filtered by the decorator) |
| `active_endpoints_owned` | (none) | `&self -> Vec<OwnedEndpoint>` default maps `active_endpoints()` to owner `None` | decorator consumes this to filter by profiled PID |

Contract invariants:

- `refresh(&self)` must not lock the resolve path. An implementor with
  refresh-mutable state guards only that state (section 11.6, SC-003).
- `wants_refresh()` default `false` means an implementor with nothing to refresh
  is never asked to; the control thread calls `refresh` only when it returns
  true.
- `active_endpoints_owned()` default preserves existing behavior for every
  implementor that does not override it (owner `None`), so a consumer that does
  not filter by owner sees the same endpoints as before.

## Implementor matrix

| Implementor | `refresh` | `wants_refresh` | `active_endpoints_owned` | `active_endpoints` |
|-------------|-----------|-----------------|--------------------------|--------------------|
| `SocketTableAttributor` (`fragcap-attr`) | `&self`, locks `Mutex<RefreshState>`, publishes index | is_due or is_requested (promoted inherent) | overrides: PID per endpoint from the index | unchanged (all) |
| `PublishedResolver` (`fragcap-attr`) | `&self` no-op | default false | default (owner None) | unchanged |
| `ScriptedAttributor` (`fragcap-attr`) | `&self` no-op | default false | default (owner None) | unchanged (script endpoints) |
| `RoleStampingAttributor` (`fragcap` facade) | `&self` forwards to inner | forwards to inner | default | overrides: filters `inner.active_endpoints_owned()` to profiled PIDs |
| `StubAttributor` (`traits.rs`, `pipeline`) | `&self` | default / test-set | default | unchanged |
| `Fixed` (`session.rs` test) | `&self` | default false | default | unchanged |
| `PanicOnEndpoints` (`pipeline` test) | `&self` | default false | default/panic as designed | unchanged |

## `SocketTableAttributor` internal shape (changed, `fragcap-attr::socket`)

```text
pub struct SocketTableAttributor {
    refresh_state: Mutex<RefreshState>,   // source, namer, retained  (refresh-only)
    clock: Arc<dyn Clock>,
    config: AttributorConfig,
    published: Arc<PublishedIndex>,       // arc-swap, read wait-free by resolve
    schedule: Arc<RefreshSchedule>,       // atomics
}

struct RefreshState {
    source: Box<dyn SocketTableSource>,
    namer: Box<dyn ProcessNamer>,
    retained: RetentionMap,
}
```

- `resolve`, `wants_refresh`, `active_endpoints`, `active_endpoints_owned` read
  only `published` / `schedule` / `clock`; they never lock `refresh_state`, which
  is what preserves the lock-free resolve path.
- `refresh(&self)` locks `refresh_state`, performs the identical table
  read / naming / retention-aging logic, then publishes the new
  `AttributionIndex`.

## `AttributionIndex::endpoints_owned` (new, `fragcap-attr::index`)

```text
pub fn endpoints_owned(&self, at: Timestamp) -> Vec<OwnedEndpoint>
```

Mirrors `endpoints(at)` but pairs each endpoint with its owning PID from the
table entry / retained record. Sorted deterministically like `endpoints`.

## Profiled endpoint set (derived, `fragcap` facade)

Not a stored type. `RoleStampingAttributor::active_endpoints()` computes it as:
the subset of `inner.active_endpoints_owned()` whose `owner` is a key in the
`BindingPublisher` snapshot (`BindingMap`, keyed by stage-bound PID). Empty
snapshot -> empty profiled set -> bootstrap filter retained.
