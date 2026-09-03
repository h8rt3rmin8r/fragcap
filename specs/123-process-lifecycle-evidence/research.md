# Research: Complete Process Lifecycle Evidence

## R1: Process instance identity

**Decision**: Identify an observed process lifetime by PID plus its creation event timestamp. Treat query-only snapshot identity as a separate limited authority with no invented creation instant.

**Rationale**: A PID alone is reusable. ETW creation time is already the authority the process tree uses to prevent reuse transfer, while snapshots explicitly carry stale-parent risk.

**Alternatives considered**: PID plus image name still collides on same-image reuse. Opening a process for creation time adds a target handle and is prohibited. Random instance identifiers would hide rather than express the evidence basis.

## R2: Collection and relevance filtering

**Decision**: Retain a bounded raw event window as events are consumed, then emit only launch-root, declared-stage, packet-owner, and ancestry-relevant process instances during final reconciliation. Count every event beyond the cap.

**Rationale**: Relevance can become known after a descendant or socket owner appears. Early filtering would silently omit a parent needed later; writing every system process to the target bundle would violate target scope.

**Alternatives considered**: Unbounded retention fails the finite-ownership requirement. Immediate all-system serialization over-collects unrelated process data. Immediate stage-only filtering loses undeclared ancestry and later socket owners.

## R3: Socket-owner authority

**Decision**: Add a deterministic `FlowRegistry` snapshot and derive owner transitions exclusively from its retained packet observations.

**Rationale**: The registry already owns session-local `flow_id`, packet timestamp, attribution, role, stage, fidelity, and loss. Running the socket table again would produce a second answer at a different time.

**Alternatives considered**: Re-querying IP Helper risks disagreement. Inferring ownership from launch or stage matching confuses process presence with socket use. Reading pcapng back is redundant and loses in-memory loss detail.

## R4: Ordering and late events

**Decision**: Preserve observation sequence in the raw report but serialize reconciled records by event time, stable kind rank, process instance, and flow identifier. Mark inversions and unresolved intervals rather than rewriting timestamps.

**Rationale**: ETW delivery and packet processing may interleave differently between runs. Evidence-time order produces deterministic output while stable tie breaks make the order total.

**Alternatives considered**: Arrival order is schedule-dependent. Sorting only by timestamp leaves equal-time output nondeterministic. Normalizing timestamps would alter observations.

## R5: Writer and crash behavior

**Decision**: Serialize the bounded reconciled report after capture into newline-framed JSON, flushing complete records and writing the trailer last. A missing or invalid trailer is a partial trace and cannot support a complete manifest claim.

**Rationale**: Raw process events are streamed into the bounded report during the session; final relevance and PID-generation reconciliation requires the complete retained interval. The sidecar still preserves a readable prefix on interruption or writer failure.

**Alternatives considered**: Immediate sidecar writes cannot know eventual relevance and would retain unrelated system events. A monolithic JSON document loses its entire readable shape on interruption. A second temporary raw sidecar expands sensitive artifact ownership without user value.

## R6: Watcher loss authority

**Decision**: Sample the existing ETW watcher report before stopping it and retain kernel event loss, kernel buffer loss, parser loss already folded into event loss, rundown ignored, and unexpected watcher termination separately.

**Rationale**: These counters already exist at the platform authority. Collapsing them into one boolean would prevent an operator from locating the incomplete interval.

**Alternatives considered**: Adding watcher counts to packet statistics would corrupt packet conservation. Reporting zero when the watcher is absent would confuse unavailable with lossless.
