# Data Model: Correlated Native Evidence

## Flow history

One canonical flow id, first and last packet times, packet count, and ordered resolved or unresolved owner/fidelity segments.

## Accepted connection and correlation

A connection carries transport, normalized endpoints, accepted and terminal times. Its final result is matched, flow-only, ambiguous, or unavailable with a stable reason, optional anchors, fidelity, packet count, and accounting contribution. Streams and messages retain distinct ids.

## HTTP transaction

Connection and stream identity, metadata, body segments, response phases, native phase timings, terminal outcome, losses, and correlation. It becomes standard-representable, partial, or failed.

## Artifact declaration and manifest

Each expected role has an optional safe path, authority, content type, sensitivity, finalization, evidence completeness, loss, and correlation capability. The v2 document also carries schema and product versions, session state, target, effects, and cleanup. Session state is derived from artifact truth.

## State transitions

```text
manifest: absent -> crash-prefix -> final published
application: header -> observations -> correlations -> trailer
HAR: absent -> staging -> complete published | failed absent
legacy manifest: read-only -> normalized legacy view
```
