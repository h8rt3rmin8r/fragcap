# Data Model: Installed Direct Route Acceptance

## Installed journey receipt

One run records the product build identity, MSI or ZIP source, effective executable path, target-store path, session-registry path, generated bundle path, observed native artifact names, authorization plan identity, cold client process generation, route and proxy terminal state, and cleanup result. A package harness receipt carries a distinct controlled-root marker and cannot be promoted to default-path evidence.

## Selected flow

An exact flow joins a one-use route receipt, process generation and ancestry, original address family and tuple, packet direction and disposition, proxy attempt, client response, two-way payload counts, terminal reason, and loss. Reused tuples have separate generation-bound identities. A helper flow never inherits the client identity.

## Passive packet evidence

The Npcap source reports original selected packet retained, absent after diversion, or history incomplete. It records the source observation window and uncertainty. It does not determine route reachability by itself.

## Debug report

The report is a bounded, source-preserving projection of available native bundle artifacts. It exposes opaque flow labels, source version and completeness, sequence gaps, queue and writer loss, route and proxy stages, packet correlation availability, calibration verdict, and cleanup. The shareable transformation names every omitted private field.

## State transitions

- A route progresses from candidate to exact selection, client response, proxy exchange, and terminal outcome. Refusal and incomplete proof terminate without positive reachability.
- Passive packet evidence is evaluated independently from route state and may remain unavailable after a positive exact route.
- A native bundle progresses through crash prefix, partial or complete finalization, and exact cleanup or recovery-required state. Report export observes a snapshot and never mutates that state.
