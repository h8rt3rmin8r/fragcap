# Research: Generic UDP Observations

## Observation Boundary

**Decision**: Observe only complete application payloads accepted by the S115 association, immediately before each directional forwarding operation.

**Rationale**: S115 already validates framing, ownership, policy, and exact reply sources. This point preserves one datagram boundary without retaining refused content.

**Alternatives considered**: Observing socket buffers includes refused traffic. Observing after send loses evidence on transport failure and can confuse accepted bytes with confirmed remote receipt.

## Retention

**Decision**: Reuse the body per-connection and session retention limits. One event represents the full observed datagram and retains at most its available prefix.

**Rationale**: A shared sensitive-data budget honors the operator's existing limit. Splitting a datagram into event chunks would erase the primary UDP boundary contract.

**Alternatives considered**: A new UDP budget multiplies sensitive storage. Whole-datagram-only retention wastes a remaining partial budget and hides exact omission size.

## Identity And Ordering

**Decision**: Maintain an independent monotonic ingress sequence per direction. Record the pinned client endpoint and actual selected or observed remote endpoint.

**Rationale**: Sequence plus timestamp represents duplicates and reordering without inferring request-response pairs. Exact endpoints preserve S115 ownership truth.

**Alternatives considered**: Payload hashes invite false deduplication. A shared sequence obscures directional reorder. Requested domain alone does not name the selected peer.

## Error Visibility

**Decision**: Record only socket errors returned to the relay, typed by direction and operation. State that their relationship to ICMP is platform-dependent and unknown.

**Rationale**: UDP stacks expose asynchronous network errors inconsistently. Claiming ICMP type, delivery, or absence from a generic I/O error violates the instrument-truth principle.

**Alternatives considered**: Raw ICMP capture creates another privileged socket and duplicates packet authority. Ignoring returned errors loses observable transport failure.

## Artifact And Queue Loss

**Decision**: Add a new application event to JSON Lines version 2 and extend the existing bounded writer loss map with connection, direction, and endpoint identity plus exact overflow totals.

**Rationale**: The application stream already owns payload evidence, correlation, crash readability, and storage failure. The S109 bounded loss-map pattern prevents unbounded cardinality.

**Alternatives considered**: A UDP sidecar duplicates artifact authority. Aggregate-only queue loss cannot reconcile which direction and peer lost evidence.

## Dependencies

**Decision**: Add no dependency.

**Rationale**: Existing bytes, Tokio, serde_json, retention counters, event sink, and endpoint types cover the slice.
