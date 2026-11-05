# Data Model: Native Blind Relay

| Entity | Identity | State and invariant |
| --- | --- | --- |
| Inspection decision | Connection ID or UDP association ID | Frozen scope, action, reason, proof, and decision time. Only verified game enters inspection under exact-game scope. |
| Relay transfer | Connection/association ID and direction | Monotonic observed byte or datagram count, omitted application bytes, activity timing, one terminal outcome. Payload is never retained for excluded owners. |
| Native connection diagnostic | Session and connection ID | Decision, admission, reached/completed stages, terminal cause, relay totals, and bounded failure detail. |
| Route owner | Managed launch process instance | Continues to own inherited proxy settings until exact release; game observation end does not imply release. |

Reconciliation: accepted connections equal live plus terminal connections. A terminal connection is classified once as relayed, inspected, or pre-decision/refused. Relay directional totals count bytes accepted by the destination write or datagrams accepted by the destination socket. Framed HTTP/2 counts DATA payload, not HPACK or frame overhead. Generic observed bytes may be higher if a later write fails. Omission is an application-evidence decision and is not packet loss. Connection and datagram populations remain separate.
