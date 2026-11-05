# Contract: Native Relay Disposition

An injected S191 decision is frozen before transport-specific inspection. `Inspect` follows the current native engines. `Uninspected` follows the authenticated, policy-checked relay path and carries the original reason, including unresolved ownership. Every descendant HTTP/2 stream inherits the connection decision. A UDP association freezes its independent decision at first admitted datagram. No later TLS error or owner telemetry changes either decision.

An HTTP/1 CONNECT relay returns success only after the permitted origin is connected. It then copies raw bytes in both directions, propagates half-close, accounts directional totals, and ends on exact closure, cancellation, transport failure, idle, or finite session limit. It never constructs a Fragcap leaf or logs TLS secrets.

Plaintext HTTP framing may transform hop headers and chunking for correct forwarding, but its excluded application request/response/body/derived events are not retained. SOCKS5 TCP uses the same opaque relay without generic chunk export. UDP preserves datagram boundaries, association pinning, contacted-peer reply ownership, and destination policy. An excluded QUIC Initial is forwarded as an original datagram, with no native QUIC endpoint.

Connection diagnostics retain bounded authority, endpoint, action/reason, duration, directional totals, terminal cause, omission, and failure. Packet evidence remains independently governed. Raw packet scope exclusion is not counted as packet loss. Exact managed route-owner release precedes proxy retirement.
