# Security Checklist: Generic UDP Observations

**Purpose**: Preserve the scoped relay and sensitive evidence boundary while adding payload retention.

- [x] Evidence exists only after authenticated, target-scoped routing
- [x] No new listener, tenant, route, or destination policy is introduced
- [x] Immutable client endpoint and exact contacted-peer checks remain authoritative
- [x] Refused, malformed, fragmented, spoofed, and unsolicited inputs retain no payload
- [x] Forwarding does not depend on payload capture, queue admission, or storage
- [x] Per-association, session, queue, and localized loss maps are finite
- [x] Payload bytes use the existing protected application artifact
- [x] Packet truth and proxy-observed datagram truth remain distinct
- [x] ICMP and OS error visibility are not overstated
- [x] Cleanup owns all sockets, mappings, and terminal accounting
- [x] No target process handle, memory access, injection, or security bypass is added
- [x] Controlled tests require no Internet, account, game, elevation, or unrelated traffic
