# Contract: Application queue and performance report version 2

## Version authority

New campaign and worker records MUST carry performance report schema version 2.
Version 2 makes the capacity-proof fields required and gives
`payload_bytes_observed` the total-produced meaning below. Historical version 1
reports remain readable under their original field set. An unknown version MUST
be rejected. The separate performance budget registry remains schema version 1.
Retained reference and soak files keep the registry digest under which they were
measured. The current registry MAY explicitly recognize that digest only when a
reviewed compatibility decision preserves every predicate those files prove;
fresh reports MUST carry the current registry digest.

## Default queue

The Deep Capture facade publishes one default application-observation queue
capacity of 16,384 events. Every ordinary native product session and every
canonical performance worker MUST use it.

Admission remains immediate. At capacity, a producer receives `queue-full`; it
does not wait, retry, or affect traffic forwarding. The rejected event advances
the existing aggregate event loss plus its applicable body, streaming, generic
stream, generic UDP, QUIC stream, or QUIC datagram byte loss counter.

Explicit low-capacity construction remains available for controlled overload
tests. The default change does not alter event identity, serialized record
content, classification, correlation, record order, flush behavior, storage
failure, or terminal cleanup.

## Ready but unscheduled consumer

Writer readiness authorizes lease publication but makes no claim about future
thread scheduling. The private test boundary may hold the consumer after it has
signaled readiness and before it receives the first event.

While held, exactly 16,384 pending events MUST be accepted with queue peak
16,384 and zero loss. The next event MUST return `queue-full` and advance exact
loss counters. Releasing the consumer MUST drain accepted events in order and
leave queue current zero.

## Performance payload projection

For every protocol byte class, the harness first sums observed and retained
bytes seen by the application writer. Writer-observed bytes include storage
loss but exclude queue loss. The harness separately sums both named loss
counters from the trailer.

The harness MUST publish:

```text
payload_bytes_omitted = writer_observed
                      - payload_bytes_retained
                      - payload_bytes_storage_dropped
payload_bytes_observed = writer_observed
                       + payload_bytes_queue_dropped
```

The evaluator MUST continue to require:

```text
payload_bytes_observed = payload_bytes_retained
                       + payload_bytes_omitted
                       + payload_bytes_queue_dropped
                       + payload_bytes_storage_dropped
```

Any queue or storage loss remains a hard short-campaign failure. The corrected
projection explains the failure truthfully; it does not waive it.

## Capacity proof

Every version 2 sample MUST publish `application_events_attempted` and
`application_queue_capacity`. Attempted events advance exactly once at the sink
boundary, independently from accepted and dropped disposition counters. Final
synthetic correlation records do not advance it. The evaluator requires:

```text
application_events_attempted <= application_queue_capacity
```

This predicate is independent from queue peak. A passing low peak proves the
consumer kept pace in that run; the attempted-event predicate proves the whole
canonical burst still fits if the consumer makes no progress after readiness.

## Hosted acceptance

The final pull-request head requires an initial Windows native performance run
with fourteen passing cases, seven samples per case, zero hard invariant
failures, zero application-event loss, zero queue or storage byte loss, bounded
memory and artifact size, complete-burst capacity proof, and clean shutdown. A
rerun is diagnostic only.
