# Phase 1 Data Model: Socket Table Attributor

**Slice**: S10 | **Date**: 2026-08-09 | **Spec**: [spec.md](spec.md)

Everything here lives in `fragcap-attr`. Nothing is added to `fragcap-core`
except the `Sync` bound on one trait, because a socket table is a platform
concept and P-2 keeps core free of them.

## The immutable half

These are values. They have no clock, no I/O, and no interior mutability, and
every matching rule is a method on one of them.

### `SocketTableEntry`

One row of the platform's table, normalized.

| Field | Type | Notes |
| --- | --- | --- |
| `proto` | `Proto` | From `fragcap-core` |
| `local` | `SocketAddr` | The bind address and port |
| `remote` | `Option<SocketAddr>` | `Some` for TCP only. Section 8.4 forbids inventing one for UDP, so the type cannot carry one |
| `pid` | `u32` | Owning process identifier |
| `created` | `Option<Timestamp>` | The socket creation instant, when the platform reports one |

`remote` being `Option` rather than the protocol implying it is deliberate: a
TCP listening socket has no peer either, and a constructor that derived the
field from `proto` would have to invent one.

### `SocketTable`

A whole snapshot, as an immutable value: a `Vec<SocketTableEntry>` plus the
instant it was taken. Constructible from declared entries, which is what makes
every rule below testable with no platform (FR-004).

### `ProcessNames`

`HashMap<u32, Arc<str>>`, resolved during refresh for the identifiers the table
reported (FR-033a). Shared strings, because a name repeats across every socket
a process holds and again on every packet of every flow it owns.

### `RetentionMap`

`HashMap<Endpoint, RetainedEntry>`, where a `RetainedEntry` carries the owner,
the process identifier, the socket's creation instant, and `last_seen`, the
instant the endpoint was last observed present in a table. FR-018a measures the
grace period from `last_seen` and not from the refresh that noticed the
absence.

### `AttributionIndex`

The published value. A `SocketTable`, a `ProcessNames`, and a `RetentionMap`.
This is the whole of what a lookup may read (SC-015): if an answer needs it, it
is in here before the lookup begins.

Carries the two lookup entry points:

- `resolve(&self, key: &FlowKey, at: Timestamp) -> Option<Attribution>`
- `endpoints(&self, at: Timestamp) -> Vec<Endpoint>`

### `Match` and the ordering

Matching produces a `Candidate` carrying the entry and its `MatchRank`:

| Rank | Meaning |
| --- | --- |
| `BothEndpoints` | TCP, local and remote both equal |
| `ExactLocal` | Local address and port equal |
| `WildcardBind` | Bind address unspecified, same family, port equal |
| `DualStack` | IPv6 unspecified bind, IPv4 local endpoint, port equal |

The ranks are ordered most exact first (FR-008), and they are mutually
exclusive by construction: each is tested in order and the first that holds
fixes the rank. Within a rank, candidates are ordered by `created`, latest
first, with `None` sorting last (FR-008a), then by ascending `pid` (FR-008b).
The last is arbitrary and exists only so the order is total; a comment says so,
because an arbitrary rule with no explanation invites someone to make it
meaningful.

An entry whose `created` is later than the packet's instant is not a candidate
at all (FR-009), which is a filter before ranking rather than a term in it.

## The mutable half

### `Clock`

```text
trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}
```

Two implementations: `SystemClock`, and `TestClock`, which returns a declared
instant and can be advanced. `TestClock::set` and `advance` take `&self` over
an atomic, because the clock is held as `Arc<dyn Clock>` and a test that could
not advance it through a shared handle could not drive the cadence at all.

Scoped to this crate deliberately (CHK037): it exists because sections 11.2 and
11.4 are otherwise untestable at tier 1, not as a workspace-wide abstraction.

### `SocketTableSource`

```text
trait SocketTableSource: Send {
    fn read(&mut self) -> Result<SocketTable, AttrError>;
}
```

Two implementations: `DeclaredTable`, which returns declared contents and can
be scripted to fail, and `IpHelperTable` behind the platform feature.

### `ProcessNamer`

```text
trait ProcessNamer: Send {
    fn names(&mut self, pids: &[u32]) -> HashMap<u32, Arc<str>>;
}
```

