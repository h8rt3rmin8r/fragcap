# Research: S160 QUIC observation headroom

## R-1: Writer readiness is necessary but not sufficient

**Decision**: Treat run 35567343253 as evidence of post-readiness consumer
descheduling and supersede S158's readiness-only diagnosis.

**Rationale**: S158 withholds the sink until the writer signals readiness. The
merged main campaign still reached the exact 4,096-event ceiling in two
`quic-on` windows and lost 616 events in total, while the same PR head had
passed. The producer therefore can outrun or out-schedule a ready consumer. A
startup handshake cannot prove future scheduling.

**Alternatives considered**: Repeat the readiness fix (rejected because the
failed source already contains it); accept a rerun (rejected because it leaves
main nondeterministic); assign platform thread priority (rejected because it is
platform-specific, not a scheduling guarantee, and does not bound ownership).

## R-2: Fourfold finite headroom replaces the disproven fixed capacity

**Decision**: Establish 16,384 events as one shared default application queue
capacity for product sessions and the performance harness.

**Rationale**: The canonical workload's complete artifact remains near 12 MiB,
well below the existing 32 MiB artifact and 256 MiB worker ceilings, while the
4,096-event queue has now saturated after both serialization optimization and
writer-readiness correction. Fourfold headroom absorbs the finite metadata-heavy
burst without making the queue unbounded. Each performance sample additionally
publishes accepted plus dropped event attempts and its queue capacity, then
fails when the full canonical burst exceeds that capacity even if concurrent
draining happened to avoid loss. The hosted campaign remains the memory and
workload authority. Exact loss behavior still begins immediately beyond the new
finite bound.

This explicitly reverses S150 and S158's rejection of queue expansion. That
rejection depended on the claim that avoidable serialization work and then
unpublished consumer startup fully explained saturation. Fresh merged-main
evidence disproves that premise. Preserving the old number would now replicate
known bad logic.

**Alternatives considered**: Keep 4,096 and reduce the workload (rejected
because it would hide a product-path capacity defect); permit a small loss
percentage (rejected because the canonical case is the loss-free baseline and
the hard gate should remain strict); block producers or retry admission
(rejected because artifact pressure must not affect forwarding); use an
unbounded channel (rejected by finite ownership and memory requirements).

## R-3: A post-readiness stall is the deterministic regression

**Decision**: Extend the existing private writer-start test seam with a
post-readiness callback. A controlled test lets lease construction finish,
holds the consumer before its receive loop, admits exactly the declared
capacity, proves zero loss, then proves the next event is refused and counted.

**Rationale**: Sleeps and repeated hosted runs cannot prove the boundary. The
existing private seam already controls writer startup without exposing an
operator switch. Splitting readiness signaling from receive-loop entry models
the observed race directly while production supplies no-op callbacks.

**Alternatives considered**: Stall before readiness (rejected because S158
already covers it and the lease cannot publish); sleep the writer (rejected as
timing-dependent); expose a public fault-control option (rejected because the
seam is test-only).

## R-4: Performance observed bytes must include losses exactly once

**Decision**: Treat artifact trailer observed-byte fields as writer-observed
bytes, including records that reached the writer but did not survive storage.
Compute intentional omission as writer observed minus retained minus
storage-dropped bytes, then compute total observed as writer observed plus
queue-dropped bytes.

**Rationale**: The current helper reads observed bytes after queue admission,
before storage succeeds. Storage-dropped bytes are therefore already present in
that writer-observed value, while queue-dropped bytes are absent. The prior
helper subtracted queue loss from omission even though queue loss never entered
the writer total. Adding only queue loss to total observed and subtracting
storage loss from omission partitions every byte exactly once without altering
artifact data or historical files.

**Alternatives considered**: Remove loss from the equation (rejected because
the report would stop describing total produced observations); count dropped
bytes as omission (rejected because the disposition would be false); change the
application artifact schema (rejected because the necessary authorities already
exist).

Because the correction changes one field's meaning and makes two fields
required, new campaign records advance from performance report schema version 1
to version 2. Version 1 remains readable under its historical contract, and an
unknown version remains an error. Silently redefining version 1 was rejected
because the discriminator would cease to protect retained evidence.

## R-5: First-attempt hosted evidence remains authoritative

**Decision**: Require the initial Windows performance conclusion on the final
pull-request head to pass. Preserve any failure artifact and correct the branch
before another head is evaluated.

**Rationale**: Every recurrence has demonstrated an identical-head green retry.
A retry therefore diagnoses variability but cannot establish determinism.

**Alternatives considered**: Automatic failed-job retry (rejected because it
normalizes the defect); local installed-product execution (rejected by operator
direction and unnecessary for the controlled source and hosted campaign).

## R-6: Historical evidence keeps its original registry identity

**Decision**: Keep the S128 reference and soak digests unchanged and declare
their original registry digest compatible in the current registry. Fresh
campaign reports must carry the current digest.

**Rationale**: The only budget change is a queue ceiling increase from 4,096 to
16,384. Historical peaks remain below both ceilings, but replacing their digest
would falsely claim those measurements used the new capacity. An explicit
compatibility list preserves provenance while allowing the unchanged evidence
to demonstrate the unaffected timing, leak, cleanup, and low-peak properties.

**Alternatives considered**: Rewrite the evidence digest (rejected as false
provenance); discard the historical evidence (rejected because its unaffected
claims remain valid); require a new two-hour soak for a ceiling-only correction
(rejected because fresh short campaigns own current capacity and memory while
the historical soak still owns long-duration stability).
