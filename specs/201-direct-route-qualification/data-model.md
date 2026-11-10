# Data Model: Direct Route Qualification

## Candidate timing

One pending first packet carries original tuple, receipt monotonic instant, finite deadline, packet bytes and address. Diagnostic checkpoints carry socket-event publication, process-proof availability, decision instant, and actual hold duration. No unrelated payload is interpreted or retained by these diagnostics.

Transition: received -> immediate reinjection, pending, or selected continuation; pending -> exact selected, proven nonselected pass-through, uncertain expiry pass-through, or explicit loss. Every delivered packet gets exactly one terminal disposition.

## Route authority

One session `RoutingPlan` contains child environment and an optional signed direct-socket component. The direct component names the stored target, exact final role, TCP/UDP support, resource bounds, Administrator requirement, and component version. The operator reviews it before effect; the coordinator consumes the plan once.

## Compatibility case and fact

An exact case includes stored target identity, launch topology, inspection scope, address family, selected routing kind/version, native backend/version, product version, and target version where available. A positive route fact names the final-client process generation and original flow joined to proxy and passive packet evidence. Historical environment-only facts remain visible but cannot qualify direct-route admission.

## Route outcome

Per original flow, record reached-and-inspected, reached-but-opaque, refused, bypassed, ambiguous, or unavailable without inventing TLS or protocol success. Packet flow identity is independently owned by passive Capture. Cleanup and Doctor retain separate source-of-truth authority for residual effects.