Takes the identifiers the table reported, so the default implementation
enumerates once rather than per identifier. Two implementations: `DeclaredNames`
for tests, and `ToolhelpNamer` behind the platform feature. Returning a map
rather than a `Result` is deliberate: a name that cannot be resolved is a
missing name, not a failure, and FR-032 requires the attribution be produced
anyway.

### `PublishedIndex`

```text
struct PublishedIndex(ArcSwap<AttributionIndex>);

impl PublishedIndex {
    fn load(&self) -> Arc<AttributionIndex>;
    fn publish(&self, index: AttributionIndex);
}
```

`Send + Sync`, shared behind an `Arc`. This is the section 11.6 mechanism and
the reason it is a separate type rather than a private field is in research.md
R-5: a test of concurrent resolution across a publication needs to publish from
one thread while others read, which a `&mut self` method on a shared object
cannot express, and S13's control thread will need the same seam.

### `RefreshSchedule`

Interior mutability over atomics, shared behind an `Arc` so that a lookup on
one thread can record a request the owner reads on another.

| Field | Purpose |
| --- | --- |
| `last_refresh` | When the last refresh completed |
| `last_request` | When the last rate-limited request was recorded |
| `requested` | Whether a refresh is pending |

Methods: `is_due(now, interval)`, `request_triggered(now, limit)` for FR-014
and FR-015, `request_immediate()` for FR-013 and FR-016, and `take_request()`
for the owner.

`request_triggered` returning a bool that says whether the request was recorded
is what makes the rate limit observable in a test (SC-005) without reading
private state.

### `AttributorConfig`

`interval` defaulting to one second, `retention` to thirty seconds,
`trigger_limit` to two hundred milliseconds (FR-011, FR-011a, FR-015, FR-018).
Plain values on this struct, not profile keys; the reason is in the spec's
clarifications.

### `SocketTableAttributor`

The owner. Holds the config, the source, the namer, the clock, an
`Arc<PublishedIndex>`, an `Arc<RefreshSchedule>`, and the retention map it
carries forward between refreshes.

Implements `FlowAttributor`:

- `resolve` loads the published index, matches, and on an unresolved lookup
  against an endpoint the index does not carry, reads the injected clock and
  records a triggered request against it. It reads the socket table never,
  enumerates nothing, and opens no handle (SC-015, FR-017). The clock read is
  confined to that path: a resolved lookup touches only the index.
- `refresh` reads a table, ages the retention map against the new table,
  resolves names for the identifiers present, builds an `AttributionIndex`, and
  publishes it. On a read failure it leaves the published index alone and
  returns the error (FR-030).
- `active_endpoints` loads the index and reports current plus retained, against
  the injected clock's instant because the trait method carries none (FR-023).
  That is the right instant here and the wrong one in `resolve`: "currently
  active" is a question about now, and "who owned this flow" is always a
  question about then.

`refresh` taking `&mut self` is the trait's shape and is kept. The attributor
is owned by whoever drives the cadence; the pipeline holds it as
`Arc<dyn FlowAttributor>` and calls only the `&self` methods, which is what it
does today.

## What changes in `fragcap-core`

One line, plus the pipeline's consequence.

- `FlowAttributor: Send` becomes `FlowAttributor: Send + Sync`. Both existing
  implementors already satisfy it.
- `Pipeline` holds `Arc<dyn FlowAttributor>` in place of
  `Arc<Mutex<Box<dyn FlowAttributor>>>`, and the per-packet lock in
  `capture_loop` goes. `Pipeline::new` keeps its `Box<dyn FlowAttributor>`
  parameter, because `Arc<dyn T>` is constructible from it and no caller need
  change.

## Entity map

```text
SocketTableSource ──read──▶ SocketTable ─┐
ProcessNamer ─────names──▶ ProcessNames ─┼─▶ AttributionIndex ──▶ PublishedIndex
RetentionMap ────aged────▶ RetentionMap ─┘         ▲                    │
                                                   │                  load
SocketTableAttributor ─────refresh─────────────────┘                    │
        │                                                               ▼
        └──────────────────resolve───────────────────────────────▶ Attribution?
```
