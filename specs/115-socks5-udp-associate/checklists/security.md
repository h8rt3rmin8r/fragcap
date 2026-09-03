# Security Requirements Checklist: Scoped SOCKS5 UDP Association

- [x] Capability authentication precedes UDP socket creation.
- [x] Association lifetime is inseparable from its TCP control connection.
- [x] Client IP is bound to the TCP peer and UDP port is immutable after declaration or first learning.
- [x] Spoofed local sources cause no DNS, policy, mapping, or send effect.
- [x] Destination policy is applied after resolution to every candidate.
- [x] Replies require an exact previously contacted peer mapping.
- [x] Fragmentation, malformed frames, oversized frames, and saturation are counted.
- [x] Socket count, datagram memory, peer count, and idle lifetime are finite.
- [x] No capability or UDP payload is emitted into evidence.
- [x] Control EOF, timeout, cancellation, and cleanup release every socket and mapping.
- [x] Reflection and local-hijack tests are mandatory and cannot be weakened.
