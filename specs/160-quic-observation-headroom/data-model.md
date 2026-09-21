# Data Model: S160 QUIC observation headroom

## Application queue authority

`DefaultApplicationQueueCapacity` is one shared finite event count with value
16,384. Ordinary native sessions and the canonical performance harness consume
the same authority. Explicit test callers may still request smaller capacities
to exercise overload.

The queue states remain `open`, `retired`, and `drained`. Readiness is a separate
consumer fact and does not imply future scheduling.

## Writer scheduling boundary

`WriterReady` means the dedicated writer initialized its owned state and made
the lease publishable. `WriterReceiveReleased` is a private test-only boundary
immediately before the receive loop. Production releases it immediately. A
controlled test may hold it after lease publication to model a ready but
descheduled consumer.

State progression is `allocated -> worker-spawned -> ready -> lease-published ->
receive-loop -> draining -> joined`. A controlled stall exists only between
`lease-published` and `receive-loop`.

## Payload accounting

`WriterObservedBytes` is the sum of observed lengths dequeued by the artifact
writer before storage succeeds or fails. `RetainedBytes`, `OmittedBytes`, and
`StorageDroppedBytes` partition that value.

`QueueDroppedBytes` and `StorageDroppedBytes` are mutually exclusive losses that
never entered the writer or did not survive artifact storage, respectively.
Every queue-loss byte authority uses the event's observed length, not its
possibly omitted or truncated retained vector length.
`TotalObservedBytes` is:

```text
WriterObservedBytes + QueueDroppedBytes
```

The complete conservation relation is:

```text
TotalObservedBytes = RetainedBytes + OmittedBytes + QueueDroppedBytes + StorageDroppedBytes
```

All additions and subtractions remain saturating because the source counters are
monotonic bounded integers.

## Performance report versions

`PerformanceReportV1` is the historical read-only campaign format. Its
loss-free records remain valid under their original field set and meaning.

`PerformanceReportV2` is emitted by every new campaign. Every record carries
schema version 2. Case samples require `application_events_attempted` and
`application_queue_capacity`, and `payload_bytes_observed` means total produced
observations across accepted and lost dispositions. Unknown versions are
refused rather than guessed.

The performance budget registry remains registry schema version 1. Its version
is a separate contract and does not change when the campaign report advances.

## Performance sample

The existing performance sample fields retain their names in report version 2.
After S160,
`payload_bytes_observed` carries `TotalObservedBytes`, while retained, omitted,
queue-dropped, and storage-dropped carry their exact dispositions. Loss-free
historical samples are semantically identical because both dropped values are
zero.

`application_events_attempted` is incremented exactly once at the application
sink boundary for every producer admission attempt. It is independent from
accepted and dropped totals because a storage failure can count an already
accepted event again. Synthetic terminal correlation records do not pass
through that boundary.
`application_queue_capacity` is the exact capacity used by that worker. A
canonical sample is eligible to pass only when attempted events are at or below
capacity, proving the complete burst could be admitted without consumer
progress.

## Invariants

- Product and canonical harness default application queue capacities are equal.
- Canonical attempted application events do not exceed that shared capacity.
- Capacity is finite and queue admission never waits for the consumer.
- Exactly capacity events can be pending without loss when the consumer is held.
- The next event is refused and advances its event and applicable byte counters.
- Accepted observed bytes equal retained plus omitted bytes.
- Total observed bytes equal all four mutually exclusive dispositions.
- Queue current is zero after terminal drain.
- Artifact contents, order, classification, forwarding, trust, and cleanup do
  not change because capacity changed.
