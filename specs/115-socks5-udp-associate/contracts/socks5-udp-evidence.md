# Contract: SOCKS5 UDP Evidence And Loss

- `socks.udp.association` records endpoint mode, relay endpoint, and establishment outcome.
- `socks.udp.datagram` records direction, address form, actual remote endpoint when observed, payload length, and forwarded outcome. It never records payload bytes.
- `socks.udp.drop` records one stable reason: malformed, fragmentation-unsupported, client-source-refused, destination-refused, resolution-failed, peer-limit, oversized, unsolicited-peer, timeout, cancellation, or transport-failed.
- Runtime accounting reconciles client ingress and upstream ingress independently with their forwarded and dropped outcomes.
- Peak and terminal peer counts demonstrate bounded mapping ownership and cleanup.
- All events use the existing session and connection identifiers. Packet/process correlation remains a finalization-time connection-window decision.
