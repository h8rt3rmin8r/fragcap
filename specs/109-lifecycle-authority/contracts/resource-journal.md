# Contract: Resource Journal and Recovery

## Stream Shape

The first complete record is `resource-journal.header` with schema version, session id, and plan id. Transition records have monotonic sequence numbers and stable resource ids. Orderly completion adds one trailer. A missing trailer is a readable crash prefix.

## Ordering

`obligation.pending` is synchronized before its external effect starts. Later records state not-applied, applied, cleanup-pending, released, retained, failed, or timed-out.

## Parsing

Readers accept supported complete records in sequence. A torn final line marks an incomplete prefix. An invalid complete record, unsupported version, sequence conflict, path escape, or impossible transition invalidates recovery for the affected journal.

## Recovery

Inspection and planning are read-only. Execution occurs only through an effect adapter after current ownership is verified. Each attempt and result is synchronized to the same journal. Repeated recovery is idempotent.

## Compaction

Compaction is atomic and available only when all obligations are terminal. The compacted representation retains every resource id, kind, ownership proof, recovery action, and final disposition.
