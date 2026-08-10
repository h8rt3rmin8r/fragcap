### Decisions

**2026-08-10. `FlowAttributor::refresh` changes from `&mut self` to `&self`, an
architecture-of-record trait change taken through the deviation process and
promoted to specification section 29, together with two added trait methods.
Recorded while implementing slice 015 (the S13 follow-ups, issues #18 and #19).**

- **`refresh(&self)` is the deviation.** Specification section 8.5 declared it
  `&mut self`, and section 8.6 places the socket-table refresh on the pipeline
  control thread; the two could not both hold, because a `&mut self` method
  cannot be called through the `Arc<dyn FlowAttributor>` the capture threads
  share for lock-free `resolve` (section 11.6). `refresh` now takes `&self`;
  `SocketTableAttributor` carries its refresh-mutable state (the table source, the
  process namer, and the retention map) behind a single `Mutex` that the resolve
  path never touches, so section 11.6 is preserved. A concurrency test drives
  `refresh` through a shared `Arc` while several threads resolve.
- **Two trait methods were added, both defaulted.** `wants_refresh(&self) ->
  bool` (default `false`) lets the control thread gate the refresh on the section
  11.2 cadence without `fragcap-core` naming the schedule type that lives in
  `fragcap-attr` (a P-2 guard). `active_endpoints_owned(&self) -> Vec<OwnedEndpoint>`
  (default maps `active_endpoints` to an unknown owner) carries the owning
  identifier the section 12.2 narrowing needs. The defaults mean the scripted and
  stub attributors and the read-only resolver change behavior in no way.
- **Narrowing filters in the role-stamping decorator, and keeps unknown-owner
  endpoints.** The session's `RoleStampingAttributor` holds the binding snapshot
  whose keys are the profiled process identifiers, so it is the one seam that can
  perform the join (P-3: the pipeline and `fragcap-attr` learn no profiles). It
  excludes an endpoint only when its owner is known and not profiled; an endpoint
  with no known owner is kept. On the live socket-table backend every endpoint
  carries an owner, so this is exactly "admit only profiled"; on the offline
  scripted substrate no endpoint carries one, so it is a pass-through and the
  offline goldens are byte-identical.
- **The CLI `RefreshDriver` and the read/write split were retired on the live
  path.** With `refresh(&self)` the pipeline shares and refreshes the real
  attributor directly, so the separate refresh thread and the `PublishedResolver`
  it fed are no longer used there. `PublishedResolver` is retained as a valid
  read-only view (removing it entirely is a separable cleanup); the CLI
  `inner_attributor` component field is removed, and the filter-narrowed event now
  reports the profiled endpoint count.
- **Numbering.** This is spec directory `015-attribution-pipeline-integration`, a
  follow-up to S13, and is not the roadmap's reserved slice S15 (streaming sinks).
  `docs/plans/README.md` records that the roadmap slices S15 through S18 take
  directory ordinals 018 through 021.
