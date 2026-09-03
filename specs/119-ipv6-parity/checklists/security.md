# Security Checklist: Complete IPv6 Parity

**Purpose**: Verify address-family expansion preserves the scoped inspection boundary
**Created**: 2026-09-03

- [x] One session authorizes one exact loopback endpoint
- [x] Wildcard, mapped wildcard, and external listeners are forbidden
- [x] IPv6 selection is operator-visible and has no hidden family fallback
- [x] Scoped addresses use bounded numeric local indexes only
- [x] Zone identifiers do not enter TLS identity or application authority
- [x] Mapped aliases cannot duplicate policy grants, peer ownership, or correlation
- [x] Dual-stack connection attempts have one finite owner, deadline, and winner
- [x] Loser sockets are cancelled before application forwarding
- [x] Doctor probes are ephemeral, read-only, family-specific, and loopback-only
- [x] Existing authentication, destination policy, trust, retention, and cleanup remain authoritative
- [x] No process access, packet interception, global proxy mutation, or target key extraction is introduced
