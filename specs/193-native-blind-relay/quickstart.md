# Controlled Verification: Native Blind Relay

1. Run a local TLS origin with an independent pinned client and a second client trusting the test session authority. Give the pinned client middleware ownership and the second client exact game ownership. Both use the same hostname through the authenticated proxy. The first must complete original TLS with no Fragcap key or decrypted export; the second must produce inspected evidence.
2. Exercise HTTP/1 CONNECT and plaintext HTTP, cleartext HTTP/2 requests and CONNECT, SOCKS TCP half-close/cancellation, and SOCKS UDP with QUIC Initial over IPv4 and IPv6. Check authentication, destination denial, exact terminal accounting, and excluded-content absence.
3. Send a progressing transfer longer than the old header deadline, then a genuinely idle tunnel. Check activity timing, finite teardown, and no duration clamp.
4. Saturate optional evidence and fail a writer while forwarding. Verify unchanged client delivery and named evidence loss. Fail game TLS after an `Inspect` decision and confirm there is no relay fallback.
5. End the game while a managed launcher still owns a route. Check separate game observation, route-owner release, proxy drain, and cleanup facts. The controlled fixture uses no private title or traffic.
