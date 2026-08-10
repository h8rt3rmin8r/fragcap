# Research: Attribution Session-to-Pipeline Integration

**Slice**: 015 (follow-up to S13; issues #18, #19)
**Date**: 2026-08-10

This records the design decisions taken under autopilot, each evaluated against
the architecture of record and the constitution. Grounded in a full read of the
affected files (`traits.rs`, `socket.rs`, `index.rs`, `schedule.rs`,
`resolver.rs`, `scripted.rs`, `pipeline/mod.rs`, `session.rs`, `assemble.rs`).

## D-1: The trait surface of the deviation (three deltas, all in section 29)

**Decision**: `FlowAttributor` changes in three ways, recorded together as one
dated specification section 29 deviation:

1. `fn refresh(&mut self) -> Result<(), AttrError>` becomes `fn refresh(&self)
   -> Result<(), AttrError>`. Issue #19's explicit ask; it is what lets the
   pipeline control thread drive refresh through the shared `Arc<dyn
   FlowAttributor>`.
2. Add `fn wants_refresh(&self) -> bool { false }` (default false). The section
   8.6 control thread lives in `fragcap-core`, which cannot depend on
   `fragcap-attr` where the section 11.2 `RefreshSchedule` lives (P-2 / section
   8.3). A default-false trait method lets the control thread ask "is a refresh
   due?" without `fragcap-core` naming the schedule type. `SocketTableAttributor`
   promotes its existing inherent `wants_refresh`; decorators forward to inner;
   scripted/stub/resolver take the default.
3. Add `fn active_endpoints_owned(&self) -> Vec<OwnedEndpoint>` with a default
   that maps `active_endpoints()` to owner `None`. Narrowing to profiled
   processes (issue #18) requires the owning PID, which `active_endpoints() ->
   Vec<Endpoint>` has dropped. `OwnedEndpoint` is a new small value type in
   `fragcap-core::flow`. `SocketTableAttributor` overrides it to carry the PID;
   the role-stamping decorator consumes it to filter.

**Rationale**: Each delta is the minimal seam for one required behavior, and a
default on the two new methods means the scripted attributor, the stub doubles,
and `PublishedResolver` compile unchanged in behavior (they never refresh and
their endpoints are already "all profiled" by test construction). The trait
stays dyn-compatible and `Send + Sync` (the existing compile-time assertions in
`traits.rs` continue to hold).

**Alternatives considered**:
- A single `drive_refresh(&self) -> Result<bool, AttrError>` folding cadence and
  refresh into one call. Rejected: it duplicates the still-needed `refresh(&self)`
  (tests and the CLI call refresh directly) and hides the cadence decision from
  the control thread that section 8.6 assigns it to.
- Changing `active_endpoints()`'s return type to carry the owner. Rejected: it
  breaks every caller and both offline callers that only want endpoints, for a
  need that only the narrowing path has. A defaulted second method is smaller.
- Passing the profiled-PID set into a new `active_endpoints(profiled)` on the
  trait and filtering in the pipeline. Rejected: the pipeline (`fragcap-core`)
  has no profiled set; it lives in the session. Filtering belongs in the
  decorator that already holds the binding snapshot (D-3).

## D-2: Interior mutability for refresh (single Mutex over the refresh-only state)

**Decision**: `SocketTableAttributor`'s refresh-mutable fields, `source: Box<dyn
SocketTableSource>`, `namer: Box<dyn ProcessNamer>`, and `retained:
RetentionMap`, move together behind one `Mutex<RefreshState>`. `refresh(&self)`
locks it, does the table read / naming / retention aging exactly as today, then
publishes the new `AttributionIndex` to the existing `arc-swap` cell.

**Rationale**: All three fields are touched only by `refresh`. `resolve`,
`wants_refresh`, and `active_endpoints` read only `published` (arc-swap),
`schedule` (atomics), and `clock`, none of which the Mutex covers, so the
per-packet resolve path stays wait-free and lock-free (section 11.6, SC-003).
Only one thread (the control thread) ever calls `refresh`, so the Mutex is
uncontended in practice; it exists to satisfy `&self`, not to arbitrate
concurrent refreshes. The scope is broader than the spec's shorthand "retention
map" because `namer.names(&mut self)` and the sequencing `source.read()` are
also `&mut`; the whole refresh-only set moves together.

**Alternatives considered**:
- Fold the retained state into the published `AttributionIndex` and drop the
  separate map. Rejected for this slice: it entangles retention aging with
  publication and is a larger change than wrapping the existing fields; the
  Mutex keeps the refresh logic byte-for-byte identical, which the existing
  socket.rs tests then still cover unchanged.
- A hand-rolled `AtomicPtr` reclamation scheme. Rejected: it adds `unsafe` to a
  workspace that has none outside a platform binding, for no gain over the
  arc-swap already present (E-b).
- `RwLock`. Rejected: refresh has a single writer and no concurrent readers of
  the guarded state, so a plain `Mutex` is the smaller commitment.

## D-3: Where the profiled filter runs (the role-stamping decorator)

**Decision**: `RoleStampingAttributor::active_endpoints()` filters to profiled
endpoints. It already holds the `BindingPublisher` snapshot, whose `BindingMap`
keys are exactly the stage-bound (profiled) PIDs. It calls
`inner.active_endpoints_owned()` and keeps only endpoints whose owner PID is a
key in the snapshot. The pipeline control thread is unchanged: it still calls
`attributor.active_endpoints()`, now on the decorator, which returns the profiled
set.

**Rationale**: The decorator is the one place already above both `fragcap-attr`
(the socket table) and the session (the profiled set), and it already joins the
two by PID for role stamping. Filtering there means the pipeline needs no
profiled-set input and no change, and stubs/offline are unaffected because the
scripted attributor's owned-endpoints default returns owner `None` and the
offline stamper's snapshot is whatever the session bound (the offline goldens do
not narrow a kernel filter at all). When nothing is bound yet the snapshot is
empty, so the profiled set is empty and narrowing yields nothing, which keeps the
S13 bootstrap filter until the first profiled endpoint (spec edge case).

**Alternatives considered**:
- Filtering inside `SocketTableAttributor`. Rejected: it does not hold the
  profiled set; the session does, and reaching it into `fragcap-attr` would
  invert P-3.
- Recomputing stage matching in the pipeline. Rejected: the session owns stage
  matching (S12); the pipeline consumes its published result (P-3).

## D-4: Driving refresh from the pipeline control thread; removing RefreshDriver

**Decision**: The section 8.6 control thread in `pipeline/mod.rs` (`Pipeline::run`,
the existing control closure) gains, at the top of its loop, before it reads
`active_endpoints()`:

```text
if attributor.wants_refresh() { let _ = attributor.refresh(); }
```

The CLI's `RefreshDriver` (`assemble.rs`, `#[cfg(all(feature = "socket-table",
windows))]`), its `REFRESH_POLL_INTERVAL`, the `refresh_driver` field, and its
stop call in `orchestrator.rs` are removed. `live_components` wraps the real
`SocketTableAttributor` directly (`RoleStampingAttributor::new(Arc::new(attributor))`)
instead of wrapping `attributor.resolver()` and spawning the driver.

**Rationale**: Section 8.6 places the socket-table refresh on the control
thread; `refresh(&self)` is exactly what makes that possible through the shared
`Arc`. A failed refresh is ignored (the previously published index stays intact,
FR-030), matching the driver's current behavior. `wants_refresh()` default-false
means the offline/scripted path never refreshes, so offline goldens are
byte-identical. The first loop iteration refreshes because `RefreshSchedule::is_due`
is true before any refresh (last == NEVER), so no separate initial refresh is
needed.

**Alternatives considered**:
- Keep `RefreshDriver` and only change the signature. Rejected: it leaves the
  workaround the issue exists to collapse, and two control threads (the driver
  and the pipeline's) where section 8.6 specifies one.

## D-5: PublishedResolver is retained, not removed

**Decision**: `PublishedResolver` keeps existing; its `refresh` moves to `&self`
(still a no-op) and its module doc is softened to note the read/write split is
now optional rather than required. `SocketTableAttributor::resolver()` stays.
The live path stops using it (it now shares the real attributor), but it is a
valid public read-only view and removing it (and its tests, exports, and the
assemble.rs test at line 756) is a separable cleanup.

**Rationale**: The deviation is already three trait deltas plus a wiring change;
ripping out the split in the same slice widens the blast radius of an
architecture-of-record change without adding capability. Keeping it, correctly
signed, is the lower-risk choice, and it stays pub-exported so it is not dead
code. Recorded as a candidate follow-up cleanup.

## D-6: The narrowed-endpoints message reports the profiled count (FR-007)

**Decision**: `orchestrator.rs` emits "filter narrowed to N endpoint(s)" from the
profiled (filtered) endpoint set, i.e. the stamper's `active_endpoints()`, not
`components.inner_attributor.active_endpoints()` (the unfiltered inner).

**Rationale**: After D-3 the number the operator sees must be the number actually
compiled into the filter, or the message overstates the narrowing. The stamper is
the attributor the pipeline installs the filter from.

## Tier boundary

Every decision is verified at tier 1. The socket-table wiring (D-4's
`live_components` change, `RefreshDriver` removal) is `#[cfg(all(feature =
"socket-table", windows))]` and is not built or run in the default CI job, so the
new tier-1 tests drive refresh and narrowing through test doubles and the
`Pipeline` itself, never the real backend. The live path stays compiled-only and
unexecuted, reported as such.
