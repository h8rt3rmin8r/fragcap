### Decisions

**2026-08-10. Six decisions taken while implementing S12, recorded for promotion
to specification section 29.**

- **The stage binding is written onto the process node rather than held in a
  side-map.** `fragcap-core` gains `ProcessTree::bind_stage`, writing the field
  S11 reserved for exactly this. A side-map would split the node's state across
  two owners and thread the map through everywhere the tree already goes.
- **`descends_from` is evaluated once, on the start event, over current
  bindings.** S11 guarantees causal creation order, so a stage that matches an
  ancestor binds before its descendant is evaluated. No deferred re-evaluation
  queue is introduced for a reordering the event source does not produce.
- **A process matching more than one stage binds the first in declaration
  order.** Section 15.4 already makes an ambiguous image match within a chain an
  error; a total order over declaration position makes the residual case
  deterministic rather than dependent on iteration order.
- **The watching-discard counter is the session's own, not `CaptureStats`.** The
  discard happens upstream of the pipeline whose conservation identity
  `CaptureStats` carries, so a field there would break that identity or sit
  unused until S13 and S14 wire the session in. `WatcherReport` and
  `SourceStats` set the precedent that a component's own accounting is a separate
  value the run assembles.
- **The acquisition timeout and the duration bound are measured from arm.** A
  single clock origin for both, and a session that never acquires still ends: by
  the acquisition timeout when set, or by the duration bound or an operator
  interrupt otherwise.
- **A live service does not keep the all-exited stop condition from firing.**
  Section 10.4 says a service is never awaited, because waiting on something
  already running deadlocks; a platform service that outlives the session must
  not keep it from recognizing that its gameplay processes have all exited.
